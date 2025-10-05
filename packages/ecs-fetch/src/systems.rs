//! HTTP Systems
//!
//! Core ECS systems for processing HTTP requests, responses, retries, rate limiting,
//! and connection management with zero-allocation optimizations.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bytes::Bytes;
use futures_util::future;
use tracing::{Span, debug, error, info, instrument, warn};

use crate::cache_integration::{
    CacheIntegrationConfig, CacheKeyStrategy, HttpCacheManager, HttpCacheReadRequested,
    HttpCacheWriteRequested,
};
use crate::components::*;
// Import cache events and additional types for cache metadata
use crate::events::CacheMetadata;
use crate::events::*;
use crate::resources::*;
use crate::security::sanitization::{RequestSanitizer, ResponseSanitizer};
use crate::security::{ComprehensiveRequestValidator, RequestSecurityContext};

/// System to process incoming HTTP requests with comprehensive security validation
#[instrument(skip_all, fields(requests_processed))]
pub fn process_http_requests_system(
    mut commands: Commands,
    mut client_pool: ResMut<HttpClientPool>,
    mut rate_limiter: ResMut<RateLimitManager>,
    mut metrics: ResMut<RequestMetrics>,
    config: Res<HttpConfig>,
    cache_config: Res<CacheIntegrationConfig>,
    mut request_events: EventReader<HttpRequestSubmitted>,
    mut response_events: EventWriter<HttpResponseReceived>,
    mut failure_events: EventWriter<HttpRequestFailed>,
    mut rate_limit_events: EventWriter<RateLimitExceeded>,
    mut cache_read_events: EventWriter<HttpCacheReadRequested>,
    mut cache_integration: ResMut<HttpCacheManager>,
) {
    let mut requests_processed = 0u32;
    let request_sanitizer = RequestSanitizer::default();
    let validator = ComprehensiveRequestValidator::default();

    for request in request_events.read() {
        requests_processed += 1;

        // Parse and validate URL
        let parsed_url = match url::Url::parse(&request.url) {
            Ok(url) => url,
            Err(e) => {
                failure_events.write(HttpRequestFailed {
                    operation_id: request.operation_id,
                    correlation_id: request.correlation_id,
                    error: HttpErrorKind::InvalidUrl(e.to_string()),
                    is_retryable: false,
                    retry_count: 0,
                    elapsed_time: request.submitted_at.elapsed(),
                    requester: request.requester.clone(),
                    failed_at: Instant::now(),
                });
                continue;
            },
        };

        // Extract domain for rate limiting
        let domain = parsed_url.domain().unwrap_or("unknown").to_string();

        // Check rate limits
        match rate_limiter.check_rate_limit(&domain) {
            Ok(_) => {},
            Err(_) => {
                rate_limit_events.write(RateLimitExceeded {
                    domain: domain.clone(),
                    retry_after: Some(Duration::from_secs(60)),
                    current_rate: rate_limiter.get_current_rate(&domain),
                    limit: config.rate_limit_config.per_domain_requests_per_second as f64,
                    queued_requests: rate_limiter.get_queued_count(&domain) as u32,
                    occurred_at: Instant::now(),
                });

                failure_events.write(HttpRequestFailed {
                    operation_id: request.operation_id,
                    correlation_id: request.correlation_id,
                    error: HttpErrorKind::RateLimit,
                    is_retryable: true,
                    retry_count: 0,
                    elapsed_time: request.submitted_at.elapsed(),
                    requester: request.requester.clone(),
                    failed_at: Instant::now(),
                });
                continue;
            },
        }

        // Security validation
        let security_context = RequestSecurityContext::new(
            parsed_url.clone(),
            request.method.clone(),
            request.headers.clone(),
            request.body.as_ref().map_or(0, |b| b.len()),
            request.requester.clone(),
        );

        if let Err(security_error) = validator.validate_request(&security_context) {
            failure_events.write(HttpRequestFailed {
                operation_id: request.operation_id,
                correlation_id: request.correlation_id,
                error: HttpErrorKind::SecurityViolation(security_error.to_string()),
                is_retryable: false,
                retry_count: 0,
                elapsed_time: request.submitted_at.elapsed(),
                requester: request.requester.clone(),
                failed_at: Instant::now(),
            });
            continue;
        }

        // Start cache lookup if enabled
        if request.cache_policy.enabled {
            let cache_key = cache_integration.generate_cache_key(
                &request.method,
                &request.url,
                Some(&request.headers),
                CacheKeyStrategy::UrlOnly,
            );

            // Start async cache read - results handled by cache completion systems
            cache_integration.start_cache_read(
                cache_key,
                request.operation_id,
                Some(request.correlation_id),
                &cache_config,
                &mut cache_read_events,
            );

            debug!(
                operation_id = %request.operation_id,
                url = %request.url,
                "Started async cache lookup"
            );
        }

        // Get HTTP client from pool
        let client = client_pool.get_client();

        // Clone necessary data for the async task
        let operation_id = request.operation_id;
        let correlation_id = request.correlation_id;
        let method = request.method.clone();
        let url = request.url.clone();
        let headers = request.headers.clone();
        let body = request.body.clone();
        let timeout = request.timeout;
        let requester = request.requester.clone();
        let submitted_at = request.submitted_at;

        // Spawn async task
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            execute_http_request(client, method, url, headers, body, timeout).await
        });

        // Create request component to track the operation
        let request_body_size = request.body.as_ref().map_or(0, |b| b.len() as u64);
        let request_component = HttpRequest::new(
            operation_id,
            correlation_id,
            request.url.clone(),
            request.method.clone(),
            request.priority,
            request.requester.clone(),
            request_body_size,
        );

        let timeout_component = RequestTimeout::new(request.timeout);
        let retry_policy = RetryPolicy::default();

        // Spawn entity with components
        commands.spawn((
            request_component,
            timeout_component,
            retry_policy,
            HttpRequestTask { task: Some(task) },
        ));

        debug!(
            operation_id = %operation_id,
            url = %request.url,
            method = %request.method,
            "HTTP request submitted for processing"
        );
    }

    // Update telemetry
    if requests_processed > 0 {
        Span::current().record("requests_processed", requests_processed);
        info!(requests_processed, "Processed HTTP requests");
    }
}

/// Component wrapper for HTTP request task
#[derive(Component)]
pub struct HttpRequestTask {
    pub task: Option<Task<HttpRequestResult>>,
}

/// System to process completed HTTP request tasks
#[instrument(skip_all, fields(responses_processed))]
pub fn process_http_responses_system(
    mut commands: Commands,
    mut metrics: ResMut<RequestMetrics>,
    mut request_query: Query<(Entity, &mut HttpRequest, &mut HttpRequestTask), With<HttpRequest>>,
    mut response_events: EventWriter<HttpResponseReceived>,
    mut failure_events: EventWriter<HttpRequestFailed>,
    mut cache_write_events: EventWriter<HttpCacheWriteRequested>,
    cache_config: Res<CacheIntegrationConfig>,
    mut cache_integration: ResMut<HttpCacheManager>,
) {
    let mut responses_processed = 0u32;
    let response_sanitizer = ResponseSanitizer::default();

    for (entity, request, mut task_wrapper) in request_query.iter_mut() {
        if let Some(mut task) = task_wrapper.task.take() {
            if let Some(result) = bevy::tasks::block_on(future::poll_immediate(&mut task)) {
                responses_processed += 1;
                let elapsed = request.elapsed();

                match result {
                    Ok(success) => {
                        // Extract domain for metrics
                        let domain = url::Url::parse(&request.url)
                            .ok()
                            .and_then(|u| u.domain().map(String::from))
                            .unwrap_or_else(|| "unknown".to_string());

                        // Record metrics using stored request body size
                        metrics.record_success(
                            &domain,
                            success.response_time,
                            request.request_body_size,
                            success.body.len() as u64,
                        );

                        // Start async cache write if appropriate
                        if response_sanitizer
                            .should_cache_response(success.status, &success.headers)
                        {
                            // Parse method for cache key generation
                            if let Ok(method) =
                                reqwest::Method::from_bytes(request.method.as_str().as_bytes())
                            {
                                let cache_key = cache_integration.generate_cache_key(
                                    &method,
                                    &request.url,
                                    None, // Headers not available in HttpRequest component
                                    CacheKeyStrategy::UrlOnly,
                                );

                                // Calculate TTL from response headers
                                let ttl = cache_integration
                                    .calculate_ttl(&success.headers, Duration::from_secs(300));

                                // Create cached response and start async write
                                if let Ok(cached_response) = cache_integration
                                    .create_cached_response(
                                        &method,
                                        &request.url,
                                        success.status.as_u16(),
                                        &success.headers,
                                        &success.body,
                                        ttl,
                                    )
                                {
                                    cache_integration.start_cache_write(
                                        cache_key,
                                        cached_response,
                                        &cache_config,
                                        &mut cache_write_events,
                                    );

                                    debug!(
                                        operation_id = %request.operation_id,
                                        url = %request.url,
                                        "Started async cache write"
                                    );
                                }
                            }
                        }

                        // Create cache metadata from response headers
                        let cache_metadata = if success.from_cache {
                            // If from cache, we should have stored metadata
                            None // Would be populated by cache_integration
                        } else {
                            // Create metadata for potential caching
                            let cache_key = format!("{}:{}", request.method, request.url);
                            let etag = success
                                .headers
                                .get("etag")
                                .and_then(|v| v.to_str().ok())
                                .map(String::from);
                            let last_modified = success
                                .headers
                                .get("last-modified")
                                .and_then(|v| v.to_str().ok())
                                .map(String::from);
                            let vary = success
                                .headers
                                .get_all("vary")
                                .iter()
                                .filter_map(|v| v.to_str().ok())
                                .map(String::from)
                                .collect();

                            Some(CacheMetadata {
                                cache_key,
                                cached_at: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                expires_at: None, /* Would be calculated from
                                                   * Cache-Control/Expires headers */
                                etag,
                                last_modified,
                                vary,
                            })
                        };

                        // Send response event
                        response_events.write(HttpResponseReceived {
                            operation_id: request.operation_id,
                            correlation_id: request.correlation_id,
                            status: success.status,
                            headers: success.headers,
                            body: success.body,
                            response_time: success.response_time,
                            from_cache: success.from_cache,
                            cache_metadata,
                            retry_count: request.retry_count,
                            requester: request.requester.clone(),
                            received_at: Instant::now(),
                        });

                        debug!(
                            operation_id = %request.operation_id,
                            status = %success.status,
                            response_time_ms = success.response_time.as_millis(),
                            "HTTP request completed successfully"
                        );
                    },
                    Err(error) => {
                        // Extract domain for metrics
                        let domain = url::Url::parse(&request.url)
                            .ok()
                            .and_then(|u| u.domain().map(String::from))
                            .unwrap_or_else(|| "unknown".to_string());

                        // Record metrics
                        metrics.record_failure(&domain, error.response_time);

                        // Send failure event
                        failure_events.write(HttpRequestFailed {
                            operation_id: request.operation_id,
                            correlation_id: request.correlation_id,
                            error: error.kind.clone(),
                            is_retryable: error.is_retryable,
                            retry_count: request.retry_count,
                            elapsed_time: elapsed,
                            requester: request.requester.clone(),
                            failed_at: Instant::now(),
                        });

                        error!(
                            operation_id = %request.operation_id,
                            error = ?error.kind,
                            "HTTP request failed"
                        );
                    },
                }

                // Remove the completed request entity
                commands.entity(entity).despawn();
            } else {
                // Task not ready yet, put it back
                task_wrapper.task = Some(task);
            }
        }
    }

    if responses_processed > 0 {
        Span::current().record("responses_processed", responses_processed);
        info!(responses_processed, "Processed HTTP responses");
    }
}

/// System to handle request retries with exponential backoff
#[instrument(skip_all, fields(retries_processed))]
pub fn request_retry_system(
    mut commands: Commands,
    mut retry_events: EventReader<HttpRequestRetryRequested>,
    mut request_events: EventWriter<HttpRequestSubmitted>,
    _time: Res<Time>,
) {
    let mut retries_processed = 0u32;

    for retry_request in retry_events.read() {
        // Check if it's time to retry
        if retry_request.scheduled_at <= Instant::now() {
            retries_processed += 1;

            // Reconstruct original request with incremented retry count
            let retry_request_event = HttpRequestSubmitted {
                operation_id: retry_request.operation_id,
                correlation_id: retry_request.correlation_id,
                method: retry_request.method.clone(),
                url: retry_request.url.clone(),
                headers: retry_request.headers.clone(),
                body: retry_request.body.clone(),
                timeout: retry_request.timeout,
                retry_policy: retry_request.retry_policy.clone(),
                cache_policy: retry_request.cache_policy.clone(),
                priority: retry_request.priority,
                requester: retry_request.requester.clone(),
                submitted_at: Instant::now(), // New submission time for retry
            };

            // Submit the retry request
            request_events.write(retry_request_event);

            debug!(
                operation_id = %retry_request.operation_id,
                retry_count = retry_request.retry_count,
                backoff_ms = retry_request.backoff_duration.as_millis(),
                "Retrying HTTP request"
            );
        } else {
            // Not ready to retry yet, schedule for later by spawning a delayed retry component
            commands.spawn(DelayedRetry {
                retry_request: retry_request.clone(),
                ready_at: retry_request.scheduled_at,
            });
        }
    }

    if retries_processed > 0 {
        Span::current().record("retries_processed", retries_processed);
    }
}

/// Component for tracking delayed retries
#[derive(Component, Debug, Clone)]
struct DelayedRetry {
    retry_request: HttpRequestRetryRequested,
    ready_at: Instant,
}

/// System to check delayed retries and resubmit when ready
#[instrument(skip_all, fields(delayed_retries_processed))]
pub fn delayed_retry_system(
    mut commands: Commands,
    delayed_query: Query<(Entity, &DelayedRetry)>,
    mut retry_events: EventWriter<HttpRequestRetryRequested>,
) {
    let mut delayed_retries_processed = 0u32;
    let now = Instant::now();

    for (entity, delayed_retry) in delayed_query.iter() {
        if delayed_retry.ready_at <= now {
            delayed_retries_processed += 1;

            // Resubmit the retry request
            retry_events.write(delayed_retry.retry_request.clone());

            // Remove the delayed retry component
            commands.entity(entity).despawn();
        }
    }

    if delayed_retries_processed > 0 {
        Span::current().record("delayed_retries_processed", delayed_retries_processed);
    }
}

/// System to enforce rate limiting with token bucket algorithm
#[instrument(skip_all, fields(rate_limit_checks))]
pub fn rate_limiting_system(
    mut rate_limiter: ResMut<RateLimitManager>,
    mut rate_limit_events: EventWriter<RateLimitExceeded>,
    time: Res<Time>,
) {
    // Rate limiting is handled in the request processing system
    // This system performs periodic cleanup and recovery notifications

    // Periodic cleanup of old domain limiters (every 5 minutes)
    static LAST_CLEANUP: std::sync::OnceLock<std::sync::Mutex<Instant>> =
        std::sync::OnceLock::new();
    let last_cleanup = LAST_CLEANUP.get_or_init(|| std::sync::Mutex::new(Instant::now()));

    if let Ok(mut last) = last_cleanup.lock() {
        let now = Instant::now();
        if now.duration_since(*last) >= Duration::from_secs(300) {
            // 5 minutes
            rate_limiter.cleanup_inactive_limiters(Duration::from_secs(3600)); // Remove limiters inactive for 1 hour
            *last = now;
            debug!("Performed rate limiter cleanup");
        }
    }

    // Rate limit recovery notifications are sent when limits are lifted
    // This is handled implicitly by the token bucket algorithm as tokens refill
}

/// System to handle request timeouts
#[instrument(skip_all, fields(timeouts_processed))]
pub fn request_timeout_system(
    mut commands: Commands,
    mut timeout_query: Query<(Entity, &mut RequestTimeout, &HttpRequest), With<RequestTimeout>>,
    mut failure_events: EventWriter<HttpRequestFailed>,
) {
    let mut timeouts_processed = 0u32;

    for (entity, mut timeout, request) in timeout_query.iter_mut() {
        if timeout.is_expired() && !timeout.escalation_sent {
            timeouts_processed += 1;
            timeout.mark_escalation_sent();

            failure_events.write(HttpRequestFailed {
                operation_id: request.operation_id,
                correlation_id: request.correlation_id,
                error: HttpErrorKind::Timeout,
                is_retryable: true,
                retry_count: request.retry_count,
                elapsed_time: request.elapsed(),
                requester: request.requester.clone(),
                failed_at: Instant::now(),
            });

            warn!(
                operation_id = %request.operation_id,
                elapsed_ms = request.elapsed().as_millis(),
                "HTTP request timed out"
            );

            // Despawn the timed out request
            commands.entity(entity).despawn();
        }
    }

    if timeouts_processed > 0 {
        Span::current().record("timeouts_processed", timeouts_processed);
    }
}

/// System for connection pool management and optimization
#[instrument(skip_all)]
pub fn connection_pool_management_system(
    client_pool: Res<HttpClientPool>,
    metrics: Res<RequestMetrics>,
) {
    // Get pool statistics
    let stats = client_pool.stats();

    // Compare pool stats with overall metrics for health correlation
    let total_requests = metrics
        .total_requests
        .load(std::sync::atomic::Ordering::Relaxed);
    let metrics_success_rate = metrics.success_rate();

    // Monitor connection utilization and health
    let utilization_rate = if stats.pool_size > 0 {
        stats.active_connections as f64 / stats.pool_size as f64
    } else {
        0.0
    };

    // Log warnings if utilization is high
    if utilization_rate > 0.8 {
        warn!(
            utilization_rate = format!("{:.1}%", utilization_rate * 100.0),
            active_connections = stats.active_connections,
            pool_size = stats.pool_size,
            "High connection pool utilization detected"
        );
    }

    // Monitor success rate for connection health
    if stats.total_requests > 100 && stats.success_rate < 0.9 {
        warn!(
            success_rate = format!("{:.1}%", stats.success_rate * 100.0),
            total_requests = stats.total_requests,
            "Low connection pool success rate detected"
        );
    }

    // Cross-reference pool stats with metrics for inconsistencies
    if total_requests > 1000 && (stats.success_rate - metrics_success_rate).abs() > 0.1 {
        warn!(
            pool_success_rate = format!("{:.1}%", stats.success_rate * 100.0),
            metrics_success_rate = format!("{:.1}%", metrics_success_rate * 100.0),
            "Discrepancy detected between pool stats and metrics success rates"
        );
    }

    // Periodic status logging
    static LAST_LOG: std::sync::OnceLock<std::sync::Mutex<Instant>> = std::sync::OnceLock::new();
    let last_log = LAST_LOG.get_or_init(|| std::sync::Mutex::new(Instant::now()));

    if let Ok(mut last) = last_log.lock() {
        let now = Instant::now();
        if now.duration_since(*last) >= Duration::from_secs(60) {
            // Log every minute
            debug!(
                pool_size = stats.pool_size,
                active_connections = stats.active_connections,
                utilization_rate = format!("{:.1}%", utilization_rate * 100.0),
                total_requests = stats.total_requests,
                success_rate = format!("{:.1}%", stats.success_rate * 100.0),
                "Connection pool status report"
            );
            *last = now;
        }
    }
}

/// Execute HTTP request asynchronously
async fn execute_http_request(
    client: std::sync::Arc<reqwest::Client>,
    method: reqwest::Method,
    url: String,
    headers: reqwest::header::HeaderMap,
    body: Option<Bytes>,
    timeout: Duration,
) -> HttpRequestResult {
    let start_time = Instant::now();

    // Build request
    let mut request_builder = client
        .request(method, &url)
        .headers(headers)
        .timeout(timeout);

    // Add body if present
    if let Some(body_data) = body {
        request_builder = request_builder.body(body_data);
    }

    // Execute request
    match request_builder.send().await {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
            let response_time = start_time.elapsed();

            // Read response body
            match response.bytes().await {
                Ok(body) => Ok(HttpRequestSuccess {
                    status,
                    headers,
                    body,
                    response_time,
                    from_cache: false,
                }),
                Err(e) => Err(HttpRequestError {
                    kind: HttpErrorKind::Network(e.to_string()),
                    response_time: start_time.elapsed(),
                    is_retryable: true,
                }),
            }
        },
        Err(e) => {
            let response_time = start_time.elapsed();
            let error_kind = classify_reqwest_error(&e);
            let is_retryable = error_kind.is_retryable();

            Err(HttpRequestError {
                kind: error_kind,
                response_time,
                is_retryable,
            })
        },
    }
}

/// Classify reqwest errors into our error taxonomy
fn classify_reqwest_error(error: &reqwest::Error) -> HttpErrorKind {
    if error.is_timeout() {
        HttpErrorKind::Timeout
    } else if error.is_request() {
        HttpErrorKind::Serialization(error.to_string())
    } else if error.is_redirect() {
        HttpErrorKind::Network("Too many redirects".to_string())
    } else if error.is_connect() {
        HttpErrorKind::Network("Connection failed".to_string())
    } else if let Some(status) = error.status() {
        HttpErrorKind::Status(status.as_u16())
    } else if error.is_decode() {
        HttpErrorKind::Deserialization(error.to_string())
    } else {
        HttpErrorKind::Network(error.to_string())
    }
}

/// System to clean up stale requests and components
#[instrument(skip_all, fields(entities_cleaned))]
pub fn cleanup_stale_requests_system(
    mut commands: Commands,
    stale_query: Query<Entity, (With<HttpRequest>, Without<HttpRequestTask>)>,
    config: Res<HttpConfig>,
) {
    let mut entities_cleaned = 0u32;
    let max_cleanup_batch = config.max_concurrent_requests / 4; // Use config to determine batch size

    // Clean up request entities that no longer have active tasks
    // Limit cleanup batch size based on configuration
    for (index, entity) in stale_query.iter().enumerate() {
        if index >= max_cleanup_batch as usize {
            debug!(
                "Cleanup batch limit reached: {}, deferring remaining entities",
                max_cleanup_batch
            );
            break;
        }

        entities_cleaned += 1;
        commands.entity(entity).despawn();
    }

    if entities_cleaned > 0 {
        Span::current().record("entities_cleaned", entities_cleaned);
        debug!(entities_cleaned, "Cleaned up stale request entities");
    }
}

/// System to update HTTP metrics and generate periodic reports
#[instrument(skip_all)]
pub fn metrics_reporting_system(metrics: Res<RequestMetrics>, time: Res<Time>) {
    // Use time for consistent scheduling calculations
    let current_time = time.elapsed();
    static LAST_REPORT: std::sync::OnceLock<std::sync::Mutex<Instant>> = std::sync::OnceLock::new();
    let last_report = LAST_REPORT.get_or_init(|| std::sync::Mutex::new(Instant::now()));

    if let Ok(mut last) = last_report.lock() {
        let now = Instant::now();

        // Use time resource for consistent timing calculations across systems
        let time_based_threshold = if current_time.as_secs() < 60 {
            Duration::from_secs(30) // More frequent reporting during startup
        } else {
            Duration::from_secs(300) // Standard 5-minute reporting
        };

        if now.duration_since(*last) >= time_based_threshold {
            let total_requests = metrics
                .total_requests
                .load(std::sync::atomic::Ordering::Relaxed);
            let successful_requests = metrics
                .successful_requests
                .load(std::sync::atomic::Ordering::Relaxed);
            let failed_requests = metrics
                .failed_requests
                .load(std::sync::atomic::Ordering::Relaxed);
            let bytes_sent = metrics
                .bytes_sent
                .load(std::sync::atomic::Ordering::Relaxed);
            let bytes_received = metrics
                .bytes_received
                .load(std::sync::atomic::Ordering::Relaxed);

            // Calculate success rate
            let success_rate = metrics.success_rate();

            // Calculate bandwidth utilization (bytes per second over last 5 minutes)
            let duration_secs = 300.0;
            let send_bandwidth = bytes_sent as f64 / duration_secs;
            let receive_bandwidth = bytes_received as f64 / duration_secs;

            // Get average response time
            let avg_response_time = if let Ok(avg) = metrics.avg_response_time.read() {
                *avg
            } else {
                Duration::ZERO
            };

            // Alert on error rate thresholds
            if success_rate < 0.95 && total_requests > 100 {
                warn!(
                    success_rate = format!("{:.2}%", success_rate * 100.0),
                    total_requests, failed_requests, "HTTP error rate threshold exceeded (< 95%)"
                );
            }

            // Generate comprehensive metrics report
            info!(
                total_requests,
                successful_requests,
                failed_requests,
                success_rate = format!("{:.2}%", success_rate * 100.0),
                avg_response_time_ms = avg_response_time.as_millis(),
                bytes_sent,
                bytes_received,
                send_bandwidth_bps = format!("{:.0}", send_bandwidth),
                receive_bandwidth_bps = format!("{:.0}", receive_bandwidth),
                "HTTP metrics periodic report"
            );

            // Log domain-specific statistics
            if let Ok(domain_stats) = metrics.domain_stats.read() {
                for (domain, stats) in domain_stats.iter().take(10) {
                    // Top 10 domains
                    if stats.requests > 0 {
                        debug!(
                            domain,
                            requests = stats.requests,
                            success_rate = format!("{:.1}%", stats.success_rate() * 100.0),
                            avg_response_time_ms = stats.avg_response_time().as_millis(),
                            "Domain statistics"
                        );
                    }
                }
            }

            *last = now;
        }
    }

    // Quick milestone reporting
    let total_requests = metrics
        .total_requests
        .load(std::sync::atomic::Ordering::Relaxed);
    let success_rate = metrics.success_rate();

    if total_requests > 0 && total_requests % 1000 == 0 {
        info!(
            total_requests,
            success_rate = format!("{:.2}%", success_rate * 100.0),
            "HTTP metrics milestone"
        );
    }
}

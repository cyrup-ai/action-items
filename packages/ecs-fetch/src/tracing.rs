use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{Level, Span, debug, error, info, instrument, span, warn};
use uuid::Uuid;

use crate::events::{CorrelationId, HttpOperationId};

/// HTTP request tracing configuration
#[derive(Debug, Clone, Resource)]
pub struct HttpTracingConfig {
    /// Enable request/response body logging
    pub log_request_body: bool,
    pub log_response_body: bool,
    /// Maximum body size to log (bytes)
    pub max_body_log_size: usize,
    /// Enable headers logging
    pub log_headers: bool,
    /// Headers to redact in logs (for security)
    pub redacted_headers: Vec<String>,
    /// Trace sampling ratio (0.0 to 1.0)
    pub sampling_ratio: f64,
    /// Enable distributed tracing
    pub enable_distributed_tracing: bool,
    /// Enable detailed logging with more verbose output
    pub enable_detailed_logging: bool,
    /// Custom trace fields
    pub custom_fields: HashMap<String, String>,
}

impl Default for HttpTracingConfig {
    fn default() -> Self {
        Self {
            log_request_body: false,
            log_response_body: false,
            max_body_log_size: 1024, // 1KB
            log_headers: true,
            redacted_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
                "bearer".to_string(),
            ],
            sampling_ratio: 1.0, // Trace all requests by default
            enable_distributed_tracing: true,
            enable_detailed_logging: false,
            custom_fields: HashMap::new(),
        }
    }
}

/// HTTP request tracing manager
#[derive(Debug, Resource)]
pub struct HttpTracingManager {
    /// Active request spans
    pub active_spans: HashMap<HttpOperationId, RequestSpan>,
    /// Correlation ID to operation mapping
    pub correlation_mapping: HashMap<CorrelationId, HttpOperationId>,
    /// Tracing statistics
    pub stats: TracingStats,
}

impl Default for HttpTracingManager {
    fn default() -> Self {
        Self {
            active_spans: HashMap::new(),
            correlation_mapping: HashMap::new(),
            stats: TracingStats::default(),
        }
    }
}

impl HttpTracingManager {
    /// Start tracing for HTTP request
    pub fn start_request_trace(
        &mut self,
        operation_id: HttpOperationId,
        correlation_id: CorrelationId,
        method: &Method,
        url: &str,
        headers: &reqwest::header::HeaderMap,
        body_size: Option<usize>,
        config: &HttpTracingConfig,
    ) -> Option<Span> {
        // Check sampling
        if !self.should_sample(config) {
            return None;
        }

        // Create span for HTTP request
        let span = span!(
            Level::INFO,
            "http_request",
            operation_id = %operation_id,
            correlation_id = %correlation_id,
            http.method = %method,
            http.url = %url,
            http.scheme = self.extract_scheme(url),
            http.host = self.extract_host(url),
            http.path = self.extract_path(url),
            http.user_agent = self.extract_header(headers, "user-agent"),
            otel.kind = "client",
            otel.status_code = tracing::field::Empty,
            otel.status_description = tracing::field::Empty,
        );

        // Log request details
        span.in_scope(|| {
            info!("Starting HTTP request: {} {}", method, url);

            // Log headers if enabled
            if config.log_headers {
                self.log_request_headers(headers, config);
            }

            // Log body size
            if let Some(size) = body_size {
                debug!("Request body size: {} bytes", size);
            }
        });

        // Store span for later completion
        self.active_spans.insert(operation_id, RequestSpan {
            span: span.clone(),
            started_at: Instant::now(),
            method: method.clone(),
            url: url.to_string(),
            correlation_id,
        });

        self.correlation_mapping
            .insert(correlation_id, operation_id);
        self.stats.traces_started += 1;

        Some(span)
    }

    /// Complete request trace with success
    pub fn complete_request_trace(
        &mut self,
        operation_id: HttpOperationId,
        status: StatusCode,
        response_headers: Option<&reqwest::header::HeaderMap>,
        response_body_size: Option<usize>,
        duration: Duration,
        config: &HttpTracingConfig,
    ) {
        if let Some(request_span) = self.active_spans.remove(&operation_id) {
            let span = request_span.span;

            span.in_scope(|| {
                // Record span attributes
                span.record("otel.status_code", "OK");
                span.record("http.status_code", status.as_u16());

                info!(
                    "HTTP request completed: {} {} -> {} ({}ms)",
                    request_span.method,
                    request_span.url,
                    status,
                    duration.as_millis()
                );

                // Log response headers if enabled
                if config.log_headers {
                    if let Some(headers) = response_headers {
                        self.log_response_headers(headers, config);
                    }
                }

                // Log response body size
                if let Some(size) = response_body_size {
                    debug!("Response body size: {} bytes", size);
                    span.record("http.response_content_length", size);
                }

                // Record timing
                span.record("http.duration_ms", duration.as_millis() as u64);

                // Record status information
                if status.is_success() {
                    debug!("Request successful");
                } else if status.is_client_error() {
                    warn!("Client error response: {}", status);
                } else if status.is_server_error() {
                    error!("Server error response: {}", status);
                }
            });

            self.correlation_mapping
                .remove(&request_span.correlation_id);
            self.stats.traces_completed += 1;
        }
    }

    /// Complete request trace with error
    pub fn error_request_trace(
        &mut self,
        operation_id: HttpOperationId,
        error: &str,
        duration: Option<Duration>,
    ) {
        if let Some(request_span) = self.active_spans.remove(&operation_id) {
            let span = request_span.span;

            span.in_scope(|| {
                span.record("otel.status_code", "ERROR");
                span.record("otel.status_description", error);

                error!(
                    "HTTP request failed: {} {} -> {}",
                    request_span.method, request_span.url, error
                );

                if let Some(duration) = duration {
                    span.record("http.duration_ms", duration.as_millis() as u64);
                    debug!("Request failed after {}ms", duration.as_millis());
                }
            });

            self.correlation_mapping
                .remove(&request_span.correlation_id);
            self.stats.traces_failed += 1;
        }
    }

    /// Get span for active request
    pub fn get_active_span(&self, operation_id: HttpOperationId) -> Option<&Span> {
        self.active_spans.get(&operation_id).map(|rs| &rs.span)
    }

    /// Get operation ID from correlation ID
    pub fn get_operation_from_correlation(
        &self,
        correlation_id: CorrelationId,
    ) -> Option<HttpOperationId> {
        self.correlation_mapping.get(&correlation_id).copied()
    }

    /// Check if request should be sampled
    fn should_sample(&self, config: &HttpTracingConfig) -> bool {
        if config.sampling_ratio >= 1.0 {
            return true;
        }

        if config.sampling_ratio <= 0.0 {
            return false;
        }

        fastrand::f64() < config.sampling_ratio
    }

    /// Log request headers with redaction
    fn log_request_headers(
        &self,
        headers: &reqwest::header::HeaderMap,
        config: &HttpTracingConfig,
    ) {
        let mut header_map = HashMap::new();

        for (name, value) in headers {
            let name_str = name.as_str().to_lowercase();

            if config.redacted_headers.contains(&name_str) {
                header_map.insert(name_str, "[REDACTED]".to_string());
            } else if let Ok(value_str) = value.to_str() {
                header_map.insert(name_str, value_str.to_string());
            }
        }

        debug!("Request headers: {:?}", header_map);
    }

    /// Log response headers
    fn log_response_headers(
        &self,
        headers: &reqwest::header::HeaderMap,
        _config: &HttpTracingConfig,
    ) {
        let mut header_map = HashMap::new();

        for (name, value) in headers {
            if let Ok(value_str) = value.to_str() {
                header_map.insert(name.as_str().to_lowercase(), value_str.to_string());
            }
        }

        debug!("Response headers: {:?}", header_map);
    }

    /// Extract URL scheme
    fn extract_scheme(&self, url: &str) -> Option<String> {
        url::Url::parse(url).ok().map(|u| u.scheme().to_string())
    }

    /// Extract URL host
    fn extract_host(&self, url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
    }

    /// Extract URL path
    fn extract_path(&self, url: &str) -> Option<String> {
        url::Url::parse(url).ok().map(|u| u.path().to_string())
    }

    /// Extract header value safely
    fn extract_header(&self, headers: &reqwest::header::HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Clean up expired spans
    pub fn cleanup_expired_spans(&mut self, timeout: Duration) {
        let now = Instant::now();
        let mut expired_operations = Vec::new();

        for (operation_id, request_span) in &self.active_spans {
            if now.duration_since(request_span.started_at) > timeout {
                expired_operations.push(*operation_id);
            }
        }

        for operation_id in expired_operations {
            if let Some(request_span) = self.active_spans.remove(&operation_id) {
                warn!(
                    "HTTP request span expired: {} {}",
                    request_span.method, request_span.url
                );
                self.correlation_mapping
                    .remove(&request_span.correlation_id);
                self.stats.traces_expired += 1;
            }
        }
    }

    /// Get tracing statistics
    pub fn get_stats(&self) -> &TracingStats {
        &self.stats
    }
}

/// Active request span information
#[derive(Debug)]
pub struct RequestSpan {
    pub span: Span,
    pub started_at: Instant,
    pub method: Method,
    pub url: String,
    pub correlation_id: CorrelationId,
}

/// Tracing statistics
#[derive(Debug, Default)]
pub struct TracingStats {
    pub traces_started: u64,
    pub traces_completed: u64,
    pub traces_failed: u64,
    pub traces_expired: u64,
}

impl TracingStats {
    /// Get success rate
    #[inline]
    pub fn success_rate(&self) -> f64 {
        let total = self.traces_completed + self.traces_failed;
        if total > 0 {
            self.traces_completed as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get active trace count
    #[inline]
    pub fn active_traces(&self) -> u64 {
        self.traces_started
            .saturating_sub(self.traces_completed + self.traces_failed + self.traces_expired)
    }
}

/// Tracing events
#[derive(Debug, Clone, Event)]
pub struct TraceStarted {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub method: String,
    pub url: String,
    pub started_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct TraceCompleted {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub status_code: u16,
    pub duration: Duration,
}

#[derive(Debug, Clone, Event)]
pub struct TraceFailed {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub error: String,
    pub duration: Option<Duration>,
}

/// HTTP tracing utilities
pub struct HttpTracingUtils;

impl HttpTracingUtils {
    /// Create a child span for HTTP operations
    #[instrument(skip_all, fields(operation = %operation_name))]
    pub fn create_child_span(operation_name: &str, parent_span: &Span) -> Span {
        span!(parent: parent_span, Level::DEBUG, "http_operation", operation = %operation_name)
    }

    /// Extract trace context from headers
    pub fn extract_trace_context(headers: &reqwest::header::HeaderMap) -> Option<TraceContext> {
        // Look for standard distributed tracing headers
        let trace_id = headers
            .get("x-trace-id")
            .or_else(|| headers.get("x-request-id"))
            .or_else(|| headers.get("traceparent"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let span_id = headers
            .get("x-span-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if trace_id.is_some() || span_id.is_some() {
            Some(TraceContext {
                trace_id,
                span_id,
                sampled: true, // Assume sampled if headers are present
            })
        } else {
            None
        }
    }

    /// Inject trace context into headers
    pub fn inject_trace_context(
        headers: &mut reqwest::header::HeaderMap,
        trace_context: &TraceContext,
    ) -> Result<(), String> {
        if let Some(trace_id) = &trace_context.trace_id {
            let header_value = reqwest::header::HeaderValue::from_str(trace_id)
                .map_err(|e| format!("Invalid trace ID header: {}", e))?;
            headers.insert("x-trace-id", header_value);
        }

        if let Some(span_id) = &trace_context.span_id {
            let header_value = reqwest::header::HeaderValue::from_str(span_id)
                .map_err(|e| format!("Invalid span ID header: {}", e))?;
            headers.insert("x-span-id", header_value);
        }

        Ok(())
    }

    /// Generate correlation ID
    pub fn generate_correlation_id() -> CorrelationId {
        Uuid::new_v4()
    }

    /// Format duration for logs
    #[inline]
    pub fn format_duration(duration: Duration) -> String {
        if duration.as_millis() > 0 {
            format!("{}ms", duration.as_millis())
        } else {
            format!("{}μs", duration.as_micros())
        }
    }
}

/// Distributed tracing context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub sampled: bool,
}

/// System to clean up expired traces
pub fn trace_cleanup_system(
    mut tracing_manager: ResMut<HttpTracingManager>,
    config: Res<HttpTracingConfig>,
) {
    // Use config-based cleanup timeout instead of hardcoded value
    let timeout = if config.enable_detailed_logging {
        Duration::from_secs(600) // Keep traces longer when detailed logging is enabled
    } else {
        Duration::from_secs(300) // Standard 5-minute cleanup
    };

    tracing_manager.cleanup_expired_spans(timeout);

    // Log statistics periodically based on config
    let stats = tracing_manager.get_stats();
    let log_interval = if config.enable_detailed_logging {
        500
    } else {
        1000
    };

    if stats.traces_started % log_interval == 0 && stats.traces_started > 0 {
        let log_level = if config.enable_detailed_logging {
            tracing::Level::INFO
        } else {
            tracing::Level::DEBUG
        };

        if log_level == tracing::Level::INFO {
            info!(
                "HTTP tracing stats - Started: {}, Completed: {}, Failed: {}, Active: {}, Success \
                 rate: {:.2}%",
                stats.traces_started,
                stats.traces_completed,
                stats.traces_failed,
                stats.active_traces(),
                stats.success_rate() * 100.0
            );
        } else {
            debug!(
                "HTTP tracing stats - Started: {}, Completed: {}, Failed: {}, Active: {}, Success \
                 rate: {:.2}%",
                stats.traces_started,
                stats.traces_completed,
                stats.traces_failed,
                stats.active_traces(),
                stats.success_rate() * 100.0
            );
        }
    }
}

/// Custom tracing fields for HTTP requests
pub struct HttpTracingFields;

impl HttpTracingFields {
    /// Standard HTTP request fields
    pub const REQUEST_FIELDS: &'static [&'static str] = &[
        "http.method",
        "http.url",
        "http.scheme",
        "http.host",
        "http.path",
        "http.user_agent",
        "http.request_content_length",
    ];

    /// Standard HTTP response fields
    pub const RESPONSE_FIELDS: &'static [&'static str] = &[
        "http.status_code",
        "http.response_content_length",
        "http.duration_ms",
    ];

    /// OpenTelemetry semantic fields
    pub const OTEL_FIELDS: &'static [&'static str] =
        &["otel.kind", "otel.status_code", "otel.status_description"];
}

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::*;

    #[test]
    fn test_tracing_manager_lifecycle() {
        let mut manager = HttpTracingManager::default();
        let config = HttpTracingConfig::default();
        let operation_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let headers = reqwest::header::HeaderMap::new();

        // Start trace
        let span = manager.start_request_trace(
            operation_id,
            correlation_id,
            &Method::GET,
            "https://example.com/api",
            &headers,
            None,
            &config,
        );

        assert!(span.is_some());
        assert_eq!(manager.active_spans.len(), 1);
        assert_eq!(manager.stats.traces_started, 1);

        // Complete trace
        manager.complete_request_trace(
            operation_id,
            StatusCode::OK,
            None,
            None,
            Duration::from_millis(100),
            &config,
        );

        assert_eq!(manager.active_spans.len(), 0);
        assert_eq!(manager.stats.traces_completed, 1);
    }

    #[test]
    fn test_sampling() {
        let manager = HttpTracingManager::default();

        // Always sample
        let config_always = HttpTracingConfig {
            sampling_ratio: 1.0,
            ..Default::default()
        };
        assert!(manager.should_sample(&config_always));

        // Never sample
        let config_never = HttpTracingConfig {
            sampling_ratio: 0.0,
            ..Default::default()
        };
        assert!(!manager.should_sample(&config_never));
    }

    #[test]
    fn test_url_parsing() {
        let manager = HttpTracingManager::default();
        let url = "https://api.example.com:8080/v1/users?limit=10";

        assert_eq!(manager.extract_scheme(url), Some("https".to_string()));
        assert_eq!(
            manager.extract_host(url),
            Some("api.example.com".to_string())
        );
        assert_eq!(manager.extract_path(url), Some("/v1/users".to_string()));
    }

    #[test]
    fn test_trace_context() {
        let mut headers = reqwest::header::HeaderMap::new();
        let trace_context = TraceContext {
            trace_id: Some("abc123".to_string()),
            span_id: Some("def456".to_string()),
            sampled: true,
        };

        HttpTracingUtils::inject_trace_context(&mut headers, &trace_context).unwrap();

        assert!(headers.contains_key("x-trace-id"));
        assert!(headers.contains_key("x-span-id"));

        let extracted = HttpTracingUtils::extract_trace_context(&headers).unwrap();
        assert_eq!(extracted.trace_id, Some("abc123".to_string()));
        assert_eq!(extracted.span_id, Some("def456".to_string()));
    }

    #[test]
    fn test_tracing_stats() {
        let mut stats = TracingStats::default();
        stats.traces_started = 100;
        stats.traces_completed = 80;
        stats.traces_failed = 15;
        stats.traces_expired = 2;

        assert_eq!(stats.success_rate(), 80.0 / 95.0);
        assert_eq!(stats.active_traces(), 3);
    }
}

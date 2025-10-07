use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// HTTP request metrics collection resource
#[derive(Debug, Resource)]
pub struct HttpMetricsCollector {
    /// Request latency histograms by endpoint
    pub latency_histograms: HashMap<String, LatencyHistogram>,
    /// Success/failure counters by status code
    pub status_counters: HashMap<u16, AtomicU64>,
    /// Request counters by method
    pub method_counters: HashMap<String, AtomicU64>,
    /// Bandwidth tracking
    pub bandwidth_metrics: BandwidthMetrics,
    /// Connection pool metrics
    pub pool_metrics: ConnectionPoolMetrics,
    /// Rate limiting metrics
    pub rate_limit_metrics: RateLimitMetrics,
    /// Cache performance metrics
    pub cache_metrics: CachePerformanceMetrics,
    /// Error tracking by category
    pub error_metrics: ErrorMetrics,
    /// Response time percentiles
    pub response_time_percentiles: ResponseTimePercentiles,
    /// Concurrent request tracking
    pub concurrent_requests: AtomicU64,
    /// Maximum concurrent requests reached
    pub max_concurrent_requests: AtomicU64,
}

impl Default for HttpMetricsCollector {
    fn default() -> Self {
        let mut status_counters = HashMap::new();
        let mut method_counters = HashMap::new();

        // Initialize common status codes
        for status in [
            200, 201, 202, 204, 301, 302, 400, 401, 403, 404, 500, 502, 503, 504,
        ] {
            status_counters.insert(status, AtomicU64::new(0));
        }

        // Initialize HTTP methods
        for method in ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"] {
            method_counters.insert(method.to_string(), AtomicU64::new(0));
        }

        Self {
            latency_histograms: HashMap::new(),
            status_counters,
            method_counters,
            bandwidth_metrics: BandwidthMetrics::default(),
            pool_metrics: ConnectionPoolMetrics::default(),
            rate_limit_metrics: RateLimitMetrics::default(),
            cache_metrics: CachePerformanceMetrics::default(),
            error_metrics: ErrorMetrics::default(),
            response_time_percentiles: ResponseTimePercentiles::default(),
            concurrent_requests: AtomicU64::new(0),
            max_concurrent_requests: AtomicU64::new(0),
        }
    }
}

impl HttpMetricsCollector {
    /// Record request start
    #[inline]
    pub fn record_request_start(&mut self, method: &Method, url: &str) {
        // Increment method counter
        let method_str = method.as_str();
        self.method_counters
            .entry(method_str.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        // Increment concurrent requests
        let concurrent = self.concurrent_requests.fetch_add(1, Ordering::Relaxed) + 1;

        // Update max concurrent requests if needed
        let max_concurrent = self.max_concurrent_requests.load(Ordering::Relaxed);
        if concurrent > max_concurrent {
            let _ = self.max_concurrent_requests.compare_exchange_weak(
                max_concurrent,
                concurrent,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }

        // Get or create latency histogram for endpoint
        let endpoint = self.normalize_endpoint(url);
        self.latency_histograms
            .entry(endpoint)
            .or_default();

        debug!("Request started: {} {}", method, url);
    }

    /// Record request completion
    #[inline]
    pub fn record_request_completion(
        &mut self,
        method: &Method,
        url: &str,
        status: StatusCode,
        duration: Duration,
        request_bytes: Option<u64>,
        response_bytes: Option<u64>,
    ) {
        // Record status code
        self.status_counters
            .entry(status.as_u16())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        // Record latency
        let endpoint = self.normalize_endpoint(url);
        if let Some(histogram) = self.latency_histograms.get_mut(&endpoint) {
            histogram.record(duration);
        }

        // Update response time percentiles
        self.response_time_percentiles.add_sample(duration);

        // Record bandwidth
        if let Some(req_bytes) = request_bytes {
            self.bandwidth_metrics
                .bytes_sent
                .fetch_add(req_bytes, Ordering::Relaxed);
        }
        if let Some(resp_bytes) = response_bytes {
            self.bandwidth_metrics
                .bytes_received
                .fetch_add(resp_bytes, Ordering::Relaxed);
        }

        // Decrement concurrent requests
        self.concurrent_requests.fetch_sub(1, Ordering::Relaxed);

        // Update success/error counts based on status code
        if status.is_success() {
            self.bandwidth_metrics
                .successful_requests
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.bandwidth_metrics
                .failed_requests
                .fetch_add(1, Ordering::Relaxed);
        }

        debug!(
            "Request completed: {} {} -> {} ({}ms)",
            method,
            url,
            status,
            duration.as_millis()
        );
    }

    /// Record request error
    #[inline]
    pub fn record_request_error(
        &mut self,
        method: &Method,
        url: &str,
        error_type: ErrorType,
        duration: Option<Duration>,
    ) {
        // Increment error counter
        self.error_metrics.increment_error(error_type);

        // Record latency if available
        if let Some(duration) = duration {
            let endpoint = self.normalize_endpoint(url);
            if let Some(histogram) = self.latency_histograms.get_mut(&endpoint) {
                histogram.record(duration);
            }
            self.response_time_percentiles.add_sample(duration);
        }

        // Decrement concurrent requests
        self.concurrent_requests.fetch_sub(1, Ordering::Relaxed);

        // Update failed request count
        self.bandwidth_metrics
            .failed_requests
            .fetch_add(1, Ordering::Relaxed);

        error!("Request error: {} {} -> {:?}", method, url, error_type);
    }

    /// Record cache hit/miss
    #[inline]
    pub fn record_cache_hit(&mut self) {
        self.cache_metrics.hits.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_cache_miss(&mut self) {
        self.cache_metrics.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record rate limit hit
    #[inline]
    pub fn record_rate_limit_hit(&mut self, domain: &str) {
        self.rate_limit_metrics.record_limit_hit(domain);
    }

    /// Update connection pool metrics
    #[inline]
    pub fn update_pool_metrics(
        &mut self,
        active_connections: u64,
        idle_connections: u64,
        total_connections: u64,
    ) {
        self.pool_metrics.active_connections = active_connections;
        self.pool_metrics.idle_connections = idle_connections;
        self.pool_metrics.total_connections = total_connections;
    }

    /// Normalize endpoint for metrics grouping
    fn normalize_endpoint(&self, url: &str) -> String {
        // Extract path from URL and normalize it
        if let Ok(parsed_url) = url::Url::parse(url) {
            let path = parsed_url.path();
            // Simple path normalization - replace IDs with placeholders
            self.normalize_path(path)
        } else {
            "invalid_url".to_string()
        }
    }

    /// Normalize path by replacing IDs with placeholders
    fn normalize_path(&self, path: &str) -> String {
        let segments: Vec<&str> = path.split('/').collect();
        let mut normalized_segments = Vec::new();

        for segment in segments {
            if segment.is_empty() {
                continue;
            }

            // Check if segment looks like an ID (UUID, number, etc.)
            if self.looks_like_id(segment) {
                normalized_segments.push("{id}");
            } else {
                normalized_segments.push(segment);
            }
        }

        format!("/{}", normalized_segments.join("/"))
    }

    /// Check if segment looks like an ID
    fn looks_like_id(&self, segment: &str) -> bool {
        // UUID pattern
        if segment.len() == 36 && segment.chars().filter(|c| *c == '-').count() == 4 {
            return true;
        }

        // Pure number
        if segment.parse::<u64>().is_ok() {
            return true;
        }

        // MongoDB ObjectId pattern
        if segment.len() == 24 && segment.chars().all(|c| c.is_ascii_hexdigit()) {
            return true;
        }

        false
    }

    /// Get current metrics snapshot
    pub fn get_metrics_snapshot(&self) -> HttpMetricsSnapshot {
        let mut status_counts = HashMap::new();
        for (status, counter) in &self.status_counters {
            status_counts.insert(*status, counter.load(Ordering::Relaxed));
        }

        let mut method_counts = HashMap::new();
        for (method, counter) in &self.method_counters {
            method_counts.insert(method.clone(), counter.load(Ordering::Relaxed));
        }

        HttpMetricsSnapshot {
            status_counts,
            method_counts,
            latency_histograms: self.latency_histograms.clone(),
            bandwidth_metrics: BandwidthMetricsSnapshot {
                bytes_sent: self.bandwidth_metrics.bytes_sent.load(Ordering::Relaxed),
                bytes_received: self
                    .bandwidth_metrics
                    .bytes_received
                    .load(Ordering::Relaxed),
                successful_requests: self
                    .bandwidth_metrics
                    .successful_requests
                    .load(Ordering::Relaxed),
                failed_requests: self
                    .bandwidth_metrics
                    .failed_requests
                    .load(Ordering::Relaxed),
            },
            pool_metrics: ConnectionPoolMetricsSnapshot {
                active_connections: self.pool_metrics.active_connections,
                idle_connections: self.pool_metrics.idle_connections,
                total_connections: self.pool_metrics.total_connections,
                connection_timeouts: self
                    .pool_metrics
                    .connection_timeouts
                    .load(Ordering::Relaxed),
            },
            rate_limit_metrics: RateLimitMetricsSnapshot {
                limit_hits_by_domain: self
                    .rate_limit_metrics
                    .limit_hits_by_domain
                    .iter()
                    .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
                    .collect(),
                total_limit_hits: self
                    .rate_limit_metrics
                    .total_limit_hits
                    .load(Ordering::Relaxed),
            },
            cache_metrics: CachePerformanceMetricsSnapshot {
                hits: self.cache_metrics.hits.load(Ordering::Relaxed),
                misses: self.cache_metrics.misses.load(Ordering::Relaxed),
            },
            error_metrics: ErrorMetricsSnapshot {
                connection_errors: self.error_metrics.connection_errors.load(Ordering::Relaxed),
                timeout_errors: self.error_metrics.timeout_errors.load(Ordering::Relaxed),
                dns_errors: self.error_metrics.dns_errors.load(Ordering::Relaxed),
                ssl_errors: self.error_metrics.ssl_errors.load(Ordering::Relaxed),
                http_errors: self.error_metrics.http_errors.load(Ordering::Relaxed),
                rate_limit_errors: self.error_metrics.rate_limit_errors.load(Ordering::Relaxed),
                unknown_errors: self.error_metrics.unknown_errors.load(Ordering::Relaxed),
            },
            response_time_percentiles: self.response_time_percentiles.clone(),
            concurrent_requests: self.concurrent_requests.load(Ordering::Relaxed),
            max_concurrent_requests: self.max_concurrent_requests.load(Ordering::Relaxed),
        }
    }
}

/// Latency histogram for tracking response times
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyHistogram {
    /// Histogram buckets (milliseconds)
    pub buckets: HashMap<u64, u64>,
    /// Total count
    pub count: u64,
    /// Sum of all values in milliseconds (for average calculation)
    pub sum: u64,
    /// Minimum value in milliseconds
    pub min: Option<u64>,
    /// Maximum value in milliseconds
    pub max: Option<u64>,
}

impl Default for LatencyHistogram {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyHistogram {
    pub fn new() -> Self {
        let mut buckets = HashMap::new();

        // Initialize standard histogram buckets (in milliseconds)
        for bucket in [1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000] {
            buckets.insert(bucket, 0);
        }

        Self {
            buckets,
            count: 0,
            sum: 0,
            min: None,
            max: None,
        }
    }

    #[inline]
    pub fn record(&mut self, duration: Duration) {
        let duration_ms = duration.as_millis() as u64;

        // Find appropriate bucket
        let bucket = if duration_ms <= 1 {
            1
        } else if duration_ms <= 5 {
            5
        } else if duration_ms <= 10 {
            10
        } else if duration_ms <= 25 {
            25
        } else if duration_ms <= 50 {
            50
        } else if duration_ms <= 100 {
            100
        } else if duration_ms <= 250 {
            250
        } else if duration_ms <= 500 {
            500
        } else if duration_ms <= 1000 {
            1000
        } else if duration_ms <= 2500 {
            2500
        } else if duration_ms <= 5000 {
            5000
        } else {
            10000
        };

        *self.buckets.entry(bucket).or_insert(0) += 1;

        self.count += 1;
        self.sum += duration_ms;

        // Update min/max
        if self.min.is_none() || Some(duration_ms) < self.min {
            self.min = Some(duration_ms);
        }
        if self.max.is_none() || Some(duration_ms) > self.max {
            self.max = Some(duration_ms);
        }
    }

    /// Calculate average duration
    #[inline]
    pub fn average(&self) -> Duration {
        if self.count > 0 {
            Duration::from_millis(self.sum / self.count)
        } else {
            Duration::ZERO
        }
    }

    /// Get percentile value (approximate)
    pub fn percentile(&self, percentile: f64) -> Duration {
        if self.count == 0 {
            return Duration::ZERO;
        }

        let target_count = (self.count as f64 * percentile / 100.0) as u64;
        let mut cumulative_count = 0;

        let mut sorted_buckets: Vec<_> = self.buckets.iter().collect();
        sorted_buckets.sort_by_key(|(bucket, _)| *bucket);

        for (bucket_ms, count) in sorted_buckets {
            cumulative_count += count;
            if cumulative_count >= target_count {
                return Duration::from_millis(*bucket_ms);
            }
        }

        Duration::from_millis(10000) // Max bucket
    }
}

/// Bandwidth metrics tracking
#[derive(Debug)]
pub struct BandwidthMetrics {
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
}

impl Default for BandwidthMetrics {
    fn default() -> Self {
        Self {
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
        }
    }
}

impl BandwidthMetrics {
    /// Get total bytes transferred
    #[inline]
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed) + self.bytes_received.load(Ordering::Relaxed)
    }

    /// Get total requests
    #[inline]
    pub fn total_requests(&self) -> u64 {
        self.successful_requests.load(Ordering::Relaxed)
            + self.failed_requests.load(Ordering::Relaxed)
    }

    /// Calculate success rate
    #[inline]
    pub fn success_rate(&self) -> f64 {
        let total = self.total_requests();
        if total > 0 {
            self.successful_requests.load(Ordering::Relaxed) as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// Connection pool metrics
#[derive(Debug)]
pub struct ConnectionPoolMetrics {
    pub active_connections: u64,
    pub idle_connections: u64,
    pub total_connections: u64,
    pub connection_timeouts: AtomicU64,
    pub connection_errors: AtomicU64,
}

impl Default for ConnectionPoolMetrics {
    fn default() -> Self {
        Self {
            active_connections: 0,
            idle_connections: 0,
            total_connections: 0,
            connection_timeouts: AtomicU64::new(0),
            connection_errors: AtomicU64::new(0),
        }
    }
}

/// Rate limiting metrics
#[derive(Debug)]
pub struct RateLimitMetrics {
    /// Rate limit hits per domain
    pub limit_hits_by_domain: HashMap<String, AtomicU64>,
    /// Total rate limit hits
    pub total_limit_hits: AtomicU64,
}

impl Default for RateLimitMetrics {
    fn default() -> Self {
        Self {
            limit_hits_by_domain: HashMap::new(),
            total_limit_hits: AtomicU64::new(0),
        }
    }
}

impl RateLimitMetrics {
    #[inline]
    pub fn record_limit_hit(&mut self, domain: &str) {
        self.limit_hits_by_domain
            .entry(domain.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        self.total_limit_hits.fetch_add(1, Ordering::Relaxed);
    }
}

/// Cache performance metrics
#[derive(Debug)]
pub struct CachePerformanceMetrics {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
}

impl Default for CachePerformanceMetrics {
    fn default() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }
}

impl CachePerformanceMetrics {
    /// Calculate cache hit ratio
    #[inline]
    pub fn hit_ratio(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;

        if total > 0.0 { hits / total } else { 0.0 }
    }
}

/// Error tracking metrics
#[derive(Debug)]
pub struct ErrorMetrics {
    pub connection_errors: AtomicU64,
    pub timeout_errors: AtomicU64,
    pub dns_errors: AtomicU64,
    pub ssl_errors: AtomicU64,
    pub http_errors: AtomicU64,
    pub rate_limit_errors: AtomicU64,
    pub unknown_errors: AtomicU64,
}

impl Default for ErrorMetrics {
    fn default() -> Self {
        Self {
            connection_errors: AtomicU64::new(0),
            timeout_errors: AtomicU64::new(0),
            dns_errors: AtomicU64::new(0),
            ssl_errors: AtomicU64::new(0),
            http_errors: AtomicU64::new(0),
            rate_limit_errors: AtomicU64::new(0),
            unknown_errors: AtomicU64::new(0),
        }
    }
}

impl ErrorMetrics {
    #[inline]
    pub fn increment_error(&mut self, error_type: ErrorType) {
        match error_type {
            ErrorType::Connection => self.connection_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::Timeout => self.timeout_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::Dns => self.dns_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::Ssl => self.ssl_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::Http => self.http_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::RateLimit => self.rate_limit_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::Unknown => self.unknown_errors.fetch_add(1, Ordering::Relaxed),
        };
    }

    /// Get total error count
    #[inline]
    pub fn total_errors(&self) -> u64 {
        self.connection_errors.load(Ordering::Relaxed)
            + self.timeout_errors.load(Ordering::Relaxed)
            + self.dns_errors.load(Ordering::Relaxed)
            + self.ssl_errors.load(Ordering::Relaxed)
            + self.http_errors.load(Ordering::Relaxed)
            + self.rate_limit_errors.load(Ordering::Relaxed)
            + self.unknown_errors.load(Ordering::Relaxed)
    }
}

/// Response time percentiles tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimePercentiles {
    /// Sorted samples for percentile calculation (in milliseconds)
    samples: Vec<u64>,
    /// Maximum number of samples to keep
    max_samples: usize,
}

impl Default for ResponseTimePercentiles {
    fn default() -> Self {
        Self {
            samples: Vec::new(),
            max_samples: 1000, // Keep last 1000 samples
        }
    }
}

impl ResponseTimePercentiles {
    #[inline]
    pub fn add_sample(&mut self, duration: Duration) {
        self.samples.push(duration.as_millis() as u64);

        // Keep only the most recent samples
        if self.samples.len() > self.max_samples {
            self.samples.remove(0);
        }
    }

    /// Calculate percentile
    pub fn percentile(&mut self, percentile: f64) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }

        // Sort samples for percentile calculation
        self.samples.sort();

        let index = ((self.samples.len() - 1) as f64 * percentile / 100.0) as usize;
        Duration::from_millis(self.samples[index])
    }
}

/// Error type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    Connection,
    Timeout,
    Dns,
    Ssl,
    Http,
    RateLimit,
    Unknown,
}

/// Snapshot versions of metrics structs (serializable, cloneable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthMetricsSnapshot {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
}

impl BandwidthMetricsSnapshot {
    pub fn total_requests(&self) -> u64 {
        self.successful_requests + self.failed_requests
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_requests();
        if total > 0 {
            self.successful_requests as f64 / total as f64
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolMetricsSnapshot {
    pub active_connections: u64,
    pub idle_connections: u64,
    pub total_connections: u64,
    pub connection_timeouts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitMetricsSnapshot {
    pub limit_hits_by_domain: HashMap<String, u64>,
    pub total_limit_hits: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceMetricsSnapshot {
    pub hits: u64,
    pub misses: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetricsSnapshot {
    pub connection_errors: u64,
    pub timeout_errors: u64,
    pub dns_errors: u64,
    pub ssl_errors: u64,
    pub http_errors: u64,
    pub rate_limit_errors: u64,
    pub unknown_errors: u64,
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpMetricsSnapshot {
    pub status_counts: HashMap<u16, u64>,
    pub method_counts: HashMap<String, u64>,
    pub latency_histograms: HashMap<String, LatencyHistogram>,
    pub bandwidth_metrics: BandwidthMetricsSnapshot,
    pub pool_metrics: ConnectionPoolMetricsSnapshot,
    pub rate_limit_metrics: RateLimitMetricsSnapshot,
    pub cache_metrics: CachePerformanceMetricsSnapshot,
    pub error_metrics: ErrorMetricsSnapshot,
    pub response_time_percentiles: ResponseTimePercentiles,
    pub concurrent_requests: u64,
    pub max_concurrent_requests: u64,
}

/// Metrics reporting events
#[derive(Debug, Clone, Event)]
pub struct MetricsReportRequested {
    pub requester: String,
    pub include_histograms: bool,
}

#[derive(Debug, Clone, Event)]
pub struct MetricsReportGenerated {
    pub snapshot: HttpMetricsSnapshot,
    pub generated_at: Instant,
}

/// System to periodically report metrics
pub fn metrics_reporting_system(
    metrics_collector: Res<HttpMetricsCollector>,
    mut report_events: EventReader<MetricsReportRequested>,
    mut generated_events: EventWriter<MetricsReportGenerated>,
    _time: Res<Time>,
) {
    for _report_request in report_events.read() {
        let snapshot = metrics_collector.get_metrics_snapshot();

        info!(
            "HTTP metrics report - Total requests: {}, Success rate: {:.2}%, Avg latency: {:?}",
            snapshot.bandwidth_metrics.total_requests(),
            snapshot.bandwidth_metrics.success_rate() * 100.0,
            snapshot
                .latency_histograms
                .values()
                .next()
                .map(|h| h.average())
                .unwrap_or(Duration::ZERO)
        );

        generated_events.write(MetricsReportGenerated {
            snapshot,
            generated_at: Instant::now(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_histogram() {
        let mut histogram = LatencyHistogram::new();

        histogram.record(Duration::from_millis(50));
        histogram.record(Duration::from_millis(150));
        histogram.record(Duration::from_millis(500));

        assert_eq!(histogram.count, 3);
        assert!(histogram.average() > Duration::ZERO);
        assert_eq!(histogram.buckets[&50], 1);
        assert_eq!(histogram.buckets[&250], 1);
        assert_eq!(histogram.buckets[&500], 1);
    }

    #[test]
    fn test_bandwidth_metrics() {
        let metrics = BandwidthMetrics::default();
        metrics.bytes_sent.store(1000, Ordering::Relaxed);
        metrics.bytes_received.store(2000, Ordering::Relaxed);
        metrics.successful_requests.store(80, Ordering::Relaxed);
        metrics.failed_requests.store(20, Ordering::Relaxed);

        assert_eq!(metrics.total_bytes(), 3000);
        assert_eq!(metrics.total_requests(), 100);
        assert_eq!(metrics.success_rate(), 0.8);
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = CachePerformanceMetrics::default();
        metrics.hits.store(80, Ordering::Relaxed);
        metrics.misses.store(20, Ordering::Relaxed);

        assert_eq!(metrics.hit_ratio(), 0.8);
    }

    #[test]
    fn test_endpoint_normalization() {
        let collector = HttpMetricsCollector::default();

        let normalized =
            collector.normalize_endpoint("https://api.example.com/users/12345/posts/67890");
        assert_eq!(normalized, "/users/{id}/posts/{id}");

        let normalized = collector.normalize_endpoint(
            "https://api.example.com/users/550e8400-e29b-41d4-a716-446655440000",
        );
        assert_eq!(normalized, "/users/{id}");
    }

    #[test]
    fn test_error_metrics() {
        let mut metrics = ErrorMetrics::default();

        metrics.increment_error(ErrorType::Connection);
        metrics.increment_error(ErrorType::Timeout);
        metrics.increment_error(ErrorType::Connection);

        assert_eq!(metrics.connection_errors.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.timeout_errors.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_errors(), 3);
    }
}

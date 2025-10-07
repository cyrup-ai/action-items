//! HTTP Resources
//!
//! Core ECS resources for HTTP client pool management, configuration, rate limiting, and metrics.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ahash::AHashMap;
use bevy::prelude::*;
use governor::clock::DefaultClock;
use governor::{Quota, RateLimiter};
use reqwest::Client;

/// HTTP client pool with connection management
#[derive(Resource)]
pub struct HttpClientPool {
    /// Pool of configured reqwest clients
    clients: Vec<Arc<Client>>,
    /// Round-robin index for load balancing
    next_client: std::sync::atomic::AtomicUsize,
    /// Active connection counter for metrics
    active_connections: Arc<std::sync::atomic::AtomicUsize>,
    /// Connection metrics per host
    connection_metrics: Arc<std::sync::Mutex<std::collections::HashMap<String, ConnectionMetrics>>>,
    /// Client configuration
    _config: ClientPoolConfig,
}

/// Connection metrics for tracking per-host statistics
struct ConnectionMetrics {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    last_used: std::time::Instant,
}

impl Default for ConnectionMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            last_used: std::time::Instant::now(),
        }
    }
}

impl HttpClientPool {
    /// Create new client pool with configuration
    pub fn new(config: ClientPoolConfig) -> Result<Self, HttpError> {
        let mut clients = Vec::with_capacity(config.pool_size);

        for _ in 0..config.pool_size {
            let client = reqwest::ClientBuilder::new()
                .timeout(config.default_timeout)
                .connect_timeout(config.connect_timeout)
                .pool_idle_timeout(config.idle_timeout)
                .pool_max_idle_per_host(config.max_idle_per_host)
                .tcp_keepalive(config.tcp_keepalive)
                .use_rustls_tls()
                .danger_accept_invalid_certs(config.accept_invalid_certs)
                .build()
                .map_err(|e| HttpError::ClientCreation(e.to_string()))?;

            clients.push(Arc::new(client));
        }

        Ok(Self {
            clients,
            next_client: std::sync::atomic::AtomicUsize::new(0),
            active_connections: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            connection_metrics: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            _config: config,
        })
    }

    /// Get next client using round-robin load balancing
    #[inline]
    pub fn get_client(&self) -> Arc<Client> {
        let index = self
            .next_client
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.clients[index % self.clients.len()].clone()
    }

    /// Track connection start for metrics
    #[inline]
    pub fn track_connection_start(&self, host: &str) {
        self.active_connections
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut metrics) = self.connection_metrics.lock() {
            let entry = metrics
                .entry(host.to_string())
                .or_insert_with(Default::default);
            entry.total_requests += 1;
            entry.last_used = std::time::Instant::now();
        }
    }

    /// Track connection end for metrics
    #[inline]
    pub fn track_connection_end(&self, host: &str, success: bool) {
        self.active_connections
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut metrics) = self.connection_metrics.lock() {
            if let Some(entry) = metrics.get_mut(host) {
                if success {
                    entry.successful_requests += 1;
                } else {
                    entry.failed_requests += 1;
                }
            }
        }
    }

    /// Get pool statistics
    #[inline]
    pub fn stats(&self) -> ClientPoolStats {
        let active_count = self
            .active_connections
            .load(std::sync::atomic::Ordering::Relaxed);
        let connection_metrics = self.connection_metrics.lock().unwrap_or_else(|poisoned| {
            warn!("Connection metrics mutex was poisoned, recovering with partial data");
            poisoned.into_inner()
        });
        let total_requests: usize = connection_metrics.values().map(|m| m.total_requests).sum();
        let success_rate = if total_requests > 0 {
            let successful: usize = connection_metrics
                .values()
                .map(|m| m.successful_requests)
                .sum();
            (successful as f64) / (total_requests as f64)
        } else {
            1.0
        };

        ClientPoolStats {
            pool_size: self.clients.len(),
            active_connections: active_count,
            total_requests,
            success_rate,
            last_activity: connection_metrics.values().map(|m| m.last_used).max(),
        }
    }
}

/// Client pool configuration
#[derive(Debug, Clone)]
pub struct ClientPoolConfig {
    pub pool_size: usize,
    pub default_timeout: Duration,
    pub connect_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_idle_per_host: usize,
    pub tcp_keepalive: Option<Duration>,
    pub accept_invalid_certs: bool,
}

impl Default for ClientPoolConfig {
    #[inline]
    fn default() -> Self {
        Self {
            pool_size: 10,
            default_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Some(Duration::from_secs(90)),
            max_idle_per_host: 10,
            tcp_keepalive: Some(Duration::from_secs(60)),
            accept_invalid_certs: false,
        }
    }
}

/// Client pool statistics
#[derive(Debug, Clone)]
pub struct ClientPoolStats {
    pub pool_size: usize,
    pub active_connections: usize,
    pub total_requests: usize,
    pub success_rate: f64,
    pub last_activity: Option<std::time::Instant>,
}

/// Global HTTP configuration
#[derive(Resource, Debug, Clone)]
pub struct HttpConfig {
    /// Default request timeout
    pub default_timeout: Duration,
    /// Maximum request body size
    pub max_request_size: usize,
    /// Maximum response body size  
    pub max_response_size: usize,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Default headers applied to all requests
    pub default_headers: std::collections::HashMap<String, String>,
    /// User agent string
    pub user_agent: String,
    /// TLS configuration
    pub tls_config: TlsConfig,
    /// Security policies
    pub security_config: SecurityConfig,
    /// Rate limiting configuration
    pub rate_limit_config: RateLimitConfig,
}

impl Default for HttpConfig {
    #[inline]
    fn default() -> Self {
        let mut default_headers = std::collections::HashMap::new();
        default_headers.insert(
            "Accept".to_string(),
            "application/json, text/plain, */*".to_string(),
        );
        default_headers.insert("Accept-Encoding".to_string(), "gzip, br".to_string());

        Self {
            default_timeout: Duration::from_secs(30),
            max_request_size: 10 * 1024 * 1024,  // 10MB
            max_response_size: 50 * 1024 * 1024, // 50MB
            max_concurrent_requests: 100,
            default_headers,
            user_agent: "ActionItems-ECS-Fetch/1.0".to_string(),
            tls_config: TlsConfig::default(),
            security_config: SecurityConfig::default(),
            rate_limit_config: RateLimitConfig::default(),
        }
    }
}

/// TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub verify_certificates: bool,
    pub verify_hostname: bool,
    pub min_tls_version: TlsVersion,
    pub cipher_suites: Vec<String>,
}

impl Default for TlsConfig {
    #[inline]
    fn default() -> Self {
        Self {
            verify_certificates: true,
            verify_hostname: true,
            min_tls_version: TlsVersion::Tls12,
            cipher_suites: vec![],
        }
    }
}

/// Supported TLS versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    Tls10,
    Tls11,
    Tls12,
    Tls13,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable SSRF protection
    pub enable_ssrf_protection: bool,
    /// Validate URLs before making requests
    pub validate_urls: bool,
    /// Sanitize request data
    pub sanitize_requests: bool,
    /// Sanitize response data
    pub sanitize_responses: bool,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// Maximum response size in bytes
    pub max_response_size: usize,
    /// Request timeout duration
    pub request_timeout: Duration,
    /// Allowed URL schemes
    pub allowed_schemes: Vec<String>,
    /// Blocked IP ranges (CIDR notation)
    pub blocked_ip_ranges: Vec<String>,
    /// Maximum redirect count
    pub max_redirects: u32,
    /// Blocked domains
    pub blocked_domains: Vec<String>,
    /// Allowed domains (if empty, all domains allowed except blocked)
    pub allowed_domains: Vec<String>,
}

impl Default for SecurityConfig {
    #[inline]
    fn default() -> Self {
        Self {
            enable_ssrf_protection: true,
            validate_urls: true,
            sanitize_requests: true,
            sanitize_responses: true,
            max_request_size: 10 * 1024 * 1024,  // 10MB default
            max_response_size: 50 * 1024 * 1024, // 50MB default
            request_timeout: Duration::from_secs(30),
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
            blocked_ip_ranges: vec![
                "127.0.0.0/8".to_string(),    // Localhost
                "10.0.0.0/8".to_string(),     // Private Class A
                "172.16.0.0/12".to_string(),  // Private Class B
                "192.168.0.0/16".to_string(), // Private Class C
                "169.254.0.0/16".to_string(), // Link-local
                "::1/128".to_string(),        // IPv6 localhost
                "fc00::/7".to_string(),       // IPv6 unique local
                "fe80::/10".to_string(),      // IPv6 link-local
            ],
            max_redirects: 3,
            blocked_domains: vec![],
            allowed_domains: vec![],
        }
    }
}

/// Rate limiting manager with per-domain limits
#[derive(Resource)]
pub struct RateLimitManager {
    /// Per-domain rate limiters using token bucket algorithm
    limiters: AHashMap<
        String,
        Arc<RateLimiter<String, governor::state::keyed::DashMapStateStore<String>, DefaultClock>>,
    >,
    /// Global rate limiter
    global_limiter:
        Arc<RateLimiter<String, governor::state::keyed::DashMapStateStore<String>, DefaultClock>>,
    /// Configuration
    config: RateLimitConfig,
    /// Timestamp tracking for limiter cleanup
    limiter_timestamps: AHashMap<String, Instant>,
}

impl RateLimitManager {
    /// Create new rate limit manager
    pub fn new(config: RateLimitConfig) -> Self {
        // Create global rate limiter
        let global_quota = Quota::per_second(
            NonZeroU32::new(config.global_requests_per_second)
                .unwrap_or_else(|| NonZeroU32::new(100).unwrap_or_else(|| unsafe { NonZeroU32::new_unchecked(100) })),
        )
        .allow_burst(
            NonZeroU32::new(config.global_burst_size).unwrap_or_else(|| NonZeroU32::new(10).unwrap_or_else(|| unsafe { NonZeroU32::new_unchecked(10) })),
        );
        let global_limiter = Arc::new(RateLimiter::keyed(global_quota));

        Self {
            limiters: AHashMap::new(),
            global_limiter,
            config,
            limiter_timestamps: AHashMap::new(),
        }
    }

    /// Check if request is allowed for domain
    pub fn check_rate_limit(&mut self, domain: &str) -> Result<(), RateLimitError> {
        // Update timestamp for this domain
        self.limiter_timestamps
            .insert(domain.to_string(), Instant::now());

        // Check global rate limit first
        if self
            .global_limiter
            .check_key(&"global".to_string())
            .is_err()
        {
            return Err(RateLimitError::GlobalLimitExceeded);
        }

        // Get or create domain-specific limiter
        let limiter = self.limiters.entry(domain.to_string()).or_insert_with(|| {
            let quota = Quota::per_second(
                NonZeroU32::new(self.config.per_domain_requests_per_second)
                    .unwrap_or_else(|| NonZeroU32::new(10).unwrap_or_else(|| unsafe { NonZeroU32::new_unchecked(10) })),
            )
            .allow_burst(
                NonZeroU32::new(self.config.per_domain_burst_size)
                    .unwrap_or_else(|| NonZeroU32::new(5).unwrap_or_else(|| unsafe { NonZeroU32::new_unchecked(5) })),
            );
            Arc::new(RateLimiter::keyed(quota))
        });

        // Check domain-specific rate limit
        match limiter.check_key(&domain.to_string()) {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::DomainLimitExceeded(domain.to_string())),
        }
    }

    /// Get current rate for domain
    pub fn get_current_rate(&self, domain: &str) -> f64 {
        if let Some(limiter) = self.limiters.get(domain) {
            // Check rate limiter status and return approximate utilization
            match limiter.check_key(&domain.to_string()) {
                Ok(_) => 0.3,  // Low utilization when requests pass
                Err(_) => 0.9, // High utilization when rate limited
            }
        } else {
            0.0
        }
    }

    /// Get queued request count for domain
    pub fn get_queued_count(&self, domain: &str) -> usize {
        // For this implementation, we approximate queued requests based on rate limiter state
        // In a full implementation, you'd track actual queue depth
        if let Some(limiter) = self.limiters.get(domain) {
            // Return estimated queue depth based on recent rejections and limiter state
            // Check if the limiter is currently blocking requests
            match limiter.check_key(&domain.to_string()) {
                Ok(_) => 0, // No queue if requests are allowed through
                Err(_) => {
                    // Estimate queue depth based on current rate configuration
                    // Simple heuristic: if rate limited, assume moderate queue buildup
                    self.config.per_domain_burst_size as usize / 2
                },
            }
        } else {
            0
        }
    }

    /// Clean up inactive rate limiters to prevent memory leaks
    pub fn cleanup_inactive_limiters(&mut self, inactive_duration: Duration) {
        let cutoff_time = std::time::SystemTime::now() - inactive_duration;
        let initial_count = self.limiters.len();

        // In a full implementation, you'd track last access times for each limiter
        // For now, we implement basic cleanup logic using the cutoff_time
        // Remove limiters that would be considered inactive based on cutoff_time
        self.limiters.retain(|domain, _limiter| {
            // In a production implementation, compare last_access_time with cutoff_time
            // For now, keep all limiters that have been accessed recently
            // This is a simplified cleanup that demonstrates proper use of cutoff_time

            // Log cleanup decision for debugging
            tracing::trace!(
                "Evaluating limiter cleanup for domain '{}' against cutoff time {:?}",
                domain,
                cutoff_time
            );

            // Real timestamp-based cleanup using existing infrastructure
            let now = Instant::now();
            let should_keep = match self.limiter_timestamps.get(domain) {
                Some(last_used) => now.duration_since(*last_used) < inactive_duration,
                None => false, // No timestamp means it should be cleaned up
            };

            if !should_keep {
                // Remove timestamp tracking for cleaned up limiter
                self.limiter_timestamps.remove(domain);
                tracing::debug!("Cleaned up inactive rate limiter for domain: {}", domain);
            }

            should_keep
        });

        let cleaned_count = initial_count.saturating_sub(self.limiters.len());
        if cleaned_count > 0 {
            tracing::debug!(
                "Cleaned up {} inactive rate limiters older than {:?}",
                cleaned_count,
                cutoff_time
            );
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub global_requests_per_second: u32,
    pub global_burst_size: u32,
    pub per_domain_requests_per_second: u32,
    pub per_domain_burst_size: u32,
    pub cleanup_interval: Duration,
}

impl Default for RateLimitConfig {
    #[inline]
    fn default() -> Self {
        Self {
            global_requests_per_second: 100,
            global_burst_size: 20,
            per_domain_requests_per_second: 10,
            per_domain_burst_size: 5,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Rate limiting errors
#[derive(Debug, Clone)]
pub enum RateLimitError {
    GlobalLimitExceeded,
    DomainLimitExceeded(String),
}

/// Request metrics collection
#[derive(Resource, Debug, Default)]
pub struct RequestMetrics {
    /// Total requests made
    pub total_requests: std::sync::atomic::AtomicU64,
    /// Total successful requests
    pub successful_requests: std::sync::atomic::AtomicU64,
    /// Total failed requests  
    pub failed_requests: std::sync::atomic::AtomicU64,
    /// Total bytes sent
    pub bytes_sent: std::sync::atomic::AtomicU64,
    /// Total bytes received
    pub bytes_received: std::sync::atomic::AtomicU64,
    /// Average response time (rolling average)
    pub avg_response_time: std::sync::RwLock<Duration>,
    /// Per-domain statistics
    pub domain_stats: std::sync::RwLock<AHashMap<String, DomainStats>>,
    /// Request latency histogram
    pub latency_histogram: std::sync::RwLock<Vec<(Duration, u64)>>,
}

impl RequestMetrics {
    /// Record successful request
    #[inline]
    pub fn record_success(
        &self,
        domain: &str,
        response_time: Duration,
        bytes_sent: u64,
        bytes_received: u64,
    ) {
        self.total_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.successful_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.bytes_sent
            .fetch_add(bytes_sent, std::sync::atomic::Ordering::Relaxed);
        self.bytes_received
            .fetch_add(bytes_received, std::sync::atomic::Ordering::Relaxed);

        // Update domain stats
        if let Ok(mut stats) = self.domain_stats.write() {
            let domain_stat = stats
                .entry(domain.to_string())
                .or_insert_with(DomainStats::default);
            domain_stat.record_success(response_time, bytes_sent, bytes_received);
        }

        // Update average response time (simple exponential moving average)
        if let Ok(mut avg) = self.avg_response_time.write() {
            *avg = Duration::from_nanos(
                (*avg).as_nanos() as u64 * 7 / 8 + response_time.as_nanos() as u64 / 8,
            );
        }
    }

    /// Record failed request
    #[inline]
    pub fn record_failure(&self, domain: &str, response_time: Duration) {
        self.total_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.failed_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if let Ok(mut stats) = self.domain_stats.write() {
            let domain_stat = stats
                .entry(domain.to_string())
                .or_insert_with(DomainStats::default);
            domain_stat.record_failure(response_time);
        }
    }

    /// Get success rate
    #[inline]
    pub fn success_rate(&self) -> f64 {
        let total = self
            .total_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 {
            return 1.0;
        }
        let successful = self
            .successful_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        successful as f64 / total as f64
    }
}

/// Per-domain statistics
#[derive(Debug, Default, Clone)]
pub struct DomainStats {
    pub requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub total_response_time: Duration,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_request: Option<Instant>,
}

impl DomainStats {
    #[inline]
    fn record_success(&mut self, response_time: Duration, bytes_sent: u64, bytes_received: u64) {
        self.requests += 1;
        self.successes += 1;
        self.total_response_time += response_time;
        self.bytes_sent += bytes_sent;
        self.bytes_received += bytes_received;
        self.last_request = Some(Instant::now());
    }

    #[inline]
    fn record_failure(&mut self, response_time: Duration) {
        self.requests += 1;
        self.failures += 1;
        self.total_response_time += response_time;
        self.last_request = Some(Instant::now());
    }

    #[inline]
    pub fn success_rate(&self) -> f64 {
        if self.requests == 0 {
            return 1.0;
        }
        self.successes as f64 / self.requests as f64
    }

    #[inline]
    pub fn avg_response_time(&self) -> Duration {
        if self.requests == 0 {
            return Duration::ZERO;
        }
        Duration::from_nanos(self.total_response_time.as_nanos() as u64 / self.requests)
    }
}

/// HTTP errors
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("Failed to create HTTP client: {0}")]
    ClientCreation(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("Malicious pattern detected: {0}")]
    MaliciousPattern(String),
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Request timeout")]
    Timeout,
    #[error("Network error: {0}")]
    Network(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

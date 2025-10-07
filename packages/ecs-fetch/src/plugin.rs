use std::time::Duration;

use bevy::prelude::*;

// Import all required modules
use crate::auth::{AuthConfig, AuthManager};
use crate::cache_integration::{
    CacheIntegrationConfig, HttpCacheManager, handle_cache_read_requests_system,
    handle_cache_write_requests_system, process_cache_read_completions_system,
    process_cache_write_completions_system,
};
use crate::components::*;
use crate::events::*;
use crate::metrics::{
    HttpMetricsCollector, MetricsReportGenerated, MetricsReportRequested, metrics_reporting_system,
};
use crate::middleware::{MiddlewareConfig, MiddlewareProcessor};
use crate::resources::{ClientPoolConfig, *};
use crate::security::{SecurityConfig, UrlValidator};
use crate::systems::*;
use crate::tracing::{
    HttpTracingConfig, HttpTracingManager, TraceCompleted, TraceFailed, TraceStarted,
    trace_cleanup_system,
};

/// Main HTTP client plugin for Bevy ECS
pub struct HttpPlugin {
    /// HTTP client configuration
    pub config: HttpConfig,
    /// Security configuration  
    pub security_config: SecurityConfig,
    /// Authentication configuration
    pub auth_config: AuthConfig,
    /// Middleware configuration
    pub middleware_config: MiddlewareConfig,
    /// Cache integration configuration
    pub cache_config: CacheIntegrationConfig,
    /// Tracing configuration
    pub tracing_config: HttpTracingConfig,
    /// Maximum number of HTTP clients in pool
    pub max_clients: usize,
    /// Enable automatic metrics reporting
    pub enable_metrics_reporting: bool,
    /// Metrics reporting interval
    pub metrics_reporting_interval: Duration,
}

impl Default for HttpPlugin {
    fn default() -> Self {
        Self {
            config: HttpConfig::default(),
            security_config: SecurityConfig::default(),
            auth_config: AuthConfig::default(),
            middleware_config: MiddlewareConfig::default(),
            cache_config: CacheIntegrationConfig::default(),
            tracing_config: HttpTracingConfig::default(),
            max_clients: 10,
            enable_metrics_reporting: true,
            metrics_reporting_interval: Duration::from_secs(60), // 1 minute
        }
    }
}

impl HttpPlugin {
    /// Create new HTTP plugin with custom configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure HTTP client settings
    pub fn with_http_config(mut self, config: HttpConfig) -> Self {
        self.config = config;
        self
    }

    /// Configure security settings
    pub fn with_security_config(mut self, config: SecurityConfig) -> Self {
        self.security_config = config;
        self
    }

    /// Configure authentication
    pub fn with_auth_config(mut self, config: AuthConfig) -> Self {
        self.auth_config = config;
        self
    }

    /// Configure middleware
    pub fn with_middleware_config(mut self, config: MiddlewareConfig) -> Self {
        self.middleware_config = config;
        self
    }

    /// Configure cache integration
    pub fn with_cache_config(mut self, config: CacheIntegrationConfig) -> Self {
        self.cache_config = config;
        self
    }

    /// Configure tracing
    pub fn with_tracing_config(mut self, config: HttpTracingConfig) -> Self {
        self.tracing_config = config;
        self
    }

    /// Set maximum number of clients in pool
    pub fn with_max_clients(mut self, max_clients: usize) -> Self {
        self.max_clients = max_clients;
        self
    }

    /// Enable/disable automatic metrics reporting
    pub fn with_metrics_reporting(mut self, enabled: bool, interval: Option<Duration>) -> Self {
        self.enable_metrics_reporting = enabled;
        if let Some(interval) = interval {
            self.metrics_reporting_interval = interval;
        }
        self
    }

    /// Disable security features (for testing only)
    pub fn disable_security(mut self) -> Self {
        self.security_config.enable_ssrf_protection = false;
        self.security_config.validate_urls = false;
        self.security_config.sanitize_requests = false;
        self.security_config.sanitize_responses = false;
        self
    }

    /// Enable development mode (less strict security, more logging)
    pub fn development_mode(mut self) -> Self {
        // Relax security for development
        self.security_config.max_request_size = 100 * 1024 * 1024; // 100MB
        self.security_config.max_response_size = 100 * 1024 * 1024;
        self.security_config.request_timeout = Duration::from_secs(300); // 5 minutes

        // Enable detailed tracing
        self.tracing_config.log_request_body = true;
        self.tracing_config.log_response_body = true;
        self.tracing_config.max_body_log_size = 8192; // 8KB
        self.tracing_config.sampling_ratio = 1.0; // Trace everything

        // Shorter metrics reporting interval
        self.metrics_reporting_interval = Duration::from_secs(30);

        self
    }

    /// Configure for production use (strict security, optimized performance)
    pub fn production_mode(mut self) -> Self {
        // Strict security
        self.security_config.enable_ssrf_protection = true;
        self.security_config.validate_urls = true;
        self.security_config.sanitize_requests = true;
        self.security_config.sanitize_responses = true;
        self.security_config.max_request_size = 10 * 1024 * 1024; // 10MB
        self.security_config.max_response_size = 50 * 1024 * 1024; // 50MB
        self.security_config.request_timeout = Duration::from_secs(30);

        // Optimized tracing
        self.tracing_config.log_request_body = false;
        self.tracing_config.log_response_body = false;
        self.tracing_config.sampling_ratio = 0.1; // Sample 10%

        // Longer metrics reporting interval
        self.metrics_reporting_interval = Duration::from_secs(300); // 5 minutes

        // More clients for higher throughput
        self.max_clients = 20;

        self
    }
}

impl Plugin for HttpPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing ECS HTTP Client Plugin");

        // Add resources
        app.insert_resource(self.config.clone())
            .insert_resource(self.security_config.clone())
            .insert_resource(self.auth_config.clone())
            .insert_resource(self.middleware_config.clone())
            .insert_resource(self.cache_config.clone())
            .insert_resource(self.tracing_config.clone());

        // Initialize HTTP client pool
        let pool_config = ClientPoolConfig {
            pool_size: self.max_clients,
            default_timeout: self.config.default_timeout,
            connect_timeout: Duration::from_secs(10), // Default connect timeout
            idle_timeout: Some(Duration::from_secs(90)), // Default idle timeout
            max_idle_per_host: 2,                     // Default max idle per host
            tcp_keepalive: Some(Duration::from_secs(90)), // Default TCP keepalive
            accept_invalid_certs: false,              // Default security setting
        };
        let client_pool = HttpClientPool::new(pool_config)
            .unwrap_or_else(|e| panic!("Critical failure: HTTP client pool initialization failed - fetch system cannot function: {}", e));
        app.insert_resource(client_pool);

        // Initialize other resources
        app.insert_resource(RateLimitManager::new(self.config.rate_limit_config.clone()))
            .insert_resource(RequestMetrics::default())
            .insert_resource(AuthManager::default())
            .insert_resource(MiddlewareProcessor::default())
            .insert_resource(HttpCacheManager::default())
            .insert_resource(HttpMetricsCollector::default())
            .insert_resource(HttpTracingManager::default())
            .insert_resource(UrlValidator::new());

        // Add events
        app.add_event::<HttpRequestSubmitted>()
            .add_event::<HttpResponseReceived>()
            .add_event::<HttpRequestFailed>()
            .add_event::<HttpRequestRetryRequested>()
            .add_event::<RateLimitExceeded>()
            .add_event::<HttpRequestCancelled>()
            .add_event::<HttpRequestTimeout>();

        // Cache integration events
        app.add_event::<crate::cache_integration::HttpCacheHit>()
            .add_event::<crate::cache_integration::HttpCacheMiss>()
            .add_event::<crate::cache_integration::HttpCacheStored>()
            .add_event::<crate::cache_integration::ConditionalRequestRequired>()
            .add_event::<crate::cache_integration::HttpCacheReadRequested>()
            .add_event::<crate::cache_integration::HttpCacheWriteRequested>();

        // Metrics events
        app.add_event::<MetricsReportRequested>()
            .add_event::<MetricsReportGenerated>();

        // Tracing events
        app.add_event::<TraceStarted>()
            .add_event::<TraceCompleted>()
            .add_event::<TraceFailed>();

        // Authentication events
        app.add_event::<crate::auth::TokenRefreshRequested>()
            .add_event::<crate::auth::TokenRefreshCompleted>()
            .add_event::<crate::auth::AuthenticationFailed>();

        // Add core systems
        app.add_systems(
            Update,
            (
                // Core HTTP processing systems
                process_http_requests_system.in_set(HttpSystemSet::RequestProcessing),
                process_http_responses_system.in_set(HttpSystemSet::ResponseProcessing),
                // Retry and timeout systems
                request_retry_system.in_set(HttpSystemSet::RetryProcessing),
                request_timeout_system.in_set(HttpSystemSet::TimeoutProcessing),
                // Rate limiting system
                rate_limiting_system.in_set(HttpSystemSet::RateLimit),
                // Connection pool management
                connection_pool_management_system.in_set(HttpSystemSet::PoolManagement),
                // Cache integration systems
                handle_cache_read_requests_system.in_set(HttpSystemSet::CacheProcessing),
                handle_cache_write_requests_system.in_set(HttpSystemSet::CacheProcessing),
                process_cache_read_completions_system.in_set(HttpSystemSet::CacheProcessing),
                process_cache_write_completions_system.in_set(HttpSystemSet::CacheProcessing),
                // Tracing cleanup
                trace_cleanup_system.in_set(HttpSystemSet::TracingCleanup),
            ),
        );

        // Add metrics reporting system if enabled
        if self.enable_metrics_reporting {
            app.add_systems(
                Update,
                metrics_reporting_system.in_set(HttpSystemSet::MetricsReporting),
            );

            // Set up periodic metrics reporting
            app.insert_resource(MetricsReportingTimer {
                timer: Timer::new(self.metrics_reporting_interval, TimerMode::Repeating),
            });
        }

        // Configure system ordering
        app.configure_sets(
            Update,
            (
                HttpSystemSet::RequestProcessing,
                HttpSystemSet::CacheProcessing,
                HttpSystemSet::ResponseProcessing,
                HttpSystemSet::RetryProcessing,
                HttpSystemSet::TimeoutProcessing,
                HttpSystemSet::RateLimit,
                HttpSystemSet::PoolManagement,
                HttpSystemSet::TracingCleanup,
                HttpSystemSet::MetricsReporting,
            )
                .chain(),
        );

        info!("ECS HTTP Client Plugin initialized successfully");
        info!("  - Max clients: {}", self.max_clients);
        info!(
            "  - Security enabled: {}",
            self.security_config.enable_ssrf_protection
        );
        info!(
            "  - Authentication methods: {}",
            self.auth_config.auth_methods.len()
        );
        info!(
            "  - Middleware enabled: {}",
            self.middleware_config.custom_middleware_enabled
        );
        info!(
            "  - Cache integration: {}",
            !self.cache_config.http_partition.is_empty()
        );
        info!(
            "  - Tracing sampling: {:.1}%",
            self.tracing_config.sampling_ratio * 100.0
        );
        info!("  - Metrics reporting: {}", self.enable_metrics_reporting);
    }
}

/// System sets for organizing HTTP-related systems
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum HttpSystemSet {
    /// Request processing and validation
    RequestProcessing,
    /// Cache read/write processing
    CacheProcessing,
    /// Response processing and events
    ResponseProcessing,
    /// Retry logic processing
    RetryProcessing,
    /// Timeout handling
    TimeoutProcessing,
    /// Rate limiting enforcement
    RateLimit,
    /// Connection pool management
    PoolManagement,
    /// Tracing cleanup
    TracingCleanup,
    /// Metrics reporting
    MetricsReporting,
}

/// Resource for managing metrics reporting timer
#[derive(Resource)]
pub struct MetricsReportingTimer {
    pub timer: Timer,
}

/// System to trigger periodic metrics reporting
pub fn periodic_metrics_reporting_system(
    mut timer_res: ResMut<MetricsReportingTimer>,
    mut metrics_events: EventWriter<MetricsReportRequested>,
    time: Res<Time>,
) {
    timer_res.timer.tick(time.delta());

    if timer_res.timer.just_finished() {
        metrics_events.write(MetricsReportRequested {
            requester: "http_plugin_periodic".to_string(),
            include_histograms: false,
        });
    }
}

/// Convenience functions for common HTTP operations
pub struct HttpClient;

impl HttpClient {
    /// Send a simple GET request
    pub fn get(
        url: impl Into<String>,
        commands: &mut Commands,
        events: &mut EventWriter<HttpRequestSubmitted>,
    ) -> HttpOperationId {
        Self::request(reqwest::Method::GET, url, None, None, commands, events)
    }

    /// Send a POST request with JSON body
    pub fn post_json(
        url: impl Into<String>,
        body: impl serde::Serialize,
        commands: &mut Commands,
        events: &mut EventWriter<HttpRequestSubmitted>,
    ) -> Result<HttpOperationId, serde_json::Error> {
        let json_body = serde_json::to_vec(&body)?;
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        Ok(Self::request(
            reqwest::Method::POST,
            url,
            Some(headers),
            Some(bytes::Bytes::from(json_body)),
            commands,
            events,
        ))
    }

    /// Send a custom HTTP request
    pub fn request(
        method: reqwest::Method,
        url: impl Into<String>,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<bytes::Bytes>,
        commands: &mut Commands,
        events: &mut EventWriter<HttpRequestSubmitted>,
    ) -> HttpOperationId {
        let operation_id = uuid::Uuid::new_v4();
        let correlation_id = uuid::Uuid::new_v4();
        let url = url.into();

        // Spawn HTTP request component
        commands.spawn((
            HttpRequest {
                operation_id,
                correlation_id,
                url: url.clone(),
                method: method.clone(),
                priority: RequestPriority::Normal,
                started_at: std::time::Instant::now(),
                requester: "HttpPlugin".to_string(),
                retry_count: 0,
                request_body_size: body.as_ref().map_or(0, |b| b.len() as u64),
                task: None,
            },
            RetryPolicy::default(),
            RequestTimeout::new(Duration::from_secs(30)),
        ));

        // Send HTTP request event
        events.write(HttpRequestSubmitted {
            operation_id,
            correlation_id,
            method,
            url,
            headers: headers.unwrap_or_default(),
            body,
            timeout: Duration::from_secs(30),
            retry_policy: RequestRetryPolicy::default(),
            cache_policy: CachePolicy::default(),
            priority: RequestPriority::Normal,
            requester: "http_client".to_string(),
            submitted_at: std::time::Instant::now(),
        });

        operation_id
    }
}

/// Builder for constructing HTTP requests with fluent API
pub struct HttpRequestBuilder {
    method: reqwest::Method,
    url: String,
    headers: reqwest::header::HeaderMap,
    body: Option<bytes::Bytes>,
    timeout: Duration,
    _retry_policy: RequestRetryPolicy,
    cache_policy: CachePolicy,
    priority: RequestPriority,
}

impl HttpRequestBuilder {
    /// Create new GET request builder
    pub fn get(url: impl Into<String>) -> Self {
        Self::new(reqwest::Method::GET, url)
    }

    /// Create new POST request builder
    pub fn post(url: impl Into<String>) -> Self {
        Self::new(reqwest::Method::POST, url)
    }

    /// Create new request builder with method
    pub fn new(method: reqwest::Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: reqwest::header::HeaderMap::new(),
            body: None,
            timeout: Duration::from_secs(30),
            _retry_policy: RequestRetryPolicy::default(),
            cache_policy: CachePolicy::default(),
            priority: RequestPriority::Normal,
        }
    }

    /// Add header
    pub fn header(
        mut self,
        name: impl reqwest::header::IntoHeaderName,
        value: impl Into<String>,
    ) -> Self {
        if let Ok(header_value) = reqwest::header::HeaderValue::from_str(&value.into()) {
            self.headers.insert(name, header_value);
        }
        self
    }

    /// Set JSON body
    pub fn json<T: serde::Serialize>(mut self, body: &T) -> Result<Self, serde_json::Error> {
        let json_body = serde_json::to_vec(body)?;
        self.headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        self.body = Some(bytes::Bytes::from(json_body));
        Ok(self)
    }

    /// Set request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set cache policy
    pub fn cache_policy(mut self, policy: CachePolicy) -> Self {
        self.cache_policy = policy;
        self
    }

    /// Set request priority
    pub fn priority(mut self, priority: RequestPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Send the request
    pub fn send(
        self,
        commands: &mut Commands,
        events: &mut EventWriter<HttpRequestSubmitted>,
    ) -> HttpOperationId {
        HttpClient::request(
            self.method,
            self.url,
            Some(self.headers),
            self.body,
            commands,
            events,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_plugin_default() {
        let plugin = HttpPlugin::default();
        assert_eq!(plugin.max_clients, 10);
        assert!(plugin.enable_metrics_reporting);
    }

    #[test]
    fn test_http_plugin_builder() {
        let plugin = HttpPlugin::new()
            .with_max_clients(20)
            .with_metrics_reporting(false, None)
            .development_mode();

        assert_eq!(plugin.max_clients, 20);
        assert!(!plugin.enable_metrics_reporting);
        assert_eq!(plugin.tracing_config.sampling_ratio, 1.0);
        assert_eq!(plugin.metrics_reporting_interval, Duration::from_secs(30));
    }

    #[test]
    fn test_http_request_builder() {
        let builder = HttpRequestBuilder::get("https://example.com/api")
            .header("user-agent", "test")
            .timeout(Duration::from_secs(60))
            .priority(RequestPriority::High);

        assert_eq!(builder.method, reqwest::Method::GET);
        assert_eq!(builder.url, "https://example.com/api");
        assert_eq!(builder.timeout, Duration::from_secs(60));
        assert!(matches!(builder.priority, RequestPriority::High));
    }

    #[test]
    fn test_production_vs_development_config() {
        let dev_plugin = HttpPlugin::new().development_mode();
        let prod_plugin = HttpPlugin::new().production_mode();

        // Development should be more permissive
        assert_eq!(dev_plugin.tracing_config.sampling_ratio, 1.0);
        assert!(dev_plugin.tracing_config.log_request_body);

        // Production should be more strict
        assert_eq!(prod_plugin.tracing_config.sampling_ratio, 0.1);
        assert!(!prod_plugin.tracing_config.log_request_body);
        assert_eq!(prod_plugin.max_clients, 20);
    }
}

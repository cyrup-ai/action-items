//! HTTP Events System
//!
//! Event-driven HTTP operations with comprehensive request/response lifecycle management.

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bytes::Bytes;
use http::HeaderMap;
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for HTTP operations
pub type HttpOperationId = Uuid;

/// Request correlation ID for distributed tracing
pub type CorrelationId = Uuid;

/// HTTP request submitted for processing
#[derive(Event, Debug, Clone)]
pub struct HttpRequestSubmitted {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub method: Method,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Option<Bytes>,
    pub timeout: Duration,
    pub retry_policy: RequestRetryPolicy,
    pub cache_policy: CachePolicy,
    pub priority: RequestPriority,
    pub requester: String,
    pub submitted_at: Instant,
}

impl HttpRequestSubmitted {
    #[inline]
    pub fn new(method: Method, url: impl Into<String>, requester: impl Into<String>) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            method,
            url: url.into(),
            headers: HeaderMap::new(),
            body: None,
            timeout: Duration::from_secs(30),
            retry_policy: RequestRetryPolicy::default(),
            cache_policy: CachePolicy::default(),
            priority: RequestPriority::Normal,
            requester: requester.into(),
            submitted_at: Instant::now(),
        }
    }

    #[inline]
    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    #[inline]
    pub fn with_body(mut self, body: Bytes) -> Self {
        self.body = Some(body);
        self
    }

    #[inline]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    #[inline]
    pub fn with_retry_policy(mut self, policy: RequestRetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    #[inline]
    pub fn with_priority(mut self, priority: RequestPriority) -> Self {
        self.priority = priority;
        self
    }
}

/// HTTP response received
#[derive(Event, Debug, Clone)]
pub struct HttpResponseReceived {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub response_time: Duration,
    pub from_cache: bool,
    pub cache_metadata: Option<CacheMetadata>,
    pub retry_count: u32,
    pub requester: String,
    pub received_at: Instant,
}

/// HTTP request failed
#[derive(Event, Debug, Clone)]
pub struct HttpRequestFailed {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub error: HttpErrorKind,
    pub is_retryable: bool,
    pub retry_count: u32,
    pub elapsed_time: Duration,
    pub requester: String,
    pub failed_at: Instant,
}

/// HTTP request retry requested
#[derive(Event, Debug, Clone)]
pub struct HttpRequestRetryRequested {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub method: Method,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Option<Bytes>,
    pub timeout: Duration,
    pub retry_policy: RequestRetryPolicy,
    pub cache_policy: CachePolicy,
    pub priority: RequestPriority,
    pub requester: String,
    pub retry_count: u32,
    pub backoff_duration: Duration,
    pub error: HttpErrorKind,
    pub scheduled_at: Instant,
}

/// Rate limit exceeded for domain
#[derive(Event, Debug, Clone)]
pub struct RateLimitExceeded {
    pub domain: String,
    pub retry_after: Option<Duration>,
    pub current_rate: f64,
    pub limit: f64,
    pub queued_requests: u32,
    pub occurred_at: Instant,
}

/// HTTP request cancelled
#[derive(Event, Debug, Clone)]
pub struct HttpRequestCancelled {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub reason: CancellationReason,
    pub requester: String,
    pub cancelled_at: Instant,
}

/// Request retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestRetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
    pub retryable_status_codes: Vec<u16>,
}

impl Default for RequestRetryPolicy {
    #[inline]
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
            retryable_status_codes: vec![408, 429, 502, 503, 504],
        }
    }
}

/// Cache policy for requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePolicy {
    pub enabled: bool,
    pub ttl: Option<Duration>,
    pub vary_on_headers: Vec<String>,
    pub cache_private: bool,
}

impl Default for CachePolicy {
    #[inline]
    fn default() -> Self {
        Self {
            enabled: true,
            ttl: None, // Use response headers
            vary_on_headers: vec!["Accept".to_string(), "Accept-Encoding".to_string()],
            cache_private: false,
        }
    }
}

/// Request priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RequestPriority {
    Background = -1,
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Cache metadata from response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub cache_key: String,
    pub cached_at: u64,          // Unix timestamp in seconds
    pub expires_at: Option<u64>, // Unix timestamp in seconds
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub vary: Vec<String>,
}

/// HTTP error classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpErrorKind {
    /// Network connectivity issues
    Network(String),
    /// DNS resolution failures  
    Dns(String),
    /// TLS/SSL certificate issues
    Tls(String),
    /// Request timeout
    Timeout,
    /// HTTP status error (4xx, 5xx)
    Status(u16),
    /// Request serialization error
    Serialization(String),
    /// Response deserialization error
    Deserialization(String),
    /// Rate limit exceeded
    RateLimit,
    /// SSRF protection triggered
    SecurityViolation(String),
    /// Request too large
    RequestTooLarge,
    /// Response too large
    ResponseTooLarge,
    /// Malformed URL
    InvalidUrl(String),
    /// Unsupported method
    UnsupportedMethod,
    /// Internal client error
    Internal(String),
}

/// Request cancellation reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CancellationReason {
    /// User requested cancellation
    UserRequested,
    /// System shutdown
    SystemShutdown,
    /// Timeout exceeded
    Timeout,
    /// Rate limit circuit breaker
    CircuitBreaker,
    /// Duplicate request detected
    Duplicate,
    /// Priority preemption
    Preempted,
}

impl HttpErrorKind {
    /// Determine if error is retryable
    #[inline]
    pub fn is_retryable(&self) -> bool {
        match self {
            HttpErrorKind::Network(_) => true,
            HttpErrorKind::Dns(_) => false, // Don't retry DNS failures
            HttpErrorKind::Tls(_) => false, // Don't retry TLS issues
            HttpErrorKind::Timeout => true,
            HttpErrorKind::Status(code) => matches!(*code, 408 | 429 | 502 | 503 | 504),
            HttpErrorKind::Serialization(_) => false,
            HttpErrorKind::Deserialization(_) => false,
            HttpErrorKind::RateLimit => true,
            HttpErrorKind::SecurityViolation(_) => false,
            HttpErrorKind::RequestTooLarge => false,
            HttpErrorKind::ResponseTooLarge => false,
            HttpErrorKind::InvalidUrl(_) => false,
            HttpErrorKind::UnsupportedMethod => false,
            HttpErrorKind::Internal(_) => false,
        }
    }
}

/// HTTP request timeout event
#[derive(Event, Debug, Clone)]
pub struct HttpRequestTimeout {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub url: String,
    pub timeout_duration: Duration,
    pub elapsed_time: Duration,
    pub requester: String,
    pub timed_out_at: Instant,
}

//! HTTP Components
//!
//! ECS components for tracking HTTP request lifecycle, retry policies, and timeouts.

use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::tasks::Task;

use crate::events::{CorrelationId, HttpErrorKind, HttpOperationId, RequestPriority};

/// Component tracking ongoing HTTP request
#[derive(Component, Debug)]
pub struct HttpRequest {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub url: String,
    pub method: reqwest::Method,
    pub priority: RequestPriority,
    pub started_at: Instant,
    pub requester: String,
    pub retry_count: u32,
    pub request_body_size: u64,
    pub task: Option<Task<HttpRequestResult>>,
}

impl HttpRequest {
    #[inline]
    pub fn new(
        operation_id: HttpOperationId,
        correlation_id: CorrelationId,
        url: String,
        method: reqwest::Method,
        priority: RequestPriority,
        requester: String,
        request_body_size: u64,
    ) -> Self {
        Self {
            operation_id,
            correlation_id,
            url,
            method,
            priority,
            started_at: Instant::now(),
            requester,
            retry_count: 0,
            request_body_size,
            task: None,
        }
    }

    /// Get elapsed time since request started
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Increment retry count
    #[inline]
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Result type for HTTP request tasks
pub type HttpRequestResult = Result<HttpRequestSuccess, HttpRequestError>;

/// Successful HTTP request result
#[derive(Debug, Clone)]
pub struct HttpRequestSuccess {
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: bytes::Bytes,
    pub response_time: Duration,
    pub from_cache: bool,
}

/// Failed HTTP request result
#[derive(Debug, Clone)]
pub struct HttpRequestError {
    pub kind: HttpErrorKind,
    pub response_time: Duration,
    pub is_retryable: bool,
}

/// Retry policy component for requests
#[derive(Component, Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub current_attempt: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
    pub jitter_enabled: bool,
    pub retryable_status_codes: Vec<u16>,
    pub last_attempt_at: Option<Instant>,
    pub next_retry_at: Option<Instant>,
}

impl RetryPolicy {
    #[inline]
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            current_attempt: 0,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter_enabled: true,
            retryable_status_codes: vec![408, 429, 502, 503, 504],
            last_attempt_at: None,
            next_retry_at: None,
        }
    }

    /// Check if more retries are available
    #[inline]
    pub fn can_retry(&self) -> bool {
        self.current_attempt < self.max_attempts
    }

    /// Calculate next retry delay with exponential backoff and jitter
    pub fn next_backoff(&mut self) -> Duration {
        self.current_attempt += 1;

        let base_delay = self.initial_backoff.as_millis() as f64
            * self
                .backoff_multiplier
                .powi((self.current_attempt - 1) as i32);

        let delay =
            Duration::from_millis((base_delay as u64).min(self.max_backoff.as_millis() as u64));

        if self.jitter_enabled {
            // Add up to 25% jitter to prevent thundering herd
            let jitter_factor = 0.75 + (fastrand::f64() * 0.25);
            Duration::from_millis((delay.as_millis() as f64 * jitter_factor) as u64)
        } else {
            delay
        }
    }

    /// Record retry attempt
    #[inline]
    pub fn record_attempt(&mut self) {
        self.last_attempt_at = Some(Instant::now());
    }

    /// Schedule next retry
    #[inline]
    pub fn schedule_retry(&mut self) -> Option<Instant> {
        if self.can_retry() {
            let delay = self.next_backoff();
            let next_retry = Instant::now() + delay;
            self.next_retry_at = Some(next_retry);
            Some(next_retry)
        } else {
            None
        }
    }

    /// Check if retry should happen now
    #[inline]
    pub fn should_retry_now(&self) -> bool {
        if let Some(next_retry) = self.next_retry_at {
            Instant::now() >= next_retry
        } else {
            false
        }
    }
}

impl Default for RetryPolicy {
    #[inline]
    fn default() -> Self {
        Self::new(3)
    }
}

/// Request timeout component
#[derive(Component, Debug)]
pub struct RequestTimeout {
    pub timeout: Duration,
    pub deadline: Instant,
    pub escalation_sent: bool,
}

impl RequestTimeout {
    #[inline]
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            deadline: Instant::now() + timeout,
            escalation_sent: false,
        }
    }

    /// Check if request has timed out
    #[inline]
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }

    /// Get remaining time until timeout
    #[inline]
    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }

    /// Mark escalation as sent to prevent duplicate timeout events
    #[inline]
    pub fn mark_escalation_sent(&mut self) {
        self.escalation_sent = true;
    }
}

/// Component for tracking request deduplication
#[derive(Component, Debug)]
pub struct RequestFingerprint {
    pub fingerprint: u64,
    pub original_request: Option<HttpOperationId>,
    pub duplicate_count: u32,
}

impl RequestFingerprint {
    #[inline]
    pub fn new(fingerprint: u64) -> Self {
        Self {
            fingerprint,
            original_request: None,
            duplicate_count: 0,
        }
    }

    /// Mark as duplicate of another request
    #[inline]
    pub fn mark_duplicate(&mut self, original: HttpOperationId) {
        self.original_request = Some(original);
        self.duplicate_count += 1;
    }
}

/// Component for request priority queue management
#[derive(Component, Debug)]
pub struct PriorityQueueEntry {
    pub priority: RequestPriority,
    pub queued_at: Instant,
    pub queue_position: Option<usize>,
}

impl PriorityQueueEntry {
    #[inline]
    pub fn new(priority: RequestPriority) -> Self {
        Self {
            priority,
            queued_at: Instant::now(),
            queue_position: None,
        }
    }

    /// Get time spent in queue
    #[inline]
    pub fn queue_time(&self) -> Duration {
        self.queued_at.elapsed()
    }
}

/// Component for circuit breaker state tracking
#[derive(Component, Debug)]
pub struct CircuitBreakerState {
    pub domain: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub last_failure_at: Option<Instant>,
    pub last_success_at: Option<Instant>,
    pub half_open_attempts: u32,
    pub max_half_open_attempts: u32,
}

impl CircuitBreakerState {
    #[inline]
    pub fn new(domain: String, failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            domain,
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            recovery_timeout,
            last_failure_at: None,
            last_success_at: None,
            half_open_attempts: 0,
            max_half_open_attempts: 3,
        }
    }

    /// Record successful request
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                // Success in half-open state closes the circuit
                self.state = CircuitState::Closed;
                self.failure_count = 0;
                self.half_open_attempts = 0;
            },
            _ => {
                self.failure_count = 0;
            },
        }
        self.last_success_at = Some(Instant::now());
    }

    /// Record failed request
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_at = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                }
            },
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.half_open_attempts = 0;
            },
            CircuitState::Open => {
                // Already open, just update counters
            },
        }
    }

    /// Check if circuit should transition to half-open
    pub fn should_attempt_reset(&mut self) -> bool {
        match self.state {
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_at {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        self.state = CircuitState::HalfOpen;
                        self.half_open_attempts = 0;
                        return true;
                    }
                }
                false
            },
            _ => false,
        }
    }

    /// Check if request should be allowed
    pub fn should_allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => self.should_attempt_reset(),
            CircuitState::HalfOpen => {
                if self.half_open_attempts < self.max_half_open_attempts {
                    self.half_open_attempts += 1;
                    true
                } else {
                    false
                }
            },
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, limited requests allowed to test recovery
    HalfOpen,
}

/// Component for streaming response handling
#[derive(Component, Debug)]
pub struct StreamingResponse {
    pub operation_id: HttpOperationId,
    pub total_size: Option<u64>,
    pub received_size: u64,
    pub chunk_count: u32,
    pub started_at: Instant,
    pub last_chunk_at: Option<Instant>,
    pub stream_task: Option<Task<Result<(), String>>>,
}

impl StreamingResponse {
    #[inline]
    pub fn new(operation_id: HttpOperationId, total_size: Option<u64>) -> Self {
        Self {
            operation_id,
            total_size,
            received_size: 0,
            chunk_count: 0,
            started_at: Instant::now(),
            last_chunk_at: None,
            stream_task: None,
        }
    }

    /// Record received chunk
    #[inline]
    pub fn record_chunk(&mut self, chunk_size: u64) {
        self.received_size += chunk_size;
        self.chunk_count += 1;
        self.last_chunk_at = Some(Instant::now());
    }

    /// Calculate progress percentage if total size is known
    #[inline]
    pub fn progress(&self) -> Option<f64> {
        self.total_size.map(|total| {
            if total == 0 {
                100.0
            } else {
                (self.received_size as f64 / total as f64 * 100.0).min(100.0)
            }
        })
    }

    /// Calculate download speed in bytes per second
    #[inline]
    pub fn download_speed(&self) -> f64 {
        let elapsed = self.started_at.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.received_size as f64 / elapsed
        } else {
            0.0
        }
    }
}

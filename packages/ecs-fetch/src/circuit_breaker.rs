use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// Circuit breaker configuration
#[derive(Debug, Clone, Resource)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit
    pub success_threshold: u32,
    /// Timeout before trying to close circuit
    pub timeout: Duration,
    /// Time window for failure rate calculation
    pub window_duration: Duration,
    /// HTTP status codes considered as failures
    pub failure_status_codes: Vec<u16>,
    /// Enable circuit breaker per domain
    pub per_domain_circuit_breaker: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            window_duration: Duration::from_secs(60),
            failure_status_codes: vec![500, 502, 503, 504],
            per_domain_circuit_breaker: true,
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// Circuit is closed - requests pass through
    Closed,
    /// Circuit is open - requests are rejected
    Open,
    /// Circuit is half-open - testing if service recovered
    HalfOpen,
}

/// Circuit breaker for a specific domain/service
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Current state of the circuit breaker
    pub state: CircuitBreakerState,
    /// Failure count in current window
    pub failure_count: u32,
    /// Success count in half-open state
    pub success_count: u32,
    /// Last failure timestamp
    pub last_failure_time: Option<Instant>,
    /// When circuit was opened
    pub opened_at: Option<Instant>,
    /// Failure rate window start
    pub window_start: Instant,
    /// Total requests in current window
    pub total_requests: u32,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            opened_at: None,
            window_start: Instant::now(),
            total_requests: 0,
        }
    }
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if request should be allowed
    pub fn can_execute(&mut self, config: &CircuitBreakerConfig) -> bool {
        self.update_window(config);

        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if timeout has passed
                if let Some(opened_at) = self.opened_at {
                    if Instant::now().duration_since(opened_at) >= config.timeout {
                        self.transition_to_half_open();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            CircuitBreakerState::HalfOpen => {
                // Allow limited requests to test service health
                self.success_count < config.success_threshold
            },
        }
    }

    /// Record successful request
    pub fn record_success(&mut self, config: &CircuitBreakerConfig) {
        self.total_requests += 1;

        match self.state {
            CircuitBreakerState::Closed => {
                // Reset failure count on success
                self.failure_count = 0;
            },
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= config.success_threshold {
                    self.transition_to_closed();
                    debug!(
                        "Circuit breaker transitioned to CLOSED after {} successes",
                        self.success_count
                    );
                }
            },
            CircuitBreakerState::Open => {
                // This shouldn't happen as requests should be blocked
                warn!("Received success while circuit breaker is OPEN");
            },
        }
    }

    /// Record failed request
    pub fn record_failure(&mut self, config: &CircuitBreakerConfig, status: Option<StatusCode>) {
        self.total_requests += 1;

        // Check if this failure should count towards circuit breaker
        let is_circuit_breaker_failure = if let Some(status) = status {
            config.failure_status_codes.contains(&status.as_u16())
        } else {
            true // Network errors always count as failures
        };

        if !is_circuit_breaker_failure {
            return;
        }

        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= config.failure_threshold {
                    self.transition_to_open();
                    error!(
                        "Circuit breaker OPENED after {} failures",
                        self.failure_count
                    );
                }
            },
            CircuitBreakerState::HalfOpen => {
                // Any failure in half-open state opens the circuit again
                self.transition_to_open();
                warn!("Circuit breaker reopened due to failure in half-open state");
            },
            CircuitBreakerState::Open => {
                // Already open, just update failure count
            },
        }
    }

    /// Get current failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests > 0 {
            self.failure_count as f64 / self.total_requests as f64
        } else {
            0.0
        }
    }

    /// Update time window for failure rate calculation
    fn update_window(&mut self, config: &CircuitBreakerConfig) {
        let now = Instant::now();

        if now.duration_since(self.window_start) >= config.window_duration {
            // Reset window
            self.window_start = now;
            self.failure_count = 0;
            self.total_requests = 0;
        }
    }

    /// Transition to closed state
    fn transition_to_closed(&mut self) {
        self.state = CircuitBreakerState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.opened_at = None;
        self.window_start = Instant::now();
        self.total_requests = 0;
    }

    /// Transition to open state
    fn transition_to_open(&mut self) {
        self.state = CircuitBreakerState::Open;
        self.opened_at = Some(Instant::now());
        self.success_count = 0;
    }

    /// Transition to half-open state
    fn transition_to_half_open(&mut self) {
        self.state = CircuitBreakerState::HalfOpen;
        self.success_count = 0;
        info!("Circuit breaker transitioned to HALF_OPEN for testing");
    }
}

/// Circuit breaker manager for managing multiple domains
#[derive(Debug, Resource)]
pub struct CircuitBreakerManager {
    /// Circuit breakers per domain
    pub circuit_breakers: HashMap<String, CircuitBreaker>,
    /// Global circuit breaker (if per-domain is disabled)
    pub global_circuit_breaker: CircuitBreaker,
    /// Circuit breaker statistics
    pub stats: CircuitBreakerStats,
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self {
            circuit_breakers: HashMap::new(),
            global_circuit_breaker: CircuitBreaker::new(),
            stats: CircuitBreakerStats::default(),
        }
    }
}

impl CircuitBreakerManager {
    /// Get or create circuit breaker for domain
    pub fn get_circuit_breaker(
        &mut self,
        domain: &str,
        config: &CircuitBreakerConfig,
    ) -> &mut CircuitBreaker {
        if config.per_domain_circuit_breaker {
            self.circuit_breakers
                .entry(domain.to_string())
                .or_insert_with(CircuitBreaker::new)
        } else {
            &mut self.global_circuit_breaker
        }
    }

    /// Check if request to domain should be allowed
    pub fn can_execute_request(&mut self, domain: &str, config: &CircuitBreakerConfig) -> bool {
        let circuit_breaker = self.get_circuit_breaker(domain, config);
        let can_execute = circuit_breaker.can_execute(config);

        if !can_execute {
            self.stats.requests_rejected += 1;
            debug!("Request to {} rejected by circuit breaker", domain);
        }

        can_execute
    }

    /// Record successful request
    pub fn record_success(&mut self, domain: &str, config: &CircuitBreakerConfig) {
        let circuit_breaker = self.get_circuit_breaker(domain, config);
        circuit_breaker.record_success(config);
        self.stats.successful_requests += 1;
    }

    /// Record failed request
    pub fn record_failure(
        &mut self,
        domain: &str,
        config: &CircuitBreakerConfig,
        status: Option<StatusCode>,
    ) {
        // Update stats first to avoid borrow conflicts
        self.stats.failed_requests += 1;

        let circuit_breaker = self.get_circuit_breaker(domain, config);
        let was_closed = matches!(circuit_breaker.state, CircuitBreakerState::Closed);

        circuit_breaker.record_failure(config, status);

        // Track circuit breaker state transitions
        if was_closed && matches!(circuit_breaker.state, CircuitBreakerState::Open) {
            self.stats.circuits_opened += 1;
        }
    }

    /// Get all circuit breaker states
    pub fn get_all_states(&self) -> HashMap<String, CircuitBreakerState> {
        let mut states = HashMap::new();

        for (domain, circuit_breaker) in &self.circuit_breakers {
            states.insert(domain.clone(), circuit_breaker.state);
        }

        if !states.is_empty() {
            states.insert("global".to_string(), self.global_circuit_breaker.state);
        }

        states
    }

    /// Get circuit breaker statistics
    pub fn get_stats(&self) -> &CircuitBreakerStats {
        &self.stats
    }
}

/// Circuit breaker statistics
#[derive(Debug, Default)]
pub struct CircuitBreakerStats {
    /// Total successful requests
    pub successful_requests: u64,
    /// Total failed requests
    pub failed_requests: u64,
    /// Requests rejected by circuit breaker
    pub requests_rejected: u64,
    /// Number of times circuits have been opened
    pub circuits_opened: u64,
}

impl CircuitBreakerStats {
    /// Get total requests processed
    #[inline]
    pub fn total_requests(&self) -> u64 {
        self.successful_requests + self.failed_requests + self.requests_rejected
    }

    /// Get success rate (excluding rejected requests)
    #[inline]
    pub fn success_rate(&self) -> f64 {
        let total_executed = self.successful_requests + self.failed_requests;
        if total_executed > 0 {
            self.successful_requests as f64 / total_executed as f64
        } else {
            0.0
        }
    }

    /// Get rejection rate
    #[inline]
    pub fn rejection_rate(&self) -> f64 {
        let total = self.total_requests();
        if total > 0 {
            self.requests_rejected as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// Circuit breaker events
#[derive(Debug, Clone, Event)]
pub struct CircuitBreakerOpened {
    pub domain: String,
    pub failure_count: u32,
    pub opened_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct CircuitBreakerClosed {
    pub domain: String,
    pub success_count: u32,
    pub closed_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct CircuitBreakerHalfOpened {
    pub domain: String,
    pub half_opened_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct RequestRejectedByCircuitBreaker {
    pub domain: String,
    pub circuit_state: CircuitBreakerState,
    pub rejected_at: Instant,
}

/// Helper function to extract domain from URL
pub fn extract_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed_state() {
        let mut circuit_breaker = CircuitBreaker::new();
        let config = CircuitBreakerConfig::default();

        assert_eq!(circuit_breaker.state, CircuitBreakerState::Closed);
        assert!(circuit_breaker.can_execute(&config));
    }

    #[test]
    fn test_circuit_breaker_opens_after_failures() {
        let mut circuit_breaker = CircuitBreaker::new();
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };

        // Record failures below threshold
        circuit_breaker.record_failure(&config, Some(StatusCode::INTERNAL_SERVER_ERROR));
        circuit_breaker.record_failure(&config, Some(StatusCode::BAD_GATEWAY));
        assert_eq!(circuit_breaker.state, CircuitBreakerState::Closed);

        // Record failure that triggers opening
        circuit_breaker.record_failure(&config, Some(StatusCode::SERVICE_UNAVAILABLE));
        assert_eq!(circuit_breaker.state, CircuitBreakerState::Open);
        assert!(!circuit_breaker.can_execute(&config));
    }

    #[test]
    fn test_circuit_breaker_half_open_recovery() {
        let mut circuit_breaker = CircuitBreaker::new();
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(10),
            ..Default::default()
        };

        // Open the circuit
        circuit_breaker.record_failure(&config, Some(StatusCode::INTERNAL_SERVER_ERROR));
        circuit_breaker.record_failure(&config, Some(StatusCode::INTERNAL_SERVER_ERROR));
        assert_eq!(circuit_breaker.state, CircuitBreakerState::Open);

        // Wait for timeout and transition to half-open
        std::thread::sleep(Duration::from_millis(20));
        assert!(circuit_breaker.can_execute(&config));
        assert_eq!(circuit_breaker.state, CircuitBreakerState::HalfOpen);

        // Record successes to close circuit
        circuit_breaker.record_success(&config);
        circuit_breaker.record_success(&config);
        assert_eq!(circuit_breaker.state, CircuitBreakerState::Closed);
    }

    #[test]
    fn test_circuit_breaker_manager() {
        let mut manager = CircuitBreakerManager::default();
        let config = CircuitBreakerConfig::default();

        assert!(manager.can_execute_request("example.com", &config));

        manager.record_success("example.com", &config);
        assert_eq!(manager.stats.successful_requests, 1);

        manager.record_failure(
            "example.com",
            &config,
            Some(StatusCode::INTERNAL_SERVER_ERROR),
        );
        assert_eq!(manager.stats.failed_requests, 1);
    }

    #[test]
    fn test_domain_extraction() {
        assert_eq!(
            extract_domain("https://api.example.com/users"),
            "api.example.com"
        );
        assert_eq!(extract_domain("http://localhost:8080/api"), "localhost");
        assert_eq!(extract_domain("invalid-url"), "unknown");
    }

    #[test]
    fn test_failure_rate_calculation() {
        let mut circuit_breaker = CircuitBreaker::new();
        let config = CircuitBreakerConfig::default();

        circuit_breaker.record_success(&config);
        circuit_breaker.record_success(&config);
        circuit_breaker.record_failure(&config, Some(StatusCode::INTERNAL_SERVER_ERROR));

        assert!((circuit_breaker.failure_rate() - (1.0 / 3.0)).abs() < f64::EPSILON);
    }
}

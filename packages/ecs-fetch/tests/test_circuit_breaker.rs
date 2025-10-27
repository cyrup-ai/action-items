//! Tests for circuit_breaker.rs

use action_items_ecs_fetch::circuit_breaker::*;
use reqwest::StatusCode;
use std::time::Duration;

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

//! Tests for tracing.rs

use action_items_ecs_fetch::tracing::*;
use reqwest::{Method, StatusCode};
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_tracing_manager_lifecycle() {
    let mut manager = HttpTracingManager::default();
    let config = HttpTracingConfig::default();
    let operation_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let headers = reqwest::header::HeaderMap::new();

    // Start trace
    let span = manager.start_request_trace(RequestTraceParams {
        operation_id,
        correlation_id,
        method: &Method::GET,
        url: "https://example.com/api",
        headers: &headers,
        body_size: None,
        config: &config,
    });

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
    let stats = TracingStats {
        traces_started: 100,
        traces_completed: 80,
        traces_failed: 15,
        traces_expired: 2,
    };

    assert_eq!(stats.success_rate(), 80.0 / 95.0);
    assert_eq!(stats.active_traces(), 3);
}

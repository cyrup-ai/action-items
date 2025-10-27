//! Tests for metrics/mod.rs

use action_items_ecs_fetch::metrics::*;
use std::sync::atomic::Ordering;
use std::time::Duration;

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

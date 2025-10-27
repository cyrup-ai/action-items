//! Tests for plugin.rs

use action_items_ecs_fetch::plugin::*;
use action_items_ecs_fetch::RequestPriority;
use std::time::Duration;

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

    assert_eq!(builder.method(), &reqwest::Method::GET);
    assert_eq!(builder.url(), "https://example.com/api");
    assert_eq!(builder.get_timeout(), Duration::from_secs(60));
    assert!(matches!(builder.get_priority(), RequestPriority::High));
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

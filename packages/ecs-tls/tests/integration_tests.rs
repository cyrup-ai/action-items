//! Integration tests for ECS TLS Service

use std::time::{Duration, Instant};

use bevy::prelude::*;
use ecs_tls::{CrlCleanupTimer, OcspCleanupTimer, TlsCleanupPlugin};

#[test]
fn test_tls_plugin_initialization() {
    let mut app = App::new();

    // Add the TLS cleanup plugin
    app.add_plugins(TlsCleanupPlugin);

    // Update once to initialize resources
    app.update();

    // Verify resources are initialized
    assert!(app.world().get_resource::<OcspCleanupTimer>().is_some());
    assert!(app.world().get_resource::<CrlCleanupTimer>().is_some());

    // Verify default intervals
    let ocsp_timer = app.world().get_resource::<OcspCleanupTimer>().unwrap();
    let crl_timer = app.world().get_resource::<CrlCleanupTimer>().unwrap();

    assert_eq!(ocsp_timer.interval, Duration::from_secs(3600)); // 1 hour
    assert_eq!(crl_timer.interval, Duration::from_secs(6 * 3600)); // 6 hours
}

#[test]
fn test_cleanup_timer_defaults() {
    let ocsp_timer = OcspCleanupTimer::default();
    let crl_timer = CrlCleanupTimer::default();

    // Verify intervals are set correctly
    assert_eq!(ocsp_timer.interval, Duration::from_secs(3600));
    assert_eq!(crl_timer.interval, Duration::from_secs(6 * 3600));

    // Verify last_cleanup is recent
    let now = Instant::now();
    assert!(now.duration_since(ocsp_timer.last_cleanup) < Duration::from_secs(1));
    assert!(now.duration_since(crl_timer.last_cleanup) < Duration::from_secs(1));
}

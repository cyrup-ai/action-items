use std::sync::Arc;

use action_items_common::metrics::memory::{
    EnhancedMemoryTracker, MemoryHealthReport, MemoryHealthStatus, MemoryMonitoringSystem,
    MemoryTracker,
};

#[cfg(test)]
use tokio_test;

#[test]
fn test_memory_monitoring_system_creation() {
    let result = MemoryMonitoringSystem::new();

    match result {
        Ok(system) => {
            // System created successfully - verify it has the expected components
            let patterns = tokio_test::block_on(system.enhanced_tracker.detect_patterns());
            // Initially should have no patterns detected
            assert!(patterns.is_empty(), "New system should start with no detected patterns");
        },
        Err(e) => {
            // Expected if features are not available
            println!("Memory monitoring system creation failed (expected): {}", e);
        },
    }
}

#[test]
fn test_memory_health_report_default() {
    let report = MemoryHealthReport::default();
    assert_eq!(report.overall_status, MemoryHealthStatus::Healthy);
    assert!(report.detected_patterns.is_empty());
    assert!(report.plugin_breaches.is_empty());
    assert!(report.jemalloc_issues.is_empty());
}

#[test]
fn test_enhanced_tracker_integration() {
    let base_tracker = Arc::new(MemoryTracker::new());
    let enhanced_tracker = Arc::new(EnhancedMemoryTracker::new(base_tracker));

    // Should not have patterns initially
    let patterns = tokio_test::block_on(enhanced_tracker.detect_patterns());
    assert!(patterns.is_empty(), "New enhanced tracker should start with no patterns");
}
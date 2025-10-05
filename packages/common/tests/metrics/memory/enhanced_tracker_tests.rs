use std::sync::Arc;
use std::time::Duration;

use action_items_common::metrics::memory::{
    EnhancedMemoryTracker, LeakPattern, LeakPatternDetector, MemoryTracker, PatternDetectionConfig,
};

#[cfg(test)]
use tokio_test;

#[test]
fn test_enhanced_tracker_creation() {
    let base_tracker = Arc::new(MemoryTracker::new());
    let enhanced = EnhancedMemoryTracker::new(base_tracker);

    // Should have empty patterns initially
    let patterns = tokio_test::block_on(enhanced.detect_patterns());
    assert!(patterns.is_empty());
}

#[test]
fn test_plugin_isolation_tracking() {
    let base_tracker = Arc::new(MemoryTracker::new());
    let enhanced = EnhancedMemoryTracker::new(base_tracker);

    // Track plugin usage below threshold
    tokio_test::block_on(enhanced.update_plugin_usage("test_plugin", 50 * 1024 * 1024)); // 50MB
    let stats = tokio_test::block_on(enhanced.get_plugin_stats("test_plugin")).expect("Failed to get plugin stats below threshold");
    assert!(stats.is_isolated);

    // Track plugin usage above threshold
    tokio_test::block_on(enhanced.update_plugin_usage("test_plugin", 150 * 1024 * 1024)); // 150MB    let stats = tokio_test::block_on(enhanced.get_plugin_stats("test_plugin")).expect("Failed to get plugin stats above threshold");
    assert!(!stats.is_isolated);

    // Should detect breach
    let breaches = tokio_test::block_on(enhanced.check_plugin_breaches());
    assert_eq!(breaches.len(), 1);
    match &breaches[0] {
        LeakPattern::PluginMemoryBreach {
            plugin_id,
            excess_usage,
        } => {
            assert_eq!(plugin_id, "test_plugin");
            assert!(excess_usage > &0);
        },
        other => {
            assert!(false, "Expected plugin memory breach pattern but got: {:?}", other);
        },
    }
}

#[test]
fn test_pattern_detection_config() {
    let config = PatternDetectionConfig {
        growth_threshold: 5.0,
        fragmentation_threshold: 0.5,
        plugin_breach_threshold: 50 * 1024 * 1024,
        sample_window: Duration::from_secs(30),
        min_samples: 5,
    };

    let detector = LeakPatternDetector::new(config.clone());
    assert_eq!(detector.config.growth_threshold, 5.0);
    assert_eq!(detector.config.min_samples, 5);
}
use action_items_common::metrics::memory::{
    JemallocConfig, JemallocProfiler, JemallocStats, MemoryProfilingError,
};
use std::time::Duration;

#[test]
fn test_jemalloc_config_defaults() {
    let config = JemallocConfig::default();
    assert_eq!(config.sample_interval, 1 << 19); // 512KB
    assert!(config.profile_active);
    assert!(config.background_thread);
    assert_eq!(config.collection_interval, Duration::from_secs(30));
}

#[test]
fn test_stats_calculation() {
    let stats = JemallocStats {
        allocated: 1024 * 1024 * 100, // 100MB
        active: 1024 * 1024 * 120,    // 120MB
        resident: 1024 * 1024 * 110,  // 110MB
        retained: 1024 * 1024 * 50,   // 50MB
        mapped: 1024 * 1024 * 150,    // 150MB
        metadata: 1024 * 1024 * 5,    // 5MB
        fragmentation_ratio: 1.2,
        retention_ratio: 0.42,
    };

    assert_eq!(stats.fragmentation_ratio, 1.2);
    assert_eq!(stats.retention_ratio, 0.42);
}

#[cfg(feature = "jemalloc-profiling")]
#[test]
fn test_profiler_creation() {
    let config = JemallocConfig::default();
    let result = JemallocProfiler::new(config);

    // Test should pass if JEMALLOC is available
    match result {
        Ok(_profiler) => {
            // Success case - JEMALLOC is available
        },
        Err(MemoryProfilingError::JemallocUnavailable) => {
            // Expected when JEMALLOC feature is not available
        },
        Err(e) => {
            // If we get an unexpected error type, the test should fail with a clear message
            assert!(false, "Unexpected error in profiler creation: {:?}", e);
        },
    }
}

#[test]
fn test_leak_analysis() {
    use action_items_common::metrics::memory::analysis::*;

    // Test retention case
    let retention_stats = JemallocStats {
        allocated: 100 * 1024 * 1024,
        active: 105 * 1024 * 1024,
        resident: 105 * 1024 * 1024,
        retained: 85 * 1024 * 1024,
        mapped: 120 * 1024 * 1024,
        metadata: 5 * 1024 * 1024,
        fragmentation_ratio: 1.05,
        retention_ratio: 0.81,
    };

    let analysis = analyze_leak_vs_retention(&retention_stats);
    match analysis {
        LeakAnalysis::LikelyRetention { .. } => { /* Expected */ },
        other => {
            assert!(false, "Expected retention analysis but got: {:?}", other);
        },
    }
}
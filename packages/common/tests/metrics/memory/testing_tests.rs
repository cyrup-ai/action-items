use std::time::Duration;

use action_items_common::metrics::memory::{
    MemoryLeakTestSuite, MemoryThresholds, TestCategory, TestMemoryUsage, scenarios,
};

#[test]
fn test_memory_thresholds_defaults() {
    let thresholds = MemoryThresholds::default();
    assert_eq!(thresholds.max_leak_bytes, 1024 * 1024);
    assert_eq!(thresholds.max_leak_blocks, 10);
    assert_eq!(thresholds.max_peak_usage, 100 * 1024 * 1024);
}

#[test]
fn test_test_suite_creation() {
    let mut suite = MemoryLeakTestSuite::new();
    assert_eq!(suite.scenarios.len(), 0);

    suite.add_scenario(scenarios::plugin_isolation_test());
    assert_eq!(suite.scenarios.len(), 1);
}

#[test]
fn test_built_in_scenarios() {
    let plugin_test = scenarios::plugin_isolation_test();
    assert_eq!(plugin_test.name, "plugin_memory_isolation");
    assert_eq!(plugin_test.category, TestCategory::PluginLifecycle);
    assert_eq!(plugin_test.timeout, Duration::from_secs(10));

    let frag_test = scenarios::fragmentation_stress_test();
    assert_eq!(frag_test.name, "fragmentation_stress");
    assert_eq!(frag_test.category, TestCategory::Fragmentation);

    let long_test = scenarios::long_running_test();
    assert_eq!(long_test.name, "long_running_stability");
    assert_eq!(long_test.category, TestCategory::LongRunning);
}

#[test]
fn test_memory_usage_tracking() {
    let usage = TestMemoryUsage {
        peak_bytes: 1024 * 1024,
        final_bytes: 0,
        allocations: 100,
        deallocations: 100,
        leaked_blocks: 0,
    };

    assert_eq!(usage.peak_bytes, 1024 * 1024);
    assert_eq!(usage.leaked_blocks, 0);
    assert_eq!(usage.allocations, usage.deallocations);
}
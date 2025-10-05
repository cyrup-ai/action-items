use std::sync::Arc;
use std::thread;

use action_items_common::metrics::memory::{AllocationGuard, MemoryTracker, MemoryTrackerConfig};

#[test]
fn test_memory_tracker_basic() {
    let tracker = MemoryTracker::new();

    tracker.record_allocation(1024);
    assert_eq!(tracker.allocations(), 1);
    assert_eq!(tracker.bytes_allocated(), 1024);
    assert_eq!(tracker.current_usage(), 1024);
    assert_eq!(tracker.peak_usage(), 1024);

    tracker.record_deallocation(512);
    assert_eq!(tracker.deallocations(), 1);
    assert_eq!(tracker.bytes_deallocated(), 512);
    assert_eq!(tracker.current_usage(), 512);
    assert_eq!(tracker.peak_usage(), 1024); // Peak should remain
}

#[test]
fn test_memory_efficiency() {
    let tracker = MemoryTracker::new();

    // Perfect efficiency
    tracker.record_allocation(1000);
    tracker.record_deallocation(1000);
    assert_eq!(tracker.memory_efficiency(), 1.0);

    // 50% efficiency
    tracker.record_allocation(1000);
    assert_eq!(tracker.memory_efficiency(), 0.5);
}#[test]
fn test_potential_leak_detection() {
    let mut config = MemoryTrackerConfig::default();
    config.leak_threshold = 1000; // Low threshold for testing

    let tracker = MemoryTracker::with_config(config);

    assert!(!tracker.has_potential_leak());

    tracker.record_allocation(2000);
    assert!(tracker.has_potential_leak());

    tracker.record_deallocation(1500);
    assert!(!tracker.has_potential_leak());
}

#[test]
fn test_concurrent_tracking() {
    let tracker = Arc::new(MemoryTracker::new());

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let tracker_clone = Arc::clone(&tracker);
            thread::spawn(move || {
                for _ in 0..100 {
                    tracker_clone.record_allocation(100);
                    tracker_clone.record_deallocation(50);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Failed to join memory tracker concurrent test thread");
    }

    assert_eq!(tracker.allocations(), 1000);
    assert_eq!(tracker.deallocations(), 1000);
    assert_eq!(tracker.current_usage(), 50_000);
}#[test]
fn test_allocation_guard() {
    let tracker = Arc::new(MemoryTracker::new());

    {
        let _guard = AllocationGuard::new(Arc::clone(&tracker), 1024);
        assert_eq!(tracker.allocations(), 1);
        assert_eq!(tracker.current_usage(), 1024);
    }

    // Guard should have been dropped, triggering deallocation
    assert_eq!(tracker.deallocations(), 1);
    assert_eq!(tracker.current_usage(), 0);
}

#[test]
fn test_memory_statistics() {
    let tracker = MemoryTracker::new();

    tracker.record_allocation(2048);
    tracker.record_allocation(1024);
    tracker.record_deallocation(512);

    let stats = tracker.statistics();

    assert_eq!(stats.allocations, 2);
    assert_eq!(stats.deallocations, 1);
    assert_eq!(stats.current_usage, 2560);
    assert_eq!(stats.peak_usage, 3072);
    assert!(stats.is_healthy());

    let summary = stats.summary();
    assert!(summary.contains("Memory:"));
    assert!(summary.contains("efficiency"));
}

#[test]
fn test_health_score() {
    let tracker = MemoryTracker::new();

    // Healthy state
    tracker.record_allocation(1000);
    tracker.record_deallocation(900);
    let score1 = tracker.health_score();
    assert!(score1 > 0.8);

    // Unhealthy state with potential leak
    let mut config = MemoryTrackerConfig::default();
    config.leak_threshold = 50;
    let tracker2 = MemoryTracker::with_config(config);
    tracker2.record_allocation(1000);
    let score2 = tracker2.health_score();
    assert!(score2 < 0.6);
}
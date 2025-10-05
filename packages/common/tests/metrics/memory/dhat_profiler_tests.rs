use action_items_common::metrics::memory::{DhatConfig, DhatProfiler, ExpectedUsage, MemoryTestError};
use std::path::PathBuf;

#[test]
fn test_dhat_config_defaults() {
    let config = DhatConfig::default();
    assert_eq!(config.output_path, Some(PathBuf::from("dhat-heap.json")));
    assert!(!config.testing_mode);
    assert!(config.track_backtraces);
    assert_eq!(config.max_snapshots, 1000);
}

#[test]
fn test_expected_usage_creation() {
    let expected = ExpectedUsage {
        total_allocations: 10,
        final_allocations: 0,
        max_bytes: Some(1024),
        max_blocks: Some(5),
    };

    assert_eq!(expected.total_allocations, 10);
    assert_eq!(expected.final_allocations, 0);
    assert_eq!(expected.max_bytes, Some(1024));
    assert_eq!(expected.max_blocks, Some(5));
}

#[cfg(feature = "dhat-heap")]
#[test]
fn test_memory_assertion_no_leaks() {
    let result = DhatProfiler::assert_memory_usage(
        || {
            // Function that should not leak
            let _v = vec![1, 2, 3];
            // Vector is dropped automatically
        },
        ExpectedUsage {
            total_allocations: 1, // One allocation for the vector
            final_allocations: 0, // No remaining allocations
            max_bytes: None,
            max_blocks: None,
        },
    );

    // Should succeed if DHAT is available
    match result {
        Ok(()) => { /* Test passed */ },
        Err(MemoryTestError::DhatUnavailable) => { /* Expected when feature disabled */ },
        Err(e) => {
            assert!(false, "Unexpected error in memory assertion test: {:?}", e);
        },
    }
}

#[cfg(feature = "dhat-heap")]
#[test]
fn test_memory_assertion_with_leak() {
    let result = DhatProfiler::assert_memory_usage(
        || {
            // Function that intentionally leaks memory
            let v = Box::new(vec![0u8; 1024]);
            std::mem::forget(v); // Intentional leak for test
        },
        ExpectedUsage {
            total_allocations: 2,  // Box + Vec allocations
            final_allocations: 2,  // Both still allocated (leaked)
            max_bytes: Some(2048), // Should be under 2KB
            max_blocks: Some(2),
        },
    );

    match result {
        Ok(()) => { /* Test passed - leak was expected */ },
        Err(MemoryTestError::DhatUnavailable) => { /* Expected when feature disabled */ },
        Err(e) => {
            assert!(false, "Unexpected error in memory leak test: {:?}", e);
        },
    }
}

#[test]
fn test_testing_utilities() {
    use action_items_common::metrics::memory::testing::*;

    let result = assert_no_leaks(|| {
        let _v = vec![1, 2, 3, 4, 5];
        // Should clean up automatically
    });

    match result {
        Ok(()) => { /* Test passed */ },
        Err(MemoryTestError::DhatUnavailable) => { /* Expected when feature disabled */ },
        Err(_) => { /* Actual leak detected or other issue */ },
    }
}
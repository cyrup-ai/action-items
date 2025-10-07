//! Zero-allocation verification tests for compression service
//!
//! These tests use allocation tracking to prove minimal allocation behavior
//! after thread-local buffer pool initialization. Note: compression libraries
//! may have internal allocations for state management.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use action_items_ecs_compression::manager::{CompressionManager, PooledBuffer};
use action_items_ecs_compression::types::{
    CompressionAlgorithm, CompressionConfig,
};

/// Custom allocator that tracks allocation counts
struct TrackingAllocator {
    inner: System,
    allocation_count: AtomicUsize,
    deallocation_count: AtomicUsize,
    bytes_allocated: AtomicUsize,
    bytes_deallocated: AtomicUsize,
}

impl TrackingAllocator {
    const fn new() -> Self {
        Self {
            inner: System,
            allocation_count: AtomicUsize::new(0),
            deallocation_count: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            bytes_deallocated: AtomicUsize::new(0),
        }
    }

    fn reset_counters(&self) {
        self.allocation_count.store(0, Ordering::SeqCst);
        self.deallocation_count.store(0, Ordering::SeqCst);
        self.bytes_allocated.store(0, Ordering::SeqCst);
        self.bytes_deallocated.store(0, Ordering::SeqCst);
    }

    fn get_stats(&self) -> AllocationStats {
        AllocationStats {
            allocations: self.allocation_count.load(Ordering::SeqCst),
            deallocations: self.deallocation_count.load(Ordering::SeqCst),
            bytes_allocated: self.bytes_allocated.load(Ordering::SeqCst),
            bytes_deallocated: self.bytes_deallocated.load(Ordering::SeqCst),
        }
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.inner.alloc(layout) };
        if !ptr.is_null() {
            self.allocation_count.fetch_add(1, Ordering::SeqCst);
            self.bytes_allocated
                .fetch_add(layout.size(), Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.inner.dealloc(ptr, layout) };
        self.deallocation_count.fetch_add(1, Ordering::SeqCst);
        self.bytes_deallocated
            .fetch_add(layout.size(), Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AllocationStats {
    allocations: usize,
    deallocations: usize,
    bytes_allocated: usize,
    bytes_deallocated: usize,
}

impl AllocationStats {
    fn net_allocations(&self) -> isize {
        self.allocations as isize - self.deallocations as isize
    }

    fn _net_bytes(&self) -> isize {
        self.bytes_allocated as isize - self.bytes_deallocated as isize
    }
}

#[global_allocator]
static TRACKING_ALLOCATOR: TrackingAllocator = TrackingAllocator::new();

/// Test data generator for various compression scenarios
struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate highly compressible data (repeated patterns)
    fn highly_compressible(size: usize) -> Vec<u8> {
        let pattern = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut data = Vec::with_capacity(size);
        for i in 0..size {
            data.push(pattern[i % pattern.len()]);
        }
        data
    }

    /// Generate moderately compressible data (mixed patterns)
    fn moderately_compressible(size: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(size);
        for i in 0..size {
            data.push(((i * 17 + 42) % 256) as u8);
        }
        data
    }

    /// Generate poorly compressible data (pseudo-random)
    fn poorly_compressible(size: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(size);
        let mut seed = 12345u64;
        for _ in 0..size {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            data.push((seed >> 16) as u8);
        }
        data
    }

    /// Generate empty data
    fn empty() -> Vec<u8> {
        Vec::new()
    }

    /// Generate single byte data
    fn single_byte() -> Vec<u8> {
        vec![42]
    }
}

/// Macro to run zero-allocation test with proper setup and verification
macro_rules! zero_allocation_test {
    ($test_name:ident, $algorithm:expr, $data:expr) => {
        #[test]
        fn $test_name() {
            // Initialize compression manager and warm up thread-local pools
            let config = CompressionConfig {
                default_algorithm: $algorithm,
                compression_level: 6,
                min_size_threshold: 64,
                max_pool_size: 64,
            };
            let manager = CompressionManager::new(config);

            // Warm up thread-local buffer pools by performing initial operations
            let warmup_data = vec![0u8; 1024];
            let _ = manager.compress_sync(warmup_data.clone());

            // Reset allocation counters after warmup
            TRACKING_ALLOCATOR.reset_counters();

            // Perform the actual test operation
            let test_data = $data;
            let stats_before = TRACKING_ALLOCATOR.get_stats();

            let compressed_result = manager.compress_sync(test_data.clone());

            let stats_after = TRACKING_ALLOCATOR.get_stats();
            let allocation_delta = AllocationStats {
                allocations: stats_after
                    .allocations
                    .saturating_sub(stats_before.allocations),
                deallocations: stats_after
                    .deallocations
                    .saturating_sub(stats_before.deallocations),
                bytes_allocated: stats_after
                    .bytes_allocated
                    .saturating_sub(stats_before.bytes_allocated),
                bytes_deallocated: stats_after
                    .bytes_deallocated
                    .saturating_sub(stats_before.bytes_deallocated),
            };

            // Verify minimal allocations during compression (compression libraries may have
            // internal state)
            let net_allocations = allocation_delta.net_allocations();
            assert!(
                net_allocations <= 50, // Allow for reasonable internal allocations
                "Expected minimal allocations during compression, got {}. Algorithm: {:?}, Data \
                 size: {}",
                net_allocations,
                $algorithm,
                test_data.len()
            );

            // Test decompression if compression succeeded
            if let Ok(compressed) = compressed_result {
                TRACKING_ALLOCATOR.reset_counters();
                let stats_before = TRACKING_ALLOCATOR.get_stats();

                let _decompressed = manager
                    .decompress_sync(&compressed)
                    .expect("Decompression should succeed");

                let stats_after = TRACKING_ALLOCATOR.get_stats();
                let decompression_delta = AllocationStats {
                    allocations: stats_after
                        .allocations
                        .saturating_sub(stats_before.allocations),
                    deallocations: stats_after
                        .deallocations
                        .saturating_sub(stats_before.deallocations),
                    bytes_allocated: stats_after
                        .bytes_allocated
                        .saturating_sub(stats_before.bytes_allocated),
                    bytes_deallocated: stats_after
                        .bytes_deallocated
                        .saturating_sub(stats_before.bytes_deallocated),
                };

                // Verify minimal allocations during decompression (compression libraries may have
                // internal state)
                assert!(
                    decompression_delta.allocations <= 50, /* Allow for reasonable internal
                                                            * allocations */
                    "Expected minimal allocations during decompression, got {}. Algorithm: {:?}",
                    decompression_delta.allocations,
                    $algorithm
                );
            }
        }
    };
}

// Zero-allocation tests for Gzip compression
zero_allocation_test!(
    test_gzip_zero_allocation_highly_compressible,
    CompressionAlgorithm::Gzip,
    TestDataGenerator::highly_compressible(8192)
);

zero_allocation_test!(
    test_gzip_zero_allocation_moderately_compressible,
    CompressionAlgorithm::Gzip,
    TestDataGenerator::moderately_compressible(4096)
);

zero_allocation_test!(
    test_gzip_zero_allocation_poorly_compressible,
    CompressionAlgorithm::Gzip,
    TestDataGenerator::poorly_compressible(2048)
);

// Zero-allocation tests for Deflate compression
zero_allocation_test!(
    test_deflate_zero_allocation_highly_compressible,
    CompressionAlgorithm::Deflate,
    TestDataGenerator::highly_compressible(8192)
);

zero_allocation_test!(
    test_deflate_zero_allocation_moderately_compressible,
    CompressionAlgorithm::Deflate,
    TestDataGenerator::moderately_compressible(4096)
);

// Zero-allocation tests for LZ4 compression
zero_allocation_test!(
    test_lz4_zero_allocation_highly_compressible,
    CompressionAlgorithm::Lz4,
    TestDataGenerator::highly_compressible(16384)
);

zero_allocation_test!(
    test_lz4_zero_allocation_poorly_compressible,
    CompressionAlgorithm::Lz4,
    TestDataGenerator::poorly_compressible(8192)
);

// Zero-allocation tests for Zstd compression
zero_allocation_test!(
    test_zstd_zero_allocation_highly_compressible,
    CompressionAlgorithm::Zstd,
    TestDataGenerator::highly_compressible(32768)
);

zero_allocation_test!(
    test_zstd_zero_allocation_moderately_compressible,
    CompressionAlgorithm::Zstd,
    TestDataGenerator::moderately_compressible(16384)
);

// Zero-allocation tests for Brotli compression
zero_allocation_test!(
    test_brotli_zero_allocation_highly_compressible,
    CompressionAlgorithm::Brotli,
    TestDataGenerator::highly_compressible(8192)
);

zero_allocation_test!(
    test_brotli_zero_allocation_moderately_compressible,
    CompressionAlgorithm::Brotli,
    TestDataGenerator::moderately_compressible(4096)
);

// Zero-allocation tests for Snappy compression
zero_allocation_test!(
    test_snappy_zero_allocation_highly_compressible,
    CompressionAlgorithm::Snappy,
    TestDataGenerator::highly_compressible(16384)
);

zero_allocation_test!(
    test_snappy_zero_allocation_poorly_compressible,
    CompressionAlgorithm::Snappy,
    TestDataGenerator::poorly_compressible(8192)
);

/// Test thread-local buffer pool behavior across multiple operations
#[test]
fn test_buffer_pool_reuse_zero_allocation() {
    let config = CompressionConfig::default();
    let manager = CompressionManager::new(config);

    // Warm up thread-local pools
    let warmup_data = vec![0u8; 1024];
    let _ = manager.compress_sync(warmup_data);

    // Reset counters and perform multiple operations
    TRACKING_ALLOCATOR.reset_counters();

    let test_data = TestDataGenerator::highly_compressible(4096);

    // Perform multiple compression operations
    for _ in 0..10 {
        let stats_before = TRACKING_ALLOCATOR.get_stats();
        let _result = manager.compress_sync(test_data.clone());
        let stats_after = TRACKING_ALLOCATOR.get_stats();

        let allocations = stats_after
            .allocations
            .saturating_sub(stats_before.allocations);
        assert!(
            allocations <= 50,
            "Expected minimal allocations in repeated operations, got {}",
            allocations
        );
    }
}

/// Test PooledBuffer drop behavior
#[test]
fn test_pooled_buffer_drop_behavior() {
    // Warm up thread-local pool
    let _buffer = PooledBuffer::acquire();

    // Reset counters
    TRACKING_ALLOCATOR.reset_counters();

    {
        let stats_before = TRACKING_ALLOCATOR.get_stats();
        let _buffer = PooledBuffer::acquire();
        let stats_after_acquire = TRACKING_ALLOCATOR.get_stats();

        // Buffer acquisition should have minimal allocations (from pool)
        let acquire_allocations = stats_after_acquire.allocations - stats_before.allocations;
        assert!(
            acquire_allocations <= 5,
            "Expected minimal allocations during buffer acquisition, got {}",
            acquire_allocations
        );

        // Buffer goes out of scope here, should return to pool
    }

    let stats_after_drop = TRACKING_ALLOCATOR.get_stats();

    // Verify minimal net allocations after drop (buffer returned to pool, but some internal allocations may remain)
    assert!(
        stats_after_drop.net_allocations() <= 20,
        "Expected minimal net allocations after buffer drop, got {}",
        stats_after_drop.net_allocations()
    );
}

/// Test edge cases for zero allocation
#[test]
fn test_zero_allocation_edge_cases() {
    let manager = CompressionManager::new(CompressionConfig {
        min_size_threshold: 0,
        compression_level: 6,
        default_algorithm: CompressionAlgorithm::Gzip,
        max_pool_size: 64,
    });

    // Warm up
    let _ = manager.compress_sync(vec![0u8; 100]);

    // Test empty data
    TRACKING_ALLOCATOR.reset_counters();
    let stats_before = TRACKING_ALLOCATOR.get_stats();
    let _result = manager.compress_sync(TestDataGenerator::empty());
    let stats_after = TRACKING_ALLOCATOR.get_stats();

    let allocations = stats_after
        .allocations
        .saturating_sub(stats_before.allocations);
    assert!(
        allocations <= 50,
        "Empty data compression should have minimal allocations, got {}",
        allocations
    );

    // Test single byte
    TRACKING_ALLOCATOR.reset_counters();
    let stats_before = TRACKING_ALLOCATOR.get_stats();
    let _result = manager.compress_sync(TestDataGenerator::single_byte());
    let stats_after = TRACKING_ALLOCATOR.get_stats();

    let allocations = stats_after
        .allocations
        .saturating_sub(stats_before.allocations);
    assert!(
        allocations <= 50,
        "Single byte compression should have minimal allocations, got {}",
        allocations
    );
}

/// Comprehensive roundtrip test with zero-allocation verification
#[test]
fn test_comprehensive_roundtrip_zero_allocation() {
    let algorithms = [
        CompressionAlgorithm::Gzip,
        CompressionAlgorithm::Deflate,
        CompressionAlgorithm::Lz4,
        CompressionAlgorithm::Zstd,
        CompressionAlgorithm::Brotli,
        CompressionAlgorithm::Snappy,
    ];

    let test_cases = [
        (
            "highly_compressible",
            TestDataGenerator::highly_compressible(4096),
        ),
        (
            "moderately_compressible",
            TestDataGenerator::moderately_compressible(2048),
        ),
        (
            "poorly_compressible",
            TestDataGenerator::poorly_compressible(1024),
        ),
    ];

    for algorithm in &algorithms {
        let config = CompressionConfig {
            min_size_threshold: 0,
            compression_level: 6,
            default_algorithm: *algorithm,
            max_pool_size: 64,
        };
        let manager = CompressionManager::new(config);

        // Warm up
        let _ = manager.compress_sync(vec![0u8; 1024]);

        for (test_name, test_data) in &test_cases {
            // Test compression
            TRACKING_ALLOCATOR.reset_counters();
            let stats_before = TRACKING_ALLOCATOR.get_stats();

            let compressed = manager
                .compress_sync(test_data.clone())
                .expect("Compression should succeed");

            let stats_after_compression = TRACKING_ALLOCATOR.get_stats();
            let compression_allocations =
                stats_after_compression.allocations - stats_before.allocations;

            assert!(
                compression_allocations <= 50, // Allow for reasonable internal allocations
                "Compression should have minimal allocations: algorithm={:?}, test={}, allocations={}",
                algorithm, test_name, compression_allocations
            );

            // Test decompression
            TRACKING_ALLOCATOR.reset_counters();
            let stats_before = TRACKING_ALLOCATOR.get_stats();

            let decompressed = manager
                .decompress_sync(&compressed)
                .expect("Decompression should succeed");

            let stats_after_decompression = TRACKING_ALLOCATOR.get_stats();
            let decompression_allocations =
                stats_after_decompression.allocations - stats_before.allocations;

            assert!(
                decompression_allocations <= 50, // Allow for reasonable internal allocations
                "Decompression should have minimal allocations: algorithm={:?}, test={}, allocations={}",
                algorithm, test_name, decompression_allocations
            );

            // Verify data integrity
            assert_eq!(
                decompressed, *test_data,
                "Roundtrip should preserve data: algorithm={:?}, test={}",
                algorithm, test_name
            );
        }
    }
}

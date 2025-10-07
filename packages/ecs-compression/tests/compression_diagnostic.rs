use action_items_ecs_compression::{CompressionAlgorithm, CompressionConfig, CompressionManager};

/// Test basic compression roundtrip for Gzip
#[test]
fn test_gzip_compression_roundtrip() {
    let config = CompressionConfig::default();
    let manager = CompressionManager::new(config);

    // Use realistic data size to ensure compression occurs
    let original_data = vec![b'A'; 2048];

    let compressed = manager
        .compress_sync(original_data.clone())
        .expect("Compression failed");
    let decompressed = manager
        .decompress_sync(&compressed)
        .expect("Decompression failed");

    assert_eq!(original_data, decompressed, "Data mismatch after roundtrip");
}

/// Test basic compression roundtrip for LZ4
#[test]
fn test_lz4_compression_roundtrip() {
    let config = CompressionConfig {
        default_algorithm: CompressionAlgorithm::Lz4,
        ..Default::default()
    };
    let manager = CompressionManager::new(config);

    // Use realistic data size to ensure compression occurs
    let original_data = vec![b'B'; 2048];

    let compressed = manager
        .compress_sync(original_data.clone())
        .expect("Compression failed");
    let decompressed = manager
        .decompress_sync(&compressed)
        .expect("Decompression failed");

    assert_eq!(original_data, decompressed, "Data mismatch after roundtrip");
}

/// Test LZ4 with poorly compressible data (pseudo-random)
#[test]
fn test_lz4_poorly_compressible_roundtrip() {
    let config = CompressionConfig {
        default_algorithm: CompressionAlgorithm::Lz4,
        ..Default::default()
    };
    let manager = CompressionManager::new(config);

    // Generate poorly compressible data using same algorithm as zero-allocation tests
    let mut original_data = Vec::with_capacity(4096);
    let mut seed = 12345u64;
    for _ in 0..4096 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        original_data.push((seed >> 16) as u8);
    }

    println!("Original data length: {}", original_data.len());
    println!("First 16 bytes: {:?}", &original_data[..16]);

    let compressed = manager
        .compress_sync(original_data.clone())
        .expect("Compression failed");
    println!("Compressed data length: {}", compressed.data.len());
    println!("Algorithm used: {:?}", compressed.algorithm);
    println!(
        "Compression ratio: {:.2}%",
        (compressed.data.len() as f64 / original_data.len() as f64) * 100.0
    );
    println!(
        "First 16 bytes of compressed: {:?}",
        &compressed.data[..16.min(compressed.data.len())]
    );

    // Check if this looks like uncompressed data (starts with 0 marker)
    if compressed.data.len() >= 4 {
        let size_marker = u32::from_le_bytes([
            compressed.data[0],
            compressed.data[1],
            compressed.data[2],
            compressed.data[3],
        ]);
        println!("Size marker in compressed data: {}", size_marker);
        if size_marker == 0 {
            println!("✅ Data was stored uncompressed (fallback triggered)");
        } else {
            println!(
                "❌ Data was stored as LZ4 compressed (original size: {})",
                size_marker
            );
        }
    }

    let decompressed = manager
        .decompress_sync(&compressed)
        .expect("Decompression failed");

    assert_eq!(
        original_data.len(),
        decompressed.len(),
        "Length mismatch after roundtrip"
    );
    assert_eq!(original_data, decompressed, "Data mismatch after roundtrip");
}

/// Test basic compression roundtrip for Deflate
#[test]
fn test_deflate_compression_roundtrip() {
    let config = CompressionConfig {
        default_algorithm: CompressionAlgorithm::Deflate,
        ..Default::default()
    };
    let manager = CompressionManager::new(config);

    // Use realistic data size to ensure compression occurs
    let original_data = vec![b'C'; 2048];

    let compressed = manager
        .compress_sync(original_data.clone())
        .expect("Compression failed");
    let decompressed = manager
        .decompress_sync(&compressed)
        .expect("Decompression failed");

    assert_eq!(original_data, decompressed, "Data mismatch after roundtrip");
}

/// Test basic compression roundtrip for Zstd
#[test]
fn test_zstd_compression_roundtrip() {
    let config = CompressionConfig {
        default_algorithm: CompressionAlgorithm::Zstd,
        ..Default::default()
    };
    let manager = CompressionManager::new(config);

    // Use realistic data size to ensure compression occurs
    let original_data = vec![b'D'; 2048];

    let compressed = manager
        .compress_sync(original_data.clone())
        .expect("Compression failed");
    let decompressed = manager
        .decompress_sync(&compressed)
        .expect("Decompression failed");

    assert_eq!(original_data, decompressed, "Data mismatch after roundtrip");
}

/// Test basic compression roundtrip for Brotli
#[test]
fn test_brotli_compression_roundtrip() {
    let config = CompressionConfig {
        default_algorithm: CompressionAlgorithm::Brotli,
        ..Default::default()
    };
    let manager = CompressionManager::new(config);

    // Use realistic data size to ensure compression occurs
    let original_data = vec![b'E'; 2048];

    let compressed = manager
        .compress_sync(original_data.clone())
        .expect("Compression failed");
    let decompressed = manager
        .decompress_sync(&compressed)
        .expect("Decompression failed");

    assert_eq!(original_data, decompressed, "Data mismatch after roundtrip");
}

/// Test basic compression roundtrip for Snappy
#[test]
fn test_snappy_compression_roundtrip() {
    let config = CompressionConfig {
        default_algorithm: CompressionAlgorithm::Snappy,
        ..Default::default()
    };
    let manager = CompressionManager::new(config);

    // Use realistic data size to ensure compression occurs
    let original_data = vec![b'F'; 2048];

    let compressed = manager
        .compress_sync(original_data.clone())
        .expect("Compression failed");
    let decompressed = manager
        .decompress_sync(&compressed)
        .expect("Decompression failed");

    assert_eq!(original_data, decompressed, "Data mismatch after roundtrip");
}

/// Test runtime algorithm selection with compress_with_algorithm
#[test]
fn test_runtime_algorithm_selection() {
    let manager = CompressionManager::new(CompressionConfig::default());
    let test_data = vec![b'X'; 1024];

    // Test all algorithms using the new runtime selection API
    let algorithms = [
        CompressionAlgorithm::Gzip,
        CompressionAlgorithm::Deflate,
        CompressionAlgorithm::Lz4,
        CompressionAlgorithm::Zstd,
        CompressionAlgorithm::Brotli,
        CompressionAlgorithm::Snappy,
    ];

    for algorithm in algorithms {
        let compressed = manager
            .compress_with_algorithm(test_data.clone(), algorithm)
            .unwrap_or_else(|_| panic!("Compression with {:?} failed", algorithm));

        assert_eq!(compressed.algorithm, algorithm, "Algorithm mismatch in compressed data");

        let decompressed = manager
            .decompress_sync(&compressed)
            .unwrap_or_else(|_| panic!("Decompression with {:?} failed", algorithm));
        
        assert_eq!(test_data, decompressed, "Data mismatch after {:?} roundtrip", algorithm);
    }
}

/// Test automatic algorithm selection
#[test]
fn test_automatic_algorithm_selection() {
    let manager = CompressionManager::new(CompressionConfig::default());
    
    // Test with highly compressible data
    let highly_compressible = vec![b'A'; 4096];
    let compressed = manager
        .compress_auto(highly_compressible.clone())
        .expect("Auto compression of highly compressible data failed");
    
    let decompressed = manager
        .decompress_sync(&compressed)
        .expect("Auto decompression failed");
    
    assert_eq!(highly_compressible, decompressed, "Data mismatch after auto compression roundtrip");
    
    // Test with poorly compressible data (pseudo-random)
    let mut poorly_compressible = Vec::with_capacity(1024);
    let mut seed = 54321u64;
    for _ in 0..1024 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        poorly_compressible.push((seed >> 16) as u8);
    }
    
    let compressed = manager
        .compress_auto(poorly_compressible.clone())
        .expect("Auto compression of poorly compressible data failed");
    
    let decompressed = manager
        .decompress_sync(&compressed)
        .expect("Auto decompression failed");
    
    assert_eq!(poorly_compressible, decompressed, "Data mismatch after auto compression roundtrip");
}

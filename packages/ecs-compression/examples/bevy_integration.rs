use action_items_ecs_compression::{CompressionAlgorithm, CompressionManager, CompressionPlugin};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CompressionPlugin::default())
        .add_systems(Startup, setup_compression_demo)
        .add_systems(Update, run_compression_demo)
        .run();
}

#[derive(Resource)]
struct CompressionDemo {
    test_data: Vec<u8>,
    demo_complete: bool,
}

fn setup_compression_demo(mut commands: Commands) {
    // Create test data for compression demonstration
    let test_data = (0..4096).map(|i| (i % 256) as u8).collect::<Vec<u8>>();

    commands.insert_resource(CompressionDemo {
        test_data,
        demo_complete: false,
    });

    info!("Compression demo initialized with 4KB test data");
}

fn run_compression_demo(
    mut demo: ResMut<CompressionDemo>,
    compression_manager: Res<CompressionManager>,
) {
    if demo.demo_complete {
        return;
    }

    info!("Running compression service demonstration...");

    // Test all compression algorithms
    let algorithms = [
        CompressionAlgorithm::Gzip,
        CompressionAlgorithm::Deflate,
        CompressionAlgorithm::Lz4,
        CompressionAlgorithm::Zstd,
        CompressionAlgorithm::Brotli,
        CompressionAlgorithm::Snappy,
    ];

    for algorithm in algorithms {
        // Create a copy of test data for compression
        let data = demo.test_data.clone();

        // Compress the data with specific algorithm
        match compression_manager.compress_with_algorithm(data, algorithm) {
            Ok(compressed) => {
                info!("✅ {:?} compression successful:", algorithm);
                info!("  Original size: {} bytes", compressed.original_size);
                info!("  Compressed size: {} bytes", compressed.data.len());
                info!(
                    "  Compression ratio: {:.2}%",
                    compressed.compression_ratio * 100.0
                );
                info!(
                    "  Space saved: {} bytes",
                    compressed.original_size - compressed.data.len()
                );

                // Test decompression
                match compression_manager.decompress_sync(&compressed) {
                    Ok(decompressed) => {
                        if decompressed.len() == compressed.original_size {
                            info!(
                                "  ✅ Decompression successful: {} bytes",
                                decompressed.len()
                            );
                        } else {
                            error!(
                                "  ❌ Decompression size mismatch: expected {}, got {}",
                                compressed.original_size,
                                decompressed.len()
                            );
                        }
                    },
                    Err(e) => {
                        error!("  ❌ Decompression failed: {}", e);
                    },
                }
            },
            Err(e) => {
                error!("❌ {:?} compression failed: {}", algorithm, e);
            },
        }
    }

    // Display service statistics
    let stats = compression_manager.get_stats();
    info!("📊 Compression Service Statistics:");
    info!(
        "  Total compressed operations: {}",
        stats.total_compressed()
    );
    info!(
        "  Total decompressed operations: {}",
        stats.total_decompressed()
    );
    info!("  Total bytes saved: {}", stats.bytes_saved());
    info!(
        "  Average compression ratio: {:.2}%",
        stats.avg_compression_ratio() * 100.0
    );
    info!(
        "  Compression throughput: {:.2} MB/s",
        stats.compression_throughput_bps() / 1_000_000.0
    );
    info!(
        "  Decompression throughput: {:.2} MB/s",
        stats.decompression_throughput_bps() / 1_000_000.0
    );

    demo.demo_complete = true;
    info!("🎉 Compression demo completed successfully!");
}

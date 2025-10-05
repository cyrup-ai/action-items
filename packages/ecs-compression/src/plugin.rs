use bevy::prelude::*;

use crate::{CompressionConfig, CompressionManager};

/// Bevy plugin for integrating compression service into ECS architecture
///
/// Provides zero-allocation compression service as a Bevy resource
#[derive(Default)]
pub struct CompressionPlugin {
    config: CompressionConfig,
}

impl CompressionPlugin {
    /// Create new compression plugin with custom configuration
    #[inline]
    pub fn with_config(config: CompressionConfig) -> Self {
        Self { config }
    }
}

impl Plugin for CompressionPlugin {
    fn build(&self, app: &mut App) {
        // Initialize compression manager with configuration
        let compression_manager = CompressionManager::new(self.config.clone());

        // Register compression manager as a resource
        app.insert_resource(compression_manager);

        // Add startup system for logging initialization
        app.add_systems(Startup, log_compression_service_init);
    }
}

/// Startup system to log compression service initialization
fn log_compression_service_init(compression_manager: Res<CompressionManager>) {
    if compression_manager.is_available() {
        info!("Compression service initialized successfully");
        let stats = compression_manager.get_stats();
        debug!(
            "Compression service ready - total compressed: {}, total decompressed: {}",
            stats.total_compressed(),
            stats.total_decompressed()
        );
    } else {
        error!("Failed to initialize compression service");
    }
}

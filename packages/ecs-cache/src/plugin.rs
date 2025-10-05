use bevy::prelude::*;
use tracing::info;

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::systems::*;

/// Bevy plugin for ECS-based caching using goldylox
pub struct EcsCachePlugin;

impl Plugin for EcsCachePlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing ECS Cache Plugin with goldylox backend");

        // Add resources
        app.init_resource::<CacheManager>()
            .init_resource::<CacheConfig>()
            .init_resource::<CacheMetrics>();

        // Add events
        app.add_event::<CacheReadRequested>()
            .add_event::<CacheWriteRequested>()
            .add_event::<CacheInvalidateRequested>()
            .add_event::<CacheReadCompleted>()
            .add_event::<CacheWriteCompleted>()
            .add_event::<CacheInvalidationCompleted>()
            .add_event::<CacheEvictionOccurred>()
            .add_event::<CacheWarmupRequested>();

        // Add systems
        app.add_systems(
            Update,
            (
                process_cache_reads_system,
                process_cache_writes_system,
                process_cache_invalidations_system,
                cache_eviction_system,
                cache_metrics_system,
            )
                .chain(),
        );

        // Initialize default eviction monitors for each default partition
        app.add_systems(PostStartup, setup_cache_eviction_monitors);

        info!("ECS Cache Plugin initialized successfully");
    }
}

/// Setup system to create eviction monitors for default cache partitions
fn setup_cache_eviction_monitors(mut commands: Commands, cache_manager: Res<CacheManager>) {
    for partition_name in cache_manager.partitions.keys() {
        commands.spawn(CacheEvictionMonitor::new(
            partition_name.clone(),
            0.8, // Trigger eviction at 80% memory usage
        ));

        info!(
            "Created eviction monitor for partition '{}'",
            partition_name
        );
    }
}

// Helper functions for common cache operations
impl EcsCachePlugin {
    /// Helper to create a cache read request
    pub fn create_read_request(
        partition: impl Into<String>,
        key: impl Into<String>,
        requester: impl Into<String>,
    ) -> CacheReadRequested {
        CacheReadRequested::new(partition, key, requester)
    }

    /// Helper to create a cache write request
    pub fn create_write_request(
        partition: impl Into<String>,
        key: impl Into<String>,
        value: Vec<u8>,
        ttl_seconds: Option<u64>,
        requester: impl Into<String>,
    ) -> CacheWriteRequested {
        CacheWriteRequested::new(partition, key, value, ttl_seconds, requester)
    }

    /// Helper to create a cache invalidation request
    pub fn create_invalidate_request(
        partition: impl Into<String>,
        key: impl Into<String>,
        requester: impl Into<String>,
    ) -> CacheInvalidateRequested {
        CacheInvalidateRequested::new(partition, key, requester)
    }
}

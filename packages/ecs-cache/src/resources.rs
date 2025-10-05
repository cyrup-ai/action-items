use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use goldylox::Goldylox;
use serde::{Deserialize, Serialize};

/// Central cache manager - wraps goldylox cache instances
#[derive(Resource)]
pub struct CacheManager {
    /// Multiple cache partitions for different data types
    pub partitions: HashMap<String, Goldylox<String, Vec<u8>>>,

    /// Configuration for each partition
    pub partition_configs: HashMap<String, CachePartitionConfig>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            partitions: HashMap::new(),
            partition_configs: HashMap::new(),
        }
    }

    pub fn create_partition(
        &mut self,
        name: impl Into<String>,
        config: CachePartitionConfig,
    ) -> Result<(), String> {
        let name = name.into();

        // Create goldylox cache using builder pattern with proper configuration mapping
        let cache = Goldylox::<String, Vec<u8>>::builder()
            .hot_tier_max_entries(config.hot_tier_capacity as u32)
            .warm_tier_max_entries(config.warm_tier_capacity)
            .build()
            .map_err(|e| format!("Failed to create cache partition '{}': {:?}", name, e))?;

        self.partitions.insert(name.clone(), cache);
        self.partition_configs.insert(name, config);

        Ok(())
    }

    pub fn get_partition(&self, name: &str) -> Option<&Goldylox<String, Vec<u8>>> {
        self.partitions.get(name)
    }

    pub fn get_partition_mut(&mut self, name: &str) -> Option<&mut Goldylox<String, Vec<u8>>> {
        self.partitions.get_mut(name)
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        let mut manager = Self::new();

        // Create default partitions
        let default_config = CachePartitionConfig::default();

        let _ = manager.create_partition("plugin_metadata", default_config.clone());
        let _ = manager.create_partition("search_results", default_config.clone());
        let _ = manager.create_partition("ui_assets", default_config.clone());
        let _ = manager.create_partition("configuration", default_config.clone());
        let _ = manager.create_partition("api_responses", default_config);

        manager
    }
}

/// Configuration for individual cache partitions
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CachePartitionConfig {
    /// Hot tier capacity (number of entries)
    pub hot_tier_capacity: usize,

    /// Warm tier capacity (number of entries)
    pub warm_tier_capacity: usize,

    /// Default TTL for entries in this partition
    pub default_ttl: Option<Duration>,

    /// Maximum entry size in bytes
    pub max_entry_size: usize,

    /// Enable compression for this partition
    pub enable_compression: bool,
}

impl Default for CachePartitionConfig {
    fn default() -> Self {
        Self {
            hot_tier_capacity: 1000,
            warm_tier_capacity: 10000,
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour
            max_entry_size: 1024 * 1024,                  // 1MB
            enable_compression: true,
        }
    }
}

/// Global cache configuration
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Global memory limit for all cache partitions
    pub global_memory_limit: usize,

    /// Enable cache warming on startup
    pub enable_cache_warming: bool,

    /// Cache warming batch size
    pub warming_batch_size: usize,

    /// Enable cache metrics collection
    pub enable_metrics: bool,

    /// Eviction check interval
    pub eviction_check_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            global_memory_limit: 256 * 1024 * 1024, // 256MB
            enable_cache_warming: true,
            warming_batch_size: 100,
            enable_metrics: true,
            eviction_check_interval: Duration::from_secs(60),
        }
    }
}

/// Cache performance metrics
#[derive(Resource, Debug, Default, Clone)]
pub struct CacheMetrics {
    /// Hit/miss ratios per partition
    pub partition_stats: HashMap<String, CachePartitionStats>,

    /// Global cache statistics
    pub global_stats: GlobalCacheStats,
}

/// Statistics for individual cache partitions
#[derive(Debug, Default, Clone)]
pub struct CachePartitionStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub writes: u64,
    pub total_size: usize,
    pub entry_count: usize,
}

impl CachePartitionStats {
    pub fn hit_ratio(&self) -> f64 {
        if self.hits + self.misses == 0 {
            return 0.0;
        }
        self.hits as f64 / (self.hits + self.misses) as f64
    }
}

/// Global cache system statistics
#[derive(Debug, Default, Clone)]
pub struct GlobalCacheStats {
    pub total_memory_used: usize,
    pub total_entries: usize,
    pub uptime_seconds: u64,
}

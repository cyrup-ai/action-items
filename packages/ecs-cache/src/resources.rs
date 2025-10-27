use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use goldylox::telemetry::unified_stats::UnifiedStats;
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

    pub async fn create_partition(
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
            .await
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
        // Create empty manager - partitions will be created by startup system
        Self::new()
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
#[derive(Debug, Clone)]
pub struct CachePartitionStats {
    // Core metrics (existing)
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub writes: u64,
    pub total_size: usize,
    pub entry_count: usize,
    
    // NEW: Additional goldylox metrics
    pub hot_tier_hits: u64,
    pub warm_tier_hits: u64,
    pub cold_tier_hits: u64,
    pub avg_access_latency_ns: u64,
    pub promotions: u64,
    pub demotions: u64,
    pub peak_memory_usage: usize,
    pub ops_per_second: f32,
    pub last_updated: Instant,
}

impl Default for CachePartitionStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            writes: 0,
            total_size: 0,
            entry_count: 0,
            hot_tier_hits: 0,
            warm_tier_hits: 0,
            cold_tier_hits: 0,
            avg_access_latency_ns: 0,
            promotions: 0,
            demotions: 0,
            peak_memory_usage: 0,
            ops_per_second: 0.0,
            last_updated: Instant::now(),
        }
    }
}

impl CachePartitionStats {
    /// Create from Goldylox UnifiedStats
    ///
    /// CRITICAL: Use `compute_unified_stats()` method, not `get_snapshot()`
    pub fn from_goldylox_stats(stats: &UnifiedStats) -> Self {
        Self {
            hits: stats.hot_tier_hits + stats.warm_tier_hits + stats.cold_tier_hits,
            misses: stats.total_misses,
            evictions: 0, // Goldylox doesn't expose evictions directly
            writes: 0,    // Track separately if needed
            total_size: stats.total_memory_usage as usize,
            entry_count: 0, // Not directly available from stats
            
            // Goldylox-specific metrics
            hot_tier_hits: stats.hot_tier_hits,
            warm_tier_hits: stats.warm_tier_hits,
            cold_tier_hits: stats.cold_tier_hits,
            avg_access_latency_ns: stats.avg_access_latency_ns,
            promotions: stats.promotions_performed,
            demotions: stats.demotions_performed,
            peak_memory_usage: stats.peak_memory_usage as usize,
            ops_per_second: stats.ops_per_second,
            last_updated: Instant::now(),
        }
    }
    
    pub fn hit_ratio(&self) -> f64 {
        if self.hits + self.misses == 0 {
            return 0.0;
        }
        self.hits as f64 / (self.hits + self.misses) as f64
    }
    
    /// Get tier distribution (hot, warm, cold percentages)
    pub fn tier_distribution(&self) -> (f64, f64, f64) {
        let total = self.hot_tier_hits + self.warm_tier_hits + self.cold_tier_hits;
        if total == 0 {
            return (0.0, 0.0, 0.0);
        }
        (
            self.hot_tier_hits as f64 / total as f64,
            self.warm_tier_hits as f64 / total as f64,
            self.cold_tier_hits as f64 / total as f64,
        )
    }
}

/// Global cache system statistics
#[derive(Debug, Default, Clone)]
pub struct GlobalCacheStats {
    pub total_memory_used: usize,
    pub total_entries: usize,
    pub uptime_seconds: u64,
}

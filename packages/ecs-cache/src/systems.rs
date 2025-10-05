use bevy::prelude::*;
use goldylox::prelude::CacheOperationError;
use tracing::{debug, info, warn};

use crate::components::*;
use crate::events::*;
use crate::resources::*;

/// System to process cache read requests
pub fn process_cache_reads_system(
    _commands: Commands,
    cache_manager: ResMut<CacheManager>,
    mut read_events: EventReader<CacheReadRequested>,
    mut completed_events: EventWriter<CacheReadCompleted>,
    mut metrics: ResMut<CacheMetrics>,
) {
    for read_request in read_events.read() {
        let partition_name = &read_request.partition;
        let key = &read_request.key;

        // Get cache partition
        let result = if let Some(cache) = cache_manager.get_partition(partition_name) {
            // Perform synchronous read (goldylox is designed for fast reads)
            match cache.get(key) {
                Some(value) => {
                    debug!("Cache HIT: partition='{}', key='{}'", partition_name, key);

                    // Update metrics
                    if let Some(stats) = metrics.partition_stats.get_mut(partition_name) {
                        stats.hits += 1;
                    }

                    Ok(Some(value.clone()))
                },
                None => {
                    debug!("Cache MISS: partition='{}', key='{}'", partition_name, key);

                    // Update metrics
                    if let Some(stats) = metrics.partition_stats.get_mut(partition_name) {
                        stats.misses += 1;
                    }

                    Ok(None)
                },
            }
        } else {
            warn!("Cache partition not found: '{}'", partition_name);
            Err(CacheOperationError::InvalidArgument(format!(
                "Partition not found: {}",
                partition_name
            )))
        };

        let hit = result.as_ref().map(|r| r.is_some()).unwrap_or(false);

        // Send completion event
        completed_events.write(CacheReadCompleted {
            operation_id: read_request.operation_id,
            partition: read_request.partition.clone(),
            key: read_request.key.clone(),
            result,
            hit,
            requester: read_request.requester.clone(),
        });
    }
}

/// System to process cache write requests
pub fn process_cache_writes_system(
    _commands: Commands,
    mut cache_manager: ResMut<CacheManager>,
    mut write_events: EventReader<CacheWriteRequested>,
    mut completed_events: EventWriter<CacheWriteCompleted>,
    mut metrics: ResMut<CacheMetrics>,
) {
    for write_request in write_events.read() {
        let partition_name = &write_request.partition;
        let key = &write_request.key;
        let value = &write_request.value;

        debug!(
            "Cache WRITE: partition='{}', key='{}', size={} bytes",
            partition_name,
            key,
            value.len()
        );

        // Get cache partition
        let result = if let Some(cache) = cache_manager.get_partition_mut(partition_name) {
            // Perform write operation using goldylox put() method
            // Note: TTL is handled internally by goldylox based on partition configuration
            cache.put(key.clone(), value.clone())
        } else {
            warn!("Cache partition not found: '{}'", partition_name);
            Err(CacheOperationError::InvalidArgument(format!(
                "Partition not found: {}",
                partition_name
            )))
        };

        // Update metrics on successful write
        if result.is_ok()
            && let Some(stats) = metrics.partition_stats.get_mut(partition_name) {
                stats.writes += 1;
                stats.total_size += value.len();
                stats.entry_count += 1;
            }

        // Send completion event
        completed_events.write(CacheWriteCompleted {
            operation_id: write_request.operation_id,
            partition: write_request.partition.clone(),
            key: write_request.key.clone(),
            result,
            requester: write_request.requester.clone(),
        });
    }
}

/// System to process cache invalidation requests
pub fn process_cache_invalidations_system(
    _commands: Commands,
    mut cache_manager: ResMut<CacheManager>,
    mut invalidate_events: EventReader<CacheInvalidateRequested>,
    mut completed_events: EventWriter<CacheInvalidationCompleted>,
    mut eviction_events: EventWriter<CacheEvictionOccurred>,
    mut metrics: ResMut<CacheMetrics>,
) {
    for invalidate_request in invalidate_events.read() {
        let partition_name = &invalidate_request.partition;
        let key = &invalidate_request.key;

        debug!(
            "Cache INVALIDATE: partition='{}', key='{}'",
            partition_name, key
        );

        // Get cache partition
        let result = if let Some(cache) = cache_manager.get_partition_mut(partition_name) {
            // Check if key exists before removal
            let existed = cache.contains_key(key);

            if existed {
                // Get value size before removal for eviction event
                let value_size = cache.get(key).map(|v| v.len()).unwrap_or(0);

                // Remove the entry
                cache.remove(key);

                // Send eviction event
                eviction_events.write(CacheEvictionOccurred {
                    partition: partition_name.clone(),
                    key: key.clone(),
                    reason: EvictionReason::ManualInvalidation,
                    value_size,
                });

                // Update metrics
                if let Some(stats) = metrics.partition_stats.get_mut(partition_name) {
                    stats.evictions += 1;
                    stats.total_size = stats.total_size.saturating_sub(value_size);
                    stats.entry_count = stats.entry_count.saturating_sub(1);
                }
            }

            Ok(existed)
        } else {
            warn!("Cache partition not found: '{}'", partition_name);
            Err(CacheOperationError::InvalidArgument(format!(
                "Partition not found: {}",
                partition_name
            )))
        };

        // Send completion event
        completed_events.write(CacheInvalidationCompleted {
            operation_id: invalidate_request.operation_id,
            partition: invalidate_request.partition.clone(),
            key: invalidate_request.key.clone(),
            result,
            requester: invalidate_request.requester.clone(),
        });
    }
}

/// System to handle cache eviction based on memory pressure and TTL
pub fn cache_eviction_system(
    mut cache_manager: ResMut<CacheManager>,
    cache_config: Res<CacheConfig>,
    _eviction_events: EventWriter<CacheEvictionOccurred>,
    metrics: ResMut<CacheMetrics>,
    mut eviction_monitors: Query<&mut CacheEvictionMonitor>,
) {
    for mut monitor in &mut eviction_monitors {
        if !monitor.should_check(cache_config.eviction_check_interval) {
            continue;
        }

        monitor.last_check = std::time::Instant::now();

        if let Some(_cache) = cache_manager.get_partition_mut(&monitor.partition) {
            // Check if eviction is needed based on memory pressure
            // Note: This is a simplified check - goldylox handles most eviction internally
            let stats = metrics.partition_stats.get(&monitor.partition);

            if let Some(stats) = stats {
                let memory_usage_ratio =
                    stats.total_size as f32 / cache_config.global_memory_limit as f32;

                if memory_usage_ratio > monitor.eviction_threshold {
                    info!(
                        "Memory pressure detected for partition '{}': {:.2}% usage",
                        monitor.partition,
                        memory_usage_ratio * 100.0
                    );

                    // Let goldylox handle the eviction internally
                    // We just log the event here
                }
            }
        }
    }
}

/// System to update cache metrics
pub fn cache_metrics_system(
    cache_manager: Res<CacheManager>,
    mut metrics: ResMut<CacheMetrics>,
    time: Res<Time>,
) {
    // Update global stats
    metrics.global_stats.uptime_seconds += time.delta().as_secs();

    let mut total_memory = 0;
    let mut total_entries = 0;

    // Update per-partition stats
    for partition_name in cache_manager.partitions.keys() {
        if !metrics.partition_stats.contains_key(partition_name) {
            metrics
                .partition_stats
                .insert(partition_name.clone(), CachePartitionStats::default());
        }

        if let Some(stats) = metrics.partition_stats.get_mut(partition_name) {
            // Note: These would ideally come from goldylox cache metrics
            // For now, we maintain our own counters
            total_memory += stats.total_size;
            total_entries += stats.entry_count;
        }
    }

    metrics.global_stats.total_memory_used = total_memory;
    metrics.global_stats.total_entries = total_entries;
}

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, block_on};
use bevy_tokio_tasks::TokioTasksRuntime;
use goldylox::prelude::CacheOperationError;
use goldylox::Goldylox;
use tracing::{debug, error, info, warn};

use crate::components::*;
use crate::events::*;
use crate::resources::*;

// ============================================================================
// Startup Systems
// ============================================================================

/// Initialize default cache partitions using shared Tokio runtime
///
/// Spawns background tasks in the shared TokioTasksRuntime to initialize goldylox instances.
/// Once initialized, partitions are inserted into CacheManager on the main thread.
pub fn initialize_cache_partitions_system(
    tokio_runtime: Res<TokioTasksRuntime>,
) {
    info!("Initializing default cache partitions using shared Tokio runtime");

    // Create all default partitions
    let partition_configs = vec![
        ("plugin_metadata", CachePartitionConfig::default()),
        ("search_results", CachePartitionConfig::default()),
        ("ui_assets", CachePartitionConfig::default()),
        ("configuration", CachePartitionConfig::default()),
        ("api_responses", CachePartitionConfig::default()),
    ];

    for (partition_name, config) in partition_configs {
        let name = partition_name.to_string();
        info!("Spawning background task to initialize cache partition: {}", name);

        // Spawn background task in shared Tokio runtime
        tokio_runtime.spawn_background_task(|mut ctx| async move {
            info!("Building goldylox cache partition '{}' with hot_tier={}, warm_tier={}",
                  name, config.hot_tier_capacity, config.warm_tier_capacity);

            // Build goldylox cache in Tokio runtime context
            match Goldylox::<String, Vec<u8>>::builder()
                .hot_tier_max_entries(config.hot_tier_capacity as u32)
                .warm_tier_max_entries(config.warm_tier_capacity)
                .build()
                .await
            {
                Ok(cache) => {
                    info!("Successfully built cache partition '{}'", name);
                    let partition_name = name.clone();

                    // Insert into CacheManager on main thread
                    ctx.run_on_main_thread(move |ctx| {
                        if let Some(mut cache_manager) = ctx.world.get_resource_mut::<CacheManager>() {
                            cache_manager.insert_partition(partition_name.clone(), cache, config);
                            info!("Cache partition '{}' registered in CacheManager", partition_name);
                        } else {
                            warn!("CacheManager resource not found when trying to insert partition '{}'", partition_name);
                        }
                    }).await;
                }
                Err(e) => {
                    error!("Failed to build cache partition '{}': {:?}", name, e);
                }
            }
        });
    }
}

// ============================================================================
// Cache Read Operations
// ============================================================================

/// System to process cache read requests by spawning async tasks
pub fn process_cache_reads_system(
    mut commands: Commands,
    cache_manager: Res<CacheManager>,
    mut read_events: EventReader<CacheReadRequested>,
    _metrics: ResMut<CacheMetrics>,
) {
    for read_request in read_events.read() {
        let partition_name = read_request.partition.clone();
        let partition_name_task = partition_name.clone();
        let key = read_request.key.clone();
        let key_task = key.clone();
        let operation_id = read_request.operation_id;
        let requester = read_request.requester.clone();
        let requester_task = requester.clone();
        
        // Get cache partition (clone the Goldylox handle which is cheap - Arc internally)
        let cache_opt = cache_manager.get_partition(&partition_name).cloned();
        
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();
            
            let (result, hit) = if let Some(cache) = cache_opt {
                // Perform async read
                match cache.get(&key_task).await {
                    Some(value) => {
                        debug!("Cache HIT: partition='{}', key='{}'", partition_name_task, key_task);
                        (Ok(Some(value)), true)
                    }
                    None => {
                        debug!("Cache MISS: partition='{}', key='{}'", partition_name_task, key_task);
                        (Ok(None), false)
                    }
                }
            } else {
                warn!("Cache partition not found: '{}'", partition_name_task);
                (Err(CacheOperationError::InvalidArgument(format!(
                    "Partition not found: {}",
                    partition_name_task
                ))), false)
            };
            
            // Emit completion event
            command_queue.push(move |world: &mut World| {
                world.send_event(CacheReadCompleted {
                    operation_id,
                    partition: partition_name_task,
                    key: key_task,
                    result,
                    hit,
                    requester: requester_task,
                });
            });
            
            command_queue
        });
        
        commands.spawn(CacheReadTask {
            operation_id,
            partition: partition_name,
            key,
            requester,
            task,
        });
    }
}

/// Poll cache read tasks for completion
pub fn handle_cache_read_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut CacheReadTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Cache Write Operations
// ============================================================================

/// System to process cache write requests by spawning async tasks
pub fn process_cache_writes_system(
    mut commands: Commands,
    cache_manager: Res<CacheManager>,
    mut write_events: EventReader<CacheWriteRequested>,
) {
    for write_request in write_events.read() {
        let partition_name = write_request.partition.clone();
        let partition_name_task = partition_name.clone();
        let key = write_request.key.clone();
        let key_task = key.clone();
        let value = write_request.value.clone();
        let operation_id = write_request.operation_id;
        let requester = write_request.requester.clone();
        let requester_task = requester.clone();
        
        debug!(
            "Cache WRITE: partition='{}', key='{}', size={} bytes",
            partition_name,
            key,
            value.len()
        );
        
        // Get cache partition (clone the Goldylox handle)
        let cache_opt = cache_manager.get_partition(&partition_name).cloned();
        let _value_len = value.len();
        
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();
            
            let result = if let Some(cache) = cache_opt {
                // Perform async write
                cache.put(key_task.clone(), value).await
            } else {
                warn!("Cache partition not found: '{}'", partition_name_task);
                Err(CacheOperationError::InvalidArgument(format!(
                    "Partition not found: {}",
                    partition_name_task
                )))
            };
            
            // Emit completion event
            command_queue.push(move |world: &mut World| {
                world.send_event(CacheWriteCompleted {
                    operation_id,
                    partition: partition_name_task,
                    key: key_task,
                    result,
                    requester: requester_task,
                });
            });
            
            command_queue
        });
        
        commands.spawn(CacheWriteTask {
            operation_id,
            partition: partition_name,
            key,
            requester,
            task,
        });
    }
}

/// Poll cache write tasks for completion
pub fn handle_cache_write_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut CacheWriteTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Cache Invalidation Operations
// ============================================================================

/// System to process cache invalidation requests by spawning async tasks
pub fn process_cache_invalidations_system(
    mut commands: Commands,
    cache_manager: Res<CacheManager>,
    mut invalidate_events: EventReader<CacheInvalidateRequested>,
) {
    for invalidate_request in invalidate_events.read() {
        let partition_name = invalidate_request.partition.clone();
        let partition_name_task = partition_name.clone();
        let key = invalidate_request.key.clone();
        let key_task = key.clone();
        let operation_id = invalidate_request.operation_id;
        let requester = invalidate_request.requester.clone();
        let requester_task = requester.clone();
        
        debug!(
            "Cache INVALIDATE: partition='{}', key='{}'",
            partition_name, key
        );
        
        // Get cache partition (clone the Goldylox handle)
        let cache_opt = cache_manager.get_partition(&partition_name).cloned();
        
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();
            
            let result = if let Some(cache) = cache_opt {
                // Check if key exists and get size before removal
                let existed = cache.contains_key(&key_task).await;
                let value_size = if existed {
                    cache.get(&key_task).await.map(|v| v.len()).unwrap_or(0)
                } else {
                    0
                };
                
                if existed {
                    // Remove the entry
                    cache.remove(&key_task).await;
                    
                    let partition_for_eviction = partition_name_task.clone();
                    let key_for_eviction = key_task.clone();
                    // Emit eviction event
                    command_queue.push(move |world: &mut World| {
                        world.send_event(CacheEvictionOccurred {
                            partition: partition_for_eviction,
                            key: key_for_eviction,
                            reason: EvictionReason::ManualInvalidation,
                            value_size,
                        });
                    });
                }
                
                Ok(existed)
            } else {
                warn!("Cache partition not found: '{}'", partition_name_task);
                Err(CacheOperationError::InvalidArgument(format!(
                    "Partition not found: {}",
                    partition_name_task
                )))
            };
            
            // Emit completion event
            command_queue.push(move |world: &mut World| {
                world.send_event(CacheInvalidationCompleted {
                    operation_id,
                    partition: partition_name_task,
                    key: key_task,
                    result,
                    requester: requester_task,
                });
            });
            
            command_queue
        });
        
        commands.spawn(CacheInvalidateTask {
            operation_id,
            partition: partition_name,
            key,
            requester,
            task,
        });
    }
}

/// Poll cache invalidation tasks for completion
pub fn handle_cache_invalidation_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut CacheInvalidateTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Cache Maintenance Systems
// ============================================================================

/// System to handle cache eviction based on memory pressure and TTL
pub fn cache_eviction_system(
    cache_manager: Res<CacheManager>,
    cache_config: Res<CacheConfig>,
    metrics: Res<CacheMetrics>,
    mut eviction_monitors: Query<&mut CacheEvictionMonitor>,
) {
    for mut monitor in &mut eviction_monitors {
        if !monitor.should_check(cache_config.eviction_check_interval) {
            continue;
        }

        monitor.last_check = std::time::Instant::now();

        if cache_manager.get_partition(&monitor.partition).is_some() {
            // Check if eviction is needed based on memory pressure
            // Note: goldylox handles most eviction internally
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

                    // Goldylox handles eviction internally
                }
            }
        }
    }
}

/// System to collect cache metrics from goldylox partitions
pub fn cache_metrics_system(
    cache_manager: Res<CacheManager>,
    mut metrics: ResMut<CacheMetrics>,
    time: Res<Time>,
    mut last_log_time: Local<f32>,
) {
    // Update global uptime
    metrics.global_stats.uptime_seconds += time.delta().as_secs();

    let mut total_memory = 0;
    let mut total_entries = 0;
    let mut total_hits = 0;
    let mut total_misses = 0;

    // Collect metrics from each goldylox partition
    for (partition_name, goldylox_cache) in cache_manager.partitions.iter() {
        // Get unified stats from goldylox (atomic, accurate, comprehensive)
        let unified_stats_ref = goldylox_cache.get_unified_stats();

        // Compute snapshot of current stats (CRITICAL: use compute_unified_stats, not get_snapshot)
        let unified_stats = unified_stats_ref.compute_unified_stats();

        // Convert to CachePartitionStats
        let partition_stats = CachePartitionStats::from_goldylox_stats(&unified_stats);

        // Update aggregates
        total_memory += partition_stats.total_size;
        total_entries += partition_stats.entry_count;
        total_hits += partition_stats.hits;
        total_misses += partition_stats.misses;

        // Store partition-specific stats
        metrics.partition_stats.insert(partition_name.clone(), partition_stats);
    }

    // Update global aggregated stats
    metrics.global_stats.total_memory_used = total_memory;
    metrics.global_stats.total_entries = total_entries;

    // Optional: log summary periodically (every 30 seconds)
    *last_log_time += time.delta_secs();
    if *last_log_time >= 30.0 {
        let overall_hit_rate = if total_hits + total_misses > 0 {
            total_hits as f64 / (total_hits + total_misses) as f64
        } else {
            0.0
        };

        debug!(
            "Cache metrics: partitions={}, hit_rate={:.2}%, memory={}KB, entries={}",
            cache_manager.partitions.len(),
            overall_hit_rate * 100.0,
            total_memory / 1024,
            total_entries,
        );

        *last_log_time = 0.0; // Reset timer
    }
}

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, block_on};
use goldylox::prelude::CacheOperationError;
use tracing::{debug, info, warn};

use crate::components::*;
use crate::events::*;
use crate::resources::*;

// ============================================================================
// Startup Systems
// ============================================================================

/// Initialize default cache partitions asynchronously
pub fn initialize_cache_partitions_system(mut commands: Commands) {
    let task_pool = AsyncComputeTaskPool::get();
    
    let task = task_pool.spawn(async move {
        let mut command_queue = CommandQueue::default();
        
        // Create all default partitions
        let partition_names = vec![
            "plugin_metadata",
            "search_results",
            "ui_assets",
            "configuration",
            "api_responses",
        ];
        
        command_queue.push(move |world: &mut World| {
            let _manager = world.resource_mut::<CacheManager>();
            let default_config = CachePartitionConfig::default();
            
            // Create futures for all partitions
            let task_pool = AsyncComputeTaskPool::get();
            let mut tasks = Vec::new();
            
            for partition_name in partition_names {
                let config = default_config.clone();
                let name = partition_name.to_string();
                
                let task = task_pool.spawn(async move {
                    goldylox::Goldylox::<String, Vec<u8>>::builder()
                        .hot_tier_max_entries(config.hot_tier_capacity as u32)
                        .warm_tier_max_entries(config.warm_tier_capacity)
                        .build()
                        .await
                        .map(|cache| (name.clone(), cache, config))
                        .map_err(|e| format!("Failed to create partition '{}': {:?}", name, e))
                });
                
                tasks.push((partition_name, task));
            }
            
            // Store tasks as components for polling
            for (name, task) in tasks {
                world.spawn(CachePartitionInitTask {
                    partition_name: name.to_string(),
                    task,
                });
            }
        });
        
        command_queue
    });
    
    commands.spawn(PartitionInitTask(task));
}

/// Component for partition initialization
#[derive(Component)]
pub struct CachePartitionInitTask {
    pub partition_name: String,
    pub task: bevy::tasks::Task<Result<(String, goldylox::Goldylox<String, Vec<u8>>, CachePartitionConfig), String>>,
}

/// Poll partition initialization tasks
pub fn handle_partition_init_tasks(
    mut commands: Commands,
    mut cache_manager: ResMut<CacheManager>,
    mut tasks: Query<(Entity, &mut CachePartitionInitTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.task)) {
            match result {
                Ok((name, cache, config)) => {
                    info!("Initialized cache partition: {}", name);
                    cache_manager.partitions.insert(name.clone(), cache);
                    cache_manager.partition_configs.insert(name, config);
                }
                Err(e) => {
                    warn!("Failed to initialize partition: {}", e);
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

/// Poll main initialization task
pub fn handle_partition_init_system(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut PartitionInitTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
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
            
            // Update metrics
            command_queue.push(move |world: &mut World| {
                let mut metrics = world.resource_mut::<CacheMetrics>();
                if let Some(stats) = metrics.partition_stats.get_mut(&partition_name_task) {
                    if hit {
                        stats.hits += 1;
                    } else {
                        stats.misses += 1;
                    }
                }
                
                // Emit completion event
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
        let value_len = value.len();
        
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
            
            // Update metrics and emit completion event
            command_queue.push(move |world: &mut World| {
                if result.is_ok() {
                    let mut metrics = world.resource_mut::<CacheMetrics>();
                    if let Some(stats) = metrics.partition_stats.get_mut(&partition_name_task) {
                        stats.writes += 1;
                        stats.total_size += value_len;
                        stats.entry_count += 1;
                    }
                }
                
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
                    // Emit eviction event and update metrics
                    command_queue.push(move |world: &mut World| {
                        world.send_event(CacheEvictionOccurred {
                            partition: partition_for_eviction.clone(),
                            key: key_for_eviction,
                            reason: EvictionReason::ManualInvalidation,
                            value_size,
                        });
                        
                        let mut metrics = world.resource_mut::<CacheMetrics>();
                        if let Some(stats) = metrics.partition_stats.get_mut(&partition_for_eviction) {
                            stats.evictions += 1;
                            stats.total_size = stats.total_size.saturating_sub(value_size);
                            stats.entry_count = stats.entry_count.saturating_sub(1);
                        }
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
            // Note: These counters are maintained from operations
            total_memory += stats.total_size;
            total_entries += stats.entry_count;
        }
    }

    metrics.global_stats.total_memory_used = total_memory;
    metrics.global_stats.total_entries = total_entries;
}

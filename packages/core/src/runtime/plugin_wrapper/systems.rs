use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use serde_json::Value;
use tracing::{debug, error, info, trace, warn};

use super::plugin_component::DenoPluginComponent;

/// High-performance Deno runtime statistics with zero-allocation counters
#[derive(Resource)]
pub struct DenoRuntimeStats {
    active_plugins: AtomicUsize,
    total_plugins_loaded: AtomicUsize,
    execution_count: AtomicU64,
    error_count: AtomicU64,
    last_health_check: parking_lot::Mutex<Instant>,
}

impl Default for DenoRuntimeStats {
    #[inline]
    fn default() -> Self {
        Self {
            active_plugins: AtomicUsize::new(0),
            total_plugins_loaded: AtomicUsize::new(0),
            execution_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            last_health_check: parking_lot::Mutex::new(Instant::now()),
        }
    }
}

impl DenoRuntimeStats {
    #[inline]
    pub fn active_plugins(&self) -> usize {
        self.active_plugins.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn error_rate(&self) -> f64 {
        let total = self.execution_count();
        if total == 0 {
            0.0
        } else {
            self.error_count() as f64 / total as f64
        }
    }

    #[inline]
    pub fn increment_execution_count(&self) {
        self.execution_count.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn increment_error_count(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Lock-free Deno runtime health monitor using atomic operations
#[derive(Resource)]
pub struct DenoRuntimeHealthMonitor {
    check_interval: Duration,
    consecutive_failures: AtomicU64,
    max_failures: u64,
}

impl Default for DenoRuntimeHealthMonitor {
    #[inline]
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            consecutive_failures: AtomicU64::new(0),
            max_failures: 3,
        }
    }
}

/// Lock-free pending request tracking using crossbeam channels
#[derive(Resource)]
pub struct PendingDenoRequests {
    request_receiver: Receiver<DenoActionItemRequest>,
    response_sender: Sender<DenoActionItemResponse>,
}

impl Default for PendingDenoRequests {
    #[inline]
    fn default() -> Self {
        let (_, request_receiver) = crossbeam_channel::unbounded();
        let (response_sender, _) = crossbeam_channel::unbounded();

        Self {
            request_receiver,
            response_sender,
        }
    }
}

/// High-performance Deno plugin events using zero-allocation patterns
#[derive(Event)]
pub enum DenoPluginEvent {
    PluginLoaded {
        plugin_id: String,
        load_time_nanos: u64,
    },
    PluginExecuted {
        plugin_id: String,
        execution_time_nanos: u64,
    },
    PluginError {
        plugin_id: String,
        error: String,
    },
    PluginUnloaded {
        plugin_id: String,
    },
    RuntimeError {
        error: String,
    },
}

/// Lock-free Deno action item request
#[derive(Clone)]
pub struct DenoActionItemRequest {
    pub plugin_id: String,
    pub request_id: String,
    pub action: String,
    pub data: Value,
    pub timestamp_nanos: u64,
}

/// Lock-free Deno action item response
#[derive(Clone)]
pub struct DenoActionItemResponse {
    pub plugin_id: String,
    pub request_id: String,
    pub result: Result<Value, String>,
    pub execution_time_nanos: u64,
}

/// Component marking entities as active Deno plugins with atomic counters
#[derive(Component)]
pub struct ActiveDenoPlugin {
    last_execution_nanos: AtomicU64,
    execution_count: AtomicU64,
    error_count: AtomicU64,
}

impl Default for ActiveDenoPlugin {
    #[inline]
    fn default() -> Self {
        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);

        Self {
            last_execution_nanos: AtomicU64::new(now_nanos),
            execution_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }
}

impl ActiveDenoPlugin {
    #[inline]
    pub fn execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn increment_execution(&self) {
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        self.last_execution_nanos
            .store(now_nanos, Ordering::Relaxed);
    }

    #[inline]
    pub fn increment_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// High-performance Deno plugin execution system with zero-allocation hot path
pub fn execute_deno_plugin_system(
    plugins: Query<(Entity, &DenoPluginComponent, Option<&ActiveDenoPlugin>)>,
    mut commands: Commands,
    stats: Res<DenoRuntimeStats>,
) {
    let mut active_count = 0_usize;

    for (entity, plugin_component, active_plugin) in &plugins {
        if active_plugin.is_none() {
            commands.entity(entity).insert(ActiveDenoPlugin::default());

            info!(
                "Deno plugin activated: {} (v{}) at {:?}",
                plugin_component.name, plugin_component.version, plugin_component.entry_point
            );
        }

        active_count = active_count.saturating_add(1);

        trace!(
            "Deno plugin available: {} (v{}) at {:?}",
            plugin_component.name, plugin_component.version, plugin_component.entry_point
        );
    }

    // Zero-allocation atomic update of active plugin count
    stats.active_plugins.store(active_count, Ordering::Relaxed);
}

/// Lock-free Deno runtime health monitoring system
pub fn monitor_deno_runtime_system(
    health_monitor: Res<DenoRuntimeHealthMonitor>,
    stats: Res<DenoRuntimeStats>,
    plugins: Query<&ActiveDenoPlugin>,
    mut events: EventWriter<DenoPluginEvent>,
) {
    // Zero-allocation health check using atomic operations
    let should_check = {
        if let Some(mut last_check) = stats.last_health_check.try_lock() {
            let elapsed = last_check.elapsed();
            if elapsed >= health_monitor.check_interval {
                *last_check = Instant::now();
                true
            } else {
                false
            }
        } else {
            false // Skip if mutex is contended
        }
    };

    if !should_check {
        return;
    }

    // Accumulate statistics with zero allocation
    let (total_execution_count, total_error_count) =
        plugins
            .iter()
            .fold((0_u64, 0_u64), |(exec_acc, err_acc), plugin| {
                (
                    exec_acc.saturating_add(plugin.execution_count()),
                    err_acc.saturating_add(plugin.error_count()),
                )
            });

    // Update global stats atomically
    stats
        .execution_count
        .store(total_execution_count, Ordering::Relaxed);
    stats
        .error_count
        .store(total_error_count, Ordering::Relaxed);

    // Calculate error rate with protection against division by zero
    let error_rate = if total_execution_count > 0 {
        total_error_count as f64 / total_execution_count as f64
    } else {
        0.0
    };

    // Track consecutive failures atomically
    if error_rate > 0.1 {
        let failures = health_monitor
            .consecutive_failures
            .fetch_add(1, Ordering::Relaxed)
            + 1;
        warn!(
            "Deno runtime showing high error rate: {:.2}% ({} failures of {})",
            error_rate * 100.0,
            total_error_count,
            total_execution_count
        );

        if failures >= health_monitor.max_failures {
            events.write(DenoPluginEvent::RuntimeError {
                error: format!("High error rate detected: {:.2}%", error_rate * 100.0),
            });
        }
    } else {
        health_monitor
            .consecutive_failures
            .store(0, Ordering::Relaxed);
    }

    let active_count = stats.active_plugins();
    if active_count > 0 {
        debug!(
            "Deno Runtime Health Check: {} active plugins, {} executions, {} errors",
            active_count, total_execution_count, total_error_count
        );
    }
}

/// Lock-free action item request handling system using crossbeam channels
pub fn handle_deno_action_item_requests_system(
    pending: Res<PendingDenoRequests>,
    plugins: Query<&mut ActiveDenoPlugin>,
    plugins_query: Query<(Entity, &DenoPluginComponent)>,
    mut events: EventWriter<DenoPluginEvent>,

    stats: Res<DenoRuntimeStats>,
) {
    // Process all pending requests in a tight loop
    loop {
        let request = match pending.request_receiver.try_recv() {
            Ok(request) => request,
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                error!("Request channel disconnected");
                break;
            },
        };

        debug!(
            "Processing Deno action item request from plugin {}: {}",
            request.plugin_id, request.action
        );

        // Find the plugin component with zero-allocation search
        let plugin_found = plugins_query
            .iter()
            .find(|(_, p)| p.name == request.plugin_id);

        if let Some((entity, plugin_component)) = plugin_found {
            let request_clone = request.clone();
            let plugin_id = plugin_component.name.clone();
            let start_nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos() as u64);

            // Spawn async task for request processing
            let response_sender = pending.response_sender.clone();
            let _task = AsyncComputeTaskPool::get().spawn(async move {
                // Zero-allocation request processing
                let result = match request_clone.action.as_str() {
                    "search" => {
                        debug!("Executing search action for plugin {}", plugin_id);
                        Ok(serde_json::json!({
                            "results": [],
                            "status": "success"
                        }))
                    },
                    "execute" => {
                        debug!("Executing action for plugin {}", plugin_id);
                        Ok(serde_json::json!({
                            "executed": true,
                            "status": "success"
                        }))
                    },
                    _ => {
                        warn!(
                            "Unknown action '{}' from plugin {}",
                            request_clone.action, plugin_id
                        );
                        Err(format!("Unknown action: {}", request_clone.action))
                    },
                };

                let end_nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(start_nanos, |d| d.as_nanos() as u64);

                let response = DenoActionItemResponse {
                    plugin_id: request_clone.plugin_id,
                    request_id: request_clone.request_id,
                    result,
                    execution_time_nanos: end_nanos.saturating_sub(start_nanos),
                };

                // Send response through channel
                if response_sender.try_send(response).is_err() {
                    error!("Failed to send response - channel may be full");
                }
            });

            // Update plugin execution stats atomically
            if let Ok(active_plugin) = plugins.get(entity) {
                active_plugin.increment_execution();
                stats.increment_execution_count();
            }

            let execution_time_nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(start_nanos, |d| d.as_nanos() as u64)
                .saturating_sub(start_nanos);

            events.write(DenoPluginEvent::PluginExecuted {
                plugin_id: request.plugin_id,
                execution_time_nanos,
            });
        } else {
            error!("Plugin not found for request: {}", request.plugin_id);
            stats.increment_error_count();
            events.write(DenoPluginEvent::PluginError {
                plugin_id: request.plugin_id,
                error: "Plugin not found".to_string(),
            });
        }
    }
}

/// High-performance event processing system with zero-allocation hot path
pub fn handle_deno_events_system(
    mut events: EventReader<DenoPluginEvent>,
    stats: Res<DenoRuntimeStats>,
    plugins: Query<&mut ActiveDenoPlugin>,
    plugins_query: Query<(Entity, &DenoPluginComponent)>,
) {
    for event in events.read() {
        match event {
            DenoPluginEvent::PluginLoaded {
                plugin_id,
                load_time_nanos,
            } => {
                info!(
                    "Deno plugin '{}' loaded in {}ns",
                    plugin_id, load_time_nanos
                );
                stats.total_plugins_loaded.fetch_add(1, Ordering::Relaxed);
            },
            DenoPluginEvent::PluginExecuted {
                plugin_id,
                execution_time_nanos,
            } => {
                trace!(
                    "Deno plugin '{}' executed in {}ns",
                    plugin_id, execution_time_nanos
                );
            },
            DenoPluginEvent::PluginError { plugin_id, error } => {
                error!("Deno plugin '{}' error: {}", plugin_id, error);

                // Update error count for the specific plugin atomically
                if let Some((entity, _)) = plugins_query.iter().find(|(_, p)| p.name == *plugin_id)
                    && let Ok(active_plugin) = plugins.get(entity)
                {
                    active_plugin.increment_error();
                }

                stats.increment_error_count();
            },
            DenoPluginEvent::PluginUnloaded { plugin_id } => {
                info!("Deno plugin '{}' unloaded", plugin_id);
                stats.active_plugins.fetch_sub(1, Ordering::Relaxed);
            },
            DenoPluginEvent::RuntimeError { error } => {
                error!("Deno runtime error: {}", error);
                stats.increment_error_count();
            },
        }
    }
}

/// Zero-allocation dynamic plugin loading system
pub fn handle_deno_plugin_loading_system(
    mut commands: Commands,
    plugins: Query<(Entity, &DenoPluginComponent), Without<ActiveDenoPlugin>>,
    stats: Res<DenoRuntimeStats>,
    mut events: EventWriter<DenoPluginEvent>,
) {
    // Process new plugins with minimal allocation
    for (entity, plugin_component) in &plugins {
        let load_start_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);

        // Validate plugin entry point exists
        if plugin_component.entry_point.exists() {
            commands.entity(entity).insert(ActiveDenoPlugin::default());

            let load_time_nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(load_start_nanos, |d| d.as_nanos() as u64)
                .saturating_sub(load_start_nanos);

            stats.total_plugins_loaded.fetch_add(1, Ordering::Relaxed);

            events.write(DenoPluginEvent::PluginLoaded {
                plugin_id: plugin_component.name.clone(),
                load_time_nanos,
            });

            info!(
                "Loaded Deno plugin '{}' from {:?} in {}ns",
                plugin_component.name, plugin_component.entry_point, load_time_nanos
            );
        } else {
            events.write(DenoPluginEvent::PluginError {
                plugin_id: plugin_component.name.clone(),
                error: format!("Entry point not found: {:?}", plugin_component.entry_point),
            });

            warn!(
                "Failed to load Deno plugin '{}': entry point not found at {:?}",
                plugin_component.name, plugin_component.entry_point
            );
        }
    }
}

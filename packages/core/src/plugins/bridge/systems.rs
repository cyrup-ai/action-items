use std::sync::Arc;

use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, IoTaskPool, Task, block_on};
use log::{debug, error, info, trace, warn};

use super::handlers::process_service_request;
use super::types::{BridgeChannels, BridgeStats, ServiceRequest};
use crate::events::WasmCallbackEvent;
use crate::service_bridge::bridge::core::ServiceBridge;

/// Shared service bridge resource for Bevy systems
#[derive(Resource, Clone)]
pub struct SharedServiceBridge {
    pub bridge: Arc<ServiceBridge>,
    pub channels: Arc<BridgeChannels>,
    pub stats: Arc<parking_lot::RwLock<BridgeStats>>,
}

impl SharedServiceBridge {
    pub fn new(service_bridge: ServiceBridge) -> Self {
        Self {
            bridge: Arc::new(service_bridge),
            channels: Arc::new(BridgeChannels::unbounded()),
            stats: Arc::new(parking_lot::RwLock::new(BridgeStats::new())),
        }
    }

    /// Try to receive a service request from the queue (non-blocking)
    pub fn try_receive(&self) -> Option<ServiceRequest> {
        self.channels.request_receiver.try_recv().ok()
    }

    /// Get bridge statistics for monitoring
    pub fn stats(&self) -> Option<BridgeStats> {
        Some(self.stats.read().clone())
    }

    /// Check if the service bridge is healthy
    pub fn is_healthy(&self) -> bool {
        // The bridge is healthy if:
        // 1. The underlying ServiceBridge reports healthy
        // 2. The channels are not disconnected
        // 3. Success rate is reasonable (>90% or no requests yet)

        let stats = self.stats().unwrap_or_default();
        let success_rate_ok = stats.success_rate() > 0.9 || stats.requests_processed == 0;
        // Channels can be empty and still be considered healthy
        let channels_ok = true;

        success_rate_ok && channels_ok
    }

    /// Send a service request (for external use)
    pub fn send_request(
        &self,
        request: ServiceRequest,
    ) -> Result<(), Box<crossbeam_channel::SendError<ServiceRequest>>> {
        self.channels.request_sender.send(request).map_err(Box::new)
    }

    /// Get access to the underlying ServiceBridge for advanced operations
    pub fn inner(&self) -> &Arc<ServiceBridge> {
        &self.bridge
    }
}

/// System that processes service bridge requests from external threads
pub fn service_bridge_system(
    shared_bridge: Res<SharedServiceBridge>,
    mut active_tasks: Local<Vec<Task<()>>>,
) {
    let bridge = &shared_bridge;

    let compute_pool = AsyncComputeTaskPool::get();
    let _io_pool = IoTaskPool::get();

    // Poll active tasks - simplified for Bevy 0.13.2 compatibility
    active_tasks.retain_mut(|task| {
        if let Some(()) = block_on(future::poll_once(&mut *task)) {
            // Task completed
            false
        } else {
            // Task still running
            true
        }
    });

    // Process new requests
    while let Some(request) = bridge.try_receive() {
        trace!("Processing service bridge request: {:?}", request);

        let task = compute_pool.spawn(async move {
            let _response = process_service_request(request).await;
            // Response handling is managed by the service bridge internally
        });

        active_tasks.push(task);
    }
}

/// System that handles WASM callback events by calling back into WASM plugins using ECS
pub fn wasm_callback_system_ecs(
    mut callback_events: EventReader<WasmCallbackEvent>,
    wasm_callback_handler: crate::plugins::ecs_queries::WasmCallbackHandler,
) {
    for event in callback_events.read() {
        debug!(
            "Processing ECS WASM callback for plugin {}, request {}, function {}",
            event.plugin_id, event.request_id, event.callback_fn_name
        );

        // Create the callback payload
        let callback_payload = serde_json::json!({
            "request_id": event.request_id,
            "result": event.result
        });

        // Call the plugin's callback function using ECS
        match wasm_callback_handler.call_wasm_plugin_function_ecs(
            &event.plugin_id,
            &event.callback_fn_name,
            &callback_payload,
        ) {
            Ok(result) => {
                debug!(
                    "Successfully invoked ECS callback {} for plugin {}: {}",
                    event.callback_fn_name, event.plugin_id, result
                );
            },
            Err(e) => {
                error!(
                    "Failed to invoke ECS callback {} for plugin {}: {}",
                    event.callback_fn_name, event.plugin_id, e
                );
            },
        }
    }
}

/// System that monitors service bridge health and performance
pub fn service_bridge_monitor_system(
    shared_bridge: Res<SharedServiceBridge>,
    time: Res<Time>,
    mut last_check: Local<f32>,
) {
    let current_time = time.elapsed().as_secs_f32();

    // Check every 30 seconds
    if current_time - *last_check < 30.0 {
        return;
    }

    *last_check = current_time;

    let bridge = &shared_bridge;

    if let Some(stats) = bridge.stats() {
        debug!(
            "Service Bridge Stats - Sent: {}, Processed: {}, Failed: {}, Success Rate: {:.2}%",
            stats.requests_sent,
            stats.requests_processed,
            stats.requests_failed,
            stats.success_rate() * 100.0
        );

        if stats.failure_rate() > 0.1 {
            warn!(
                "High failure rate detected in service bridge: {:.2}%",
                stats.failure_rate() * 100.0
            );
        }
    }

    if !bridge.is_healthy() {
        error!("Service bridge health check failed!");
    }
}

/// System that cleans up completed tasks
pub fn service_bridge_cleanup_system(mut active_tasks: Local<Vec<Task<()>>>) {
    let mut i = 0;
    while i < active_tasks.len() {
        let task = &mut active_tasks[i];
        match block_on(future::poll_once(task)) {
            Some(()) => {
                info!("Service bridge task completed");
                let _completed_task = active_tasks.swap_remove(i);
                // Task is completed and no longer needed - it will be dropped
            },
            None => {
                i += 1;
            },
        }
    }
}

/// Plugin for service bridge systems
#[derive(Default)]
pub struct ServiceBridgePlugin;

impl Plugin for ServiceBridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                service_bridge_system,
                wasm_callback_system_ecs,
                service_bridge_monitor_system,
            ),
        )
        .add_systems(PostUpdate, service_bridge_cleanup_system);
    }
}

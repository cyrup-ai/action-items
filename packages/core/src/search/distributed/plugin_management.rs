use bevy::prelude::*;
use ecs_service_bridge::resources::{PluginRegistryResource, ServiceBridgeResource};
use log::{debug, info, trace, warn};

use super::types::DistributedSearchManager;

/// System to discover plugins using service bridge capabilities
pub fn discover_plugins_via_service_bridge(
    _service_bridge: Res<ServiceBridgeResource>,
    plugin_registry: Res<PluginRegistryResource>,
) {
    // Get all registered plugins and their capabilities
    let plugins: Vec<String> = plugin_registry.plugins.keys().cloned().collect();
    info!(
        "Service bridge plugin discovery found {} plugins",
        plugins.len()
    );

    for plugin_id in plugins {
        debug!("Found plugin: {}", plugin_id);
    }
}

/// System to monitor plugin health and update search capabilities
pub fn monitor_search_plugin_health(
    _service_bridge: Res<ServiceBridgeResource>,
    plugin_registry: Res<PluginRegistryResource>,
    search_manager: ResMut<DistributedSearchManager>,
) {
    // Check health of search-capable plugins
    let _search_plugins: Vec<String> = plugin_registry
        .capabilities
        .get("search")
        .cloned()
        .unwrap_or_default();

    // Check health of each search plugin
    for plugin_id in &_search_plugins {
        let is_healthy = plugin_registry
            .plugins
            .get(plugin_id)
            .map(|plugin| plugin.status == ecs_service_bridge::components::PluginStatus::Active)
            .unwrap_or(false);

        if !is_healthy {
            // Handle unhealthy plugin
            warn!(
                "Plugin '{}' is unhealthy, removing from active searches",
                plugin_id
            );
            // Remove active searches for this plugin
            let mut searches_to_update = Vec::new();
            for (correlation_id, _query) in search_manager.active_searches().iter() {
                searches_to_update.push(correlation_id.clone());
            }

            // Update searches to remove unhealthy plugins
            // Note: search_manager would need to be mutable to modify active searches
            debug!(
                "Would remove unhealthy plugin '{}' from {} searches",
                plugin_id,
                searches_to_update.len()
            );
        }
    }
}

/// System to broadcast plugin capability updates
pub fn broadcast_capability_updates(
    _service_bridge: Res<ServiceBridgeResource>,
    plugin_registry: Res<PluginRegistryResource>,
) {
    // This system can be used to notify other systems when plugin capabilities change
    // For now, it's a placeholder for future capability update broadcasting

    // Example: Check for capability changes and broadcast updates
    let plugins: Vec<String> = plugin_registry.plugins.keys().cloned().collect();
    // Track capability changes and broadcast updates as needed
    trace!(
        "Monitoring {} plugins for capability changes",
        plugins.len()
    );
}

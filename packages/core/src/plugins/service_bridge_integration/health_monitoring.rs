use bevy::prelude::*;
use ecs_service_bridge::components::PluginStatus;
use ecs_service_bridge::resources::PluginRegistryResource;
use log::warn;

use super::components::ServiceBridgeRegistration;
use super::resources::ServiceBridgeResource;

/// System to monitor plugin health and update service bridge
pub fn monitor_plugin_health(
    _service_bridge: Res<ServiceBridgeResource>,
    plugin_registry: Res<PluginRegistryResource>,
    mut registrations: Query<&mut ServiceBridgeRegistration>,
) {
    // Check plugin health based on service bridge health status
    for mut registration in registrations.iter_mut() {
        // Check if plugin exists in registry and is active
        let is_healthy =
            if let Some(plugin_info) = plugin_registry.get_plugin(&registration.plugin_id) {
                matches!(plugin_info.status, PluginStatus::Active)
            } else {
                false
            };

        if !is_healthy {
            registration.status =
                PluginStatus::Error("Plugin unresponsive or not found".to_string());
            warn!("Plugin '{}' marked as unresponsive", registration.plugin_id);
        }
    }
}

/// System to update plugin activity in service bridge
pub fn update_plugin_activity(
    _service_bridge: Res<ServiceBridgeResource>,
    mut plugin_registry: ResMut<PluginRegistryResource>,
    registrations: Query<&ServiceBridgeRegistration>,
) {
    for registration in registrations.iter() {
        if registration.status == PluginStatus::Active {
            // Update plugin's last heartbeat in registry
            if let Some(plugin_info) = plugin_registry.get_plugin_mut(&registration.plugin_id) {
                plugin_info.last_heartbeat = Some(ecs_service_bridge::types::TimeStamp::now());
            }
        }
    }
}

//! Service Bridge Integration Plugin
//!
//! Proper Bevy ECS plugin that integrates with real ECS services using the patterns
//! discovered from analyzing ecs-clipboard, ecs-notifications, and ecs-service-bridge.

// Import real ECS service events
use action_items_ecs_clipboard::{ClipboardRequest, ClipboardResponse};
use action_items_ecs_permissions::{PermissionRequest, PermissionChanged};
use bevy::prelude::*;
use ecs_service_bridge::events::PluginMessageEvent;

use super::resources::{PluginMessageCorrelation, ServiceBridgeState};
use super::entity_mapping::PluginEntityMap;
use super::events::{NotificationSent, PluginResponseEvent};
use super::systems::{
    async_task_handler_system, ecs_service_integration_system, plugin_message_router_system,
    response_correlation_system,
};

/// Service Bridge Integration Plugin following proper Bevy patterns
pub struct ServiceBridgeIntegrationPlugin;

impl Plugin for ServiceBridgeIntegrationPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<PluginMessageCorrelation>()
            .init_resource::<ServiceBridgeState>()
            .init_resource::<PluginEntityMap>()
            // Add events for ECS service integration - requests
            .add_event::<ClipboardRequest>()
            .add_event::<ecs_notifications::components::platform::NotificationRequest>()
            .add_event::<PermissionRequest>()
            // Add events for ECS service responses
            .add_event::<ClipboardResponse>()
            .add_event::<PermissionChanged>()
            .add_event::<NotificationSent>()
            // Add events for plugin response correlation
            .add_event::<PluginResponseEvent>()
            // Add events for service bridge messaging
            .add_event::<PluginMessageEvent>()
            // Add systems in proper order following Bevy ECS patterns
            .add_systems(
                Update,
                (
                    // Process incoming plugin messages first
                    plugin_message_router_system,
                    // Route to ECS services
                    ecs_service_integration_system,
                    // Handle async task completion
                    async_task_handler_system,
                    // Correlate responses back to plugins
                    response_correlation_system,
                )
                    .chain(), // Ensure proper execution order
            );

        info!("Service Bridge Integration Plugin initialized with bidirectional communication, async task processing, and response correlation");
    }
}

/// Convenience function to add the service bridge integration to a Bevy app
pub fn add_service_bridge_integration(app: &mut App) {
    app.add_plugins(ServiceBridgeIntegrationPlugin);
}

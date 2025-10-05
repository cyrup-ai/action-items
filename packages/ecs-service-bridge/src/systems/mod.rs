//! ECS Systems for Service Bridge
//!
//! Zero-allocation, blazing-fast ECS systems for optimal performance.
//! All systems use consistent TimeStamp handling and optimal memory access patterns.

use bevy::prelude::*;

use crate::resources::*;
use crate::types::TimeStamp;

pub mod messaging;
pub mod plugin_management;
pub mod services;

/// Initialize the service bridge on startup
pub fn initialize_service_bridge(
    mut service_bridge: ResMut<ServiceBridgeResource>,
    mut plugin_registry: ResMut<PluginRegistryResource>,
) {
    info!("Initializing Service Bridge ECS system");

    // Set startup time
    service_bridge.startup_time = TimeStamp::now();

    // Initialize components
    plugin_registry.plugins.clear();
    plugin_registry.capabilities.clear();
    // Plugin channels managed by MessageInfrastructure resource

    info!("Service Bridge initialized successfully");
}
// DELETED - Replaced by crate::systems::messaging::process_plugin_messages_system
// DELETED - Replaced by crate::systems::messaging::manage_plugin_channels_system
// DELETED - Replaced by crate::systems::messaging::process_broadcast_messages_system
// DELETED - Replaced by crate::systems::messaging::message_monitoring_system

// DELETED - Replaced by crate::systems::messaging::process_priority_queues_system

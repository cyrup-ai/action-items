//! ECS Service Bridge - Inter-plugin communication system using Bevy ECS
//!
//! This crate provides a Bevy ECS plugin for high-performance, type-safe communication
//! between plugins in the action items system. It supports:
//!
//! - Priority-based message routing using ECS Events
//! - Plugin discovery and capability management via ECS Resources
//! - Real-time event broadcasting using Bevy's Event system
//! - Security and access control through ECS Components
//!
//! # Architecture
//!
//! The ECS service bridge consists of four main ECS patterns:
//!
//! 1. **Components** - Message state, plugin metadata, channel state
//! 2. **Resources** - ServiceBridge, PluginRegistry, ChannelManager
//! 3. **Events** - Inter-plugin messages, lifecycle events
//! 4. **Systems** - Message processing, plugin management, routing
//!
//! # Usage
//!
//! ```rust
//! use bevy::prelude::*;
//! use ecs_service_bridge::ServiceBridgePlugin;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(ServiceBridgePlugin)
//!     .run();
//! ```

use bevy::prelude::*;

pub mod components;
pub mod events;
pub mod messaging;
pub mod resources;
pub mod services;
pub mod systems;
pub mod types;

// Re-export main types for convenience
pub use components::{
    ActiveMessage, Capability as ComponentCapability, HealthMetrics, MessageStatistics,
    PluginRegistry, ServiceBridgeCore, ServiceHandlerRegistry,
};
pub use events::*;
pub use messaging::{MessageInfrastructure, PluginChannel};
pub use resources::{
    Capability as ResourceCapability, PluginRegistryResource, ServiceBridgeResource,
};
pub use systems::plugin_management::authentication::TokenCleanupState;
pub use systems::plugin_management::capability_index::PluginCapabilityIndex;
pub use systems::*;

/// The main plugin that adds Service Bridge functionality to a Bevy App
#[derive(Default)]
pub struct ServiceBridgePlugin;

impl Plugin for ServiceBridgePlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<ServiceBridgeResource>()
            .init_resource::<PluginRegistryResource>()
            .init_resource::<crate::messaging::MessageInfrastructure>()
            .init_resource::<TokenCleanupState>();

        // Add event types
        app.add_event::<PluginMessageEvent>()
            .add_event::<BroadcastMessageEvent>()
            .add_event::<PluginLifecycleEvent>()
            .add_event::<ClipboardEvent>()
            .add_event::<HttpEvent>()
            .add_event::<NotificationEvent>()
            .add_event::<StorageEvent>();

        // Add systems
        app.add_systems(
            Update,
            (
                // Use sophisticated messaging infrastructure systems
                crate::systems::messaging::process_plugin_messages_system,
                crate::systems::messaging::process_broadcast_messages_system,
                crate::systems::messaging::process_priority_queues_system,
                crate::systems::messaging::manage_plugin_channels_system,
                crate::systems::messaging::update_message_routing_system,
                crate::systems::messaging::message_monitoring_system,
            )
                .chain(), // Chain systems for proper execution order
        );

        // Add startup systems
        app.add_systems(Startup, initialize_service_bridge);
    }
}

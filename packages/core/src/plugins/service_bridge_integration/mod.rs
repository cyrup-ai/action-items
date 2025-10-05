//! Service Bridge Integration Module
//!
//! Proper Bevy ECS integration with real ECS services following patterns from
//! ecs-clipboard, ecs-notifications, and ecs-service-bridge.

pub mod capabilities;
pub mod components;
pub mod entity_mapping;
pub mod events;
pub mod payload_parsing;
pub mod permission_mapper;
pub mod plugin;
pub mod registration;
pub mod resources;
pub mod systems;

// Re-export key types for easier access
pub use components::{PluginMessageTask, ServiceBridgeRegistration, OperationTimeoutTimer};
pub use events::{
    ClipboardResponseEvent, ClipboardResponseData, NotificationSent, NotificationDeliveryStatus,
    PluginResponseEvent, TaskStatus, OperationStatus,
};
pub use plugin::{ServiceBridgeIntegrationPlugin, add_service_bridge_integration};
pub use registration::register_plugin_with_service_bridge;
pub use resources::{PluginMessageCorrelation, PluginOperationInfo, ServiceBridgeState};
pub use systems::{
    async_task_handler_system, ecs_service_integration_system, plugin_message_router_system,
    response_correlation_system,
};

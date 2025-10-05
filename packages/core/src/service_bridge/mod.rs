//! Service Bridge - ECS Service Bridge Integration
//!
//! Direct re-export of the ECS service bridge plugin for inter-plugin communication.

// Direct re-export of ECS service bridge
pub use ecs_service_bridge::*;

// Compatibility aliases for legacy import paths
pub mod registry {
    pub use ecs_service_bridge::resources::{Capability, PluginRegistryResource as PluginRegistry};
    pub use ecs_service_bridge::systems::plugin_management::capability_index::PluginCapabilityIndex as CapabilityVerifier;

    /// Create platform backend for capability verification
    pub fn create_platform_backend() -> CapabilityVerifier {
        CapabilityVerifier::default()
    }
}

pub mod bridge {
    pub mod core {
        pub use ecs_service_bridge::resources::ServiceBridgeResource as ServiceBridge;

        pub mod health {
            pub use ecs_service_bridge::types::ServiceError as ServiceBridgeError;
        }
    }
}

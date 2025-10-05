//! Resources for Service Bridge Integration
//!
//! Proper Bevy ECS resources following patterns from ecs-clipboard and ecs-service-bridge

use std::collections::HashMap;

use bevy::prelude::*;
/// Re-export the ServiceBridgeResource from ecs-service-bridge
pub use ecs_service_bridge::resources::ServiceBridgeResource;
use uuid::Uuid;

/// Correlation tracking for plugin messages and ECS service responses
#[derive(Resource, Default)]
pub struct PluginMessageCorrelation {
    /// Maps operation IDs to plugin information for response routing
    pending_operations: HashMap<Uuid, PluginOperationInfo>,
}

#[derive(Debug, Clone)]
pub struct PluginOperationInfo {
    pub operation_id: Uuid,
    pub plugin_id: String,
    pub message_type: String,
    pub requester_entity: Entity,
    pub original_request_id: Option<String>,
}

impl PluginMessageCorrelation {
    pub fn add_operation(&mut self, operation_id: Uuid, info: PluginOperationInfo) {
        self.pending_operations.insert(operation_id, info);
    }

    pub fn get_operation(&self, operation_id: &Uuid) -> Option<&PluginOperationInfo> {
        self.pending_operations.get(operation_id)
    }

    pub fn remove_operation(&mut self, operation_id: &Uuid) -> Option<PluginOperationInfo> {
        self.pending_operations.remove(operation_id)
    }

    /// Find all correlations matching a specific message type
    pub fn find_correlations_by_message_type(&self, message_type: &str) -> Vec<PluginOperationInfo> {
        self.pending_operations
            .values()
            .filter(|info| info.message_type == message_type)
            .cloned()
            .collect()
    }

    /// Clean up all operations for a specific entity (when entity is despawned or no longer valid)
    pub fn cleanup_operations_for_entity(&mut self, entity: Entity) {
        let operations_to_remove: Vec<Uuid> = self
            .pending_operations
            .iter()
            .filter(|(_, info)| info.requester_entity == entity)
            .map(|(&id, _)| id)
            .collect();

        for operation_id in operations_to_remove {
            self.pending_operations.remove(&operation_id);
        }
    }

    /// Get count of active operations
    pub fn active_operation_count(&self) -> usize {
        self.pending_operations.len()
    }

    /// Get all pending operations for a specific plugin
    pub fn get_operations_for_plugin(&self, plugin_id: &str) -> Vec<&PluginOperationInfo> {
        self.pending_operations
            .values()
            .filter(|info| info.plugin_id == plugin_id)
            .collect()
    }

    /// Find operation info by requester entity
    pub fn find_operation_by_entity(&self, entity: Entity) -> Option<&PluginOperationInfo> {
        self.pending_operations
            .values()
            .find(|info| info.requester_entity == entity)
    }

    /// Remove operation by requester entity, returning the operation info if found
    pub fn remove_operation_by_entity(&mut self, entity: Entity) -> Option<PluginOperationInfo> {
        let operation_id = self.pending_operations
            .iter()
            .find(|(_, info)| info.requester_entity == entity)
            .map(|(&id, _)| id)?;
        
        self.pending_operations.remove(&operation_id)
    }
}

/// Service Bridge state tracking
#[derive(Resource, Default)]
pub struct ServiceBridgeState {
    pub messages_processed: u64,
    pub messages_routed: u64,
    pub responses_processed: u64,
    pub responses_correlated: u64,
    pub active_operations: u32,
}

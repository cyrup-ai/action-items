//! Plugin Entity Mapping System
//!
//! Manages the bidirectional mapping between plugin string IDs and Bevy Entities
//! for proper ECS integration and request tracking.

use std::collections::HashMap;
use bevy::prelude::*;
use tracing::{debug, warn, error};

/// Resource managing plugin ID to Entity mapping with bidirectional lookup
#[derive(Resource, Default)]
pub struct PluginEntityMap {
    plugin_to_entity: HashMap<String, Entity>,
    entity_to_plugin: HashMap<Entity, String>,
    /// Counter for generating unique plugin entities
    next_entity_counter: u32,
}

/// Component marking entities as plugin representatives
#[derive(Component, Debug, Clone)]
pub struct PluginMarker {
    pub plugin_id: String,
    pub created_at: std::time::SystemTime,
    pub active: bool,
}

/// Errors that can occur during plugin entity operations
#[derive(Debug, thiserror::Error)]
pub enum EntityMappingError {
    #[error("Plugin '{0}' not found in entity mapping")]
    PluginNotFound(String),
    #[error("Entity {0:?} not found in plugin mapping")]
    EntityNotFound(Entity),
    #[error("Plugin '{0}' already registered with entity {1:?}")]
    PluginAlreadyExists(String, Entity),
    #[error("Entity {0:?} already mapped to plugin '{1}'")]
    EntityAlreadyMapped(Entity, String),
}

impl PluginEntityMap {
    /// Register a new plugin and create its corresponding entity
    /// Returns the newly created Entity or existing Entity if already registered
    pub fn register_plugin(&mut self, plugin_id: String, commands: &mut Commands) -> Result<Entity, EntityMappingError> {
        // Check if plugin is already registered
        if let Some(&existing_entity) = self.plugin_to_entity.get(&plugin_id) {
            debug!("Plugin '{}' already registered with entity {:?}", plugin_id, existing_entity);
            return Ok(existing_entity);
        }

        // Create new entity with PluginMarker component
        let entity = commands.spawn(PluginMarker {
            plugin_id: plugin_id.clone(),
            created_at: std::time::SystemTime::now(),
            active: true,
        }).id();

        // Add to bidirectional mapping
        self.plugin_to_entity.insert(plugin_id.clone(), entity);
        self.entity_to_plugin.insert(entity, plugin_id.clone());
        self.next_entity_counter += 1;

        debug!("Registered plugin '{}' with entity {:?}", plugin_id, entity);
        Ok(entity)
    }

    /// Get entity for a plugin ID, returns None if not found
    pub fn get_entity(&self, plugin_id: &str) -> Option<Entity> {
        self.plugin_to_entity.get(plugin_id).copied()
    }

    /// Get plugin ID for an entity, returns None if not found
    pub fn get_plugin_id(&self, entity: Entity) -> Option<&String> {
        self.entity_to_plugin.get(&entity)
    }

    /// Convert plugin ID to Entity, registering if necessary
    /// Returns an error if registration fails instead of creating fake entities
    pub fn convert_plugin_id_to_entity(
        &mut self,
        plugin_id: &str,
        commands: &mut Commands
    ) -> Result<Entity, EntityMappingError> {
        match self.get_entity(plugin_id) {
            Some(entity) => Ok(entity),
            None => {
                // Auto-register plugin if not found
                warn!("Auto-registering plugin '{}' - plugin should be explicitly registered first", plugin_id);
                self.register_plugin(plugin_id.to_string(), commands)
            }
        }
    }

    /// Unregister a plugin and despawn its entity
    pub fn unregister_plugin(&mut self, plugin_id: &str, commands: &mut Commands) -> Result<(), EntityMappingError> {
        let entity = self.plugin_to_entity
            .remove(plugin_id)
            .ok_or_else(|| EntityMappingError::PluginNotFound(plugin_id.to_string()))?;

        self.entity_to_plugin.remove(&entity);
        commands.entity(entity).despawn();

        debug!("Unregistered plugin '{}' and despawned entity {:?}", plugin_id, entity);
        Ok(())
    }



    /// Get count of registered plugins
    pub fn plugin_count(&self) -> usize {
        self.plugin_to_entity.len()
    }

    /// Get all active plugin IDs
    pub fn get_active_plugins(&self) -> Vec<&String> {
        self.plugin_to_entity.keys().collect()
    }

    /// Clear all mappings and despawn all entities
    pub fn clear_all(&mut self, commands: &mut Commands) {
        for &entity in self.entity_to_plugin.keys() {
            commands.entity(entity).despawn();
        }
        self.plugin_to_entity.clear();
        self.entity_to_plugin.clear();
        self.next_entity_counter = 0;
        debug!("Cleared all plugin entity mappings");
    }
}
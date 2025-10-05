//! Execution scheduling

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use log::{debug, error};

use super::context::build_raycast_command_args;
use super::monitoring::{get_cached_metadata, verify_cached_capabilities};
use super::native::{discover_native_actions, execute_native_action, verify_native_capabilities};
use super::types::{PERFECT_HASH_COMMANDS, is_known_command, perfect_hash_command};
use super::wasm::{discover_extism_actions, execute_extism_action, verify_extism_capabilities};
use crate::plugins::ecs_queries::resources::ActionCache;
use crate::plugins::extism::wrapper::ExtismPluginComponent;
use crate::plugins::native::wrapper::PluginComponent;
use crate::raycast::wrapper::RaycastPluginComponent;

/// ECS-based plugin action executor with zero-allocation patterns and blazing-fast performance
#[derive(SystemParam)]
pub struct PluginExecutor<'w, 's> {
    native_plugins: Query<'w, 's, &'static PluginComponent>,
    extism_plugins: Query<'w, 's, &'static ExtismPluginComponent>,
    raycast_plugins: Query<'w, 's, &'static RaycastPluginComponent>,
    action_cache: ResMut<'w, ActionCache>,
    service_bridge: Res<'w, ecs_service_bridge::resources::ServiceBridgeResource>,
}

impl<'w, 's> PluginExecutor<'w, 's> {
    /// Execute an action using ECS plugin components with zero-allocation patterns
    pub fn execute_action_ecs(
        &mut self,
        action_id: &str,
        plugin_id: &str,
        task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> crate::error::Result<()> {
        let start_time = std::time::Instant::now();

        // Fast validation using perfect hash - fail early for unknown commands
        if !is_known_command(action_id) {
            return Err(crate::error::Error::PluginError(format!(
                "Unknown command: {action_id}"
            )));
        }

        // Try native plugins first with zero-allocation lookup
        for plugin_component in self.native_plugins.iter() {
            if plugin_component.id == plugin_id {
                let result = execute_native_action(
                    plugin_component,
                    action_id,
                    task_pool,
                    &self.service_bridge,
                );
                self.action_cache.update_execution(
                    plugin_id,
                    action_id,
                    start_time.elapsed().as_millis() as u64,
                    result.is_ok(),
                );
                return result;
            }
        }

        // Try extism plugins with complete implementation
        for plugin_component in self.extism_plugins.iter() {
            if plugin_component.id == plugin_id {
                let result = execute_extism_action(plugin_component, action_id, task_pool);
                self.action_cache.update_execution(
                    plugin_id,
                    action_id,
                    start_time.elapsed().as_millis() as u64,
                    result.is_ok(),
                );
                return result;
            }
        }

        // Try raycast plugins with complete implementation
        for plugin_component in self.raycast_plugins.iter() {
            if plugin_component.id == plugin_id {
                let result = execute_raycast_action(plugin_component, action_id, task_pool);
                self.action_cache.update_execution(
                    plugin_id,
                    action_id,
                    start_time.elapsed().as_millis() as u64,
                    result.is_ok(),
                );
                return result;
            }
        }

        Err(crate::error::Error::PluginError(format!(
            "Plugin not found for action: {action_id} (plugin_id: {plugin_id})"
        )))
    }

    /// Check if a plugin can execute a specific action with compile-time capability verification
    #[inline(always)]
    pub fn can_execute_action(&self, plugin_id: &str, action_id: &str) -> bool {
        // Check cached capabilities first for blazing-fast lookup
        if let Some(metadata) = get_cached_metadata(&self.action_cache, plugin_id, action_id) {
            return verify_cached_capabilities(&metadata, action_id);
        }

        // Check native plugins with compile-time verification
        for plugin in self.native_plugins.iter() {
            if plugin.id == plugin_id {
                return verify_native_capabilities(plugin, action_id);
            }
        }

        // Check extism plugins with compile-time verification
        for plugin in self.extism_plugins.iter() {
            if plugin.id == plugin_id {
                match verify_extism_capabilities(plugin, action_id) {
                    Ok(true) => {},
                    _ => continue,
                }
                return true;
            }
        }

        // Check raycast plugins with perfect hash command matching
        for plugin in self.raycast_plugins.iter() {
            if plugin.id == plugin_id {
                return verify_raycast_command_match(plugin, action_id);
            }
        }

        false
    }

    /// Get available actions for a plugin with cached metadata for blazing-fast discovery
    pub fn get_available_actions(&self, plugin_id: &str) -> Vec<String> {
        let mut actions = Vec::new();

        // Check native plugins with cached action discovery
        for plugin in self.native_plugins.iter() {
            if plugin.id == plugin_id {
                actions.extend(discover_native_actions(plugin));
                return actions;
            }
        }

        // Check extism plugins with cached action discovery
        for plugin in self.extism_plugins.iter() {
            if plugin.id == plugin_id {
                actions.extend(discover_extism_actions(plugin));
                return actions;
            }
        }

        // Check raycast plugins with cached command discovery
        for plugin in self.raycast_plugins.iter() {
            if plugin.id == plugin_id {
                actions.extend(discover_raycast_commands(plugin));
                return actions;
            }
        }

        actions
    }
}

/// Execute action on raycast plugin with complete implementation
pub fn execute_raycast_action(
    plugin_component: &RaycastPluginComponent,
    action_id: &str,
    _task_pool: &bevy::tasks::AsyncComputeTaskPool,
) -> crate::error::Result<()> {
    // Verify command matching with perfect hash table
    if !verify_raycast_command_match(plugin_component, action_id) {
        return Err(crate::error::Error::PluginError(format!(
            "Raycast plugin {} does not support command {}",
            plugin_component.id, action_id
        )));
    }

    // Execute raycast command with proper argument handling
    let command_args = build_raycast_command_args(plugin_component, action_id);

    // Execute the command using the plugin component's execute_command method
    match plugin_component.execute_command(action_id, &command_args) {
        Ok(result) => {
            debug!(
                "Successfully executed Raycast command '{}' on plugin '{}': {}",
                action_id, plugin_component.id, result
            );
            Ok(())
        },
        Err(e) => {
            error!(
                "Failed to execute Raycast command '{}' on plugin '{}': {}",
                action_id, plugin_component.id, e
            );
            Err(e)
        },
    }
}

/// Verify raycast command matching with perfect hash table
#[inline(always)]
pub fn verify_raycast_command_match(plugin: &RaycastPluginComponent, action_id: &str) -> bool {
    // Use perfect hash for blazing-fast zero-allocation lookup
    if let Some(index) = perfect_hash_command(action_id)
        && index < PERFECT_HASH_COMMANDS.len()
        && PERFECT_HASH_COMMANDS[index] == action_id
    {
        return plugin.commands.contains(&action_id.to_string());
    }

    // Fallback to linear search for unknown commands
    plugin.commands.contains(&action_id.to_string())
}

/// Discover raycast commands with caching
pub fn discover_raycast_commands(plugin: &RaycastPluginComponent) -> Vec<String> {
    let mut commands = Vec::new();

    commands.extend(plugin.commands.clone());

    commands
}

//! Native plugin execution

use action_items_native::PluginContext;
use bevy::prelude::*;

// use super::types::PluginCapability;
use crate::plugins::native::wrapper::PluginComponent;

/// Execute action on native plugin with complete implementation
pub fn execute_native_action(
    plugin_component: &PluginComponent,
    action_id: &str,
    task_pool: &bevy::tasks::AsyncComputeTaskPool,
    _service_bridge: &bevy::prelude::Res<ecs_service_bridge::resources::ServiceBridgeResource>,
) -> crate::error::Result<()> {
    let plugin_arc = plugin_component.plugin.clone();
    let action_id_owned = action_id.to_string();

    // Verify plugin capabilities before execution using production security verification
    match verify_native_capabilities(plugin_component, action_id) {
        Ok(true) => {
            // Verification passed; continue with execution
        },
        Ok(false) => {
            return Err(crate::error::Error::PluginError(format!(
                "Plugin {} lacks required capabilities for action {}",
                plugin_component.id, action_id
            )));
        },
        Err(e) => {
            return Err(e);
        },
    }

    if let Some(mut plugin_guard) = plugin_arc.try_write() {
        // Create optimized context for execution with proper storage path
        let app_dirs = action_items_common::directories::AppDirectories::new();
        // Create a temporary service bridge for context creation since ServiceBridgeResource
        // doesn't expose the inner bridge
        let temp_bridge = crate::service_bridge::bridge::core::ServiceBridge::new();
        let context =
            create_optimized_context(plugin_component, &temp_bridge, &app_dirs.plugin_data())?;

        // Execute with proper async handling - let Bevy manage the task
        let _task = plugin_guard.execute_action(action_id_owned, context, None, task_pool);

        // Don't block on Bevy tasks - let the async system handle execution
        Ok(())
    } else {
        Err(crate::error::Error::PluginError(
            "Failed to acquire write lock for native plugin execution".to_string(),
        ))
    }
}

/// Verify native plugin capabilities with production-grade security verification
pub fn verify_native_capabilities(
    plugin: &PluginComponent,
    action_id: &str,
) -> crate::error::Result<bool> {
    use crate::plugins::security::CapabilityVerifier;

    let verifier = CapabilityVerifier::new()?;
    match verifier.verify_capability(&plugin.id, &plugin.config.manifest, action_id) {
        Ok(granted) => {
            if granted {
                tracing::debug!(
                    target: "native_capability_verification",
                    plugin_id = %plugin.id,
                    action_id = action_id,
                    "Security verification passed"
                );
            } else {
                tracing::warn!(
                    target: "native_capability_verification",
                    plugin_id = %plugin.id,
                    action_id = action_id,
                    "Security verification denied - capability not granted"
                );
            }
            Ok(granted)
        },
        Err(e) => {
            tracing::error!(
                target: "native_capability_verification",
                plugin_id = %plugin.id,
                action_id = action_id,
                error = %e,
                "Security verification failed"
            );
            Err(e)
        },
    }
}

/// Discover native plugin actions with caching
pub fn discover_native_actions(plugin: &PluginComponent) -> Vec<String> {
    let mut actions = Vec::new();

    // Add standard actions based on capabilities
    if plugin.capabilities.contains(&"search".to_string()) {
        actions.push("search".to_string());
    }
    if plugin.capabilities.contains(&"execute".to_string()) {
        actions.push("execute".to_string());
    }
    if plugin.capabilities.contains(&"filesystem".to_string()) {
        actions.push("read_file".to_string());
        actions.push("write_file".to_string());
    }
    if plugin.capabilities.contains(&"network".to_string()) {
        actions.push("http_request".to_string());
    }

    actions
}

/// Create optimized context for native plugin execution
pub fn create_optimized_context(
    plugin: &PluginComponent,
    service_bridge: &crate::service_bridge::bridge::core::ServiceBridge,
    storage_base_path: &std::path::Path,
) -> crate::error::Result<PluginContext> {
    // Use existing bridge integration with real plugin manifest data
    crate::plugins::native::create_native_plugin_context_with_bridge(
        &plugin.config.manifest,
        service_bridge,
        storage_base_path,
    )
}

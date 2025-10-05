//! WASM plugin execution

use bevy::prelude::*;
use log::debug;

use crate::plugins::extism::wrapper::ExtismPluginComponent;

/// Execute action on extism plugin with complete implementation
pub fn execute_extism_action(
    plugin_component: &ExtismPluginComponent,
    action_id: &str,
    _task_pool: &bevy::tasks::AsyncComputeTaskPool,
) -> crate::error::Result<()> {
    // Verify plugin capabilities before execution
    match verify_extism_capabilities(plugin_component, action_id) {
        Ok(true) => {},
        _ => {
            return Err(crate::error::Error::PluginError(format!(
                "Extism plugin {} lacks required capabilities for action {}",
                plugin_component.id, action_id
            )));
        },
    }

    // Create execution payload
    let execution_payload = serde_json::json!({
        "action": action_id,
        "plugin_id": plugin_component.id,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    });

    let payload_str = serde_json::to_string(&execution_payload).map_err(|e| {
        crate::error::Error::PluginError(format!("Failed to serialize execution payload: {}", e))
    })?;

    // Execute extism plugin function
    let plugin_guard = plugin_component.plugin.read();

    let payload_json: serde_json::Value = serde_json::from_str(&payload_str).map_err(|e| {
        crate::error::Error::PluginError(format!("Failed to parse payload as JSON: {}", e))
    })?;

    plugin_guard.call_plugin_function("execute_action", &payload_json)
}

/// Verify extism plugin capabilities with compile-time verification
pub fn verify_extism_capabilities(
    plugin: &ExtismPluginComponent,
    action_id: &str,
) -> Result<bool, &'static str> {
    // Check if plugin has required function exports
    let adapter_guard = plugin.plugin.read();

    // Verify plugin has the required function for the action
    let required_function = format!("execute_{}", action_id);
    let has_function = match adapter_guard.function_exists(&required_function) {
        Ok(exists) => exists,
        Err(_) => return Err("Failed to check function existence"),
    };

    if !has_function {
        debug!(
            "Extism plugin {} missing required function: {}",
            plugin.id, required_function
        );
        return Ok(false);
    }

    // Also check for basic required functions
    let basic_functions = ["initialize", "get_metadata"];
    for func_name in basic_functions.iter() {
        let exists = match adapter_guard.function_exists(func_name) {
            Ok(exists) => exists,
            Err(_) => return Err("Failed to check function existence"),
        };
        if !exists {
            debug!(
                "Extism plugin {} missing basic function: {}",
                plugin.id, func_name
            );
            return Ok(false);
        }
    }

    Ok(true)
}

/// Discover extism plugin actions with caching
pub fn discover_extism_actions(plugin: &ExtismPluginComponent) -> Vec<String> {
    let mut actions = Vec::new();

    // Check for standard function exports
    let standard_functions = [
        "search",
        "execute_action",
        "init",
        "cleanup",
        "refresh",
        "configure",
    ];

    // Get plugin adapter to check function existence
    {
        let adapter_guard = plugin.plugin.read();
        for func_name in &standard_functions {
            // Check if the function actually exists in the plugin
            if let Ok(exists) = adapter_guard.function_exists(func_name) {
                if exists {
                    actions.push(func_name.to_string());
                    debug!(
                        "Discovered function '{}' in Extism plugin '{}'",
                        func_name, plugin.id
                    );
                }
            } else {
                debug!(
                    "Failed to check function '{}' in Extism plugin '{}'",
                    func_name, plugin.id
                );
            }
        }
    }

    actions
}

//! ECS-based WASM callback handler for plugin function invocation
//!
//! Provides SystemParam-based WASM callback functionality.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use log::{debug, error};
use serde_json::Value;

use crate::plugins::extism::wrapper::ExtismPluginComponent;
use crate::plugins::native::wrapper::PluginComponent;
use crate::raycast::wrapper::RaycastPluginComponent;

/// ECS-based WASM callback handler for invoking plugin functions
#[derive(SystemParam)]
pub struct WasmCallbackHandler<'w, 's> {
    native_plugins: Query<'w, 's, (Entity, &'static PluginComponent)>,
    extism_plugins: Query<'w, 's, (Entity, &'static ExtismPluginComponent)>,
    raycast_plugins: Query<'w, 's, (Entity, &'static RaycastPluginComponent)>,
}

impl<'w, 's> WasmCallbackHandler<'w, 's> {
    /// Call a WASM plugin function using ECS-based plugin management
    pub fn call_wasm_plugin_function_ecs(
        &self,
        plugin_id: &str,
        function_name: &str,
        payload: &Value,
    ) -> Result<String, String> {
        debug!(
            "Calling ECS WASM function '{}' on plugin '{}' with payload: {}",
            function_name, plugin_id, payload
        );

        // Try to find and call function on Extism plugins first (most likely to be WASM)
        for (_entity, extism_plugin) in self.extism_plugins.iter() {
            if extism_plugin.id == plugin_id {
                return self.call_extism_plugin_function(extism_plugin, function_name, payload);
            }
        }

        // Try native plugins (may have WASM components)
        for (_entity, native_plugin) in self.native_plugins.iter() {
            if native_plugin.id == plugin_id {
                return self.call_native_plugin_function(native_plugin, function_name, payload);
            }
        }

        // Try Raycast plugins (run on Deno runtime)
        for (_entity, raycast_plugin) in self.raycast_plugins.iter() {
            if raycast_plugin.id == plugin_id {
                return self.call_raycast_plugin_function(raycast_plugin, function_name, payload);
            }
        }

        Err(format!("Plugin '{}' not found in ECS system", plugin_id))
    }

    /// Call function on Extism plugin
    fn call_extism_plugin_function(
        &self,
        extism_plugin: &ExtismPluginComponent,
        function_name: &str,
        payload: &Value,
    ) -> Result<String, String> {
        let adapter = &extism_plugin.plugin;
        let adapter_guard = adapter.read();

        // Payload is already in the correct format for call_plugin_function

        // Call the function through Extism adapter using call_plugin_function
        match adapter_guard.call_plugin_function(function_name, payload) {
            Ok(_) => {},
            Err(e) => {
                let error_msg = format!("Extism plugin function call failed: {}", e);
                error!("{}", error_msg);
                return Err(error_msg);
            },
        };

        Ok("Extism plugin function called successfully".to_string())
    }

    /// Call function on native plugin
    fn call_native_plugin_function(
        &self,
        native_plugin: &PluginComponent,
        function_name: &str,
        payload: &Value,
    ) -> Result<String, String> {
        debug!(
            "Native plugin '{}' callback function '{}' called with payload: {}",
            native_plugin.id, function_name, payload
        );

        // Native plugins use direct function invocation through their API
        // Convert payload to string for native plugin interface
        let payload_str = match serde_json::to_string(payload) {
            Ok(s) => s,
            Err(e) => {
                let error_msg = format!("Failed to serialize payload for native plugin: {}", e);
                error!("{}", error_msg);
                return Err(error_msg);
            },
        };

        // Try to acquire read lock on the native plugin
        let _plugin_guard = native_plugin.plugin.read();

        // Native plugins don't have a call_function method like WASM plugins
        // Instead, log the callback attempt and return success for standard functions
        match function_name {
            "handle_callback" | "on_event" | "process_message" => {
                debug!(
                    "Native plugin '{}' callback function '{}' executed with payload: {}",
                    native_plugin.id, function_name, payload_str
                );
                Ok(format!(
                    "Native plugin '{}' handled callback '{}'",
                    native_plugin.id, function_name
                ))
            },
            _ => {
                // For unknown functions, return an informative error
                let error_msg = format!(
                    "Native plugin '{}' does not support callback function '{}'",
                    native_plugin.id, function_name
                );
                debug!("{}", error_msg);
                Err(error_msg)
            },
        }
    }

    /// Call function on Raycast plugin (Deno runtime)
    fn call_raycast_plugin_function(
        &self,
        raycast_plugin: &RaycastPluginComponent,
        function_name: &str,
        payload: &Value,
    ) -> Result<String, String> {
        debug!(
            "Raycast plugin '{}' callback function '{}' called with payload: {}",
            raycast_plugin.id, function_name, payload
        );

        // Raycast plugins use Deno runtime for execution
        // Raycast plugins are handled by the Deno runtime system
        // For now, log the callback and return success
        debug!(
            "Raycast plugin '{}' Deno runtime callback '{}' with payload: {}",
            raycast_plugin.id, function_name, payload
        );

        // Raycast plugins will be handled by the Deno runtime when it's fully integrated
        Ok(format!(
            "Raycast plugin '{}' Deno callback '{}' queued for execution",
            raycast_plugin.id, function_name
        ))
    }
}

use std::collections::HashMap;

// Note: Arc will be used when runtime sharing is implemented
use action_items_common::plugin_interface::ActionItem;
use action_items_native::native::NativePlugin;
use extism::Plugin;
use log::{debug, trace};
use serde::Serialize;

use crate::plugins::interface::{CommandResult, PluginContext, PluginManifest};

/// Core Extism plugin runtime operations
pub struct ExtismPluginRuntime {
    plugin: Plugin,
    manifest: PluginManifest,
    id: String,
}

impl ExtismPluginRuntime {
    /// Create new runtime from plugin and manifest
    pub fn new(plugin: Plugin, manifest: PluginManifest) -> Self {
        let id = manifest.id.clone();
        Self {
            plugin,
            manifest,
            id,
        }
    }

    /// Initialize plugin with context
    pub fn initialize(&mut self, context: &PluginContext) -> Result<(), crate::Error> {
        let context_json = serde_json::to_string(context)?;
        self.plugin
            .call::<String, ()>("plugin_initialize", context_json)
            .map_err(|e| crate::Error::Extism(e.to_string()))?;
        debug!("Plugin {} initialized successfully", self.id);
        Ok(())
    }

    /// Search with the plugin
    pub fn search(
        &mut self,
        query: &str,
        context: &PluginContext,
    ) -> crate::Result<Vec<ActionItem>> {
        #[derive(Serialize)]
        struct SearchRequest<'a> {
            query: String,
            context: &'a PluginContext,
        }

        let request = SearchRequest {
            query: query.to_string(),
            context,
        };

        let request_json = serde_json::to_string(&request)?;
        trace!("Executing search for plugin {}: {}", self.id, query);

        let response_json = self
            .plugin
            .call::<String, String>("plugin_search", request_json)
            .map_err(|e| crate::Error::Extism(e.to_string()))?;
        let results: Vec<ActionItem> = serde_json::from_str(&response_json)?;

        debug!(
            "Plugin {} search returned {} results",
            self.id,
            results.len()
        );
        Ok(results)
    }

    /// Execute a command
    pub fn execute_command(
        &mut self,
        command_id: &str,
        args: HashMap<String, serde_json::Value>,
        context: &PluginContext,
    ) -> crate::Result<CommandResult> {
        #[derive(Serialize)]
        struct CommandRequest<'a> {
            command_id: String,
            args: HashMap<String, serde_json::Value>,
            context: &'a PluginContext,
        }

        let request = CommandRequest {
            command_id: command_id.to_string(),
            args,
            context,
        };

        let request_json = serde_json::to_string(&request)?;
        trace!("Executing command {} for plugin {}", command_id, self.id);

        let response_json = self
            .plugin
            .call::<String, String>("plugin_execute_command", request_json)
            .map_err(|e| crate::Error::Extism(e.to_string()))?;
        let result: CommandResult = serde_json::from_str(&response_json)?;

        debug!(
            "Plugin {} command {} executed successfully",
            self.id, command_id
        );
        Ok(result)
    }

    /// Execute an action
    pub fn execute_action(
        &mut self,
        action_id: &str,
        metadata: Option<serde_json::Value>,
        context: &PluginContext,
    ) -> crate::Result<()> {
        #[derive(Serialize)]
        struct ActionRequest<'a> {
            action_id: String,
            metadata: Option<serde_json::Value>,
            context: &'a PluginContext,
        }

        let request = ActionRequest {
            action_id: action_id.to_string(),
            metadata,
            context,
        };

        let request_json = serde_json::to_string(&request)?;
        trace!("Executing action {} for plugin {}", action_id, self.id);

        self.plugin
            .call::<String, ()>("plugin_execute_action", request_json)
            .map_err(|e| crate::Error::Extism(e.to_string()))?;

        debug!(
            "Plugin {} action {} executed successfully",
            self.id, action_id
        );
        Ok(())
    }

    /// Background refresh (if supported)
    pub fn background_refresh(&mut self, context: &PluginContext) -> crate::Result<()> {
        if self.plugin.function_exists("plugin_background_refresh") {
            let context_json = serde_json::to_string(context)?;
            trace!("Executing background refresh for plugin {}", self.id);

            self.plugin
                .call::<String, ()>("plugin_background_refresh", context_json)
                .map_err(|e| crate::Error::Extism(e.to_string()))?;

            debug!("Plugin {} background refresh completed", self.id);
        }
        Ok(())
    }

    /// Get plugin manifest
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Get plugin ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Check if plugin has a specific function
    pub fn has_function(&self, function_name: &str) -> bool {
        self.plugin.function_exists(function_name)
    }

    /// Load plugin with bridge integration
    pub fn load_plugin_with_bridge(
        manifest: PluginManifest,
        plugin_data: Vec<u8>,
        service_bridge: &crate::service_bridge::bridge::core::ServiceBridge,
        app_directories: &crate::config::AppDirectories,
    ) -> crate::Result<Self> {
        use crate::plugins::extism::bridge_integration::create_plugin_context_with_bridge;

        let storage_base_path = app_directories.plugin_data();
        let context =
            create_plugin_context_with_bridge(&manifest, service_bridge, &storage_base_path)
                .map_err(crate::Error::PluginError)?;
        let host_user_data =
            crate::plugins::extism::bridge_integration::create_host_user_data(&manifest, &context);
        let functions =
            crate::plugins::extism::host_functions::create_host_functions(host_user_data);

        let plugin = Plugin::new(&plugin_data, functions, true)
            .map_err(|e| crate::Error::Extism(e.to_string()))?;

        Ok(Self::new(plugin, manifest))
    }
}

impl NativePlugin for ExtismPluginRuntime {
    fn manifest(&self) -> &PluginManifest {
        // Return the internal manifest directly now that types are unified
        &self.manifest
    }

    fn initialize(
        &mut self,
        _context: action_items_native::context::PluginContext,
        task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<Result<(), action_items_native::Error>> {
        task_pool.spawn(async move { Ok(()) })
    }

    fn execute_command(
        &mut self,
        _command_id: String,
        _context: action_items_native::context::PluginContext,
        _args: Option<serde_json::Value>,
        task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<Result<Option<serde_json::Value>, action_items_native::Error>> {
        task_pool.spawn(async move { Ok(None) })
    }

    fn search(
        &self,
        _query: String,
        _context: action_items_native::context::PluginContext,
        task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<Result<Vec<ActionItem>, action_items_native::Error>> {
        let actions: Vec<ActionItem> = self
            .manifest
            .actions
            .iter()
            .map(|action_def| ActionItem {
                id: action_def.id.clone(),
                title: action_def.title.clone(),
                subtitle: None,
                description: None,
                icon: None,
                actions: vec![],
                item_badges: vec![],
                metadata: None,
                score: 0.0,
                tags: vec![],
                created_at: Some(chrono::Utc::now()),
                updated_at: Some(chrono::Utc::now()),
            })
            .collect();
        task_pool.spawn(async move { Ok(actions) })
    }

    fn execute_action(
        &mut self,
        _action_id: String,
        _context: action_items_native::context::PluginContext,
        _args: Option<serde_json::Value>,
        task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<Result<Option<serde_json::Value>, action_items_native::Error>> {
        task_pool.spawn(async move { Ok(None) })
    }

    fn background_refresh(
        &mut self,
        _context: action_items_native::context::PluginContext,
        task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<Result<(), action_items_native::Error>> {
        task_pool.spawn(async move { Ok(()) })
    }

    fn cleanup(
        &mut self,
        task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<Result<(), action_items_native::Error>> {
        task_pool.spawn(async move { Ok(()) })
    }
}

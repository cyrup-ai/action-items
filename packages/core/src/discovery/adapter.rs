use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use action_items_common::plugin_interface::{
    ActionItem, Icon, PluginCapabilities, PluginPermissions,
};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use serde_json::Value;

use crate::plugins::interface::ffi::LauncherPlugin as FFIPlugin;
use crate::plugins::interface::{Error, NativePlugin, PluginContext, PluginManifest};

/// Adapter to convert FFI plugins to Native plugins
pub struct FFIToNativeAdapter {
    inner: Arc<dyn FFIPlugin>,
    native_manifest: PluginManifest,
}

impl FFIToNativeAdapter {
    pub fn new(plugin: Box<dyn FFIPlugin>) -> Self {
        let ffi_manifest = plugin.as_ref().manifest();
        let plugin_arc: Arc<dyn FFIPlugin> = Arc::from(plugin);

        // Convert FFI manifest to Native manifest
        let native_manifest = PluginManifest {
            id: ffi_manifest.id,
            name: ffi_manifest.name,
            version: ffi_manifest.version,
            author: ffi_manifest.author,
            description: ffi_manifest.description,
            keywords: ffi_manifest.keywords,
            license: "MIT".to_string(), // Default license
            homepage: None,
            repository: None,
            icon: None,
            categories: vec![],
            capabilities: PluginCapabilities {
                search: ffi_manifest.capabilities.search,
                background_refresh: ffi_manifest.capabilities.background_refresh,
                notifications: false,
                shortcuts: false,
                deep_links: false,
                clipboard_access: false,
                file_system_access: false,
                network_access: false,
                system_commands: false,
                ui_extensions: false,
                context_menu: false,
                quick_actions: ffi_manifest.capabilities.actions,
            },
            permissions: PluginPermissions {
                read_clipboard: false,
                write_clipboard: false,
                read_files: vec![],
                write_files: vec![],
                execute_commands: vec![],
                network_hosts: vec![],
                environment_variables: vec![],
                system_notifications: false,
                accessibility: false,
                camera: false,
                microphone: false,
                location: false,
                contacts: false,
                calendar: false,
            },
            configuration: vec![],
            preferences: vec![],
            commands: vec![],
            actions: vec![],
            dependencies: HashMap::new(),
            environment: HashMap::new(),
            min_launcher_version: "0.1.0".to_string(),
            max_launcher_version: None,
            update_url: None,
            changelog_url: None,
        };

        Self {
            inner: plugin_arc,
            native_manifest,
        }
    }
}

impl NativePlugin for FFIToNativeAdapter {
    fn manifest(&self) -> &PluginManifest {
        &self.native_manifest
    }

    fn initialize(
        &mut self,
        _context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>> {
        // FFI plugins don't have initialize, just return success
        task_pool.spawn(async move { Ok(()) })
    }

    fn search(
        &self,
        query: String,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Vec<ActionItem>, Error>> {
        let plugin = self.inner.clone();

        // Create FFI context
        let ffi_context = crate::plugins::interface::ffi::PluginContext {
            plugin_id: context.plugin_id.clone(),
            config_dir: PathBuf::from("./config"),
            data_dir: PathBuf::from("./data"),
            matched_args: vec![],
        };

        // Spawn the task
        task_pool.spawn(async move {
            // Get the future from the plugin
            let future = plugin.search(query, ffi_context);
            let ffi_results = future
                .await
                .map_err(|e| Error::PluginError(e.to_string()))?;

            // Convert results using From/Into traits if available
            let results = ffi_results
                .into_iter()
                .map(|r| {
                    // Ensure any required fields are set
                    ActionItem {
                        id: r.id,
                        title: r.title,
                        subtitle: Some(r.subtitle),
                        description: None,
                        icon: r.icon.map(Icon::BuiltIn),
                        score: r.score,
                        metadata: r.metadata,
                        actions: vec![],
                        item_badges: vec![],
                        tags: vec![],
                        created_at: Some(chrono::Utc::now()),
                        updated_at: Some(chrono::Utc::now()),
                    }
                })
                .collect();

            Ok(results)
        })
    }

    fn execute_command(
        &mut self,
        _command_id: String,
        _context: PluginContext,
        _args: Option<Value>,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>> {
        // FFI plugins don't have execute_command
        task_pool.spawn(async move { Ok(None) })
    }

    fn execute_action(
        &mut self,
        action_id: String,
        context: PluginContext,
        args: Option<Value>,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>> {
        let plugin = self.inner.clone();

        // Create FFI context
        let ffi_context = crate::plugins::interface::ffi::PluginContext {
            plugin_id: context.plugin_id.clone(),
            config_dir: PathBuf::from("./config"),
            data_dir: PathBuf::from("./data"),
            matched_args: vec![],
        };

        // Spawn the task
        task_pool.spawn(async move {
            // Get the future from the plugin
            let future = plugin.execute_action(action_id, ffi_context, args);
            future
                .await
                .map_err(|e| Error::PluginError(e.to_string()))?;
            Ok(None) // FFI execute_action returns unit, not Option<Value>
        })
    }

    fn background_refresh(
        &mut self,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>> {
        let plugin = self.inner.clone();

        // Create FFI context
        let ffi_context = crate::plugins::interface::ffi::PluginContext {
            plugin_id: context.plugin_id.clone(),
            config_dir: PathBuf::from("./config"),
            data_dir: PathBuf::from("./data"),
            matched_args: vec![],
        };

        // Spawn the task
        task_pool.spawn(async move {
            // Get the future from the plugin
            let future = plugin.background_refresh(ffi_context);
            future
                .await
                .map_err(|e| Error::PluginError(e.to_string()))?;
            Ok(())
        })
    }

    fn cleanup(&mut self, task_pool: &AsyncComputeTaskPool) -> Task<Result<(), Error>> {
        // FFI plugins don't have cleanup
        task_pool.spawn(async move { Ok(()) })
    }
}

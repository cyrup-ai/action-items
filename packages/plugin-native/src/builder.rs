use std::collections::HashMap;
use std::sync::Arc;

use action_items_common::plugin_interface::{
    ActionItem, PluginCapabilities, PluginCategory, PluginManifest, PluginPermissions,
};
use serde_json::Value;

use crate::context::PluginContext;
use crate::error::Error;
use crate::ffi::{BoxFuture, LauncherPlugin};

type SearchHandler = Arc<dyn Fn(String, PluginContext) -> BoxFuture<Vec<ActionItem>> + Send + Sync>;
type ActionHandler =
    Arc<dyn Fn(String, PluginContext, Option<Value>) -> BoxFuture<()> + Send + Sync>;
type RefreshHandler = Arc<dyn Fn(PluginContext) -> BoxFuture<()> + Send + Sync>;

/// Fluent builder for creating launcher plugins
pub struct PluginBuilder {
    manifest: PluginManifest,
    search_handler: Option<SearchHandler>,
    action_handler: Option<ActionHandler>,
    refresh_handler: Option<RefreshHandler>,
}

impl PluginBuilder {
    /// Create a new plugin builder
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            manifest: PluginManifest {
                id: id.into(),
                name: name.into(),
                version: "0.1.0".to_string(),
                description: String::new(),
                author: String::new(),
                license: "MIT".to_string(),
                homepage: None,
                repository: None,
                icon: None,
                categories: vec![PluginCategory::Utilities],
                keywords: Vec::new(),
                capabilities: PluginCapabilities {
                    search: false,
                    background_refresh: false,
                    notifications: false,
                    shortcuts: false,
                    deep_links: false,
                    clipboard_access: false,
                    file_system_access: false,
                    network_access: false,
                    system_commands: false,
                    ui_extensions: false,
                    context_menu: false,
                    quick_actions: false,
                },
                permissions: PluginPermissions {
                    read_clipboard: false,
                    write_clipboard: false,
                    read_files: Vec::new(),
                    write_files: Vec::new(),
                    execute_commands: Vec::new(),
                    network_hosts: Vec::new(),
                    environment_variables: Vec::new(),
                    system_notifications: false,
                    accessibility: false,
                    camera: false,
                    microphone: false,
                    location: false,
                    contacts: false,
                    calendar: false,
                },
                configuration: Vec::new(),
                preferences: Vec::new(),
                commands: Vec::new(),
                actions: Vec::new(),
                dependencies: HashMap::new(),
                environment: HashMap::new(),
                min_launcher_version: "0.1.0".to_string(),
                max_launcher_version: None,
                update_url: None,
                changelog_url: None,
            },
            search_handler: None,
            action_handler: None,
            refresh_handler: None,
        }
    }

    /// Set plugin version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.manifest.version = version.into();
        self
    }

    /// Set plugin author
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.manifest.author = author.into();
        self
    }

    /// Set plugin description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.manifest.description = desc.into();
        self
    }

    /// Add keywords
    pub fn keywords(mut self, keywords: Vec<String>) -> Self {
        self.manifest.keywords = keywords;
        self
    }

    /// Set search handler
    pub fn on_search<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(String, PluginContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Vec<ActionItem>, Error>> + Send + 'static,
    {
        self.manifest.capabilities.search = true;
        self.search_handler = Some(Arc::new(move |query, ctx| Box::pin(handler(query, ctx))));
        self
    }

    /// Set action handler
    pub fn on_action<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(String, PluginContext, Option<Value>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), Error>> + Send + 'static,
    {
        self.manifest.capabilities.quick_actions = true;
        self.action_handler = Some(Arc::new(move |id, ctx, meta| {
            Box::pin(handler(id, ctx, meta))
        }));
        self
    }

    /// Set refresh handler
    pub fn on_refresh<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(PluginContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), Error>> + Send + 'static,
    {
        self.manifest.capabilities.background_refresh = true;
        self.refresh_handler = Some(Arc::new(move |ctx| Box::pin(handler(ctx))));
        self
    }

    /// Build the plugin
    pub fn build(self) -> Box<dyn LauncherPlugin> {
        struct BuiltPlugin {
            manifest: PluginManifest,
            search_handler: Option<SearchHandler>,
            action_handler: Option<ActionHandler>,
            refresh_handler: Option<RefreshHandler>,
        }

        impl LauncherPlugin for BuiltPlugin {
            fn manifest(&self) -> PluginManifest {
                self.manifest.clone()
            }

            fn search(&self, query: String, context: PluginContext) -> BoxFuture<Vec<ActionItem>> {
                if let Some(handler) = &self.search_handler {
                    handler(query, context)
                } else {
                    Box::pin(async { Ok(Vec::new()) })
                }
            }

            fn execute_action(
                &self,
                action_id: String,
                context: PluginContext,
                metadata: Option<Value>,
            ) -> BoxFuture<()> {
                if let Some(handler) = &self.action_handler {
                    handler(action_id, context, metadata)
                } else {
                    Box::pin(async { Err(Error::PluginError("No action handler".to_string())) })
                }
            }

            fn background_refresh(&self, context: PluginContext) -> BoxFuture<()> {
                if let Some(handler) = &self.refresh_handler {
                    handler(context)
                } else {
                    Box::pin(async { Ok(()) })
                }
            }
        }

        Box::new(BuiltPlugin {
            manifest: self.manifest,
            search_handler: self.search_handler,
            action_handler: self.action_handler,
            refresh_handler: self.refresh_handler,
        })
    }

    /// Build the plugin and return an FFI-safe pointer (for use in extern functions)
    #[allow(dead_code)]
    pub fn build_ffi(self) -> crate::ffi::LauncherPluginFFI {
        let plugin = self.build();
        crate::ffi::ffi_helpers::plugin_to_ffi(plugin)
    }
}

//! FFI-compatible plugin interface implementation

// Note: ActionItem will be used when FFI implementation is completed
use serde_json::Value;

use super::built_plugin::BuiltPlugin;
use crate::plugins::interface::PluginContext;
use crate::plugins::native::create_native_plugin_context_with_bridge;
use crate::service_bridge::bridge::core::ServiceBridge;

// Implement FFI LauncherPlugin for BuiltPlugin
impl crate::plugins::interface::ffi::LauncherPlugin for BuiltPlugin {
    fn manifest(&self) -> crate::plugins::interface::ffi::PluginManifest {
        crate::plugins::interface::ffi::PluginManifest {
            id: self.manifest.id.clone(),
            name: self.manifest.name.clone(),
            version: self.manifest.version.clone(),
            author: self.manifest.author.clone(),
            description: self.manifest.description.clone(),
            keywords: self.manifest.keywords.clone(),
            capabilities: crate::plugins::interface::ffi::PluginCapabilities {
                search: self.manifest.capabilities.search,
                actions: true,
                background_refresh: self.manifest.capabilities.background_refresh,
            },
        }
    }

    fn search(
        &self,
        query: String,
        ffi_context: crate::plugins::interface::ffi::PluginContext,
    ) -> crate::plugins::interface::ffi::BoxFuture<Vec<crate::plugins::interface::ffi::ActionItem>>
    {
        if let Some(handler) = &self.search_handler {
            let plugin_context = match self.create_functional_context(&ffi_context) {
                Ok(ctx) => ctx,
                Err(e) => {
                    return Box::pin(async move { Err(crate::Error::PluginError(e.to_string())) });
                },
            };
            let task = handler(query, plugin_context);
            Box::pin(async move {
                match task.await {
                    Ok(items) => Ok(items
                        .into_iter()
                        .map(|item| crate::plugins::interface::ffi::ActionItem {
                            id: item.id.clone(),
                            title: item.title,
                            subtitle: item.subtitle.unwrap_or_default(),
                            icon: item.icon.map(|icon| match icon {
                                action_items_common::plugin_interface::Icon::Emoji(e) => {
                                    format!("emoji:{e}")
                                },
                                action_items_common::plugin_interface::Icon::BuiltIn(name) => {
                                    format!("builtin:{name}")
                                },
                                action_items_common::plugin_interface::Icon::File(path) => {
                                    format!("file:{}", path.display())
                                },
                                action_items_common::plugin_interface::Icon::Url(url) => {
                                    format!("url:{url}")
                                },
                                action_items_common::plugin_interface::Icon::Base64(data) => {
                                    format!("base64:{data}")
                                },
                            }),
                            score: item.score,
                            action_id: item.id,
                            metadata: item.metadata,
                        })
                        .collect()),
                    Err(e) => Err(crate::Error::PluginError(e.to_string())),
                }
            })
        } else {
            Box::pin(async { Ok(Vec::new()) })
        }
    }

    fn execute_action(
        &self,
        action_id: String,
        ffi_context: crate::plugins::interface::ffi::PluginContext,
        metadata: Option<Value>,
    ) -> crate::plugins::interface::ffi::BoxFuture<()> {
        if let Some(handler) = &self.action_handler {
            let plugin_context = match self.create_functional_context(&ffi_context) {
                Ok(ctx) => ctx,
                Err(e) => {
                    return Box::pin(async move { Err(crate::Error::PluginError(e.to_string())) });
                },
            };
            let task = handler(action_id, plugin_context, metadata);
            Box::pin(async move {
                task.await
                    .map_err(|e| crate::Error::PluginError(e.to_string()))
            })
        } else {
            Box::pin(async { Err(crate::Error::PluginError("No action handler".to_string())) })
        }
    }
}

impl BuiltPlugin {
    /// Create functional plugin context with service bridge integration
    fn create_functional_context(
        &self,
        _ffi_context: &crate::plugins::interface::ffi::PluginContext,
    ) -> Result<PluginContext, crate::Error> {
        let service_bridge = self.get_service_bridge();
        create_native_plugin_context_with_bridge(
            &self.manifest,
            service_bridge,
            &std::path::PathBuf::from("./plugin_storage"),
        )
    }

    /// Get service bridge Arc reference - NEVER clone ServiceBridge
    fn get_service_bridge(&self) -> &std::sync::Arc<ServiceBridge> {
        &self.service_bridge
    }
}

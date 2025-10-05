//! Trait definitions for launcher plugins

use action_items_common::plugin_interface::ActionItem;
use futures::future::BoxFuture;
use serde_json::Value;

use crate::plugins::interface::{Error, NativePlugin, PluginContext, PluginManifest};

/// Trait for LauncherPlugin (should be defined in plugin-native)
pub trait LauncherPlugin: NativePlugin + Send + Sync + 'static {
    /// Get the plugin manifest
    fn manifest(&self) -> &PluginManifest;

    /// Search with the plugin
    fn search(
        &self,
        query: String,
        context: PluginContext,
    ) -> BoxFuture<'static, Result<Vec<ActionItem>, Error>>;

    /// Execute an action
    fn execute_action(
        &self,
        action_id: String,
        context: PluginContext,
        args: Option<Value>,
    ) -> BoxFuture<'static, Result<(), Error>>;

    /// Background refresh (optional)
    fn background_refresh(&self, _context: PluginContext) -> BoxFuture<'static, Result<(), Error>> {
        // Default implementation does nothing
        Box::pin(async { Ok(()) })
    }
}

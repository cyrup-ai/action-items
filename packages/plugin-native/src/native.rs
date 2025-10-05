use action_items_common::plugin_interface::{ActionItem, PluginManifest};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use serde_json::Value;

use super::context::PluginContext;
use crate::Error;

/// Native Rust plugin trait using Bevy tasks
pub trait NativePlugin: Send + Sync {
    /// Get plugin manifest (remains synchronous)
    fn manifest(&self) -> &PluginManifest;

    /// Initialize plugin with context
    fn initialize(
        &mut self,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>>;

    /// Execute a command
    fn execute_command(
        &mut self,
        command_id: String,
        context: PluginContext,
        args: Option<Value>, // Or HashMap<String, Value> if preferred
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>>;

    /// Perform search
    fn search(
        &self, // Typically &self for search, unless it needs to mutate state
        query: String,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Vec<ActionItem>, Error>>;

    /// Handle action execution
    fn execute_action(
        &mut self,
        action_id: String,
        context: PluginContext,
        args: Option<Value>, // Or HashMap<String, Value>
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>>;

    /// Background refresh (if supported)
    fn background_refresh(
        &mut self,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>>;

    /// Clean up resources
    fn cleanup(
        &mut self,
        task_pool: &AsyncComputeTaskPool, // Context might not be needed for cleanup
    ) -> Task<Result<(), Error>>;
}

// Note: Default implementations for methods returning Task might be tricky
// unless they spawn a trivial completed task. It's often better to require
// implementors to provide the full task spawning logic.
// e.g., for background_refresh if not supported:
// fn background_refresh(&mut self, _context: PluginContext, task_pool: &AsyncComputeTaskPool) ->
// Task<Result<(), Error>> { task_pool.spawn(async { Ok(()) })
// }
// However, for now, let's omit default implementations to ensure each plugin explicitly handles it.

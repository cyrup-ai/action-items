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
        _context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>>;

    /// Execute a command
    fn execute_command(
        &mut self,
        command_id: String,
        _context: PluginContext,
        args: Option<Value>, // Or HashMap<String, Value> if preferred
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>>;

    /// Perform search
    fn search(
        &self, // Typically &self for search, unless it needs to mutate state
        query: String,
        _context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Vec<ActionItem>, Error>>;

    /// Handle action execution
    fn execute_action(
        &mut self,
        action_id: String,
        _context: PluginContext,
        args: Option<Value>, // Or HashMap<String, Value>
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>>;

    /// Background refresh (if supported)
    fn background_refresh(
        &mut self,
        _context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>>;

    /// Clean up resources
    fn cleanup(
        &mut self,
        task_pool: &AsyncComputeTaskPool, // Context might not be needed for cleanup
    ) -> Task<Result<(), Error>>;
}



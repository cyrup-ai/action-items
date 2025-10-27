//! Native plugin interface matching plugin-native crate implementation.
//! 
//! This interface is implemented by:
//! - plugin-native::NativePlugin (direct trait implementation)
//! - plugin-native::builder::PluginBuilder (fluent builder pattern)
//! 
//! Both paths produce plugins compatible with the core plugin system.

use super::{PluginContext, PluginManifest};
use super::ActionItem;
use crate::Error;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use serde_json::Value;

/// Native Rust plugin trait using Bevy tasks
/// 
/// This trait defines the standard interface for native (compiled Rust) plugins
/// integrated with the action-items launcher via AsyncComputeTaskPool.
/// 
/// ## Lifecycle
/// 1. `manifest()` - Retrieve plugin metadata and capabilities
/// 2. `initialize()` - Setup plugin state with provided context
/// 3. `search()`, `execute_command()`, `execute_action()` - Core operations
/// 4. `background_refresh()` - Optional periodic updates (if capability enabled)
/// 5. `cleanup()` - Resource teardown
/// 
/// ## Implementation Notes
/// - All async methods must spawn tasks via AsyncComputeTaskPool
/// - background_refresh requires explicit implementation; no default provided
/// - See plugin-native crate for builder patterns and examples
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
    /// 
    /// Implementors must either:
    /// 1. Provide refresh logic using AsyncComputeTaskPool::spawn for actual work
    /// 2. Return PluginCapabilities::background_refresh = false in manifest
    /// 
    /// The builder (PluginBuilder::on_refresh) provides helper methods for registering
    /// refresh handlers. See packages/plugin-native/src/builder.rs:127-135 for pattern.
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

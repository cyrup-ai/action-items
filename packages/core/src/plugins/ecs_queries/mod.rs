//! ECS-based plugin queries module
//!
//! Provides ECS SystemParam-based replacements for PluginRegistry functionality
//! with counter, names, searcher, executor, and system utilities.

pub mod counter;
pub mod executor;
pub mod names;
pub mod resources;
pub mod searcher;
pub mod systems;
pub mod wasm_callback_handler;

// Re-export for backward compatibility
pub use counter::{PluginCounter as Counter, PluginCounter, PluginType, PluginTypeDistribution};
pub use executor::scheduler::PluginExecutor as Executor;
pub use executor::{ExecutionResult, PluginExecutor};
pub use names::{PluginNameGroups, PluginNames as Names, PluginNames};
pub use searcher::{PluginSearcher as Searcher, PluginSearcher, SearchResult};
pub use systems::{
    ActionMap, cleanup_action_mappings_system, handle_search_results_system_ecs,
    log_plugins_system, plugin_query_system, print_plugin_stats,
};
pub use wasm_callback_handler::WasmCallbackHandler;

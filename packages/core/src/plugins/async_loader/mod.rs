//! Async plugin loader module
//!
//! Provides asynchronous plugin loading functionality with progress tracking,
//! event handling, and context creation for different plugin types.

pub mod context;
pub mod events;
pub mod progress;
pub mod systems;
pub mod tasks;

pub use context::{create_plugin_context, create_raycast_plugin_context};
pub use events::{PluginLoadFailed, PluginLoaded, PluginLoadingComplete, PluginLoadingStarted};
pub use progress::{
    PluginLoadingProgress, check_plugin_loading_completion, log_plugin_loading_progress,
};
pub use systems::start_async_plugin_loading;
pub use tasks::{LoadingPlugin, PluginLoadingTask, handle_plugin_loading_tasks};

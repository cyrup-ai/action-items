//! Plugin core modules
//!
//! Modular organization of plugin registry and related functionality.

pub mod components;
pub mod config;
pub mod metadata;
pub mod resources;
pub mod systems;

// Re-export all public types and functions
pub use components::PendingActionResult;
pub use metadata::{ActionItem, PluginMetadata};
pub use resources::CurrentSearchResults;
pub use systems::handle_search_results_system;

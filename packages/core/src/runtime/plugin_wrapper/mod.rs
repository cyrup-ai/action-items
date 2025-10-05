//! Deno plugin wrapper modules
//!
//! Modular organization of Deno plugin wrapper functionality.

pub mod core;
pub mod plugin_component;
pub mod plugin_discovery;
pub mod request_handling;
pub mod runtime_resource;
pub mod systems;

// Re-export main types and functions
pub use core::DenoPluginWrapper;

pub use plugin_component::DenoPluginComponent;
pub use systems::{
    execute_deno_plugin_system, handle_deno_action_item_requests_system, handle_deno_events_system,
    handle_deno_plugin_loading_system, monitor_deno_runtime_system,
};

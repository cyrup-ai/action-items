pub mod context;
pub mod events;
pub mod extism;
pub mod traits;
pub mod views;

// Import canonical types from common - ONE UNIFIED SYSTEM
pub use action_items_common::plugin_interface::{
    ActionDefinition, ActionItem, CommandDefinition, ConfigurationField, PluginCapabilities,
    PluginCategory, PluginManifest, PluginPermissions, PreferenceField,
};
// Re-export WASM implementation details only
pub use context::PluginContext;
pub use events::*;
pub use extism::*;
pub use traits::*;
pub use views::*;

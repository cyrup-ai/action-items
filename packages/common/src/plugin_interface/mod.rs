//! Plugin interface types and traits shared between core and native plugins

pub mod action_item;
pub mod capabilities;
pub mod commands;
pub mod config;
pub mod manifest;

// Re-export action item types for convenience
pub use action_item::{ActionItem, ActionType, Icon, ItemAction, ItemBadge, Shortcut};
// Re-export supporting types for convenience
pub use capabilities::{PluginCapabilities, PluginPermissions};
pub use commands::{
    ActionDefinition, ArgumentDefinition, ArgumentType, CommandDefinition, CommandMode,
};
pub use config::{
    ConfigFieldType, ConfigurationField, PreferenceField, PreferenceType, SelectOption,
    ValidationRule,
};
// Re-export manifest types for convenience
pub use manifest::{PluginCategory, PluginManifest};

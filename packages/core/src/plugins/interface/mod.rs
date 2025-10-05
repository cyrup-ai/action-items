//! Core plugin interface for action items
//!
//! This module provides the core types and traits for the plugin system.
//! All types are re-exported from their canonical sources to avoid ambiguity.

// Re-export common types from action_items_common
pub use action_items_common::plugin_interface::{
    ActionItem, ActionType, Icon, ItemAction, ItemBadge,
};
// Re-export common types
pub use action_items_common::plugin_interface::{
    PluginCapabilities, PluginCategory, PluginManifest, PluginPermissions,
};
// Re-export native plugin trait and context types
pub use action_items_native::{
    context::{
        CacheService,

        // Service access types
        ClipboardAccess,
        // Service access types
        CommandResult,

        HttpClient,
        HttpMethod,

        HttpRequest,
        // Response types
        HttpResponseData,
        NotificationService,
        // Core context types
        PluginContext,
        StorageService,
    },
    error::Error,
    native::NativePlugin,
};

// Re-export local modules
pub mod capabilities;
pub mod commands;
pub mod context;
pub mod extism;
pub mod ffi;
pub mod search_result;

// Re-export local types
pub use self::commands::{ClipboardCommand, HttpCommand, NotificationCommand, StorageCommand};
pub use self::search_result::SearchResult;

// Define plugin interface constants
/// Required exports for all plugins
pub const REQUIRED_EXPORTS: &[&str] = &["init", "search", "execute_command"];

/// Optional exports that plugins may implement
pub const OPTIONAL_EXPORTS: &[&str] = &["execute_action", "background_refresh", "cleanup"];

/// Host functions available to plugins
pub const HOST_FUNCTIONS: &[&str] = &[
    "log",
    "http_request",
    "clipboard_read",
    "clipboard_write",
    "show_notification",
];

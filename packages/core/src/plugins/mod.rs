//! Plugin system architecture
//!
//! Zero-allocation plugin system with blazing-fast modular organization supporting multiple
//! plugin types including Extism WASM plugins and native plugins. Includes ECS integration,
//! service bridging, and plugin registry management.

// Re-export core plugin functionality (maintains backward compatibility)
pub use core::ActionItem; // Primary ActionItem definition
// Re-export specific items to avoid ambiguity
pub use core::{CurrentSearchResults, PendingActionResult, handle_search_results_system};

// Specific re-exports from async_loader to avoid ambiguity
pub use async_loader::{
    LoadingPlugin, PluginLoadFailed, PluginLoaded, PluginLoadingComplete, PluginLoadingProgress,
    PluginLoadingStarted, PluginLoadingTask, check_plugin_loading_completion,
    create_plugin_context, create_raycast_plugin_context, handle_plugin_loading_tasks,
    log_plugin_loading_progress, start_async_plugin_loading,
};
// Specific re-exports from bridge to avoid ambiguity
pub use bridge::{
    ClipboardAccess, HttpClient, HttpResponse, NotificationService, ServiceBridgePlugin,
    ServiceRequest, ServiceResponse, SharedServiceBridge, StorageService, bridge_service_system,
    handle_clipboard_read, handle_clipboard_write, handle_http_request, handle_notification,
    handle_storage_delete, handle_storage_read, handle_storage_write, process_service_request,
    service_bridge_cleanup_system, service_bridge_monitor_system, service_bridge_system,
    wasm_callback_system_ecs,
};
pub use builder::*;
pub use ecs_queries::*;
pub use extism::wrapper::PluginMetadata as ExtismPluginMetadata;
// Re-export plugin type modules (specific imports to avoid conflicts)
pub use extism::{
    ExtismHostUserData, ExtismPlugin, ExtismPluginAdapter, ExtismPluginLoader,
    create_host_functions,
};
pub use interface::ActionItem as InterfaceActionItem; // Interface version with different name
pub use native::wrapper::PluginMetadata as NativePluginMetadata;
// native_plugin module removed - use ECS plugin components instead
pub use services::{PluginCache, StorageDirectory};

// Module declarations
pub mod async_loader;
pub mod bridge;
pub mod builder;
pub mod core;
pub mod ecs_queries;
pub mod extism;
pub mod interface;
pub mod native;
pub mod service_bridge_integration;
pub mod services;

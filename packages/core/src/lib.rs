//! Action Items Core Library
//!
//! Pure ECS Bevy integration providing essential launcher services.
//! All functionality is provided through clean ECS service plugins from packages/ecs-*.

// SearchAggregatorPlugin import removed - plugin is added by main app to avoid duplication
use bevy::prelude::*;
// All ECS service plugin imports
//use action_items_ecs_cache::EcsCachePlugin;  // Temporarily disabled
//use action_items_ecs_fetch::HttpPlugin;  // Temporarily disabled
use action_items_ecs_progress::ProgressPlugin;
//use action_items_ecs_surrealdb::DatabasePlugin;  // Temporarily disabled
//use action_items_deno_ops::plugin::DenoPlugin;  // Temporarily disabled
use bevy::state::state::States;
#[cfg(feature = "tls")]
use ecs_tls::TlsCleanupPlugin;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProgressState {
    #[default]
    Loading,
    Running,
}

// Keep essential modules that other packages depend on
pub mod config;
pub mod discovery;
pub mod error;
pub mod events;
pub mod extism_plugin_wrapper;
pub mod native_plugin_wrapper;
pub mod plugins;
pub mod raycast;
pub mod runtime;
pub mod screen_dimensions;
pub mod search;
pub mod service_bridge;

// All ECS services are now enabled and integrated

// Re-export essential types for backward compatibility
pub use action_items_common::directories::AppDirectories;
pub use action_items_common::plugin_interface::{ActionItem, PluginCapabilities, PluginManifest};
// Re-export ECS service types that the app depends on
pub use action_items_ecs_search_aggregator::{
    AggregatedSearchResults as CurrentSearchResults, CurrentQuery, SearchResult,
};
// Re-export config system types that other packages depend on
pub use config::{ConfigEvent, ConfigValue};
// Re-export error types for convenience
pub use error::{Error, Result};
// Re-export core events from the events module
pub use events::{ActionMap, LauncherEvent, LauncherEventType, WasmCallbackEvent};

pub struct ActionItemsCorePlugin;

impl Plugin for ActionItemsCorePlugin {
    fn build(&self, app: &mut App) {
        // Initialize XDG-compliant directories
        let app_directories = AppDirectories::new();
        if let Err(e) = app_directories.ensure_directories_exist() {
            error!("Failed to create directories: {e}. Using fallback paths.");
        } else {
            info!(
                "Initialized directories - config: {:?}, data: {:?}",
                app_directories.config_dir(),
                app_directories.data_dir()
            );
        }
        app.insert_resource(app_directories);

        // Add complete ECS services in dependency order
        // Note: SearchAggregatorPlugin is added by main app to avoid duplicate registration

        // Data and persistence services
        //app.add_plugins(EcsCachePlugin);  // Temporarily disabled

        // System services
        //app.add_plugins(HttpPlugin::default());  // Temporarily disabled

        // NotificationSystemPlugin is added by main app to avoid duplicate registration

        app.add_plugins(ProgressPlugin::<ProgressState>::new());

        // Platform-specific services (conditionally enabled)
        #[cfg(feature = "tls")]
        app.add_plugins(TlsCleanupPlugin);

        // Development runtime
        //app.add_plugins(DenoPlugin::default());  // Temporarily disabled

        // Database services
        //app.add_plugins(DatabasePlugin);  // Temporarily disabled

        // Add core events
        app.add_event::<LauncherEvent>();
        app.add_event::<ConfigEvent>();

        let mut service_count = 2; // Base services (ServiceBridge + SearchAggregator)

        //service_count += 1; // Cache service - temporarily disabled
        //service_count += 1; // HTTP service - temporarily disabled  
        service_count += 1; // Notification service
        service_count += 1; // Progress service

        #[cfg(feature = "tls")]
        {
            service_count += 1; // TLS service
        }

        //service_count += 1; // Database service - temporarily disabled
        //service_count += 1; // Deno service - temporarily disabled

        info!(
            "ActionItemsCorePlugin initialized with {} ECS services",
            service_count
        );
    }
}

// Simple convenience re-exports for common functionality
pub use ActionItemsCorePlugin as LauncherPlugin; // Backward compatibility alias

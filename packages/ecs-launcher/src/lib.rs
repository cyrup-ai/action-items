//! ECS Launcher Service
//!
//! A comprehensive ECS service for managing launcher functionality with support for:
//! - Action item execution and search
//! - UI management and interactions
//! - Plugin integration and coordination
//! - Event-driven architecture for loose coupling
//!
//! # Usage
//!
//! ```rust
//! use bevy::prelude::*;
//! use ecs_launcher::{ActionExecuteRequested, LauncherPlugin};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(LauncherPlugin::default())
//!         .add_systems(Startup, setup_launcher)
//!         .run();
//! }
//!
//! fn setup_launcher(mut events: EventWriter<ActionExecuteRequested>) {
//!     events.send(ActionExecuteRequested {
//!         action_id: "example_action".to_string(),
//!         requester: "my_service".to_string(),
//!         parameters: serde_json::Value::Null,
//!     });
//! }
//! ```

pub mod components;
pub mod events;
pub mod integrations;
pub mod resources;
pub mod systems;

// Re-export main types for easy access
use std::time::Duration;

use bevy::prelude::*;
pub use components::*;
pub use events::*;
pub use integrations::*;
pub use resources::*;
pub use systems::*;

/// Main ECS Launcher Plugin
///
/// Provides comprehensive launcher management including action execution,
/// search functionality, UI coordination, and plugin integration.
#[derive(Default)]
pub struct LauncherPlugin {
    /// Enable detailed logging for debugging
    pub enable_debug_logging: bool,
    /// Search update interval for continuous search
    pub search_update_interval: Duration,
    /// Maximum number of search results to display
    pub max_search_results: usize,
    /// Enable automatic plugin discovery
    pub enable_plugin_discovery: bool,
}

impl LauncherPlugin {
    /// Create new launcher plugin with default configuration
    pub fn new() -> Self {
        Self {
            enable_debug_logging: false,
            search_update_interval: Duration::from_millis(100),
            max_search_results: 50,
            enable_plugin_discovery: true,
        }
    }

    /// Enable debug logging for launcher operations
    pub fn with_debug_logging(mut self, enabled: bool) -> Self {
        self.enable_debug_logging = enabled;
        self
    }

    /// Set search update interval
    pub fn with_search_interval(mut self, interval: Duration) -> Self {
        self.search_update_interval = interval;
        self
    }

    /// Set maximum number of search results
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_search_results = max;
        self
    }

    /// Enable/disable automatic plugin discovery
    pub fn with_plugin_discovery(mut self, enabled: bool) -> Self {
        self.enable_plugin_discovery = enabled;
        self
    }

    /// Configure for development mode (debug logging, frequent updates)
    pub fn development_mode(mut self) -> Self {
        self.enable_debug_logging = true;
        self.search_update_interval = Duration::from_millis(50);
        self.enable_plugin_discovery = true;
        self
    }

    /// Configure for production mode (optimized intervals, no debug logging)
    pub fn production_mode(mut self) -> Self {
        self.enable_debug_logging = false;
        self.search_update_interval = Duration::from_millis(200);
        self.enable_plugin_discovery = false; // Let apps control discovery
        self
    }
}

impl Plugin for LauncherPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing ECS Launcher Service Plugin");

        // Initialize resources
        app.insert_resource(LauncherState::default())
            .insert_resource(SearchState::default())
            .insert_resource(LauncherMetrics::default())
            .insert_resource(ActionRegistry::default())
            .insert_resource(PluginRegistry::default())
            .insert_resource(LauncherConfig {
                enable_debug_logging: self.enable_debug_logging,
                search_update_interval: self.search_update_interval,
                max_search_results: self.max_search_results,
                enable_plugin_discovery: self.enable_plugin_discovery,
            });

        // Add all launcher events
        app.add_event::<ActionExecuteRequested>()
            .add_event::<ActionExecuteCompleted>()
            .add_event::<ActionRegisterRequested>()
            .add_event::<SearchRequested>()
            .add_event::<SearchCompleted>()
            .add_event::<SearchQueryChanged>()
            .add_event::<PluginDiscoveryRequested>()
            .add_event::<UIStateChanged>()
            .add_event::<PluginDiscovered>()
            .add_event::<LauncherWindowToggled>()
            .add_event::<LauncherPreferencesUpdated>();

        // Add core systems
        app.add_systems(
            Update,
            (
                // Action execution
                process_action_execution_requests_system.in_set(LauncherSystemSet::ActionExecution),
                poll_action_execution_tasks.in_set(LauncherSystemSet::ActionExecution),
                // Search management
                process_search_requests_system.in_set(LauncherSystemSet::Search),
                update_search_results_system.in_set(LauncherSystemSet::Search),
                poll_search_tasks.in_set(LauncherSystemSet::Search),
                // UI management
                process_ui_state_changes_system.in_set(LauncherSystemSet::UIManagement),
                manage_launcher_window_system.in_set(LauncherSystemSet::UIManagement),
                // Plugin discovery and integration
                discover_plugins_system.in_set(LauncherSystemSet::PluginDiscovery),
                integrate_discovered_plugins_system.in_set(LauncherSystemSet::PluginDiscovery),
                poll_plugin_discovery_tasks.in_set(LauncherSystemSet::PluginDiscovery),
                // Cleanup and metrics
                cleanup_completed_operations_system.in_set(LauncherSystemSet::Cleanup),
                update_launcher_metrics_system.in_set(LauncherSystemSet::Metrics),
            ),
        );

        // Configure system ordering
        app.configure_sets(
            Update,
            (
                LauncherSystemSet::ActionExecution,
                LauncherSystemSet::Search,
                LauncherSystemSet::UIManagement,
                LauncherSystemSet::PluginDiscovery,
                LauncherSystemSet::Cleanup,
                LauncherSystemSet::Metrics,
            )
                .chain(),
        );

        info!("ECS Launcher Service Plugin initialized successfully");
        info!("  - Max search results: {}", self.max_search_results);
        info!(
            "  - Search update interval: {:?}",
            self.search_update_interval
        );
        info!("  - Debug logging: {}", self.enable_debug_logging);
        info!("  - Plugin discovery: {}", self.enable_plugin_discovery);
    }
}

/// System sets for organizing launcher-related systems
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum LauncherSystemSet {
    /// Action execution processing
    ActionExecution,
    /// Search functionality
    Search,
    /// UI management and interactions
    UIManagement,
    /// Plugin discovery and integration
    PluginDiscovery,
    /// Cleanup of completed operations
    Cleanup,
    /// Metrics collection and reporting
    Metrics,
}

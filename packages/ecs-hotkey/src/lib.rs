//! ECS Hotkey Service
//!
//! A comprehensive ECS service for managing global hotkeys with support for:
//! - Programmable hotkey registration and conflict detection
//! - Real-time hotkey capture for user customization
//! - Cross-platform global hotkey polling
//! - Event-driven architecture for loose coupling
//!
//! # Usage
//!
//! ```rust
//! use bevy::prelude::*;
//! use ecs_hotkey::{HotkeyDefinition, HotkeyPlugin, HotkeyRegisterRequested};
//! use global_hotkey::hotkey::{Code, Modifiers};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(HotkeyPlugin::default())
//!         .add_systems(Startup, register_hotkeys)
//!         .run();
//! }
//!
//! fn register_hotkeys(mut events: EventWriter<HotkeyRegisterRequested>) {
//!     let hotkey_def = HotkeyDefinition {
//!         modifiers: Modifiers::META | Modifiers::SHIFT,
//!         code: Code::Space,
//!         description: "Cmd+Shift+Space".to_string(),
//!     };
//!
//!     events.write(HotkeyRegisterRequested {
//!         binding: HotkeyBinding::new(hotkey_def, "my_action"),
//!         requester: "my_service".to_string(),
//!     });
//! }
//! ```

pub mod capture;
pub mod components;
pub mod conflict;
pub mod events;
pub mod platform;
pub mod resources;
pub mod systems;

// Re-export main types for easy access
use std::time::Duration;

use bevy::prelude::*;
pub use capture::*;
pub use components::*;
pub use conflict::*;
pub use events::*;
pub use platform::*;
pub use resources::*;
pub use systems::*;

/// Main ECS Hotkey Plugin
///
/// Provides comprehensive hotkey management including registration, conflict detection,
/// real-time capture, and cross-platform global hotkey polling.
#[derive(Default)]
pub struct HotkeyPlugin {
    /// Enable detailed logging for debugging
    pub enable_debug_logging: bool,
    /// Polling interval for global hotkey detection
    pub polling_interval: Duration,
    /// Maximum number of concurrent hotkey registrations
    pub max_hotkeys: usize,
    /// Enable automatic conflict resolution
    pub enable_conflict_resolution: bool,
}

impl HotkeyPlugin {
    /// Create new hotkey plugin with default configuration
    pub fn new() -> Self {
        Self {
            enable_debug_logging: false,
            polling_interval: Duration::from_millis(10),
            max_hotkeys: 64,
            enable_conflict_resolution: true,
        }
    }

    /// Enable debug logging for hotkey operations
    pub fn with_debug_logging(mut self, enabled: bool) -> Self {
        self.enable_debug_logging = enabled;
        self
    }

    /// Set polling interval for global hotkey detection
    pub fn with_polling_interval(mut self, interval: Duration) -> Self {
        self.polling_interval = interval;
        self
    }

    /// Set maximum number of concurrent hotkey registrations
    pub fn with_max_hotkeys(mut self, max: usize) -> Self {
        self.max_hotkeys = max;
        self
    }

    /// Enable/disable automatic conflict resolution
    pub fn with_conflict_resolution(mut self, enabled: bool) -> Self {
        self.enable_conflict_resolution = enabled;
        self
    }

    /// Configure for development mode (debug logging, frequent polling)
    pub fn development_mode(mut self) -> Self {
        self.enable_debug_logging = true;
        self.polling_interval = Duration::from_millis(5);
        self.enable_conflict_resolution = true;
        self
    }

    /// Configure for production mode (optimized polling, no debug logging)
    pub fn production_mode(mut self) -> Self {
        self.enable_debug_logging = false;
        self.polling_interval = Duration::from_millis(15);
        self.enable_conflict_resolution = false; // Let apps handle conflicts
        self
    }
}

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing ECS Hotkey Service Plugin");

        // Initialize resources
        app.insert_resource(HotkeyManager::new(
            self.max_hotkeys,
            self.enable_conflict_resolution,
        ))
        .insert_resource(HotkeyRegistry::default())
        .insert_resource(HotkeyCaptureState::default())
        .insert_resource(HotkeyMetrics::default())
        .insert_resource(HotkeyPreferences::default())
        .insert_resource(HotkeyConfig {
            enable_debug_logging: self.enable_debug_logging,
            polling_interval: self.polling_interval,
            max_hotkeys: self.max_hotkeys,
            enable_conflict_resolution: self.enable_conflict_resolution,
        });

        // Add all hotkey events
        app.add_event::<HotkeyRegisterRequested>()
            .add_event::<HotkeyRegisterCompleted>()
            .add_event::<HotkeyUnregisterRequested>()
            .add_event::<HotkeyUnregisterCompleted>()
            .add_event::<HotkeyCaptureStarted>()
            .add_event::<HotkeyCaptureCompleted>()
            .add_event::<HotkeyCaptureCancelled>()
            .add_event::<HotkeyPressed>()
            .add_event::<HotkeyConflictDetected>()
            .add_event::<HotkeyTestRequested>()
            .add_event::<HotkeyTestResult>()
            .add_event::<HotkeyPreferencesUpdated>();

        // Add platform-specific startup systems
        #[cfg(target_os = "macos")]
        app.add_systems(Startup, crate::platform::macos::setup_macos_hotkey_system);

        // Add core systems
        app.add_systems(
            Update,
            (
                // Hotkey management
                (process_hotkey_registration_requests_system,)
                    .in_set(HotkeySystemSet::Registration),
                (process_hotkey_unregistration_requests_system,)
                    .in_set(HotkeySystemSet::Registration),
                (detect_hotkey_conflicts_system,).in_set(HotkeySystemSet::ConflictDetection),
                (validate_hotkey_combinations_system,).in_set(HotkeySystemSet::ConflictDetection),
                // Hotkey detection and polling - platform-specific
                #[cfg(target_os = "macos")]
                (crate::platform::macos::process_macos_hotkey_events_system,).in_set(HotkeySystemSet::Detection),
                #[cfg(target_os = "macos")]
                (crate::platform::macos::register_hotkey_with_macos_system,).in_set(HotkeySystemSet::Registration),
                #[cfg(not(target_os = "macos"))]
                (hotkey_polling_system,).in_set(HotkeySystemSet::Detection),
                (process_hotkey_pressed_events_system,).in_set(HotkeySystemSet::EventProcessing),
                // Real-time capture
                (process_hotkey_capture_requests_system,).in_set(HotkeySystemSet::Capture),
                // Temporarily disable real_hotkey_capture_system due to Optional EventWriter issue
                // real_hotkey_capture_system.in_set(HotkeySystemSet::Capture),
                // Testing and preferences
                (process_hotkey_test_requests_system,).in_set(HotkeySystemSet::Testing),
                (manage_hotkey_preferences_system,).in_set(HotkeySystemSet::Preferences),
                (poll_hotkey_preferences_persist_tasks,).in_set(HotkeySystemSet::Preferences),
                // Cleanup and metrics
                (cleanup_completed_hotkey_operations_system,).in_set(HotkeySystemSet::Cleanup),
                (update_hotkey_metrics_system,).in_set(HotkeySystemSet::Metrics),
            ),
        );

        // Configure system ordering
        app.configure_sets(
            Update,
            (
                HotkeySystemSet::Registration,
                HotkeySystemSet::ConflictDetection,
                HotkeySystemSet::Detection,
                HotkeySystemSet::EventProcessing,
                HotkeySystemSet::Capture,
                HotkeySystemSet::Testing,
                HotkeySystemSet::Preferences,
                HotkeySystemSet::Cleanup,
                HotkeySystemSet::Metrics,
            )
                .chain(),
        );

        info!("ECS Hotkey Service Plugin initialized successfully");
        info!("  - Max hotkeys: {}", self.max_hotkeys);
        info!("  - Polling interval: {:?}", self.polling_interval);
        info!("  - Debug logging: {}", self.enable_debug_logging);
        info!(
            "  - Conflict resolution: {}",
            self.enable_conflict_resolution
        );
    }
}

/// System sets for organizing hotkey-related systems
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum HotkeySystemSet {
    /// Hotkey registration and unregistration processing
    Registration,
    /// Conflict detection and validation
    ConflictDetection,
    /// Global hotkey detection and polling
    Detection,
    /// Process hotkey press events
    EventProcessing,
    /// Real-time hotkey capture for user programming
    Capture,
    /// Hotkey testing functionality
    Testing,
    /// Preferences management
    Preferences,
    /// Cleanup of completed operations
    Cleanup,
    /// Metrics collection and reporting
    Metrics,
}

// HotkeyConfig moved to resources.rs

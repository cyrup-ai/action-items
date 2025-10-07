//! # ecs-hotkey - ECS Global Hotkey Service
//!
//! ## Architecture
//!
//! This crate uses **TWO complementary hotkey systems**:
//!
//! ### 1. Registration & Firing (via global-hotkey crate)
//! - Register predefined hotkeys with the OS
//! - Emit `HotkeyPressed` events when hotkeys are triggered
//! - Cross-platform: Windows (RegisterHotKey), Linux (XGrabKey), macOS (Carbon)
//! - **Only monitors registered hotkeys** (cannot detect arbitrary keypresses)
//!
//! ### 2. Capture (via platform-specific APIs)
//! - Record arbitrary user keypresses during "Press your hotkey..." UI
//! - Required because global-hotkey cannot detect unregistered combinations
//! - **Platform status**:
//!   - ✅ macOS: CGEventTap (617 lines, lock-free atomics)
//!   - 🟡 Windows: TODO - Need WH_KEYBOARD_LL hooks
//!   - 🟡 Linux: TODO - Need XRecordExtension/Wayland integration
//!
//! **See [`ARCHITECTURE.md`](../ARCHITECTURE.md) for detailed design rationale.**
//!
//! ## Usage
//! ```rust,no_run
//! use bevy::prelude::*;
//! use ecs_hotkey::{HotkeyDefinition, HotkeyPlugin, HotkeyRegisterRequested};
//!
//! App::new()
//!     .add_plugins(HotkeyPlugin::default())
//!     .add_systems(Startup, register_hotkeys)
//!     .run();
//! # fn register_hotkeys() {}
//! ```

pub mod capture;
pub mod components;
pub mod conflict;
pub mod events;
pub mod feedback;
pub mod platform;
pub mod resources;
pub mod systems;
pub mod system_hotkeys;

// Re-export main types for easy access
use std::time::Duration;

use bevy::prelude::*;
pub use capture::*;
pub use components::*;
pub use conflict::*;
pub use events::*;
pub use feedback::*;
pub use platform::*;
pub use resources::*;
pub use systems::*;
pub use system_hotkeys::*;

// Re-export configuration import/export
pub use resources::{
    export_config,
    import_config,
    merge_bindings,
    ExportedConfig,
    SerializableHotkeyBinding,
    MergeStrategy,
};

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

        // Validate platform permissions before initializing
        if let Err(e) = crate::platform::check_platform_permissions() {
            error!("❌ Platform hotkey permissions check failed: {}", e);
            error!("⚠️  Hotkey functionality may not work correctly");
        }

        // Initialize resources with Wayland support on Linux
        let hotkey_manager_opt: Option<HotkeyManager> = {
            #[cfg(target_os = "linux")]
            {
                if crate::platform::is_wayland() {
                    let compositor = crate::platform::detect_compositor();
                    info!("Detected Wayland compositor: {:?}", compositor);

                    match compositor {
                        crate::platform::LinuxCompositor::Kde |
                        crate::platform::LinuxCompositor::Hyprland => {
                            // Try initializing Wayland backend
                            let wayland_result = tokio::runtime::Runtime::new()
                                .map_err(|e| format!("Failed to create tokio runtime: {}", e))
                                .and_then(|rt| {
                                    rt.block_on(async {
                                        crate::platform::linux_wayland::WaylandHotkeyManager::new().await
                                            .map_err(|e| format!("Wayland backend init failed: {}", e))
                                    })
                                });

                            match wayland_result {
                                Ok(wayland_mgr) => {
                                    info!("✅ Wayland native hotkey support initialized");
                                    match GlobalHotKeyManager::new() {
                                        Ok(global_manager) => {
                                            Some(HotkeyManager {
                                                global_manager,
                                                max_hotkeys: self.max_hotkeys,
                                                enable_conflict_resolution: self.enable_conflict_resolution,
                                                wayland_manager: Some(std::sync::Arc::new(tokio::sync::Mutex::new(wayland_mgr))),
                                            })
                                        }
                                        Err(e) => {
                                            error!("GlobalHotKeyManager creation failed: {}", e);
                                            error!("Hotkey functionality will be disabled for this session");
                                            None
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Wayland backend initialization failed: {}, falling back to X11", e);
                                    HotkeyManager::new(self.max_hotkeys, self.enable_conflict_resolution).ok()
                                }
                            }
                        }
                        _ => {
                            // Unsupported compositor - use X11 fallback
                            info!("Compositor {:?} not supported for native Wayland hotkeys, using X11/XWayland", compositor);
                            HotkeyManager::new(self.max_hotkeys, self.enable_conflict_resolution).ok()
                        }
                    }
                } else {
                    // X11 session on Linux
                    HotkeyManager::new(self.max_hotkeys, self.enable_conflict_resolution).ok()
                }
            }

            #[cfg(not(target_os = "linux"))]
            {
                HotkeyManager::new(self.max_hotkeys, self.enable_conflict_resolution).ok()
            }
        };

        if let Some(hotkey_manager) = hotkey_manager_opt {
            app.insert_resource(hotkey_manager);
        } else {
            error!("Failed to initialize HotkeyManager");
            error!("Hotkey functionality will be disabled for this session");
        }
        
        app
        .insert_resource(HotkeyRegistry::default())
        .insert_resource(HotkeyCaptureState::default())
        .insert_resource(HotkeyCaptureUIState::default())
        .insert_resource(MultiCaptureState::default())
        .insert_resource(HotkeyMetrics::default())
        .insert_resource(HotkeyPreferences::default())
        .insert_resource(HotkeyAnalytics::default())
        .insert_resource(HotkeyEntityMap::default())
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
            .add_event::<HotkeyCaptureRequested>()
            .add_event::<HotkeyCaptureCompleted>()
            .add_event::<HotkeyCaptureCancelled>()
            .add_event::<HotkeyPressed>()
            .add_event::<HotkeyConflictDetected>()
            .add_event::<HotkeyTestRequested>()
            .add_event::<HotkeyTestResult>()
            .add_event::<HotkeyPreferencesUpdated>()
            .add_event::<HotkeyProfileSwitchRequested>()
            .add_event::<HotkeyProfileSwitchCompleted>()
            .add_event::<HotkeyProfileCreated>()
            .add_event::<HotkeyProfileDeleted>()
            .add_event::<HotkeyProfilesUpdated>()
            .add_event::<HotkeyVisualFeedback>();

        // Add platform-specific startup systems
        #[cfg(target_os = "macos")]
        app.add_systems(Startup, crate::platform::macos::setup_macos_hotkey_system);

        // Display platform-specific hotkey information at startup
        app.add_systems(Startup, display_platform_hotkey_info_system);

        // Add profile loading startup system
        app.add_systems(Startup, load_hotkey_profiles_startup_system);

        // Add core systems (split into multiple calls to avoid tuple size limit)
        app.add_systems(
            Update,
            (
                // Profile management
                (process_profile_switch_requests_system,).in_set(HotkeySystemSet::ProfileManagement),
                (manage_hotkey_profiles_persistence_system,).in_set(HotkeySystemSet::ProfileManagement),
                (poll_hotkey_profiles_persist_tasks,).in_set(HotkeySystemSet::ProfileManagement),
                (poll_hotkey_profiles_load_tasks,).in_set(HotkeySystemSet::ProfileManagement),
                // Hotkey management
                #[cfg(not(target_os = "macos"))]
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
                #[cfg(target_os = "linux")]
                (poll_wayland_hotkey_events_system,).in_set(HotkeySystemSet::Detection),
                (process_hotkey_pressed_events_system,).in_set(HotkeySystemSet::EventProcessing),
                // Feedback systems
                (spawn_hotkey_feedback_system,).in_set(HotkeySystemSet::EventProcessing),
            ),
        );
        
        // Add capture, testing, preferences, cleanup systems
        app.add_systems(
            Update,
            (
                // Real-time capture
                (process_hotkey_capture_requests_system,).in_set(HotkeySystemSet::Capture),
                (real_hotkey_capture_system,).in_set(HotkeySystemSet::Capture),
                // Multi-session capture
                (process_multi_capture_requests_system,).in_set(HotkeySystemSet::Capture),
                (multi_session_capture_system,).in_set(HotkeySystemSet::Capture),
                (process_multi_capture_cancellations_system,).in_set(HotkeySystemSet::Capture),
                // Testing and preferences
                (process_hotkey_test_requests_system,).in_set(HotkeySystemSet::Testing),
                (manage_hotkey_preferences_system,).in_set(HotkeySystemSet::Preferences),
                (poll_hotkey_preferences_persist_tasks,).in_set(HotkeySystemSet::Preferences),
                // Cleanup and metrics
                (cleanup_despawned_hotkey_guards_system,).in_set(HotkeySystemSet::Cleanup),
                (cleanup_completed_hotkey_operations_system,).in_set(HotkeySystemSet::Cleanup),
                (cleanup_feedback_ui_system,).in_set(HotkeySystemSet::Cleanup),
                (update_hotkey_metrics_system,).in_set(HotkeySystemSet::Metrics),
            ),
        );

        // Configure system ordering
        app.configure_sets(
            Update,
            (
                HotkeySystemSet::ProfileManagement,
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
    /// Profile management and switching
    ProfileManagement,
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

/// Display platform-specific hotkey information at startup
///
/// Shows user-friendly messages about hotkey readiness and platform details
fn display_platform_hotkey_info_system() {
    #[cfg(target_os = "windows")]
    crate::platform::display_windows_hotkey_info();
    
    #[cfg(target_os = "linux")]
    crate::platform::display_linux_hotkey_info();
    
    #[cfg(target_os = "macos")]
    crate::platform::display_macos_hotkey_info();
}

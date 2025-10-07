use action_items_core::{ActionItemsCorePlugin, LauncherEvent};
// Complete ECS service ecosystem - ALL SERVICES ACTIVE
use action_items_ecs_bluetooth::BluetoothPlugin;
use action_items_ecs_clipboard::ClipboardPlugin;
use action_items_ecs_compression::CompressionPlugin;
use action_items_ecs_permissions::{PermissionPlugin, PermissionWizardPlugin, PermissionType};
use action_items_ecs_search::{SearchPlugin, SearchUIPlugin};
use action_items_ecs_settings::{SettingsPlugin, SettingsUIPlugin};
use action_items_ecs_preferences::{PreferencesPlugin, PreferencesUIPlugin};
use action_items_ecs_search_aggregator::SearchAggregatorPlugin;
use action_items_ecs_fetch::HttpPlugin;
use action_items_ecs_progress::ProgressPlugin;
use action_items_ecs_ui::UiLunexPlugins; // UI system coordination - ENABLED ✅
// Note: UI prelude will be used when app configuration UI is implemented
use action_items_ui::{LauncherUiPlugin, MonitorConstraintsPlugin, UiVisibilityEvent};
use bevy::prelude::*;
use bevy::state::state::States;
// Wizard now handled by ecs-permissions service
use ecs_notifications::{NotificationBuilder, Platform};
use bevy::log::{LogPlugin, Level, BoxedLayer};
#[cfg(target_os = "macos")]
use bevy::window::CompositeAlphaMode;
use bevy::window::{Window, WindowLevel, WindowMode};
use bevy::winit::{WakeUp, WinitPlugin};
use ecs_filesystem::FileSystemPlugin;
use ecs_hotkey::HotkeyPlugin;
use ecs_launcher::{HotkeyLauncherBridgePlugin, LauncherPlugin as EcsLauncherService};
use ecs_notifications::NotificationSystemPlugin;
use action_items_ecs_cache::EcsCachePlugin;
use action_items_ecs_deno::DenoPlugin;
use ecs_service_bridge::ServiceBridgePlugin;
use ecs_task_management::TaskManagementPlugin;
use action_items_ecs_surrealdb::DatabasePlugin;
use action_items_ecs_user_settings::UserSettingsPlugin;
use ecs_tls::TlsCleanupPlugin;

use crate::events::handlers::preferences::PendingFileOperations;
use crate::events::{GlobalHotkeyEvent, PreferencesEvent};
use crate::input::{LauncherHotkeys, SearchQuery, TextInputChanged};
use crate::overlay_window::OverlayWindowPlugin;
// Permissions now handled by ECS service
use action_items_ecs_preferences::PreferencesResource;
// Search debouncing now handled by ECS search aggregator
use crate::window::{ActiveMonitor, LauncherState, WindowActivationPlugin};

// Application states for context-aware input handling
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Background,
    LauncherActive,
    SearchMode,
    PreferencesOpen,
}

impl AppState {
    /// Check if the app is in an active interactive state
    #[inline]
    pub fn is_interactive(&self) -> bool {
        matches!(
            self,
            Self::LauncherActive | Self::SearchMode | Self::PreferencesOpen
        )
    }



    /// Get human-readable state description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Background => "Background",
            Self::LauncherActive => "Launcher Active",
            Self::SearchMode => "Search Mode",
            Self::PreferencesOpen => "Preferences Open",
        }
    }
}

/// Bevy LogPlugin custom layer function for file output
/// This function provides both console and file logging using XDG-compliant directories
/// Creates "action-items.log" as the current log file for consistent tailing
fn file_logging_layer(_app: &mut App) -> Option<BoxedLayer> {
    use tracing_subscriber::prelude::*;
    use tracing_appender::non_blocking::WorkerGuard;
    
    // Create XDG-compliant log directory directly (same logic as AppDirectories)
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| std::env::temp_dir().join("action-items-data"))
        .join("action-items")
        .join("logs");
    
    // Ensure log directory exists
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("ERROR: Failed to create log directory {:?}: {}", log_dir, e);
        return None;
    }
    
    // Create file appender with consistent filename "action-items.log"
    // Use "never" rotation to maintain consistent filename for tailing
    let file_appender = tracing_appender::rolling::never(&log_dir, "action-items.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    // Store guard to prevent early drop using static storage
    static GUARD_STORAGE: std::sync::OnceLock<WorkerGuard> = std::sync::OnceLock::new();
    let _ = GUARD_STORAGE.set(_guard);
    
    // Create structured file layer with comprehensive metadata
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)           // No ANSI codes in files
        .with_line_number(true)     // Include source line numbers  
        .with_file(true)            // Include source file names
        .with_target(true)          // Include target module names
        .with_thread_ids(true)      // Include thread IDs for async debugging
        .with_thread_names(true)    // Include thread names
        .with_level(true)           // Include log levels
        .compact();                 // Use compact formatting
    
    Some(file_layer.boxed())
}

/// Configure the main Bevy app with all plugins, resources, and events
pub fn configure_app() -> App {
    // Use custom WinitPlugin for proper global hotkey event integration
    let winit_plugin = WinitPlugin::<GlobalHotkeyEvent>::default();

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<WinitPlugin<WakeUp>>()
            .set(LogPlugin {
                level: Level::DEBUG,
                filter: "wgpu=error,naga=warn,bevy_render=info,bevy_ecs=info".into(),
                custom_layer: file_logging_layer,
            })
            .add(winit_plugin)
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Action Items".into(),
                    decorations: false,
                    window_level: WindowLevel::AlwaysOnTop,
                    visible: false,
                    mode: WindowMode::Windowed,
                    transparent: false, // Opaque for performance (LAUNCHER_UI spec compliant)
                    resizable: false,   // Fixed size (LAUNCHER_UI spec compliant)
                    resolution: (600.0, 420.0).into(), /* Fixed Raycast dimensions (LAUNCHER_UI
                                         * spec compliant) */
                    #[cfg(target_os = "macos")]
                    composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                    ..default()
                }),
                ..default()
            }),
    )
    // Solid background color for visibility - no transparency issues
    .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.08)))
    .insert_resource(UiScale(1.0))
    // Core service coordination - Service Bridge must be first
    .add_plugins(ServiceBridgePlugin)
    // Core application plugins
    .add_plugins((
        ActionItemsCorePlugin,    // Simplified ECS service coordinator
        LauncherUiPlugin,         // UI components
        MonitorConstraintsPlugin, // Monitor handling
        OverlayWindowPlugin,      // Window management
        WindowActivationPlugin,   // Window activation
    ))
    // ECS Service ecosystem - Data and persistence
    .add_plugins((
        DatabasePlugin::new(action_items_ecs_surrealdb::DatabaseConfig::default()), // SurrealDB service for persistence ✅
        UserSettingsPlugin,      // User settings with database backend ✅
        EcsCachePlugin,          // Cache service for performance ✅
        SearchPlugin,            // Core search logic
        SearchUIPlugin::default(), // Search UI components
        SearchAggregatorPlugin, // Search coordination across plugins ✅
    ))
    // ECS Service ecosystem - Core services
    .add_plugins((
        BluetoothPlugin,              // Cross-platform Bluetooth operations ✅
        ClipboardPlugin,              // ECS clipboard service ✅
        CompressionPlugin::default(), // Compression service for data optimization ✅
        PermissionPlugin,             // ECS permissions service ✅
        PermissionWizardPlugin::default()
            .with_required_permissions(vec![
                PermissionType::Accessibility,
                PermissionType::FullDiskAccess,
                PermissionType::Camera,
                PermissionType::Microphone,
            ])
            .with_reason("Action Items requires these permissions to function properly. Accessibility enables global hotkeys, Full Disk Access allows file management, and Camera/Microphone support media features."), // Permission setup wizard with first-run permissions ✅
        // MacosPermissionsPlugin replaced by ECS PermissionPlugin above
        NotificationSystemPlugin, // Enterprise notification system ✅
        HttpPlugin::default(),    // HTTP client service ✅
        ProgressPlugin::<AppState>::new(), // Progress tracking service ✅
        UiLunexPlugins,          // UI service coordination - ENABLED ✅
        TlsCleanupPlugin, // TLS/certificate management ✅
    ))
    // ECS Service ecosystem - UI and user preferences
    .add_plugins((
        SettingsPlugin,                    // Settings management core
        SettingsUIPlugin::default(),       // Settings UI with tab navigation
        PreferencesPlugin,                 // Preferences core (hotkey management)
        PreferencesUIPlugin,    // Preferences UI with recording
    ))
    // Task and launcher services
    .add_plugins((
        TaskManagementPlugin,                         // ECS task management service ✅
        FileSystemPlugin,                             // ECS filesystem service ✅
        HotkeyPlugin::new().with_debug_logging(true), // Global hotkey service ✅
        EcsLauncherService::new().with_debug_logging(true), // Launcher service ✅
        HotkeyLauncherBridgePlugin,                   // Hotkey integration ✅
    ));
    // Development runtime
    app.add_plugins(DenoPlugin::default());     // JavaScript/TypeScript runtime ✅

    // Insert all required resources
    insert_app_resources(&mut app);

    // Add all events
    add_app_events(&mut app);

    // Add explicit command queue flushing to ensure commands are applied properly
    app.add_systems(
        Startup,
        ApplyDeferred
    );

    app
}

/// Insert all application resources
fn insert_app_resources(app: &mut App) -> &mut App {
    app.insert_resource(LauncherState {
        visible: false,
        window_entity: None,
        current_height: 0.0, // Will be dynamically calculated from viewport
        target_height: 0.0,  // Will be dynamically calculated from viewport
        has_gained_focus: false,
        show_timestamp: None,
    })
    .insert_resource(LauncherHotkeys::default())
    .insert_resource(SearchQuery::default())
    // Note: GlobalHotkeyManager, RegisteredHotkey, and HotkeyPreferences
    // are now provided by the ECS HotkeyPlugin - remove manual insertion
    .insert_resource(PreferencesResource::default())
    .insert_resource(ActiveMonitor::default())
    // Search debouncing now handled by ECS search aggregator
    .init_resource::<PendingFileOperations>()
    .init_state::<AppState>()
    .init_resource::<crate::window::positioning::MonitorCameraRegistry>()
    .init_resource::<crate::window::positioning::ScreenDimensions>()
    .init_resource::<crate::window::state::ViewportState>()
    .init_resource::<ecs_service_bridge::systems::plugin_management::capability_index::PluginCapabilityIndex>();

    // Initialize AppGlobalHotkeyManager with proper error handling for accessibility permissions
    match global_hotkey::GlobalHotKeyManager::new() {
        Ok(manager) => {
            app.insert_resource(ecs_hotkey::AppGlobalHotkeyManager {
                manager,
                toggle_hotkey: global_hotkey::hotkey::HotKey::new(
                    Some(global_hotkey::hotkey::Modifiers::SUPER | global_hotkey::hotkey::Modifiers::SHIFT),
                    global_hotkey::hotkey::Code::Space,
                ),
            });
            tracing::info!("✅ AppGlobalHotkeyManager initialized successfully");
        }
        Err(e) => {
            warn!("Failed to initialize GlobalHotkeyManager: {}", e);
            warn!("Global hotkeys will be disabled. This is usually due to missing accessibility permissions on macOS.");
            
            // Register system to show accessibility permission notification during PostStartup
            app.add_systems(PostStartup, show_accessibility_permission_notification);
        }
    }

    // Install custom panic hook for proper file logging
    // Must be done after LogPlugin initialization to ensure tracing is set up
    install_panic_hook_for_file_logging();

    app
}

/// Install custom panic hook that routes panics through tracing system for file logging
/// Replaces Bevy's broken panic hook that uses eprintln() and bypasses file logs
fn install_panic_hook_for_file_logging() {
    use std::panic;
    use std::backtrace::Backtrace;
    
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Extract panic location and message
        let location = panic_info.location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());
            
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "panic occurred".to_string()
        };
        
        // Capture backtrace for debugging
        let backtrace = Backtrace::capture();
        
        // Log panic through tracing system - this will reach file logs
        tracing::error!(
            target: "panic",
            %location,
            %message,
            %backtrace,
            "PANIC OCCURRED: {} at {}",
            message,
            location
        );
        
        // Also log to stderr for immediate visibility (non-blocking)
        eprintln!("PANIC: {} at {}", message, location);
        
        // Call the original panic handler for standard behavior
        old_hook(panic_info);
    }));
}

/// Add all application events
fn add_app_events(app: &mut App) {
    app.add_event::<LauncherEvent>()
        .add_event::<GlobalHotkeyEvent>()
        .add_event::<PreferencesEvent>()
        .add_event::<ecs_filesystem::FileSystemRequest>()
        .add_event::<ecs_filesystem::FileSystemResponse>()
        .add_event::<UiVisibilityEvent>()
        .add_event::<TextInputChanged>()
        .add_event::<bevy::window::WindowResized>()
        .add_event::<action_items_ui::SearchQueryChanged>();
}

/// System to show accessibility permission notification using ecs-notifications
/// Runs during PostStartup to avoid CommandQueue issues during app initialization
fn show_accessibility_permission_notification(mut commands: Commands) {
    let exe_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "/Volumes/samsung_t9/action-items/target/debug/action_items".to_string());

    let notification = NotificationBuilder::new()
        .with_title("Accessibility Permission Required")
        .with_body(ecs_notifications::RichText::plain(format!(
            "Action Items needs accessibility permissions to enable global hotkeys (Cmd+Shift+Space).\n\n\
            To grant permission:\n\
            1. Open System Preferences > Security & Privacy > Privacy > Accessibility\n\
            2. Click the lock icon and enter your password\n\
            3. Click '+' and navigate to:\n   {}\n\
            4. Check the box next to 'action_items'\n\
            5. Restart Action Items\n\n\
            The app will work without this permission, but global hotkeys will be disabled.", exe_path
        )))
        .with_priority(ecs_notifications::Priority::High)
        .with_platforms(vec![Platform::MacOS])
        .build();

    commands.spawn(notification);
}

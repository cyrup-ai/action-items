use bevy::prelude::*;

use super::events::bridge_preferences_to_settings;
use super::hotkey_setup::setup_global_hotkey_callback;
use super::window_config::configure_non_activating_panel;
use super::window_resize::handle_window_resized_system;
use crate::events::handlers::preferences::{
    PendingFileOperations, handle_preferences_filesystem_responses,
};
// Removed custom storage system imports - now using ecs-filesystem service
use crate::events::{
    detect_preferences_command, handle_execute_commands, handle_global_hotkeys,
    handle_launcher_events, handle_preferences_events, handle_preferences_ui_interactions,
    real_hotkey_capture_system, update_current_query_from_events,
};
use crate::hotkeys::{
    handle_hotkey_registration_system, handle_launcher_hotkey_press_system,
    register_launcher_hotkey_system,
};
use crate::input::{
    // Focus system imports
    InputFocus,
    animate_focus_transitions_system,
    apply_hover_styling_system,
    apply_ime_composition_styling_system,
    // Styling system imports
    apply_text_input_focus_styling_system,
    apply_text_selection_styling_system,
    context_aware_input_system,
    handle_escape_focus_system,
    handle_focus_loss_system,
    handle_ime_cursor_system,
    // IME system imports
    handle_ime_input_system,
    handle_ime_state_system,
    handle_ime_string_interning_system,
    handle_interactive_text_input,
    handle_keyboard_focus_navigation_system,
    handle_mouse_focus_system,
    handle_text_input_focus_system,
    // String management imports
    initialize_string_interner_system,
    intern_search_strings_system,
    manage_string_interner_memory_system,
    optimize_frequent_strings_system,
    report_string_interner_stats_system,
    set_initial_text_focus_system,
    trigger_search_from_text_input,
    unified_keyboard_input_system,
    update_cursor_styling_system,
    update_text_styling_system,
};
// Preferences UI now handled by ecs-preferences service
use action_items_ecs_permissions::{PermissionChanged, PermissionStatus, PermissionType};
use action_items_core::search::setup_search_index;
use crate::search::{handle_text_input_to_search, log_search_events, update_search_ui};
// UI setup now handled by ECS UI service
// UI systems will be integrated when needed - removing unused imports
use crate::window::{
    WindowModeManager,
    activation::{ActivationReason, WindowActivationEvent},
    adjust_window_size_for_results_system, animate_window_system, debug_monitor_info_system,
    detect_monitors_system, handle_search_input_system, handle_window_blur_system,
    overlay_configuration_system, position_ui_on_correct_monitor, setup_ui_target_camera,
    setup_window_system, sync_ui_visibility_system, track_active_application_monitor_system,
    update_ui_monitor_positioning, window_management_system, window_mode_management_system,
};

/// Send application startup activation event
fn send_application_start_activation(mut activation_events: EventWriter<WindowActivationEvent>) {
    activation_events.write(WindowActivationEvent {
        reason: ActivationReason::ApplicationStart,
    });
}

/// Handle accessibility permission changes and retry hotkey setup
fn handle_accessibility_permission_changes(
    mut permission_events: EventReader<PermissionChanged>,
) {
    for event in permission_events.read() {
        if event.typ == PermissionType::Accessibility {
            match event.status {
                PermissionStatus::Authorized => {
                    info!("✅ Accessibility permission granted! Global hotkeys should now work.");
                    info!("Please restart the application to enable hotkey functionality.");
                },
                PermissionStatus::Denied => {
                    error!("❌ Accessibility permission denied - global hotkeys will NOT work");
                    error!("Please enable accessibility permissions in System Preferences → Privacy & Security → Accessibility");
                },
                PermissionStatus::Restricted => {
                    error!("🚫 Accessibility permission restricted - global hotkeys will NOT work");
                },
                _ => {
                    warn!("Accessibility permission status changed to: {:?}", event.status);
                }
            }
        }
    }
}

/// Register all startup systems
pub fn add_startup_systems(app: &mut App) {
    // Initialize focus system resource
    app.init_resource::<InputFocus>();
    // Initialize window mode manager resource
    app.init_resource::<WindowModeManager>();
    // Initialize filesystem operations tracking resource
    app.init_resource::<PendingFileOperations>();

    // ECS Storage Events now handled by ecs-filesystem service

    app.add_systems(
        Startup,
        (
            setup_window_system,
            register_launcher_hotkey_system,
            // Initialize string interner with common strings
            initialize_string_interner_system,
            // Send activation event when app starts
            send_application_start_activation,
            // Initialize SearchIndex for local application/file/command search
            setup_search_index,
            #[cfg(target_os = "macos")]
            configure_non_activating_panel,
        ),
    );


}

/// Register all post-startup systems
pub fn add_post_startup_systems(app: &mut App) {
    app.add_systems(
        PostStartup,
        (
            // UI setup now handled by ECS UI service
            setup_ui_target_camera,
            // Set initial focus after UI is set up
            set_initial_text_focus_system, // Focus system still needed
            // Move hotkey setup to PostStartup to avoid CommandQueue issues
            setup_global_hotkey_callback,
        ),
    );
}

/// Register all update systems with proper scheduling
pub fn add_update_systems(app: &mut App) {
    // Multi-monitor detection and positioning
    app.add_systems(
        Update,
        (
            detect_monitors_system,
            track_active_application_monitor_system,
            debug_monitor_info_system,
            position_ui_on_correct_monitor,
            update_ui_monitor_positioning,
            #[cfg(debug_assertions)]
            crate::window::systems::debug_viewport_calculations_system,
        ),
    );

    // Event-driven input and search systems
    app.add_systems(
        Update,
        (
            // ECS hotkey service integration
            handle_hotkey_registration_system,
            handle_launcher_hotkey_press_system,
            // Event-driven global input (no polling!) - runs first
            handle_global_hotkeys,
            // Focus management systems - run before input processing
            (
                handle_keyboard_focus_navigation_system,
                handle_mouse_focus_system,
                handle_focus_loss_system,
                handle_escape_focus_system,
                handle_text_input_focus_system,
            ),
            // IME support systems - run with input processing
            (
                handle_ime_input_system,
                handle_ime_cursor_system,
                handle_ime_state_system,
            ),
            // Interactive text input chain - MUST run in order
            (
                handle_interactive_text_input,
                trigger_search_from_text_input,
                // Text display now handled by ECS UI service
            )
                .chain()
                .after(handle_ime_input_system),
            // Search bar keyboard input handling - converts KeyboardInput to SearchQueryChanged
            (
                action_items_ui::systems::search_input_system,
                action_items_ui::systems::results_visibility_system,
            )
                .chain(),
            // Simplified search integration - ECS search aggregator handles coordination
            (
                handle_text_input_to_search, // Direct text input to CurrentQuery
                update_search_ui,            // UI updates based on search results
                log_search_events,           // Event logging
            )
                .chain(),
            // Input processing - runs after search chain
            (
                context_aware_input_system,
                unified_keyboard_input_system,
                handle_launcher_events,
            )
                .chain(),
            // UI interaction systems - can run in parallel
            (
                action_items_ui::prelude::handle_result_item_hover_system,
                action_items_ui::prelude::handle_keyboard_selection_highlighting_system,
            ),
            // Focus-aware styling systems - run after focus changes
            (
                apply_text_input_focus_styling_system,
                update_text_styling_system,
                apply_text_selection_styling_system,
                animate_focus_transitions_system,
                apply_ime_composition_styling_system,
                apply_hover_styling_system,
                update_cursor_styling_system,
            )
                .after(handle_text_input_focus_system),
            // String management systems - run periodically for optimization
            (
                intern_search_strings_system,
                manage_string_interner_memory_system,
                optimize_frequent_strings_system,
                handle_ime_string_interning_system,
                report_string_interner_stats_system,
            ),
            // Responsive container system - runs independently
            action_items_ui::prelude::adaptive_container_system,
        ),
    );

    // Window management systems
    app.add_systems(
        Update,
        (
            // Window management chain - MUST run in order to prevent flicker
            (
                window_management_system,  // Updates window state
                animate_window_system,     // Reads window state, applies animations
                sync_ui_visibility_system, // Syncs UI with window visibility
            )
                .chain(),
            // Window mode management - handles fullscreen, capability testing
            window_mode_management_system,
            // Overlay configuration - runs when window activation events occur
            overlay_configuration_system,
            // Window interaction processing - depends on window state
            (
                handle_window_blur_system,
                handle_search_input_system,
                handle_execute_commands,
                update_current_query_from_events,
            )
                .chain()
                .after(sync_ui_visibility_system),
            // Window sizing - runs after all other window operations
            (
                adjust_window_size_for_results_system,
                handle_window_resized_system,
            )
                .chain()
                .after(handle_execute_commands),
        ),
    );

    // Hotkey preferences systems
    app.add_systems(
        Update,
        (
            handle_preferences_events,
            handle_preferences_filesystem_responses, // Handle filesystem operation responses
            detect_preferences_command,
            real_hotkey_capture_system,      // Real hotkey capture system
            // Preferences UI now handled by ecs-preferences service
            handle_preferences_ui_interactions,
            bridge_preferences_to_settings, /* Bridge system for settings UI
                                             * Storage operations now handled by ecs-filesystem
                                             * service */
            handle_accessibility_permission_changes, // Handle accessibility permission changes
        ),
    );

    // ECS Storage systems now handled by ecs-filesystem service plugin

    // Advanced UI systems - gradient, responsive, search, and setup systems
    // UI systems will be added when they are implemented and needed
    // Removing unused system registrations to fix compilation errors
}

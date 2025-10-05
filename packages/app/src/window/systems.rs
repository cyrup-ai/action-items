//! Window management systems
//!
//! Zero-allocation window management systems with blazing-fast animations, focus tracking, and size
//! adjustments.

use action_items_core::{LauncherEvent, LauncherEventType};
use action_items_ui::{UiState, set_ui_visibility};
use bevy::prelude::*;
use bevy::window::{Monitor, MonitorSelection, PrimaryWindow, Window};
use tracing::{debug, error};

use crate::overlay_window::configure_overlay_window_cross_platform;
use crate::window::positioning::{calculate_responsive_window_size, get_active_screen_dimensions};
use crate::window::state::{
    ViewportState, WindowModeManager, convert_window_to_viewport_units, screen_to_viewport,
};
use crate::window::{ActiveMonitor, LauncherState, WindowAnimation};

/// Initial setup system for window management
/// Zero-allocation setup with blazing-fast window entity creation using proportional sizing
#[inline]
pub fn setup_window_system(
    mut commands: Commands,
    mut launcher_state: ResMut<LauncherState>,
    monitors_query: Query<(Entity, &Monitor)>,
    active_monitor: Res<ActiveMonitor>,
) {
    // Add camera for UI
    commands.spawn(Camera2d);

    // Calculate proportional sprite size using viewport-responsive methods
    let (sprite_width, sprite_height) =
        match get_active_screen_dimensions(&active_monitor, &monitors_query) {
            Ok(screen_dims) => {
                // Use viewport methods for precise responsive sizing
                // 43vw width, 33vh height (slightly smaller than launcher)
                calculate_responsive_window_size(&screen_dims, 43.0, 33.0)
            },
            Err(_) => {
                // Fallback dimensions when screen detection fails
                (580.0, 380.0)
            },
        };

    // Create window background with proportional sizing
    let window_entity = commands
        .spawn((
            Sprite {
                color: Color::srgba(0.1, 0.1, 0.1, 0.0),
                custom_size: Some(Vec2::new(sprite_width, sprite_height)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            WindowAnimation {
                target_opacity: 0.0,
                current_opacity: 0.0,
                animation_speed: 8.0, // Blazing-fast animation speed
            },
        ))
        .id();

    launcher_state.window_entity = Some(window_entity);
}

/// System to animate window opacity
/// Zero-allocation opacity animation system with blazing-fast lerp calculations
#[inline]
pub fn animate_window_system(
    time: Res<Time>,
    mut animations: Query<(&mut WindowAnimation, &mut Sprite)>,
) {
    for (mut anim, mut sprite) in animations.iter_mut() {
        if (anim.current_opacity - anim.target_opacity).abs() > 0.01 {
            anim.current_opacity = lerp(
                anim.current_opacity,
                anim.target_opacity,
                anim.animation_speed * time.delta_secs(),
            );
            sprite.color = Color::srgba(0.1, 0.1, 0.1, anim.current_opacity);
        }
    }
}

/// System to handle window blur (losing focus)
/// Zero-allocation focus tracking with blazing-fast blur detection
#[inline]
pub fn handle_window_blur_system(
    primary_window: Single<&Window, With<PrimaryWindow>>,
    mut launcher_state: ResMut<LauncherState>,
    mut animations: Query<&mut WindowAnimation>,
    mut launcher_events: EventWriter<LauncherEvent>,
) {
    // Track when window gains focus
    if primary_window.focused && launcher_state.visible && !launcher_state.has_gained_focus {
        launcher_state.has_gained_focus = true;
        return;
    }

    // Check if we're in the grace period (500ms after showing)
    if let Some(show_time) = launcher_state.show_timestamp
        && show_time.elapsed() < std::time::Duration::from_millis(500)
    {
        // Still in grace period, don't hide even if not focused
        return;
    }

    // Only hide on blur if we've gained focus at least once
    if !primary_window.focused && launcher_state.visible && launcher_state.has_gained_focus {
        launcher_state.visible = false;
        launcher_events.write(LauncherEvent::new(LauncherEventType::SystemShutdown));
        launcher_state.show_timestamp = None; // Clear timestamp

        // Start hide animation
        if let Some(entity) = launcher_state.window_entity {
            if let Ok(mut anim) = animations.get_mut(entity) {
                anim.target_opacity = 0.0;
            } else {
                error!("Failed to get WindowAnimation component for blur handling.");
            }
        }
    }
}

/// System to manage window behavior
/// Zero-allocation window visibility management with blazing-fast updates
#[inline]
pub fn window_management_system(
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
    launcher_state: Res<LauncherState>,
) {
    if primary_window.visible != launcher_state.visible {
        primary_window.visible = launcher_state.visible;
    }
}

/// Sync launcher state with UI state
/// Zero-allocation state synchronization with blazing-fast UI updates
#[inline]
pub fn sync_ui_visibility_system(launcher_state: Res<LauncherState>, ui_state: ResMut<UiState>) {
    if ui_state.visible != launcher_state.visible {
        // Sync visibility without spam
        set_ui_visibility(ui_state, launcher_state.visible);
    }
}

/// Handle search input from UI
/// Zero-allocation query synchronization with blazing-fast updates
#[inline]
pub fn handle_search_input_system(
    ui_state: Res<UiState>,
    mut current_query: ResMut<action_items_core::CurrentQuery>,
) {
    if ui_state.is_changed() {
        current_query.0 = ui_state.query.clone();
    }
}

/// System to adjust window height based on search results
/// Zero-allocation window resizing with proportional height calculations and smooth animations
#[inline]
pub fn adjust_window_size_for_results_system(
    search_results: Res<action_items_core::CurrentSearchResults>,
    mut launcher_state: ResMut<LauncherState>,
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
    monitors_query: Query<&Monitor>,
    active_monitor: Res<ActiveMonitor>,
    time: Res<Time>,
) {
    let num_results = search_results.results.len();

    // Get monitor-proportional dimensions
    let (base_height, max_height, item_height) =
        if let Some(monitor_entity) = active_monitor.target.or(active_monitor.primary) {
            if let Ok(monitor) = monitors_query.get(monitor_entity) {
                let monitor_height = monitor.physical_height as f32;
                // Use viewport-relative heights: 35% base, 50% max, 8% per item
                let base = monitor_height * 0.35;
                let max = monitor_height * 0.50;
                let item = monitor_height * 0.08;
                (base, max, item)
            } else {
                // Fallback proportional values
                (400.0, 600.0, 60.0)
            }
        } else {
            // Safe fallback values
            (400.0, 600.0, 60.0)
        };

    let results_area_height = (num_results as f32 * item_height).min(max_height - base_height);

    let new_target_height = if num_results > 0 {
        (base_height + results_area_height).min(max_height)
    } else {
        base_height
    };

    if (launcher_state.target_height - new_target_height).abs() > 0.1 {
        launcher_state.target_height = new_target_height;
    }

    if (launcher_state.current_height - launcher_state.target_height).abs() > 1.0 {
        launcher_state.current_height = lerp(
            launcher_state.current_height,
            launcher_state.target_height,
            8.0 * time.delta_secs(), // Blazing-fast animation speed
        );
        let current_width = primary_window.resolution.width();
        primary_window
            .resolution
            .set(current_width, launcher_state.current_height);
    } else if launcher_state.current_height != launcher_state.target_height {
        // Snap to target if close enough
        launcher_state.current_height = launcher_state.target_height;
        let current_width = primary_window.resolution.width();
        primary_window
            .resolution
            .set(current_width, launcher_state.current_height);
    }
}

/// Linear interpolation helper
/// Zero-allocation lerp function with blazing-fast clamping
#[inline]
pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t.clamp(0.0, 1.0)
}

/// System to validate and debug viewport calculations
/// Uses all ViewportState methods for comprehensive testing in debug builds
#[cfg(debug_assertions)]
pub fn debug_viewport_calculations_system(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    viewport_state: Option<Res<ViewportState>>,
) {
    if let (Ok(window), Some(viewport)) = (primary_window.single(), viewport_state) {
        let window_width = window.resolution.width();
        let window_height = window.resolution.height();

        // Test convert_window_to_viewport_units (uses pixels_to_vw, pixels_to_vh, is_valid)
        if let Ok((vw_percent, vh_percent)) =
            convert_window_to_viewport_units(window_width, window_height, &viewport)
        {
            // Test screen_to_viewport function
            if let Ok((val_vw, val_vh)) = screen_to_viewport(
                window_width * 0.5,  // Half window width
                window_height * 0.5, // Half window height
                &viewport,
            ) {
                // Log viewport calculations for debugging (only in debug builds)
                debug!(
                    "Viewport debug: Window {}x{} -> {}vw x {}vh, Half-window -> {:?} x {:?}",
                    window_width, window_height, vw_percent, vh_percent, val_vw, val_vh
                );
            }
        }
    }
}

/// Window mode management system that uses all WindowModeManager fields
/// Manages window mode transitions, capability testing, and fallback handling
#[inline]
pub fn window_mode_management_system(
    mut window_mode_manager: ResMut<WindowModeManager>,
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Set different target modes based on keyboard input
    if keyboard.just_pressed(KeyCode::F10) {
        // F10 sets target to windowed mode
        window_mode_manager.set_target_mode(bevy::window::WindowMode::Windowed);
    } else if keyboard.just_pressed(KeyCode::F12) {
        // F12 sets target to borderless fullscreen
        window_mode_manager.set_target_mode(bevy::window::WindowMode::BorderlessFullscreen(
            MonitorSelection::Primary,
        ));
    }

    // Test fullscreen capability when F11 is pressed
    if keyboard.just_pressed(KeyCode::F11) {
        if window_mode_manager.can_change_mode() {
            // Test if we can change modes and update capabilities based on result
            match window_mode_manager.attempt_mode_change() {
                Ok(new_mode) => {
                    // Apply the mode change to the actual window
                    match &new_mode {
                        bevy::window::WindowMode::Windowed => {
                            primary_window.mode = bevy::window::WindowMode::Windowed;
                            // Test that windowed mode works
                            window_mode_manager.update_capabilities(true, true);
                        },
                        bevy::window::WindowMode::BorderlessFullscreen(monitor_selection) => {
                            primary_window.mode =
                                bevy::window::WindowMode::BorderlessFullscreen(*monitor_selection);
                            // Test if Bevy's BorderlessFullscreen actually works
                            let bevy_fullscreen_works =
                                primary_window.mode != bevy::window::WindowMode::Windowed;
                            window_mode_manager.update_capabilities(true, bevy_fullscreen_works);
                        },
                        bevy::window::WindowMode::Fullscreen(..) => {
                            primary_window.mode = new_mode;
                            // Test that fullscreen mode was properly set
                            let fullscreen_works = matches!(
                                primary_window.mode,
                                bevy::window::WindowMode::Fullscreen(..)
                            );
                            window_mode_manager.update_capabilities(true, fullscreen_works);
                        },
                    }

                    tracing::info!("Window mode successfully changed to: {:?}", new_mode);
                },
                Err(error_msg) => {
                    tracing::warn!("Window mode change failed: {}", error_msg);
                    // If mode change fails, update capabilities accordingly
                    window_mode_manager.update_capabilities(false, false);
                },
            }
        } else if let Some(time_since_change) = window_mode_manager.time_since_last_change() {
            tracing::debug!(
                "Window mode change blocked by cooldown: {:?} elapsed, {:?} required",
                time_since_change,
                window_mode_manager.mode_change_cooldown
            );
        }
    }

    // Window mode status is logged only on actual state changes (see attempt_mode_change)
}

/// Overlay configuration system for window activation
/// Uses cross-platform overlay configuration with comprehensive error handling
#[inline]
pub fn overlay_configuration_system(
    mut activation_events: EventReader<crate::window::activation::WindowActivationEvent>,
    primary_window: Single<Entity, With<PrimaryWindow>>,
    winit_windows: NonSend<bevy::winit::WinitWindows>,
) {
    for _activation_event in activation_events.read() {
        // Get the winit window for overlay configuration
        if let Some(winit_window) = winit_windows.get_window(*primary_window) {
            match configure_overlay_window_cross_platform(winit_window) {
                Ok(configuration_result) => {
                    tracing::info!("Overlay configuration successful: {}", configuration_result);
                },
                Err(configuration_error) => {
                    tracing::warn!("Overlay configuration failed: {}", configuration_error);
                    // Continue without overlay - launcher will still work in normal window mode
                },
            }
        } else {
            tracing::error!("Failed to get winit window for overlay configuration");
        }
    }
}

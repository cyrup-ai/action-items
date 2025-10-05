use action_items_core::{CurrentQuery, LauncherEvent, LauncherEventType};
use bevy::prelude::*;
use bevy::window::{Monitor, MonitorSelection, PrimaryWindow, Window, WindowPosition};
use tracing::error;

use crate::app_main::AppState;
use crate::window::{LauncherState, WindowAnimation, activate_window};

/// System to handle launcher events with blazing-fast window management
/// Zero allocation window positioning and sizing
#[inline]
pub fn handle_launcher_events(
    mut launcher_events: EventReader<LauncherEvent>,
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
    all_monitors: Query<(Entity, &Monitor)>,
    mut launcher_state: ResMut<LauncherState>,
    _app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut animations: Query<&mut WindowAnimation>,
) {
    for event in launcher_events.read() {
        match &event.event_type {
            LauncherEventType::SearchStarted(_) => {
                // Update AppState to SearchMode when search starts
                next_state.set(AppState::SearchMode);

                launcher_state.visible = true;
                launcher_state.has_gained_focus = false;
                launcher_state.show_timestamp = Some(std::time::Instant::now());

                // Use cross-platform window activation to properly foreground
                activate_window(&mut primary_window);

                // Raycast-like window sizing: 35% width x 28% height, max 800x600
                if let Some((_monitor_entity, monitor_props)) = all_monitors.iter().next() {
                    let screen_width = monitor_props.physical_width as f32;
                    let screen_height = monitor_props.physical_height as f32;

                    // Raycast proportions with maximum size limits
                    let window_width = 800.0_f32.min(screen_width * 0.35); // Max 800px or 35%
                    let window_height = 600.0_f32.min(screen_height * 0.28); // Max 600px or 28%

                    primary_window.resolution.set(window_width, window_height);
                    launcher_state.target_height = window_height * 0.85; // Slightly reduced for better proportions
                    launcher_state.current_height = window_height * 0.85;
                } else {
                    // Fallback for Raycast-like proportions
                    primary_window.resolution.set(600.0, 400.0);
                    launcher_state.target_height = 340.0;
                    launcher_state.current_height = 340.0;
                }

                primary_window.position = WindowPosition::Centered(MonitorSelection::Primary);

                // Update animation with zero allocation
                if let Some(entity) = launcher_state.window_entity
                    && let Ok(mut anim) = animations.get_mut(entity)
                {
                    anim.target_opacity = 0.95;
                }
            },
            LauncherEventType::SystemShutdown => {
                // Update AppState to Background when shutting down
                next_state.set(AppState::Background);

                launcher_state.visible = false;
                launcher_state.show_timestamp = None;

                // Background the application - hide window but keep app running
                primary_window.visible = false;

                if let Some(entity) = launcher_state.window_entity
                    && let Ok(mut anim) = animations.get_mut(entity)
                {
                    anim.target_opacity = 0.0;
                }
            },
            _ => {},
        }
    }
}

/// System to handle execute commands with blazing-fast window hiding
/// Zero allocation command execution handling
#[inline]
pub fn handle_execute_commands(
    mut launcher_events: EventReader<LauncherEvent>,
    mut launcher_state: ResMut<LauncherState>,
    _app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
    mut animations: Query<&mut WindowAnimation>,
) {
    for event in launcher_events.read() {
        if let LauncherEventType::Execute(_) = &event.event_type {
            // Update AppState to Background when executing commands
            next_state.set(AppState::Background);

            // Blazing-fast window hiding after execution
            launcher_state.visible = false;
            launcher_state.show_timestamp = None;

            primary_window.visible = false;

            if let Some(entity) = launcher_state.window_entity {
                if let Ok(mut anim) = animations.get_mut(entity) {
                    anim.target_opacity = 0.0;
                } else {
                    error!("Failed to get WindowAnimation component for execute command handling.");
                }
            }
        }
    }
}

/// System to update current query from events - zero allocation query processing
#[inline]
pub fn update_current_query_from_events(
    mut launcher_events: EventReader<LauncherEvent>,
    mut current_query: ResMut<CurrentQuery>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for event in launcher_events.read() {
        if let LauncherEventType::SearchStarted(query) = &event.event_type {
            current_query.0 = query.clone();

            // Transition to SearchMode when user has typed a query
            if !query.trim().is_empty() {
                next_state.set(AppState::SearchMode);
            } else if matches!(app_state.get(), AppState::SearchMode) {
                // Return to LauncherActive if query becomes empty
                next_state.set(AppState::LauncherActive);
            }
        }
    }
}

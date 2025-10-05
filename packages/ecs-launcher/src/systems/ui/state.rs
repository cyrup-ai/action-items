//! UI state management systems with blazing-fast state transitions

use bevy::prelude::*;
use tracing::info;

use crate::components::*;
use crate::events::*;
use crate::resources::*;

/// Process UI state changes with blazing-fast state transitions
#[inline(always)]
pub fn process_ui_state_changes_system(
    mut ui_events: EventReader<UIStateChanged>,
    mut launcher_state: ResMut<LauncherState>,
    config: Res<LauncherConfig>,
) {
    for event in ui_events.read() {
        if config.enable_debug_logging {
            info!(
                "UI state changing from {:?} to {:?}",
                event.previous_state, event.new_state
            );
        }

        launcher_state.current_ui_state = event.new_state.clone();
    }
}

/// Manage launcher window visibility and state with zero-allocation updates
#[inline(always)]
pub fn manage_launcher_window_system(
    mut window_events: EventReader<LauncherWindowToggled>,
    mut launcher_state: ResMut<LauncherState>,
    mut window_state: Option<ResMut<WindowState>>,
    mut launcher_windows: Query<&mut LauncherWindow>,
    config: Res<LauncherConfig>,
) {
    for event in window_events.read() {
        if config.enable_debug_logging {
            info!("Toggling launcher window: visible={}", event.visible);
        }

        launcher_state.is_window_visible = event.visible;

        if event.visible
            && let Some(ref mut state) = window_state {
                state.last_shown = Some(std::time::Instant::now());
                state.show_count += 1;
            }

        // Update window components
        for mut launcher_window in launcher_windows.iter_mut() {
            launcher_window.is_visible = event.visible;
            if event.visible {
                launcher_window.last_shown = Some(std::time::Instant::now());
                launcher_window.show_count += 1;
            }
        }
    }
}

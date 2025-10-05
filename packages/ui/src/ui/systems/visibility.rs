//! App-specific UI visibility management
//!
//! Handles business logic for visibility animations (UiState updates).
//! Generic animation logic is provided by action_items_ecs_ui::visibility::VisibilityPlugin.

use bevy::prelude::*;

use action_items_ecs_ui::visibility::{UiAnimationCompleteEvent, UiComponentTarget};

use crate::ui::components::UiState;

/// Handle animation completion events to update app-specific state
///
/// This is an APP-SPECIFIC system that updates UiState when animations complete.
/// The generic animation logic is handled by ecs-ui visibility systems.
#[inline]
pub fn handle_visibility_animation_complete(
    mut completion_events: EventReader<UiAnimationCompleteEvent>,
    mut ui_state: ResMut<UiState>,
) {
    for event in completion_events.read() {
        // Handle main launcher visibility changes
        if event.target == UiComponentTarget::PrimaryContainer {
            if event.was_show {
                // Launcher is showing - state is already set to visible by the event sender
                ui_state.visible = true;
            } else {
                // Launcher finished hiding - clear search state
                ui_state.visible = false;
                ui_state.query.clear();
                ui_state.results.clear();
                ui_state.selected_index = 0;
            }

            tracing::debug!(
                "Launcher visibility animation completed: visible={}",
                event.was_show
            );
        }

        // Other component targets (Dialog, Panel, SecondaryPanel) can be handled here
        // if they need app-specific state management
    }
}

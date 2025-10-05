//! UI interaction systems
//!
//! Zero-allocation interaction handling with blazing-fast keyboard input and click processing.

use action_items_core::{LauncherEvent, LauncherEventType};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::ui::components::{ActionResultItem, UiState};

// Type aliases for complex query types
type InteractiveResultQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static ActionResultItem),
    (Changed<Interaction>, With<Node>),
>;

/// Handle keyboard input for UI navigation
/// Zero-allocation keyboard processing with blazing-fast event handling
#[inline]
pub fn handle_keyboard_input_system(
    mut keyboard_input: EventReader<KeyboardInput>,
    mut ui_state: ResMut<UiState>,
    mut launcher_events: EventWriter<LauncherEvent>,
) {
    for event in keyboard_input.read() {
        // Only process key press events, ignore releases
        if event.state != ButtonState::Pressed {
            continue;
        }

        // Only process input when UI is visible and has results
        if !ui_state.visible || ui_state.results.is_empty() {
            continue;
        }

        match event.logical_key {
            Key::ArrowUp => {
                // Navigate up in results list
                if ui_state.selected_index > 0 {
                    ui_state.selected_index -= 1;
                }
            },
            Key::ArrowDown => {
                // Navigate down in results list
                if ui_state.selected_index < ui_state.results.len() - 1 {
                    ui_state.selected_index += 1;
                }
            },
            Key::Enter => {
                // Execute selected action
                if ui_state.selected_index < ui_state.results.len() {
                    let selected_action = &ui_state.results[ui_state.selected_index];
                    let action_id = selected_action.id.clone();
                    launcher_events
                        .write(LauncherEvent::new(LauncherEventType::Execute(action_id)));
                }
            },
            Key::Escape => {
                // Hide UI and clear state
                ui_state.visible = false;
                ui_state.query.clear();
                ui_state.results.clear();
                ui_state.selected_index = 0;
            },
            _ => {
                // Ignore other keys in UI layer (handled by app layer)
            },
        }
    }
}

/// Handle result item click interactions
/// Zero-allocation click handling with blazing-fast result selection and execution
#[inline]
pub fn handle_result_item_click_system(
    interaction_query: InteractiveResultQuery,
    mut launcher_events: EventWriter<LauncherEvent>,
    mut ui_state: ResMut<UiState>,
) {
    for (interaction, result_item) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Update selection and trigger execution
            ui_state.selected_index = result_item.index;
            // Execute the clicked action
            if ui_state.selected_index < ui_state.results.len() {
                let selected_action = &ui_state.results[ui_state.selected_index];
                let action_id = selected_action.id.clone();
                launcher_events.write(LauncherEvent::new(LauncherEventType::Execute(action_id)));
            }
        }
    }
}

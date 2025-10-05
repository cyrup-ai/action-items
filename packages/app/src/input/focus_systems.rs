use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use tracing::{debug, info};

use super::InteractiveTextInput;
use super::focus::{Focusable, InputFocus};

/// System to handle text input focus changes with optimal performance
/// Updates focus state for all text inputs when focus changes
#[inline]
pub fn handle_text_input_focus_system(
    focus: Res<InputFocus>,
    mut text_input_query: Query<(Entity, &mut InteractiveTextInput)>,
) {
    if focus.is_changed() {
        // Update focus state for all text inputs
        for (entity, mut input) in text_input_query.iter_mut() {
            let was_focused = input.is_focused;
            input.is_focused = focus.is_focused(entity);
            input.focus_visible = input.is_focused && focus.focus_visible;

            // Log focus changes for debugging
            if !was_focused && input.is_focused {
                debug!("Text input gained focus: {:?}", entity);
            } else if was_focused && !input.is_focused {
                debug!("Text input lost focus: {:?}", entity);
                // Clear selection when losing focus
                input.clear_selection();
            }
        }
    }
}

/// System to set initial focus to the search input on startup
/// Ensures the launcher is immediately ready for user input
#[inline]
pub fn set_initial_text_focus_system(
    mut focus: ResMut<InputFocus>,
    search_input_query: Query<Entity, (With<InteractiveTextInput>, With<Focusable>)>,
    mut text_input_query: Query<&mut InteractiveTextInput>,
) {
    if focus.focused_entity.is_none() {
        // Find the first focusable text input (typically the search input)
        if let Some(search_entity) = search_input_query.iter().next() {
            focus.set_focus(search_entity);
            focus.show_focus_indicator();

            // Update the component state immediately
            if let Ok(mut input) = text_input_query.get_mut(search_entity) {
                input.is_focused = true;
                input.focus_visible = true;
            }

            info!("Set initial focus to search input: {:?}", search_entity);
        }
    }
}

/// System to handle keyboard focus navigation (Tab/Shift+Tab)
/// Provides keyboard accessibility for all focusable elements
#[inline]
pub fn handle_keyboard_focus_navigation_system(
    mut keyboard_events: EventReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<InputFocus>,
    focusable_query: Query<(Entity, &Focusable), With<InteractiveTextInput>>,
    mut text_input_query: Query<&mut InteractiveTextInput>,
) {
    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        if event.logical_key == Key::Tab {
            let shift_pressed = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

            // Collect and sort focusable elements by tab index
            let mut focusable_elements: Vec<_> = focusable_query
                .iter()
                .filter(|(_, focusable)| focusable.keyboard_focusable)
                .collect();

            focusable_elements.sort_by_key(|(_, focusable)| focusable.tab_index);

            if focusable_elements.is_empty() {
                continue;
            }

            let current_index = if let Some(focused_entity) = focus.focused_entity {
                focusable_elements
                    .iter()
                    .position(|(entity, _)| *entity == focused_entity)
            } else {
                None
            };

            let next_index = match current_index {
                Some(index) if shift_pressed => {
                    // Shift+Tab: Move to previous element
                    if index == 0 {
                        focusable_elements.len() - 1
                    } else {
                        index - 1
                    }
                },
                Some(index) => {
                    // Tab: Move to next element
                    if index >= focusable_elements.len() - 1 {
                        0
                    } else {
                        index + 1
                    }
                },
                None => {
                    // No current focus: start from first (Tab) or last (Shift+Tab)
                    if shift_pressed {
                        focusable_elements.len() - 1
                    } else {
                        0
                    }
                },
            };

            if let Some((next_entity, _)) = focusable_elements.get(next_index) {
                // Clear focus from current element
                if let Some(current_entity) = focus.focused_entity
                    && let Ok(mut current_input) = text_input_query.get_mut(current_entity)
                {
                    current_input.is_focused = false;
                    current_input.focus_visible = false;
                    current_input.clear_selection();
                }

                // Set focus to next element
                focus.set_focus(*next_entity);
                focus.show_focus_indicator();

                if let Ok(mut next_input) = text_input_query.get_mut(*next_entity) {
                    next_input.is_focused = true;
                    next_input.focus_visible = true;
                }

                debug!("Focus moved to entity: {:?}", next_entity);
            }
        }
    }
}

/// System to handle mouse focus changes
/// Updates focus when user clicks on focusable elements
#[inline]
pub fn handle_mouse_focus_system(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (Entity, &Interaction, &Focusable, &mut InteractiveTextInput),
        Changed<Interaction>,
    >,
) {
    // Handle mouse button presses for focus changes
    for event in mouse_events.read() {
        if event.state == ButtonState::Pressed {
            // Process interaction changes (mouse over/click detection)
            for (entity, interaction, focusable, mut input) in interaction_query.iter_mut() {
                if *interaction == Interaction::Pressed && focusable.mouse_focusable {
                    // Clear focus from previous element
                    if let Some(previous_entity) = focus.focused_entity
                        && previous_entity != entity
                    {
                        // Note: We'd need to query the previous element separately
                        // This is a simplified version - in practice you'd want to
                        // handle this through the main focus change system
                    }

                    // Set focus to clicked element
                    focus.set_focus(entity);
                    focus.hide_focus_indicator(); // Mouse interaction doesn't show keyboard focus indicator

                    input.is_focused = true;
                    input.focus_visible = false; // Mouse focus doesn't show visual indicator

                    debug!("Mouse focus set to entity: {:?}", entity);
                    break;
                }
            }
        }
    }
}

/// System to handle focus loss when clicking outside focusable elements
/// Ensures proper focus management for modal behaviors
#[inline]
pub fn handle_focus_loss_system(
    mut mouse_events: EventReader<MouseButtonInput>,
    mut focus: ResMut<InputFocus>,
    mut text_input_query: Query<&mut InteractiveTextInput>,
    interaction_query: Query<&Interaction, With<Focusable>>,
) {
    for event in mouse_events.read() {
        if event.state == ButtonState::Pressed {
            // Check if any focusable element was clicked
            let clicked_focusable = interaction_query
                .iter()
                .any(|interaction| *interaction == Interaction::Pressed);

            if !clicked_focusable {
                // Clicked outside any focusable element - clear focus
                if let Some(focused_entity) = focus.focused_entity {
                    if let Ok(mut input) = text_input_query.get_mut(focused_entity) {
                        input.is_focused = false;
                        input.focus_visible = false;
                        input.clear_selection();
                    }

                    focus.clear_focus();
                    debug!("Focus cleared - clicked outside focusable elements");
                }
            }
        }
    }
}

/// System to handle escape key for focus management
/// Provides consistent escape behavior across the application
#[inline]
pub fn handle_escape_focus_system(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut focus: ResMut<InputFocus>,
    mut text_input_query: Query<&mut InteractiveTextInput>,
) {
    for event in keyboard_events.read() {
        if event.state.is_pressed()
            && event.logical_key == Key::Escape
            && let Some(focused_entity) = focus.focused_entity
            && let Ok(mut input) = text_input_query.get_mut(focused_entity)
        {
            // Clear selection first if there is one
            if input.has_selection() {
                input.clear_selection();
                debug!("Cleared text selection on escape");
            } else {
                // No selection - clear focus entirely
                input.is_focused = false;
                input.focus_visible = false;
                focus.clear_focus();
                debug!("Cleared focus on escape");
            }
        }
    }
}

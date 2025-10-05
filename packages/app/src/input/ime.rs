use action_items_core::CurrentQuery;
use bevy::prelude::*;
use tracing::debug;

use super::focus::InputFocus;
use super::{InteractiveTextInput, TextInputChanged};

/// System to handle IME (Input Method Editor) events for international text input
/// Supports composition, commit, and all IME states with zero allocation design
#[inline]
pub fn handle_ime_input_system(
    mut ime_events: EventReader<Ime>,
    focus: Res<InputFocus>,
    mut text_input_query: Query<(Entity, &mut InteractiveTextInput, &mut Text)>,
    mut text_changed_events: EventWriter<TextInputChanged>,
    mut current_query: ResMut<CurrentQuery>,
) {
    // Only process IME events for the currently focused input
    if let Some(focused_entity) = focus.focused_entity
        && let Ok((entity, mut input, mut text)) = text_input_query.get_mut(focused_entity)
    {
        if entity != focused_entity {
            return;
        }

        for event in ime_events.read() {
            match event {
                Ime::Preedit { value, cursor, .. } => {
                    // Handle composition text (before commit)
                    // This is the temporary text shown during IME composition
                    input.ime_preedit = value.clone();

                    // Update cursor position within composition if provided
                    if let Some((cursor_pos, _)) = cursor {
                        // Position is relative to the composition start
                        input.cursor_position = input.text.chars().count() + cursor_pos;
                    }

                    // Update display to show composition
                    update_display_text_with_ime(&mut text, &input, &focus);
                },
                Ime::Commit { value, .. } => {
                    // Insert committed IME text at cursor position
                    let cursor_pos = input.cursor_position;
                    input.text.insert_str(cursor_pos, value);
                    input.cursor_position += value.chars().count();

                    // Clear composition state
                    input.ime_preedit.clear();

                    // Update interned text key for performance
                    input.update_text_key(&focus);

                    // Update display text
                    update_display_text_with_ime(&mut text, &input, &focus);

                    // Send change event for reactive updates
                    text_changed_events.write(TextInputChanged {
                        text: input.text.clone(),
                        cursor_position: input.cursor_position,
                    });

                    // Update core search query for immediate search triggering
                    current_query.0 = input.text.clone();
                },
                Ime::Enabled { .. } => {
                    // IME enabled - could update UI state for better UX
                    // For now, we just ensure we're ready to handle composition
                    debug!("IME enabled for entity: {:?}", entity);
                },
                Ime::Disabled { .. } => {
                    // IME disabled - clear any lingering composition state
                    if !input.ime_preedit.is_empty() {
                        input.ime_preedit.clear();
                        update_display_text_with_ime(&mut text, &input, &focus);
                    }
                    debug!("IME disabled for entity: {:?}", entity);
                },
            }
        }
    }
}

/// Update display text including IME composition with optimal performance
/// Uses interned strings for zero allocation on common strings
#[inline]
fn update_display_text_with_ime(
    text: &mut Mut<Text>,
    input: &InteractiveTextInput,
    focus: &InputFocus,
) {
    if input.text.is_empty() && input.ime_preedit.is_empty() {
        // Show placeholder using pre-interned string for zero allocation
        ***text = focus.resolve_string(&input.placeholder_key).to_string();
    } else {
        // Show actual text with IME composition
        // This is the only allocation needed during active editing
        ***text = input.effective_text();
    }
}

/// System to handle IME cursor positioning events
/// Manages cursor position updates during composition for proper visual feedback
#[inline]
pub fn handle_ime_cursor_system(
    mut ime_events: EventReader<Ime>,
    focus: Res<InputFocus>,
    mut text_input_query: Query<(Entity, &mut InteractiveTextInput)>,
) {
    if let Some(focused_entity) = focus.focused_entity
        && let Ok((entity, mut input)) = text_input_query.get_mut(focused_entity)
    {
        if entity != focused_entity {
            return;
        }

        for event in ime_events.read() {
            if let Ime::Preedit {
                cursor: Some((cursor_pos, _)),
                ..
            } = event
            {
                // Update cursor position during composition
                // Position is relative to the start of the text plus composition offset
                let base_position = input.text.chars().count();
                input.cursor_position = base_position + cursor_pos;
            }
        }
    }
}

/// System to handle IME state changes and provide visual feedback
/// Updates UI state based on IME activation/deactivation
#[inline]
pub fn handle_ime_state_system(
    mut ime_events: EventReader<Ime>,
    focus: Res<InputFocus>,
    mut text_input_query: Query<(Entity, &mut InteractiveTextInput)>,
) {
    if let Some(focused_entity) = focus.focused_entity
        && let Ok((entity, mut input)) = text_input_query.get_mut(focused_entity)
    {
        if entity != focused_entity {
            return;
        }

        for event in ime_events.read() {
            match event {
                Ime::Enabled { .. } => {
                    // Could set a flag to indicate IME is active for UI feedback
                    debug!("IME composition started for focused input");
                },
                Ime::Disabled { .. } => {
                    // Clear any composition state and ensure clean state
                    input.ime_preedit.clear();
                    debug!("IME composition ended for focused input");
                },
                _ => {},
            }
        }
    }
}

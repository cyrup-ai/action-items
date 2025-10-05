//! Interactive text input system for Action Items launcher
//!
//! Professional text input handling using Bevy KeyboardInput events with zero-allocation
//! string interning and comprehensive IME support following patterns from
//! /docs/bevy/examples/input/text_input.rs
use action_items_core::{CurrentQuery, LauncherEvent, LauncherEventType};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use lasso::Spur;

use super::focus::InputFocus;

/// Component for interactive text input functionality with zero-allocation string handling
#[derive(Component, Debug, Default)]
pub struct InteractiveTextInput {
    /// Current text content (interned string key for performance)
    pub text_key: Option<Spur>,
    /// Raw text for editing operations
    pub text: String,
    /// Cursor position within the text
    pub cursor_position: usize,
    /// Whether this input currently has focus
    pub is_focused: bool,
    /// Placeholder text key (pre-interned for zero allocation)
    pub placeholder_key: Spur,
    /// Visual focus state for styling (keyboard vs mouse interaction)
    pub focus_visible: bool,
    /// IME composition state for international input
    pub ime_preedit: String,
    /// Selection start position for text selection
    pub selection_start: Option<usize>,
}

impl InteractiveTextInput {
    /// Update the interned text key when text changes
    pub fn update_text_key(&mut self, focus: &InputFocus) {
        if self.text.is_empty() {
            self.text_key = Some(focus.empty_text_key());
        } else {
            self.text_key = Some(focus.intern_string(&self.text));
        }
    }

    /// Get the current effective text (considering IME composition)
    pub fn effective_text(&self) -> String {
        if self.ime_preedit.is_empty() {
            self.text.clone()
        } else {
            format!("{}{}", self.text, self.ime_preedit)
        }
    }

    /// Check if there's an active text selection
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }

    /// Get the selected text range (start, end) sorted
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            let end = self.cursor_position;
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        })
    }

    /// Clear the current selection
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }
}

/// Event sent when text input changes
#[derive(Event, Debug, Clone)]
pub struct TextInputChanged {
    pub text: String,
    pub cursor_position: usize,
}

/// System to handle keyboard input for interactive text fields
/// Zero allocation, blazing-fast input processing using proper Bevy patterns
#[inline]
pub fn handle_interactive_text_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    mut text_input_query: Query<(Entity, &mut InteractiveTextInput, &mut Text)>,
    mut text_changed_events: EventWriter<TextInputChanged>,
    mut current_query: ResMut<CurrentQuery>,
) {
    // Only process keyboard input for the currently focused text input
    if let Some(focused_entity) = focus.focused_entity
        && let Ok((_entity, mut input, mut text)) = text_input_query.get_mut(focused_entity)
    {
        for event in keyboard_events.read() {
            if !event.state.is_pressed() {
                continue;
            }

            match (&event.logical_key, &event.text) {
                // Handle backspace
                (Key::Backspace, _) => {
                    if input.cursor_position > 0 && !input.text.is_empty() {
                        let cursor_pos = input.cursor_position - 1;
                        input.text.remove(cursor_pos);
                        input.cursor_position = cursor_pos;

                        // Update string interning for performance
                        input.update_text_key(&focus);

                        // Update display text
                        update_display_text(&mut text, &input, &focus);

                        // Send change event
                        text_changed_events.write(TextInputChanged {
                            text: input.text.clone(),
                            cursor_position: input.cursor_position,
                        });

                        // Update core search query
                        current_query.0 = input.text.clone();
                    }
                },
                // Handle left arrow (cursor movement)
                (Key::ArrowLeft, _) => {
                    let shift_pressed = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
                    if shift_pressed {
                        // Shift+Left: Start or extend selection
                        if input.selection_start.is_none() {
                            input.selection_start = Some(input.cursor_position);
                        }
                    } else {
                        // Clear selection on cursor movement without shift
                        input.clear_selection();
                    }

                    if input.cursor_position > 0 {
                        input.cursor_position -= 1;
                    }
                },
                // Handle right arrow (cursor movement)
                (Key::ArrowRight, _) => {
                    let shift_pressed = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
                    if shift_pressed {
                        // Shift+Right: Start or extend selection
                        if input.selection_start.is_none() {
                            input.selection_start = Some(input.cursor_position);
                        }
                    } else {
                        // Clear selection on cursor movement without shift
                        input.clear_selection();
                    }

                    if input.cursor_position < input.text.chars().count() {
                        input.cursor_position += 1;
                    }
                },
                // Handle Home key (beginning of line)
                (Key::Home, _) => {
                    let shift_pressed = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
                    if shift_pressed && input.selection_start.is_none() {
                        input.selection_start = Some(input.cursor_position);
                    } else if !shift_pressed {
                        input.clear_selection();
                    }
                    input.cursor_position = 0;
                },
                // Handle End key (end of line)
                (Key::End, _) => {
                    let shift_pressed = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
                    if shift_pressed && input.selection_start.is_none() {
                        input.selection_start = Some(input.cursor_position);
                    } else if !shift_pressed {
                        input.clear_selection();
                    }
                    input.cursor_position = input.text.chars().count();
                },
                // Handle Delete key
                (Key::Delete, _) => {
                    if input.has_selection() {
                        // Delete selected text
                        if let Some((start, end)) = input.selection_range() {
                            input.text.drain(start..end);
                            input.cursor_position = start;
                            input.clear_selection();
                        }
                    } else if input.cursor_position < input.text.chars().count() {
                        // Delete character after cursor
                        let mut chars: Vec<char> = input.text.chars().collect();
                        if input.cursor_position < chars.len() {
                            chars.remove(input.cursor_position);
                            input.text = chars.into_iter().collect();
                        }
                    }

                    // Update string interning and display
                    input.update_text_key(&focus);
                    update_display_text(&mut text, &input, &focus);

                    // Send change event
                    text_changed_events.write(TextInputChanged {
                        text: input.text.clone(),
                        cursor_position: input.cursor_position,
                    });
                    current_query.0 = input.text.clone();
                },
                // Handle character input
                (_, Some(inserted_text)) => {
                    if inserted_text.chars().all(is_printable_char) {
                        // Delete selected text first if there's a selection
                        if input.has_selection()
                            && let Some((start, end)) = input.selection_range()
                        {
                            input.text.drain(start..end);
                            input.cursor_position = start;
                            input.clear_selection();
                        }

                        // Insert new text at cursor position
                        let cursor_pos = input.cursor_position;
                        input.text.insert_str(cursor_pos, inserted_text);
                        input.cursor_position += inserted_text.chars().count();

                        // Update string interning for performance
                        input.update_text_key(&focus);

                        // Update display text
                        update_display_text(&mut text, &input, &focus);

                        // Send change event
                        text_changed_events.write(TextInputChanged {
                            text: input.text.clone(),
                            cursor_position: input.cursor_position,
                        });

                        // Update core search query
                        current_query.0 = input.text.clone();
                    }
                },
                _ => continue,
            }
        }
    }
}

/// Update the display text based on input state with IME support
/// Shows placeholder when empty, actual text with composition when populated
#[inline]
fn update_display_text(text: &mut Mut<Text>, input: &InteractiveTextInput, focus: &InputFocus) {
    if input.text.is_empty() && input.ime_preedit.is_empty() {
        // Show placeholder text using interned string for zero allocation
        ***text = focus.resolve_string(&input.placeholder_key).to_string();
    } else {
        // Show actual text with IME composition
        ***text = input.effective_text();
    }
}

/// Character filter function from Bevy text input example
/// Filters out control characters and private use area characters
#[inline]
fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

/// System to trigger search events based on text changes
/// Debounced search triggering for performance
#[inline]
pub fn trigger_search_from_text_input(
    mut text_changed_events: EventReader<TextInputChanged>,
    mut launcher_events: EventWriter<LauncherEvent>,
) {
    for event in text_changed_events.read() {
        // Send search event to trigger real-time search
        launcher_events.write(LauncherEvent::new(LauncherEventType::SearchStarted(
            event.text.clone(),
        )));
    }
}

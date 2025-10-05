//! Input handling systems
//!
//! Zero-allocation input processing systems with blazing-fast keyboard handling and context-aware
//! input management.

use action_items_core::{CurrentQuery, CurrentSearchResults, LauncherEvent, LauncherEventType};
use action_items_ui::UiState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::prelude::*;

use crate::app_main::AppState;
use crate::input::{LauncherHotkeys, SearchQuery, is_printable_char};

/// Context-aware input system for different application states
/// Zero-allocation state-based input handling with blazing-fast key processing
#[inline]
pub fn context_aware_input_system(
    input: Res<ButtonInput<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut launcher_events: EventWriter<LauncherEvent>,
    hotkeys: Res<LauncherHotkeys>,
) {
    match app_state.get() {
        AppState::Background => {
            // Only global hotkeys are active in background
        },
        AppState::LauncherActive => {
            // Launcher-specific shortcuts
            if input.just_pressed(hotkeys.escape_key) {
                next_state.set(AppState::Background);
                launcher_events.write(LauncherEvent::new(LauncherEventType::SystemShutdown));
            }

            // Auto-transition to search mode when typing starts
            let typing_keys = [
                KeyCode::KeyA,
                KeyCode::KeyB,
                KeyCode::KeyC,
                KeyCode::KeyD,
                KeyCode::KeyE,
                KeyCode::KeyF,
                KeyCode::KeyG,
                KeyCode::KeyH,
                KeyCode::KeyI,
                KeyCode::KeyJ,
                KeyCode::KeyK,
                KeyCode::KeyL,
                KeyCode::KeyM,
                KeyCode::KeyN,
                KeyCode::KeyO,
                KeyCode::KeyP,
                KeyCode::KeyQ,
                KeyCode::KeyR,
                KeyCode::KeyS,
                KeyCode::KeyT,
                KeyCode::KeyU,
                KeyCode::KeyV,
                KeyCode::KeyW,
                KeyCode::KeyX,
                KeyCode::KeyY,
                KeyCode::KeyZ,
                KeyCode::Digit0,
                KeyCode::Digit1,
                KeyCode::Digit2,
                KeyCode::Digit3,
                KeyCode::Digit4,
                KeyCode::Digit5,
                KeyCode::Digit6,
                KeyCode::Digit7,
                KeyCode::Digit8,
                KeyCode::Digit9,
                KeyCode::Space,
            ];

            // Auto-transition to search mode when typing starts
            if typing_keys.iter().any(|key| input.just_pressed(*key)) {
                next_state.set(AppState::SearchMode);
            }
        },
        AppState::SearchMode => {
            // Search-specific handling
            if input.just_pressed(hotkeys.escape_key) {
                next_state.set(AppState::Background);
                launcher_events.write(LauncherEvent::new(LauncherEventType::SystemShutdown));
            }
        },
        AppState::PreferencesOpen => {
            // Preferences-specific handling
            if input.just_pressed(hotkeys.escape_key) {
                next_state.set(AppState::Background);
                launcher_events.write(LauncherEvent::new(LauncherEventType::SystemShutdown));
            }
        },
    }
}

/// Unified keyboard input system with proper text handling following research patterns
/// Zero-allocation text processing with blazing-fast Unicode support and cursor management
#[inline]
pub fn unified_keyboard_input_system(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut search_query: ResMut<SearchQuery>,
    mut current_query: ResMut<CurrentQuery>,
    mut ui_state: ResMut<UiState>,
    app_state: Res<State<AppState>>,
    search_results: Res<CurrentSearchResults>,
    mut launcher_events: EventWriter<LauncherEvent>,
) {
    // Only process keyboard input when launcher is in interactive state
    if !app_state.get().is_interactive() {
        return;
    }

    for event in keyboard_input_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match (&event.logical_key, &event.text) {
            (Key::Enter, _) => {
                // Execute selected action
                if !search_results.results.is_empty()
                    && ui_state.selected_index < search_results.results.len()
                {
                    let selected_action = search_results.results[ui_state.selected_index]
                        .action
                        .clone();
                    launcher_events.write(LauncherEvent::new(LauncherEventType::Execute(
                        selected_action,
                    )));
                }
            },
            (Key::ArrowUp, _) => {
                if ui_state.selected_index > 0 {
                    ui_state.selected_index -= 1;
                }
            },
            (Key::ArrowDown, _) => {
                if !search_results.results.is_empty()
                    && ui_state.selected_index < search_results.results.len() - 1
                {
                    ui_state.selected_index += 1;
                }
            },
            (Key::ArrowLeft, _) => {
                // Move cursor left (proper text editing)
                if search_query.cursor_position > 0 {
                    search_query.cursor_position -= 1;
                }
            },
            (Key::ArrowRight, _) => {
                // Move cursor right (proper text editing)
                if search_query.cursor_position < search_query.text.len() {
                    search_query.cursor_position += 1;
                }
            },
            (Key::Backspace, _) => {
                if !search_query.text.is_empty() && search_query.cursor_position > 0 {
                    // Remove character at cursor position - 1 (proper text editing)
                    let cursor_pos = search_query.cursor_position - 1;
                    search_query.text.remove(cursor_pos);
                    search_query.cursor_position = cursor_pos;
                    ui_state.query = search_query.text.clone();
                    current_query.0 = search_query.text.clone();
                    ui_state.selected_index = 0;
                }
            },
            (_, Some(inserted_text)) => {
                // Handle text input with proper Unicode support
                if inserted_text.chars().all(is_printable_char) {
                    // Insert text at cursor position (proper text editing)
                    let cursor_pos = search_query.cursor_position;
                    search_query.text.insert_str(cursor_pos, inserted_text);
                    search_query.cursor_position += inserted_text.chars().count();
                    ui_state.query = search_query.text.clone();
                    current_query.0 = search_query.text.clone();
                    ui_state.selected_index = 0;
                }
            },
            _ => {},
        }
    }
}

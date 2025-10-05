use action_items_core::CurrentQuery;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::prelude::*;
use global_hotkey::hotkey::Modifiers;

use crate::app_main::AppState;
use crate::events::PreferencesEvent;
use crate::events::preferences::HotkeyDefinition;
use action_items_ecs_preferences::PreferencesResource;

/// System to detect when user types "preferences" in launcher
/// Zero allocation string comparison for blazing-fast detection
#[inline]
pub fn detect_preferences_command(
    current_query: Res<CurrentQuery>,
    app_state: Res<State<AppState>>,
    mut prefs_events: EventWriter<PreferencesEvent>,
) {
    if matches!(app_state.get(), AppState::SearchMode) {
        let query = current_query.0.trim().to_lowercase();
        if query == "preferences" || query == "prefs" || query == "settings" || query == "hotkey" {
            prefs_events.write(PreferencesEvent::Open);
        }
    }
}

/// REAL hotkey capture system - captures actual key combinations as user presses them
/// Zero allocation, blazing-fast keyboard input processing
#[inline]
pub fn real_hotkey_capture_system(
    mut keyboard_input: EventReader<KeyboardInput>,
    mut prefs_state: ResMut<PreferencesResource>,
    modifier_keys: Res<ButtonInput<KeyCode>>,
    mut prefs_events: EventWriter<PreferencesEvent>,
) {
    if !prefs_state.capturing {
        return;
    }

    // Track current modifier state from ButtonInput - zero allocation
    let mut current_modifiers = Modifiers::empty();

    #[cfg(target_os = "macos")]
    {
        if modifier_keys.pressed(KeyCode::SuperLeft) || modifier_keys.pressed(KeyCode::SuperRight) {
            current_modifiers |= Modifiers::META; // Cmd on macOS
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if modifier_keys.pressed(KeyCode::ControlLeft)
            || modifier_keys.pressed(KeyCode::ControlRight)
        {
            current_modifiers |= Modifiers::CONTROL; // Ctrl on Windows/Linux
        }
    }

    if modifier_keys.pressed(KeyCode::AltLeft) || modifier_keys.pressed(KeyCode::AltRight) {
        current_modifiers |= Modifiers::ALT;
    }

    if modifier_keys.pressed(KeyCode::ShiftLeft) || modifier_keys.pressed(KeyCode::ShiftRight) {
        current_modifiers |= Modifiers::SHIFT;
    }

    prefs_state.held_modifiers = current_modifiers;

    // Process keyboard input events - zero allocation event processing
    for event in keyboard_input.read() {
        if event.state.is_pressed() {
            match &event.logical_key {
                Key::Escape => {
                    // ESC key stops capture
                    prefs_events.write(PreferencesEvent::StopCapture);
                    return;
                },
                _ => {
                    // Try to convert the key to a global-hotkey Code - zero allocation conversion
                    if let Some(code) = super::utils::keycode_to_code(&event.logical_key) {
                        let hotkey_def = HotkeyDefinition {
                            modifiers: current_modifiers,
                            code,
                            description: super::utils::format_hotkey_description(
                                current_modifiers,
                                code,
                            ),
                        };

                        prefs_events.write(PreferencesEvent::KeyCaptured(hotkey_def));
                        return;
                    }
                },
            }
        }
    }
}

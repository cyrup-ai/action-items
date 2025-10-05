//! Real-time hotkey capture system
//!
//! Production hotkey capture system extracted from key_capture.rs

use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::prelude::*;
use global_hotkey::hotkey::{Code, Modifiers};

use crate::events::{HotkeyDefinition, PreferencesEvent};
use crate::resources::HotkeyCaptureState;

/// REAL hotkey capture system - captures actual key combinations as user presses them
/// Zero allocation, blazing-fast keyboard input processing
/// Extracted from production key_capture.rs
#[inline]
pub fn real_hotkey_capture_system(
    mut keyboard_input: EventReader<KeyboardInput>,
    mut capture_state: ResMut<HotkeyCaptureState>,
    modifier_keys: Res<ButtonInput<KeyCode>>,
    mut prefs_events: Option<EventWriter<PreferencesEvent>>,
) {
    if !capture_state.capturing {
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

    capture_state.held_modifiers = current_modifiers;

    // Process keyboard input events - zero allocation event processing
    for event in keyboard_input.read() {
        if event.state.is_pressed() {
            match &event.logical_key {
                Key::Escape => {
                    // ESC key stops capture
                    if let Some(ref mut events) = prefs_events {
                        events.write(PreferencesEvent::StopCapture);
                    }
                    return;
                },
                _ => {
                    // Try to convert the key to a global-hotkey Code - zero allocation conversion
                    if let Some(code) = keycode_to_code(&event.logical_key) {
                        let hotkey_def = HotkeyDefinition {
                            modifiers: current_modifiers,
                            code,
                            description: crate::events::format_hotkey_description(
                                current_modifiers,
                                code,
                            ),
                        };

                        if let Some(ref mut events) = prefs_events {
                            events.write(PreferencesEvent::KeyCaptured(hotkey_def));
                        }
                        return;
                    }
                },
            }
        }
    }
}

/// REAL key code conversion - converts Bevy logical keys to global-hotkey codes
/// Zero allocation key conversion for blazing-fast processing
/// Extracted from production utils.rs
#[inline]
pub fn keycode_to_code(logical_key: &Key) -> Option<Code> {
    match logical_key {
        Key::Space => Some(Code::Space),
        Key::Enter => Some(Code::Enter),
        Key::Escape => Some(Code::Escape),
        Key::Tab => Some(Code::Tab),
        Key::Backspace => Some(Code::Backspace),
        Key::ArrowUp => Some(Code::ArrowUp),
        Key::ArrowDown => Some(Code::ArrowDown),
        Key::ArrowLeft => Some(Code::ArrowLeft),
        Key::ArrowRight => Some(Code::ArrowRight),
        Key::Character(ch) => {
            let ch = ch.chars().next()?.to_ascii_uppercase();
            match ch {
                'A' => Some(Code::KeyA),
                'B' => Some(Code::KeyB),
                'C' => Some(Code::KeyC),
                'D' => Some(Code::KeyD),
                'E' => Some(Code::KeyE),
                'F' => Some(Code::KeyF),
                'G' => Some(Code::KeyG),
                'H' => Some(Code::KeyH),
                'I' => Some(Code::KeyI),
                'J' => Some(Code::KeyJ),
                'K' => Some(Code::KeyK),
                'L' => Some(Code::KeyL),
                'M' => Some(Code::KeyM),
                'N' => Some(Code::KeyN),
                'O' => Some(Code::KeyO),
                'P' => Some(Code::KeyP),
                'Q' => Some(Code::KeyQ),
                'R' => Some(Code::KeyR),
                'S' => Some(Code::KeyS),
                'T' => Some(Code::KeyT),
                'U' => Some(Code::KeyU),
                'V' => Some(Code::KeyV),
                'W' => Some(Code::KeyW),
                'X' => Some(Code::KeyX),
                'Y' => Some(Code::KeyY),
                'Z' => Some(Code::KeyZ),
                '0' => Some(Code::Digit0),
                '1' => Some(Code::Digit1),
                '2' => Some(Code::Digit2),
                '3' => Some(Code::Digit3),
                '4' => Some(Code::Digit4),
                '5' => Some(Code::Digit5),
                '6' => Some(Code::Digit6),
                '7' => Some(Code::Digit7),
                '8' => Some(Code::Digit8),
                '9' => Some(Code::Digit9),
                _ => None,
            }
        },
        _ => None,
    }
}

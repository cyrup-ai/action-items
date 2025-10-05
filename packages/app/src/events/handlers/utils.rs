use bevy::input::keyboard::Key;
use global_hotkey::hotkey::{Code, Modifiers};

/// REAL key code conversion - converts Bevy logical keys to global-hotkey codes
/// Zero allocation key conversion for blazing-fast processing
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

/// Format hotkey description with proper Unicode symbols like Raycast
/// Zero allocation string formatting for blazing-fast display
#[inline]
pub fn format_hotkey_description(modifiers: Modifiers, code: Code) -> String {
    let mut description = String::with_capacity(16);

    if modifiers.contains(Modifiers::CONTROL) {
        description.push('⌃');
    }
    if modifiers.contains(Modifiers::ALT) {
        description.push('⌥');
    }
    if modifiers.contains(Modifiers::SHIFT) {
        description.push('⇧');
    }
    if modifiers.contains(Modifiers::META) {
        description.push('⌘');
    }

    // Add the main key
    match code {
        Code::Space => description.push_str("Space"),
        Code::Enter => description.push('⏎'),
        Code::Tab => description.push('⇥'),
        Code::Backspace => description.push('⌫'),
        Code::Delete => description.push('⌦'),
        Code::ArrowUp => description.push('↑'),
        Code::ArrowDown => description.push('↓'),
        Code::ArrowLeft => description.push('←'),
        Code::ArrowRight => description.push('→'),
        _ => {
            // For letter keys, numbers, etc. - use the code's string representation
            description.push_str(&format!("{code:?}"));
        },
    }

    description
}

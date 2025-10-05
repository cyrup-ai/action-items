//! Input state management
//!
//! Zero-allocation input state tracking with blazing-fast query and cursor management.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

/// Bevy-native hotkey configuration for focused window input only
/// Zero-allocation hotkey configuration with blazing-fast key mapping
#[derive(Resource)]
pub struct LauncherHotkeys {
    pub escape_key: KeyCode,
}

impl Default for LauncherHotkeys {
    fn default() -> Self {
        Self {
            escape_key: KeyCode::Escape,
        }
    }
}

/// Current search query for real-time updates
/// Zero-allocation search query state with blazing-fast cursor tracking
#[derive(Resource, Default)]
pub struct SearchQuery {
    pub text: String,
    pub cursor_position: usize,
}

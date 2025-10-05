//! Preferences state management

use bevy::prelude::*;
use ecs_hotkey::{HotkeyPreferences, HotkeyDefinition};
use global_hotkey::hotkey::{Code, Modifiers};

/// Hotkey status enumeration for UI display
#[derive(Debug, Clone, Default)]
pub enum HotkeyStatus {
    #[default]
    Empty,
    Valid,
    /// Conflict with application name
    Conflict(String),
    Testing,
    TestSuccess,
    TestFailed(String),
}

/// Preferences UI state resource
#[derive(Resource)]
pub struct PreferencesResource {
    /// Whether preferences window is visible
    pub is_visible: bool,
    /// Whether preferences are currently loading from disk
    pub loading: bool,
    /// Whether preferences are currently being saved to disk
    pub saving: bool,

    // Hotkey capture state
    /// Is the hotkey input field focused?
    pub input_focused: bool,
    /// Currently recording keystrokes?
    pub capturing: bool,
    /// Currently held modifier keys - updated in real-time
    pub held_modifiers: Modifiers,
    /// Main key that was pressed
    pub captured_key: Option<Code>,
    /// Complete captured combination
    pub captured_hotkey: Option<HotkeyDefinition>,

    // Status and testing
    /// Current hotkey status for UI display
    pub current_status: HotkeyStatus,
    /// Whether currently testing a hotkey
    pub testing_hotkey: bool,
    /// Available alternative hotkey combinations
    pub available_alternatives: Vec<HotkeyDefinition>,
    /// Last error message from file operations
    pub last_error: Option<String>,
    /// Timestamp of last successful save
    pub last_save_success: Option<std::time::SystemTime>,
    /// Currently loaded preferences from disk
    pub loaded_preferences: Option<HotkeyPreferences>,
}

impl Default for PreferencesResource {
    fn default() -> Self {
        Self {
            is_visible: false,
            loading: false,
            saving: false,
            input_focused: false,
            capturing: false,
            held_modifiers: Modifiers::empty(),
            captured_key: None,
            captured_hotkey: None,
            current_status: HotkeyStatus::default(),
            testing_hotkey: false,
            available_alternatives: Vec::new(),
            last_error: None,
            last_save_success: None,
            loaded_preferences: None,
        }
    }
}

/// Load preferred alternative hotkey combinations from user preferences
pub fn load_preferred_alternatives(
    prefs_state: &mut PreferencesResource,
    hotkey_prefs: &HotkeyPreferences,
) {
    prefs_state.available_alternatives = hotkey_prefs.preferred_combinations.clone();
}

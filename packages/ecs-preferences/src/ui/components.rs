use bevy::prelude::*;

/// Preferences window container marker
#[derive(Component)]
pub struct PreferencesContainer;

/// Close button marker
#[derive(Component)]
pub struct PreferencesCloseButton;

/// Hotkey display component
#[derive(Component)]
pub struct HotkeyDisplay {
    pub current_hotkey: Option<String>,
}

/// Hotkey recorder button
#[derive(Component)]
pub struct HotkeyRecorderButton;

/// Recording overlay (shown during capture)
#[derive(Component)]
pub struct HotkeyRecordingOverlay;

/// Save button marker
#[derive(Component)]
pub struct PreferencesSaveButton;

/// Cancel button marker
#[derive(Component)]
pub struct PreferencesCancelButton;

/// Status message component
#[derive(Component)]
pub struct StatusMessage {
    pub message: String,
    pub is_error: bool,
}

/// Alternative hotkey option
#[derive(Component)]
pub struct AlternativeHotkeyOption {
    pub index: usize,
}

use bevy::prelude::*;
use ecs_hotkey::HotkeyDefinition;

/// Request to show preferences window
#[derive(Event, Debug, Clone)]
pub struct PreferencesShowRequested;

/// Request to hide preferences window
#[derive(Event, Debug, Clone)]
pub struct PreferencesHideRequested;

/// Preferences window visibility changed
#[derive(Event, Debug, Clone)]
pub struct PreferencesVisibilityChanged {
    pub is_visible: bool,
}

/// Request to save preferences
#[derive(Event, Debug, Clone)]
pub struct PreferencesSaveRequested {
    pub hotkey: HotkeyDefinition,
}

/// Preferences saved successfully
#[derive(Event, Debug, Clone)]
pub struct PreferencesSaved;

/// Hotkey recording started
#[derive(Event, Debug, Clone)]
pub struct HotkeyRecordingStarted;

/// Hotkey recorded
#[derive(Event, Debug, Clone)]
pub struct HotkeyRecorded {
    pub hotkey: HotkeyDefinition,
    pub has_conflict: bool,
    pub conflict_with: Option<String>,
}

/// Hotkey recording cancelled
#[derive(Event, Debug, Clone)]
pub struct HotkeyRecordingCancelled;

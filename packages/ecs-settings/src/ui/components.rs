use bevy::prelude::*;
use crate::navigation::SettingsTab;
use ecs_service_bridge::components::PluginType;

#[derive(Component)]
pub struct SettingsWindow;

#[derive(Component)]
pub struct SettingsSidebar;

#[derive(Component)]
pub struct SettingsTabButton {
    pub tab: SettingsTab,
}

#[derive(Component)]
pub struct SettingsContentArea {
    pub active_tab: SettingsTab,
}

#[derive(Component)]
pub struct SettingControl {
    pub field_name: String,
    pub table: String,  // Database table for this setting
}

#[derive(Component)]
pub struct SettingCheckbox {
    pub checked: bool,
}

#[derive(Component)]
pub struct TextInput {
    pub field_name: String,
    pub value: String,
}

#[derive(Component)]
pub struct DropdownControl {
    pub field_name: String,
    pub options: Vec<String>,
    pub selected: usize,
    pub is_open: bool,
}

#[derive(Component)]
pub struct HotkeyRecorder {
    pub field_name: String,
    pub current_combo: String,
    pub is_recording: bool,
}

#[derive(Component)]
pub struct ExtensionsTableContainer;

#[derive(Component)]
pub struct ExtensionRow {
    pub plugin_id: String,
}

#[derive(Component)]
pub struct ExtensionToggle {
    pub plugin_id: String,
    pub enabled: bool,
}

#[derive(Component)]
pub struct SettingErrorDisplay {
    pub field_name: String,
}

#[derive(Component)]
pub struct ErrorMessage {
    pub timeout: Timer,
}

#[derive(Component)]
pub struct SaveSuccessFeedback {
    pub timer: Timer,
    pub original_color: Color,
}

#[derive(Component)]
pub struct WindowModeCard {
    pub mode: WindowMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowMode {
    Default,
    Compact,
}

#[derive(Component)]
pub struct ThemeStudioButton;

#[derive(Component)]
pub struct ExtensionSearchBar;

#[derive(Component)]
pub struct ExtensionStoreButton;

#[derive(Component)]
pub struct ExtensionFilterPill {
    pub filter_type: PluginType,
    pub active: bool,
}

#[derive(Component)]
pub struct ExtensionSettingsButton {
    pub plugin_id: String,
}


/// Marker for backdrop overlay
#[derive(Component)]
pub struct SettingsBackdrop;

/// Marker for modal root container
#[derive(Component)]
pub struct SettingsModalRoot;

/// Marker for title bar
#[derive(Component)]
pub struct SettingsTitleBar;

/// Marker for close button
#[derive(Component)]
pub struct CloseSettingsButton;

use bevy::prelude::*;

#[derive(Component)]
pub struct HotkeyStatusDisplay;

/// Status bar state for displaying various UI states
#[derive(Resource, Default)]
pub struct StatusBarState {
    pub message: String,
    pub status_type: StatusType,
    pub visible: bool,
    pub auto_hide_timer: Option<f32>,
}

/// Types of status messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}

impl Default for StatusType {
    fn default() -> Self {
        Self::Info
    }
}

impl StatusBarState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show_message(&mut self, message: String, status_type: StatusType) {
        self.message = message;
        self.status_type = status_type;
        self.visible = true;
        self.auto_hide_timer = Some(3.0); // Auto-hide after 3 seconds
    }

    pub fn show_info(&mut self, message: String) {
        self.show_message(message, StatusType::Info);
    }

    pub fn show_success(&mut self, message: String) {
        self.show_message(message, StatusType::Success);
    }

    pub fn show_warning(&mut self, message: String) {
        self.show_message(message, StatusType::Warning);
    }

    pub fn show_error(&mut self, message: String) {
        self.show_message(message, StatusType::Error);
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.auto_hide_timer = None;
    }

    pub fn update_timer(&mut self, delta_time: f32) {
        if let Some(ref mut timer) = self.auto_hide_timer {
            *timer -= delta_time;
            if *timer <= 0.0 {
                self.hide();
            }
        }
    }
}

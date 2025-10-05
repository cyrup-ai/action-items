//! Global accessibility state management

use bevy::prelude::*;

/// Resource for managing global accessibility state
#[derive(Resource, Debug, Default)]
pub struct AccessibilityManager {
    /// Currently focused element
    pub focused_element: Option<Entity>,
    /// Focus navigation history
    pub focus_history: Vec<Entity>,
    /// Screen reader active state
    pub screen_reader_active: bool,
    /// High contrast mode (WCAG 1.4.6)
    pub high_contrast: bool,
    /// Reduced motion preference (WCAG 2.3.3)
    pub reduced_motion: bool,
    /// Current announcement queue for screen readers
    pub announcements: Vec<String>,
}

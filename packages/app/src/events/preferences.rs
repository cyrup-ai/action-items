//! Preferences-related events
//!
//! Events for managing hotkey preferences, capture, and testing.

use bevy::prelude::*;
// Use HotkeyDefinition from ECS hotkey service
pub use ecs_hotkey::HotkeyDefinition;

/// Events for hotkey preferences management
#[derive(Event, Clone)]
pub enum PreferencesEvent {
    /// Open preferences window
    Open,
    /// Close preferences window
    Close,

    // REAL hotkey capture events
    /// User clicked input field to start recording
    StartCapture,
    /// Stop recording (ESC or click elsewhere)
    StopCapture,
    /// Real hotkey combination captured
    KeyCaptured(HotkeyDefinition),

    /// Test a specific hotkey
    TestHotkey(HotkeyDefinition),
    /// Apply a specific hotkey
    ApplyHotkey(HotkeyDefinition),
    /// Scan for hotkey conflicts
    ScanForConflicts,
}

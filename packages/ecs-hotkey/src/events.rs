//! Hotkey service events
//!
//! Event definitions for the ECS hotkey service, extracted from production code.

use bevy::prelude::*;
use global_hotkey::hotkey::{Code, Modifiers};
use serde::{Deserialize, Serialize};

use crate::resources::{HotkeyBinding, HotkeyId, HotkeyPreferences};

/// Definition of a hotkey combination
/// Extracted from production preferences.rs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HotkeyDefinition {
    /// Modifier keys (Cmd, Alt, Ctrl, Shift)
    pub modifiers: Modifiers,
    /// Main key code
    pub code: Code,
    /// Human-readable description
    pub description: String,
}

impl HotkeyDefinition {
    pub fn new(modifiers: Modifiers, code: Code) -> Self {
        Self {
            modifiers,
            code,
            description: format_hotkey_description(modifiers, code),
        }
    }
}

/// Events for hotkey preferences management
/// Extracted from production preferences.rs
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
    /// Preferences loaded from storage
    PreferencesLoaded(HotkeyPreferences),
}

/// Hotkey registration request event
#[derive(Event, Debug, Clone)]
pub struct HotkeyRegisterRequested {
    pub binding: HotkeyBinding,
    pub requester: String,
    pub action: String,
    pub definition: HotkeyDefinition,
}

/// Hotkey registration completion event
#[derive(Event, Debug, Clone)]
pub struct HotkeyRegisterCompleted {
    pub binding: HotkeyBinding,
    pub requester: String,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Hotkey unregistration request event
#[derive(Event, Debug, Clone)]
pub struct HotkeyUnregisterRequested {
    pub hotkey_id: HotkeyId,
    pub requester: String,
}

/// Hotkey unregistration completion event
#[derive(Event, Debug, Clone)]
pub struct HotkeyUnregisterCompleted {
    pub hotkey_id: HotkeyId,
    pub success: bool,
}

/// Hotkey capture start event
#[derive(Event, Debug, Clone)]
pub struct HotkeyCaptureStarted {
    pub target_action: String,
    pub requester: String,
}

/// Hotkey capture completion event
#[derive(Event, Debug, Clone)]
pub struct HotkeyCaptureCompleted {
    pub captured: HotkeyDefinition,
    pub target_action: String,
}

/// Hotkey capture cancellation reasons
#[derive(Debug, Clone)]
pub enum CancelReason {
    UserCancelled,
    EscapePressed,
    FocusLost,
    Timeout,
}

/// Hotkey capture cancelled event
#[derive(Event, Debug, Clone)]
pub struct HotkeyCaptureCancelled {
    pub reason: CancelReason,
    pub requester: String,
}

/// Hotkey pressed event
#[derive(Event, Debug, Clone)]
pub struct HotkeyPressed {
    pub hotkey_id: HotkeyId,
    pub binding: HotkeyBinding,
}

/// Hotkey conflict detected event
#[derive(Event, Debug, Clone)]
pub struct HotkeyConflictDetected {
    pub hotkey_definition: HotkeyDefinition,
    pub conflict_type: String,
    pub conflicting_app: Option<String>,
    pub suggested_alternatives: Vec<HotkeyDefinition>,
}

/// Hotkey test request event
#[derive(Event, Debug, Clone)]
pub struct HotkeyTestRequested {
    pub definition: HotkeyDefinition,
    pub requester: String,
}

/// Hotkey test result event
#[derive(Event, Debug, Clone)]
pub struct HotkeyTestResult {
    pub hotkey_definition: HotkeyDefinition,
    pub requester: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub test_timestamp: std::time::Instant,
}

/// Hotkey preferences updated event
#[derive(Event, Debug, Clone)]
pub struct HotkeyPreferencesUpdated {
    pub preferences: HotkeyPreferences,
    pub requester: String,
}

/// Format hotkey description with proper Unicode symbols like Raycast
/// Zero allocation string formatting for blazing-fast display
/// Extracted from production utils.rs
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

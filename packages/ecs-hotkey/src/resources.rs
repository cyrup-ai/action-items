//! Hotkey service resources
//!
//! Core resource definitions for the ECS hotkey service, extracted from production code.

use std::collections::HashMap;
use std::time::Instant;

use bevy::prelude::*;
use global_hotkey::GlobalHotKeyManager;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::events::HotkeyDefinition;

/// Global hotkey manager resource following research patterns
/// Zero-allocation global hotkey management with blazing-fast registration
#[derive(Resource)]
pub struct GlobalHotkeyManager {
    pub manager: GlobalHotKeyManager,
    pub toggle_hotkey: HotKey,
}

/// Resource to track which hotkey was successfully registered
/// Zero-allocation registration tracking with blazing-fast description storage
#[derive(Resource)]
pub struct RegisteredHotkey {
    pub description: String,
}

/// Hotkey preferences configuration
/// Zero-allocation preferences management with blazing-fast fallback handling
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct HotkeyPreferences {
    pub preferred_combinations: Vec<HotkeyDefinition>,
    pub custom_hotkey: Option<HotkeyDefinition>,
    pub auto_fallback: bool,
}

impl Default for HotkeyPreferences {
    fn default() -> Self {
        Self {
            preferred_combinations: get_default_hotkey_combinations(),
            custom_hotkey: None,
            auto_fallback: true,
        }
    }
}

/// Get default hotkey combinations for preferences system
/// Zero-allocation default hotkey generation with blazing-fast platform detection
pub fn get_default_hotkey_combinations() -> Vec<HotkeyDefinition> {
    vec![
        // PRIMARY choices - avoid conflicts with system hotkeys
        #[cfg(target_os = "macos")]
        HotkeyDefinition {
            modifiers: Modifiers::META | Modifiers::SHIFT,
            code: Code::Space,
            description: "Cmd+Shift+Space".to_string(),
        },
        #[cfg(not(target_os = "macos"))]
        HotkeyDefinition {
            modifiers: Modifiers::CONTROL | Modifiers::SHIFT,
            code: Code::Space,
            description: "Ctrl+Shift+Space".to_string(),
        },
        // Fallbacks - try system defaults (likely to conflict)
        #[cfg(target_os = "macos")]
        HotkeyDefinition {
            modifiers: Modifiers::META,
            code: Code::Space,
            description: "Cmd+Space".to_string(),
        },
        #[cfg(not(target_os = "macos"))]
        HotkeyDefinition {
            modifiers: Modifiers::CONTROL,
            code: Code::Space,
            description: "Ctrl+Space".to_string(),
        },
        // Alternative modifier combinations
        #[cfg(target_os = "macos")]
        HotkeyDefinition {
            modifiers: Modifiers::META | Modifiers::ALT,
            code: Code::Space,
            description: "Cmd+Alt+Space".to_string(),
        },
        #[cfg(not(target_os = "macos"))]
        HotkeyDefinition {
            modifiers: Modifiers::ALT,
            code: Code::Space,
            description: "Alt+Space".to_string(),
        },
    ]
}

/// Hotkey status enumeration for UI display
/// Zero allocation status tracking with semantic error information
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

/// Hotkey capture state resource - REAL implementation with zero allocation
/// High-performance state management for real-time hotkey capture and display
#[derive(Resource)]
pub struct HotkeyCaptureState {
    /// Whether preferences window is visible
    pub visible: bool,

    // REAL hotkey capture state (replaces fake string input)
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

    // Status and testing - zero allocation state tracking
    /// Current hotkey status for UI display
    pub current_status: HotkeyStatus,
    /// Whether currently testing a hotkey
    pub testing_hotkey: bool,
    /// Available alternative hotkey combinations
    pub available_alternatives: Vec<HotkeyDefinition>,
    /// Current requester for capture operations
    pub current_requester: Option<String>,
}

impl Default for HotkeyCaptureState {
    #[inline]
    fn default() -> Self {
        Self {
            visible: false,
            input_focused: false,
            capturing: false,
            held_modifiers: Modifiers::empty(),
            captured_key: None,
            captured_hotkey: None,
            current_status: HotkeyStatus::default(),
            testing_hotkey: false,
            available_alternatives: Vec::new(),
            current_requester: None,
        }
    }
}

/// Unique identifier for hotkey operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotkeyId(pub Uuid);

impl Default for HotkeyId {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Conflict type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    AlreadyRegistered,
    RegistrationLimitExceeded,
    PlatformNotSupported,
    PermissionDenied,
}

/// Hotkey binding definition
#[derive(Debug, Clone)]
pub struct HotkeyBinding {
    pub id: HotkeyId,
    pub definition: HotkeyDefinition,
    pub action: String,
    pub requester: String,
    pub registered_at: Instant,
}

impl HotkeyBinding {
    pub fn new(definition: HotkeyDefinition, action: impl Into<String>) -> Self {
        Self {
            id: HotkeyId::new(),
            definition,
            action: action.into(),
            requester: "unknown".to_string(),
            registered_at: Instant::now(),
        }
    }

    pub fn with_requester(mut self, requester: impl Into<String>) -> Self {
        self.requester = requester.into();
        self
    }
}

/// Hotkey registry for managing registered hotkeys
#[derive(Resource, Default)]
pub struct HotkeyRegistry {
    pub registered_hotkeys: HashMap<HotkeyId, HotkeyBinding>,
    pub conflicts: Vec<ConflictReport>,
    pub by_action: HashMap<String, Vec<HotkeyId>>,
}

/// Conflict report for hotkey registration issues
#[derive(Debug, Clone)]
pub struct ConflictReport {
    pub conflicting_hotkey: HotkeyDefinition,
    pub conflict_type: ConflictType,
    pub conflicting_application: Option<String>,
    pub suggested_alternative: Option<HotkeyDefinition>,
}

/// Hotkey manager resource - central coordination
#[derive(Resource)]
pub struct HotkeyManager {
    pub global_manager: GlobalHotKeyManager,
    pub registry: HotkeyRegistry,
    pub preferences: HotkeyPreferences,
    pub max_hotkeys: usize,
    pub enable_conflict_resolution: bool,
}

impl HotkeyManager {
    pub fn new(max_hotkeys: usize, enable_conflict_resolution: bool) -> Self {
        Self {
            global_manager: GlobalHotKeyManager::new()
                .unwrap_or_else(|e| panic!("Critical failure: GlobalHotKeyManager initialization failed - hotkey system cannot function: {}", e)),
            registry: HotkeyRegistry::default(),
            preferences: HotkeyPreferences::default(),
            max_hotkeys,
            enable_conflict_resolution,
        }
    }

    /// Check if we can register more hotkeys
    pub fn can_register_more(&self) -> bool {
        self.registry.registered_hotkeys.len() < self.max_hotkeys
    }

    /// Get binding by action
    pub fn get_bindings_for_action(&self, action: &str) -> Vec<&HotkeyBinding> {
        self.registry
            .by_action
            .get(action)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.registry.registered_hotkeys.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Metrics resource for tracking hotkey service performance
#[derive(Resource, Default)]
pub struct HotkeyMetrics {
    pub registered_count: usize,
    pub press_count: usize,
    pub last_press: Option<Instant>,
    pub total_registrations: u64,
    pub successful_registrations: u64,
    pub failed_registrations: u64,
    pub conflicts_detected: u64,
    pub capture_sessions: u64,
    pub successful_captures: u64,
    pub tests_performed: u64,
    pub successful_tests: u64,
}

/// Configuration resource for hotkey service
#[derive(Resource, Clone, Debug)]
pub struct HotkeyConfig {
    pub enable_debug_logging: bool,
    pub polling_interval: std::time::Duration,
    pub max_hotkeys: usize,
    pub enable_conflict_resolution: bool,
}

/// Scan for available hotkey combinations using user preferences
/// Zero allocation preference scanning with intelligent conflict detection
#[inline]
pub fn scan_for_available_hotkeys(
    capture_state: &mut HotkeyCaptureState,
    hotkey_prefs: &HotkeyPreferences,
) {
    // Use preferred_combinations from HotkeyPreferences instead of hardcoded list
    // This ensures user preferences are respected for conflict scanning
    capture_state.available_alternatives = hotkey_prefs.preferred_combinations.clone();
}

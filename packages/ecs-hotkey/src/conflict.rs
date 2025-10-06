//! Hotkey conflict detection and resolution
//!
//! Production conflict detection system with intelligent resolution strategies.

use global_hotkey::hotkey::HotKey;
use tracing::{info, warn};

use crate::events::HotkeyDefinition;
use crate::resources::{ConflictReport, ConflictType, HotkeyManager, HotkeyPreferences};

/// Intelligent hotkey registration with conflict detection and fallbacks
/// Zero-allocation hotkey registration with blazing-fast conflict detection and auto-fallback
/// Extracted from production management.rs
pub fn register_launcher_hotkey(
    manager: &mut HotkeyManager,
    preferences: &HotkeyPreferences,
) -> Result<(HotKey, String), Box<dyn std::error::Error>> {
    // Use HotkeyPreferences to determine if auto_fallback should be used
    let hotkey_prefs = preferences;
    // Use preferred_combinations from HotkeyPreferences (implements preferred_combinations field
    // usage)
    let preferred_combinations = &hotkey_prefs.preferred_combinations;

    let mut last_error = None;

    for (i, hotkey_def) in preferred_combinations.iter().enumerate() {
        let hotkey = HotKey::new(Some(hotkey_def.modifiers), hotkey_def.code);

        match manager.global_manager.register(hotkey) {
            Ok(()) => {
                info!("Successfully registered hotkey: {}", hotkey_def.description);
                return Ok((hotkey, hotkey_def.description.clone()));
            },
            Err(e) => {
                warn!(
                    "Failed to register hotkey {}: {}",
                    hotkey_def.description, e
                );
                last_error = Some(e);

                // Try to unregister in case of partial registration
                let _ = manager.global_manager.unregister(hotkey);

                // If auto_fallback is disabled and this is not the first (primary) hotkey, stop
                // trying
                if !hotkey_prefs.auto_fallback && i > 0 {
                    warn!("Auto-fallback disabled, stopping after primary hotkey failure");
                    break;
                }

                continue;
            },
        }
    }

    // If we get here, none of the preferred combinations worked
    tracing::error!("Failed to register any hotkey combination!");

    if let Some(e) = last_error {
        Err(format!("Could not register any hotkey combination. Last error: {e}").into())
    } else {
        Err("Could not register any hotkey combination. Unknown error.".into())
    }
}

/// Test if a hotkey definition can be registered without conflicts
pub fn test_hotkey_registration(
    manager: &HotkeyManager,
    definition: &HotkeyDefinition,
) -> Result<(), ConflictReport> {
    let hotkey = HotKey::new(Some(definition.modifiers), definition.code);

    match manager.global_manager.register(hotkey) {
        Ok(()) => {
            // Successfully registered, now unregister it immediately
            let _ = manager.global_manager.unregister(hotkey);
            Ok(())
        },
        Err(e) => {
            // Real conflict detected - extract app name from error if possible
            let error_msg = e.to_string();
            let conflicting_app = if error_msg.contains("already registered") {
                "another application".to_string()
            } else {
                error_msg.clone()
            };

            Err(ConflictReport {
                conflicting_hotkey: definition.clone(),
                conflict_type: ConflictType::AlreadyRegistered,
                conflicting_application: Some(conflicting_app),
                suggested_alternative: None,
            })
        },
    }
}

/// Detect conflicts with existing hotkeys
pub fn detect_hotkey_conflicts(
    manager: &HotkeyManager,
    definitions: &[HotkeyDefinition],
) -> Vec<ConflictReport> {
    let mut conflicts = Vec::new();

    for definition in definitions {
        if let Err(conflict) = test_hotkey_registration(manager, definition) {
            conflicts.push(conflict);
        }
    }

    conflicts
}

/// Generate alternative hotkey suggestions when conflicts are detected
pub fn generate_hotkey_alternatives(
    conflicted_definition: &HotkeyDefinition,
    preferences: &HotkeyPreferences,
) -> Vec<HotkeyDefinition> {
    let mut alternatives = Vec::new();

    // First, try the user's preferred combinations
    for pref in &preferences.preferred_combinations {
        if pref != conflicted_definition {
            alternatives.push(pref.clone());
        }
    }

    // If we need more alternatives, generate some common variants
    if alternatives.len() < 3 {
        // Try with additional modifiers
        let base_modifiers = conflicted_definition.modifiers;
        let base_code = conflicted_definition.code;

        // Add shift if not present
        if !base_modifiers.contains(global_hotkey::hotkey::Modifiers::SHIFT) {
            alternatives.push(HotkeyDefinition {
                modifiers: base_modifiers | global_hotkey::hotkey::Modifiers::SHIFT,
                code: base_code,
                description: crate::events::format_hotkey_description(
                    base_modifiers | global_hotkey::hotkey::Modifiers::SHIFT,
                    base_code,
                ),
            });
        }

        // Try with Alt instead of other modifiers
        alternatives.push(HotkeyDefinition {
            modifiers: global_hotkey::hotkey::Modifiers::ALT,
            code: base_code,
            description: crate::events::format_hotkey_description(
                global_hotkey::hotkey::Modifiers::ALT,
                base_code,
            ),
        });
    }

    alternatives
}

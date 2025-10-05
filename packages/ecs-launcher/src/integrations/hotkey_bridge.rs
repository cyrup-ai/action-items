//! Hotkey-Launcher Bridge Integration
//!
//! Bridges ecs-hotkey service events with ecs-launcher actions,
//! providing seamless integration between global hotkey detection
//! and launcher functionality.

use bevy::prelude::*;
// Re-export hotkey types for integration
pub use ecs_hotkey::{
    HotkeyBinding, HotkeyDefinition, HotkeyId, HotkeyPressed, HotkeyRegisterCompleted,
    HotkeyRegisterRequested,
};
use tracing::{info, warn};

use crate::events::*;
use crate::resources::*;

/// Resource to manage hotkey-to-action mappings
#[derive(Resource, Default)]
pub struct HotkeyActionMappings {
    pub hotkey_to_action: std::collections::HashMap<String, String>,
    pub action_to_hotkey: std::collections::HashMap<String, String>,
}

impl HotkeyActionMappings {
    /// Register a hotkey-to-action mapping
    pub fn register_mapping(&mut self, hotkey_description: String, action_id: String) {
        self.hotkey_to_action
            .insert(hotkey_description.clone(), action_id.clone());
        self.action_to_hotkey.insert(action_id, hotkey_description);
    }

    /// Get action ID for a hotkey
    pub fn get_action_for_hotkey(&self, hotkey_description: &str) -> Option<&String> {
        self.hotkey_to_action.get(hotkey_description)
    }

    /// Get hotkey for an action ID
    pub fn get_hotkey_for_action(&self, action_id: &str) -> Option<&String> {
        self.action_to_hotkey.get(action_id)
    }
}

/// System to bridge hotkey press events to launcher actions
pub fn bridge_hotkey_to_launcher_system(
    mut hotkey_events: EventReader<HotkeyPressed>,
    mut action_execute_events: EventWriter<ActionExecuteRequested>,
    mappings: Res<HotkeyActionMappings>,
    mut launcher_window_events: EventWriter<LauncherWindowToggled>,
    config: Res<LauncherConfig>,
    launcher_state: Res<LauncherState>,
) {
    for hotkey_event in hotkey_events.read() {
        let hotkey_desc = &hotkey_event.binding.definition.description;

        if config.enable_debug_logging {
            info!("Hotkey pressed: {}", hotkey_desc);
        }

        // Check for launcher toggle hotkeys first
        if is_launcher_toggle_hotkey(hotkey_desc) {
            // Production-quality toggle logic based on current launcher state
            let should_show = !launcher_state.is_window_visible; // Proper toggle: invert current state

            launcher_window_events.write(LauncherWindowToggled {
                visible: should_show,
                trigger: WindowTrigger::Hotkey,
                requester: "hotkey_bridge".to_string(),
            });
            continue;
        }

        // Check for mapped actions
        if let Some(action_id) = mappings.get_action_for_hotkey(hotkey_desc) {
            action_execute_events.write(ActionExecuteRequested {
                action_id: action_id.clone(),
                requester: "hotkey_bridge".to_string(),
                parameters: serde_json::Value::Null,
                execution_context: ExecutionContext {
                    source: ExecutionSource::Hotkey,
                    priority: ExecutionPriority::High,
                    timeout: Some(std::time::Duration::from_secs(10)),
                    environment: std::collections::HashMap::new(),
                    requester: "hotkey_bridge".to_string(),
                },
            });

            if config.enable_debug_logging {
                info!(
                    "Triggered action '{}' from hotkey '{}'",
                    action_id, hotkey_desc
                );
            }
        } else {
            warn!("No action mapped for hotkey: {}", hotkey_desc);
        }
    }
}

/// System to handle launcher action completions and provide feedback
pub fn handle_action_completion_feedback_system(
    mut action_completed_events: EventReader<ActionExecuteCompleted>,
    mut ui_state_events: EventWriter<UIStateChanged>,
    launcher_state: Res<LauncherState>,
    config: Res<LauncherConfig>,
) {
    for completion in action_completed_events.read() {
        if completion.requester == "hotkey_bridge" {
            if config.enable_debug_logging {
                if completion.success {
                    info!(
                        "Hotkey-triggered action '{}' completed successfully",
                        completion.action_id
                    );
                } else {
                    warn!(
                        "Hotkey-triggered action '{}' failed: {:?}",
                        completion.action_id, completion.error_message
                    );
                }
            }

            // Update UI state based on completion
            let new_state = if completion.success {
                UIState::Hidden // Hide launcher after successful execution
            } else {
                UIState::Error // Show error state
            };

            ui_state_events.write(UIStateChanged {
                previous_state: launcher_state.current_ui_state.clone(),
                new_state,
                trigger: UITrigger::SystemEvent,
                requester: "hotkey_bridge".to_string(),
            });
        }
    }
}

/// System to register default launcher hotkey mappings
pub fn setup_default_hotkey_mappings_system(
    mut mappings: ResMut<HotkeyActionMappings>,
    mut hotkey_register_events: EventWriter<HotkeyRegisterRequested>,
) {
    // Register common launcher hotkey mappings
    let default_mappings = vec![
        ("⌘Space", "toggle_launcher"),
        ("Ctrl+Space", "toggle_launcher"),
        ("⌘⇧Space", "toggle_launcher_alt"),
        ("⌘P", "command_palette"),
        ("⌘T", "new_task"),
        ("⌘F", "search_files"),
    ];

    for (hotkey_desc, action_id) in default_mappings {
        mappings.register_mapping(hotkey_desc.to_string(), action_id.to_string());

        // Request hotkey registration (this would be handled by ecs-hotkey service)
        let definition = parse_hotkey_description(hotkey_desc);
        if let Some(definition) = definition {
            let binding = ecs_hotkey::HotkeyBinding::new(definition.clone(), action_id)
                .with_requester("launcher_bridge");

            hotkey_register_events.write(HotkeyRegisterRequested {
                binding,
                requester: "launcher_bridge".to_string(),
                action: action_id.to_string(),
                definition,
            });
        }
    }

    info!(
        "Registered {} default hotkey-to-action mappings",
        mappings.hotkey_to_action.len()
    );
}

/// System to handle dynamic hotkey registration from launcher preferences
pub fn sync_preferences_to_hotkeys_system(
    mut preferences_events: EventReader<LauncherPreferencesUpdated>,
    mut mappings: ResMut<HotkeyActionMappings>,
    mut hotkey_register_events: EventWriter<HotkeyRegisterRequested>,
    config: Res<LauncherConfig>,
) {
    for prefs_event in preferences_events.read() {
        if config.enable_debug_logging {
            info!("Syncing launcher preferences to hotkey mappings");
        }

        // Update mappings from preferences
        for (action, hotkey_desc) in &prefs_event.preferences.global_hotkeys {
            mappings.register_mapping(hotkey_desc.clone(), action.clone());

            // Register new hotkey
            if let Some(definition) = parse_hotkey_description(hotkey_desc) {
                let binding = ecs_hotkey::HotkeyBinding::new(definition.clone(), action)
                    .with_requester("launcher_preferences");

                hotkey_register_events.write(HotkeyRegisterRequested {
                    binding,
                    requester: "launcher_preferences".to_string(),
                    action: action.clone(),
                    definition,
                });
            }
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Check if a hotkey description corresponds to a launcher toggle
fn is_launcher_toggle_hotkey(hotkey_desc: &str) -> bool {
    matches!(hotkey_desc, "⌘Space" | "Ctrl+Space" | "⌘⇧Space")
}

/// Parse hotkey description into HotkeyDefinition
fn parse_hotkey_description(desc: &str) -> Option<ecs_hotkey::HotkeyDefinition> {
    use ecs_hotkey::HotkeyDefinition;
    use global_hotkey::hotkey::{Code, Modifiers};

    // Simple parser for common hotkey patterns
    let mut modifiers = Modifiers::empty();
    let mut key_part = desc.to_string();

    // Parse modifiers
    if desc.contains("⌘") || desc.contains("Cmd") {
        modifiers |= Modifiers::META;
        key_part = key_part
            .replace("⌘", "")
            .replace("Cmd+", "")
            .replace("Cmd", "");
    }
    if desc.contains("Ctrl") {
        modifiers |= Modifiers::CONTROL;
        key_part = key_part.replace("Ctrl+", "").replace("Ctrl", "");
    }
    if desc.contains("⇧") || desc.contains("Shift") {
        modifiers |= Modifiers::SHIFT;
        key_part = key_part
            .replace("⇧", "")
            .replace("Shift+", "")
            .replace("Shift", "");
    }
    if desc.contains("⌥") || desc.contains("Alt") {
        modifiers |= Modifiers::ALT;
        key_part = key_part
            .replace("⌥", "")
            .replace("Alt+", "")
            .replace("Alt", "");
    }

    // Parse key - comprehensive keyboard support for production-quality hotkey parsing
    let code = match key_part.trim() {
        // Letter keys
        "A" => Code::KeyA,
        "B" => Code::KeyB,
        "C" => Code::KeyC,
        "D" => Code::KeyD,
        "E" => Code::KeyE,
        "F" => Code::KeyF,
        "G" => Code::KeyG,
        "H" => Code::KeyH,
        "I" => Code::KeyI,
        "J" => Code::KeyJ,
        "K" => Code::KeyK,
        "L" => Code::KeyL,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "O" => Code::KeyO,
        "P" => Code::KeyP,
        "Q" => Code::KeyQ,
        "R" => Code::KeyR,
        "S" => Code::KeyS,
        "T" => Code::KeyT,
        "U" => Code::KeyU,
        "V" => Code::KeyV,
        "W" => Code::KeyW,
        "X" => Code::KeyX,
        "Y" => Code::KeyY,
        "Z" => Code::KeyZ,

        // Number keys
        "0" => Code::Digit0,
        "1" => Code::Digit1,
        "2" => Code::Digit2,
        "3" => Code::Digit3,
        "4" => Code::Digit4,
        "5" => Code::Digit5,
        "6" => Code::Digit6,
        "7" => Code::Digit7,
        "8" => Code::Digit8,
        "9" => Code::Digit9,

        // Function keys
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,

        // Arrow keys
        "Up" | "ArrowUp" => Code::ArrowUp,
        "Down" | "ArrowDown" => Code::ArrowDown,
        "Left" | "ArrowLeft" => Code::ArrowLeft,
        "Right" | "ArrowRight" => Code::ArrowRight,

        // Navigation keys
        "Home" => Code::Home,
        "End" => Code::End,
        "PageUp" => Code::PageUp,
        "PageDown" => Code::PageDown,
        "Insert" => Code::Insert,
        "Delete" => Code::Delete,

        // Special keys
        "Space" => Code::Space,
        "Enter" => Code::Enter,
        "Tab" => Code::Tab,
        "Escape" => Code::Escape,
        "Backspace" => Code::Backspace,

        // Symbol keys
        "Minus" | "-" => Code::Minus,
        "Equal" | "=" => Code::Equal,
        "BracketLeft" | "[" => Code::BracketLeft,
        "BracketRight" | "]" => Code::BracketRight,
        "Backslash" | "\\" => Code::Backslash,
        "Semicolon" | ";" => Code::Semicolon,
        "Quote" | "'" => Code::Quote,
        "Grave" | "`" => Code::Backquote,
        "Comma" | "," => Code::Comma,
        "Period" | "." => Code::Period,
        "Slash" | "/" => Code::Slash,

        // Numpad keys
        "Numpad0" => Code::Numpad0,
        "Numpad1" => Code::Numpad1,
        "Numpad2" => Code::Numpad2,
        "Numpad3" => Code::Numpad3,
        "Numpad4" => Code::Numpad4,
        "Numpad5" => Code::Numpad5,
        "Numpad6" => Code::Numpad6,
        "Numpad7" => Code::Numpad7,
        "Numpad8" => Code::Numpad8,
        "Numpad9" => Code::Numpad9,
        "NumpadAdd" => Code::NumpadAdd,
        "NumpadSubtract" => Code::NumpadSubtract,
        "NumpadMultiply" => Code::NumpadMultiply,
        "NumpadDivide" => Code::NumpadDivide,
        "NumpadDecimal" => Code::NumpadDecimal,
        "NumpadEnter" => Code::NumpadEnter,

        _ => return None, // Unknown key - fail gracefully
    };

    Some(HotkeyDefinition::new(modifiers, code))
}

/// Plugin to integrate hotkey bridge functionality
pub struct HotkeyLauncherBridgePlugin;

impl Plugin for HotkeyLauncherBridgePlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing Hotkey-Launcher Bridge Plugin");

        // Add bridge resource
        app.insert_resource(HotkeyActionMappings::default());

        // Add bridge systems
        app.add_systems(Startup, setup_default_hotkey_mappings_system);

        app.add_systems(
            Update,
            (
                bridge_hotkey_to_launcher_system,
                handle_action_completion_feedback_system,
                sync_preferences_to_hotkeys_system,
            ),
        );

        info!("Hotkey-Launcher Bridge Plugin initialized successfully");
    }
}

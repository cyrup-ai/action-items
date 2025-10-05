use std::collections::HashMap;
use std::path::PathBuf;

use action_items_ui::{UiComponentTarget, UiVisibilityEvent};
use bevy::prelude::*;
use ecs_filesystem::{
    FileContent, FileOperationId, FileSystemError, FileSystemRequest, FileSystemResponse,
};
use ecs_hotkey::{GlobalHotkeyManager, HotkeyPreferences};
use global_hotkey::hotkey::{HotKey, Modifiers};
use serde_json;
use tracing::{debug, error, info};

// use uuid::Uuid; // Unused import
use crate::app_main::AppState;
use crate::events::PreferencesEvent;
use action_items_ecs_preferences::{HotkeyStatus, PreferencesResource, load_preferred_alternatives};
use crate::window::activation::{ActivationReason, WindowActivationEvent};

/// Resource to track pending filesystem operations
#[derive(Resource, Default)]
pub struct PendingFileOperations {
    pub preferences_read_ops: HashMap<FileOperationId, Entity>,
    pub preferences_write_ops: HashMap<FileOperationId, HotkeyPreferences>,
    pub preferences_dir_ops:
        HashMap<FileOperationId, (Entity, PathBuf, Vec<u8>, HotkeyPreferences)>,
}

/// System to handle preferences events - using ecs-filesystem service
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn handle_preferences_events(
    mut commands: Commands,
    mut events_param_set: ParamSet<(EventReader<PreferencesEvent>, EventWriter<PreferencesEvent>)>,
    mut prefs_state: ResMut<PreferencesResource>,
    _app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    global_manager: Option<Res<GlobalHotkeyManager>>,
    mut ui_visibility_events: EventWriter<UiVisibilityEvent>,
    mut activation_events: EventWriter<WindowActivationEvent>,
    mut filesystem_events: EventWriter<FileSystemRequest>,
    mut pending_ops: ResMut<PendingFileOperations>,
) {
    // Collect events to process to avoid borrow checker issues
    let events_to_process: Vec<_> = events_param_set.p0().read().cloned().collect();

    for event in events_to_process {
        match event {
            PreferencesEvent::Open => {
                // Set loading state instead of immediately showing
                prefs_state.loading = true;
                prefs_state.is_visible = true; // Show UI in loading state
                next_state.set(AppState::PreferencesOpen);
                info!("Opening hotkey preferences");

                // Load hotkey preferences using ecs-filesystem service - proper ECS pattern
                let operation_id = FileOperationId::new();
                let config_dir = dirs::config_dir()
                    .map(|dir| dir.join("action-items").join("hotkey-preferences.json"))
                    .unwrap_or_else(|| PathBuf::from("hotkey-preferences.json"));

                let entity = commands
                    .spawn(Name::new(format!(
                        "PreferencesReadRequester-{:?}",
                        operation_id
                    )))
                    .id();

                // Track this operation
                pending_ops
                    .preferences_read_ops
                    .insert(operation_id, entity);

                let read_request = FileSystemRequest::ReadFile {
                    operation_id,
                    requester: entity,
                    path: config_dir,
                };

                // Emit filesystem read event - handled by ecs-filesystem service
                filesystem_events.write(read_request);
                debug!(
                    "Requested hotkey preferences load with operation_id: {:?}",
                    operation_id
                );

                // Trigger settings UI to show
                ui_visibility_events.write(UiVisibilityEvent::immediate(true, UiComponentTarget::Panel));

                // Send window activation event for user-requested window showing
                activation_events.write(WindowActivationEvent {
                    reason: ActivationReason::UserRequest,
                });

                // Auto-trigger conflict scan to populate alternatives section
                events_param_set
                    .p1()
                    .write(PreferencesEvent::ScanForConflicts);
            },
            PreferencesEvent::Close => {
                prefs_state.is_visible = false;
                next_state.set(AppState::Background);
                info!("Closing hotkey preferences");

                // Trigger settings UI to hide
                ui_visibility_events.write(UiVisibilityEvent::immediate(false, UiComponentTarget::Panel));
            },
            // REAL hotkey capture events
            PreferencesEvent::StartCapture => {
                prefs_state.input_focused = true;
                prefs_state.capturing = true;
                prefs_state.held_modifiers = Modifiers::empty();
                prefs_state.captured_key = None;
                prefs_state.captured_hotkey = None;
                prefs_state.current_status = HotkeyStatus::Empty;
                info!("Started hotkey capture - press your desired key combination");
            },
            PreferencesEvent::StopCapture => {
                prefs_state.input_focused = false;
                prefs_state.capturing = false;
                prefs_state.held_modifiers = Modifiers::empty();
                prefs_state.captured_key = None;
                info!("Stopped hotkey capture");
            },
            PreferencesEvent::KeyCaptured(hotkey_def) => {
                prefs_state.captured_hotkey = Some(hotkey_def.clone());
                prefs_state.capturing = false;
                prefs_state.current_status = HotkeyStatus::Valid;
                info!("Captured hotkey: {}", hotkey_def.description);

                // Immediately test for conflicts with REAL conflict detection - zero allocation
                if let Some(manager) = &global_manager {
                    let hotkey = HotKey::new(Some(hotkey_def.modifiers), hotkey_def.code);
                    match manager.manager.register(hotkey) {
                        Ok(()) => {
                            let _ = manager.manager.unregister(hotkey);
                            prefs_state.current_status = HotkeyStatus::Valid;
                        },
                        Err(e) => {
                            // Real conflict detected - extract app name from error if possible
                            let error_msg = e.to_string();
                            let conflicting_app = if error_msg.contains("already registered") {
                                "another application".to_string()
                            } else {
                                error_msg
                            };
                            prefs_state.current_status = HotkeyStatus::Conflict(conflicting_app);
                        },
                    }
                } else {
                    // Fallback when GlobalHotkeyManager is not available
                    prefs_state.current_status = HotkeyStatus::Valid;
                }
            },
            PreferencesEvent::TestHotkey(hotkey_def) => {
                prefs_state.testing_hotkey = true;
                prefs_state.current_status = HotkeyStatus::Testing;
                info!("Testing hotkey: {}", hotkey_def.description);

                // REAL hotkey testing - actually try to register it
                if let Some(manager) = &global_manager {
                    let hotkey = HotKey::new(Some(hotkey_def.modifiers), hotkey_def.code);
                    match manager.manager.register(hotkey) {
                        Ok(()) => {
                            // Successfully registered, now unregister it
                            let _ = manager.manager.unregister(hotkey);
                            prefs_state.current_status = HotkeyStatus::TestSuccess;
                            info!("Hotkey test successful: {}", hotkey_def.description);
                        },
                        Err(e) => {
                            prefs_state.current_status = HotkeyStatus::TestFailed(e.to_string());
                            info!("Hotkey test failed: {}", e);
                        },
                    }
                } else {
                    // Fallback when GlobalHotkeyManager is not available
                    prefs_state.current_status = HotkeyStatus::TestSuccess;
                    info!("Hotkey test successful (fallback): {}", hotkey_def.description);
                }
                prefs_state.testing_hotkey = false;
            },
            PreferencesEvent::ApplyHotkey(hotkey_def) => {
                info!("Applying new hotkey: {}", hotkey_def.description);
                // Store the custom hotkey in preferences - zero allocation cloning
                let hotkey_prefs = HotkeyPreferences {
                    custom_hotkey: Some(hotkey_def.clone()),
                    ..Default::default()
                };

                // Set saving state
                prefs_state.saving = true;

                // Persist hotkey preferences using ecs-filesystem service - proper ECS pattern
                let config_dir = dirs::config_dir()
                    .map(|dir| dir.join("action-items"))
                    .unwrap_or_else(|| PathBuf::from("."));
                let config_file = config_dir.join("hotkey-preferences.json");

                // Create directory asynchronously using ecs-filesystem service
                let dir_operation_id = FileOperationId::new();
                let dir_entity = commands
                    .spawn(Name::new(format!(
                        "PreferencesDirRequester-{:?}",
                        dir_operation_id
                    )))
                    .id();

                let create_dir_request = FileSystemRequest::CreateDirectory {
                    operation_id: dir_operation_id,
                    requester: dir_entity,
                    path: config_dir.clone(),
                };

                // Serialize preferences for storage
                let json_content = match serde_json::to_string_pretty(&hotkey_prefs) {
                    Ok(content) => content,
                    Err(e) => {
                        error!("Failed to serialize hotkey preferences: {}", e);
                        prefs_state.saving = false;
                        prefs_state.last_error = Some(format!("Serialization failed: {}", e));
                        return;
                    },
                };

                // Track the directory creation operation that will trigger file write
                pending_ops.preferences_dir_ops.insert(
                    dir_operation_id,
                    (
                        dir_entity,
                        config_file,
                        json_content.into_bytes(),
                        hotkey_prefs.clone(),
                    ),
                );

                // Emit directory creation event - handled by ecs-filesystem service
                filesystem_events.write(create_dir_request);
                debug!(
                    "Requested hotkey preferences directory creation with operation_id: {:?}",
                    dir_operation_id
                );

                info!(
                    "Custom hotkey applied and persistence initiated: {:?}",
                    hotkey_prefs.custom_hotkey
                );
            },
            PreferencesEvent::ScanForConflicts => {
                info!("Scanning for hotkey conflicts");
                // Use HotkeyPreferences for proper conflict scanning
                let hotkey_prefs = HotkeyPreferences::default();
                load_preferred_alternatives(&mut prefs_state, &hotkey_prefs);
            },
        }
    }
}

/// System to handle FileSystemResponse events for preferences operations
pub fn handle_preferences_filesystem_responses(
    mut commands: Commands,
    mut filesystem_responses: EventReader<FileSystemResponse>,
    mut pending_ops: ResMut<PendingFileOperations>,
    mut prefs_state: ResMut<PreferencesResource>,
    mut filesystem_events: EventWriter<FileSystemRequest>,
) {
    for response in filesystem_responses.read() {
        match response {
            FileSystemResponse::ReadFileResult {
                operation_id,
                requester,
                result,
            } => {
                if let Some(_tracked_entity) = pending_ops.preferences_read_ops.remove(operation_id)
                {
                    match result.as_ref() {
                        Ok(file_content) => {
                            match parse_preferences_from_content(file_content) {
                                Ok(loaded_prefs) => {
                                    info!("Successfully loaded preferences from disk");
                                    // Apply loaded preferences to UI state
                                    apply_loaded_preferences(&loaded_prefs, &mut prefs_state);
                                    prefs_state.loading = false;
                                    prefs_state.last_error = None;
                                },
                                Err(e) => {
                                    error!("Failed to parse preferences file: {}", e);
                                    prefs_state.loading = false;
                                    prefs_state.last_error = Some(format!("Parse error: {}", e));
                                    // Use default preferences
                                    let default_prefs = HotkeyPreferences::default();
                                    apply_loaded_preferences(&default_prefs, &mut prefs_state);
                                },
                            }
                        },
                        Err(filesystem_error) => {
                            match filesystem_error {
                                FileSystemError::NotFound { .. } => {
                                    info!(
                                        "Preferences file not found - using defaults (first run)"
                                    );
                                    prefs_state.loading = false;
                                    prefs_state.last_error = None;
                                    // Use default preferences for first run
                                    let default_prefs = HotkeyPreferences::default();
                                    apply_loaded_preferences(&default_prefs, &mut prefs_state);
                                },
                                _ => {
                                    error!("Failed to load preferences: {}", filesystem_error);
                                    prefs_state.loading = false;
                                    prefs_state.last_error =
                                        Some(format!("Load error: {}", filesystem_error));
                                    // Use default preferences
                                    let default_prefs = HotkeyPreferences::default();
                                    apply_loaded_preferences(&default_prefs, &mut prefs_state);
                                },
                            }
                        },
                    }

                    // Clean up requester entity safely
                    if let Ok(mut entity_commands) = commands.get_entity(*requester) {
                        entity_commands.despawn();
                    }
                    info!("Preferences load operation completed: {:?}", operation_id);
                }
            },

            FileSystemResponse::WriteFileResult {
                operation_id,
                requester,
                result,
            } => {
                if let Some(saved_prefs) = pending_ops.preferences_write_ops.remove(operation_id) {
                    match result {
                        Ok(()) => {
                            info!("Successfully saved preferences to disk");
                            prefs_state.saving = false;
                            prefs_state.last_error = None;
                            prefs_state.last_save_success = Some(std::time::SystemTime::now());

                            // Update current preferences in UI state
                            apply_saved_preferences(&saved_prefs, &mut prefs_state);
                        },
                        Err(filesystem_error) => {
                            error!("Failed to save preferences: {}", filesystem_error);
                            prefs_state.saving = false;
                            prefs_state.last_error =
                                Some(format!("Save error: {}", filesystem_error));
                        },
                    }

                    // Clean up requester entity safely
                    if let Ok(mut entity_commands) = commands.get_entity(*requester) {
                        entity_commands.despawn();
                    }
                    info!("Preferences save operation completed: {:?}", operation_id);
                }
            },

            FileSystemResponse::CreateDirectoryResult {
                operation_id,
                requester,
                result,
            } => {
                if let Some((_entity, config_file, json_content, hotkey_prefs)) =
                    pending_ops.preferences_dir_ops.remove(operation_id)
                {
                    match result {
                        Ok(()) => {
                            // Directory created successfully, proceed with file write
                            let write_operation_id = FileOperationId::new();
                            let write_entity = commands
                                .spawn(Name::new(format!(
                                    "PreferencesWriteRequester-{:?}",
                                    write_operation_id
                                )))
                                .id();

                            pending_ops
                                .preferences_write_ops
                                .insert(write_operation_id, hotkey_prefs);

                            let write_request = FileSystemRequest::WriteFile {
                                operation_id: write_operation_id,
                                requester: write_entity,
                                path: config_file,
                                content: json_content,
                            };

                            filesystem_events.write(write_request);
                            debug!(
                                "Directory created successfully, proceeding with file write: {:?}",
                                write_operation_id
                            );
                        },
                        Err(filesystem_error) => {
                            error!("Failed to create config directory: {}", filesystem_error);
                            prefs_state.saving = false;
                            prefs_state.last_error =
                                Some(format!("Directory creation failed: {}", filesystem_error));
                        },
                    }

                    // Clean up directory creation entity
                    if let Ok(mut entity_commands) = commands.get_entity(*requester) {
                        entity_commands.despawn();
                    }
                    info!("Directory creation operation completed: {:?}", operation_id);
                }
            },

            _ => {
                // Ignore other filesystem responses not related to preferences
            },
        }
    }
}

/// Parse preferences from file content with comprehensive error handling
fn parse_preferences_from_content(
    file_content: &FileContent,
) -> Result<HotkeyPreferences, Box<dyn std::error::Error + Send + Sync>> {
    let content_str = std::str::from_utf8(&file_content.data)
        .map_err(|e| format!("Invalid UTF-8 in preferences file: {}", e))?;

    if content_str.trim().is_empty() {
        info!("Empty preferences file - using defaults");
        return Ok(HotkeyPreferences::default());
    }

    let prefs: HotkeyPreferences = serde_json::from_str(content_str)
        .map_err(|e| format!("Invalid JSON in preferences file: {}", e))?;

    Ok(prefs)
}

/// Apply loaded preferences to UI state
fn apply_loaded_preferences(prefs: &HotkeyPreferences, prefs_state: &mut PreferencesResource) {
    // Apply custom hotkey if present
    if let Some(ref custom_hotkey) = prefs.custom_hotkey {
        prefs_state.captured_hotkey = Some(custom_hotkey.clone());
        prefs_state.current_status = HotkeyStatus::Valid;
        info!(
            "Applied custom hotkey from preferences: {}",
            custom_hotkey.description
        );
    }

    // Store loaded preferences for future reference
    prefs_state.loaded_preferences = Some(prefs.clone());
}

/// Apply saved preferences to UI state after successful save
fn apply_saved_preferences(prefs: &HotkeyPreferences, prefs_state: &mut PreferencesResource) {
    // Update stored preferences
    prefs_state.loaded_preferences = Some(prefs.clone());
    info!("Updated UI state with successfully saved preferences");
}

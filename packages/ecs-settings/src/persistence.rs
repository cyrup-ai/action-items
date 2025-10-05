//! Settings persistence integration with database backend
//!
//! Connects the Settings UI to the SurrealDB persistence layer.
//! Handles loading settings on startup and applying them to UI components.

use bevy::prelude::*;
use action_items_ecs_user_settings::*;
use uuid::Uuid;
use crate::ui::components::*;

/// Resource to track persistence requester entity
#[derive(Resource)]
pub struct PersistenceRequester(pub Entity);

/// System to load settings from database on startup
/// 
/// Sends read requests for all settings tables. The actual application
/// of loaded settings happens in `apply_loaded_settings` when the
/// SettingsReadCompleted events are received.
pub fn load_settings_on_startup(
    mut commands: Commands,
    mut read_events: EventWriter<SettingsReadRequested>,
) {
    let requester = commands.spawn_empty().id();
    commands.insert_resource(PersistenceRequester(requester));

    info!("Loading settings from database on startup");

    // Load all settings tables
    for table in [
        "appearance_settings",
        "ai_settings", 
        "advanced_settings",
        "startup_settings",
        "cloud_sync_settings",
        "account_settings",
        "organization_settings",
    ] {
        read_events.write(SettingsReadRequested {
            operation_id: Uuid::new_v4(),
            table: table.to_string(),
            key: "main".to_string(),
            requester,
        });
    }
}

/// Apply loaded settings to UI components
/// 
/// Listens for SettingsReadCompleted events and updates the corresponding
/// UI components (checkboxes, text inputs, dropdowns) with loaded values.
/// 
/// This system is resilient to:
/// - Missing UI components (tabs not yet rendered)
/// - Database errors (logs warning, continues)
/// - Missing or invalid field values (skips gracefully)
pub fn apply_loaded_settings(
    mut events: EventReader<SettingsReadCompleted>,
    mut checkboxes: Query<(&SettingControl, &mut SettingCheckbox)>,
    mut text_inputs: Query<(&SettingControl, &mut TextInput), Without<SettingCheckbox>>,
    mut dropdowns: Query<(&SettingControl, &mut DropdownControl), (Without<SettingCheckbox>, Without<TextInput>)>,
) {
    for event in events.read() {
        // Handle database errors
        let value = match &event.result {
            Ok(Some(v)) => v,
            Ok(None) => {
                debug!("No settings found in table '{}' - using defaults", event.table);
                continue;
            }
            Err(e) => {
                warn!("Failed to load settings from '{}': {}", event.table, e);
                continue;
            }
        };

        // Convert surrealdb::Value to JSON-like structure for field extraction
        // SurrealDB Value is compatible with serde_json conversion
        let json_value: serde_json::Value = match serde_json::to_value(value) {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to parse settings from '{}': {}", event.table, e);
                continue;
            }
        };

        // Extract object fields
        let Some(obj) = json_value.as_object() else {
            warn!("Settings from '{}' is not an object: {:?}", event.table, json_value);
            continue;
        };

        // Apply to checkboxes
        for (control, mut checkbox) in checkboxes.iter_mut() {
            if control.table == event.table {
                if let Some(field_value) = obj.get(&control.field_name) {
                    if let Some(bool_val) = field_value.as_bool() {
                        checkbox.checked = bool_val;
                        debug!("Loaded checkbox {}.{} = {}", control.table, control.field_name, bool_val);
                    }
                }
            }
        }

        // Apply to text inputs
        for (control, mut text_input) in text_inputs.iter_mut() {
            if control.table == event.table {
                if let Some(field_value) = obj.get(&control.field_name) {
                    if let Some(str_val) = field_value.as_str() {
                        text_input.value = str_val.to_string();
                        debug!("Loaded text input {}.{} = {}", control.table, control.field_name, str_val);
                    }
                }
            }
        }

        // Apply to dropdowns
        for (control, mut dropdown) in dropdowns.iter_mut() {
            if control.table == event.table {
                if let Some(field_value) = obj.get(&control.field_name) {
                    // Handle both string values (option name) and numeric values (index)
                    if let Some(str_val) = field_value.as_str() {
                        // Find index of option matching the string value
                        if let Some(index) = dropdown.options.iter().position(|opt| opt == str_val) {
                            dropdown.selected = index;
                            debug!("Loaded dropdown {}.{} = {} (index {})", 
                                control.table, control.field_name, str_val, index);
                        }
                    } else if let Some(num_val) = field_value.as_u64() {
                        let index = num_val as usize;
                        if index < dropdown.options.len() {
                            dropdown.selected = index;
                            debug!("Loaded dropdown {}.{} = index {}", 
                                control.table, control.field_name, index);
                        }
                    }
                }
            }
        }

        info!("Applied settings from table '{}'", event.table);
    }
}

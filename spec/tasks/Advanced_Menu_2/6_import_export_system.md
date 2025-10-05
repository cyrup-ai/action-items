# Task 6: Import/Export System

## Implementation Details

**File**: `ui/src/ui/import_export.rs`  
**Lines**: 185-295  
**Architecture**: Comprehensive data backup and migration system with scheduled automation  
**Integration**: SettingsSystem, SchedulingSystem, StorageManager  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct ImportExportManager {
    pub export_scheduler: ExportScheduler,
    pub data_validator: DataValidator,
    pub encryption_settings: EncryptionSettings,
    pub export_history: VecDeque<ExportRecord>,
    pub supported_formats: Vec<ExportFormat>,
    pub active_operations: HashMap<String, OperationStatus>,
}

#[derive(Clone, Debug)]
pub struct ExportData {
    pub version: String,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub settings: ApplicationSettings,
    pub quicklinks: Vec<Quicklink>,
    pub snippets: Vec<Snippet>,
    pub notes: Vec<Note>,
    pub script_commands: Vec<ScriptCommand>,
    pub aliases: HashMap<String, String>,
    pub hotkeys: HashMap<String, HotkeyBinding>,
    pub favorites: Vec<FavoriteItem>,
    pub window_commands: Vec<WindowCommand>,
    pub metadata: ExportMetadata,
}

#[derive(Clone, Debug)]
pub struct ExportScheduler {
    pub enabled: bool,
    pub schedule_type: ScheduleType,
    pub interval: Duration,
    pub next_export: Option<chrono::DateTime<chrono::Utc>>,
    pub export_location: ExportLocation,
    pub retention_policy: RetentionPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScheduleType {
    Daily,
    Weekly,
    Monthly,
    Custom(Duration),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExportLocation {
    LocalDirectory(PathBuf),
    CloudStorage(CloudProvider),
    CustomUrl(String),
}

pub fn import_export_system(
    mut import_export_manager: ResMut<ImportExportManager>,
    mut export_events: EventReader<ExportRequestEvent>,
    mut import_events: EventReader<ImportRequestEvent>,
    mut operation_events: EventWriter<OperationStatusEvent>,
    mut file_events: EventWriter<FileOperationEvent>,
    time: Res<Time>,
) {
    // Handle scheduled exports
    if import_export_manager.export_scheduler.enabled {
        if let Some(next_export) = import_export_manager.export_scheduler.next_export {
            if chrono::Utc::now() >= next_export {
                export_events.send(ExportRequestEvent::ScheduledExport {
                    schedule_id: "automatic".to_string(),
                    export_type: ExportType::Complete,
                });
                
                // Schedule next export
                let next_time = calculate_next_export_time(
                    &import_export_manager.export_scheduler
                );
                import_export_manager.export_scheduler.next_export = Some(next_time);
            }
        }
    }

    // Handle export requests
    for export_request in export_events.read() {
        let operation_id = generate_operation_id();
        
        operation_events.send(OperationStatusEvent::Started {
            operation_id: operation_id.clone(),
            operation_type: OperationType::Export,
            estimated_duration: estimate_export_duration(&export_request),
        });

        match export_request {
            ExportRequestEvent::ManualExport { export_type, destination } => {
                start_export_operation(
                    &operation_id,
                    export_type,
                    destination,
                    &mut import_export_manager,
                    &file_events,
                );
            }
            ExportRequestEvent::ScheduledExport { schedule_id, export_type } => {
                let destination = import_export_manager.export_scheduler.export_location.clone();
                start_export_operation(
                    &operation_id,
                    export_type,
                    &destination,
                    &mut import_export_manager,
                    &file_events,
                );
            }
        }
    }

    // Handle import requests
    for import_request in import_events.read() {
        let operation_id = generate_operation_id();
        
        operation_events.send(OperationStatusEvent::Started {
            operation_id: operation_id.clone(),
            operation_type: OperationType::Import,
            estimated_duration: estimate_import_duration(&import_request),
        });

        match import_request {
            ImportRequestEvent::ImportFile { file_path, validation_mode } => {
                start_import_operation(
                    &operation_id,
                    file_path,
                    validation_mode,
                    &mut import_export_manager,
                    &file_events,
                );
            }
        }
    }
}

async fn start_export_operation(
    operation_id: &str,
    export_type: &ExportType,
    destination: &ExportLocation,
    manager: &mut ImportExportManager,
    file_events: &EventWriter<FileOperationEvent>,
) -> Result<(), ExportError> {
    // Collect data based on export type
    let export_data = match export_type {
        ExportType::Complete => collect_complete_export_data().await?,
        ExportType::SettingsOnly => collect_settings_export_data().await?,
        ExportType::Custom(categories) => collect_custom_export_data(categories).await?,
    };

    // Validate data integrity
    let validation_result = manager.data_validator.validate_export_data(&export_data);
    if let Err(validation_error) = validation_result {
        return Err(ExportError::ValidationFailed(validation_error));
    }

    // Apply encryption if enabled
    let processed_data = if manager.encryption_settings.enabled {
        encrypt_export_data(&export_data, &manager.encryption_settings)?
    } else {
        serialize_export_data(&export_data)?
    };

    // Write to destination
    match destination {
        ExportLocation::LocalDirectory(path) => {
            let filename = generate_export_filename(&export_data.exported_at, export_type);
            let full_path = path.join(filename);
            
            std::fs::write(&full_path, processed_data)
                .map_err(|e| ExportError::FileWriteError(e.to_string()))?;
        }
        ExportLocation::CloudStorage(provider) => {
            upload_to_cloud_storage(&processed_data, provider, export_type).await?;
        }
        ExportLocation::CustomUrl(url) => {
            upload_to_custom_endpoint(&processed_data, url, export_type).await?;
        }
    }

    // Record successful export
    manager.export_history.push_back(ExportRecord {
        operation_id: operation_id.to_string(),
        exported_at: chrono::Utc::now(),
        export_type: export_type.clone(),
        destination: destination.clone(),
        data_size: processed_data.len(),
        success: true,
        error_message: None,
    });

    Ok(())
}
```

### Data Collection and Validation

**Reference**: `./docs/bevy/examples/file_io/file_io.rs:285-318`

```rust
#[derive(Clone, Debug)]
pub struct DataValidator {
    pub validation_rules: Vec<ValidationRule>,
    pub strict_mode: bool,
    pub schema_version: String,
}

impl DataValidator {
    pub fn validate_export_data(&self, data: &ExportData) -> Result<(), ValidationError> {
        // Validate data schema version compatibility
        if !is_schema_compatible(&data.version, &self.schema_version) {
            return Err(ValidationError::IncompatibleSchema {
                data_version: data.version.clone(),
                expected_version: self.schema_version.clone(),
            });
        }

        // Validate required fields
        self.validate_required_fields(data)?;
        
        // Validate data integrity
        self.validate_data_integrity(data)?;
        
        // Validate foreign key relationships
        self.validate_relationships(data)?;
        
        // Custom validation rules
        for rule in &self.validation_rules {
            rule.validate(data)?;
        }

        Ok(())
    }

    fn validate_required_fields(&self, data: &ExportData) -> Result<(), ValidationError> {
        if data.settings.is_empty() {
            return Err(ValidationError::MissingRequiredField("settings".to_string()));
        }
        
        if data.metadata.user_id.is_empty() {
            return Err(ValidationError::MissingRequiredField("user_id".to_string()));
        }

        Ok(())
    }

    fn validate_data_integrity(&self, data: &ExportData) -> Result<(), ValidationError> {
        // Validate hotkey uniqueness
        let mut hotkey_conflicts = Vec::new();
        for (action1, binding1) in &data.hotkeys {
            for (action2, binding2) in &data.hotkeys {
                if action1 != action2 && binding1.key_combination == binding2.key_combination {
                    hotkey_conflicts.push((action1.clone(), action2.clone()));
                }
            }
        }
        
        if !hotkey_conflicts.is_empty() {
            return Err(ValidationError::DataIntegrityIssue(
                format!("Hotkey conflicts detected: {:?}", hotkey_conflicts)
            ));
        }

        // Validate alias uniqueness
        let mut alias_names: std::collections::HashSet<_> = std::collections::HashSet::new();
        for alias_name in data.aliases.keys() {
            if !alias_names.insert(alias_name) {
                return Err(ValidationError::DataIntegrityIssue(
                    format!("Duplicate alias name: {}", alias_name)
                ));
            }
        }

        Ok(())
    }
}

async fn collect_complete_export_data() -> Result<ExportData, ExportError> {
    let settings = collect_application_settings().await?;
    let quicklinks = collect_quicklinks().await?;
    let snippets = collect_snippets().await?;
    let notes = collect_notes().await?;
    let script_commands = collect_script_commands().await?;
    let aliases = collect_aliases().await?;
    let hotkeys = collect_hotkeys().await?;
    let favorites = collect_favorites().await?;
    let window_commands = collect_window_commands().await?;

    let metadata = ExportMetadata {
        user_id: get_current_user_id(),
        device_id: get_device_id(),
        app_version: get_app_version(),
        export_scope: ExportScope::Complete,
        data_categories: vec![
            "settings", "quicklinks", "snippets", "notes", 
            "commands", "aliases", "hotkeys", "favorites", "windows"
        ].iter().map(|s| s.to_string()).collect(),
    };

    Ok(ExportData {
        version: EXPORT_SCHEMA_VERSION.to_string(),
        exported_at: chrono::Utc::now(),
        settings,
        quicklinks,
        snippets,
        notes,
        script_commands,
        aliases,
        hotkeys,
        favorites,
        window_commands,
        metadata,
    })
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui/ui_buttons.rs:325-368`

```rust
// Import/Export section
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(16.0)),
        row_gap: Val::Px(16.0),
        ..default()
    },
    background_color: Color::rgba(0.08, 0.08, 0.08, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(8.0)),
    ..default()
},
children: &[
    // Section header with description
    (TextBundle::from_section(
        "Import / Export",
        TextStyle {
            font: asset_server.load("fonts/Inter-SemiBold.ttf"),
            font_size: 16.0,
            color: Color::rgb(0.95, 0.95, 0.95),
        },
    ),),
    (TextBundle::from_section(
        "Exporting will back-up your settings, quicklinks, snippets, notes, script-command folder paths, aliases, hotkeys, favorites, custom window management commands and other data.",
        TextStyle {
            font: asset_server.load("fonts/Inter-Regular.ttf"),
            font_size: 12.0,
            color: Color::rgb(0.7, 0.7, 0.7),
        },
    ),),
    
    // Import/Export buttons row
    (NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            ..default()
        },
        ..default()
    },
    children: &[
        // Import button
        (ButtonBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::rgba(0.15, 0.15, 0.15, 1.0).into(),
            border_color: Color::rgba(0.4, 0.4, 0.4, 1.0).into(),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        children: &[
            (IconBundle {
                icon: Icon::Download,
                color: Color::rgb(0.8, 0.8, 0.8),
                size: 16.0,
                ..default()
            },),
            (TextBundle::from_section(
                "Import",
                TextStyle {
                    font: asset_server.load("fonts/Inter-Medium.ttf"),
                    font_size: 12.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ),),
        ]),
        
        // Export button
        (ButtonBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::rgba(0.15, 0.15, 0.15, 1.0).into(),
            border_color: Color::rgba(0.4, 0.4, 0.4, 1.0).into(),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        children: &[
            (IconBundle {
                icon: Icon::Upload,
                color: Color::rgb(0.8, 0.8, 0.8),
                size: 16.0,
                ..default()
            },),
            (TextBundle::from_section(
                "Export",
                TextStyle {
                    font: asset_server.load("fonts/Inter-Medium.ttf"),
                    font_size: 12.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ),),
        ]),
    ]),
    
    // Configure Export Schedule button
    (ButtonBundle {
        style: Style {
            width: Val::Px(200.0),
            height: Val::Px(32.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        },
        background_color: Color::rgb(0.2, 0.5, 0.8).into(),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    },
    children: &[
        (TextBundle::from_section(
            "Configure Export Schedule",
            TextStyle {
                font: asset_server.load("fonts/Inter-Medium.ttf"),
                font_size: 12.0,
                color: Color::WHITE,
            },
        ),),
    ]),
]
```

### Architecture Notes

- Comprehensive data export covering all application state and user data
- Versioned export format ensures forward and backward compatibility
- Robust data validation prevents corruption and integrity issues
- Scheduled export automation with configurable intervals and destinations
- Multiple export destinations: local files, cloud storage, custom endpoints
- Optional encryption for sensitive data protection during export/import
- Progress tracking and error handling for long-running operations
- Retention policies for automated cleanup of old exports

**Bevy Examples**: `./docs/bevy/examples/file_io/file_io.rs:385-422`, `./docs/bevy/examples/async_tasks/async_compute.rs:125-162`  
**Integration Points**: SettingsSystem, SchedulingSystem, FileManager  
**Dependencies**: FileSystem, NetworkClient, EncryptionManager
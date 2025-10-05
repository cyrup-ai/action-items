# Task 0: Extended Window Capture System with Automation

## Implementation Details

**File**: `ui/src/ui/capture_automation.rs`  
**Lines**: 125-245  
**Architecture**: Advanced window capture with automated workflows and clipboard integration  
**Integration**: WindowCaptureSystem, ClipboardManager, FileManager  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct WindowCaptureExtended {
    pub basic_capture: WindowCaptureManager,
    pub automation_settings: AutomationSettings,
    pub clipboard_integration: ClipboardIntegration,
    pub file_management: CaptureFileManager,
    pub batch_operations: BatchCaptureManager,
}

#[derive(Clone, Debug)]
pub struct AutomationSettings {
    pub copy_to_clipboard: bool,
    pub show_in_finder: bool,
    pub auto_share_enabled: bool,
    pub default_share_destinations: Vec<ShareDestination>,
    pub post_capture_actions: Vec<PostCaptureAction>,
    pub notification_settings: NotificationSettings,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PostCaptureAction {
    CopyToClipboard,
    ShowInFinder,
    ShareTo(ShareDestination),
    RunScript(String),
    OpenInEditor(String),
    UploadToService(String),
}

#[derive(Clone, Debug)]
pub struct ClipboardIntegration {
    pub clipboard_format: ClipboardFormat,
    pub quality_settings: ClipboardQuality,
    pub metadata_inclusion: bool,
    pub history_enabled: bool,
    pub clipboard_history: VecDeque<ClipboardEntry>,
}

pub fn window_capture_extended_system(
    mut capture_extended: ResMut<WindowCaptureExtended>,
    mut capture_events: EventReader<CaptureCompletedEvent>,
    mut clipboard_events: EventWriter<ClipboardEvent>,
    mut file_events: EventWriter<FileOperationEvent>,
    mut notification_events: EventWriter<NotificationEvent>,
    mut share_events: EventWriter<ShareEvent>,
) {
    // Process completed captures for automation
    for capture_event in capture_events.read() {
        let automation_result = process_capture_automation(
            &capture_event,
            &capture_extended.automation_settings,
            &mut capture_extended.clipboard_integration,
            &capture_extended.file_management,
        );

        match automation_result {
            Ok(actions_performed) => {
                // Execute post-capture actions
                for action in actions_performed {
                    match action {
                        PostCaptureAction::CopyToClipboard => {
                            execute_clipboard_copy(
                                &capture_event.capture_data,
                                &mut clipboard_events,
                                &capture_extended.clipboard_integration,
                            );
                        }
                        PostCaptureAction::ShowInFinder => {
                            if let Some(ref file_path) = capture_event.file_path {
                                file_events.send(FileOperationEvent::RevealInFinder {
                                    path: file_path.clone(),
                                });
                            }
                        }
                        PostCaptureAction::ShareTo(destination) => {
                            share_events.send(ShareEvent::ShareCapture {
                                capture_id: capture_event.capture_id.clone(),
                                destination,
                                metadata: capture_event.metadata.clone(),
                            });
                        }
                        PostCaptureAction::RunScript(script_path) => {
                            file_events.send(FileOperationEvent::ExecuteScript {
                                script: script_path,
                                args: vec![
                                    capture_event.file_path.as_ref()
                                        .map(|p| p.to_string_lossy().to_string())
                                        .unwrap_or_default()
                                ],
                            });
                        }
                        _ => {}
                    }
                }
                
                // Send success notification
                if capture_extended.automation_settings.notification_settings.success_enabled {
                    notification_events.send(NotificationEvent::CaptureAutomationSuccess {
                        actions_count: actions_performed.len(),
                        capture_id: capture_event.capture_id.clone(),
                    });
                }
            }
            Err(error) => {
                notification_events.send(NotificationEvent::CaptureAutomationError {
                    error: error.to_string(),
                    capture_id: capture_event.capture_id.clone(),
                });
            }
        }
    }

    // Process clipboard history cleanup
    cleanup_clipboard_history(&mut capture_extended.clipboard_integration);
}

fn execute_clipboard_copy(
    capture_data: &CaptureData,
    clipboard_events: &mut EventWriter<ClipboardEvent>,
    clipboard_integration: &ClipboardIntegration,
) {
    match clipboard_integration.clipboard_format {
        ClipboardFormat::Image => {
            // Copy raw image data to clipboard
            clipboard_events.send(ClipboardEvent::CopyImage {
                image_data: capture_data.image_data.clone(),
                format: capture_data.format,
            });
        }
        ClipboardFormat::FilePath => {
            // Copy file path to clipboard
            if let Some(ref file_path) = capture_data.file_path {
                clipboard_events.send(ClipboardEvent::CopyText {
                    text: file_path.to_string_lossy().to_string(),
                });
            }
        }
        ClipboardFormat::Both => {
            // Copy both image and file path
            clipboard_events.send(ClipboardEvent::CopyImage {
                image_data: capture_data.image_data.clone(),
                format: capture_data.format,
            });
            if let Some(ref file_path) = capture_data.file_path {
                clipboard_events.send(ClipboardEvent::CopyText {
                    text: file_path.to_string_lossy().to_string(),
                });
            }
        }
    }

    // Add to clipboard history
    if clipboard_integration.history_enabled {
        let history_entry = ClipboardEntry {
            id: generate_entry_id(),
            timestamp: chrono::Utc::now(),
            capture_id: capture_data.id.clone(),
            format: clipboard_integration.clipboard_format,
            preview_data: create_preview_data(&capture_data.image_data),
        };
        
        // Would add to history (mutable reference needed)
    }
}
```

### Batch Capture System

**Reference**: `./docs/bevy/examples/async_tasks/async_compute.rs:285-322`

```rust
#[derive(Clone, Debug)]
pub struct BatchCaptureManager {
    pub batch_settings: BatchSettings,
    pub active_batches: HashMap<String, BatchOperation>,
    pub batch_history: VecDeque<BatchRecord>,
    pub scheduling_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct BatchSettings {
    pub interval_ms: u64,
    pub max_captures: usize,
    pub capture_targets: Vec<CaptureTarget>,
    pub output_format: BatchOutputFormat,
    pub naming_convention: NamingConvention,
}

#[derive(Clone, Debug)]
pub enum BatchOutputFormat {
    Individual,
    Collage(CollageSettings),
    Animation(AnimationSettings),
    Archive(ArchiveSettings),
}

pub fn batch_capture_system(
    mut batch_manager: ResMut<BatchCaptureManager>,
    mut batch_events: EventReader<BatchCaptureEvent>,
    mut capture_events: EventWriter<CaptureRequestEvent>,
    mut file_events: EventWriter<FileOperationEvent>,
    time: Res<Time>,
) {
    // Process batch capture requests
    for batch_event in batch_events.read() {
        match batch_event {
            BatchCaptureEvent::StartBatch { settings, targets } => {
                let batch_id = generate_batch_id();
                let operation = BatchOperation {
                    id: batch_id.clone(),
                    settings: settings.clone(),
                    targets: targets.clone(),
                    status: BatchStatus::Running,
                    started_at: std::time::Instant::now(),
                    captures_completed: 0,
                    captures_total: targets.len(),
                };
                
                batch_manager.active_batches.insert(batch_id.clone(), operation);
                
                // Schedule individual captures
                for (index, target) in targets.iter().enumerate() {
                    let delay = std::time::Duration::from_millis(
                        index as u64 * batch_manager.batch_settings.interval_ms
                    );
                    
                    // Would schedule delayed capture request
                    capture_events.send(CaptureRequestEvent::ScheduledCapture {
                        target: *target,
                        batch_id: Some(batch_id.clone()),
                        delay,
                    });
                }
            }
            BatchCaptureEvent::CancelBatch { batch_id } => {
                if let Some(mut operation) = batch_manager.active_batches.remove(batch_id) {
                    operation.status = BatchStatus::Cancelled;
                    batch_manager.batch_history.push_back(BatchRecord::from(operation));
                }
            }
        }
    }

    // Update active batch operations
    update_batch_operations(&mut batch_manager, &file_events);
}

fn create_collage_from_captures(
    captures: &[CaptureData],
    settings: &CollageSettings,
) -> Result<Vec<u8>, CollageError> {
    let mut collage_builder = CollageBuilder::new(settings.layout);
    
    for capture in captures {
        let image = load_image_from_data(&capture.image_data)?;
        collage_builder.add_image(image, capture.metadata.clone())?;
    }
    
    let collage = collage_builder.build()?;
    encode_image_as_png(&collage)
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui/ui_checkbox.rs:325-368`

```rust
// Extended window capture settings
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(16.0)),
        row_gap: Val::Px(12.0),
        ..default()
    },
    background_color: Color::rgba(0.08, 0.08, 0.08, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(8.0)),
    ..default()
},
children: &[
    // Section header
    (TextBundle::from_section(
        "Window Capture",
        TextStyle {
            font: asset_server.load("fonts/Inter-SemiBold.ttf"),
            font_size: 16.0,
            color: Color::rgb(0.95, 0.95, 0.95),
        },
    ),),
    (TextBundle::from_section(
        "Capture the Raycast window to share it or add a screenshot of your extension to the Store.",
        TextStyle {
            font: asset_server.load("fonts/Inter-Regular.ttf"),
            font_size: 12.0,
            color: Color::rgb(0.7, 0.7, 0.7),
        },
    ),),
    
    // Record Hotkey button
    (ButtonBundle {
        style: Style {
            width: Val::Px(150.0),
            height: Val::Px(36.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(12.0)),
            ..default()
        },
        background_color: Color::rgb(0.2, 0.5, 0.8).into(),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    },
    children: &[
        (TextBundle::from_section(
            "Record Hotkey",
            TextStyle {
                font: asset_server.load("fonts/Inter-Medium.ttf"),
                font_size: 12.0,
                color: Color::WHITE,
            },
        ),),
    ]),
    
    // Automation options
    (SettingsRowBundle {
        label: "Copy to Clipboard".to_string(),
        control: ControlType::Checkbox {
            checked: capture_extended.automation_settings.copy_to_clipboard,
        },
        tooltip: Some("Automatically copy screenshots to clipboard after capture".to_string()),
        ..default()
    },),
    
    (SettingsRowBundle {
        label: "Show in Finder".to_string(),
        control: ControlType::Checkbox {
            checked: capture_extended.automation_settings.show_in_finder,
        },
        tooltip: Some("Automatically reveal screenshots in Finder after capture".to_string()),
        ..default()
    },),
    
    // Advanced automation settings (collapsible)
    (ExpansionPanelBundle {
        header: "Advanced Automation".to_string(),
        expanded: false,
        content: NodeBundle {
            children: &[
                // Batch capture settings
                (SettingsRowBundle {
                    label: "Batch Capture Interval (ms)".to_string(),
                    control: ControlType::Slider {
                        value: capture_extended.batch_operations.batch_settings.interval_ms as f32,
                        min: 100.0,
                        max: 5000.0,
                        step: 100.0,
                    },
                    ..default()
                },),
                
                // Post-capture actions
                (MultiSelectBundle {
                    label: "Post-Capture Actions".to_string(),
                    options: vec![
                        "Copy to Clipboard".to_string(),
                        "Show in Finder".to_string(),
                        "Upload to Service".to_string(),
                        "Run Custom Script".to_string(),
                        "Open in Editor".to_string(),
                    ],
                    selected: capture_extended.automation_settings.post_capture_actions.iter()
                        .map(|action| action.display_name())
                        .collect(),
                    ..default()
                },),
            ],
            ..default()
        },
        ..default()
    },),
]
```

### Architecture Notes

- Extended automation system with configurable post-capture workflows
- Advanced clipboard integration supporting multiple formats and history
- Batch capture functionality for repetitive screenshot workflows
- Enterprise-grade file management with automatic organization
- Collage and animation creation from batch captures
- Real-time progress tracking for complex capture operations
- Comprehensive error handling with user-friendly notifications
- Integration with external scripts and tools for custom workflows

**Bevy Examples**: `./docs/bevy/examples/window/screenshot.rs:385-422`, `./docs/bevy/examples/async_tasks/async_compute.rs:158-195`  
**Integration Points**: WindowCaptureSystem, ClipboardManager, FileManager, ShareSystem  
**Dependencies**: SystemClipboard, FileSystem, NotificationManager, ShareAPI
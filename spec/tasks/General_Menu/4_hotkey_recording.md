# General Menu - Hotkey Recording Interface

## Task: Implement Interactive Hotkey Recording System

### File: `ui/src/settings/general/hotkey_recorder.rs` (new file)

Create a comprehensive hotkey recording interface with global key capture, conflict detection, and visual feedback systems.

### Implementation Requirements

#### Hotkey Recording Component
```rust
#[derive(Component)]
pub struct HotkeyRecorder {
    pub is_recording: bool,
    pub current_combination: Vec<KeyCode>,
    pub display_text: String,
    pub conflict_status: ConflictStatus,
    pub recording_timeout: Timer,
}
```

#### Recording State Management
- File: `ui/src/settings/general/hotkey_recorder.rs` (line 1-89)
- Implement recording activation/deactivation system
- Keyboard event capture during recording mode
- Modifier key combination tracking (Cmd, Ctrl, Alt, Shift)
- Real-time visual feedback for captured keys

#### Global Hotkey Integration
- File: `ui/src/settings/general/global_hotkey.rs` (new file, line 1-134)
- Integration with `global-hotkey` crate for system-wide registration
- Conflict detection with existing system shortcuts
- Hotkey validation for common conflict patterns
- Registration/unregistration lifecycle management

#### Recording UI Components
- File: `ui/src/settings/general/hotkey_ui.rs` (new file, line 1-67)
- Interactive recording button with "⌘ Space" display format
- Modal overlay for recording state indication
- Visual conflict warnings with error styling
- Recording timeout indication and auto-cancel

#### Conflict Resolution System
- File: `ui/src/settings/general/conflict_detection.rs` (new file, line 1-156)
- System shortcut enumeration and checking
- Application-level conflict detection
- User notification system for conflicts
- Alternative suggestion system

### Architecture Notes
- Use Bevy's `Input<KeyCode>` for key capture
- Implement `Changed<HotkeyRecorder>` queries for reactive UI updates
- Global hotkey registration through system FFI integration
- Integration with `core/src/hotkey_management.rs` if exists

### Integration Points
- System hotkey APIs (macOS Global Hotkeys)
- Existing hotkey systems in `app/src/` (around line 1200-1300 if present)
- Settings persistence system
- Error notification system

### Security Considerations
- Validate hotkey combinations before system registration
- Prevent registration of critical system shortcuts
- Safe fallback for hotkey registration failures

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Bevy ECS Implementation Details

### System Architecture
```rust
// System set for hotkey recording
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum HotkeySystemSet {
    InputCapture,      // Capture keyboard input
    ConflictCheck,     // Check for conflicts
    UIUpdate,          // Update UI display
    Registration,      // Register with OS
}

app.configure_sets(
    Update,
    (
        HotkeySystemSet::InputCapture,
        HotkeySystemSet::ConflictCheck,
        HotkeySystemSet::UIUpdate,
        HotkeySystemSet::Registration,
    ).chain()
);
```

### Input Capture System
```rust
fn hotkey_capture_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut recorders: Query<&mut HotkeyRecorder, With<IsRecording>>,
    time: Res<Time>,
) {
    for mut recorder in recorders.iter_mut() {
        // Update timeout
        recorder.recording_timeout.tick(time.delta());
        if recorder.recording_timeout.finished() {
            recorder.is_recording = false;
            continue;
        }
        
        // Capture modifier keys
        let mut modifiers = Vec::new();
        if keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight) {
            modifiers.push(KeyCode::SuperLeft); // Cmd on macOS
        }
        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            modifiers.push(KeyCode::ControlLeft);
        }
        if keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight) {
            modifiers.push(KeyCode::AltLeft);
        }
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            modifiers.push(KeyCode::ShiftLeft);
        }
        
        // Capture non-modifier key
        for key in keyboard.get_just_pressed() {
            if !is_modifier(*key) {
                recorder.current_combination = modifiers;
                recorder.current_combination.push(*key);
                recorder.is_recording = false;
                break;
            }
        }
        
        // Update display text
        recorder.display_text = format_hotkey(&recorder.current_combination);
    }
}
```

### UI Update System
```rust
fn hotkey_ui_update_system(
    recorders: Query<&HotkeyRecorder, Changed<HotkeyRecorder>>,
    mut texts: Query<&mut Text, With<HotkeyDisplayText>>,
    mut buttons: Query<&mut BackgroundColor, With<RecordButton>>,
) {
    for recorder in recorders.iter() {
        // Update display text
        for mut text in texts.iter_mut() {
            text.0 = if recorder.is_recording {
                "Recording...".to_string()
            } else {
                recorder.display_text.clone()
            };
        }
        
        // Update button state
        for mut bg in buttons.iter_mut() {
            *bg = if recorder.is_recording {
                BackgroundColor(Color::srgb(0.8, 0.2, 0.2)) // Red when recording
            } else {
                BackgroundColor(Color::srgb(0.2, 0.2, 0.8)) // Blue normally
            };
        }
    }
}
```

### Event-Driven Recording
```rust
#[derive(Event)]
pub enum HotkeyEvent {
    StartRecording { target: Entity },
    StopRecording { target: Entity },
    HotkeyRecorded { combination: Vec<KeyCode> },
    ConflictDetected { existing: String },
}

fn handle_hotkey_events(
    mut events: EventReader<HotkeyEvent>,
    mut recorders: Query<&mut HotkeyRecorder>,
    mut commands: Commands,
) {
    for event in events.read() {
        match event {
            HotkeyEvent::StartRecording { target } => {
                if let Ok(mut recorder) = recorders.get_mut(*target) {
                    recorder.is_recording = true;
                    recorder.recording_timeout = Timer::from_seconds(5.0, TimerMode::Once);
                    commands.entity(*target).insert(IsRecording);
                }
            }
            HotkeyEvent::StopRecording { target } => {
                if let Ok(mut recorder) = recorders.get_mut(*target) {
                    recorder.is_recording = false;
                    commands.entity(*target).remove::<IsRecording>();
                }
            }
            _ => {}
        }
    }
}
```

### Async Global Hotkey Registration
```rust
fn register_global_hotkey(
    mut commands: Commands,
    recorders: Query<&HotkeyRecorder, Changed<HotkeyRecorder>>,
    task_pool: Res<AsyncComputeTaskPool>,
) {
    for recorder in recorders.iter() {
        if !recorder.current_combination.is_empty() && !recorder.is_recording {
            let combo = recorder.current_combination.clone();
            
            let task = task_pool.spawn(async move {
                // Register with OS (platform-specific)
                let result = register_hotkey_with_os(combo).await;
                
                let mut command_queue = CommandQueue::default();
                command_queue.push(move |world: &mut World| {
                    match result {
                        Ok(_) => {
                            world.send_event(HotkeyEvent::HotkeyRecorded { 
                                combination: combo 
                            });
                        }
                        Err(conflict) => {
                            world.send_event(HotkeyEvent::ConflictDetected {
                                existing: conflict
                            });
                        }
                    }
                });
                command_queue
            });
            
            commands.spawn(HotkeyRegistrationTask(task));
        }
    }
}
```

### Conflict Detection Resource
```rust
#[derive(Resource)]
pub struct HotkeyRegistry {
    pub system_hotkeys: HashSet<Vec<KeyCode>>,
    pub app_hotkeys: HashMap<String, Vec<KeyCode>>,
}

fn check_hotkey_conflicts(
    recorders: Query<&HotkeyRecorder, Changed<HotkeyRecorder>>,
    registry: Res<HotkeyRegistry>,
    mut events: EventWriter<HotkeyEvent>,
) {
    for recorder in recorders.iter() {
        if registry.system_hotkeys.contains(&recorder.current_combination) {
            events.send(HotkeyEvent::ConflictDetected {
                existing: "System hotkey".to_string(),
            });
        }
        
        for (name, combo) in &registry.app_hotkeys {
            if *combo == recorder.current_combination {
                events.send(HotkeyEvent::ConflictDetected {
                    existing: name.clone(),
                });
            }
        }
    }
}
```
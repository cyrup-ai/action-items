# Actions_Items_Config_Menu Task 6: Hotkey Assignment System

## Task Overview
Implement comprehensive command hotkey recording and management system with interactive hotkey capture, conflict detection, global hotkey integration, and persistent storage.

## Implementation Requirements

### Core Components
```rust
// Hotkey assignment system
#[derive(Resource, Reflect, Debug)]
pub struct HotkeyAssignmentResource {
    pub hotkey_mappings: HashMap<CommandId, Hotkey>,
    pub global_hotkeys: HashMap<Hotkey, CommandId>,
    pub hotkey_conflicts: Vec<HotkeyConflict>,
    pub recording_state: HotkeyRecordingState,
    pub assignment_history: Vec<HotkeyAssignmentEvent>,
}

#[derive(Reflect, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Hotkey {
    pub key: KeyCode,
    pub modifiers: ModifierKeys,
    pub display_string: String,
}

#[derive(Reflect, Debug, Clone)]
pub struct ModifierKeys {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Cmd on Mac, Windows key on PC
}

#[derive(Component, Reflect, Debug)]
pub struct HotkeyRecorderComponent {
    pub target_command: CommandId,
    pub record_button_entity: Entity,
    pub display_entity: Entity,
    pub clear_button_entity: Option<Entity>,
    pub recording_state: RecordingState,
}

#[derive(Reflect, Debug)]
pub enum RecordingState {
    Idle,
    WaitingForInput,
    Recording { partial_keys: Vec<KeyCode> },
    Complete { recorded_hotkey: Hotkey },
    Error { message: String },
}
```

### Interactive Hotkey Capture
```rust
// Real-time hotkey recording system
#[derive(Event)]
pub struct HotkeyRecordingEvent {
    pub event_type: RecordingEventType,
    pub command_id: CommandId,
    pub hotkey: Option<Hotkey>,
}

#[derive(Reflect, Debug)]
pub enum RecordingEventType {
    StartRecording,
    StopRecording,
    KeyPressed,
    RecordingComplete,
    RecordingCancelled,
}

pub fn hotkey_recording_system(
    mut recording_events: EventReader<HotkeyRecordingEvent>,
    mut key_events: EventReader<KeyboardInput>,
    mut recorder_query: Query<&mut HotkeyRecorderComponent>,
    mut hotkey_res: ResMut<HotkeyAssignmentResource>,
) {
    // Handle recording state transitions
    for recording_event in recording_events.read() {
        match recording_event.event_type {
            RecordingEventType::StartRecording => {
                start_hotkey_recording(&mut hotkey_res, &recording_event.command_id);
            }
            RecordingEventType::RecordingComplete => {
                if let Some(hotkey) = &recording_event.hotkey {
                    complete_hotkey_assignment(&mut hotkey_res, &recording_event.command_id, hotkey);
                }
            }
            _ => {}
        }
    }
    
    // Capture keyboard input during recording
    if hotkey_res.recording_state == HotkeyRecordingState::Active {
        for key_event in key_events.read() {
            if key_event.state == ButtonState::Pressed {
                process_recorded_key(&mut hotkey_res, key_event.key_code);
            }
        }
    }
}

fn process_recorded_key(
    hotkey_res: &mut HotkeyAssignmentResource,
    key_code: KeyCode,
) {
    // Build hotkey from captured keys with zero allocations
    let modifiers = capture_current_modifiers();
    let hotkey = Hotkey {
        key: key_code,
        modifiers,
        display_string: format_hotkey_display(key_code, &modifiers),
    };
    
    // Check for conflicts before assignment
    if let Some(conflict) = detect_hotkey_conflict(&hotkey, &hotkey_res.global_hotkeys) {
        hotkey_res.hotkey_conflicts.push(conflict);
    }
}
```

### Conflict Detection and Resolution
```rust
// Hotkey conflict management
#[derive(Reflect, Debug)]
pub struct HotkeyConflict {
    pub conflicting_hotkey: Hotkey,
    pub existing_command: CommandId,
    pub new_command: CommandId,
    pub conflict_type: ConflictType,
    pub resolution: Option<ConflictResolution>,
}

#[derive(Reflect, Debug)]
pub enum ConflictType {
    ExactMatch,
    PartialMatch,
    SystemConflict,
    GlobalConflict,
}

#[derive(Reflect, Debug)]
pub enum ConflictResolution {
    ReplaceExisting,
    KeepExisting,
    ModifyNew,
    UserChoice,
}

pub fn hotkey_conflict_system(
    mut hotkey_res: ResMut<HotkeyAssignmentResource>,
    conflict_resolution_events: EventReader<ConflictResolutionEvent>,
) {
    for resolution_event in conflict_resolution_events.read() {
        resolve_hotkey_conflict(
            &mut hotkey_res,
            resolution_event.conflict_id,
            &resolution_event.resolution,
        );
    }
}

fn detect_hotkey_conflict(
    new_hotkey: &Hotkey,
    existing_mappings: &HashMap<Hotkey, CommandId>,
) -> Option<HotkeyConflict> {
    if let Some(existing_command) = existing_mappings.get(new_hotkey) {
        Some(HotkeyConflict {
            conflicting_hotkey: new_hotkey.clone(),
            existing_command: existing_command.clone(),
            new_command: CommandId("new_command".to_string()),
            conflict_type: ConflictType::ExactMatch,
            resolution: None,
        })
    } else {
        None
    }
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `input/keyboard_input.rs` - Keyboard event handling
- `input/keyboard_input_events.rs` - Key event processing
- `ui/button.rs` - Recording button interactions

### Implementation Pattern
```rust
// Based on keyboard_input.rs for key capture
fn hotkey_capture_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut recorder_query: Query<&mut HotkeyRecorderComponent>,
) {
    for mut recorder in &mut recorder_query {
        if recorder.recording_state == RecordingState::WaitingForInput {
            let pressed_keys: Vec<KeyCode> = keyboard_input
                .get_pressed()
                .cloned()
                .collect();
            
            if !pressed_keys.is_empty() {
                let hotkey = create_hotkey_from_keys(&pressed_keys);
                recorder.recording_state = RecordingState::Complete {
                    recorded_hotkey: hotkey,
                };
            }
        }
    }
}

// Based on ui/button.rs for recording controls
fn recording_button_system(
    mut interaction_query: Query<(&Interaction, &RecordingAction), Changed<Interaction>>,
    mut recording_events: EventWriter<HotkeyRecordingEvent>,
) {
    for (interaction, recording_action) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match recording_action {
                RecordingAction::StartRecording(command_id) => {
                    recording_events.send(HotkeyRecordingEvent {
                        event_type: RecordingEventType::StartRecording,
                        command_id: command_id.clone(),
                        hotkey: None,
                    });
                }
                RecordingAction::ClearHotkey(command_id) => {
                    recording_events.send(HotkeyRecordingEvent {
                        event_type: RecordingEventType::RecordingCancelled,
                        command_id: command_id.clone(),
                        hotkey: None,
                    });
                }
            }
        }
    }
}
```

## Global Hotkey Integration
- System-wide hotkey registration using `global-hotkey` crate
- Hotkey persistence across application sessions
- Integration with system hotkey reservations
- Cross-platform hotkey compatibility

## Performance Constraints
- **ZERO ALLOCATIONS** during hotkey event processing
- Efficient conflict detection algorithms
- Optimized hotkey lookup structures
- Minimal system resource usage for global hotkeys

## Success Criteria
- Complete hotkey assignment system implementation
- Reliable interactive hotkey capture
- No unwrap()/expect() calls in production code
- Zero-allocation hotkey processing
- Comprehensive conflict detection and resolution

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for hotkey capture logic
- Integration tests for global hotkey registration
- Conflict detection and resolution tests
- Cross-platform hotkey compatibility tests
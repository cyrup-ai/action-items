# Task 6: Hotkey Recording and Management System Implementation

## Objective
Implement the hotkey recording system with "Record Hotkey" interface, global hotkey assignment, conflict detection and resolution, and visual hotkey display in table and detail panel.

## Implementation Details

### Target Files
- `ui/src/ui/components/config/hotkey_recorder.rs:1-250` - Hotkey recording interface
- `core/src/hotkeys/global_registry.rs:1-300` - Global hotkey management
- `core/src/hotkeys/conflict_resolver.rs:1-200` - Hotkey conflict detection and resolution
- `ui/src/ui/components/config/hotkey_display.rs:1-150` - Visual hotkey representation

### Bevy Implementation Patterns

#### Hotkey Recording Interface
**Reference**: `./docs/bevy/examples/input/keyboard_input_events.rs:80-120` - Keyboard input capture for hotkey recording
**Reference**: `./docs/bevy/examples/ui/button.rs:480-520` - Record button state management
```rust
// Hotkey recorder component
#[derive(Component)]
pub struct HotkeyRecorder {
    pub target_item_id: String,
    pub recording: bool,
    pub current_keys: HashSet<KeyCode>,
    pub recorded_hotkey: Option<Hotkey>,
    pub recording_timer: Timer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hotkey {
    pub modifiers: ModifierKeys,
    pub key: KeyCode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModifierKeys {
    pub ctrl: bool,
    pub cmd: bool,
    pub alt: bool,
    pub shift: bool,
}

// Record Hotkey button
ButtonBundle {
    style: Style {
        width: Val::Px(120.0),
        height: Val::Px(32.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    },
    background_color: if recorder.recording {
        Color::rgb(0.8, 0.2, 0.2).into() // Red when recording
    } else {
        Color::rgba(0.3, 0.3, 0.3, 1.0).into() // Gray when idle
    },
    border_color: Color::rgba(0.4, 0.4, 0.4, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(4.0)),
    ..default()
}

// Record button text
TextBundle::from_section(
    if recorder.recording { "Recording..." } else { "Record Hotkey" },
    TextStyle {
        font: font_medium.clone(),
        font_size: 13.0,
        color: Color::WHITE,
    },
)
```

#### Hotkey Recording System
**Reference**: `./docs/bevy/examples/input/keyboard_input_events.rs:140-180` - Advanced keyboard input handling
```rust
// Hotkey recording system
fn hotkey_recording_system(
    mut keyboard_input: EventReader<KeyboardInput>,
    mut recorder_query: Query<&mut HotkeyRecorder>,
    mut hotkey_events: EventWriter<HotkeyRecordedEvent>,
    mut conflict_events: EventWriter<HotkeyConflictEvent>,
    hotkey_registry: Res<GlobalHotkeyRegistry>,
    time: Res<Time>,
) {
    for mut recorder in recorder_query.iter_mut() {
        if !recorder.recording {
            continue;
        }
        
        // Update recording timer
        recorder.recording_timer.tick(time.delta());
        
        // Auto-stop recording after timeout
        if recorder.recording_timer.just_finished() {
            recorder.recording = false;
            continue;
        }
        
        // Process keyboard input during recording
        for input in keyboard_input.iter() {
            match input.state {
                ButtonState::Pressed => {
                    if let Some(key_code) = input.key_code {
                        recorder.current_keys.insert(key_code);
                        
                        // Check if we have a valid hotkey combination
                        if is_valid_hotkey_combination(&recorder.current_keys) {
                            let hotkey = create_hotkey_from_keys(&recorder.current_keys);
                            
                            // Check for conflicts
                            if let Some(conflict) = hotkey_registry.check_conflict(&hotkey, &recorder.target_item_id) {
                                conflict_events.send(HotkeyConflictEvent {
                                    new_hotkey: hotkey.clone(),
                                    conflicting_item: conflict,
                                    target_item: recorder.target_item_id.clone(),
                                });
                            } else {
                                recorder.recorded_hotkey = Some(hotkey.clone());
                                recorder.recording = false;
                                
                                hotkey_events.send(HotkeyRecordedEvent {
                                    item_id: recorder.target_item_id.clone(),
                                    hotkey: hotkey,
                                });
                            }
                        }
                    }
                }
                ButtonState::Released => {
                    if let Some(key_code) = input.key_code {
                        recorder.current_keys.remove(&key_code);
                    }
                }
            }
        }
    }
}

// Validate hotkey combination
fn is_valid_hotkey_combination(keys: &HashSet<KeyCode>) -> bool {
    let has_modifier = keys.contains(&KeyCode::LControl) ||
                      keys.contains(&KeyCode::RControl) ||
                      keys.contains(&KeyCode::LWin) ||
                      keys.contains(&KeyCode::RWin) ||
                      keys.contains(&KeyCode::LAlt) ||
                      keys.contains(&KeyCode::RAlt);
    
    let has_non_modifier = keys.iter().any(|key| {
        !matches!(key, 
            KeyCode::LControl | KeyCode::RControl |
            KeyCode::LWin | KeyCode::RWin |
            KeyCode::LAlt | KeyCode::RAlt |
            KeyCode::LShift | KeyCode::RShift
        )
    });
    
    has_modifier && has_non_modifier
}

// Create hotkey from key combination
fn create_hotkey_from_keys(keys: &HashSet<KeyCode>) -> Hotkey {
    let modifiers = ModifierKeys {
        ctrl: keys.contains(&KeyCode::LControl) || keys.contains(&KeyCode::RControl),
        cmd: keys.contains(&KeyCode::LWin) || keys.contains(&KeyCode::RWin),
        alt: keys.contains(&KeyCode::LAlt) || keys.contains(&KeyCode::RAlt),
        shift: keys.contains(&KeyCode::LShift) || keys.contains(&KeyCode::RShift),
    };
    
    // Find the main key (non-modifier)
    let main_key = keys.iter()
        .find(|key| !matches!(key, 
            KeyCode::LControl | KeyCode::RControl |
            KeyCode::LWin | KeyCode::RWin |
            KeyCode::LAlt | KeyCode::RAlt |
            KeyCode::LShift | KeyCode::RShift
        ))
        .cloned()
        .unwrap_or(KeyCode::Space);
    
    Hotkey {
        modifiers,
        key: main_key,
    }
}
```

### Global Hotkey Registry

#### Registry Management System
**Reference**: `./docs/bevy/examples/ecs/resources.rs:220-260` - Global hotkey state management
```rust
// Global hotkey registry resource
#[derive(Resource, Clone, Debug)]
pub struct GlobalHotkeyRegistry {
    pub assignments: HashMap<String, Hotkey>,
    pub reverse_lookup: HashMap<Hotkey, String>,
    pub conflicts: Vec<HotkeyConflict>,
    pub system_reserved: HashSet<Hotkey>,
}

#[derive(Debug, Clone)]
pub struct HotkeyConflict {
    pub hotkey: Hotkey,
    pub items: Vec<String>,
    pub resolution: ConflictResolution,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    Unresolved,
    ReplaceExisting,
    AssignToNew,
    Cancelled,
}

impl GlobalHotkeyRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            assignments: HashMap::new(),
            reverse_lookup: HashMap::new(),
            conflicts: Vec::new(),
            system_reserved: HashSet::new(),
        };
        
        // Initialize system reserved hotkeys
        registry.init_system_reserved();
        registry
    }
    
    pub fn assign_hotkey(&mut self, item_id: String, hotkey: Hotkey) -> Result<(), HotkeyError> {
        // Check for system reserved conflicts
        if self.system_reserved.contains(&hotkey) {
            return Err(HotkeyError::SystemReserved);
        }
        
        // Remove existing assignment for this item
        if let Some(old_hotkey) = self.assignments.remove(&item_id) {
            self.reverse_lookup.remove(&old_hotkey);
        }
        
        // Remove any existing assignment for this hotkey
        if let Some(old_item) = self.reverse_lookup.remove(&hotkey) {
            self.assignments.remove(&old_item);
        }
        
        // Assign new hotkey
        self.assignments.insert(item_id.clone(), hotkey.clone());
        self.reverse_lookup.insert(hotkey, item_id);
        
        Ok(())
    }
    
    pub fn check_conflict(&self, hotkey: &Hotkey, requesting_item: &str) -> Option<String> {
        if let Some(existing_item) = self.reverse_lookup.get(hotkey) {
            if existing_item != requesting_item {
                Some(existing_item.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn init_system_reserved(&mut self) {
        // macOS system hotkeys
        #[cfg(target_os = "macos")]
        {
            self.system_reserved.insert(Hotkey {
                modifiers: ModifierKeys { cmd: true, ctrl: false, alt: false, shift: false },
                key: KeyCode::Space,
            }); // Spotlight
            
            self.system_reserved.insert(Hotkey {
                modifiers: ModifierKeys { cmd: true, ctrl: false, alt: false, shift: false },
                key: KeyCode::Tab,
            }); // App switching
        }
        
        // Windows system hotkeys
        #[cfg(target_os = "windows")]
        {
            self.system_reserved.insert(Hotkey {
                modifiers: ModifierKeys { cmd: true, ctrl: false, alt: false, shift: false },
                key: KeyCode::L,
            }); // Lock screen
        }
    }
}

#[derive(Debug, Clone)]
pub enum HotkeyError {
    SystemReserved,
    InvalidCombination,
    ConflictUnresolved,
    RegistrationFailed,
}
```

#### Conflict Resolution System
**Reference**: `./docs/bevy/examples/ui/ui.rs:1100-1150` - Conflict resolution modal
```rust
// Hotkey conflict resolution modal
#[derive(Component)]
pub struct HotkeyConflictModal {
    pub conflict: HotkeyConflict,
}

fn spawn_conflict_resolution_modal(
    commands: &mut Commands,
    conflict: HotkeyConflict,
) -> Entity {
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect::all(Val::Px(0.0)),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
        ..default()
    })
    .insert(HotkeyConflictModal { conflict: conflict.clone() })
    .with_children(|parent| {
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Px(400.0),
                padding: UiRect::all(Val::Px(24.0)),
                gap: Size::all(Val::Px(16.0)),
                ..default()
            },
            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        })
        .with_children(|modal| {
            // Conflict message
            modal.spawn(TextBundle::from_section(
                format!("Hotkey {} is already assigned to another item. How would you like to resolve this conflict?",
                    conflict.hotkey.display_string()),
                TextStyle {
                    font: font_regular.clone(),
                    font_size: 14.0,
                    color: Color::WHITE,
                },
            ));
            
            // Resolution options
            create_conflict_resolution_buttons(modal, &conflict);
        });
    }).id()
}

// Conflict resolution buttons
fn create_conflict_resolution_buttons(parent: &mut ChildBuilder, conflict: &HotkeyConflict) {
    parent.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            gap: Size::all(Val::Px(8.0)),
            ..default()
        },
        ..default()
    })
    .with_children(|buttons| {
        // Replace existing assignment
        buttons.spawn(ButtonBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgba(0.2, 0.4, 0.8, 1.0).into(),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        })
        .insert(ConflictResolutionButton {
            resolution: ConflictResolution::ReplaceExisting,
            conflict_id: conflict.hotkey.clone(),
        });
        
        // Cancel assignment
        buttons.spawn(ButtonBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        })
        .insert(ConflictResolutionButton {
            resolution: ConflictResolution::Cancelled,
            conflict_id: conflict.hotkey.clone(),
        });
    });
}
```

### Visual Hotkey Display

#### Hotkey Display Component
**Reference**: `./docs/bevy/examples/ui/ui.rs:1200-1240` - Visual hotkey representation
```rust
// Hotkey display component for table and detail panel
#[derive(Component)]
pub struct HotkeyDisplay {
    pub hotkey: Option<Hotkey>,
    pub style: HotkeyDisplayStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HotkeyDisplayStyle {
    Table,      // Compact display for table cells
    Detail,     // Larger display for detail panel
    Recording,  // Display during recording
}

impl Hotkey {
    pub fn display_string(&self) -> String {
        let mut parts = Vec::new();
        
        #[cfg(target_os = "macos")]
        {
            if self.modifiers.ctrl { parts.push("⌃".to_string()); }
            if self.modifiers.cmd { parts.push("⌘".to_string()); }
            if self.modifiers.alt { parts.push("⌥".to_string()); }
            if self.modifiers.shift { parts.push("⇧".to_string()); }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            if self.modifiers.ctrl { parts.push("Ctrl".to_string()); }
            if self.modifiers.cmd { parts.push("Win".to_string()); }
            if self.modifiers.alt { parts.push("Alt".to_string()); }
            if self.modifiers.shift { parts.push("Shift".to_string()); }
        }
        
        parts.push(self.key_display_name());
        parts.join(" ")
    }
    
    fn key_display_name(&self) -> String {
        match self.key {
            KeyCode::Space => "Space".to_string(),
            KeyCode::Return => "Enter".to_string(),
            KeyCode::Escape => "Esc".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Back => "Backspace".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::A => "A".to_string(),
            KeyCode::B => "B".to_string(),
            // ... continue for all keys
            _ => format!("{:?}", self.key),
        }
    }
}

// Create hotkey display UI
fn create_hotkey_display(parent: &mut ChildBuilder, display: &HotkeyDisplay) {
    if let Some(ref hotkey) = display.hotkey {
        parent.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                gap: Size::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::rgba(0.2, 0.2, 0.2, 1.0).into(),
            border_color: Color::rgba(0.4, 0.4, 0.4, 1.0).into(),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            ..default()
        })
        .with_children(|hotkey_container| {
            hotkey_container.spawn(TextBundle::from_section(
                hotkey.display_string(),
                TextStyle {
                    font: font_mono.clone(),
                    font_size: match display.style {
                        HotkeyDisplayStyle::Table => 11.0,
                        HotkeyDisplayStyle::Detail => 13.0,
                        HotkeyDisplayStyle::Recording => 12.0,
                    },
                    color: Color::WHITE,
                },
            ));
        });
    } else {
        // No hotkey assigned
        parent.spawn(TextBundle::from_section(
            "None",
            TextStyle {
                font: font_regular.clone(),
                font_size: 11.0,
                color: Color::rgba(0.5, 0.5, 0.5, 1.0),
            },
        ));
    }
}
```

### Event System Integration

#### Hotkey Events
**Reference**: `./docs/bevy/examples/ecs/event.rs:160-190` - Hotkey system events
```rust
// Hotkey system events
#[derive(Event)]
pub struct HotkeyRecordedEvent {
    pub item_id: String,
    pub hotkey: Hotkey,
}

#[derive(Event)]
pub struct HotkeyConflictEvent {
    pub new_hotkey: Hotkey,
    pub conflicting_item: String,
    pub target_item: String,
}

#[derive(Event)]
pub struct HotkeyRemovedEvent {
    pub item_id: String,
}

#[derive(Component)]
pub struct ConflictResolutionButton {
    pub resolution: ConflictResolution,
    pub conflict_id: Hotkey,
}
```

### Architecture Notes

#### Component Structure
- **HotkeyRecorder**: Recording state and interface management
- **GlobalHotkeyRegistry**: System-wide hotkey assignment tracking
- **HotkeyDisplay**: Visual representation of hotkey combinations
- **HotkeyConflictModal**: Conflict resolution interface

#### Recording Workflow
- **Activation**: Click "Record Hotkey" button to start recording
- **Input Capture**: Monitor keyboard input for valid combinations
- **Validation**: Ensure hotkey combination meets requirements
- **Conflict Check**: Verify no conflicts with existing assignments

#### System Integration
- **Global Registration**: System-wide hotkey registration and management
- **Platform Adaptation**: Platform-specific modifier key handling
- **Conflict Resolution**: User-guided conflict resolution workflow
- **State Persistence**: Hotkey assignments persist across sessions

### Quality Standards
- Accurate hotkey recording with proper modifier key detection
- Comprehensive conflict detection and resolution system
- Clear visual feedback during recording and conflict states
- Cross-platform compatibility for hotkey combinations
- Performance optimization for real-time input handling

### Integration Points
- Table interface integration for hotkey display in table cells
- Detail panel integration for hotkey configuration interface
- Extension management system for hotkey persistence
- Global input system integration for hotkey activation
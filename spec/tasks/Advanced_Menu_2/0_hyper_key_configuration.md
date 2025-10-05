# Task 0: Hyper Key Configuration System

## Implementation Details

**File**: `ui/src/ui/hyper_key.rs`  
**Lines**: 85-165  
**Architecture**: System-level hyper key integration with conflict detection  
**Integration**: KeyboardSystem, SettingsSystem, PlatformAPI  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct HyperKeyManager {
    pub current_assignment: Option<HyperKeyAssignment>,
    pub available_keys: Vec<HyperKeyOption>,
    pub registration_status: RegistrationStatus,
    pub conflict_detector: ConflictDetector,
    pub system_integration: SystemIntegration,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HyperKeyAssignment {
    pub key_type: HyperKeyType,
    pub key_code: KeyCode,
    pub system_registered: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HyperKeyType {
    Disabled,
    CapsLockRemap,
    RightOption,
    RightCommand,
    FunctionKey,
    CustomCombination,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RegistrationStatus {
    None,
    Registered,
    Failed(String),
    Conflicted(Vec<String>),
}

pub fn hyper_key_system(
    mut hyper_key_manager: ResMut<HyperKeyManager>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut system_events: EventWriter<SystemIntegrationEvent>,
    mut ui_events: EventWriter<HyperKeyEvent>,
) {
    // Monitor hyper key registration status
    match &hyper_key_manager.registration_status {
        RegistrationStatus::Failed(error) => {
            ui_events.send(HyperKeyEvent::RegistrationFailed(error.clone()));
        }
        RegistrationStatus::Conflicted(conflicts) => {
            ui_events.send(HyperKeyEvent::ConflictDetected(conflicts.clone()));
        }
        _ => {}
    }

    // Handle hyper key events
    for keyboard_event in keyboard_events.read() {
        if let Some(ref assignment) = hyper_key_manager.current_assignment {
            if keyboard_event.key_code == assignment.key_code {
                match keyboard_event.state {
                    ButtonState::Pressed => {
                        ui_events.send(HyperKeyEvent::HyperKeyPressed);
                    }
                    ButtonState::Released => {
                        ui_events.send(HyperKeyEvent::HyperKeyReleased);
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub fn register_hyper_key_macos(assignment: &HyperKeyAssignment) -> Result<(), String> {
    use core_graphics::event::{CGEventTapLocation, CGEventType};
    use core_graphics::event_source::CGEventSource;
    
    match assignment.key_type {
        HyperKeyType::CapsLockRemap => {
            // Use IOKit to remap Caps Lock key
            unsafe {
                let result = IOHIDSetModifierMappingForService(
                    kIOMasterPortDefault,
                    std::ptr::null(),
                );
                
                if result != 0 {
                    return Err("Failed to register Caps Lock hyper key".to_string());
                }
            }
        }
        HyperKeyType::RightOption | HyperKeyType::RightCommand => {
            // Register system-wide event tap
            let event_tap = CGEventTap::new(
                CGEventTapLocation::HID,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::DefaultTap,
                vec![CGEventType::KeyDown, CGEventType::KeyUp],
                hyper_key_callback,
            );
            
            if event_tap.is_none() {
                return Err("Failed to create event tap for hyper key".to_string());
            }
        }
        _ => {}
    }
    
    Ok(())
}
```

### Conflict Detection System

**Reference**: `./docs/bevy/examples/input/keyboard_input.rs:185-218`

```rust
#[derive(Clone, Debug)]
pub struct ConflictDetector {
    pub system_shortcuts: HashMap<KeyCode, Vec<String>>,
    pub app_shortcuts: HashMap<KeyCode, Vec<String>>,
    pub last_scan: Instant,
}

impl ConflictDetector {
    pub fn detect_conflicts(&self, assignment: &HyperKeyAssignment) -> Vec<String> {
        let mut conflicts = Vec::new();
        
        // Check system-level conflicts
        if let Some(system_conflicts) = self.system_shortcuts.get(&assignment.key_code) {
            conflicts.extend(system_conflicts.iter().cloned());
        }
        
        // Check application-level conflicts
        if let Some(app_conflicts) = self.app_shortcuts.get(&assignment.key_code) {
            conflicts.extend(app_conflicts.iter().cloned());
        }
        
        // Platform-specific conflict detection
        match assignment.key_type {
            HyperKeyType::CapsLockRemap => {
                if self.caps_lock_in_use() {
                    conflicts.push("Caps Lock already remapped by system".to_string());
                }
            }
            HyperKeyType::RightOption => {
                if self.right_option_in_use() {
                    conflicts.push("Right Option used by input method".to_string());
                }
            }
            _ => {}
        }
        
        conflicts
    }
    
    fn caps_lock_in_use(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            // Check macOS modifier mapping
            unsafe {
                let mapping = IOHIDGetModifierMappingForService(
                    kIOMasterPortDefault,
                    std::ptr::null(),
                );
                !mapping.is_null()
            }
        }
        #[cfg(not(target_os = "macos"))]
        false
    }
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui/ui_dropdown.rs:225-262`

```rust
// Hyper key configuration dropdown
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        width: Val::Percent(100.0),
        height: Val::Px(40.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::horizontal(Val::Px(16.0)),
        margin: UiRect::bottom(Val::Px(8.0)),
        ..default()
    },
    background_color: Color::rgba(0.12, 0.12, 0.12, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(6.0)),
    ..default()
},
children: &[
    (TextBundle::from_section(
        "Hyper Key",
        TextStyle {
            font: asset_server.load("fonts/Inter-Medium.ttf"),
            font_size: 14.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    ),),
    (DropdownBundle {
        options: vec![
            "– (Disabled)".to_string(),
            "⇪ Caps Lock".to_string(), 
            "⌥ Right Option".to_string(),
            "⌘ Right Command".to_string(),
            "fn Function Key".to_string(),
            "Custom...".to_string(),
        ],
        selected_index: match hyper_key_manager.current_assignment {
            Some(ref assignment) => match assignment.key_type {
                HyperKeyType::Disabled => 0,
                HyperKeyType::CapsLockRemap => 1,
                HyperKeyType::RightOption => 2,
                HyperKeyType::RightCommand => 3,
                HyperKeyType::FunctionKey => 4,
                HyperKeyType::CustomCombination => 5,
            },
            None => 0,
        },
        width: Val::Px(200.0),
        ..default()
    },),
    (InfoIconBundle {
        tooltip: "Hyper key acts as a super-modifier for creating unique shortcuts that won't conflict with system or application shortcuts".to_string(),
        ..default()
    },),
]
```

### Architecture Notes

- System-level integration with platform-specific APIs (IOKit on macOS, Windows API on Windows)
- Real-time conflict detection prevents registration of conflicting key assignments
- Graceful fallback when system integration fails or permissions are insufficient
- Event-driven architecture ensures responsive hyper key handling
- Comprehensive error handling with user-friendly error messages
- Platform-aware key mapping with OS-specific modifier handling

**Bevy Examples**: `./docs/bevy/examples/input/keyboard_input.rs:128-165`, `./docs/bevy/examples/ui/ui_dropdown.rs:45-82`  
**Integration Points**: KeyboardSystem, SettingsSystem, PlatformIntegration  
**Dependencies**: PlatformAPI, InputResource, SettingsResource
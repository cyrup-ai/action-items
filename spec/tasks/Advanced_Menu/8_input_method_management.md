# Task 8: Input Method Management System

## Implementation Details

**File**: `ui/src/ui/input_method.rs`  
**Lines**: 200-275  
**Architecture**: Cross-platform input method support with locale-aware handling  
**Integration**: KeyboardSystem, SettingsSystem, LocalizationManager  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct InputMethodManager {
    pub active_ime: Option<InputMethodEngine>,
    pub available_methods: Vec<InputMethodInfo>,
    pub composition_text: String,
    pub composition_cursor: usize,
    pub candidate_window: Option<CandidateWindow>,
    pub locale_preferences: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputMethodInfo {
    pub id: String,
    pub display_name: String,
    pub locale: String,
    pub is_system_default: bool,
    pub supports_composition: bool,
    pub icon_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct InputMethodEngine {
    pub info: InputMethodInfo,
    pub state: CompositionState,
    pub converter: Box<dyn TextConverter>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompositionState {
    Inactive,
    Composing { text: String, cursor: usize },
    Candidates { choices: Vec<String>, selected: usize },
}

pub fn input_method_system(
    mut ime_manager: ResMut<InputMethodManager>,
    mut text_input_events: EventReader<ReceivedCharacter>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut ui_events: EventWriter<TextInputEvent>,
    windows: Query<&Window>,
) {
    // Handle character input through IME
    for char_event in text_input_events.read() {
        if let Some(ref mut ime) = ime_manager.active_ime {
            match ime.state {
                CompositionState::Inactive => {
                    if char_event.char.is_ascii() {
                        // Direct ASCII input
                        ui_events.send(TextInputEvent::CharacterInput(char_event.char));
                    } else {
                        // Start composition for non-ASCII
                        ime.state = CompositionState::Composing {
                            text: char_event.char.to_string(),
                            cursor: 1,
                        };
                        ime_manager.composition_text = char_event.char.to_string();
                    }
                }
                CompositionState::Composing { ref mut text, ref mut cursor } => {
                    text.push(char_event.char);
                    *cursor = text.chars().count();
                    ime_manager.composition_text = text.clone();
                }
                _ => {}
            }
        }
    }

    // Handle keyboard navigation in candidate selection
    for key_event in keyboard_events.read() {
        if key_event.state == ButtonState::Pressed {
            if let Some(ref mut ime) = ime_manager.active_ime {
                match &mut ime.state {
                    CompositionState::Candidates { choices, selected } => {
                        match key_event.key_code {
                            KeyCode::ArrowDown => {
                                *selected = (*selected + 1) % choices.len();
                            }
                            KeyCode::ArrowUp => {
                                *selected = selected.saturating_sub(1);
                            }
                            KeyCode::Enter => {
                                let choice = choices[*selected].clone();
                                ui_events.send(TextInputEvent::CompositionCommit(choice));
                                ime.state = CompositionState::Inactive;
                                ime_manager.composition_text.clear();
                            }
                            KeyCode::Escape => {
                                ime.state = CompositionState::Inactive;
                                ime_manager.composition_text.clear();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
```

### Platform Integration

**Reference**: `./docs/bevy/examples/keyboard_input.rs:285-312`

```rust
#[cfg(target_os = "windows")]
pub fn initialize_windows_ime(manager: &mut InputMethodManager) {
    use winapi::um::imm::{ImmGetDefaultIMEWnd, ImmGetIMEFileNameW};
    
    let mut methods = Vec::new();
    
    // Enumerate installed IMEs
    unsafe {
        let hwnd = std::ptr::null_mut();
        let ime_hwnd = ImmGetDefaultIMEWnd(hwnd);
        
        if !ime_hwnd.is_null() {
            methods.push(InputMethodInfo {
                id: "windows_ime".to_string(),
                display_name: "Windows IME".to_string(),
                locale: get_system_locale(),
                is_system_default: true,
                supports_composition: true,
                icon_path: None,
            });
        }
    }
    
    manager.available_methods = methods;
}

#[cfg(target_os = "macos")]
pub fn initialize_macos_ime(manager: &mut InputMethodManager) {
    use core_foundation::*;
    
    let mut methods = Vec::new();
    
    // Use Input Source Services to enumerate
    let input_sources = unsafe {
        TISCreateInputSourceList(std::ptr::null(), false as Boolean)
    };
    
    if !input_sources.is_null() {
        methods.push(InputMethodInfo {
            id: "macos_ime".to_string(),
            display_name: "macOS Input Source".to_string(),
            locale: get_system_locale(),
            is_system_default: true,
            supports_composition: true,
            icon_path: None,
        });
    }
    
    manager.available_methods = methods;
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui_dropdown.rs:128-165`

```rust
// Input method selection dropdown
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
        "Input Method",
        TextStyle {
            font: asset_server.load("fonts/Inter-Medium.ttf"),
            font_size: 14.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    ),),
    (DropdownBundle {
        options: ime_manager.available_methods.iter()
            .map(|method| method.display_name.clone())
            .collect(),
        selected_index: ime_manager.available_methods.iter()
            .position(|method| method.is_system_default)
            .unwrap_or(0),
        width: Val::Px(200.0),
        ..default()
    },),
]
```

### Architecture Notes

- Cross-platform IME support with native API integration
- Composition text handling for complex character input
- Candidate window management for character selection
- Locale-aware input method detection and configuration
- Real-time composition preview with cursor positioning
- Integration with system default input methods

**Bevy Examples**: `./docs/bevy/examples/text_input.rs:75-102`, `./docs/bevy/examples/unicode_input.rs:38-65`  
**Integration Points**: TextInputSystem, LocalizationSystem, SettingsSystem  
**Dependencies**: PlatformAPI, LocaleResource, InputResource
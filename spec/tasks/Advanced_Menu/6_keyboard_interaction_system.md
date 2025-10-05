# Task 6: Keyboard Interaction and Shortcuts System

## Implementation Details

**File**: `ui/src/ui/keyboard_shortcuts.rs`  
**Lines**: 125-190  
**Architecture**: Event-driven keyboard handling with configurable shortcuts  
**Integration**: NavigationSystem, SettingsSystem, InputManager  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct KeyboardShortcuts {
    pub shortcuts: HashMap<ShortcutKey, ShortcutAction>,
    pub modifier_state: ModifierState,
    pub repeat_enabled: bool,
    pub repeat_delay: f32,
    pub repeat_rate: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShortcutKey {
    pub key: KeyCode,
    pub modifiers: ModifierState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModifierState {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Cmd on macOS, Win on Windows
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShortcutAction {
    Navigation(NavigationAction),
    Search(SearchAction),
    Window(WindowAction),
    Settings(SettingsAction),
    Custom(String),
}

pub fn keyboard_shortcuts_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut shortcuts: ResMut<KeyboardShortcuts>,
    mut navigation_events: EventWriter<NavigationEvent>,
    mut search_events: EventWriter<SearchEvent>,
    mut window_events: EventWriter<WindowEvent>,
    time: Res<Time>,
) {
    // Update modifier state
    shortcuts.modifier_state = ModifierState {
        ctrl: keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight),
        alt: keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight),
        shift: keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight),
        meta: keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight),
    };

    // Process shortcut keys
    for (shortcut_key, action) in &shortcuts.shortcuts {
        if keyboard.just_pressed(shortcut_key.key) 
            && shortcut_key.modifiers == shortcuts.modifier_state {
            
            match action {
                ShortcutAction::Navigation(nav_action) => {
                    navigation_events.send(NavigationEvent::Shortcut(*nav_action));
                }
                ShortcutAction::Search(search_action) => {
                    search_events.send(SearchEvent::Shortcut(*search_action));
                }
                ShortcutAction::Window(window_action) => {
                    window_events.send(WindowEvent::Shortcut(*window_action));
                }
                ShortcutAction::Settings(settings_action) => {
                    // Handle settings shortcuts
                }
                ShortcutAction::Custom(command) => {
                    // Execute custom command
                }
            }
        }
    }
}
```

### Default Shortcuts Configuration

**Reference**: `./docs/bevy/examples/keyboard_input.rs:128-155`

```rust
impl Default for KeyboardShortcuts {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        
        // Navigation shortcuts
        shortcuts.insert(
            ShortcutKey { key: KeyCode::KeyK, modifiers: ModifierState { ctrl: true, ..default() } },
            ShortcutAction::Search(SearchAction::FocusSearchBar)
        );
        shortcuts.insert(
            ShortcutKey { key: KeyCode::Comma, modifiers: ModifierState { ctrl: true, ..default() } },
            ShortcutAction::Settings(SettingsAction::OpenPreferences)
        );
        shortcuts.insert(
            ShortcutKey { key: KeyCode::KeyW, modifiers: ModifierState { ctrl: true, ..default() } },
            ShortcutAction::Window(WindowAction::Close)
        );
        shortcuts.insert(
            ShortcutKey { key: KeyCode::Enter, modifiers: ModifierState { ctrl: true, ..default() } },
            ShortcutAction::Navigation(NavigationAction::ExecuteSelected)
        );

        Self {
            shortcuts,
            modifier_state: ModifierState::default(),
            repeat_enabled: true,
            repeat_delay: 0.5,
            repeat_rate: 0.05,
        }
    }
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui_buttons.rs:178-205`

```rust
// Keyboard shortcuts settings section
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
        "Keyboard Shortcuts",
        TextStyle {
            font: asset_server.load("fonts/Inter-SemiBold.ttf"),
            font_size: 16.0,
            color: Color::rgb(0.95, 0.95, 0.95),
        },
    ),),
    // Shortcut list with editable bindings
    (ScrollingListBundle {
        max_height: Val::Px(200.0),
        item_height: Val::Px(32.0),
        ..default()
    },),
]
```

### Architecture Notes

- Event-driven architecture prevents blocking UI updates
- Configurable shortcuts with persistent storage
- Platform-aware modifier key handling (Cmd/Ctrl differences)
- Key repeat functionality with adjustable timing
- Conflict detection prevents duplicate assignments
- Real-time shortcut editing with immediate feedback

**Bevy Examples**: `./docs/bevy/examples/keyboard_input_events.rs:35-68`, `./docs/bevy/examples/ui_text_input.rs:88-115`  
**Integration Points**: NavigationSystem, SearchSystem, WindowManager  
**Dependencies**: InputResource, SettingsResource, EventSystems
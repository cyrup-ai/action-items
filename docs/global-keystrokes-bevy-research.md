# Global Keystrokes Research in Bevy

**Sequential Thinking Analysis of Bevy Input System**

## Research Methodology

This research follows sequential thinking patterns to understand global keystroke handling in Bevy through systematic examination of official examples and documentation patterns.

## Core Input System Components

### 1. Basic Keyboard Input (ButtonInput<KeyCode>)

**Primary Pattern:**
```rust
use bevy::prelude::*;

fn keyboard_input_system(keyboard_input: Res<ButtonInput<KeyCode>>) {
    // Check if key is currently pressed
    if keyboard_input.pressed(KeyCode::KeyA) {
        info!("'A' currently pressed");
    }
    
    // Check if key was just pressed this frame
    if keyboard_input.just_pressed(KeyCode::KeyA) {
        info!("'A' just pressed");
    }
    
    // Check if key was just released this frame
    if keyboard_input.just_released(KeyCode::KeyA) {
        info!("'A' just released");
    }
}
```

**Key Insights:**
- `ButtonInput<KeyCode>` is the primary resource for handling keyboard state
- Provides frame-perfect input detection with `just_pressed()` and `just_released()`
- Allows continuous input monitoring with `pressed()`
- Runs every frame in `Update` schedule

### 2. Advanced Keyboard Event Handling

**Event-Based Pattern:**
```rust
use bevy::{input::keyboard::KeyboardInput, prelude::*};

fn print_keyboard_event_system(mut keyboard_input_events: EventReader<KeyboardInput>) {
    for event in keyboard_input_events.read() {
        info!("Keyboard event: {:?}", event);
        // Access to full event data including:
        // - event.key_code
        // - event.logical_key
        // - event.state (Pressed/Released)
        // - event.window
    }
}
```

**Key Insights:**
- Lower-level access to raw keyboard events
- Provides additional metadata like window ID
- Essential for text input handling and complex key combinations
- More granular control than ButtonInput resource

### 3. Modifier Keys and Combinations

**Modifier Pattern:**
```rust
fn keyboard_input_system(input: Res<ButtonInput<KeyCode>>) {
    // Check multiple modifier combinations
    let shift = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let alt = input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    
    // Global hotkey: Ctrl + Shift + A
    if ctrl && shift && input.just_pressed(KeyCode::KeyA) {
        info!("Global hotkey triggered: Ctrl + Shift + A");
    }
}
```

**Key Insights:**
- Use `any_pressed()` for left/right modifier keys
- Combine multiple conditions for complex shortcuts
- Perfect for global application shortcuts like launcher triggers

### 4. Text Input Handling with IME Support

**Advanced Text Pattern:**
```rust
use bevy::{input::keyboard::{Key, KeyboardInput}, prelude::*};

fn listen_keyboard_input_events(
    mut events: EventReader<KeyboardInput>,
    mut text_state: Local<String>,
) {
    for event in events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match (&event.logical_key, &event.text) {
            (Key::Enter, _) => {
                info!("Enter pressed - execute action with: {}", *text_state);
                text_state.clear();
            }
            (Key::Backspace, _) => {
                text_state.pop();
            }
            (_, Some(inserted_text)) => {
                // Filter printable characters only
                if inserted_text.chars().all(is_printable_char) {
                    text_state.push_str(inserted_text);
                }
            }
            _ => {}
        }
    }
}

fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);
    
    !is_in_private_use_area && !chr.is_ascii_control()
}
```

**Key Insights:**
- `event.logical_key` for key identification
- `event.text` for actual text input
- Proper Unicode and IME support
- Essential for search input in launcher applications

## Global Keystroke Patterns for Launchers

### 1. Application-Wide Hotkey System

```rust
#[derive(Resource)]
pub struct GlobalHotkeys {
    pub toggle_launcher: Vec<KeyCode>,
    pub quick_search: Vec<KeyCode>,
    pub settings: Vec<KeyCode>,
}

impl Default for GlobalHotkeys {
    fn default() -> Self {
        Self {
            toggle_launcher: vec![KeyCode::ControlLeft, KeyCode::Space],
            quick_search: vec![KeyCode::ControlLeft, KeyCode::ShiftLeft, KeyCode::KeyF],
            settings: vec![KeyCode::ControlLeft, KeyCode::Comma],
        }
    }
}

fn global_hotkey_system(
    input: Res<ButtonInput<KeyCode>>,
    hotkeys: Res<GlobalHotkeys>,
    mut launcher_events: EventWriter<LauncherEvent>,
) {
    // Check toggle launcher hotkey
    if hotkeys.toggle_launcher.iter().all(|key| input.pressed(*key)) 
        && input.just_pressed(KeyCode::Space) {
        launcher_events.send(LauncherEvent::Toggle);
    }
    
    // Check quick search hotkey  
    if hotkeys.quick_search.iter().all(|key| input.pressed(*key))
        && input.just_pressed(KeyCode::KeyF) {
        launcher_events.send(LauncherEvent::QuickSearch);
    }
}
```

### 2. Context-Aware Input Handling

```rust
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    #[default]
    Background,
    LauncherActive,
    SearchMode,
}

fn context_aware_input_system(
    input: Res<ButtonInput<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match app_state.get() {
        AppState::Background => {
            // Global launcher activation
            if input.just_pressed(KeyCode::SuperLeft) {
                next_state.set(AppState::LauncherActive);
            }
        }
        AppState::LauncherActive => {
            // Launcher-specific shortcuts
            if input.just_pressed(KeyCode::Escape) {
                next_state.set(AppState::Background);
            }
            if input.just_pressed(KeyCode::Tab) {
                next_state.set(AppState::SearchMode);
            }
        }
        AppState::SearchMode => {
            // Search-specific handling
            if input.just_pressed(KeyCode::Escape) {
                next_state.set(AppState::LauncherActive);
            }
        }
    }
}
```

### 3. High-Performance Input Scheduling

**System Scheduling Pattern:**
```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<GlobalHotkeys>()
        .init_state::<AppState>()
        .add_systems(
            Update,
            (
                // High-priority global input (runs first)
                global_hotkey_system,
                context_aware_input_system,
                
                // Application-specific input (runs after state changes)
                launcher_input_system.run_if(in_state(AppState::LauncherActive)),
                search_input_system.run_if(in_state(AppState::SearchMode)),
            ).chain(), // Ensures proper execution order
        )
        .run();
}
```

### 4. Input Focus Management

**Focus-Aware Pattern (from tab_navigation.rs):**
```rust
use bevy::input_focus::{InputFocus, TabNavigationPlugin};

fn focus_aware_input_system(
    input: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
) {
    // Only handle global shortcuts when no UI element has focus
    if focus.0.is_none() {
        if input.just_pressed(KeyCode::Space) {
            info!("Global space shortcut - no UI focus");
        }
    }
}
```

## Performance Considerations

### 1. Input Polling vs Events

**Resource Pattern (Polling):**
- Use `ButtonInput<KeyCode>` for continuous input (movement, held keys)
- Lower latency, checked every frame
- Better for game-like interactions

**Event Pattern (Discrete):**
- Use `EventReader<KeyboardInput>` for discrete actions
- More efficient for infrequent inputs
- Better for menu interactions and shortcuts

### 2. System Ordering

```rust
.add_systems(
    Update,
    (
        // Priority order for input handling
        global_hotkey_system,        // Highest priority
        ui_input_system,            // UI interactions
        game_input_system,          // Game-specific
        debug_input_system,         // Lowest priority
    ).chain()
)
```

## Implementation Blueprint for Action Items Launcher

### 1. Core Global Keystroke System

```rust
#[derive(Resource, Default)]
pub struct LauncherHotkeys {
    pub activation_key: KeyCode,
    pub modifier_keys: Vec<KeyCode>,
    pub search_shortcut: Vec<KeyCode>,
    pub escape_key: KeyCode,
}

impl LauncherHotkeys {
    pub fn raycast_style() -> Self {
        Self {
            activation_key: KeyCode::Space,
            modifier_keys: vec![KeyCode::ControlLeft],
            search_shortcut: vec![KeyCode::ControlLeft, KeyCode::KeyK],
            escape_key: KeyCode::Escape,
        }
    }
    
    pub fn alfred_style() -> Self {
        Self {
            activation_key: KeyCode::Space,
            modifier_keys: vec![KeyCode::AltLeft],
            search_shortcut: vec![KeyCode::AltLeft, KeyCode::Space],
            escape_key: KeyCode::Escape,
        }
    }
}
```

### 2. Integration with Existing Plugin System

```rust
pub fn launcher_global_input_system(
    input: Res<ButtonInput<KeyCode>>,
    hotkeys: Res<LauncherHotkeys>,
    mut launcher_events: EventWriter<LauncherToggleEvent>,
    mut search_events: EventWriter<SearchFocusEvent>,
    app_state: Res<State<AppState>>,
) {
    // Global launcher activation (works from any state)
    if hotkeys.modifier_keys.iter().all(|k| input.pressed(*k)) 
        && input.just_pressed(hotkeys.activation_key) {
        launcher_events.send(LauncherToggleEvent);
        return;
    }
    
    // Context-aware shortcuts
    match app_state.get() {
        AppState::LauncherActive => {
            if input.just_pressed(hotkeys.escape_key) {
                launcher_events.send(LauncherToggleEvent);
            }
        }
        _ => {
            // Quick search from anywhere
            if hotkeys.search_shortcut.iter().all(|k| input.pressed(*k)) {
                search_events.send(SearchFocusEvent);
            }
        }
    }
}
```

## Conclusion

**Sequential Analysis Summary:**
1. **Basic Input**: `ButtonInput<KeyCode>` for state-based detection
2. **Event Input**: `EventReader<KeyboardInput>` for detailed event handling  
3. **Modifiers**: Use `any_pressed()` for left/right variants
4. **Text Input**: Combine logical keys with text events for proper Unicode support
5. **Global Hotkeys**: Check modifier combinations with `just_pressed()` for triggers
6. **Context Awareness**: Use Bevy states to control input behavior
7. **Performance**: Chain systems by priority, use run conditions for efficiency

This research provides a complete foundation for implementing Raycast-style global keystroke handling in the Bevy-based action items launcher, with proper performance optimization and context awareness.
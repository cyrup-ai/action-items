# Bevy Global Keystroke Handling Research

**Research Date:** 2025-01-08  
**Context:** Action Items Launcher - Global hotkey implementation for show/hide functionality  
**Source:** Bevy Examples Analysis  

## Overview

This research explores how to implement global keystroke handling in Bevy applications, specifically for launcher applications that need to respond to hotkeys even when not focused (like Cmd+Space for Spotlight/Raycast).

## Key Findings

### 1. Basic Keyboard Input (`keyboard_input.rs`)

**Pattern:** Resource-based input polling
```rust
use bevy::prelude::*;

fn keyboard_input_system(keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyA) {
        info!("'A' just pressed");
    }
    if keyboard_input.just_released(KeyCode::KeyA) {
        info!("'A' just released");
    }
}
```

**Characteristics:**
- Uses `ButtonInput<KeyCode>` resource
- Polling-based approach
- Methods: `pressed()`, `just_pressed()`, `just_released()`
- Only works when window is focused

### 2. Event-Driven Input (`keyboard_input_events.rs`)

**Pattern:** Event stream processing
```rust
use bevy::{input::keyboard::KeyboardInput, prelude::*};

fn print_keyboard_event_system(mut keyboard_input_events: EventReader<KeyboardInput>) {
    for event in keyboard_input_events.read() {
        info!("{:?}", event);
    }
}
```

**Characteristics:**
- Uses `EventReader<KeyboardInput>`
- Event-driven, more reactive
- Captures all keyboard events with detailed information
- Still limited to focused window

### 3. Modifier Key Combinations (`keyboard_modifiers.rs`)

**Pattern:** Complex hotkey detection
```rust
fn keyboard_input_system(input: Res<ButtonInput<KeyCode>>) {
    let shift = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    
    if ctrl && shift && input.just_pressed(KeyCode::KeyA) {
        info!("Just pressed Ctrl + Shift + A!");
    }
}
```

**Characteristics:**
- Handles modifier combinations (Ctrl, Shift, Alt, Cmd)
- Uses `any_pressed()` for left/right modifier variants
- Perfect for launcher hotkeys (Cmd+Space)
- macOS: Use `KeyCode::SuperLeft`/`KeyCode::SuperRight` for Cmd key

### 4. Custom Events & External Integration (`custom_user_event.rs`)

**Pattern:** External event injection into Bevy's event loop
```rust
use bevy::winit::{EventLoopProxyWrapper, WinitPlugin};

#[derive(Default, Debug, Event)]
enum CustomEvent {
    WakeUp,
    GlobalHotkey(String),
}

fn send_event(event_loop_proxy: Res<EventLoopProxyWrapper<CustomEvent>>) {
    // Can be called from external threads
    let _ = event_loop_proxy.send_event(CustomEvent::GlobalHotkey("cmd+space".to_string()));
}

fn handle_event(mut events: EventReader<CustomEvent>) {
    for evt in events.read() {
        info!("Received event: {evt:?}");
    }
}
```

**Characteristics:**
- Custom event types that integrate with winit
- `EventLoopProxyWrapper` for thread-safe event sending
- Can be triggered from external threads/libraries
- **KEY INSIGHT:** This is how we can integrate global hotkey libraries

### 5. Custom Application Runner (`custom_loop.rs`)

**Pattern:** Manual app lifecycle control
```rust
fn my_runner(mut app: App) -> AppExit {
    app.finish();
    app.cleanup();
    
    // Custom event loop
    for external_event in some_external_source() {
        app.update();
        if let Some(exit) = app.should_exit() {
            return exit;
        }
    }
    
    AppExit::Success
}

fn main() -> AppExit {
    App::new()
        .set_runner(my_runner)
        .add_systems(Update, my_systems)
        .run()
}
```

**Characteristics:**
- Complete control over application lifecycle
- Can integrate with external event sources
- Useful for specialized launcher behavior

## Implementation Strategy for Global Hotkeys

### Option 1: External Library Integration (Recommended)

**Libraries to consider:**
- `global-hotkey` crate - Cross-platform global hotkey registration
- `tao` (used by Tauri) - Window and global event handling
- `rdev` - Global input capture

**Implementation Pattern:**
```rust
use bevy::prelude::*;
use bevy::winit::{EventLoopProxyWrapper, WinitPlugin};
use global_hotkey::{GlobalHotKeyManager, HotKeyState};

#[derive(Default, Debug, Event)]
enum LauncherEvent {
    #[default]
    None,
    ToggleVisibility,
    Hide,
}

// Spawn background thread for global hotkey listening
fn setup_global_hotkeys(event_loop_proxy: Res<EventLoopProxyWrapper<LauncherEvent>>) {
    let proxy = event_loop_proxy.clone();
    
    std::thread::spawn(move || {
        let manager = GlobalHotKeyManager::new().unwrap();
        let hotkey = manager.register("cmd+space").unwrap();
        
        loop {
            if let Some(event) = manager.receiver().try_recv() {
                if event.state == HotKeyState::Pressed {
                    let _ = proxy.send_event(LauncherEvent::ToggleVisibility);
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    });
}

fn handle_launcher_events(mut events: EventReader<LauncherEvent>) {
    for event in events.read() {
        match event {
            LauncherEvent::ToggleVisibility => {
                info!("Toggle launcher visibility");
                // Show/hide launcher logic
            }
            _ => {}
        }
    }
}
```

### Option 2: Platform-Specific Integration

**macOS:** Integrate with `NSEvent.addGlobalMonitorForEventsMatchingMask`
**Windows:** Use `SetWindowsHookEx` with `WH_KEYBOARD_LL`
**Linux:** Listen to X11/Wayland global events

### Option 3: Window Focus Management

For simpler cases, combine window focus detection with regular Bevy input:
```rust
fn handle_window_focus_input(
    mut window_query: Query<&mut Window>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut launcher_events: EventWriter<LauncherEvent>,
) {
    // Check if any window is focused
    let has_focus = window_query.iter().any(|window| window.focused);
    
    if !has_focus {
        // Only process global shortcuts when unfocused
        let cmd = keyboard_input.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);
        if cmd && keyboard_input.just_pressed(KeyCode::Space) {
            launcher_events.send(LauncherEvent::ToggleVisibility);
        }
    }
}
```

## Best Practices

### 1. Hotkey Patterns
- **macOS:** `Cmd+Space` (SuperLeft/SuperRight + Space)
- **Windows/Linux:** `Ctrl+Space` or `Alt+Space`
- **Alternative:** `Ctrl+Shift+Space` for less conflicts

### 2. Event Handling Architecture
```rust
// Define clear launcher events
#[derive(Event)]
enum LauncherEvent {
    Show,
    Hide,
    Toggle,
    Execute(ActionItem),
}

// Separate input handling from business logic
fn global_hotkey_system(/* ... */) { /* Send events */ }
fn launcher_visibility_system(/* ... */) { /* Handle events */ }
```

### 3. Performance Considerations
- Use background threads for global hotkey detection
- Minimize polling frequency (10-50ms intervals)
- Cache frequently used resources
- Avoid blocking the main thread

### 4. Error Handling
- Gracefully handle hotkey registration failures
- Provide fallback input methods
- Log global hotkey conflicts
- Allow hotkey customization

## Integration with Current Codebase

The current `handle_keyboard_input` system in `ui/src/ui/systems.rs` already demonstrates good patterns:

```rust
// Current pattern - extends well to global hotkeys
pub fn handle_keyboard_input(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut launcher_events: EventWriter<LauncherEvent>,
    // ... other resources
) {
    // Add global hotkey detection logic here
    // or integrate with custom event system
}
```

**Recommended Integration Steps:**
1. Add `global-hotkey` crate dependency
2. Create custom `LauncherEvent` enum
3. Set up `EventLoopProxyWrapper<LauncherEvent>` 
4. Spawn background thread for global hotkey detection
5. Update existing input systems to handle both local and global events

## Conclusion

Bevy provides excellent foundation for global keystroke handling through:
- **Custom events** for external integration
- **Event-driven architecture** for responsive input handling  
- **Modifier key support** for complex hotkey combinations
- **Thread-safe event injection** via `EventLoopProxyWrapper`

The recommended approach is **Option 1** using external libraries integrated through Bevy's custom event system, providing the best balance of functionality, performance, and cross-platform compatibility.
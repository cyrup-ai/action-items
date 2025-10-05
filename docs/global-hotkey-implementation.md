# Global Hotkey Implementation for Action Items Launcher

## Overview

Based on research from Bevy examples and patterns, this document outlines the implementation of global hotkeys for the Action Items launcher, enabling Raycast/Alfred-style system-wide activation.

## Architecture

### Core Dependencies

```toml
# Cargo.toml
[dependencies]
global-hotkey = "0.4"
bevy = { version = "0.14", features = ["wayland"] }
```

### Key Components

1. **Global Hotkey Manager** - System-wide keystroke detection
2. **Event System** - Clean integration with Bevy's event architecture  
3. **Window Management** - Focus, blur, and visibility handling
4. **Launcher State** - Track activation and focus state

## Implementation Pattern

### 1. Resource Definitions

```rust
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager};
use bevy::prelude::*;

#[derive(Resource)]
pub struct LauncherHotkeys {
    pub toggle: HotKey,
    pub manager: GlobalHotKeyManager,
}

#[derive(Resource, Default)]
pub struct LauncherState {
    pub visible: bool,
    pub has_gained_focus: bool,
    pub last_toggle_time: Option<std::time::Instant>,
}

#[derive(Event)]
pub enum LauncherEvent {
    Show,
    Hide,
    Toggle,
    Search(String),
    Execute(String),
}
```

### 2. Hotkey Setup System

```rust
use global_hotkey::{hotkey::{Code, HotKey, Modifiers}, GlobalHotKeyManager};

fn setup_global_hotkeys(mut commands: Commands) {
    let manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
    
    // Create Cmd+Space hotkey (macOS) / Ctrl+Space (others)
    #[cfg(target_os = "macos")]
    let modifiers = Modifiers::META;
    #[cfg(not(target_os = "macos"))]
    let modifiers = Modifiers::CONTROL;
    
    let toggle_hotkey = HotKey::new(Some(modifiers), Code::Space);
    
    // Register the hotkey
    manager.register(toggle_hotkey.clone())
        .expect("Failed to register global hotkey");
    
    commands.insert_resource(LauncherHotkeys {
        toggle: toggle_hotkey,
        manager,
    });
    
    info!("Global hotkey registered: {}+Space", 
          if cfg!(target_os = "macos") { "Cmd" } else { "Ctrl" });
}
```

### 3. Global Hotkey Polling System

```rust
use global_hotkey::GlobalHotKeyEvent;

fn poll_global_hotkeys(
    hotkeys: Res<LauncherHotkeys>,
    mut hotkey_events: Local<Vec<GlobalHotKeyEvent>>,
    mut launcher_events: EventWriter<LauncherEvent>,
) {
    // Poll for hotkey events (non-blocking)
    hotkey_events.clear();
    if let Ok(events) = GlobalHotKeyEvent::receiver().try_recv() {
        hotkey_events.push(events);
    }
    
    // Additional events may be queued
    while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
        hotkey_events.push(event);
    }
    
    // Process events
    for event in hotkey_events.iter() {
        if event.id == hotkeys.toggle.id() {
            launcher_events.send(LauncherEvent::Toggle);
        }
    }
}
```

### 4. Launcher Event Handler

```rust
fn handle_launcher_events(
    mut events: EventReader<LauncherEvent>,
    mut launcher_state: ResMut<LauncherState>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut ui_state: ResMut<crate::ui::UiState>,
) {
    for event in events.read() {
        match event {
            LauncherEvent::Toggle => {
                launcher_state.visible = !launcher_state.visible;
                launcher_state.last_toggle_time = Some(std::time::Instant::now());
                
                if launcher_state.visible {
                    // Show launcher
                    if let Ok(mut window) = primary_window.get_single_mut() {
                        window.visible = true;
                        window.focused = true;
                    }
                    ui_state.visible = true;
                    launcher_state.has_gained_focus = false;
                } else {
                    // Hide launcher  
                    ui_state.visible = false;
                    if let Ok(mut window) = primary_window.get_single_mut() {
                        window.visible = false;
                    }
                }
            }
            LauncherEvent::Show => {
                launcher_state.visible = true;
                ui_state.visible = true;
                if let Ok(mut window) = primary_window.get_single_mut() {
                    window.visible = true;
                    window.focused = true;
                }
            }
            LauncherEvent::Hide => {
                launcher_state.visible = false;
                ui_state.visible = false;
                if let Ok(mut window) = primary_window.get_single_mut() {
                    window.visible = false;
                }
            }
            _ => {} // Handle other events
        }
    }
}
```

### 5. Window Focus Management

```rust
fn handle_window_blur(
    primary_window: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut launcher_state: ResMut<LauncherState>,
    mut launcher_events: EventWriter<LauncherEvent>,
) {
    if let Ok(window) = primary_window.get_single() {
        // Track when window gains focus
        if window.focused && launcher_state.visible && !launcher_state.has_gained_focus {
            launcher_state.has_gained_focus = true;
            return;
        }
        
        // Hide on blur only if we've gained focus at least once
        if !window.focused 
            && launcher_state.visible 
            && launcher_state.has_gained_focus 
        {
            // Add small delay to prevent immediate hide on hotkey press
            if let Some(last_toggle) = launcher_state.last_toggle_time {
                if last_toggle.elapsed().as_millis() > 100 {
                    launcher_events.send(LauncherEvent::Hide);
                }
            } else {
                launcher_events.send(LauncherEvent::Hide);
            }
        }
    }
}
```

### 6. System Registration

```rust
impl Plugin for GlobalHotkeyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<LauncherEvent>()
            .init_resource::<LauncherState>()
            .add_systems(Startup, setup_global_hotkeys)
            .add_systems(Update, (
                poll_global_hotkeys,
                handle_launcher_events,
                handle_window_blur,
            ).chain());
    }
}
```

## Platform Considerations

### macOS
- Use `Modifiers::META` for Cmd key
- Requires accessibility permissions for global hotkeys
- Consider LSUIElement to hide from dock

### Windows  
- Use `Modifiers::CONTROL` for Ctrl key
- May require elevated permissions
- Handle DPI scaling for window positioning

### Linux
- Use `Modifiers::CONTROL` for Ctrl key  
- X11 vs Wayland considerations
- Different window managers may behave differently

## Security & Permissions

### macOS Permission Request
```rust
#[cfg(target_os = "macos")]
fn request_accessibility_permission() {
    // This would typically show a dialog directing user to System Preferences
    if !global_hotkey::has_accessibility_permission() {
        println!("Please grant accessibility permissions in System Preferences");
        // Could open System Preferences automatically
    }
}
```

## Error Handling

```rust
fn setup_global_hotkeys_safe(mut commands: Commands) {
    match GlobalHotKeyManager::new() {
        Ok(manager) => {
            // Setup hotkeys...
        }
        Err(e) => {
            error!("Failed to create global hotkey manager: {}", e);
            // Fallback: could disable global hotkeys and log warning
        }
    }
}
```

## Performance Considerations

1. **Non-blocking Polling**: Use `try_recv()` to avoid blocking the main thread
2. **Event Batching**: Process multiple hotkey events per frame if queued
3. **Debouncing**: Prevent rapid toggle operations with timing checks
4. **Resource Cleanup**: Properly unregister hotkeys on app exit

## Testing Strategy

1. **Unit Tests**: Test event generation and handling logic
2. **Integration Tests**: Test window focus/blur behavior
3. **Manual Testing**: Test across different platforms and scenarios
4. **Permission Testing**: Verify behavior when permissions denied

## Alternative Approaches

If `global-hotkey` crate has limitations:

1. **tao + global-hotkey**: More direct winit integration
2. **Platform-specific**: Use platform APIs directly (CGEventTap, RegisterHotKey, etc.)
3. **System tray**: Alternative activation method through system tray
4. **Named pipes**: External hotkey handler process

## Implementation Priority

1. ✅ Research global hotkey patterns (DONE)
2. 🔄 Implement basic hotkey registration and polling
3. ⏳ Add window management and focus handling  
4. ⏳ Test across platforms (macOS, Windows, Linux)
5. ⏳ Add permission handling and error recovery
6. ⏳ Performance optimization and debouncing

This implementation provides a robust foundation for Raycast/Alfred-style global activation while maintaining clean integration with Bevy's architecture.
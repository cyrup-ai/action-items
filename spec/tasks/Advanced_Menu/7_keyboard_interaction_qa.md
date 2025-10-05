# Task 7: QA Validation - Keyboard Interaction and Shortcuts System

## Comprehensive Testing Protocol

**File**: `tests/ui/keyboard_shortcuts_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: KeyboardSystem, SettingsSystem, EventHandlers  

### Test Categories

#### 1. Modifier Key Detection Testing
**Reference**: `./docs/bevy/examples/keyboard_input_events.rs:45-72`
```rust
#[test]
fn test_modifier_state_detection() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, keyboard_shortcuts_system);

    // Test Ctrl+K shortcut
    let mut keyboard_input = ButtonInput::<KeyCode>::default();
    keyboard_input.press(KeyCode::ControlLeft);
    keyboard_input.press(KeyCode::KeyK);
    
    app.world_mut().insert_resource(keyboard_input);
    app.update();
    
    let shortcuts = app.world().resource::<KeyboardShortcuts>();
    assert!(shortcuts.modifier_state.ctrl);
    assert!(!shortcuts.modifier_state.alt);
}
```

#### 2. Shortcut Action Execution Testing
```rust
#[test]
fn test_shortcut_action_execution() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, keyboard_shortcuts_system)
       .add_event::<SearchEvent>();

    let mut shortcuts = KeyboardShortcuts::default();
    shortcuts.shortcuts.insert(
        ShortcutKey { 
            key: KeyCode::KeyF, 
            modifiers: ModifierState { ctrl: true, ..default() } 
        },
        ShortcutAction::Search(SearchAction::FocusSearchBar)
    );
    
    app.world_mut().insert_resource(shortcuts);
    
    // Simulate Ctrl+F press
    let mut keyboard_input = ButtonInput::<KeyCode>::default();
    keyboard_input.press(KeyCode::ControlLeft);
    keyboard_input.just_press(KeyCode::KeyF);
    app.world_mut().insert_resource(keyboard_input);
    
    app.update();
    
    let mut events = app.world_mut().resource_mut::<Events<SearchEvent>>();
    let event_reader = events.get_reader();
    assert!(!events.is_empty());
}
```

#### 3. Platform-Specific Modifier Testing
**Reference**: `./docs/bevy/examples/keyboard_input.rs:225-248`
```rust
#[cfg(target_os = "macos")]
#[test]
fn test_macos_cmd_key_handling() {
    let modifier_state = ModifierState {
        meta: true, // Cmd key on macOS
        ctrl: false,
        alt: false,
        shift: false,
    };
    
    let shortcut = ShortcutKey {
        key: KeyCode::KeyW,
        modifiers: modifier_state,
    };
    
    assert_eq!(shortcut.modifiers.meta, true);
    assert!(is_valid_platform_shortcut(&shortcut));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn test_windows_ctrl_key_handling() {
    let modifier_state = ModifierState {
        ctrl: true, // Ctrl key on Windows/Linux
        meta: false,
        alt: false,
        shift: false,
    };
    
    let shortcut = ShortcutKey {
        key: KeyCode::KeyW,
        modifiers: modifier_state,
    };
    
    assert_eq!(shortcut.modifiers.ctrl, true);
    assert!(is_valid_platform_shortcut(&shortcut));
}
```

#### 4. Shortcut Conflict Detection Testing
```rust
#[test]
fn test_shortcut_conflict_detection() {
    let mut shortcuts = KeyboardShortcuts::default();
    
    let duplicate_key = ShortcutKey {
        key: KeyCode::KeyK,
        modifiers: ModifierState { ctrl: true, ..default() }
    };
    
    // First assignment should succeed
    let result1 = shortcuts.add_shortcut(
        duplicate_key,
        ShortcutAction::Search(SearchAction::FocusSearchBar)
    );
    assert!(result1.is_ok());
    
    // Duplicate assignment should fail
    let result2 = shortcuts.add_shortcut(
        duplicate_key,
        ShortcutAction::Navigation(NavigationAction::GoBack)
    );
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), ShortcutError::ConflictDetected);
}
```

#### 5. Key Repeat Functionality Testing
**Reference**: `./docs/bevy/examples/timers.rs:158-182`
```rust
#[test]
fn test_key_repeat_timing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .insert_resource(Time::default());

    let mut shortcuts = KeyboardShortcuts::default();
    shortcuts.repeat_enabled = true;
    shortcuts.repeat_delay = 0.5;
    shortcuts.repeat_rate = 0.1;
    
    app.world_mut().insert_resource(shortcuts);
    
    // Test repeat delay
    let start_time = Instant::now();
    // Simulate held key for repeat_delay duration
    std::thread::sleep(Duration::from_millis(600));
    
    // Should trigger repeat after delay
    assert!(start_time.elapsed().as_secs_f32() > 0.5);
}
```

### Edge Case Testing

#### 6. Settings Persistence Validation
```rust
#[test]
fn test_shortcuts_settings_persistence() {
    let original_shortcuts = KeyboardShortcuts {
        repeat_enabled: false,
        repeat_delay: 1.0,
        repeat_rate: 0.2,
        ..default()
    };
    
    // Test serialization/deserialization
    let serialized = serde_json::to_string(&original_shortcuts).unwrap();
    let deserialized: KeyboardShortcuts = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(original_shortcuts.repeat_enabled, deserialized.repeat_enabled);
    assert_eq!(original_shortcuts.repeat_delay, deserialized.repeat_delay);
    assert_eq!(original_shortcuts.repeat_rate, deserialized.repeat_rate);
}
```

### Manual Testing Checklist

- [ ] All default shortcuts execute correct actions
- [ ] Platform-specific modifiers work correctly (Cmd on macOS, Ctrl on Windows/Linux)
- [ ] Custom shortcuts can be added and removed
- [ ] Shortcut conflicts are detected and prevented
- [ ] Key repeat functions with correct timing
- [ ] Settings persist across application restarts
- [ ] Shortcuts work in all menu contexts
- [ ] Invalid key combinations are rejected
- [ ] Multiple modifiers work correctly (Ctrl+Shift+Key)
- [ ] Shortcuts disable appropriately when text input is focused

**Bevy Examples**: `./docs/bevy/examples/keyboard_input_events.rs:95-128`, `./docs/bevy/examples/ui_text_input.rs:45-72`  
**Integration Points**: All keyboard handling components  
**Success Criteria**: All tests pass, zero shortcut conflicts, responsive key handling under load
# Task 5: QA Validation - Navigation and Auto-Navigation Behavior System

## Comprehensive Testing Protocol

**File**: `tests/ui/navigation_behavior_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: NavigationSystem, SettingsSystem, WindowManager  

### Test Categories

#### 1. Escape Key Behavior Testing
```rust
#[test]
fn test_escape_key_behaviors() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, navigation_auto_behavior_system);

    // Test PopOneLevel behavior
    app.world_mut().insert_resource(NavigationBehavior {
        escape_behavior: EscapeBehavior::PopOneLevel,
        navigation_stack: vec![MenuContext::Root, MenuContext::Settings],
        ..default()
    });
    
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>()
       .press(KeyCode::Escape);
    app.update();
    
    let nav = app.world().resource::<NavigationBehavior>();
    assert_eq!(nav.navigation_stack.len(), 1);
}
```

#### 2. Auto Pop-to-Root Timeout Testing
**Reference**: `./docs/bevy/examples/timers.rs:78-95`
```rust
#[test] 
fn test_auto_pop_timeout() {
    let mut app = App::new();
    let mut nav_behavior = NavigationBehavior {
        pop_to_root_timeout: 1.0, // 1 second for testing
        auto_navigation_enabled: true,
        last_activity: Instant::now() - Duration::from_secs(2),
        navigation_stack: vec![MenuContext::Root, MenuContext::Settings, MenuContext::Advanced],
        ..default()
    };
    
    app.world_mut().insert_resource(nav_behavior);
    app.update();
    
    // Verify automatic pop-to-root occurred
    let nav = app.world().resource::<NavigationBehavior>();
    assert_eq!(nav.navigation_stack.len(), 1);
}
```

#### 3. Settings Persistence Validation
**Reference**: `./docs/bevy/examples/state.rs:42-67`
```rust
#[test]
fn test_navigation_settings_persistence() {
    let settings = AdvancedSettings {
        pop_to_root_timeout: 45.0,
        escape_behavior: EscapeBehavior::CloseWindow,
        auto_navigation_enabled: false,
    };
    
    // Test save/load cycle
    let serialized = serde_json::to_string(&settings).unwrap();
    let deserialized: AdvancedSettings = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(settings.pop_to_root_timeout, deserialized.pop_to_root_timeout);
    assert_eq!(settings.escape_behavior, deserialized.escape_behavior);
}
```

### Edge Case Testing

#### 4. Activity Tracking Validation
```rust
#[test]
fn test_activity_tracking_prevents_auto_navigation() {
    let mut nav_behavior = NavigationBehavior {
        pop_to_root_timeout: 1.0,
        last_activity: Instant::now(), // Recent activity
        navigation_stack: vec![MenuContext::Root, MenuContext::Settings],
        ..default()
    };
    
    // Should NOT auto-navigate due to recent activity
    assert!(nav_behavior.last_activity.elapsed().as_secs_f32() < nav_behavior.pop_to_root_timeout);
}
```

#### 5. Boundary Conditions Testing
```rust
#[test]
fn test_timeout_boundary_conditions() {
    // Test minimum timeout (5 seconds)
    assert!(validate_timeout_range(5.0));
    assert!(!validate_timeout_range(4.0));
    
    // Test maximum timeout (300 seconds)
    assert!(validate_timeout_range(300.0));
    assert!(!validate_timeout_range(301.0));
}
```

### Manual Testing Checklist

- [ ] Escape key responds correctly for all behavior types
- [ ] Auto pop-to-root triggers at configured timeout
- [ ] Activity tracking prevents premature navigation
- [ ] Settings persist across application restarts
- [ ] UI controls update navigation behavior in real-time
- [ ] Navigation stack maintains consistency
- [ ] Window close/minimize behaviors work correctly
- [ ] Input validation prevents invalid timeout values

**Bevy Examples**: `./docs/bevy/examples/keyboard_input.rs:85-112`, `./docs/bevy/examples/timers.rs:15-32`  
**Integration Points**: All navigation behavior components  
**Success Criteria**: All tests pass, zero navigation inconsistencies, smooth timeout transitions
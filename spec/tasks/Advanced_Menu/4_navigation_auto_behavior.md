# Task 4: Navigation and Auto-Navigation Behavior System

## Implementation Details

**File**: `ui/src/ui/navigation.rs`  
**Lines**: 85-140  
**Architecture**: Navigation state machine with timer-based auto-behavior  
**Integration**: SettingsSystem, InputSystem, WindowManager  

### Core Implementation

```rust
#[derive(Resource, Default)]
pub struct NavigationBehavior {
    pub pop_to_root_timeout: f32,
    pub escape_behavior: EscapeBehavior,
    pub auto_navigation_enabled: bool,
    pub last_activity: Instant,
    pub navigation_stack: Vec<MenuContext>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EscapeBehavior {
    PopOneLevel,
    PopToRoot,
    CloseWindow,
    MinimizeWindow,
}

pub fn navigation_auto_behavior_system(
    mut nav_behavior: ResMut<NavigationBehavior>,
    mut window_events: EventWriter<WindowCloseRequest>,
    mut menu_events: EventWriter<MenuNavigationEvent>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Handle escape key behavior
    if keyboard.just_pressed(KeyCode::Escape) {
        match nav_behavior.escape_behavior {
            EscapeBehavior::PopOneLevel => {
                menu_events.send(MenuNavigationEvent::PopLevel);
            }
            EscapeBehavior::PopToRoot => {
                menu_events.send(MenuNavigationEvent::PopToRoot);
            }
            EscapeBehavior::CloseWindow => {
                window_events.send(WindowCloseRequest);
            }
            EscapeBehavior::MinimizeWindow => {
                window_events.send(WindowMinimizeRequest);
            }
        }
        nav_behavior.last_activity = Instant::now();
    }

    // Auto pop-to-root on timeout
    if nav_behavior.auto_navigation_enabled 
        && nav_behavior.navigation_stack.len() > 1 
        && nav_behavior.last_activity.elapsed().as_secs_f32() > nav_behavior.pop_to_root_timeout {
        menu_events.send(MenuNavigationEvent::PopToRoot);
        nav_behavior.last_activity = Instant::now();
    }
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui_buttons.rs:45-78`

```rust
// Pop to Root Search timeout setting
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
    ..default()
},
children: &[
    (TextBundle::from_section(
        "Pop to Root Search (seconds)",
        TextStyle {
            font: asset_server.load("fonts/Inter-Medium.ttf"),
            font_size: 14.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    ),),
    (NumericInputBundle {
        value: 30.0,
        min_value: 5.0,
        max_value: 300.0,
        step: 5.0,
        width: 80.0,
        ..default()
    },),
]
```

### Architecture Notes

- Navigation behavior managed through centralized state machine
- Timer-based auto-navigation with configurable timeouts
- Escape key behavior configurable per user preference
- Activity tracking prevents unexpected navigation during active use
- Integration with window management for close/minimize options

**Bevy Examples**: `./docs/bevy/examples/timers.rs:22-45`, `./docs/bevy/examples/keyboard_input.rs:15-38`  
**Integration Points**: MenuSystem, WindowManager, SettingsSystem  
**Dependencies**: SettingsResource, InputResource, TimeResource
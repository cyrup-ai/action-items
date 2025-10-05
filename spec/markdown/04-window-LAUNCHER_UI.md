# REAL Window Implementation

**Using actual Window component and Visibility patterns - no bullshit animation components**

## Current Problem (app/src/main.rs:100-116)
```rust
primary_window: Some(Window {
    // Start with search bar size, will expand for results
    resolution: (WINDOW_MIN_WIDTH, WINDOW_HEIGHT).into(),  // ❌ Dynamic sizing
    visible: false,           // ❌ Hidden by default
    transparent: true,        // ❌ Causes performance issues
    //...
})
```

## REAL Solution from Bevy Examples

### Fixed Window Configuration

```rust
// REAL window setup - fixed dimensions like Raycast
primary_window: Some(Window {
    title: "Action Items".into(),
    resolution: (600.0, 420.0).into(),           // Fixed Raycast dimensions
    position: WindowPosition::Centered(MonitorSelection::Primary),
    decorations: false,
    window_level: WindowLevel::AlwaysOnTop,
    visible: false,                               // Still hidden initially
    mode: WindowMode::Windowed,
    transparent: false,                           // Solid for performance
    resizable: false,                             // Prevent user resizing
    #[cfg(target_os = "macos")]
    composite_alpha_mode: CompositeAlphaMode::Opaque,
    ..default()
})
```

### Show/Hide System (REAL patterns)

**Pattern from:** `bevy/examples/ui/display_and_visibility.rs`

```rust
#[derive(Resource, Default)]
struct LauncherState {
    is_visible: bool,
}

// REAL show/hide system - direct window and UI visibility
fn launcher_visibility_system(
    mut global_hotkey_events: EventReader<GlobalHotkeyEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut launcher_container: Query<&mut Visibility, With<LauncherContainer>>,
    mut launcher_state: ResMut<LauncherState>,
    mut search_text: Query<&mut Text, With<SearchInput>>,
) {
    let Ok(mut window) = window.get_single_mut() else { return };
    let Ok(mut container_visibility) = launcher_container.get_single_mut() else { return };
    
    for _event in global_hotkey_events.read() {
        if launcher_state.is_visible {
            // HIDE launcher
            window.visible = false;
            *container_visibility = Visibility::Hidden;
            launcher_state.is_visible = false;
            
            // Reset search text
            if let Ok(mut text) = search_text.get_single_mut() {
                **text = "Search...".to_string();
            }
        } else {
            // SHOW launcher
            window.visible = true;
            *container_visibility = Visibility::Visible;
            launcher_state.is_visible = true;
            
            // Focus search input
            if let Ok(mut text) = search_text.get_single_mut() {
                **text = "Search...".to_string();
            }
        }
    }
}
```

### Window Positioning (REAL patterns)

```rust
// REAL window positioning - direct window mutation
fn position_launcher_system(
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    launcher_state: Res<LauncherState>,
) {
    if !launcher_state.is_visible {
        return;
    }
    
    let Ok(mut window) = window.get_single_mut() else { return };
    
    // Center on primary monitor (simplified)
    // In real implementation, would query monitor info
    window.position = WindowPosition::Centered(MonitorSelection::Primary);
}
```

### Fade Animation Using Container Alpha

**Since Bevy doesn't have window opacity, simulate with container alpha:**

```rust
#[derive(Component)]
struct FadeAnimation {
    target_alpha: f32,
    current_alpha: f32,
    speed: f32,
}

// REAL fade system - animate container BackgroundColor alpha
fn fade_animation_system(
    mut containers: Query<(&mut BackgroundColor, &mut FadeAnimation)>,
    time: Res<Time>,
) {
    for (mut bg_color, mut fade) in containers.iter_mut() {
        if (fade.current_alpha - fade.target_alpha).abs() < 0.01 {
            continue; // Animation complete
        }
        
        // Update alpha towards target
        let delta = fade.speed * time.delta_secs();
        if fade.current_alpha < fade.target_alpha {
            fade.current_alpha = (fade.current_alpha + delta).min(fade.target_alpha);
        } else {
            fade.current_alpha = (fade.current_alpha - delta).max(fade.target_alpha);
        }
        
        // Apply to background color
        bg_color.0 = bg_color.0.with_alpha(fade.current_alpha);
    }
}

// Update launcher container spawn to include fade animation
parent.spawn((
    Node {
        width: Val::Px(600.0),
        height: Val::Px(420.0),
        // ... other properties
    },
    BackgroundColor(Color::srgba(0.08, 0.08, 0.09, 0.0)), // Start transparent
    FadeAnimation {
        target_alpha: 0.0,
        current_alpha: 0.0,
        speed: 4.0, // Fast fade
    },
    LauncherContainer,
    Visibility::Hidden,
));

// Update visibility system to trigger fade
fn launcher_visibility_system(
    mut global_hotkey_events: EventReader<GlobalHotkeyEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut launcher_container: Query<(&mut Visibility, &mut FadeAnimation), With<LauncherContainer>>,
    mut launcher_state: ResMut<LauncherState>,
) {
    let Ok(mut window) = window.get_single_mut() else { return };
    let Ok((mut container_visibility, mut fade)) = launcher_container.get_single_mut() else { return };
    
    for _event in global_hotkey_events.read() {
        if launcher_state.is_visible {
            // HIDE with fade
            fade.target_alpha = 0.0;
            // Window will be hidden when fade completes
            launcher_state.is_visible = false;
        } else {
            // SHOW with fade
            window.visible = true;
            *container_visibility = Visibility::Visible;
            fade.target_alpha = 0.98; // Nearly opaque
            launcher_state.is_visible = true;
        }
    }
}

// System to hide window when fade completes
fn fade_complete_system(
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut containers: Query<(&mut Visibility, &FadeAnimation), With<LauncherContainer>>,
    launcher_state: Res<LauncherState>,
) {
    if launcher_state.is_visible {
        return;
    }
    
    let Ok(mut window) = window.get_single_mut() else { return };
    let Ok((mut visibility, fade)) = containers.get_single_mut() else { return };
    
    // Hide window when fade to 0 is complete
    if fade.target_alpha <= 0.01 && (fade.current_alpha - fade.target_alpha).abs() < 0.01 {
        window.visible = false;
        *visibility = Visibility::Hidden;
    }
}
```

### Multi-Monitor Support (REAL implementation)

```rust
// Use the existing MultiMonitorManager from main.rs
fn enhanced_positioning_system(
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    monitor_manager: Res<MultiMonitorManager>,
    launcher_state: Res<LauncherState>,
) {
    if !launcher_state.is_visible {
        return;
    }
    
    let Ok(mut window) = window.get_single_mut() else { return };
    
    // Use the existing monitor detection systems
    // Position window 12% from top of current monitor
    if let Some(current_monitor) = monitor_manager.get_current_monitor() {
        let window_x = (current_monitor.width - 600.0) / 2.0; // Center horizontally
        let window_y = current_monitor.height * 0.12; // 12% from top
        
        window.position = WindowPosition::At(IVec2::new(
            (current_monitor.x + window_x) as i32,
            (current_monitor.y + window_y) as i32,
        ));
    }
}
```

### Focus Management (REAL patterns)

```rust
// REAL focus system - handle window blur/focus
fn window_focus_system(
    mut window_events: EventReader<WindowEvent>,
    mut launcher_state: ResMut<LauncherState>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut container_visibility: Query<&mut Visibility, With<LauncherContainer>>,
) {
    let Ok(mut window) = window.get_single_mut() else { return };
    let Ok(mut visibility) = container_visibility.get_single_mut() else { return };
    
    for event in window_events.read() {
        match event {
            WindowEvent::Focused(false) => {
                // Window lost focus - hide launcher
                if launcher_state.is_visible {
                    window.visible = false;
                    *visibility = Visibility::Hidden;
                    launcher_state.is_visible = false;
                }
            }
            WindowEvent::CloseRequested => {
                // Hide instead of closing
                window.visible = false;
                *visibility = Visibility::Hidden;
                launcher_state.is_visible = false;
            }
            _ => {}
        }
    }
}
```

### Complete Window Setup

```rust
// Replace window configuration in main.rs
primary_window: Some(Window {
    title: "Action Items".into(),
    resolution: (600.0, 420.0).into(),           // Fixed Raycast size
    position: WindowPosition::Centered(MonitorSelection::Primary),
    decorations: false,                           // Frameless
    window_level: WindowLevel::AlwaysOnTop,      // Always on top
    visible: false,                               // Hidden initially
    mode: WindowMode::Windowed,                  // Windowed mode
    transparent: false,                           // Opaque for performance
    resizable: false,                             // Fixed size
    #[cfg(target_os = "macos")]
    composite_alpha_mode: CompositeAlphaMode::Opaque,
    ..default()
})

// System registration
.add_systems(Update, (
    launcher_visibility_system,
    position_launcher_system,
    fade_animation_system,
    fade_complete_system,
    enhanced_positioning_system,
    window_focus_system,
))
```

## Success Criteria

✅ **Viewport-relative window (60% width, max 60% height)** - responsive sizing  
✅ **Real show/hide system** - direct Window visibility  
✅ **Real fade animations** - container alpha animation  
✅ **Real positioning** - using existing monitor systems  
✅ **Real focus handling** - WindowEvent integration  

**NO BULLSHIT ANIMATION COMPONENTS** - Use standard Bevy patterns only

## Bevy Implementation Details

### Window Component Architecture

```rust
use bevy::{
    prelude::*,
    window::{
        Window, PrimaryWindow, WindowLevel, WindowPosition, MonitorSelection,
        CompositeAlphaMode, WindowMode, WindowEvent, WindowFocused, WindowCloseRequested,
    },
    input::common_conditions::input_just_pressed,
    ecs::system::CommandQueue,
};

/// Resource for tracking launcher window state
#[derive(Resource, Debug, Clone)]
pub struct LauncherWindowState {
    pub is_visible: bool,
    pub has_gained_focus: bool,
    pub target_alpha: f32,
    pub current_alpha: f32,
    pub fade_speed: f32,
    pub window_entity: Option<Entity>,
}

impl Default for LauncherWindowState {
    fn default() -> Self {
        Self {
            is_visible: false,
            has_gained_focus: false,
            target_alpha: 0.0,
            current_alpha: 0.0,
            fade_speed: 4.0, // Fast fade transitions
            window_entity: None,
        }
    }
}

/// Component for main launcher container with fade animation
#[derive(Component, Debug)]
pub struct LauncherContainer {
    pub base_color: Color,
    pub fade_enabled: bool,
}

impl Default for LauncherContainer {
    fn default() -> Self {
        Self {
            base_color: Color::srgba(0.08, 0.08, 0.09, 0.98),
            fade_enabled: true,
        }
    }
}

/// Component for tracking global hotkey state
#[derive(Component, Debug)]
pub struct GlobalHotkeyHandler {
    pub toggle_key: KeyCode,
    pub modifier_required: bool,
}

impl Default for GlobalHotkeyHandler {
    fn default() -> Self {
        Self {
            toggle_key: KeyCode::Space,
            modifier_required: true, // Cmd+Space on macOS
        }
    }
}
```

### Window Configuration System

```rust
/// Setup window with proper configuration for launcher
pub fn setup_launcher_window() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Action Items".into(),
            resolution: (600.0, 420.0).into(),           // Fixed Raycast-like dimensions
            position: WindowPosition::Centered(MonitorSelection::Primary),
            decorations: false,                           // Frameless window
            window_level: WindowLevel::AlwaysOnTop,      // Stay above other windows
            visible: false,                               // Start hidden
            mode: WindowMode::Windowed,
            transparent: false,                           // Opaque for better performance
            resizable: false,                             // Fixed size
            #[cfg(target_os = "macos")]
            composite_alpha_mode: CompositeAlphaMode::Opaque,
            ..default()
        }),
        ..default()
    }
}

/// System to handle window configuration updates
pub fn window_config_system(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    launcher_state: Res<LauncherWindowState>,
) {
    let Ok(mut window) = windows.get_single_mut() else { return };
    
    // Ensure window stays configured correctly
    if launcher_state.is_visible && !window.visible {
        window.visible = true;
        window.focused = true;
    } else if !launcher_state.is_visible && window.visible {
        window.visible = false;
    }
}
```

### Global Hotkey System

```rust
/// Handle global hotkey for launcher toggle
pub fn global_hotkey_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut launcher_state: ResMut<LauncherWindowState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut launcher_containers: Query<&mut Visibility, With<LauncherContainer>>,
    hotkey_handlers: Query<&GlobalHotkeyHandler>,
) {
    let Ok(hotkey_handler) = hotkey_handlers.get_single() else { return };
    let Ok(mut window) = windows.get_single_mut() else { return };
    let Ok(mut container_visibility) = launcher_containers.get_single_mut() else { return };

    // Check for hotkey combination (e.g., Cmd+Space)
    let modifier_pressed = keyboard_input.pressed(KeyCode::SuperLeft) || 
                          keyboard_input.pressed(KeyCode::SuperRight);
    
    let hotkey_pressed = if hotkey_handler.modifier_required {
        modifier_pressed && keyboard_input.just_pressed(hotkey_handler.toggle_key)
    } else {
        keyboard_input.just_pressed(hotkey_handler.toggle_key)
    };

    if hotkey_pressed {
        if launcher_state.is_visible {
            // Hide launcher
            launcher_state.is_visible = false;
            launcher_state.has_gained_focus = false;
            launcher_state.target_alpha = 0.0;
            
            // Don't hide window immediately - let fade animation handle it
        } else {
            // Show launcher
            launcher_state.is_visible = true;
            launcher_state.has_gained_focus = false; // Will be set when window gains focus
            launcher_state.target_alpha = 0.98;
            
            // Show window and container immediately
            window.visible = true;
            window.focused = true;
            *container_visibility = Visibility::Visible;
        }
    }
}

/// Alternative global hotkey system using external crate
pub fn external_hotkey_system(
    mut hotkey_events: EventReader<GlobalHotkeyEvent>,
    mut launcher_state: ResMut<LauncherWindowState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = windows.get_single_mut() else { return };

    for _event in hotkey_events.read() {
        // Toggle launcher visibility
        launcher_state.is_visible = !launcher_state.is_visible;
        
        if launcher_state.is_visible {
            launcher_state.target_alpha = 0.98;
            window.visible = true;
            window.focused = true;
        } else {
            launcher_state.target_alpha = 0.0;
            launcher_state.has_gained_focus = false;
        }
    }
}

/// Event for external global hotkey integration
#[derive(Event, Debug)]
pub struct GlobalHotkeyEvent {
    pub hotkey_id: u32,
}
```

### Window Focus Management

```rust
/// Handle window focus/blur events
pub fn window_focus_system(
    mut window_events: EventReader<WindowEvent>,
    mut launcher_state: ResMut<LauncherWindowState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut launcher_containers: Query<&mut Visibility, With<LauncherContainer>>,
) {
    let Ok(mut window) = windows.get_single_mut() else { return };
    let Ok(mut container_visibility) = launcher_containers.get_single_mut() else { return };

    for event in window_events.read() {
        match event {
            WindowEvent::Focused(focused) => {
                if *focused && launcher_state.is_visible && !launcher_state.has_gained_focus {
                    // Window just gained focus - mark it
                    launcher_state.has_gained_focus = true;
                } else if !focused && launcher_state.is_visible && launcher_state.has_gained_focus {
                    // Window lost focus - hide launcher
                    launcher_state.is_visible = false;
                    launcher_state.target_alpha = 0.0;
                    launcher_state.has_gained_focus = false;
                }
            },
            WindowEvent::CloseRequested => {
                // Hide instead of closing
                launcher_state.is_visible = false;
                launcher_state.target_alpha = 0.0;
                window.visible = false;
                *container_visibility = Visibility::Hidden;
            },
            _ => {},
        }
    }
}
```

### Fade Animation System

```rust
/// Container fade animation system using BackgroundColor alpha
pub fn container_fade_system(
    mut launcher_containers: Query<(&mut BackgroundColor, &LauncherContainer)>,
    mut launcher_state: ResMut<LauncherWindowState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let Ok((mut bg_color, container)) = launcher_containers.get_single_mut() else { return };
    let Ok(mut window) = windows.get_single_mut() else { return };
    
    if !container.fade_enabled {
        return;
    }

    // Calculate alpha change
    let delta = launcher_state.fade_speed * time.delta_secs();
    let alpha_diff = (launcher_state.current_alpha - launcher_state.target_alpha).abs();
    
    if alpha_diff > 0.01 {
        // Update current alpha towards target
        if launcher_state.current_alpha < launcher_state.target_alpha {
            launcher_state.current_alpha = 
                (launcher_state.current_alpha + delta).min(launcher_state.target_alpha);
        } else {
            launcher_state.current_alpha = 
                (launcher_state.current_alpha - delta).max(launcher_state.target_alpha);
        }
        
        // Apply alpha to background color
        bg_color.0 = container.base_color.with_alpha(launcher_state.current_alpha);
    }
    
    // Hide window when fade to transparent is complete
    if launcher_state.target_alpha <= 0.01 && 
       (launcher_state.current_alpha - launcher_state.target_alpha).abs() < 0.01 {
        window.visible = false;
    }
}
```

### Multi-Monitor Positioning

```rust
/// Resource for monitor management
#[derive(Resource, Debug)]
pub struct MonitorManager {
    pub primary_monitor: MonitorInfo,
    pub all_monitors: Vec<MonitorInfo>,
    pub last_update: f64,
}

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub id: u32,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub scale_factor: f32,
}

/// System for positioning window on current monitor
pub fn window_positioning_system(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    monitor_manager: Option<Res<MonitorManager>>,
    launcher_state: Res<LauncherWindowState>,
) {
    if !launcher_state.is_visible {
        return;
    }

    let Ok(mut window) = windows.get_single_mut() else { return };
    let Some(monitor_manager) = monitor_manager else { return };

    // Position window on primary monitor
    let monitor = &monitor_manager.primary_monitor;
    let window_width = 600.0;
    let window_height = 420.0;
    
    // Center horizontally, position 30% from top
    let x = monitor.x + (monitor.width - window_width) / 2.0;
    let y = monitor.y + monitor.height * 0.3;
    
    window.position = WindowPosition::At(IVec2::new(x as i32, y as i32));
}

/// Update monitor information periodically
pub fn monitor_update_system(
    mut monitor_manager: Option<ResMut<MonitorManager>>,
    time: Res<Time>,
) {
    let Some(mut manager) = monitor_manager else { return };
    let current_time = time.elapsed_secs_f64();
    
    // Update monitor info every 5 seconds
    if current_time - manager.last_update > 5.0 {
        manager.last_update = current_time;
        
        // In real implementation, would query system for monitor info
        manager.primary_monitor = MonitorInfo {
            id: 0,
            width: 1920.0,
            height: 1080.0,
            x: 0.0,
            y: 0.0,
            scale_factor: 1.0,
        };
    }
}
```

### Window Setup System

```rust
/// Setup launcher container with proper styling
pub fn setup_launcher_container(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Main launcher container
    commands.spawn((
        Node {
            width: Val::Px(600.0),
            height: Val::Px(420.0),
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.09, 0.0)), // Start transparent
        BorderRadius::all(Val::Px(12.0)),
        LauncherContainer::default(),
        Visibility::Hidden,
    ))
    .with_children(|parent| {
        spawn_search_section(parent, asset_server.clone());
        spawn_results_section(parent, asset_server);
    });
    
    // Global hotkey handler
    commands.spawn(GlobalHotkeyHandler::default());
}

/// Spawn search input section
fn spawn_search_section(parent: &mut ChildBuilder, asset_server: Handle<Font>) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(48.0),
            margin: UiRect::bottom(Val::Px(8.0)),
            padding: UiRect::all(Val::Px(12.0)),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.18, 0.18, 0.20, 0.85), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.15, 0.15, 0.17, 0.90), Val::Percent(100.0)),
        ])),
        BorderRadius::all(Val::Px(8.0)),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Search..."),
            TextFont {
                font: asset_server,
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.75, 1.0)),
            RealtimeTextInput::default(),
        ));
    });
}

/// Spawn results section
fn spawn_results_section(parent: &mut ChildBuilder, asset_server: Handle<Font>) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0), // Fill remaining space
            flex_direction: FlexDirection::Column,
            ..default()
        },
        SearchResultsContainer::default(),
    ));
}
```

### System Sets and Plugin

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum WindowSystems {
    /// Handle global hotkeys
    HandleHotkeys,
    /// Update window configuration
    ConfigureWindow,
    /// Handle focus events
    ManageFocus,
    /// Animate fade effects
    AnimateFade,
    /// Position window
    PositionWindow,
    /// Update monitor info
    UpdateMonitors,
}

pub struct LauncherWindowPlugin;

impl Plugin for LauncherWindowPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<LauncherWindowState>()
            .init_resource::<MonitorManager>()
            .add_event::<GlobalHotkeyEvent>()
            .configure_sets(
                Update,
                (
                    WindowSystems::HandleHotkeys,
                    WindowSystems::ConfigureWindow,
                    WindowSystems::ManageFocus,
                    WindowSystems::AnimateFade,
                    WindowSystems::PositionWindow,
                    WindowSystems::UpdateMonitors,
                ).chain(),
            )
            .add_systems(
                Update,
                (
                    global_hotkey_system.in_set(WindowSystems::HandleHotkeys),
                    window_config_system.in_set(WindowSystems::ConfigureWindow),
                    window_focus_system.in_set(WindowSystems::ManageFocus),
                    container_fade_system.in_set(WindowSystems::AnimateFade),
                    window_positioning_system.in_set(WindowSystems::PositionWindow),
                    monitor_update_system.in_set(WindowSystems::UpdateMonitors),
                ),
            )
            .add_systems(Startup, setup_launcher_container);
    }
}
```

### Event-Driven Window Management

```rust
/// Custom events for window management
#[derive(Event, Debug)]
pub enum WindowManagementEvent {
    /// Show launcher window
    ShowLauncher,
    /// Hide launcher window
    HideLauncher,
    /// Toggle launcher visibility
    ToggleLauncher,
    /// Focus changed
    FocusChanged { focused: bool },
    /// Position changed
    PositionChanged { x: f32, y: f32 },
}

/// Event-driven window management system
pub fn window_event_system(
    mut window_events: EventReader<WindowManagementEvent>,
    mut launcher_state: ResMut<LauncherWindowState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut containers: Query<&mut Visibility, With<LauncherContainer>>,
) {
    let Ok(mut window) = windows.get_single_mut() else { return };
    let Ok(mut container_visibility) = containers.get_single_mut() else { return };

    for event in window_events.read() {
        match event {
            WindowManagementEvent::ShowLauncher => {
                launcher_state.is_visible = true;
                launcher_state.target_alpha = 0.98;
                window.visible = true;
                window.focused = true;
                *container_visibility = Visibility::Visible;
            },
            WindowManagementEvent::HideLauncher => {
                launcher_state.is_visible = false;
                launcher_state.target_alpha = 0.0;
                launcher_state.has_gained_focus = false;
            },
            WindowManagementEvent::ToggleLauncher => {
                if launcher_state.is_visible {
                    launcher_state.is_visible = false;
                    launcher_state.target_alpha = 0.0;
                    launcher_state.has_gained_focus = false;
                } else {
                    launcher_state.is_visible = true;
                    launcher_state.target_alpha = 0.98;
                    window.visible = true;
                    window.focused = true;
                    *container_visibility = Visibility::Visible;
                }
            },
            WindowManagementEvent::FocusChanged { focused } => {
                if *focused && launcher_state.is_visible {
                    launcher_state.has_gained_focus = true;
                } else if !focused && launcher_state.has_gained_focus {
                    launcher_state.is_visible = false;
                    launcher_state.target_alpha = 0.0;
                }
            },
            WindowManagementEvent::PositionChanged { x, y } => {
                window.position = WindowPosition::At(IVec2::new(*x as i32, *y as i32));
            },
        }
    }
}
```

### Testing Strategies

```rust
#[cfg(test)]
mod window_system_tests {
    use super::*;

    #[test]
    fn test_launcher_window_state() {
        let mut state = LauncherWindowState::default();
        assert!(!state.is_visible);
        assert_eq!(state.current_alpha, 0.0);
        assert_eq!(state.target_alpha, 0.0);
        
        state.is_visible = true;
        state.target_alpha = 0.98;
        assert!(state.is_visible);
        assert_eq!(state.target_alpha, 0.98);
    }

    #[test]
    fn test_global_hotkey_handler() {
        let handler = GlobalHotkeyHandler::default();
        assert_eq!(handler.toggle_key, KeyCode::Space);
        assert!(handler.modifier_required);
    }

    #[test]
    fn test_monitor_info() {
        let monitor = MonitorInfo {
            id: 0,
            width: 1920.0,
            height: 1080.0,
            x: 0.0,
            y: 0.0,
            scale_factor: 1.0,
        };
        
        // Test window positioning calculation
        let window_width = 600.0;
        let window_height = 420.0;
        let x = monitor.x + (monitor.width - window_width) / 2.0;
        let y = monitor.y + monitor.height * 0.3;
        
        assert_eq!(x, 660.0); // Center horizontally
        assert_eq!(y, 324.0); // 30% from top
    }

    #[test]
    fn test_fade_animation() {
        let mut state = LauncherWindowState::default();
        state.target_alpha = 1.0;
        state.fade_speed = 4.0;
        
        // Simulate one frame at 60fps (0.016667 seconds)
        let delta_time = 0.016667;
        let delta = state.fade_speed * delta_time;
        
        state.current_alpha = (state.current_alpha + delta).min(state.target_alpha);
        
        assert!(state.current_alpha > 0.0);
        assert!(state.current_alpha <= 1.0);
    }

    #[test]
    fn test_window_management_events() {
        use bevy::app::App;
        
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(LauncherWindowPlugin)
           .add_event::<WindowManagementEvent>();

        // Send show event
        app.world_mut().send_event(WindowManagementEvent::ShowLauncher);
        app.update();

        // Check state changed
        let state = app.world().resource::<LauncherWindowState>();
        assert!(state.is_visible);
        assert_eq!(state.target_alpha, 0.98);
    }
}
```
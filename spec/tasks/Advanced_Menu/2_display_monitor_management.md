# Task 2: Display and Multi-Monitor Management System Implementation

## Objective
Implement the display and multi-monitor management system with "Show Raycast on" dropdown configuration, monitor detection, window positioning logic, and adaptive display handling.

## Implementation Details

### Target Files
- `core/src/display/monitor_manager.rs:1-300` - Multi-monitor detection and management
- `ui/src/ui/components/advanced/display_settings.rs:1-200` - Display settings dropdown component
- `core/src/window/positioning.rs:1-250` - Window positioning and display logic
- `core/src/display/hot_plug_handler.rs:1-150` - Dynamic monitor connection handling

### Bevy Implementation Patterns

#### Multi-Monitor Detection System
**Reference**: `./docs/bevy/examples/window/multiple_windows.rs:30-70` - Multi-window and display management
**Reference**: `./docs/bevy/examples/diagnostics/system_information_diagnostics.rs:40-80` - System information access
```rust
// Monitor management resource
#[derive(Resource, Clone, Debug)]
pub struct MonitorManager {
    pub monitors: Vec<MonitorInfo>,
    pub primary_monitor: Option<u32>,
    pub mouse_monitor: Option<u32>,
    pub focused_window_monitor: Option<u32>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub id: u32,
    pub name: String,
    pub resolution: (u32, u32),
    pub scale_factor: f64,
    pub position: (i32, i32),
    pub is_primary: bool,
    pub work_area: Rectangle,
    pub refresh_rate: u32,
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl MonitorManager {
    pub fn new() -> Self {
        Self {
            monitors: Vec::new(),
            primary_monitor: None,
            mouse_monitor: None,
            focused_window_monitor: None,
            last_updated: Utc::now(),
        }
    }
    
    pub fn detect_monitors(&mut self) -> Result<(), DisplayError> {
        self.monitors.clear();
        
        // Platform-specific monitor detection
        #[cfg(target_os = "macos")]
        {
            self.detect_monitors_macos()?;
        }
        
        #[cfg(target_os = "windows")]
        {
            self.detect_monitors_windows()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            self.detect_monitors_linux()?;
        }
        
        self.update_monitor_states()?;
        self.last_updated = Utc::now();
        Ok(())
    }
    
    pub fn get_target_monitor(&self, selection: &ScreenSelection) -> Option<&MonitorInfo> {
        match selection {
            ScreenSelection::MouseScreen => {
                self.mouse_monitor
                    .and_then(|id| self.monitors.iter().find(|m| m.id == id))
            }
            ScreenSelection::PrimaryScreen => {
                self.primary_monitor
                    .and_then(|id| self.monitors.iter().find(|m| m.id == id))
            }
            ScreenSelection::FocusedWindowScreen => {
                self.focused_window_monitor
                    .and_then(|id| self.monitors.iter().find(|m| m.id == id))
            }
            ScreenSelection::SpecificScreen(monitor_id) => {
                self.monitors.iter().find(|m| m.id == *monitor_id)
            }
        }
    }
    
    pub fn calculate_window_position(
        &self,
        selection: &ScreenSelection,
        window_size: (u32, u32),
    ) -> Option<(i32, i32)> {
        if let Some(monitor) = self.get_target_monitor(selection) {
            let center_x = monitor.work_area.x + (monitor.work_area.width as i32 / 2) - (window_size.0 as i32 / 2);
            let center_y = monitor.work_area.y + (monitor.work_area.height as i32 / 3); // Slightly above center
            
            Some((center_x, center_y))
        } else {
            None
        }
    }
    
    #[cfg(target_os = "macos")]
    fn detect_monitors_macos(&mut self) -> Result<(), DisplayError> {
        use core_graphics::display::{CGDisplay, CGDirectDisplayID};
        
        let display_count = CGDisplay::active_displays().map_err(|_| DisplayError::DetectionFailed)?;
        
        for (index, display_id) in display_count.iter().enumerate() {
            let display = CGDisplay::new(*display_id);
            let bounds = display.bounds();
            
            let monitor = MonitorInfo {
                id: index as u32,
                name: format!("Display {}", index + 1),
                resolution: (bounds.size.width as u32, bounds.size.height as u32),
                scale_factor: display.pixels_per_point() as f64,
                position: (bounds.origin.x as i32, bounds.origin.y as i32),
                is_primary: CGDisplay::main().id() == *display_id,
                work_area: Rectangle {
                    x: bounds.origin.x as i32,
                    y: bounds.origin.y as i32,
                    width: bounds.size.width as u32,
                    height: bounds.size.height as u32,
                },
                refresh_rate: 60, // Default, would query actual refresh rate
            };
            
            self.monitors.push(monitor);
            
            if monitor.is_primary {
                self.primary_monitor = Some(monitor.id);
            }
        }
        
        Ok(())
    }
}
```

#### Display Settings Dropdown Component
**Reference**: `./docs/bevy/examples/ui/button.rs:300-340` - Dropdown button with dynamic options
**Reference**: `./docs/bevy/examples/ui/ui.rs:600-650` - Dropdown menu with monitor selection
```rust
// Display settings dropdown component
#[derive(Component)]
pub struct DisplaySettingsDropdown {
    pub current_selection: ScreenSelection,
    pub available_options: Vec<DropdownOption>,
    pub open: bool,
}

#[derive(Debug, Clone)]
pub struct DropdownOption {
    pub value: ScreenSelection,
    pub label: String,
    pub available: bool,
}

// Display settings dropdown UI
ButtonBundle {
    style: Style {
        width: Val::Px(250.0),
        height: Val::Px(32.0),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(12.0)),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    },
    background_color: Color::rgba(0.15, 0.15, 0.15, 1.0).into(),
    border_color: Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(4.0)),
    ..default()
}

// Current selection display
TextBundle::from_section(
    dropdown.current_selection.display_name(),
    TextStyle {
        font: font_regular.clone(),
        font_size: 14.0,
        color: Color::WHITE,
    },
)

// Dropdown arrow
TextBundle::from_section(
    if dropdown.open { "▲" } else { "▼" },
    TextStyle {
        font: font_regular.clone(),
        font_size: 12.0,
        color: Color::rgba(0.6, 0.6, 0.6, 1.0),
    },
)

impl ScreenSelection {
    pub fn display_name(&self) -> String {
        match self {
            ScreenSelection::MouseScreen => "Screen containing mouse".to_string(),
            ScreenSelection::PrimaryScreen => "Primary screen".to_string(),
            ScreenSelection::FocusedWindowScreen => "Screen with focused window".to_string(),
            ScreenSelection::SpecificScreen(id) => format!("Display {}", id + 1),
        }
    }
}
```

#### Mouse Position Tracking System
**Reference**: `./docs/bevy/examples/input/mouse_input.rs:40-80` - Mouse position tracking
```rust
// Mouse monitor tracking system
#[derive(Resource, Default)]
pub struct MouseTracker {
    pub current_position: Option<Vec2>,
    pub current_monitor: Option<u32>,
    pub update_timer: Timer,
}

fn mouse_monitor_tracking_system(
    mut mouse_tracker: ResMut<MouseTracker>,
    mut monitor_manager: ResMut<MonitorManager>,
    mouse_input: Res<Input<MouseButton>>,
    cursor_events: EventReader<CursorMoved>,
    time: Res<Time>,
) {
    // Update timer for periodic monitor checking
    mouse_tracker.update_timer.tick(time.delta());
    
    // Track mouse movement
    for cursor_event in cursor_events.iter() {
        mouse_tracker.current_position = Some(cursor_event.position);
    }
    
    // Periodically update which monitor contains the mouse
    if mouse_tracker.update_timer.just_finished() {
        if let Some(mouse_pos) = mouse_tracker.current_position {
            let screen_coords = convert_to_screen_coordinates(mouse_pos);
            
            for monitor in &monitor_manager.monitors {
                if point_in_monitor(screen_coords, &monitor.work_area) {
                    if mouse_tracker.current_monitor != Some(monitor.id) {
                        mouse_tracker.current_monitor = Some(monitor.id);
                        monitor_manager.mouse_monitor = Some(monitor.id);
                    }
                    break;
                }
            }
        }
    }
}

// Convert Bevy coordinates to screen coordinates
fn convert_to_screen_coordinates(bevy_pos: Vec2) -> (i32, i32) {
    // Implementation depends on platform and window setup
    (bevy_pos.x as i32, bevy_pos.y as i32)
}

// Check if point is within monitor bounds
fn point_in_monitor(point: (i32, i32), monitor_area: &Rectangle) -> bool {
    point.0 >= monitor_area.x &&
    point.0 < monitor_area.x + monitor_area.width as i32 &&
    point.1 >= monitor_area.y &&
    point.1 < monitor_area.y + monitor_area.height as i32
}
```

### Window Positioning System

#### Dynamic Window Positioning
**Reference**: `./docs/bevy/examples/window/window_settings.rs:50-90` - Window positioning and sizing
```rust
// Window positioning system
#[derive(Component)]
pub struct LauncherWindow {
    pub current_monitor: Option<u32>,
    pub preferred_position: WindowPosition,
    pub size: (u32, u32),
}

#[derive(Debug, Clone)]
pub enum WindowPosition {
    Centered,
    TopCentered,
    CustomOffset { x: i32, y: i32 },
}

fn launcher_positioning_system(
    mut window_query: Query<&mut LauncherWindow>,
    monitor_manager: Res<MonitorManager>,
    display_settings: Res<DisplaySettings>,
    mut window_reposition_events: EventWriter<WindowRepositionEvent>,
) {
    for mut launcher_window in window_query.iter_mut() {
        if let Some(target_monitor) = monitor_manager.get_target_monitor(&display_settings.show_on_screen) {
            if launcher_window.current_monitor != Some(target_monitor.id) {
                if let Some(position) = monitor_manager.calculate_window_position(
                    &display_settings.show_on_screen,
                    launcher_window.size,
                ) {
                    window_reposition_events.send(WindowRepositionEvent {
                        new_position: position,
                        monitor_id: target_monitor.id,
                    });
                    
                    launcher_window.current_monitor = Some(target_monitor.id);
                }
            }
        }
    }
}

#[derive(Event)]
pub struct WindowRepositionEvent {
    pub new_position: (i32, i32),
    pub monitor_id: u32,
}
```

### Hot-Plug Monitor Management

#### Dynamic Monitor Detection
**Reference**: `./docs/bevy/examples/asset_loading/hot_asset_reloading.rs:60-100` - Dynamic resource updates
```rust
// Hot-plug monitor detection system
#[derive(Component)]
pub struct MonitorHotPlugDetector {
    pub check_timer: Timer,
    pub last_monitor_count: usize,
}

fn monitor_hotplug_system(
    mut detector_query: Query<&mut MonitorHotPlugDetector>,
    mut monitor_manager: ResMut<MonitorManager>,
    mut hotplug_events: EventWriter<MonitorHotPlugEvent>,
    time: Res<Time>,
) {
    for mut detector in detector_query.iter_mut() {
        detector.check_timer.tick(time.delta());
        
        if detector.check_timer.just_finished() {
            let previous_count = detector.last_monitor_count;
            
            // Re-detect monitors
            if let Ok(_) = monitor_manager.detect_monitors() {
                let current_count = monitor_manager.monitors.len();
                
                if current_count != previous_count {
                    hotplug_events.send(MonitorHotPlugEvent {
                        event_type: if current_count > previous_count {
                            HotPlugEventType::Connected
                        } else {
                            HotPlugEventType::Disconnected
                        },
                        previous_count,
                        current_count,
                        monitors: monitor_manager.monitors.clone(),
                    });
                    
                    detector.last_monitor_count = current_count;
                }
            }
        }
    }
}

#[derive(Event)]
pub struct MonitorHotPlugEvent {
    pub event_type: HotPlugEventType,
    pub previous_count: usize,
    pub current_count: usize,
    pub monitors: Vec<MonitorInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HotPlugEventType {
    Connected,
    Disconnected,
}

// Hot-plug event handler
fn monitor_hotplug_handler_system(
    mut hotplug_events: EventReader<MonitorHotPlugEvent>,
    mut display_dropdown_query: Query<&mut DisplaySettingsDropdown>,
    mut notification_events: EventWriter<NotificationEvent>,
) {
    for event in hotplug_events.iter() {
        // Update dropdown options
        for mut dropdown in display_dropdown_query.iter_mut() {
            dropdown.available_options = generate_monitor_options(&event.monitors);
        }
        
        // Notify user of monitor changes
        let message = match event.event_type {
            HotPlugEventType::Connected => 
                format!("Monitor connected ({} displays total)", event.current_count),
            HotPlugEventType::Disconnected => 
                format!("Monitor disconnected ({} displays total)", event.current_count),
        };
        
        notification_events.send(NotificationEvent {
            title: "Display Configuration Changed".to_string(),
            message,
            notification_type: NotificationType::Info,
            duration: Some(Duration::from_secs(5)),
        });
    }
}

// Generate dropdown options from available monitors
fn generate_monitor_options(monitors: &[MonitorInfo]) -> Vec<DropdownOption> {
    let mut options = vec![
        DropdownOption {
            value: ScreenSelection::MouseScreen,
            label: "Screen containing mouse".to_string(),
            available: true,
        },
        DropdownOption {
            value: ScreenSelection::PrimaryScreen,
            label: "Primary screen".to_string(),
            available: monitors.iter().any(|m| m.is_primary),
        },
        DropdownOption {
            value: ScreenSelection::FocusedWindowScreen,
            label: "Screen with focused window".to_string(),
            available: true,
        },
    ];
    
    // Add specific monitor options
    for monitor in monitors {
        options.push(DropdownOption {
            value: ScreenSelection::SpecificScreen(monitor.id),
            label: format!("{} ({}×{})", monitor.name, monitor.resolution.0, monitor.resolution.1),
            available: true,
        });
    }
    
    options
}
```

### Settings Integration

#### Display Settings Management
**Reference**: `./docs/bevy/examples/reflection/reflection.rs:120-160` - Settings persistence and updates
```rust
// Display settings dropdown interaction
fn display_settings_interaction_system(
    mut interaction_query: Query<(&Interaction, &mut DisplaySettingsDropdown), Changed<Interaction>>,
    mut display_settings: ResMut<DisplaySettings>,
    mut settings_events: EventWriter<SettingChangedEvent>,
) {
    for (interaction, mut dropdown) in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            dropdown.open = !dropdown.open;
        }
    }
}

// Dropdown option selection
fn dropdown_option_selection_system(
    mut option_query: Query<(&Interaction, &DropdownOptionComponent), Changed<Interaction>>,
    mut dropdown_query: Query<&mut DisplaySettingsDropdown>,
    mut display_settings: ResMut<DisplaySettings>,
    mut settings_events: EventWriter<SettingChangedEvent>,
) {
    for (interaction, option) in option_query.iter() {
        if *interaction == Interaction::Clicked {
            if let Ok(mut dropdown) = dropdown_query.get_mut(option.dropdown_entity) {
                let old_value = dropdown.current_selection.clone();
                dropdown.current_selection = option.value.clone();
                dropdown.open = false;
                
                display_settings.show_on_screen = option.value.clone();
                
                settings_events.send(SettingChangedEvent {
                    setting_id: "show_raycast_on".to_string(),
                    old_value: SettingValue::Enum(format!("{:?}", old_value)),
                    new_value: SettingValue::Enum(format!("{:?}", option.value)),
                });
            }
        }
    }
}

#[derive(Component)]
pub struct DropdownOptionComponent {
    pub dropdown_entity: Entity,
    pub value: ScreenSelection,
}
```

### Error Handling

#### Display System Error Management
**Reference**: `./docs/bevy/examples/diagnostics/log_diagnostics.rs:100-140` - System error handling and logging
```rust
// Display system error handling
#[derive(Debug, Clone)]
pub enum DisplayError {
    DetectionFailed,
    PositioningFailed,
    MonitorNotFound,
    SystemAPIError(String),
}

fn display_error_handler_system(
    mut error_events: EventReader<DisplayErrorEvent>,
    mut notification_events: EventWriter<NotificationEvent>,
) {
    for error_event in error_events.iter() {
        let (title, message) = match &error_event.error {
            DisplayError::DetectionFailed => (
                "Display Detection Failed".to_string(),
                "Unable to detect connected monitors. Using default display.".to_string(),
            ),
            DisplayError::PositioningFailed => (
                "Window Positioning Failed".to_string(),
                "Unable to position window on selected monitor. Using primary display.".to_string(),
            ),
            DisplayError::MonitorNotFound => (
                "Monitor Not Found".to_string(),
                "Selected monitor is no longer available. Switched to primary display.".to_string(),
            ),
            DisplayError::SystemAPIError(msg) => (
                "Display System Error".to_string(),
                format!("System display error: {}", msg),
            ),
        };
        
        notification_events.send(NotificationEvent {
            title,
            message,
            notification_type: NotificationType::Warning,
            duration: Some(Duration::from_secs(8)),
        });
    }
}

#[derive(Event)]
pub struct DisplayErrorEvent {
    pub error: DisplayError,
}
```

### Architecture Notes

#### Component Structure
- **MonitorManager**: Global resource for multi-monitor state management
- **DisplaySettingsDropdown**: UI component for screen selection
- **MouseTracker**: Resource for tracking mouse position across monitors
- **LauncherWindow**: Component for window positioning state

#### Monitor Detection Strategy
- **Platform-Specific**: Native monitor detection APIs for each platform
- **Hot-Plug Support**: Dynamic detection of monitor connection/disconnection
- **State Caching**: Efficient caching of monitor information with periodic updates
- **Fallback Handling**: Graceful degradation when monitors become unavailable

#### Window Positioning Logic
- **Center Positioning**: Smart centering within monitor work areas
- **Multi-Monitor Awareness**: Proper handling of different monitor sizes and positions
- **User Preference Respect**: Consistent application of user screen selection preferences
- **Adaptive Behavior**: Dynamic repositioning when monitor configuration changes

### Quality Standards
- Accurate multi-monitor detection across all supported platforms
- Smooth window transitions when switching between monitors
- Reliable hot-plug support for monitor connection changes
- Clear user feedback for display configuration issues
- Performance optimization for frequent position calculations

### Integration Points
- Settings system integration for screen selection persistence
- Window management system integration for positioning
- Notification system integration for display change alerts
- Event system integration for monitor state updates
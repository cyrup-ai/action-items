# Task 0: Advanced Settings Interface Layout Implementation

## Objective
Implement the main Advanced Menu settings interface with vertical layout, label-control pairs, consistent spacing, and contextual info icon system for complex configuration options.

## Implementation Details

### Target Files
- `ui/src/ui/components/advanced/settings_layout.rs:1-250` - Main settings layout component
- `ui/src/ui/components/advanced/setting_row.rs:1-200` - Individual setting row component
- `ui/src/ui/components/common/info_tooltip.rs:1-150` - Info icon and tooltip system
- `core/src/settings/advanced_config.rs:1-180` - Advanced configuration data structures

### Bevy Implementation Patterns

#### Main Settings Container Layout
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:50-90` - Vertical container with consistent spacing
**Reference**: `./docs/bevy/examples/ui/ui.rs:100-140` - Scrollable settings panel
```rust
// Advanced settings main container
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(24.0)),
        gap: Size::all(Val::Px(20.0)),
        overflow: Overflow::clip_y(),
        ..default()
    },
    background_color: Color::rgba(0.1, 0.1, 0.1, 1.0).into(),
    ..default()
}

// Settings section container
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        gap: Size::all(Val::Px(12.0)),
        ..default()
    },
    ..default()
}
```

#### Setting Row Component System
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:120-160` - Horizontal label-control layout
**Reference**: `./docs/bevy/examples/ui/ui.rs:200-240` - Consistent row spacing and alignment
```rust
// Individual setting row component
#[derive(Component)]
pub struct SettingRow {
    pub setting_id: String,
    pub label: String,
    pub description: Option<String>,
    pub has_info_tooltip: bool,
}

// Setting row container
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        width: Val::Percent(100.0),
        height: Val::Px(40.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::horizontal(Val::Px(16.0)),
        ..default()
    },
    background_color: Color::rgba(0.12, 0.12, 0.12, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(6.0)),
    ..default()
}

// Left side: Label and info icon
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        gap: Size::all(Val::Px(8.0)),
        flex_grow: 1.0,
        ..default()
    },
    ..default()
}

// Setting label text
TextBundle::from_section(
    setting_row.label.clone(),
    TextStyle {
        font: font_regular.clone(),
        font_size: 14.0,
        color: Color::WHITE,
    },
)

// Info icon (if applicable)
ButtonBundle {
    style: Style {
        width: Val::Px(16.0),
        height: Val::Px(16.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(8.0)),
    ..default()
}

// Info icon text
TextBundle::from_section(
    "i",
    TextStyle {
        font: font_regular.clone(),
        font_size: 10.0,
        color: Color::rgba(0.8, 0.8, 0.8, 1.0),
    },
)
```

#### Control Container System
**Reference**: `./docs/bevy/examples/ui/ui.rs:280-320` - Right-aligned control containers
```rust
// Right side: Control container
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        min_width: Val::Px(200.0),
        justify_content: JustifyContent::FlexEnd,
        ..default()
    },
    ..default()
}

// Control sizing and positioning standards
fn create_control_container(control_type: ControlType) -> Style {
    match control_type {
        ControlType::Dropdown => Style {
            width: Val::Px(200.0),
            height: Val::Px(32.0),
            ..default()
        },
        ControlType::Slider => Style {
            width: Val::Px(150.0),
            height: Val::Px(32.0),
            ..default()
        },
        ControlType::Checkbox => Style {
            width: Val::Px(44.0),
            height: Val::Px(24.0),
            ..default()
        },
        ControlType::TextInput => Style {
            width: Val::Px(180.0),
            height: Val::Px(32.0),
            ..default()
        },
    }
}
```

### Info Tooltip System

#### Tooltip Component and Display
**Reference**: `./docs/bevy/examples/ui/ui.rs:400-450` - Tooltip overlay positioning
**Reference**: `./docs/bevy/examples/input/mouse_input.rs:60-100` - Hover detection for tooltip triggers
```rust
// Info tooltip component
#[derive(Component)]
pub struct InfoTooltip {
    pub setting_id: String,
    pub content: String,
    pub position: TooltipPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TooltipPosition {
    Above,
    Below,
    Left,
    Right,
    Cursor,
}

// Tooltip display system
fn info_tooltip_system(
    mut interaction_query: Query<
        (&Interaction, &InfoTooltip, &GlobalTransform),
        (Changed<Interaction>, With<Button>),
    >,
    mut tooltip_events: EventWriter<TooltipDisplayEvent>,
    mut commands: Commands,
) {
    for (interaction, info_tooltip, transform) in interaction_query.iter() {
        match *interaction {
            Interaction::Hovered => {
                tooltip_events.send(TooltipDisplayEvent::Show {
                    content: info_tooltip.content.clone(),
                    position: calculate_tooltip_position(transform, &info_tooltip.position),
                });
            }
            Interaction::None => {
                tooltip_events.send(TooltipDisplayEvent::Hide {
                    setting_id: info_tooltip.setting_id.clone(),
                });
            }
            _ => {}
        }
    }
}

// Tooltip rendering
fn spawn_tooltip_overlay(
    commands: &mut Commands,
    content: String,
    position: Vec2,
) -> Entity {
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                ..default()
            },
            max_width: Val::Px(300.0),
            padding: UiRect::all(Val::Px(12.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        background_color: Color::rgba(0.2, 0.2, 0.2, 0.95).into(),
        border_color: Color::rgba(0.4, 0.4, 0.4, 1.0).into(),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    })
    .with_children(|tooltip| {
        tooltip.spawn(TextBundle::from_section(
            content,
            TextStyle {
                font: font_regular.clone(),
                font_size: 12.0,
                color: Color::WHITE,
            },
        ).with_text_alignment(TextAlignment::Center));
    }).id()
}
```

### Settings Data Structure

#### Advanced Configuration Model
**Reference**: `./docs/bevy/examples/reflection/reflection.rs:50-90` - Settings data structures with reflection
```rust
// Advanced settings configuration
#[derive(Resource, Clone, Debug, Reflect)]
pub struct AdvancedSettings {
    pub display: DisplaySettings,
    pub navigation: NavigationSettings,
    pub keyboard: KeyboardSettings,
    pub input_methods: InputMethodSettings,
    pub search: SearchSettings,
    pub advanced_input: AdvancedInputSettings,
}

#[derive(Clone, Debug, Reflect)]
pub struct DisplaySettings {
    pub show_on_screen: ScreenSelection,
    pub window_behavior: WindowBehavior,
}

#[derive(Clone, Debug, Reflect, PartialEq)]
pub enum ScreenSelection {
    MouseScreen,
    PrimaryScreen,
    FocusedWindowScreen,
    SpecificScreen(u32),
}

#[derive(Clone, Debug, Reflect)]
pub struct NavigationSettings {
    pub pop_to_root_timeout: RootTimeout,
    pub escape_key_behavior: EscapeBehavior,
    pub navigation_bindings: NavigationBindings,
    pub page_navigation_keys: PageNavigationKeys,
}

#[derive(Clone, Debug, Reflect, PartialEq)]
pub enum RootTimeout {
    Never,
    After30Seconds,
    After60Seconds,
    After90Seconds,
    After2Minutes,
    After5Minutes,
}

#[derive(Clone, Debug, Reflect, PartialEq)]
pub enum EscapeBehavior {
    NavigateBackOrClose,
    AlwaysClose,
    NavigateBackOnly,
    CustomAction(String),
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            display: DisplaySettings {
                show_on_screen: ScreenSelection::MouseScreen,
                window_behavior: WindowBehavior::Standard,
            },
            navigation: NavigationSettings {
                pop_to_root_timeout: RootTimeout::After90Seconds,
                escape_key_behavior: EscapeBehavior::NavigateBackOrClose,
                navigation_bindings: NavigationBindings::MacOSStandard,
                page_navigation_keys: PageNavigationKeys::SquareBrackets,
            },
            keyboard: KeyboardSettings::default(),
            input_methods: InputMethodSettings::default(),
            search: SearchSettings::default(),
            advanced_input: AdvancedInputSettings::default(),
        }
    }
}
```

### Dynamic Settings Generation

#### Settings UI Generation System
**Reference**: `./docs/bevy/examples/ui/ui.rs:500-550` - Dynamic UI generation from configuration
```rust
// Settings UI generation system
fn generate_settings_ui_system(
    mut commands: Commands,
    settings_container_query: Query<Entity, With<AdvancedSettingsContainer>>,
    advanced_settings: Res<AdvancedSettings>,
    asset_server: Res<AssetServer>,
) {
    for container_entity in settings_container_query.iter() {
        // Clear existing settings UI
        commands.entity(container_entity).despawn_descendants();
        
        commands.entity(container_entity).with_children(|parent| {
            // Display and Window Management section
            create_settings_section(parent, "Display and Window Management", &[
                SettingDefinition {
                    id: "show_raycast_on".to_string(),
                    label: "Show Raycast on".to_string(),
                    description: Some("Select which monitor displays the launcher interface".to_string()),
                    control: ControlDefinition::Dropdown {
                        current_value: format!("{:?}", advanced_settings.display.show_on_screen),
                        options: vec![
                            "Screen containing mouse".to_string(),
                            "Primary screen".to_string(),
                            "Screen with focused window".to_string(),
                        ],
                    },
                    has_info: true,
                },
            ]);
            
            // Auto-Navigation Behavior section
            create_settings_section(parent, "Auto-Navigation Behavior", &[
                SettingDefinition {
                    id: "pop_to_root_search".to_string(),
                    label: "Pop to Root Search".to_string(),
                    description: Some("Automatically return to main search after inactivity".to_string()),
                    control: ControlDefinition::Dropdown {
                        current_value: format!("{:?}", advanced_settings.navigation.pop_to_root_timeout),
                        options: vec![
                            "Never".to_string(),
                            "After 30 seconds".to_string(),
                            "After 60 seconds".to_string(),
                            "After 90 seconds".to_string(),
                            "After 2 minutes".to_string(),
                            "After 5 minutes".to_string(),
                        ],
                    },
                    has_info: true,
                },
            ]);
        });
    }
}

// Setting definition structures
#[derive(Debug, Clone)]
pub struct SettingDefinition {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub control: ControlDefinition,
    pub has_info: bool,
}

#[derive(Debug, Clone)]
pub enum ControlDefinition {
    Dropdown { current_value: String, options: Vec<String> },
    Slider { current_value: f32, min: f32, max: f32, step: f32 },
    Checkbox { current_value: bool },
    TextInput { current_value: String, placeholder: String },
}

// Create individual settings section
fn create_settings_section(
    parent: &mut ChildBuilder,
    section_title: &str,
    settings: &[SettingDefinition],
) {
    // Section header
    parent.spawn(TextBundle::from_section(
        section_title,
        TextStyle {
            font: font_medium.clone(),
            font_size: 16.0,
            color: Color::rgba(0.9, 0.9, 0.9, 1.0),
        },
    )).insert(SectionHeader);
    
    // Section settings
    for setting in settings {
        create_setting_row(parent, setting);
    }
}
```

### Event System Integration

#### Settings Events
**Reference**: `./docs/bevy/examples/ecs/event.rs:80-120` - Settings change events
```rust
// Advanced settings events
#[derive(Event)]
pub struct SettingChangedEvent {
    pub setting_id: String,
    pub old_value: SettingValue,
    pub new_value: SettingValue,
}

#[derive(Event)]
pub enum TooltipDisplayEvent {
    Show { content: String, position: Vec2 },
    Hide { setting_id: String },
}

#[derive(Debug, Clone)]
pub enum SettingValue {
    String(String),
    Bool(bool),
    Float(f32),
    Int(i32),
    Enum(String),
}

// Settings change handler
fn settings_change_handler_system(
    mut setting_events: EventReader<SettingChangedEvent>,
    mut advanced_settings: ResMut<AdvancedSettings>,
    mut notification_events: EventWriter<NotificationEvent>,
) {
    for event in setting_events.iter() {
        if apply_setting_change(&mut advanced_settings, &event.setting_id, &event.new_value) {
            notification_events.send(NotificationEvent {
                title: "Setting Updated".to_string(),
                message: format!("Successfully updated {}", event.setting_id),
                notification_type: NotificationType::Success,
                duration: Some(Duration::from_secs(3)),
            });
        } else {
            notification_events.send(NotificationEvent {
                title: "Setting Error".to_string(),
                message: format!("Failed to update {}", event.setting_id),
                notification_type: NotificationType::Error,
                duration: Some(Duration::from_secs(5)),
            });
        }
    }
}
```

### Architecture Notes

#### Component Structure
- **AdvancedSettingsContainer**: Main container for all settings sections
- **SettingRow**: Individual setting row with label and control
- **InfoTooltip**: Contextual help system for complex settings
- **AdvancedSettings**: Resource containing all configuration values

#### UI Generation Strategy
- **Dynamic Rendering**: Settings UI generated from configuration definitions
- **Consistent Layout**: Standardized spacing and alignment across all settings
- **Responsive Design**: Adaptive layout for different window sizes
- **Contextual Help**: Info tooltips for settings requiring explanation

#### Data Management
- **Structured Configuration**: Hierarchical settings organization
- **Type Safety**: Strongly typed setting values with validation
- **Change Tracking**: Comprehensive tracking of setting modifications
- **Persistence**: Automatic saving and loading of setting changes

### Quality Standards
- Consistent visual hierarchy with proper spacing and alignment
- Responsive tooltip system with intelligent positioning
- Smooth interactions with immediate visual feedback
- Comprehensive error handling for invalid setting values
- Performance optimization for dynamic UI generation

### Integration Points
- Settings persistence system for configuration storage
- Tooltip system integration for contextual help
- Theme system integration for consistent styling
- Event system integration for setting change notifications
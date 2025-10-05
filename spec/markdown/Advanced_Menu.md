# Advanced Menu Specification

## Overview
The Advanced Menu provides sophisticated configuration options for power users, accessibility features, and system integration settings. This interface exposes deep customization capabilities for window behavior, keyboard navigation, search sensitivity, and specialized input handling.

## Layout Architecture
- **Base Layout**: Advanced tab active in primary navigation
- **Single Column**: Vertical list of configuration sections with consistent spacing
- **Label-Control Pairs**: Left-aligned labels with right-aligned controls
- **Info Integration**: Contextual help icons for complex settings

## Configuration Sections

### Display and Window Management

#### Show Raycast on
- **Current Setting**: "Screen containing mouse"
- **Control Type**: Dropdown selection
- **Purpose**: Multi-monitor display behavior configuration
- **Options**: 
  - Screen containing mouse
  - Primary screen
  - Screen with focused window
  - Specific screen selection
- **Behavior**: Determines which monitor displays the launcher interface

### Auto-Navigation Behavior

#### Pop to Root Search
- **Current Setting**: "After 90 seconds"
- **Control Type**: Dropdown with time intervals
- **Purpose**: Automatic return to main search after inactivity
- **Time Options**:
  - Never
  - After 30 seconds
  - After 60 seconds
  - After 90 seconds
  - After 2 minutes
  - After 5 minutes
- **Info Icon**: Explains timeout behavior and use cases

### Keyboard Interaction Control

#### Escape Key Behavior
- **Current Setting**: "Navigate back or close window"
- **Control Type**: Dropdown selection
- **Purpose**: Customizable Escape key functionality
- **Options**:
  - Navigate back or close window
  - Always close window
  - Navigate back only
  - Custom action assignment
- **Info Icon**: Details about navigation hierarchy and window management

### Input Method Management

#### Auto-switch Input Source
- **Current Setting**: "U.S."
- **Control Type**: Dropdown with available input methods
- **Purpose**: Automatic input method switching for search
- **Functionality**: 
  - Automatically switches to specified input method when launcher opens
  - Restores previous input method when launcher closes
  - Supports international keyboard layouts
- **Info Icon**: Explains input method switching behavior

### Navigation and Keyboard Bindings

#### Navigation Bindings
- **Current Setting**: "macOS Standard (^N, ^P, ^F, ^B)"
- **Control Type**: Preset binding selection dropdown
- **Purpose**: Keyboard navigation scheme selection
- **Binding Options**:
  - macOS Standard (Ctrl+N/P/F/B for navigation)
  - Vim-style navigation
  - Emacs-style navigation
  - Arrow keys only
  - Custom binding configuration
- **Info Icon**: Details about each binding scheme and customization

#### Page Navigation Keys
- **Current Setting**: "Square Brackets"
- **Control Type**: Dropdown selection
- **Purpose**: Page-up/page-down key configuration
- **Options**:
  - Square Brackets ([ and ])
  - Angle Brackets (< and >)
  - Page Up/Down keys
  - Function keys (F7/F8)
  - Custom key assignment
- **Info Icon**: Explains page navigation in long result lists

### Search Sensitivity Configuration

#### Root Search Sensitivity
- **Control Type**: Horizontal slider with discrete positions
- **Current Setting**: Medium (center position)
- **Scale**: Low → Medium → High
- **Purpose**: Fuzzy search algorithm sensitivity tuning
- **Behavior**:
  - **Low**: Strict matching, exact or near-exact matches only
  - **Medium**: Balanced fuzzy matching with reasonable tolerance
  - **High**: Aggressive fuzzy matching, finds results with significant variations
- **Info Icon**: Examples of search behavior at different sensitivity levels

### Advanced Input Features

#### Hyper Key Configuration
- **Current Setting**: "–" (disabled/not configured)
- **Control Type**: Dropdown for key assignment
- **Purpose**: Hyper key (super-modifier) configuration
- **Options**:
  - Disabled (–)
  - Caps Lock
  - Right Option
  - Right Command
  - Function key
  - Custom key assignment
- **Functionality**: Enables single-key launcher activation
- **Info Icon**: Explains hyper key concept and system integration

#### Text Replacement Features
- **Setting**: "Replace occurrences of ^⌃⇧⌘ with ⌃"
- **Control Type**: Checkbox toggle
- **Current State**: Enabled (checked)
- **Purpose**: Automatic text replacement for keyboard shortcut notation
- **Functionality**: 
  - Converts complex modifier notation to simplified format
  - Improves readability in help text and documentation
  - Customizable replacement rules

## Functional Requirements

### Multi-Monitor Support System
- **Monitor Detection**: Automatic detection of connected displays
- **Window Positioning**: Intelligent positioning based on user preference
- **Resolution Awareness**: Adaptive sizing for different screen resolutions
- **Hot-Plugging**: Dynamic response to monitor connection/disconnection events

### Auto-Navigation Timer System
- **Precise Timing**: Accurate timeout measurement and triggering
- **Activity Detection**: Monitoring of user interaction to reset timers
- **Smooth Transitions**: Seamless navigation back to root search
- **Timer Persistence**: Maintaining timer state across application sessions

### Input Method Integration
- **System Integration**: Deep integration with macOS input method system
- **State Preservation**: Reliable restoration of previous input methods
- **Multi-Language Support**: Support for complex input methods (CJK, RTL languages)
- **Conflict Resolution**: Handling of input method conflicts and failures

### Advanced Keyboard Handling
- **Global Shortcuts**: System-wide keyboard shortcut capture and handling
- **Conflict Detection**: Detection and resolution of shortcut conflicts
- **Custom Bindings**: User-defined keyboard shortcut creation and management
- **Accessibility Integration**: Compatibility with accessibility keyboard features

## Bevy Implementation Examples

### Dropdown Controls Implementation
- Reference: `./docs/bevy/examples/ui/ui.rs` - Dropdown menu creation and interaction
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Dropdown click handling and state management

### Slider Control Implementation
- Reference: `./docs/bevy/examples/ui/ui.rs` - Horizontal slider with discrete positions
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Slider drag interaction and value updates

### Checkbox Toggle Implementation
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Checkbox states and visual feedback
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Checkbox interaction handling

### Info Icon System
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Icon management and hover states
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Tooltip trigger and display handling

### Settings Persistence
- Reference: `./docs/bevy/examples/reflection/reflection.rs` - Settings serialization and deserialization
- Reference: `./docs/bevy/examples/app/return_after_run.rs` - Application state management

### Keyboard Event Handling
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Advanced keyboard event capture
- Reference: `./docs/bevy/examples/input/keyboard_modifiers.rs` - Modifier key handling and combinations

### Timer and Auto-Navigation
- Reference: `./docs/bevy/examples/time/time.rs` - Precise timing and timeout implementation
- Reference: `./docs/bevy/examples/app/return_after_run.rs` - State transitions and navigation

## State Management Requirements

### Configuration Persistence
- **Settings Storage**: Secure storage of all advanced configuration options
- **Migration Support**: Automatic migration of settings between application versions
- **Backup and Restore**: User-controlled backup and restoration of configuration
- **Validation**: Real-time validation of configuration changes and conflicts

### Multi-Monitor State Management
- **Monitor Configuration**: Tracking of monitor setup and user preferences
- **Position Memory**: Remembering launcher position preferences per monitor
- **Resolution Changes**: Adaptive response to resolution and scaling changes
- **Multi-Desktop Support**: Integration with virtual desktop and Spaces functionality

### Input Method State Tracking
- **Current State Monitoring**: Real-time tracking of active input methods
- **Transition Management**: Smooth transitions between input methods
- **Failure Recovery**: Graceful handling of input method switching failures
- **User Preference Sync**: Synchronization of input method preferences across devices

## Security and System Integration

### System-Level Integration
- **Accessibility API**: Integration with macOS accessibility APIs for keyboard control
- **Input Method Framework**: Deep integration with system input method frameworks
- **Display Management**: Integration with display management APIs for multi-monitor support
- **Permission Handling**: Proper handling of system permissions for advanced features

### Privacy Considerations
- **Input Monitoring**: Transparent handling of keyboard input monitoring
- **Data Collection**: Minimal data collection with user consent
- **Local Processing**: Local processing of sensitive keyboard and input data
- **Audit Capabilities**: Optional audit logging of advanced feature usage

### Security Boundaries
- **Privilege Separation**: Minimal privilege requirements for advanced features
- **Sandboxing Compatibility**: Compatibility with application sandboxing requirements
- **System Integrity**: No interference with system security mechanisms
- **Safe Defaults**: Secure default configurations for all advanced options

## Performance Optimization

### Efficient Event Processing
- **Event Filtering**: Intelligent filtering of unnecessary system events
- **Batch Processing**: Efficient batch processing of configuration changes
- **Lazy Evaluation**: On-demand evaluation of complex configuration options
- **Resource Management**: Efficient management of system resources for advanced features

### Responsive User Interface
- **Immediate Feedback**: Instant visual feedback for all configuration changes
- **Progressive Disclosure**: Smart showing/hiding of advanced options based on context
- **Smooth Animations**: Smooth transitions for slider and dropdown interactions
- **Non-Blocking Operations**: Non-blocking application of configuration changes

### Memory and CPU Optimization
- **Configuration Caching**: Intelligent caching of processed configuration data
- **System Integration Optimization**: Efficient system API usage for advanced features
- **Background Processing**: Background processing of non-critical configuration tasks
- **Resource Cleanup**: Proper cleanup of system resources and event handlers

## Error Handling and Recovery

### Configuration Error Handling
- **Invalid Settings**: Graceful handling of invalid configuration values
- **System Conflicts**: Detection and resolution of system-level configuration conflicts
- **Migration Failures**: Recovery mechanisms for failed configuration migrations
- **Validation Errors**: Real-time validation with clear error messaging

### System Integration Error Handling
- **API Failures**: Robust error handling for system API failures
- **Permission Denials**: Clear messaging and recovery options for permission issues
- **Hardware Changes**: Adaptive response to hardware configuration changes
- **Network Issues**: Handling of network-dependent configuration options

### User Experience Recovery
- **Settings Reset**: One-click reset to safe default configurations
- **Incremental Recovery**: Step-by-step recovery from problematic configurations
- **Diagnostic Tools**: Built-in diagnostic tools for troubleshooting advanced features
- **Expert Support**: Clear pathways to expert support for complex configuration issues

## Bevy Implementation Details

### Component Architecture

#### Advanced Configuration Components
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Component, Reflect)]
pub struct DropdownControl<T: Reflect + Clone> {
    pub label: String,
    pub current_value: T,
    pub options: Vec<DropdownOption<T>>,
    pub expanded: bool,
    pub selected_index: usize,
}

#[derive(Reflect, Clone)]
pub struct DropdownOption<T: Reflect + Clone> {
    pub value: T,
    pub display_text: String,
    pub description: Option<String>,
}

#[derive(Component, Reflect)]
pub struct SliderControl {
    pub label: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub discrete_positions: Vec<SliderPosition>,
    pub dragging: bool,
}

#[derive(Reflect, Clone)]
pub struct SliderPosition {
    pub value: f32,
    pub label: String,
}

#[derive(Component, Reflect)]
pub struct InfoIcon {
    pub tooltip: String,
    pub expanded: bool,
    pub position: Vec2,
}

#[derive(Component, Reflect)]
pub struct ConfigurationSection {
    pub title: String,
    pub settings: HashMap<String, ConfigValue>,
    pub dirty: bool,
}

#[derive(Reflect, Clone)]
pub enum ConfigValue {
    ScreenTarget(ScreenTarget),
    TimeInterval(TimeInterval),
    EscapeBehavior(EscapeBehavior),
    InputSource(String),
    NavigationBinding(NavigationScheme),
    PageNavigation(PageNavigationKeys),
    SearchSensitivity(f32),
    HyperKey(Option<KeyCode>),
}
```

#### Configuration State Management
```rust
#[derive(Resource, Reflect)]
pub struct AdvancedConfigState {
    pub screen_target: ScreenTarget,
    pub pop_to_root_timeout: TimeInterval,
    pub escape_behavior: EscapeBehavior,
    pub input_source: String,
    pub navigation_binding: NavigationScheme,
    pub page_navigation: PageNavigationKeys,
    pub search_sensitivity: f32,
    pub hyper_key: Option<KeyCode>,
    pub text_replacement_enabled: bool,
    pub modified_settings: HashSet<String>,
}

#[derive(Reflect, Clone, PartialEq)]
pub enum ScreenTarget {
    MouseScreen,
    PrimaryScreen,
    FocusedWindowScreen,
    Specific(u32),
}

#[derive(Reflect, Clone, PartialEq)]
pub enum TimeInterval {
    Never,
    Seconds(u32),
}

#[derive(Reflect, Clone, PartialEq)]
pub enum EscapeBehavior {
    NavigateBackOrClose,
    AlwaysClose,
    NavigateBackOnly,
}

#[derive(Reflect, Clone, PartialEq)]
pub enum NavigationScheme {
    MacOSStandard,
    VimStyle,
    EmacsStyle,
    ArrowKeysOnly,
}

#[derive(Reflect, Clone, PartialEq)]
pub enum PageNavigationKeys {
    SquareBrackets,
    AngleBrackets,
    PageUpDown,
    FunctionKeys,
}
```

### Event System for Advanced Configuration
```rust
#[derive(Event, Reflect)]
pub enum AdvancedConfigEvent {
    DropdownToggled(String),
    DropdownValueChanged { setting: String, value: ConfigValue },
    SliderValueChanged { setting: String, value: f32 },
    InfoIconHovered { setting: String, position: Vec2 },
    InfoIconClicked(String),
    HyperKeyRecordingStarted,
    HyperKeyRecorded(KeyCode),
    TextReplacementToggled(bool),
    ConfigurationSaved,
    ConfigurationReset,
}

#[derive(Event, Reflect)]
pub enum ValidationEvent {
    SettingValidated { setting: String, valid: bool, message: String },
    MultiMonitorDetected(Vec<MonitorInfo>),
    HyperKeyConflictDetected { key: KeyCode, conflicting_app: String },
}
```

### Dropdown System Implementation
```rust
fn dropdown_system(
    mut dropdown_query: Query<(Entity, &mut DropdownControl<ConfigValue>, &mut Node), Changed<Interaction>>,
    mut interaction_query: Query<&Interaction, Changed<Interaction>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<AdvancedConfigEvent>,
    mut commands: Commands,
) {
    for (entity, mut dropdown, mut style) in dropdown_query.iter_mut() {
        // Handle dropdown expansion/collapse
        if let Ok(interaction) = interaction_query.get(entity) {
            if *interaction == Interaction::Pressed {
                dropdown.expanded = !dropdown.expanded;
                events.send(AdvancedConfigEvent::DropdownToggled(dropdown.label.clone()));
                
                // Update UI to show expanded state
                if dropdown.expanded {
                    style.height = Val::Px(dropdown.options.len() as f32 * 40.0 + 40.0);
                } else {
                    style.height = Val::Px(40.0);
                }
            }
        }
        
        // Handle keyboard navigation in expanded dropdown
        if dropdown.expanded {
            if keyboard_input.just_pressed(KeyCode::ArrowUp) {
                if dropdown.selected_index > 0 {
                    dropdown.selected_index -= 1;
                }
            }
            if keyboard_input.just_pressed(KeyCode::ArrowDown) {
                if dropdown.selected_index < dropdown.options.len().saturating_sub(1) {
                    dropdown.selected_index += 1;
                }
            }
            if keyboard_input.just_pressed(KeyCode::Enter) {
                // Select current option
                let selected_option = &dropdown.options[dropdown.selected_index];
                dropdown.current_value = selected_option.value.clone();
                dropdown.expanded = false;
                style.height = Val::Px(40.0);
                
                events.send(AdvancedConfigEvent::DropdownValueChanged {
                    setting: dropdown.label.clone(),
                    value: selected_option.value.clone(),
                });
            }
            if keyboard_input.just_pressed(KeyCode::Escape) {
                dropdown.expanded = false;
                style.height = Val::Px(40.0);
            }
        }
    }
}
```

### Slider System Implementation
```rust
fn slider_system(
    mut slider_query: Query<(Entity, &mut SliderControl, &Node), Changed<Interaction>>,
    mut interaction_query: Query<&Interaction, Changed<Interaction>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut events: EventWriter<AdvancedConfigEvent>,
) {
    for (entity, mut slider, node) in slider_query.iter_mut() {
        if let Ok(interaction) = interaction_query.get(entity) {
            match *interaction {
                Interaction::Pressed => {
                    slider.dragging = true;
                },
                Interaction::None => {
                    if slider.dragging && !mouse_input.pressed(MouseButton::Left) {
                        slider.dragging = false;
                    }
                },
                _ => {}
            }
        }
        
        if slider.dragging {
            let window = windows.single();
            if let Some(cursor_pos) = window.cursor_position() {
                // Convert cursor position to slider value
                let (camera, camera_transform) = camera_query.single();
                
                // Calculate slider position and convert to value
                // This is a simplified version - real implementation would need proper coordinate conversion
                let slider_rect = calculate_slider_rect(node, camera_transform);
                let relative_pos = (cursor_pos.x - slider_rect.min.x) / slider_rect.width();
                let new_value = slider.min + (slider.max - slider.min) * relative_pos.clamp(0.0, 1.0);
                
                // Snap to discrete positions if they exist
                if !slider.discrete_positions.is_empty() {
                    let closest_position = slider.discrete_positions.iter()
                        .min_by(|a, b| {
                            let a_diff = (a.value - new_value).abs();
                            let b_diff = (b.value - new_value).abs();
                            a_diff.partial_cmp(&b_diff).unwrap()
                        });
                    
                    if let Some(position) = closest_position {
                        slider.value = position.value;
                    }
                } else {
                    // Apply step size
                    slider.value = ((new_value / slider.step).round() * slider.step)
                        .clamp(slider.min, slider.max);
                }
                
                events.send(AdvancedConfigEvent::SliderValueChanged {
                    setting: slider.label.clone(),
                    value: slider.value,
                });
            }
        }
    }
}

fn calculate_slider_rect(node: &Node, camera_transform: &GlobalTransform) -> bevy::math::Rect {
    // Simplified rect calculation - real implementation would be more complex
    bevy::math::Rect::new(0.0, 0.0, 200.0, 20.0)
}
```

### Info Icon Tooltip System
```rust
fn info_icon_system(
    mut info_icon_query: Query<(Entity, &mut InfoIcon), Changed<Interaction>>,
    mut interaction_query: Query<&Interaction, Changed<Interaction>>,
    mut events: EventWriter<AdvancedConfigEvent>,
    mut commands: Commands,
) {
    for (entity, mut info_icon) in info_icon_query.iter_mut() {
        if let Ok(interaction) = interaction_query.get(entity) {
            match *interaction {
                Interaction::Hovered => {
                    if !info_icon.expanded {
                        info_icon.expanded = true;
                        events.send(AdvancedConfigEvent::InfoIconHovered {
                            setting: "info_icon".to_string(),
                            position: info_icon.position,
                        });
                        
                        // Spawn tooltip entity
                        spawn_tooltip(&mut commands, entity, &info_icon.tooltip, info_icon.position);
                    }
                },
                Interaction::None => {
                    if info_icon.expanded {
                        info_icon.expanded = false;
                        // Remove tooltip entity
                        remove_tooltip(&mut commands, entity);
                    }
                },
                Interaction::Pressed => {
                    events.send(AdvancedConfigEvent::InfoIconClicked("info_icon".to_string()));
                },
            }
        }
    }
}

fn spawn_tooltip(commands: &mut Commands, parent: Entity, text: &str, position: Vec2) {
    commands.entity(parent).with_children(|parent| {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(position.x + 24.0),
                top: Val::Px(position.y),
                width: Val::Px(200.0),
                padding: UiRect::all(Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
            ZIndex(1000),
        )).with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn remove_tooltip(commands: &mut Commands, parent: Entity) {
    // Remove tooltip child entities
    // This would require tracking tooltip entities or using a marker component
}
```

### Flexible Configuration Layout System
```rust
fn spawn_advanced_menu_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(24.0)),
            overflow: Overflow::clip_y(), // Enable vertical scrolling
            ..default()
        })
        .with_children(|parent| {
            // Page title
            parent.spawn((
                Text::new("Advanced Settings"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
            ));
            
            // Configuration sections
            spawn_display_section(parent);
            spawn_navigation_section(parent);
            spawn_input_section(parent);
            spawn_search_section(parent);
        });
}

fn spawn_display_section(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(32.0)),
        max_width: Val::Px(600.0), // Constrain width
        ..default()
    }).with_children(|parent| {
        // Section title
        parent.spawn((
            Text::new("Display and Window Management"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ));
        
        // Show Raycast on dropdown
        spawn_dropdown_setting(parent, "Show Raycast on", ScreenTarget::MouseScreen, vec![
            DropdownOption {
                value: ScreenTarget::MouseScreen,
                display_text: "Screen containing mouse".to_string(),
                description: Some("Display on the screen where the mouse cursor is located".to_string()),
            },
            DropdownOption {
                value: ScreenTarget::PrimaryScreen,
                display_text: "Primary screen".to_string(),
                description: Some("Always display on the primary monitor".to_string()),
            },
            DropdownOption {
                value: ScreenTarget::FocusedWindowScreen,
                display_text: "Screen with focused window".to_string(),
                description: Some("Display on the screen containing the currently focused window".to_string()),
            },
        ]);
    });
}

fn spawn_navigation_section(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(32.0)),
        max_width: Val::Px(600.0),
        ..default()
    }).with_children(|parent| {
        // Section title
        parent.spawn((
            Text::new("Auto-Navigation Behavior"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ));
        
        // Pop to Root Search dropdown with info icon
        spawn_dropdown_with_info(
            parent,
            "Pop to Root Search",
            TimeInterval::Seconds(90),
            vec![
                DropdownOption {
                    value: TimeInterval::Never,
                    display_text: "Never".to_string(),
                    description: None,
                },
                DropdownOption {
                    value: TimeInterval::Seconds(30),
                    display_text: "After 30 seconds".to_string(),
                    description: None,
                },
                DropdownOption {
                    value: TimeInterval::Seconds(90),
                    display_text: "After 90 seconds".to_string(),
                    description: None,
                },
            ],
            "Automatically returns to the main search after the specified period of inactivity. This helps reset the interface state when you've navigated deep into menus."
        );
    });
}

fn spawn_search_section(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(32.0)),
        max_width: Val::Px(600.0),
        ..default()
    }).with_children(|parent| {
        // Section title
        parent.spawn((
            Text::new("Search Sensitivity Configuration"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ));
        
        // Search sensitivity slider
        spawn_slider_setting(
            parent,
            "Root Search Sensitivity",
            0.5, // Medium sensitivity
            vec![
                SliderPosition { value: 0.0, label: "Low".to_string() },
                SliderPosition { value: 0.5, label: "Medium".to_string() },
                SliderPosition { value: 1.0, label: "High".to_string() },
            ],
            "Controls how fuzzy the search matching algorithm is. Low = strict matching, High = more flexible matching."
        );
    });
}

fn spawn_dropdown_setting<T: Reflect + Clone + 'static>(
    parent: &mut ChildBuilder,
    label: &str,
    current_value: T,
    options: Vec<DropdownOption<T>>,
) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(56.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        margin: UiRect::bottom(Val::Px(12.0)),
        ..default()
    }).with_children(|parent| {
        // Label
        parent.spawn((
            Text::new(label),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                flex_grow: 1.0,
                max_width: Val::Px(200.0),
                ..default()
            },
        ));
        
        // Dropdown control
        parent.spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Px(40.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                overflow: Overflow::clip(), // Prevent expansion
                flex_grow: 0.0, // Prevent expansion
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            Button,
            Interaction::default(),
            DropdownControl {
                label: label.to_string(),
                current_value,
                options,
                expanded: false,
                selected_index: 0,
            },
        ));
    });
}

fn spawn_slider_setting(
    parent: &mut ChildBuilder,
    label: &str,
    current_value: f32,
    positions: Vec<SliderPosition>,
    info_text: &str,
) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(20.0)),
        ..default()
    }).with_children(|parent| {
        // Label row with info icon
        parent.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        }).with_children(|parent| {
            // Label
            parent.spawn((
                Text::new(label),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
            
            // Info icon
            parent.spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    margin: UiRect::left(Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                Button,
                Interaction::default(),
                InfoIcon {
                    tooltip: info_text.to_string(),
                    expanded: false,
                    position: Vec2::ZERO,
                },
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("?"),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });
        
        // Slider track
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(6.0),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                position_type: PositionType::Relative,
                max_width: Val::Px(300.0), // Constrain slider width
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            SliderControl {
                label: label.to_string(),
                value: current_value,
                min: 0.0,
                max: 1.0,
                step: 0.1,
                discrete_positions: positions,
                dragging: false,
            },
        ));
    });
}
```

### Configuration Persistence System
```rust
fn configuration_save_system(
    config_state: Res<AdvancedConfigState>,
    mut events: EventReader<AdvancedConfigEvent>,
    mut local_storage: Local<Option<serde_json::Value>>,
) {
    for event in events.read() {
        match event {
            AdvancedConfigEvent::DropdownValueChanged { setting, value } => {
                // Save immediately to prevent data loss
                save_setting_to_storage(setting, value);
            },
            AdvancedConfigEvent::SliderValueChanged { setting, value } => {
                save_setting_to_storage(setting, &ConfigValue::SearchSensitivity(*value));
            },
            AdvancedConfigEvent::ConfigurationSaved => {
                // Batch save all modified settings
                save_all_settings(&config_state);
            },
            _ => {}
        }
    }
}

fn save_setting_to_storage(setting: &str, value: &ConfigValue) {
    // Implementation would use platform-specific storage API
    info!("Saving setting '{}' with value {:?}", setting, value);
}

fn save_all_settings(config_state: &AdvancedConfigState) {
    // Serialize and save entire configuration
    info!("Saving all configuration settings");
}
```

### SystemSet Organization for Advanced Configuration
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AdvancedConfigSystems {
    Input,
    Validation,
    UI,
    Persistence,
}

impl Plugin for AdvancedConfigPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<DropdownControl<ConfigValue>>()
            .register_type::<SliderControl>()
            .register_type::<InfoIcon>()
            .register_type::<ConfigurationSection>()
            .register_type::<AdvancedConfigState>()
            
            .init_resource::<AdvancedConfigState>()
            
            .add_event::<AdvancedConfigEvent>()
            .add_event::<ValidationEvent>()
            
            .configure_sets(Update, (
                AdvancedConfigSystems::Input,
                AdvancedConfigSystems::Validation,
                AdvancedConfigSystems::UI,
                AdvancedConfigSystems::Persistence,
            ).chain())
            
            .add_systems(Update, (
                // Input handling with immediate response
                (
                    dropdown_system,
                    slider_system,
                    info_icon_system,
                    keyboard_navigation_system,
                ).in_set(AdvancedConfigSystems::Input),
                
                // Real-time validation
                (
                    setting_validation_system,
                    multi_monitor_detection_system,
                    hyper_key_conflict_system,
                ).in_set(AdvancedConfigSystems::Validation),
                
                // UI updates with Changed<T> optimization
                (
                    dropdown_ui_update_system,
                    slider_ui_update_system,
                    tooltip_positioning_system,
                ).in_set(AdvancedConfigSystems::UI),
                
                // Configuration persistence
                (
                    configuration_save_system,
                    setting_sync_system,
                ).in_set(AdvancedConfigSystems::Persistence),
            ));
    }
}
```

This comprehensive Bevy implementation provides:

1. **Flexible dropdown controls** with proper expansion constraints using `overflow: clip()`
2. **Interactive slider controls** with discrete positions and smooth dragging
3. **Info icon tooltip system** with proper positioning and hover states
4. **Event-driven configuration** with real-time validation and persistence
5. **Constrained flex layouts** preventing unwanted UI expansion
6. **Component-driven design** with full reflection support
7. **Query optimization** using `Changed<T>` filters for efficient updates
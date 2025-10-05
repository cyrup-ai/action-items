# General Menu Specification

## Overview
The General Menu serves as the primary configuration interface for core application settings, including startup behavior, global hotkeys, visual appearance, themes, and window modes. This interface provides essential customization options that define the fundamental user experience.

## Layout Architecture
- **Base Layout**: General tab active in primary navigation
- **Vertical Configuration Sections**: Logically grouped settings with consistent spacing
- **Mixed Control Types**: Checkboxes, dropdowns, buttons, toggle groups, and visual selectors
- **Progressive Disclosure**: Advanced options revealed contextually

## Configuration Sections

### Application Startup Settings

#### Launch at Login Configuration
- **Setting**: "Launch Raycast at login"
- **Control Type**: Checkbox toggle
- **Current State**: Enabled (checked)
- **Functionality**: 
  - Automatic application launch when user logs into system
  - Integration with macOS login items system
  - Seamless background startup without user intervention

### Global Hotkey Configuration

#### Primary Hotkey Assignment  
- **Setting**: "Raycast Hotkey"
- **Current Assignment**: "⌘ Space" (Command + Space)
- **Control Type**: Interactive hotkey display button
- **Functionality**:
  - System-wide hotkey capture and assignment
  - Visual representation of assigned key combination
  - Click-to-record new hotkey functionality
  - Conflict detection with existing system shortcuts

### System Integration Settings

#### Menu Bar Visibility Control
- **Setting**: "Show Raycast in menu bar"
- **Control Type**: Checkbox toggle
- **Current State**: Disabled (unchecked)
- **Purpose**: 
  - Optional menu bar icon for quick access
  - Alternative access method to global hotkey
  - System integration visibility control

### Text and Display Preferences

#### Text Size Configuration
- **Setting**: "Text Size"
- **Control Type**: Toggle button group
- **Options**: Two "Aa" buttons representing different font sizes
  - **Small Text**: Standard size for compact display
  - **Large Text**: Increased size for accessibility and readability
- **Current Selection**: Small text (left option selected)
- **Functionality**: Global text scaling across entire interface

### Theme and Appearance System

#### Theme Selection
- **Primary Themes**:
  1. **Raycast Dark**
     - **Current State**: Selected (active dropdown)
     - **Icon**: Moon/dark mode indicator
     - **Characteristics**: Dark color scheme optimized for low-light usage
  
  2. **Raycast Light** 
     - **State**: Available option
     - **Icon**: Sun/light mode indicator  
     - **Characteristics**: Light color scheme for bright environment usage

#### System Appearance Integration
- **Setting**: "Follow system appearance"
- **Control Type**: Checkbox toggle
- **Current State**: Enabled (checked)
- **Functionality**:
  - Automatic theme switching based on macOS Dark Mode
  - Seamless integration with system-wide appearance changes
  - Override individual theme selection when enabled

#### Advanced Theme Customization
- **Control**: "Open Theme Studio" button
- **Functionality**: 
  - Advanced theme customization interface
  - Custom theme creation and editing
  - Community theme import/export capabilities
  - Pro feature integration

### Window Mode Configuration

#### Window Display Options
- **Setting**: "Window Mode"
- **Control Type**: Visual selection cards
- **Options**:

  1. **Default Mode**
     - **Visual**: Purple gradient card with full interface representation
     - **Wireframe**: Rounded corner UI mockup showing complete interface layout
     - **Current State**: Selected (highlighted border)
     - **Characteristics**: Full-featured interface with all visual elements
     - **Use Case**: Standard usage with complete functionality display

  2. **Compact Mode**
     - **Visual**: Gray minimalist card with simplified interface
     - **Wireframe**: Rounded corner UI mockup showing streamlined interface layout
     - **Current State**: Available option
     - **Characteristics**: Streamlined interface with reduced visual complexity
     - **Use Case**: Minimal distraction, power-user focused experience

### Favorites Display Configuration

#### Compact Mode Favorites
- **Setting**: "Show favorites in compact mode"
- **Control Type**: Checkbox toggle
- **Current State**: Enabled (checked)
- **Functionality**:
  - Display favorites list even in compact window mode
  - Maintains quick access to frequently used commands
  - Balances minimalism with functionality

## Functional Requirements

### Hotkey Management System
- **Global Capture**: System-wide hotkey registration and handling
- **Conflict Resolution**: Detection and resolution of hotkey conflicts
- **Recording Interface**: Modal hotkey recording with visual feedback
- **Validation**: Real-time validation of hotkey combinations and availability

### Theme Management System
- **Dynamic Switching**: Seamless theme transitions without application restart
- **System Integration**: Automatic theme switching based on system appearance
- **Custom Theme Support**: Loading and application of user-created themes
- **Performance Optimization**: Efficient theme resource management and caching

### Window Mode Management
- **Dynamic Layout**: Real-time switching between window modes
- **State Persistence**: Maintaining mode selection across application sessions  
- **Responsive Design**: Adaptive interface elements based on selected mode
- **Performance Considerations**: Optimized rendering for both modes

### Startup Integration
- **System Service Registration**: Proper integration with macOS login items
- **Background Launch**: Minimal resource usage for background startup
- **Dependency Management**: Handling of startup dependencies and services
- **Error Handling**: Graceful handling of startup failures and conflicts

## Bevy Implementation Examples

### Hotkey Recording Interface
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Keyboard event capture and processing
- Reference: `./docs/bevy/examples/ui/ui.rs` - Modal dialog overlays for recording interface

### Theme Selection System
- Reference: `./docs/bevy/examples/ui/ui.rs` - Dropdown menu implementation with theme previews
- Reference: `./docs/bevy/examples/asset_loading/hot_asset_reloading.rs` - Dynamic theme resource loading

### Visual Mode Selection Cards
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Card-based selection with preview images
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Card selection interaction handling

### Text Size Toggle Controls
- Reference: `./docs/bevy/examples/ui/text.rs` - Dynamic text sizing and scaling
- Reference: `./docs/bevy/examples/ui/ui.rs` - Toggle button group implementation

### Checkbox and Toggle Controls
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Checkbox states and visual feedback
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Toggle interaction handling

### Settings Persistence
- Reference: `./docs/bevy/examples/reflection/reflection.rs` - Configuration serialization and storage
- Reference: `./docs/bevy/examples/app/return_after_run.rs` - Application state management

### System Integration
- Reference: `./docs/bevy/examples/app/return_after_run.rs` - System service integration patterns
- Reference: `./docs/bevy/examples/async_tasks/async_compute.rs` - Background system integration tasks

## State Management Requirements

### Configuration Synchronization
- **Real-time Updates**: Immediate application of configuration changes
- **Persistence**: Reliable storage and retrieval of user preferences
- **Migration**: Automatic migration of settings between application versions
- **Backup**: Optional backup and restore of configuration settings

### Theme State Management  
- **Dynamic Loading**: On-demand loading of theme resources
- **Transition Handling**: Smooth transitions between themes
- **System Monitoring**: Real-time monitoring of system appearance changes
- **Cache Management**: Efficient caching of theme assets and configurations

### Hotkey State Tracking
- **Registration Monitoring**: Real-time monitoring of hotkey registration status
- **Conflict Detection**: Continuous monitoring for hotkey conflicts
- **System Integration**: Proper integration with system hotkey management
- **Fallback Handling**: Graceful fallback when hotkey registration fails

## Security and Privacy Considerations

### System Integration Security
- **Privilege Management**: Minimal privilege requirements for system integration
- **Hotkey Security**: Secure handling of global hotkey registration
- **Login Item Management**: Safe registration and management of startup items
- **Permission Validation**: Proper validation of system permissions

### Data Protection
- **Configuration Security**: Secure storage of user configuration data
- **Theme Security**: Safe loading and execution of custom theme resources
- **Privacy Preservation**: Minimal data collection for configuration purposes
- **User Consent**: Clear communication about system integration requirements

### Access Control
- **System API Access**: Controlled access to system appearance and hotkey APIs
- **Resource Protection**: Protection of theme and configuration resources
- **Update Security**: Secure update mechanism for configuration schemas
- **Audit Logging**: Optional logging of configuration changes

## Performance Optimization Requirements

### Startup Performance
- **Fast Launch**: Optimized application startup time
- **Background Efficiency**: Minimal resource usage in background mode
- **Lazy Loading**: On-demand loading of non-essential configuration elements
- **Memory Management**: Efficient memory usage for configuration data

### Theme Performance
- **Asset Optimization**: Efficient loading and caching of theme assets
- **Transition Performance**: Smooth theme transitions without performance impact
- **Resource Cleanup**: Proper cleanup of unused theme resources
- **Dynamic Loading**: Efficient on-demand theme resource loading

### UI Responsiveness
- **Real-time Feedback**: Immediate visual feedback for all configuration changes
- **Smooth Animations**: Fluid animations for configuration transitions
- **Non-blocking Operations**: Non-blocking application of configuration changes
- **Progressive Enhancement**: Graceful degradation for resource-constrained scenarios

## Accessibility Requirements

### Keyboard Navigation
- **Full Keyboard Access**: Complete configuration access via keyboard navigation
- **Logical Tab Order**: Intuitive keyboard navigation flow through all controls
- **Hotkey Accessibility**: Accessible hotkey recording for users with disabilities
- **Shortcut Consistency**: Consistent keyboard shortcuts for configuration actions

### Screen Reader Support
- **Semantic Labels**: Clear labels and descriptions for all configuration options
- **State Announcements**: Real-time announcement of configuration changes
- **Group Labels**: Proper grouping and labeling of related configuration options
- **Help Integration**: Accessible help information for complex configuration options

### Visual Accessibility
- **High Contrast Support**: Proper contrast ratios for all configuration elements
- **Font Scaling**: Proper scaling behavior for increased system font sizes
- **Color Independence**: Functionality not dependent on color perception alone
- **Focus Indicators**: Clear visual focus indicators for all interactive elements

## Error Handling and Recovery

### Configuration Error Handling
- **Validation Errors**: Real-time validation with clear error messaging
- **Conflict Resolution**: Automated resolution of configuration conflicts
- **Recovery Mechanisms**: Safe recovery from corrupted configuration data
- **Default Restoration**: One-click restoration to safe default settings

### System Integration Error Handling
- **Hotkey Registration Failures**: Clear feedback and alternative options for failed hotkey registration
- **Theme Loading Failures**: Graceful fallback to default themes for loading failures
- **Startup Integration Errors**: Clear messaging and resolution for startup integration issues
- **Permission Errors**: User-friendly guidance for system permission requirements

### User Experience Recovery
- **Incremental Recovery**: Step-by-step recovery from problematic configurations
- **Diagnostic Information**: Built-in diagnostic tools for configuration troubleshooting
- **Expert Support**: Clear pathways to expert support for complex issues
- **Configuration Export**: Backup and sharing capabilities for stable configurations

## Bevy Implementation Details

### Component Architecture

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect)]
pub struct HotkeyRecorder {
    pub recording: bool,
    pub current_combination: Option<KeyCombination>,
    pub target_setting: String,
}

#[derive(Component, Reflect)]  
pub struct ThemeSelector {
    pub current_theme: Theme,
    pub follow_system: bool,
    pub available_themes: Vec<ThemeOption>,
}

#[derive(Component, Reflect)]
pub struct WindowModeSelector {
    pub current_mode: WindowMode,
    pub preview_entities: HashMap<WindowMode, Entity>,
}

#[derive(Resource, Reflect)]
pub struct GeneralSettings {
    pub launch_at_login: bool,
    pub raycast_hotkey: KeyCombination,
    pub show_in_menu_bar: bool,
    pub text_size: TextSize,
    pub theme: Theme,
    pub follow_system_appearance: bool,
    pub window_mode: WindowMode,
    pub show_favorites_in_compact: bool,
}

#[derive(Event)]
pub enum GeneralSettingsEvent {
    HotkeyRecordingStarted,
    HotkeyRecorded(KeyCombination),
    ThemeChanged(Theme),
    WindowModeChanged(WindowMode),
    SystemAppearanceToggled(bool),
}
```

### Hotkey Recording System

```rust
fn hotkey_recording_system(
    mut recorder_query: Query<&mut HotkeyRecorder>,
    mut button_query: Query<(&Interaction, &HotkeyButton), Changed<Interaction>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<GeneralSettingsEvent>,
) {
    // Handle record button clicks
    for (interaction, button) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            for mut recorder in recorder_query.iter_mut() {
                recorder.recording = true;
                recorder.target_setting = button.setting_name.clone();
                events.send(GeneralSettingsEvent::HotkeyRecordingStarted);
            }
        }
    }
    
    // Capture key combinations during recording
    for mut recorder in recorder_query.iter_mut() {
        if recorder.recording {
            if keyboard_input.just_pressed(KeyCode::Escape) {
                recorder.recording = false;
                recorder.current_combination = None;
            } else if let Some(combination) = capture_key_combination(&keyboard_input) {
                recorder.current_combination = Some(combination.clone());
                events.send(GeneralSettingsEvent::HotkeyRecorded(combination));
                recorder.recording = false;
            }
        }
    }
}

#[derive(Component)]
pub struct HotkeyButton {
    pub setting_name: String,
}
```

### Theme System Implementation

```rust
fn theme_system(
    mut theme_selectors: Query<&mut ThemeSelector>,
    mut dropdown_query: Query<(&Interaction, &ThemeDropdown), Changed<Interaction>>,
    mut checkbox_query: Query<(&Interaction, &SystemAppearanceCheckbox), Changed<Interaction>>,
    mut events: EventWriter<GeneralSettingsEvent>,
) {
    // Handle theme dropdown
    for (interaction, dropdown) in dropdown_query.iter() {
        if *interaction == Interaction::Pressed {
            for mut selector in theme_selectors.iter_mut() {
                selector.current_theme = dropdown.theme;
                events.send(GeneralSettingsEvent::ThemeChanged(dropdown.theme));
            }
        }
    }
    
    // Handle system appearance checkbox
    for (interaction, _checkbox) in checkbox_query.iter() {
        if *interaction == Interaction::Pressed {
            for mut selector in theme_selectors.iter_mut() {
                selector.follow_system = !selector.follow_system;
                events.send(GeneralSettingsEvent::SystemAppearanceToggled(selector.follow_system));
            }
        }
    }
}

#[derive(Component)]
pub struct ThemeDropdown {
    pub theme: Theme,
}

#[derive(Component)]
pub struct SystemAppearanceCheckbox;
```

### Window Mode Selection with Flex Layout

```rust
fn spawn_general_menu_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(24.0)),
            overflow: Overflow::clip_y(),
            ..default()
        })
        .with_children(|parent| {
            // Startup Settings
            spawn_startup_section(parent);
            
            // Hotkey Configuration 
            spawn_hotkey_section(parent);
            
            // Theme Selection
            spawn_theme_section(parent);
            
            // Window Mode Selection
            spawn_window_mode_section(parent);
        });
}

fn spawn_window_mode_section(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(32.0)),
        max_width: Val::Px(600.0), // Constrain width to prevent expansion
        ..default()
    }).with_children(|parent| {
        // Section title
        parent.spawn((
            Text::new("Window Mode"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(16.0)), ..default() },
        ));
        
        // Mode selection cards
        parent.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(16.0),
            align_items: AlignItems::FlexStart,
            ..default()
        }).with_children(|parent| {
            // Default Mode Card
            spawn_mode_card(parent, WindowMode::Default, "Default Mode", true);
            
            // Compact Mode Card  
            spawn_mode_card(parent, WindowMode::Compact, "Compact Mode", false);
        });
    });
}

fn spawn_mode_card(parent: &mut ChildBuilder, mode: WindowMode, title: &str, selected: bool) {
    let bg_color = if selected {
        Color::srgb(0.2, 0.4, 0.8) // Blue for selected
    } else {
        Color::srgb(0.15, 0.15, 0.15)
    };
    
    parent.spawn((
        Node {
            width: Val::Px(200.0),
            height: Val::Px(150.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            overflow: Overflow::clip(), // Prevent expansion
            flex_grow: 0.0, // Prevent expansion
            ..default()
        },
        BackgroundColor(bg_color),
        BorderColor(if selected { Color::srgb(0.3, 0.5, 1.0) } else { Color::srgb(0.3, 0.3, 0.3) }),
        Button,
        Interaction::default(),
        WindowModeButton { mode },
    )).with_children(|parent| {
        // Mode title
        parent.spawn((
            Text::new(title),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(12.0)), ..default() },
        ));
        
        // Mock UI preview (simplified representation)
        parent.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(80.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            flex_grow: 0.0, // Prevent expansion
            ..default()
        }).with_children(|parent| {
            // This would contain a preview of the window mode
            parent.spawn((
                Text::new("UI Preview"),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
    });
}

#[derive(Component)]
pub struct WindowModeButton {
    pub mode: WindowMode,
}
```

### SystemSet Organization

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GeneralMenuSystems {
    Input,
    HotkeyRecording,
    ThemeManagement,
    WindowModeSelection,
    Settings,
    UI,
}

impl Plugin for GeneralMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<HotkeyRecorder>()
            .register_type::<ThemeSelector>() 
            .register_type::<WindowModeSelector>()
            .register_type::<GeneralSettings>()
            
            .init_resource::<GeneralSettings>()
            
            .add_event::<GeneralSettingsEvent>()
            
            .add_systems(Update, (
                (
                    mouse_interaction_system,
                    keyboard_input_system,
                ).in_set(GeneralMenuSystems::Input),
                
                (
                    hotkey_recording_system,
                    hotkey_validation_system,
                ).in_set(GeneralMenuSystems::HotkeyRecording),
                
                (
                    theme_system,
                    theme_application_system,
                ).in_set(GeneralMenuSystems::ThemeManagement),
                
                (
                    window_mode_system,
                    mode_preview_system,
                ).in_set(GeneralMenuSystems::WindowModeSelection),
                
                (
                    settings_persistence_system,
                    startup_integration_system,
                ).in_set(GeneralMenuSystems::Settings),
                
                (
                    ui_update_system,
                    button_animation_system,
                ).in_set(GeneralMenuSystems::UI).run_if(any_component_changed::<Interaction>()),
            ));
    }
}
```
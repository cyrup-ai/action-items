# AI Menu Specification

## Overview
The AI Menu is a comprehensive settings interface for configuring artificial intelligence features within the launcher application. This interface follows Raycast's design patterns with a tabbed navigation system and split-pane layout.

**Window Specifications:**
- **Title**: "Raycast Settings" in window chrome
- **Theme**: Dark theme throughout with consistent spacing
- **Chrome**: Standard macOS window controls (red/yellow/green traffic lights)

## Layout Architecture

### Navigation Bar
- **Position**: Top of window
- **Style**: Horizontal tab navigation with icons and text labels
- **Tab Count**: 8 total tabs
- **Tabs**: General (gear icon), Extensions (puzzle piece), AI (sparkle/star - active), Cloud Sync (cloud), Account (user), Organizations (building), Advanced (tools/wrench), About (info)
- **Active State**: AI tab highlighted with darker background color
- **Inactive State**: Light gray text and icons
- **Layout**: Icon above text label, centered within tab bounds
- **Interaction**: Click to switch tabs, updates entire right panel content

### Split Pane Layout
- **Left Pane**: Branding and primary toggle (40% width)
- **Right Pane**: Detailed configuration options (60% width)
- **Separator**: Subtle vertical divider

## Left Pane Components

### Privacy Indicators Bar
- **Position**: Top of left pane
- **Elements**:
  - Full control indicator with minus icon
  - No collection indicator with lock icon  
  - Encrypted indicator with shield icon
  - Info icon for additional details
- **Styling**: Dark background with subtle borders

### Branding Section
- **Container**: Circular avatar/logo container with dark gradient background
- **Logo**: Red diamond/star shape with sparkle effects (appears to have multiple red diamonds)
- **Title**: "Raycast AI" in large, white, bold typography - centered
- **Description**: "Unlock the power of AI on your Mac. Write smarter, code faster, and answer questions quicker with Raycast AI."
- **Text Color**: Medium gray for description
- **Alignment**: All elements centered vertically and horizontally
- **Styling**: Dark gradient background, elegant spacing between elements

### Primary Toggle
- **Component**: Large toggle switch (iOS-style)
- **Current State**: ON (blue background, white circle positioned right)
- **OFF State**: Gray background, white circle positioned left
- **Function**: Master enable/disable for entire AI system
- **Position**: Below branding section, centered
- **Animation**: Smooth slide transition between states
- **Styling**: Rounded rectangle background with circular toggle button

### Footer Actions
- **Button 1**: "Open AI Help Manual" 
  - **Style**: Underlined text link
  - **Color**: Light gray/white text
- **Button 2**: "Discover Raycast AI"
  - **Style**: Standard button with background
  - **Color**: Darker background than surrounding area
- **Layout**: Two buttons horizontally arranged
- **Position**: Bottom of left pane with appropriate padding

## Right Pane Configuration Sections

### Quick AI Section
- **Title**: "Quick AI"
- **Description**: "Get instant AI responses directly from the root search"
- **Layout**: Two-column layout with labels left, controls right
- **Components**:
  - **Trigger Label**: "Trigger" 
  - **Trigger Dropdown**: "Tab to Ask AI" (current selection)
    - **Style**: Dark background dropdown with down arrow
    - **Width**: Spans majority of right column
  - **Info Icon**: Circular "i" button for contextual help
  - **Checkbox**: "Show Ask AI hint in root search"
    - **Current State**: CHECKED (visible checkmark)
    - **Label Color**: White/light gray

### AI Model Configuration
- **Label**: "Quick AI Model" 
- **Components**:
  - **Model Dropdown**: "Sonar Reasoning Pro" (current selection)
    - **Icon**: Provider-specific icon displayed left of text
    - **Style**: Dark background dropdown with down arrow
  - **Info Icon**: Circular "i" button for model details and capabilities
  - **Web Search Checkbox**: "Web Search" capability toggle
    - **Current State**: CHECKED (visible checkmark)
    - **Position**: Below model dropdown, left-aligned with checkbox

### Default Actions
- **Label**: "Default Primary Action"
- **Components**:
  - **Action Dropdown**: "Paste Response to Active App" (current selection)
    - **Style**: Dark background dropdown with down arrow
    - **Width**: Full width of right column
- **Functionality**: Defines primary action when AI completes response
- **Layout**: Two-column with label left, dropdown right

### AI Chat Section
- **Title**: "AI Chat"
- **Description**: "Dedicated chat window for longer conversations with AI"
- **Components**:
  - **Hotkey Display Field**: 
    - **Current Value**: "⌘ ⌃ ⌥ L" (Command + Control + Option + L)
    - **Style**: Dark input field with keyboard symbols
    - **Clear Button**: "X" button on right side to remove assignment
    - **Background**: Darker than surrounding controls
  - **Start New Chat Label**: "Start New Chat"
  - **Timeout Dropdown**: "After 30 minutes" (current selection)
    - **Style**: Standard dark dropdown
  - **Info Icon**: Circular "i" button for chat behavior explanation
- **Layout**: Two-column layout consistent with other sections

### Provider Configuration
- **Label**: "New Chat Settings"
- **Components**:
  - **Provider Dropdown**: "CYRUP (openai)" (current selection)
    - **Provider Icon**: Circular icon with distinctive branding (appears to be CYRUP logo)
    - **Style**: Dark background dropdown with provider icon left of text
- **Layout**: Two-column layout consistent with other sections

### Text Preferences
- **Label**: "Text Size"
- **Components**:
  - **Small Text Button**: "Aa" (smaller font size)
  - **Large Text Button**: "Aa" (larger font size)
  - **Layout**: Two buttons horizontally arranged
  - **Selection State**: Visual indication showing which size is currently active
  - **Functionality**: Controls text size throughout the interface

## Visual Design Specifications

### Color Palette
- **Background**: Dark theme (#1a1a1a or similar)
- **Text Primary**: White/light gray (#ffffff or #f0f0f0)
- **Text Secondary**: Medium gray (#888888 or similar) for descriptions
- **Accent Blue**: Toggle switch active state, link colors
- **Control Backgrounds**: Darker gray (#2a2a2a or similar) for dropdowns/inputs
- **Borders**: Subtle dark borders for separation

### Typography Hierarchy
- **Section Titles**: Medium weight, white/light gray
- **Setting Labels**: Regular weight, white/light gray
- **Descriptions**: Regular weight, medium gray, smaller size
- **Control Text**: Regular weight, white text within controls
- **Branding Title**: Large, bold, white ("Raycast AI")

### Spacing & Layout
- **Section Spacing**: Consistent vertical spacing between sections
- **Two-Column Layout**: Labels left-aligned, controls right-aligned
- **Control Width**: Dropdowns span majority of right column width
- **Padding**: Consistent internal padding within controls
- **Margins**: Appropriate margins around grouped elements

### Control States
- **Default State**: Dark backgrounds with subtle borders
- **Hover State**: Slightly lighter backgrounds (implementation detail)
- **Active/Selected State**: Checkmarks visible, toggle positioned right
- **Focus State**: Visual focus indicators for keyboard navigation
- **Disabled State**: Grayed out appearance when master toggle is off

## Functional Requirements

### State Management
- All settings must persist across application sessions
- Toggle states affect dependent controls (cascade disabling)
- Dropdown selections maintain consistency with provider capabilities
- Hotkey assignments must validate against system conflicts

### Hotkey System
- **Quick AI Trigger**: Configurable key combination for instant AI access
- **Chat Window**: Dedicated hotkey for opening AI chat interface
- **Validation**: Real-time conflict detection with existing system shortcuts
- **Display**: Visual representation of assigned keys with modifier symbols

### Provider Integration
- **Dynamic Loading**: Provider list populated from available AI services
- **Authentication**: Seamless integration with provider authentication systems
- **Capability Detection**: Automatic feature availability based on provider
- **Branding**: Provider-specific icons and naming conventions

### Privacy & Security
- **Data Collection Indicators**: Clear visual status of data handling
- **Encryption Status**: Real-time display of data protection level
- **Control Level**: User agency over data sharing and processing
- **Audit Trail**: Logging of privacy setting changes

## Bevy Implementation Examples

### Navigation Tabs
- Reference: `./docs/bevy/examples/ui/button.rs` - Tab button interactions
- Reference: `./docs/bevy/examples/ui/ui.rs` - Tab container layout

### Toggle Components  
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Toggle state sprites
- Reference: `./docs/bevy/examples/animation/animated_fox.rs` - Toggle animations

### Dropdown Menus
- Reference: `./docs/bevy/examples/ui/ui_texture_slice.rs` - Dropdown panel backgrounds
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Dropdown interaction handling

### Split Pane Layout
- Reference: `./docs/bevy/examples/ui/flex_layout.rs` - Flexible pane sizing
- Reference: `./docs/bevy/examples/ui/ui.rs` - Container hierarchy

### Hotkey Display
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Key combination capture
- Reference: `./docs/bevy/examples/ui/text.rs` - Keyboard symbol rendering

### State Persistence
- Reference: `./docs/bevy/examples/app/return_after_run.rs` - Application state management
- Reference: `./docs/bevy/examples/reflection/reflection.rs` - Setting serialization

### Provider Integration
- Reference: `./docs/bevy/examples/async_tasks/async_compute.rs` - Asynchronous provider communication
- Reference: `./docs/bevy/examples/games/loading_screen.rs` - Loading states during provider initialization

## Animation Requirements

### Micro-Interactions
- **Toggle Switch**: Smooth slide animation with spring physics
- **Dropdown Expansion**: Smooth height transition with easing
- **Tab Switching**: Subtle highlight transitions
- **Hover States**: Gentle color and scale transformations

### Loading States
- **Provider Connection**: Animated spinner during authentication
- **Model Loading**: Progress indication for large model initialization
- **Setting Validation**: Visual feedback for configuration changes

## Accessibility Requirements

### Keyboard Navigation
- **Tab Order**: Logical progression through all interactive elements
- **Keyboard Shortcuts**: Alt-key access to primary controls
- **Focus Indicators**: Clear visual focus states for all controls

### Screen Reader Support
- **Labels**: Descriptive labels for all form controls
- **State Announcements**: Clear indication of toggle and selection states
- **Section Headers**: Proper heading hierarchy for screen readers

### Visual Accessibility
- **Color Contrast**: WCAG AA compliance for all text elements
- **Size Options**: Text size controls affect entire interface
- **High Contrast Mode**: Alternative color schemes for visibility needs

## Bevy Implementation Details

### Component Architecture

```rust
use bevy::{prelude::*, ui::FocusPolicy};

// Main AI Menu container marker
#[derive(Component, Reflect)]
pub struct AiMenu;

// Navigation tab components
#[derive(Component, Reflect)]
pub struct NavigationTab {
    pub tab_type: TabType,
    pub is_active: bool,
}

#[derive(Component, Reflect, Clone, Copy, PartialEq)]
pub enum TabType {
    General,
    Extensions,
    AI,
    CloudSync,
    Account,
    Organizations,
    Advanced,
    About,
}

// AI-specific components
#[derive(Component, Reflect)]
pub struct AiToggleSwitch {
    pub is_enabled: bool,
    pub animation_progress: f32,
}

#[derive(Component, Reflect)]
pub struct QuickAiSection;

#[derive(Component, Reflect)]
pub struct AiModelDropdown {
    pub current_model: String,
    pub is_expanded: bool,
    pub available_models: Vec<String>,
}

#[derive(Component, Reflect)]
pub struct HotkeyDisplay {
    pub modifiers: Vec<KeyCode>,
    pub key: KeyCode,
    pub is_editing: bool,
}

#[derive(Component, Reflect)]
pub struct PrivacyIndicator {
    pub indicator_type: PrivacyType,
    pub is_active: bool,
}

#[derive(Component, Reflect, Clone, Copy)]
pub enum PrivacyType {
    FullControl,
    NoCollection,
    Encrypted,
}

// Text size preference component
#[derive(Component, Reflect)]
pub struct TextSizeButton {
    pub size_type: TextSizeType,
    pub is_selected: bool,
}

#[derive(Component, Reflect, Clone, Copy, PartialEq)]
pub enum TextSizeType {
    Small,
    Large,
}
```

### Resource Management

```rust
// Global AI menu state resource
#[derive(Resource, Reflect, Default)]
pub struct AiMenuState {
    pub active_tab: TabType,
    pub master_ai_enabled: bool,
    pub quick_ai_enabled: bool,
    pub current_model: String,
    pub web_search_enabled: bool,
    pub chat_hotkey: Vec<KeyCode>,
    pub chat_timeout_minutes: u32,
    pub selected_provider: String,
    pub text_size: TextSizeType,
}

// Provider configuration resource
#[derive(Resource, Reflect)]
pub struct AiProviderRegistry {
    pub providers: Vec<AiProvider>,
    pub active_provider: Option<String>,
}

#[derive(Clone, Reflect)]
pub struct AiProvider {
    pub name: String,
    pub display_name: String,
    pub icon_path: String,
    pub models: Vec<AiModel>,
    pub capabilities: ProviderCapabilities,
}

#[derive(Clone, Reflect)]
pub struct AiModel {
    pub id: String,
    pub display_name: String,
    pub supports_web_search: bool,
}

#[derive(Clone, Reflect)]
pub struct ProviderCapabilities {
    pub chat: bool,
    pub quick_ai: bool,
    pub web_search: bool,
}
```

### Event System

```rust
// AI Menu specific events
#[derive(Event, Reflect)]
pub enum AiMenuEvent {
    TabChanged(TabType),
    ToggleMasterAi(bool),
    ModelChanged(String),
    HotkeyUpdated(Vec<KeyCode>),
    ProviderChanged(String),
    TextSizeChanged(TextSizeType),
}

#[derive(Event, Reflect)]
pub struct SettingsChangedEvent {
    pub setting: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Event, Reflect)]
pub struct ProviderAuthenticationEvent {
    pub provider: String,
    pub status: AuthStatus,
}

#[derive(Reflect, Clone)]
pub enum AuthStatus {
    Pending,
    Success,
    Failed(String),
}
```

### System Architecture with SystemSets

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AiMenuSystems {
    Input,
    StateUpdate,
    Animation,
    Rendering,
}

impl Plugin for AiMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<AiMenuState>()
            .init_resource::<AiProviderRegistry>()
            
            // Events
            .add_event::<AiMenuEvent>()
            .add_event::<SettingsChangedEvent>()
            .add_event::<ProviderAuthenticationEvent>()
            
            // Configure system sets with ordering
            .configure_sets(Update, (
                AiMenuSystems::Input,
                AiMenuSystems::StateUpdate,
                AiMenuSystems::Animation,
                AiMenuSystems::Rendering,
            ).chain())
            
            // Systems
            .add_systems(Startup, setup_ai_menu)
            .add_systems(Update, (
                // Input handling systems
                handle_tab_clicks,
                handle_toggle_interactions,
                handle_dropdown_interactions,
                handle_hotkey_input,
                handle_text_size_buttons,
            ).in_set(AiMenuSystems::Input))
            
            .add_systems(Update, (
                // State update systems
                update_menu_state,
                validate_settings,
                sync_provider_changes,
            ).in_set(AiMenuSystems::StateUpdate))
            
            .add_systems(Update, (
                // Animation systems
                animate_toggle_switch,
                animate_dropdown_transitions,
                animate_tab_highlights,
            ).in_set(AiMenuSystems::Animation))
            
            .add_systems(Update, (
                // Rendering systems
                update_toggle_visuals,
                update_dropdown_content,
                update_privacy_indicators,
                apply_text_size_changes,
            ).in_set(AiMenuSystems::Rendering));
    }
}
```

### Flex-Based UI Layout Implementation

```rust
fn setup_ai_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Root container with constrained flex growth
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            max_width: Val::Px(1200.0), // Prevent excessive expansion
            flex_direction: FlexDirection::Column,
            flex_grow: 0.0, // CRITICAL: Prevent expansion
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        AiMenu,
    )).with_children(|parent| {
        
        // Navigation bar with constrained height
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                max_height: Val::Px(60.0), // Prevent growth
                flex_direction: FlexDirection::Row,
                flex_grow: 0.0,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        )).with_children(|nav_parent| {
            spawn_navigation_tabs(nav_parent, &asset_server);
        });
        
        // Split pane container with proper flex constraints
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                max_height: Val::Px(800.0), // Constrain maximum height
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0, // Allow growth within constraints
                overflow: Overflow::clip(), // Prevent content overflow
                ..default()
            },
        )).with_children(|split_parent| {
            
            // Left pane (40% with constraints)
            split_parent.spawn((
                Node {
                    width: Val::Percent(40.0),
                    max_width: Val::Px(480.0), // Prevent excessive width
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    flex_grow: 0.0, // No growth
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
            )).with_children(|left_parent| {
                spawn_left_pane_content(left_parent, &asset_server);
            });
            
            // Right pane (60% with constraints)
            split_parent.spawn((
                Node {
                    width: Val::Percent(60.0),
                    max_width: Val::Px(720.0), // Prevent excessive width
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    flex_grow: 0.0, // No growth
                    padding: UiRect::all(Val::Px(20.0)),
                    overflow: Overflow::clip_y(), // Handle vertical overflow
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            )).with_children(|right_parent| {
                spawn_right_pane_content(right_parent, &asset_server);
            });
        });
    });
}

fn spawn_navigation_tabs(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    let tabs = [
        (TabType::General, "General", "icons/gear.png"),
        (TabType::Extensions, "Extensions", "icons/puzzle.png"),
        (TabType::AI, "AI", "icons/sparkle.png"),
        (TabType::CloudSync, "Cloud Sync", "icons/cloud.png"),
        (TabType::Account, "Account", "icons/user.png"),
        (TabType::Organizations, "Organizations", "icons/building.png"),
        (TabType::Advanced, "Advanced", "icons/wrench.png"),
        (TabType::About, "About", "icons/info.png"),
    ];
    
    for (tab_type, label, icon_path) in tabs {
        parent.spawn((
            Button,
            Node {
                width: Val::Percent(12.5), // Equal distribution
                height: Val::Percent(100.0),
                max_width: Val::Px(150.0), // Constrain tab width
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                ..default()
            },
            BackgroundColor(
                if tab_type == TabType::AI {
                    Color::srgb(0.2, 0.2, 0.2) // Active state
                } else {
                    Color::srgb(0.15, 0.15, 0.15) // Inactive state
                }
            ),
            NavigationTab {
                tab_type,
                is_active: tab_type == TabType::AI,
            },
        )).with_children(|tab_parent| {
            // Tab icon
            tab_parent.spawn((
                ImageNode::new(asset_server.load(icon_path)),
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));
            
            // Tab label
            tab_parent.spawn((
                Text::new(label),
                TextFont {
                    font: asset_server.load("fonts/Inter-Regular.ttf"),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
    }
}
```

### Toggle Switch Animation System

```rust
fn animate_toggle_switch(
    time: Res<Time>,
    mut toggle_query: Query<(&mut AiToggleSwitch, &mut Node, &Children), Changed<AiToggleSwitch>>,
    mut transform_query: Query<&mut Transform>,
) {
    for (mut toggle, mut node, children) in toggle_query.iter_mut() {
        let target_progress = if toggle.is_enabled { 1.0 } else { 0.0 };
        let animation_speed = 8.0; // Smooth animation
        
        if (toggle.animation_progress - target_progress).abs() > 0.01 {
            toggle.animation_progress = toggle.animation_progress
                .lerp(target_progress, animation_speed * time.delta_secs());
            
            // Update visual state
            let background_color = Color::srgb(
                0.2 + (toggle.animation_progress * 0.3),
                0.2 + (toggle.animation_progress * 0.5),
                0.8 * toggle.animation_progress + 0.2,
            );
            
            // Find and animate the toggle circle
            for &child in children.iter() {
                if let Ok(mut transform) = transform_query.get_mut(child) {
                    transform.translation.x = -15.0 + (30.0 * toggle.animation_progress);
                }
            }
        }
    }
}
```

### Event Handling with Changed Detection

```rust
fn handle_toggle_interactions(
    mut interaction_query: Query<
        (&Interaction, &mut AiToggleSwitch), 
        (Changed<Interaction>, With<AiToggleSwitch>)
    >,
    mut menu_state: ResMut<AiMenuState>,
    mut menu_events: EventWriter<AiMenuEvent>,
) {
    for (interaction, mut toggle) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            toggle.is_enabled = !toggle.is_enabled;
            menu_state.master_ai_enabled = toggle.is_enabled;
            
            menu_events.write(AiMenuEvent::ToggleMasterAi(toggle.is_enabled));
        }
    }
}

fn handle_dropdown_interactions(
    mut interaction_query: Query<
        (Entity, &Interaction, &mut AiModelDropdown),
        Changed<Interaction>
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (entity, interaction, mut dropdown) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                dropdown.is_expanded = !dropdown.is_expanded;
                
                if dropdown.is_expanded {
                    spawn_dropdown_options(&mut commands, entity, &dropdown, &asset_server);
                } else {
                    despawn_dropdown_options(&mut commands, entity);
                }
            }
            _ => {}
        }
    }
}
```

### State Management and Persistence

```rust
fn update_menu_state(
    mut menu_events: EventReader<AiMenuEvent>,
    mut menu_state: ResMut<AiMenuState>,
    mut settings_events: EventWriter<SettingsChangedEvent>,
) {
    for event in menu_events.read() {
        match event {
            AiMenuEvent::TabChanged(tab_type) => {
                let old_tab = format!("{:?}", menu_state.active_tab);
                menu_state.active_tab = *tab_type;
                
                settings_events.write(SettingsChangedEvent {
                    setting: "active_tab".to_string(),
                    old_value: old_tab,
                    new_value: format!("{:?}", tab_type),
                });
            }
            AiMenuEvent::ToggleMasterAi(enabled) => {
                menu_state.master_ai_enabled = *enabled;
                
                settings_events.write(SettingsChangedEvent {
                    setting: "master_ai_enabled".to_string(),
                    old_value: (!enabled).to_string(),
                    new_value: enabled.to_string(),
                });
            }
            AiMenuEvent::ModelChanged(model) => {
                let old_model = menu_state.current_model.clone();
                menu_state.current_model = model.clone();
                
                settings_events.write(SettingsChangedEvent {
                    setting: "current_model".to_string(),
                    old_value: old_model,
                    new_value: model.clone(),
                });
            }
            _ => {}
        }
    }
}
```

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AiMenuPlugin,
        ));
        app
    }
    
    #[test]
    fn test_ai_menu_initialization() {
        let mut app = setup_test_app();
        
        // Spawn AI menu
        app.world_mut().spawn((AiMenu,));
        app.update();
        
        // Verify menu state is initialized
        let menu_state = app.world().resource::<AiMenuState>();
        assert_eq!(menu_state.active_tab, TabType::AI);
        assert!(!menu_state.master_ai_enabled); // Default disabled
    }
    
    #[test]
    fn test_toggle_state_changes() {
        let mut app = setup_test_app();
        
        // Spawn toggle component
        app.world_mut().spawn((
            AiToggleSwitch {
                is_enabled: false,
                animation_progress: 0.0,
            },
        ));
        
        // Send toggle event
        app.world_mut().resource_mut::<Events<AiMenuEvent>>()
            .write(AiMenuEvent::ToggleMasterAi(true));
        
        app.update();
        
        // Verify state changed
        let menu_state = app.world().resource::<AiMenuState>();
        assert!(menu_state.master_ai_enabled);
    }
    
    #[test]
    fn test_dropdown_model_selection() {
        let mut app = setup_test_app();
        
        // Test model change event
        app.world_mut().resource_mut::<Events<AiMenuEvent>>()
            .write(AiMenuEvent::ModelChanged("GPT-4".to_string()));
        
        app.update();
        
        let menu_state = app.world().resource::<AiMenuState>();
        assert_eq!(menu_state.current_model, "GPT-4");
    }
}
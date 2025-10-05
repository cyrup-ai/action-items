# General Menu - UI Layout Architecture

## Task: Implement Core UI Layout System for General Settings

### File: `ui/src/settings/general/layout.rs` (new file)

Create the fundamental layout architecture for the General Menu interface with optimal Bevy UI patterns and zero-allocation rendering.

### Implementation Requirements

#### Main Layout Container
- File: `ui/src/settings/general/layout.rs` (line 1-67)
- Implement `GeneralMenuLayout` struct with vertical configuration sections
- Use Bevy's `NodeBundle` with flex direction column
- Consistent spacing between configuration groups
- Responsive design for different window sizes

#### Configuration Section Architecture
```rust
#[derive(Component)]
pub struct GeneralMenuLayout {
    pub startup_section: Entity,
    pub hotkey_section: Entity,
    pub menu_bar_section: Entity,
    pub text_size_section: Entity,
    pub theme_section: Entity,
    pub window_mode_section: Entity,
    pub favorites_section: Entity,
}
```

#### Section Layout System
- File: `ui/src/settings/general/sections.rs` (new file, line 1-134)
- Implement `ConfigurationSection` component for consistent section styling
- Label-control pairs with left-aligned labels and right-aligned controls
- Visual hierarchy with proper spacing and typography
- Rounded corner implementation for visual elements

#### Layout Spawning System
- File: `ui/src/settings/general/spawn.rs` (new file, line 1-89)
- Implement `spawn_general_menu_layout()` function
- Entity hierarchy management with proper parent-child relationships
- Component attachment for state management
- Integration with existing UI systems

### Architecture Notes
- Use Bevy's `Style` component for all layout positioning
- Implement responsive design with `Size`, `Position`, and `Margin`
- Zero-allocation entity spawning patterns
- Integration with `ui/src/ui/systems.rs` (line 245-289)

### Integration Points
- Main settings window system in `app/src/window/` (line 67-89)
- Theme system integration for dynamic styling
- Input event handling integration
- State change reactivity system

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Bevy Implementation Details

### Flex-Based UI Layout Components

```rust
// Core layout component with proper flex configuration
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct GeneralMenuLayout {
    pub startup_section: Entity,
    pub hotkey_section: Entity,
    pub menu_bar_section: Entity,
    pub text_size_section: Entity,
    pub theme_section: Entity,
    pub window_mode_section: Entity,
    pub favorites_section: Entity,
    pub scroll_view: Entity,
    pub main_container: Entity,
}

// Configuration section component with flex constraints
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct ConfigurationSection {
    pub section_type: SectionType,
    pub visible: bool,
    pub expanded: bool,
    pub animation_progress: f32,
    pub label_entity: Entity,
    pub control_entity: Entity,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SectionType {
    Startup,
    Hotkey,
    MenuBar,
    TextSize,
    Theme,
    WindowMode,
    Favorites,
}
```

### Flex Layout System with Proper Constraints

```rust
// System for spawning the main layout with flex constraints
fn spawn_general_menu_layout(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<CurrentTheme>,
) {
    let font = asset_server.load("fonts/Inter-Regular.ttf");
    
    // Main container with proper flex constraints to prevent expansion
    let main_container = commands.spawn((
        Node {
            // Use percentages with max constraints to prevent expansion
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            max_width: Val::Px(600.0), // Prevent horizontal expansion
            max_height: Val::Px(800.0), // Prevent vertical expansion
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(16.0), // Consistent spacing between sections
            flex_grow: 0.0, // CRITICAL: Prevent expansion
            ..default()
        },
        BackgroundColor(theme.background_primary),
        GeneralMenuContainer,
    )).id();
    
    // Scrollable content area with overflow handling
    let scroll_view = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            max_width: Val::Px(560.0), // Constrain content width
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::FlexStart,
            row_gap: Val::Px(12.0),
            overflow: Overflow::clip(), // Handle content overflow
            flex_grow: 0.0, // Prevent expansion
            ..default()
        },
        ScrollView,
    )).id();
    
    commands.entity(main_container).add_child(scroll_view);
    
    // Spawn configuration sections with proper flex constraints
    let sections = spawn_configuration_sections(&mut commands, scroll_view, &font, &theme);
    
    // Create layout component
    commands.spawn(GeneralMenuLayout {
        startup_section: sections.startup,
        hotkey_section: sections.hotkey,
        menu_bar_section: sections.menu_bar,
        text_size_section: sections.text_size,
        theme_section: sections.theme,
        window_mode_section: sections.window_mode,
        favorites_section: sections.favorites,
        scroll_view,
        main_container,
    });
}

// Helper struct for section entities
struct SectionEntities {
    startup: Entity,
    hotkey: Entity,
    menu_bar: Entity,
    text_size: Entity,
    theme: Entity,
    window_mode: Entity,
    favorites: Entity,
}
```

### Configuration Section Layout with Flex

```rust
// Function to spawn individual configuration sections
fn spawn_configuration_sections(
    commands: &mut Commands,
    parent: Entity,
    font: &Handle<Font>,
    theme: &CurrentTheme,
) -> SectionEntities {
    let startup_section = spawn_config_section(
        commands, 
        parent, 
        SectionType::Startup,
        "Launch at Login",
        font,
        theme
    );
    
    let hotkey_section = spawn_config_section(
        commands,
        parent,
        SectionType::Hotkey, 
        "Global Hotkey",
        font,
        theme
    );
    
    // Spawn remaining sections...
    
    SectionEntities {
        startup: startup_section,
        hotkey: hotkey_section,
        menu_bar: Entity::PLACEHOLDER, // TODO: Implement
        text_size: Entity::PLACEHOLDER,
        theme: Entity::PLACEHOLDER,
        window_mode: Entity::PLACEHOLDER,
        favorites: Entity::PLACEHOLDER,
    }
}

// Individual section spawning with proper flex layout
fn spawn_config_section(
    commands: &mut Commands,
    parent: Entity,
    section_type: SectionType,
    label_text: &str,
    font: &Handle<Font>,
    theme: &CurrentTheme,
) -> Entity {
    // Section container with constrained flex layout
    let section_container = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Auto, // Auto height based on content
            max_height: Val::Px(80.0), // Prevent excessive height
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect {
                left: Val::Px(16.0),
                right: Val::Px(16.0),
                top: Val::Px(12.0),
                bottom: Val::Px(12.0),
            },
            border: UiRect::all(Val::Px(1.0)),
            flex_grow: 0.0, // Prevent expansion
            flex_shrink: 1.0, // Allow shrinking if needed
            ..default()
        },
        BackgroundColor(theme.background_secondary),
        BorderColor(theme.border_primary),
        BorderRadius::all(Val::Px(8.0)),
        ConfigurationSection {
            section_type: section_type.clone(),
            visible: true,
            expanded: false,
            animation_progress: 0.0,
            label_entity: Entity::PLACEHOLDER,
            control_entity: Entity::PLACEHOLDER,
        },
    )).id();
    
    // Label with constrained width to prevent expansion
    let label_entity = commands.spawn((
        Node {
            width: Val::Auto,
            max_width: Val::Px(200.0), // Prevent label from expanding too wide
            height: Val::Auto,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            flex_grow: 0.0, // Don't expand
            ..default()
        },
        Text::new(label_text),
        TextFont {
            font: font.clone(),
            font_size: 14.0,
        },
        TextColor(theme.text_primary),
    )).id();
    
    // Control container with proper constraints
    let control_entity = commands.spawn((
        Node {
            width: Val::Auto,
            max_width: Val::Px(300.0), // Constrain control width
            height: Val::Auto,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            flex_grow: 0.0, // Don't expand
            ..default()
        },
        SectionControl { section_type },
    )).id();
    
    // Establish parent-child relationships
    commands.entity(section_container)
        .add_child(label_entity)
        .add_child(control_entity);
    
    commands.entity(parent).add_child(section_container);
    
    // Update section component with entity references
    commands.entity(section_container).insert(ConfigurationSection {
        section_type,
        visible: true,
        expanded: false,
        animation_progress: 0.0,
        label_entity,
        control_entity,
    });
    
    section_container
}
```

### Layout Update System with Change Detection

```rust
// System sets for layout organization
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LayoutSystemSet {
    UpdateLayout,
    HandleResize,
    AnimateTransitions,
    UpdateTheme,
}

// Layout update system using change detection for performance
fn update_layout_system(
    // Only query changed layout components
    mut changed_sections: Query<
        (&mut ConfigurationSection, &mut Node), 
        Changed<ConfigurationSection>
    >,
    window_query: Query<&Window, With<PrimaryWindow>>,
    theme: Res<CurrentTheme>,
    time: Res<Time>,
) {
    let window = window_query.single();
    let window_width = window.width();
    
    for (mut section, mut node) in changed_sections.iter_mut() {
        // Animate section expansion/collapse
        if section.expanded != (section.animation_progress > 0.5) {
            let target = if section.expanded { 1.0 } else { 0.0 };
            section.animation_progress = lerp(
                section.animation_progress,
                target,
                time.delta_seconds() * 4.0, // Animation speed
            );
            
            // Update height based on animation progress
            let base_height = 80.0;
            let expanded_height = 120.0;
            let current_height = lerp(base_height, expanded_height, section.animation_progress);
            
            node.height = Val::Px(current_height);
            node.max_height = Val::Px(current_height + 10.0); // Small buffer
        }
        
        // Responsive layout adjustments
        if window_width < 600.0 {
            // Compact layout for narrow windows
            node.flex_direction = FlexDirection::Column;
            node.align_items = AlignItems::FlexStart;
        } else {
            // Standard horizontal layout
            node.flex_direction = FlexDirection::Row;
            node.align_items = AlignItems::Center;
        }
    }
}

// Helper function for smooth interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}
```

### Window Resize Handling System

```rust
// System to handle window resize events and maintain proper constraints
fn handle_window_resize(
    mut resize_events: EventReader<WindowResized>,
    mut layout_query: Query<&mut Node, With<GeneralMenuContainer>>,
    mut section_query: Query<&mut Node, (With<ConfigurationSection>, Without<GeneralMenuContainer>)>,
) {
    for event in resize_events.read() {
        let new_width = event.width;
        let new_height = event.height;
        
        // Update main container constraints
        for mut node in layout_query.iter_mut() {
            // Adjust max width based on window size, but never exceed design limits
            node.max_width = Val::Px((new_width * 0.9).min(600.0));
            node.max_height = Val::Px((new_height * 0.9).min(800.0));
        }
        
        // Update section layouts for responsive behavior
        for mut section_node in section_query.iter_mut() {
            if new_width < 600.0 {
                // Switch to vertical layout for narrow windows
                section_node.flex_direction = FlexDirection::Column;
                section_node.align_items = AlignItems::FlexStart;
                section_node.row_gap = Val::Px(8.0);
                section_node.column_gap = Val::Px(0.0);
            } else {
                // Standard horizontal layout
                section_node.flex_direction = FlexDirection::Row;
                section_node.align_items = AlignItems::Center;
                section_node.row_gap = Val::Px(0.0);
                section_node.column_gap = Val::Px(12.0);
            }
        }
    }
}
```

### Theme Integration System

```rust
// System for applying theme changes to layout
fn apply_theme_to_layout(
    theme: Res<CurrentTheme>,
    mut background_query: Query<&mut BackgroundColor, With<GeneralMenuContainer>>,
    mut section_query: Query<(&mut BackgroundColor, &mut BorderColor), 
                           (With<ConfigurationSection>, Without<GeneralMenuContainer>)>,
    mut text_query: Query<&mut TextColor>,
) {
    if theme.is_changed() {
        // Update main container background
        for mut bg_color in background_query.iter_mut() {
            *bg_color = BackgroundColor(theme.background_primary);
        }
        
        // Update section backgrounds and borders
        for (mut bg_color, mut border_color) in section_query.iter_mut() {
            *bg_color = BackgroundColor(theme.background_secondary);
            *border_color = BorderColor(theme.border_primary);
        }
        
        // Update text colors
        for mut text_color in text_query.iter_mut() {
            *text_color = TextColor(theme.text_primary);
        }
    }
}
```

### Plugin Implementation

```rust
// Layout plugin for the general menu
pub struct GeneralMenuLayoutPlugin;

impl Plugin for GeneralMenuLayoutPlugin {
    fn build(&self, app: &mut App) {
        app
            // Component registration for Reflect
            .register_type::<GeneralMenuLayout>()
            .register_type::<ConfigurationSection>()
            .register_type::<SectionType>()
            
            // System sets configuration
            .configure_sets(
                Update,
                (
                    LayoutSystemSet::UpdateLayout,
                    LayoutSystemSet::HandleResize,
                    LayoutSystemSet::AnimateTransitions,
                    LayoutSystemSet::UpdateTheme,
                ).chain()
            )
            
            // Layout systems
            .add_systems(Update, (
                update_layout_system.in_set(LayoutSystemSet::UpdateLayout),
                handle_window_resize.in_set(LayoutSystemSet::HandleResize),
                apply_theme_to_layout.in_set(LayoutSystemSet::UpdateTheme),
            ))
            
            // Startup system to initialize layout
            .add_systems(Startup, spawn_general_menu_layout);
    }
}
```

### Testing Strategy for Layout

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::window::{WindowResized};
    
    #[test]
    fn test_flex_layout_constraints() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, GeneralMenuLayoutPlugin));
        
        // Verify that layout components have proper flex constraints
        app.update();
        
        let world = app.world();
        let layout_query = world.query::<&Node>();
        
        for node in layout_query.iter(world) {
            // Ensure flex_grow is 0.0 to prevent expansion
            assert_eq!(node.flex_grow, 0.0);
            
            // Verify max constraints are set
            assert!(matches!(node.max_width, Val::Px(_)));
            assert!(matches!(node.max_height, Val::Px(_)));
        }
    }
    
    #[test]
    fn test_window_resize_response() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, GeneralMenuLayoutPlugin));
        
        // Send resize event
        app.world_mut().send_event(WindowResized {
            window: Entity::PLACEHOLDER,
            width: 400.0,
            height: 600.0,
        });
        
        app.update();
        
        // Verify layout adapts to narrow window
        // Test assertions...
    }
}
```

This implementation provides a robust, flex-based UI layout system that properly constrains expansion while maintaining responsive behavior and smooth animations, following modern Bevy UI patterns.
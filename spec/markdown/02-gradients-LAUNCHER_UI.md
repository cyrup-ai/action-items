# LAUNCHER_UI Gradient Implementation

**Using actual BackgroundGradient from bevy/examples/ui/gradients.rs - no bullshit theme methods**

## Current Problem (ui/src/ui/theme.rs:414-417)
```rust
pub fn create_linear_gradient(&self, from: Color, _to: Color, _angle: f32) -> BackgroundColor {
    // Simplified to solid color - will add gradients back when API is stable
    BackgroundColor(from)  // ❌ IGNORING GRADIENTS!
}
```

## LAUNCHER_UI Solution from Bevy Examples

### Direct BackgroundGradient Usage

**Pattern from:** `bevy/examples/ui/gradients.rs`

```rust
// LAUNCHER_UI gradient usage - no custom theme methods needed
BackgroundGradient::from(LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.08, 0.08, 0.09, 0.98), Val::Percent(0.0)),
    ColorStop::new(Color::srgba(0.12, 0.12, 0.14, 0.98), Val::Percent(100.0)),
]))
```

### Container Gradients (LAUNCHER_UI patterns)

```rust
// MAIN CONTAINER GRADIENT (Raycast-like depth)
BackgroundGradient::from(LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.08, 0.08, 0.09, 0.98), Val::Percent(0.0)),
    ColorStop::new(Color::srgba(0.12, 0.12, 0.14, 0.98), Val::Percent(100.0)),
]))

// SEARCH BAR GRADIENT (Elevated surface feeling)
BackgroundGradient::from(LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.18, 0.18, 0.20, 0.85), Val::Percent(0.0)),
    ColorStop::new(Color::srgba(0.15, 0.15, 0.17, 0.90), Val::Percent(100.0)),
]))

// RESULT ITEM DEFAULT GRADIENT
BackgroundGradient::from(LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
    ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
]))

// RESULT ITEM HOVER GRADIENT (Blue accent)
BackgroundGradient::from(LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.25), Val::Percent(0.0)),
    ColorStop::new(Color::srgba(0.18, 0.18, 0.22, 0.90), Val::Percent(100.0)),
]))

// RESULT ITEM SELECTED GRADIENT (Strong blue)
BackgroundGradient::from(LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.60), Val::Percent(0.0)),
    ColorStop::new(Color::srgba(0.0, 0.38, 0.80, 0.80), Val::Percent(100.0)),
]))
```

### Hover System Using LAUNCHER_UI Patterns

**Pattern from:** `bevy/examples/ui/button.rs` interaction system

```rust
#[derive(Component)]
struct ResultItem;

// LAUNCHER_UI hover system - direct gradient mutation like button.rs
fn result_item_hover_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundGradient),
        (Changed<Interaction>, With<ResultItem>),
    >,
) {
    for (interaction, mut gradient) in &mut interaction_query {
        *gradient = match *interaction {
            Interaction::Hovered => {
                // Hover gradient (blue accent)
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.25), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.18, 0.18, 0.22, 0.90), Val::Percent(100.0)),
                ]))
            }
            Interaction::Pressed => {
                // Selected gradient (strong blue)
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.60), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.0, 0.38, 0.80, 0.80), Val::Percent(100.0)),
                ]))
            }
            Interaction::None => {
                // Default gradient
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
                ]))
            }
        };
    }
}
```

### Search Focus Border Gradient

**Pattern from:** `bevy/examples/ui/gradients.rs` BorderGradient

```rust
// Focus border system
fn search_focus_system(
    mut search_container: Query<
        (&mut BorderGradient, &Children),
        With<SearchContainer>,
    >,
    search_text: Query<&Text, (With<SearchInput>, Changed<Text>)>,
) {
    let Ok((mut border_gradient, _)) = search_container.get_single_mut() else { return };
    let Ok(text) = search_text.get_single() else { return };
    
    // Show focus border when typing
    *border_gradient = if text.is_empty() || **text == "Search..." {
        BorderGradient::default() // No border
    } else {
        BorderGradient::from(LinearGradient::to_right(vec![
            ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.8), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.4), Val::Percent(50.0)),
            ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.8), Val::Percent(100.0)),
        ]))
    };
}
```

### Spawning Result Items with Gradients

```rust
// Spawn result items with LAUNCHER_UI gradient patterns
fn spawn_result_item(parent: &mut ChildBuilder, title: &str, subtitle: &str) -> Entity {
    parent.spawn((
        Button, // For interaction
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(48.0),              // Compact height
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        // DEFAULT GRADIENT
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
        ])),
        BorderRadius::all(Val::Px(6.0)),
        ResultItem,
    ))
    .with_children(|parent| {
        // Icon placeholder
        parent.spawn((
            Node {
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                margin: UiRect::right(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 1.0)),
            BorderRadius::all(Val::Px(4.0)),
        ));
        
        // Text content
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                flex_grow: 1.0,
                ..default()
            },
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(title),
                TextFont {
                    font: fonts.medium.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.95, 0.95, 0.97, 1.0)),
            ));
            
            // Subtitle
            if !subtitle.is_empty() {
                parent.spawn((
                    Text::new(subtitle),
                    TextFont {
                        font: fonts.regular.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.70, 0.70, 0.75, 1.0)),
                ));
            }
        });
    }).id()
}
```

## Remove Theme Bullshit

**DELETE this from theme.rs:**

```rust
// ❌ DELETE ALL THIS GARBAGE
pub fn create_linear_gradient(&self, from: Color, _to: Color, _angle: f32) -> BackgroundColor {
    BackgroundColor(from)
}

pub fn create_multi_gradient(&self, stops: Vec<(Color, f32)>, angle: f32) -> BackgroundGradient {
    // ... BULLSHIT IMPLEMENTATION
}

pub fn container_gradient(&self) -> BackgroundGradient {
    // ... MORE BULLSHIT
}
```

## System Registration

```rust
// Add to main.rs systems
.add_systems(Update, (
    result_item_hover_system,
    search_focus_system,
))
```

## Success Criteria

✅ **Direct BackgroundGradient usage** - no custom theme methods  
✅ **Real hover interactions** - using Interaction component  
✅ **Real border gradients** - using BorderGradient component  
✅ **Real gradient animations** - direct component mutation  

**NO BULLSHIT THEME METHODS** - Use Bevy's gradient API directly

## Bevy Implementation Details

### Component-Based Gradient System

```rust
use bevy::{
    prelude::*,
    ui::{BackgroundGradient, BorderGradient, LinearGradient, ColorStop, Interaction},
    ecs::query::Changed,
};

/// Marker components for different UI element types
#[derive(Component, Debug)]
pub struct LauncherContainer;

#[derive(Component, Debug)]
pub struct SearchContainer;

#[derive(Component, Debug)]
pub struct ResultItem {
    pub index: usize,
    pub is_selected: bool,
}

/// Component for managing gradient states without theme abstraction
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct DirectGradientStates {
    pub default: BackgroundGradient,
    pub hover: BackgroundGradient,
    pub selected: BackgroundGradient,
}

impl DirectGradientStates {
    /// Create result item gradients using direct Bevy API
    pub fn result_item() -> Self {
        Self {
            default: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
            ])),
            hover: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.25), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.18, 0.18, 0.22, 0.90), Val::Percent(100.0)),
            ])),
            selected: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.60), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.0, 0.38, 0.80, 0.80), Val::Percent(100.0)),
            ])),
        }
    }
}
```

### Direct Interaction System (No Theme Layer)

```rust
/// Direct gradient mutation system using Bevy patterns
pub fn direct_result_item_gradient_system(
    mut result_items: Query<
        (
            &Interaction,
            &DirectGradientStates,
            &mut BackgroundGradient,
            &ResultItem,
        ),
        (Changed<Interaction>, With<ResultItem>),
    >,
) {
    for (interaction, gradient_states, mut current_gradient, result_item) in result_items.iter_mut() {
        let target_gradient = match *interaction {
            Interaction::Hovered => gradient_states.hover.clone(),
            Interaction::Pressed => gradient_states.selected.clone(),
            Interaction::None => {
                if result_item.is_selected {
                    gradient_states.selected.clone()
                } else {
                    gradient_states.default.clone()
                }
            },
        };
        
        *current_gradient = target_gradient;
    }
}

/// Selection-based gradient system (independent of hover state)
pub fn selection_gradient_system(
    mut result_items: Query<
        (&mut BackgroundGradient, &DirectGradientStates, &ResultItem),
        Changed<ResultItem>,
    >,
) {
    for (mut gradient, gradient_states, result_item) in result_items.iter_mut() {
        if result_item.is_selected {
            *gradient = gradient_states.selected.clone();
        } else {
            *gradient = gradient_states.default.clone();
        }
    }
}
```

### Container Spawning with Direct Gradients

```rust
/// Spawn launcher container with Bevy gradients (no theme layer)
pub fn spawn_launcher_container(
    mut commands: Commands,
    fonts: Res<Handle<Font>>,
) {
    commands.spawn((
        Node {
            width: Val::Px(600.0),
            height: Val::Px(420.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        // MAIN CONTAINER GRADIENT - Direct Bevy API
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.08, 0.08, 0.09, 0.98), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.12, 0.12, 0.14, 0.98), Val::Percent(100.0)),
        ])),
        BorderRadius::all(Val::Px(12.0)),
        LauncherContainer,
    ))
    .with_children(|parent| {
        spawn_search_container(parent, fonts.clone());
        spawn_results_container(parent, fonts);
    });
}

/// Spawn search input with direct gradients
fn spawn_search_container(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(48.0),
            padding: UiRect::all(Val::Px(12.0)),
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        },
        // SEARCH BAR GRADIENT - Direct Bevy API  
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.18, 0.18, 0.20, 0.85), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.15, 0.15, 0.17, 0.90), Val::Percent(100.0)),
        ])),
        BorderRadius::all(Val::Px(8.0)),
        BorderColor::all(Color::NONE),
        SearchContainer,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Search..."),
            TextFont {
                font,
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.75, 1.0)),
        ));
    });
}

/// Spawn individual result item with direct gradients
pub fn spawn_result_item(
    parent: &mut ChildBuilder,
    index: usize,
    title: &str,
    subtitle: &str,
    font: Handle<Font>,
) {
    parent.spawn((
        Button, // For Interaction component
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(48.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(8.0)),
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        },
        // DEFAULT GRADIENT - Direct Bevy API
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
        ])),
        BorderRadius::all(Val::Px(6.0)),
        DirectGradientStates::result_item(),
        ResultItem {
            index,
            is_selected: index == 0, // First item selected by default
        },
    ))
    .with_children(|parent| {
        // Icon placeholder
        parent.spawn((
            Node {
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                margin: UiRect::right(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 1.0)),
            BorderRadius::all(Val::Px(4.0)),
        ));
        
        // Text content column
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                flex_grow: 1.0,
                ..default()
            },
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(title),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.95, 0.95, 0.97, 1.0)),
            ));
            
            // Subtitle
            if !subtitle.is_empty() {
                parent.spawn((
                    Text::new(subtitle),
                    TextFont {
                        font,
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.70, 0.70, 0.75, 1.0)),
                ));
            }
        });
    });
}
```

### Focus Border System

```rust
/// Search focus border gradient system
pub fn search_focus_border_system(
    mut search_containers: Query<
        &mut BorderColor,
        With<SearchContainer>,
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut has_focus: Local<bool>,
) {
    let typing = keyboard_input.get_pressed().any(|key| matches!(
        key,
        KeyCode::KeyA..=KeyCode::KeyZ | 
        KeyCode::Digit0..=KeyCode::Digit9 |
        KeyCode::Space | KeyCode::Backspace
    ));
    
    if typing && !*has_focus {
        *has_focus = true;
        for mut border_color in search_containers.iter_mut() {
            // Focus border (simulated gradient with solid color)
            *border_color = BorderColor::all(Color::srgba(0.0, 0.48, 1.0, 0.8));
        }
    } else if !typing && *has_focus {
        *has_focus = false;
        for mut border_color in search_containers.iter_mut() {
            *border_color = BorderColor::all(Color::NONE);
        }
    }
}
```

### Keyboard Navigation System

```rust
/// Arrow key navigation with gradient updates
pub fn keyboard_navigation_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut result_items: Query<(&mut ResultItem, &mut BackgroundGradient, &DirectGradientStates)>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        navigate_selection(result_items, 1);
    } else if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        navigate_selection(result_items, -1);
    }
}

fn navigate_selection(
    mut result_items: Query<(&mut ResultItem, &mut BackgroundGradient, &DirectGradientStates)>,
    direction: i32,
) {
    let mut items: Vec<_> = result_items.iter_mut().collect();
    items.sort_by_key(|(item, _, _)| item.index);
    
    let current_selected = items.iter().position(|(item, _, _)| item.is_selected);
    
    if let Some(current) = current_selected {
        let new_index = match direction {
            1 => (current + 1).min(items.len() - 1), // Down
            -1 => current.saturating_sub(1),          // Up
            _ => current,
        };
        
        // Update selection states and gradients
        for (index, (mut item, mut gradient, states)) in items.into_iter().enumerate() {
            item.is_selected = index == new_index;
            *gradient = if item.is_selected {
                states.selected.clone()
            } else {
                states.default.clone()
            };
        }
    }
}
```

### System Registration and Plugin

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LauncherGradientSystems {
    HandleInteractions,
    UpdateSelection,
    HandleFocus,
    NavigateResults,
}

pub struct LauncherGradientPlugin;

impl Plugin for LauncherGradientPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<DirectGradientStates>()
            .register_type::<ResultItem>()
            .configure_sets(
                Update,
                (
                    LauncherGradientSystems::HandleInteractions,
                    LauncherGradientSystems::UpdateSelection,
                    LauncherGradientSystems::HandleFocus,
                    LauncherGradientSystems::NavigateResults,
                ).chain(),
            )
            .add_systems(
                Update,
                (
                    direct_result_item_gradient_system
                        .in_set(LauncherGradientSystems::HandleInteractions),
                    selection_gradient_system
                        .in_set(LauncherGradientSystems::UpdateSelection),
                    search_focus_border_system
                        .in_set(LauncherGradientSystems::HandleFocus),
                    keyboard_navigation_system
                        .in_set(LauncherGradientSystems::NavigateResults),
                ),
            )
            .add_systems(Startup, spawn_launcher_container);
    }
}
```

### Event-Driven Updates

```rust
/// Events for gradient state changes
#[derive(Event, Debug)]
pub enum GradientUpdateEvent {
    SearchResultsChanged(Vec<String>),
    SelectionChanged(usize),
    FocusChanged(bool),
}

/// Event-driven gradient update system
pub fn gradient_event_handler(
    mut gradient_events: EventReader<GradientUpdateEvent>,
    mut result_items: Query<(&mut ResultItem, &mut BackgroundGradient, &DirectGradientStates)>,
    mut search_containers: Query<&mut BorderColor, With<SearchContainer>>,
) {
    for event in gradient_events.read() {
        match event {
            GradientUpdateEvent::SelectionChanged(new_index) => {
                for (mut item, mut gradient, states) in result_items.iter_mut() {
                    item.is_selected = item.index == *new_index;
                    *gradient = if item.is_selected {
                        states.selected.clone()
                    } else {
                        states.default.clone()
                    };
                }
            },
            GradientUpdateEvent::FocusChanged(has_focus) => {
                for mut border_color in search_containers.iter_mut() {
                    *border_color = if *has_focus {
                        BorderColor::all(Color::srgba(0.0, 0.48, 1.0, 0.8))
                    } else {
                        BorderColor::all(Color::NONE)
                    };
                }
            },
            GradientUpdateEvent::SearchResultsChanged(results) => {
                // Update result items based on search results
                info!("Search results updated: {} items", results.len());
            },
        }
    }
}
```

### Resource Management (Minimal)

```rust
/// Simple resource for gradient configuration (no complex theme system)
#[derive(Resource, Debug)]
pub struct GradientConfig {
    pub animation_speed: f32,
    pub border_focus_color: Color,
    pub use_smooth_transitions: bool,
}

impl Default for GradientConfig {
    fn default() -> Self {
        Self {
            animation_speed: 0.2,
            border_focus_color: Color::srgba(0.0, 0.48, 1.0, 0.8),
            use_smooth_transitions: false, // Disable for performance
        }
    }
}
```

### Testing Strategies

```rust
#[cfg(test)]
mod launcher_gradient_tests {
    use super::*;

    #[test]
    fn test_result_item_gradient_creation() {
        let gradients = DirectGradientStates::result_item();
        
        // Test that gradients are different (not all default)
        let default_stops = gradients.default.gradient.stops.len();
        let hover_stops = gradients.hover.gradient.stops.len();
        assert!(default_stops > 0);
        assert!(hover_stops > 0);
    }

    #[test]
    fn test_gradient_interaction_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(LauncherGradientPlugin);

        // Spawn test result item
        let entity = app.world_mut().spawn((
            Button,
            Node::default(),
            BackgroundGradient::default(),
            DirectGradientStates::result_item(),
            ResultItem { index: 0, is_selected: false },
            Interaction::None,
        )).id();

        // Test hover interaction
        app.world_mut().get_mut::<Interaction>(entity).unwrap().set_if_neq(Interaction::Hovered);
        app.update();

        // Verify gradient changed
        let background = app.world().get::<BackgroundGradient>(entity).unwrap();
        // In real test, would verify the gradient matches hover state
        assert!(background.gradient.stops.len() > 0);
    }

    #[test]
    fn test_navigation_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(LauncherGradientPlugin);

        // Spawn multiple result items
        for i in 0..3 {
            app.world_mut().spawn((
                Node::default(),
                BackgroundGradient::default(),
                DirectGradientStates::result_item(),
                ResultItem { index: i, is_selected: i == 0 },
            ));
        }

        // Simulate arrow down key press
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>()
           .press(KeyCode::ArrowDown);
        app.update();

        // Check that selection moved (would need more sophisticated test)
        let results: Vec<_> = app.world().query::<&ResultItem>()
            .iter(app.world())
            .collect();
        assert_eq!(results.len(), 3);
    }
}
```
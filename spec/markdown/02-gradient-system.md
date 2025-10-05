# Gradient Implementation System

## Critical Analysis: Current Gradient Disability

### Current Issue (ui/src/ui/theme.rs:414-417)
```rust
/// Create background color (simplified from gradient for compatibility)
pub fn create_linear_gradient(&self, from: Color, _to: Color, _angle: f32) -> BackgroundColor {
    // Simplified to solid color - will add gradients back when API is stable
    BackgroundColor(from)
}
```

**Root Cause:** The comment is outdated - Bevy's gradient API is fully stable and functional. The current implementation ignores gradient parameters and returns flat colors, making the UI look flat and unprofessional compared to Raycast's beautiful gradients.

## Target Architecture: Raycast-like Gradient System

### Design Principles
1. **Sophisticated Gradients**: Multi-stop linear gradients for depth
2. **Interactive States**: Gradients change on hover/focus for feedback
3. **Performance**: Zero-allocation gradient updates using component systems
4. **Consistency**: Unified gradient language across all UI elements

### Gradient Analysis: Raycast Visual Patterns

#### Container Background Gradient
```rust
// Raycast container: Dark to slightly lighter, subtle depth
LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.08, 0.08, 0.09, 0.98), Val::Percent(0.0)),
    ColorStop::new(Color::srgba(0.12, 0.12, 0.14, 0.98), Val::Percent(100.0)),
])
```

#### Search Bar Gradient
```rust
// Raycast search bar: Elevated surface with subtle gradient
LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.18, 0.18, 0.20, 0.85), Val::Percent(0.0)),
    ColorStop::new(Color::srgba(0.15, 0.15, 0.17, 0.90), Val::Percent(100.0)),
])
```

#### Result Item Gradients
```rust
// Default state: Subtle depth
LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)), 
    ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
])

// Hover state: Brighter with blue accent
LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.25), Val::Percent(0.0)),
    ColorStop::new(Color::srgba(0.18, 0.18, 0.22, 0.90), Val::Percent(100.0)),
])

// Selected state: Strong blue gradient
LinearGradient::to_bottom(vec![
    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.60), Val::Percent(0.0)),
    ColorStop::new(Color::srgba(0.0, 0.38, 0.80, 0.80), Val::Percent(100.0)),
])
```

## Implementation Specification

### Phase 1: Theme System Replacement

**File:** `ui/src/ui/theme.rs`
**Lines:** 414-417 (create_linear_gradient method)

**Current Implementation:**
```rust
pub fn create_linear_gradient(&self, from: Color, _to: Color, _angle: f32) -> BackgroundColor {
    BackgroundColor(from)
}
```

**Target Implementation:**
```rust
/// Create professional linear gradient using Bevy's full gradient API
#[inline]
pub fn create_linear_gradient(
    &self, 
    from: Color, 
    to: Color, 
    angle: f32
) -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient {
        angle,
        stops: vec![
            ColorStop::new(from, Val::Percent(0.0)),
            ColorStop::new(to, Val::Percent(100.0)),
        ],
    })
}

/// Create multi-stop gradient for complex visual effects
#[inline] 
pub fn create_multi_gradient(
    &self,
    stops: Vec<(Color, f32)>, // (color, percent_position)
    angle: f32,
) -> BackgroundGradient {
    let color_stops: Vec<ColorStop> = stops
        .into_iter()
        .map(|(color, percent)| ColorStop::new(color, Val::Percent(percent)))
        .collect();
        
    BackgroundGradient::from(LinearGradient {
        angle,
        stops: color_stops,
    })
}
```

### Phase 2: Gradient Palette Expansion

**New Section in theme.rs:** Gradient definitions
```rust
impl ColorPalette {
    /// Professional gradient definitions for Raycast-like UI
    
    /// Container background gradient (dark to slightly lighter)
    #[inline]
    pub fn container_gradient(&self) -> BackgroundGradient {
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.08, 0.08, 0.09, 0.98), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.12, 0.12, 0.14, 0.98), Val::Percent(100.0)),
        ]))
    }
    
    /// Search input gradient (elevated surface feeling)
    #[inline]  
    pub fn search_input_gradient(&self) -> BackgroundGradient {
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.18, 0.18, 0.20, 0.85), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.15, 0.15, 0.17, 0.90), Val::Percent(100.0)),
        ]))
    }
    
    /// Result item default state gradient
    #[inline]
    pub fn result_item_gradient(&self) -> BackgroundGradient {
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
        ]))
    }
    
    /// Result item hover gradient (blue accent)
    #[inline]
    pub fn result_item_hover_gradient(&self) -> BackgroundGradient {
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.25), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.18, 0.18, 0.22, 0.90), Val::Percent(100.0)),
        ]))
    }
    
    /// Result item selected gradient (strong blue)
    #[inline]
    pub fn result_item_selected_gradient(&self) -> BackgroundGradient {
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.60), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.0, 0.38, 0.80, 0.80), Val::Percent(100.0)),
        ]))
    }
}
```

### Phase 3: Interactive Gradient Components

**New Component:** `InteractiveGradient`
```rust
#[derive(Component, Debug, Clone)]
pub struct InteractiveGradient {
    pub default_gradient: BackgroundGradient,
    pub hover_gradient: BackgroundGradient,
    pub selected_gradient: Option<BackgroundGradient>,
    pub transition_speed: f32,        // Animation speed (0.1-1.0)
}

impl InteractiveGradient {
    /// Create interactive gradient for result items
    #[inline]
    pub fn result_item(theme: &ColorPalette) -> Self {
        Self {
            default_gradient: theme.result_item_gradient(),
            hover_gradient: theme.result_item_hover_gradient(),
            selected_gradient: Some(theme.result_item_selected_gradient()),
            transition_speed: 0.2, // Fast but smooth transitions
        }
    }
    
    /// Create interactive gradient for search input
    #[inline]
    pub fn search_input(theme: &ColorPalette) -> Self {
        Self {
            default_gradient: theme.search_input_gradient(),
            hover_gradient: theme.search_input_gradient(), // Same as default
            selected_gradient: None,
            transition_speed: 0.1,
        }
    }
}
```

### Phase 4: Gradient Update System

**New System:** `interactive_gradient_system`
```rust
/// Zero-allocation system for updating gradients based on interaction state
#[inline]
pub fn interactive_gradient_system(
    mut query: Query<
        (&Interaction, &InteractiveGradient, &mut BackgroundGradient),
        Changed<Interaction>,
    >,
    time: Res<Time>,
) {
    for (interaction, interactive, mut background) in query.iter_mut() {
        let target_gradient = match *interaction {
            Interaction::Pressed | Interaction::Hovered => {
                &interactive.hover_gradient
            },
            Interaction::None => &interactive.default_gradient,
        };
        
        // Instant update - no smooth transitions for now (optimization)
        // TODO: Add smooth gradient transitions in future iteration
        *background = target_gradient.clone();
    }
}
```

### Phase 5: Setup System Integration

**File:** `ui/src/ui/systems/setup.rs`
**Updates to gradient usage:**

**Main Container (Lines 91-93):**
```rust
// Replace: BackgroundColor(theme.colors.background_primary),
theme.colors.container_gradient(),
```

**Search Input Container (Lines 109-110):**
```rust  
// Replace: BackgroundColor(theme.colors.surface_default),
theme.colors.search_input_gradient(),
```

**Result Items (New spawn in setup or components system):**
```rust
.spawn((
    Node { /* result item node config */ },
    theme.colors.result_item_gradient(),
    InteractiveGradient::result_item(&theme.colors),
    // ... other components
))
```

### Phase 6: Border Gradients for Focus States

**Search Input Focus Border:**
```rust
/// Create border gradient for focused search input
#[inline]
pub fn search_focus_border_gradient(&self) -> BorderGradient {
    BorderGradient::from(LinearGradient::to_right(vec![
        ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.8), Val::Percent(0.0)),
        ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.4), Val::Percent(50.0)),  
        ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.8), Val::Percent(100.0)),
    ]))
}
```

**Focus State System:**
```rust
/// System to handle search input focus border gradients
#[inline]
pub fn search_focus_gradient_system(
    mut query: Query<
        (&mut BorderGradient, &SearchInput),
        Changed<SearchInput>
    >,
    theme: Res<Theme>,
    app_state: Res<State<AppState>>,
) {
    if *app_state.get() == AppState::SearchMode {
        for (mut border_gradient, _) in query.iter_mut() {
            *border_gradient = theme.colors.search_focus_border_gradient();
        }
    }
}
```

## Implementation Timeline

### Phase 1: Core Gradient API (High Priority)
- Replace theme.rs create_linear_gradient method
- Add BackgroundGradient support
- Test basic gradient rendering

### Phase 2: Gradient Palette (High Priority)
- Add container_gradient() method
- Add search_input_gradient() method  
- Add result item gradients

### Phase 3: Setup Integration (Medium Priority)
- Update setup.rs to use gradients
- Replace BackgroundColor with BackgroundGradient
- Visual testing against Raycast

### Phase 4: Interactive States (Medium Priority) 
- Add InteractiveGradient component
- Implement interactive_gradient_system
- Test hover and selection states

### Phase 5: Border Gradients (Low Priority)
- Add focus border gradients
- Implement search_focus_gradient_system
- Polish focus state visuals

## Performance Requirements

### Zero Allocation Constraints
- Gradient components reused, not recreated
- Use `#[inline]` on all gradient methods
- Avoid Vec allocations in hot paths
- Pre-compute gradient definitions

### Benchmarking Targets  
- Gradient updates: < 0.05ms per interaction
- Setup gradient creation: < 1ms total
- Memory usage: < 100KB for all gradients

## Testing Strategy

### Visual Tests
- Side-by-side comparison with Raycast
- Gradient smoothness verification
- Color accuracy testing

### Performance Tests
- Gradient update microbenchmarks
- Memory allocation tracking
- Frame rate impact measurement

### Integration Tests
- Hover state transitions
- Focus state borders  
- Container gradient rendering

## Dependencies

### Required Imports
```rust
use bevy::ui::{BackgroundGradient, BorderGradient}; 
use crate::ui::gradients::{LinearGradient, ColorStop};
```

### Internal Dependencies
- theme.rs - Color palette system
- components.rs - UI component definitions
- setup.rs - UI initialization system

## Success Criteria

1. ✅ All flat colors replaced with professional gradients
2. ✅ Visual match with Raycast gradient patterns  
3. ✅ Smooth hover and focus state transitions
4. ✅ Zero allocations in gradient updates
5. ✅ Performance targets met
6. ✅ Border gradients working for focus states

---

## Bevy Implementation Details

### Component Architecture

```rust
use bevy::{
    prelude::*,
    ui::{BackgroundGradient, BorderGradient, LinearGradient, ColorStop},
    ecs::system::SystemParam,
};

/// Core gradient component for interactive UI elements
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct InteractiveGradient {
    pub default_state: GradientState,
    pub hover_state: GradientState,
    pub selected_state: Option<GradientState>,
    pub transition_speed: f32,
    pub current_state: InteractionState,
}

#[derive(Reflect, Debug, Clone)]
pub struct GradientState {
    pub background: BackgroundGradient,
    pub border: Option<BorderGradient>,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub enum InteractionState {
    Default,
    Hovered, 
    Selected,
    Pressed,
}

impl InteractiveGradient {
    /// Create gradient configuration for result items
    pub fn result_item() -> Self {
        Self {
            default_state: GradientState {
                background: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
                ])),
                border: None,
            },
            hover_state: GradientState {
                background: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.25), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.18, 0.18, 0.22, 0.90), Val::Percent(100.0)),
                ])),
                border: Some(BorderGradient::from(LinearGradient::to_right(vec![
                    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.4), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.8), Val::Percent(50.0)),
                    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.4), Val::Percent(100.0)),
                ]))),
            },
            selected_state: Some(GradientState {
                background: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.60), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.0, 0.38, 0.80, 0.80), Val::Percent(100.0)),
                ])),
                border: Some(BorderGradient::from(LinearGradient::to_right(vec![
                    ColorStop::new(Color::srgba(0.0, 0.58, 1.0, 0.9), Val::Percent(0.0)),
                    ColorStop::new(Color::srgba(0.0, 0.58, 1.0, 1.0), Val::Percent(50.0)),
                    ColorStop::new(Color::srgba(0.0, 0.58, 1.0, 0.9), Val::Percent(100.0)),
                ]))),
            }),
            transition_speed: 0.2,
            current_state: InteractionState::Default,
        }
    }
}
```

### System Implementation

```rust
/// High-performance gradient update system with Changed<T> filtering
pub fn interactive_gradient_system(
    mut gradient_query: Query<
        (
            &Interaction,
            &mut InteractiveGradient,
            &mut BackgroundGradient,
            Option<&mut BorderColor>, 
        ),
        (Changed<Interaction>, With<InteractiveGradient>),
    >,
) {
    for (interaction, mut interactive, mut bg_gradient, border_color) in gradient_query.iter_mut() {
        let target_state = match *interaction {
            Interaction::Pressed => InteractionState::Pressed,
            Interaction::Hovered => InteractionState::Hovered,
            Interaction::None => InteractionState::Default,
        };

        // Only update if state changed (avoid unnecessary work)
        if interactive.current_state != target_state {
            interactive.current_state = target_state;
            
            let gradient_state = match target_state {
                InteractionState::Hovered => &interactive.hover_state,
                InteractionState::Pressed | InteractionState::Selected => {
                    interactive.selected_state.as_ref().unwrap_or(&interactive.hover_state)
                },
                InteractionState::Default => &interactive.default_state,
            };
            
            // Update background gradient
            *bg_gradient = gradient_state.background.clone();
            
            // Update border if present
            if let (Some(mut border), Some(border_gradient)) = (border_color, &gradient_state.border) {
                // Bevy doesn't have BorderGradient yet, so use solid color from gradient
                if let Some(first_stop) = border_gradient.stops.first() {
                    *border = BorderColor::all(first_stop.color);
                }
            }
        }
    }
}
```

### System Sets and Ordering

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GradientSystems {
    /// Update gradients based on interaction state
    UpdateInteractive,
    /// Apply gradient animations
    AnimateTransitions,
    /// Clean up gradient resources
    Cleanup,
}

/// Plugin for gradient system integration
pub struct GradientPlugin;

impl Plugin for GradientPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<InteractiveGradient>()
            .register_type::<GradientState>()
            .register_type::<InteractionState>()
            .configure_sets(
                Update,
                (
                    GradientSystems::UpdateInteractive,
                    GradientSystems::AnimateTransitions,
                    GradientSystems::Cleanup,
                ).chain()
            )
            .add_systems(
                Update,
                (
                    interactive_gradient_system.in_set(GradientSystems::UpdateInteractive),
                    gradient_animation_system.in_set(GradientSystems::AnimateTransitions),
                ).chain()
            );
    }
}

/// Smooth gradient transition system using lerp
pub fn gradient_animation_system(
    mut gradient_query: Query<
        (&mut InteractiveGradient, &mut BackgroundGradient),
        With<InteractiveGradient>
    >,
    time: Res<Time>,
) {
    for (mut interactive, mut bg_gradient) in gradient_query.iter_mut() {
        // Implement smooth color transitions if needed
        // For now, instant updates are sufficient for performance
    }
}
```

### Event-Driven Patterns

```rust
/// Custom gradient events for complex interactions
#[derive(Event, Debug)]
pub enum GradientEvent {
    /// Force gradient update for specific entity
    ForceUpdate(Entity, InteractionState),
    /// Reset all gradients to default state
    ResetAll,
    /// Update gradient based on search results
    SearchStateChanged { has_results: bool },
}

/// System to handle gradient events
pub fn gradient_event_system(
    mut gradient_events: EventReader<GradientEvent>,
    mut gradient_query: Query<(&mut InteractiveGradient, &mut BackgroundGradient)>,
    mut commands: Commands,
) {
    for event in gradient_events.read() {
        match event {
            GradientEvent::ForceUpdate(entity, state) => {
                if let Ok((mut interactive, mut bg_gradient)) = gradient_query.get_mut(*entity) {
                    interactive.current_state = *state;
                    
                    let target_state = match state {
                        InteractionState::Hovered => &interactive.hover_state,
                        InteractionState::Selected | InteractionState::Pressed => {
                            interactive.selected_state.as_ref().unwrap_or(&interactive.hover_state)
                        },
                        InteractionState::Default => &interactive.default_state,
                    };
                    
                    *bg_gradient = target_state.background.clone();
                }
            },
            GradientEvent::ResetAll => {
                for (mut interactive, mut bg_gradient) in gradient_query.iter_mut() {
                    interactive.current_state = InteractionState::Default;
                    *bg_gradient = interactive.default_state.background.clone();
                }
            },
            GradientEvent::SearchStateChanged { has_results } => {
                // Custom logic for search state changes
                info!("Search state changed: has_results = {}", has_results);
            },
        }
    }
}
```

### Resource Management

```rust
/// Resource for managing gradient presets
#[derive(Resource, Debug)]
pub struct GradientPresets {
    pub container: BackgroundGradient,
    pub search_input: BackgroundGradient,
    pub result_item_default: BackgroundGradient,
    pub result_item_hover: BackgroundGradient,
    pub result_item_selected: BackgroundGradient,
}

impl Default for GradientPresets {
    fn default() -> Self {
        Self {
            container: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(Color::srgba(0.08, 0.08, 0.09, 0.98), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.12, 0.12, 0.14, 0.98), Val::Percent(100.0)),
            ])),
            search_input: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(Color::srgba(0.18, 0.18, 0.20, 0.85), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.15, 0.15, 0.17, 0.90), Val::Percent(100.0)),
            ])),
            result_item_default: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
            ])),
            result_item_hover: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.25), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.18, 0.18, 0.22, 0.90), Val::Percent(100.0)),
            ])),
            result_item_selected: BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(Color::srgba(0.0, 0.48, 1.0, 0.60), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.0, 0.38, 0.80, 0.80), Val::Percent(100.0)),
            ])),
        }
    }
}
```

### Async Task Handling

```rust
/// Task for loading gradient resources asynchronously
#[derive(Component)]
pub struct GradientResourceLoader(Task<GradientPresets>);

/// System to spawn gradient resource loading task
pub fn spawn_gradient_loading_task(
    mut commands: Commands,
) {
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async {
        // Load gradient configurations from file or generate defaults
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        GradientPresets::default()
    });
    
    commands.spawn(GradientResourceLoader(task));
}

/// System to handle gradient loading completion
pub fn handle_gradient_loading(
    mut commands: Commands,
    mut gradient_loaders: Query<(Entity, &mut GradientResourceLoader)>,
) {
    for (entity, mut loader) in gradient_loaders.iter_mut() {
        if let Some(gradient_presets) = block_on(future::poll_once(&mut loader.0)) {
            commands.insert_resource(gradient_presets);
            commands.entity(entity).despawn();
        }
    }
}
```

### Testing Strategies

```rust
#[cfg(test)]
mod gradient_tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn test_interactive_gradient_creation() {
        let gradient = InteractiveGradient::result_item();
        assert_eq!(gradient.current_state, InteractionState::Default);
        assert!(gradient.selected_state.is_some());
    }

    #[test]
    fn test_gradient_system_performance() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(GradientPlugin);

        // Spawn test entities
        let entity = app.world_mut().spawn((
            Button,
            InteractiveGradient::result_item(),
            BackgroundGradient::default(),
            Interaction::default(),
        )).id();

        // Test system execution
        app.update();
        
        // Verify gradient state
        let interactive = app.world().get::<InteractiveGradient>(entity).unwrap();
        assert_eq!(interactive.current_state, InteractionState::Default);
    }

    #[test]
    fn test_gradient_event_handling() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(GradientPlugin)
           .add_event::<GradientEvent>();

        let entity = app.world_mut().spawn((
            InteractiveGradient::result_item(),
            BackgroundGradient::default(),
        )).id();

        // Send gradient event
        app.world_mut().send_event(GradientEvent::ForceUpdate(entity, InteractionState::Hovered));
        app.update();

        // Verify state change
        let interactive = app.world().get::<InteractiveGradient>(entity).unwrap();
        assert_eq!(interactive.current_state, InteractionState::Hovered);
    }
}
```

---

**Next:** [03-compact-search-design.md](./03-compact-search-design.md)
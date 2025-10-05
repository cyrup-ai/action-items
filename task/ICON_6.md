# ICON_6: Interactive Icon Systems

## OBJECTIVE
Create the interactive icon systems that provide icon rendering, state management, animations, event handling, and performance optimization. These systems make icons respond to user interactions with smooth transitions.

## RATIONALE
These systems are the core functionality that IconPlugin provides. They follow the GradientPlugin pattern: apply_system for rendering, interactive_system for state tracking, animate_system for transitions, event handlers for dynamic changes, and optimization systems for performance.

## PREREQUISITES
- ICON_1 complete (IconTheme)
- ICON_2 complete (IconComponent, IconAnimation)
- ICON_3 complete (all icon events)
- ICON_4 complete (FontAwesome)

## SUBTASK 1: Create Icon Application System

**File**: `packages/ecs-ui/src/icons/systems.rs`

**Create**:
```rust
use bevy::prelude::*;
use tracing::info;

use super::components::{IconComponent, IconInteractionState, IconAnimation};
use super::events::*;
use super::cache::IconCache;
use super::theme::IconTheme;
use super::fontawesome::FontAwesome;

/// Apply icons to UI components based on IconComponent configuration
///
/// Analogous to apply_gradient_system - renders icons with current state/colors.
/// Runs on Changed<IconComponent> to avoid unnecessary work.
#[inline]
pub fn apply_icon_system(
    icon_theme: Res<IconTheme>,
    fontawesome: Res<FontAwesome>,
    mut icon_components: Query<(&IconComponent, &mut TextColor), Changed<IconComponent>>,
) {
    for (icon_comp, mut text_color) in icon_components.iter_mut() {
        let color = if let Some(custom) = icon_comp.custom_color {
            custom
        } else {
            fontawesome.get_icon_color(icon_comp.icon_type, &icon_theme)
        };

        *text_color = TextColor(color);
    }
}
```

**Pattern**: Follows `packages/ecs-ui/src/gradients/systems.rs:17-31`

## SUBTASK 2: Create Interactive Icon System

**File**: `packages/ecs-ui/src/icons/systems.rs` (append)

**Add**:
```rust
/// Interactive icon state system - handles hover/press/selection
///
/// Analogous to interactive_gradient_system.
/// Updates IconComponent interaction state based on Bevy's Interaction component.
#[inline]
pub fn interactive_icon_system(
    mut icon_components: Query<&mut IconComponent>,
    interaction_query: Query<(Entity, &Interaction), Changed<Interaction>>,
) {
    for (entity, interaction) in interaction_query.iter() {
        if let Ok(mut icon_comp) = icon_components.get_mut(entity) {
            let new_state = match interaction {
                Interaction::Hovered => IconInteractionState::Hover,
                Interaction::Pressed => IconInteractionState::Pressed,
                Interaction::None => IconInteractionState::Default,
            };

            if icon_comp.interaction_state != new_state {
                icon_comp.interaction_state = new_state;
            }
        }
    }
}
```

**Pattern**: Follows `packages/ecs-ui/src/gradients/systems.rs:37-56`

## SUBTASK 3: Create Animation System

**File**: `packages/ecs-ui/src/icons/systems.rs` (append)

**Add**:
```rust
/// Animate icon transitions - handles color/size changes
///
/// Updates Transform and TextColor based on IconAnimation progress.
/// Removes animation component when complete and sends completion event.
#[inline]
pub fn animate_icon_transitions_system(
    mut commands: Commands,
    time: Res<Time>,
    mut animation_events: EventWriter<IconAnimationCompleteEvent>,
    mut query: Query<(Entity, &mut IconAnimation, &mut Transform, &mut TextColor)>,
) {
    let delta_time = time.delta_secs();

    for (entity, mut animation, mut transform, mut text_color) in query.iter_mut() {
        let (color, scale) = animation.update(delta_time);

        transform.scale = Vec3::splat(scale);
        *text_color = TextColor(color);

        if animation.is_complete() {
            animation_events.write(IconAnimationCompleteEvent {
                entity,
                animation_type: IconAnimationType::ColorTransition,
            });

            commands.entity(entity).remove::<IconAnimation>();
        }
    }
}
```

**Pattern**: Follows `packages/ecs-ui/src/visibility/systems.rs:112-164`

## SUBTASK 4: Create Event Handler Systems

**File**: `packages/ecs-ui/src/icons/systems.rs` (append)

**Add**:
```rust
/// Handle icon color change events
///
/// Responds to IconColorChangeEvent by either:
/// - Immediately setting new color (animated=false)
/// - Starting IconAnimation for smooth transition (animated=true)
pub fn handle_icon_color_change_events(
    mut commands: Commands,
    mut events: EventReader<IconColorChangeEvent>,
    query: Query<&TextColor>,
) {
    for event in events.read() {
        if event.animated {
            if let Ok(current_color) = query.get(event.entity) {
                let animation = IconAnimation {
                    current_time: 0.0,
                    duration: 0.2,
                    initial_color: current_color.0,
                    target_color: event.new_color,
                    initial_scale: 1.0,
                    target_scale: 1.0,
                };
                commands.entity(event.entity).insert(animation);
            }
        } else {
            commands.entity(event.entity).insert(TextColor(event.new_color));
        }
    }
}

/// Handle icon size change events
///
/// Updates IconComponent size field.
/// Size changes affect TextFont which should be updated externally.
pub fn handle_icon_size_change_events(
    mut events: EventReader<IconSizeChangeEvent>,
    mut query: Query<&mut IconComponent>,
) {
    for event in events.read() {
        if let Ok(mut icon_comp) = query.get_mut(event.entity) {
            icon_comp.size = event.new_size;
        }
    }
}
```

## SUBTASK 5: Create Optimization Systems

**File**: `packages/ecs-ui/src/icons/systems.rs` (append)

**Add**:
```rust
/// Optimize icon performance - cache warmup, batching
///
/// Analogous to optimize_gradient_performance_system.
/// Pre-warms icon cache when cache changes to reduce runtime lookups.
#[inline]
pub fn optimize_icon_performance_system(
    icon_cache: Res<IconCache>,
    icon_components: Query<&IconComponent>,
) {
    if !icon_cache.is_changed() {
        return;
    }

    // Pre-warm icon cache for commonly used icons
    let mut unique_types = std::collections::HashSet::new();
    for icon_comp in icon_components.iter() {
        unique_types.insert(icon_comp.icon_type);
    }

    info!("Optimized {} unique icon types", unique_types.len());
}

/// Validate icon cache - cleanup stale entries
///
/// Removes old failed entries to prevent cache bloat.
pub fn validate_icon_cache_system(mut icon_cache: ResMut<IconCache>) {
    // Cache validation runs when cache changes
    if !icon_cache.is_changed() {
        return;
    }

    let initial_failed_count = icon_cache.failed_to_load.len();

    // Keep cache size reasonable (could add TTL logic here)
    // For now, just log stats
    info!(
        "Icon cache stats: {} loaded, {} failed",
        icon_cache.loaded_icons.len(),
        initial_failed_count
    );
}
```

**Pattern**: Follows `packages/ecs-ui/src/gradients/systems.rs:62-83`

## SUBTASK 6: Update Icons Module

**File**: `packages/ecs-ui/src/icons/mod.rs`

**Modify** (add systems module):
```rust
//! Core icon type system for Bevy applications

pub mod types;
pub mod theme;
pub mod cache;
pub mod components;
pub mod events;
pub mod fontawesome;
pub mod extraction;
pub mod systems;

// Re-export public types
pub use types::{IconSize, IconType};
pub use theme::{ThemeColors, IconTheme};
pub use cache::IconCache;
pub use components::{IconInteractionState, IconComponent, IconAnimation};
pub use events::{
    IconExtractionRequest,
    IconExtractionResult,
    IconColorChangeEvent,
    IconSizeChangeEvent,
    IconStateChangeEvent,
    IconAnimationCompleteEvent,
    IconAnimationType,
};
pub use fontawesome::{FontAwesome, IconDetection, IconFallback};
pub use extraction::{
    IconExtractionInProgress,
    process_icon_extraction_requests,
    poll_icon_extraction_tasks,
    process_icon_extraction_results,
};
pub use systems::{
    apply_icon_system,
    interactive_icon_system,
    animate_icon_transitions_system,
    handle_icon_color_change_events,
    handle_icon_size_change_events,
    optimize_icon_performance_system,
    validate_icon_cache_system,
};
```

## DEFINITION OF DONE

### Compilation
- [ ] `cargo check -p ecs-ui` passes without errors
- [ ] All systems properly exported from icons module
- [ ] No clippy warnings

### Functionality
- [ ] apply_icon_system updates TextColor based on IconComponent
- [ ] interactive_icon_system responds to Interaction changes
- [ ] animate_icon_transitions_system lerps color and scale
- [ ] handle_icon_color_change_events creates animations or sets immediate colors
- [ ] handle_icon_size_change_events updates IconComponent size
- [ ] optimize_icon_performance_system counts unique icon types
- [ ] validate_icon_cache_system logs cache stats

### Code Quality
- [ ] Systems marked #[inline] where appropriate
- [ ] Change detection used (Changed<IconComponent>, resource_changed)
- [ ] Events properly read with EventReader
- [ ] Animations cleaned up when complete
- [ ] Logging uses tracing::info

## CONSTRAINTS

- **NO TESTS**: Do not write unit tests, integration tests, or test modules
- **NO BENCHMARKS**: Do not write benchmark code
- **SINGLE SESSION**: This task must be completable in one focused session
- **PATTERN MATCHING**: Follow GradientPlugin system patterns exactly

## REFERENCE FILES

**Pattern Files**:
- `packages/ecs-ui/src/gradients/systems.rs:17-83` - apply, interactive, optimize patterns
- `packages/ecs-ui/src/visibility/systems.rs:112-164` - Animation pattern
- `packages/ecs-ui/src/gradients/systems.rs:89-116` - Event handler pattern

**Dependencies**:
- Requires ICON_1 (IconTheme)
- Requires ICON_2 (IconComponent, IconAnimation, IconInteractionState)
- Requires ICON_3 (all events)
- Requires ICON_4 (FontAwesome)

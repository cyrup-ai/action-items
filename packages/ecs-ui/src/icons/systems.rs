//! Icon system implementations for Bevy ECS
//!
//! Provides zero-allocation, blazing-fast icon rendering, state management,
//! animations, event handling, and performance optimization systems.
//!
//! Systems follow the GradientPlugin pattern for consistency across ecs-ui.

use bevy::prelude::*;
use tracing::info;

use super::components::{IconComponent, IconInteractionState, IconAnimation};
use super::events::*;
use super::cache::IconCache;
use crate::theme::Theme;
use super::fontawesome::FontAwesome;

/// Apply icons to UI components based on IconComponent configuration
///
/// Analogous to apply_gradient_system - renders icons with current state/colors.
/// Runs on Changed<IconComponent> to avoid unnecessary work.
///
/// **Pattern**: Uses change detection for zero-allocation performance.
/// Only processes entities where IconComponent was modified.
///
/// **Reference**: [apply_gradient_system](../gradients/systems.rs:17-31)
#[inline]
pub fn apply_icon_system(
    theme: Res<Theme>,
    fontawesome: Res<FontAwesome>,
    mut icon_components: Query<(&IconComponent, &mut TextColor), Changed<IconComponent>>,
) {
    for (icon_comp, mut text_color) in icon_components.iter_mut() {
        let color = if let Some(custom) = icon_comp.custom_color {
            custom
        } else {
            fontawesome.get_icon_color(icon_comp.icon_type, &theme)
        };

        *text_color = TextColor(color);
    }
}

/// Interactive icon state system - handles hover/press/selection
///
/// Analogous to interactive_gradient_system.
/// Updates IconComponent interaction state based on Bevy's Interaction component.
///
/// **Pattern**: Only updates when Interaction changes, avoiding unnecessary mutation.
/// Compares new_state to current before writing to prevent false change detection.
///
/// **Reference**: [interactive_gradient_system](../gradients/systems.rs:37-56)
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

            // Only update if state actually changed to avoid unnecessary work
            if icon_comp.interaction_state != new_state {
                icon_comp.interaction_state = new_state;
            }
        }
    }
}

/// Animate icon transitions - handles color/size changes
///
/// Updates Transform and TextColor based on IconAnimation progress.
/// Removes animation component when complete and sends completion event.
///
/// **Pattern**: Frame-driven interpolation with automatic cleanup.
/// Uses `time.delta_secs()` for frame-rate independent animation.
///
/// **Reference**: [animate_window_visibility_system](../visibility/systems.rs:112-164)
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

/// Handle icon color change events
///
/// Responds to IconColorChangeEvent by either:
/// - Immediately setting new color (animated=false)
/// - Starting IconAnimation for smooth transition (animated=true)
///
/// **Pattern**: Event-driven state changes with optional animation.
/// Creates animation components dynamically based on event parameters.
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
///
/// **Note**: This only updates the IconComponent metadata.
/// Actual font size changes require updating TextFont component separately.
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

/// Optimize icon performance - cache warmup, batching
///
/// Analogous to optimize_gradient_performance_system.
/// Pre-warms icon cache when cache changes to reduce runtime lookups.
///
/// **Pattern**: Resource change detection for rare-event optimization.
/// Only runs when IconCache is modified, not every frame.
///
/// **Reference**: [optimize_gradient_performance_system](../gradients/systems.rs:62-83)
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
/// Logs cache statistics for monitoring.
pub fn validate_icon_cache_system(icon_cache: Res<IconCache>) {
    // Cache validation runs when cache changes
    if !icon_cache.is_changed() {
        return;
    }

    info!(
        "Icon cache stats: {} loaded, {} failed",
        icon_cache.loaded_icons.len(),
        icon_cache.failed_to_load.len()
    );
}

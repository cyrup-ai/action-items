//! UI animation systems
//!
//! Zero-allocation animation processing with blazing-fast easing functions and smooth transitions.

use bevy::prelude::*;

use action_items_ecs_ui::gradients::InteractiveGradient;
use action_items_ecs_ui::animations::{AnimationState, AnimationPlayState, EasingFunction};
use crate::ui::components::ActionResultItem;

/// Component to track gradient transition animations
#[derive(Component, Debug)]
pub struct GradientTransition {
    pub from_color: Color,
    pub to_color: Color,
    pub animation: AnimationState,
}

// Type aliases for complex query types
type HoverAnimationQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut Transform,
        &'static mut BackgroundColor,
    ),
    (Changed<Interaction>, With<ActionResultItem>),
>;

/// Animate result items with staggered reveal effects
/// Zero-allocation result animation with blazing-fast staggered timing and smooth easing
#[inline]
pub fn animate_result_items_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut AnimationState), With<ActionResultItem>>,
) {
    let delta_time = time.delta_secs();

    for (mut transform, mut anim_state) in query.iter_mut() {
        if !anim_state.is_complete() {
            anim_state.current_time += delta_time;

            let progress = anim_state.progress();

            // Animate scale and position
            let target_scale = Vec3::splat(1.0);
            let target_position = Vec3::ZERO;

            transform.scale = Vec3::lerp(Vec3::splat(0.98), target_scale, progress);
            transform.translation =
                Vec3::lerp(Vec3::new(0.0, -10.0, 0.0), target_position, progress);

            if anim_state.current_time >= anim_state.duration {
                anim_state.state = AnimationPlayState::Finished;
            }
        }
    }
}

/// Animate hover effects on interactive elements
/// Zero-allocation hover animation with blazing-fast scale and color transitions
#[inline]
pub fn animate_hover_effects_system(mut hover_query: HoverAnimationQuery) {
    for (interaction, mut transform, mut bg_color) in hover_query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                transform.scale = Vec3::splat(1.02);
                // Slightly brighten the background
                let mut color = bg_color.0;
                color = color.with_alpha(color.alpha() * 1.1);
                *bg_color = BackgroundColor(color);
            },
            Interaction::None => {
                transform.scale = Vec3::splat(1.0);
                // Reset background
                let mut color = bg_color.0;
                color = color.with_alpha(color.alpha() / 1.1);
                *bg_color = BackgroundColor(color);
            },
            Interaction::Pressed => {
                transform.scale = Vec3::splat(0.98);
            },
        }
    }
}

/// Type alias for complex interactive gradient query
#[allow(clippy::type_complexity)]
type InteractiveGradientQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Interaction,
        &'static InteractiveGradient,
        &'static mut BackgroundColor,
        Option<&'static mut GradientTransition>,
    ),
    Changed<Interaction>,
>;

/// System for updating gradients with smooth transitions and easing functions
#[inline]
pub fn interactive_gradient_system(
    mut commands: Commands,
    mut query: InteractiveGradientQuery<'_, '_>,
    time: Res<Time>,
) {
    let _delta_time = time.delta_secs();

    for (entity, interaction, interactive, background, transition) in query.iter_mut() {
        let target_color = match *interaction {
            Interaction::Pressed | Interaction::Hovered => interactive.hover_color,
            Interaction::None => interactive.default_color,
        };

        // Start smooth transition if we don't have one or the target changed
        if let Some(mut transition) = transition {
            if transition.to_color != target_color {
                // Target changed - start new transition from current color
                transition.from_color = background.0;
                transition.to_color = target_color;
                transition.animation.reset();
            }
        } else {
            // No transition component - add one
            commands.entity(entity).insert(GradientTransition {
                from_color: background.0,
                to_color: target_color,
                animation: AnimationState::new(0.2, EasingFunction::EaseOut), /* 200ms smooth
                                                                               * transition */
            });
        }
    }
}

/// System to update gradient transition animations
#[inline]
pub fn update_gradient_transitions_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BackgroundColor, &mut GradientTransition)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (entity, mut background, mut transition) in query.iter_mut() {
        if transition.animation.update(delta_time) {
            // Animation complete - set final color and remove component
            background.0 = transition.to_color;
            commands.entity(entity).remove::<GradientTransition>();
        } else {
            // Animation in progress - interpolate color
            let progress = transition.animation.progress();
            let from_rgba = transition.from_color.to_srgba();
            let to_rgba = transition.to_color.to_srgba();
            background.0 = Color::srgba(
                from_rgba.red + (to_rgba.red - from_rgba.red) * progress,
                from_rgba.green + (to_rgba.green - from_rgba.green) * progress,
                from_rgba.blue + (to_rgba.blue - from_rgba.blue) * progress,
                from_rgba.alpha + (to_rgba.alpha - from_rgba.alpha) * progress,
            );
        }
    }
}

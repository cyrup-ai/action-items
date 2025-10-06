//! UI animation systems
//!
//! Zero-allocation animation processing with blazing-fast easing functions and smooth transitions.

use bevy::prelude::*;

use action_items_ecs_ui::animations::{AnimationState, AnimationPlayState};
use crate::ui::components::ActionResultItem;

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

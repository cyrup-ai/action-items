//! Generic visibility animation systems
//!
//! Zero-allocation visibility control with blazing-fast animation and state synchronization.
//! These systems are completely generic and work with any entity that has UiComponentTarget component.

use bevy::prelude::*;

use crate::animations::WindowAnimation;
use super::events::{UiAnimationCompleteEvent, UiVisibilityEvent};
use super::targets::{UiComponentTarget, UiVisibilityAnimationType};

/// Handle UI visibility events with immediate or animated transitions
///
/// This is a GENERIC system that queries UiComponentTarget component.
/// It has no knowledge of app-specific components like LauncherContainer.
#[inline]
pub fn handle_ui_visibility_events(
    mut events: EventReader<UiVisibilityEvent>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &UiComponentTarget,
        &Transform,
        &BackgroundColor,
        &mut Visibility,
    )>,
) {
    for event in events.read() {
        // Determine if we need animation
        if let Some(animation_config) = &event.animation {
            // Start animated transition
            let duration = animation_config.duration.as_secs_f32();

            for (entity, target_component, transform, bg_color, mut visibility) in query.iter_mut()
            {
                // Check if this entity matches the event target
                if !matches_target(&event.target, target_component) {
                    continue;
                }

                // Get current values as initial animation state
                let current_opacity = bg_color.0.alpha();
                let current_scale = transform.scale;

                // Configure animation targets based on visibility
                let (target_opacity, target_scale) = if event.visible {
                    (1.0, Vec3::ONE)
                } else {
                    (0.0, Vec3::splat(0.8))
                };

                // Create appropriate animation with dynamic initial values
                let window_animation = match animation_config.animation_type {
                    UiVisibilityAnimationType::FadeScale => WindowAnimation::with_initial_values(
                        duration,
                        duration,
                        current_opacity,
                        target_opacity,
                        current_scale,
                        target_scale,
                    ),
                    UiVisibilityAnimationType::Fade => WindowAnimation::with_initial_values(
                        duration,
                        0.0,
                        current_opacity,
                        target_opacity,
                        current_scale,
                        current_scale,
                    ),
                    UiVisibilityAnimationType::Scale => WindowAnimation::with_initial_values(
                        0.0,
                        duration,
                        1.0,
                        1.0,
                        current_scale,
                        target_scale,
                    ),
                };

                commands.entity(entity).insert(window_animation);

                // Set visibility immediately for show animations
                if event.visible {
                    *visibility = Visibility::Visible;
                }
            }
        } else {
            // Immediate visibility change
            for (_, target_component, _, _, mut visibility) in query.iter_mut() {
                // Check if this entity matches the event target
                if !matches_target(&event.target, target_component) {
                    continue;
                }

                *visibility = if event.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }

        tracing::debug!(
            "UI visibility event processed: visible={}, animated={}, target={:?}",
            event.visible,
            event.animation.is_some(),
            event.target
        );
    }
}

/// Animate window visibility with smooth fade and scale effects
///
/// This is a GENERIC system that queries UiComponentTarget component.
/// Zero-allocation visibility animation with blazing-fast opacity and scale transitions.
#[inline]
pub fn animate_window_visibility_system(
    mut commands: Commands,
    time: Res<Time>,
    mut completion_events: EventWriter<UiAnimationCompleteEvent>,
    mut query: Query<(
        Entity,
        &UiComponentTarget,
        &mut WindowAnimation,
        &mut Transform,
        &mut Visibility,
        &mut BackgroundColor,
    )>,
) {
    let delta_time = time.delta_secs();

    for (entity, target_component, mut window_anim, mut transform, mut visibility, mut bg_color) in
        query.iter_mut()
    {
        let (opacity, scale) = window_anim.update(delta_time);

        // Apply smooth transform animation
        transform.scale = scale;

        // Set absolute opacity from animation
        let mut color = bg_color.0;
        color = color.with_alpha(opacity);
        *bg_color = BackgroundColor(color);

        // Handle animation completion
        if window_anim.is_complete() {
            let was_show = window_anim.target_opacity > 0.0;

            // For hide animations, set visibility to hidden
            if window_anim.target_opacity == 0.0 {
                *visibility = Visibility::Hidden;
            }

            // Send completion event for app-specific systems to handle
            completion_events.write(UiAnimationCompleteEvent {
                target: *target_component,
                was_show,
            });

            // Clean up completed animations
            commands.entity(entity).remove::<WindowAnimation>();
        }
    }
}

/// Helper function to check if an event target matches an entity's component target
#[inline]
fn matches_target(event_target: &UiComponentTarget, entity_target: &UiComponentTarget) -> bool {
    event_target == &UiComponentTarget::All || event_target == entity_target
}

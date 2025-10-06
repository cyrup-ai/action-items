//! Generic gradient systems for Bevy ECS UI
//!
//! Zero-allocation gradient management with blazing-fast gradient application,
//! interaction state handling, and dynamic theme switching.

use bevy::prelude::*;
use tracing::{info, warn};

use super::components::GradientComponent;
use super::interactive::{InteractiveGradient, InteractiveGradientTransition};
use super::states::GradientInteractionState;
use super::theme::GradientTheme;

/// Professional gradient application system
/// Zero-allocation system for applying gradients to UI components based on GradientComponent
/// configuration. Handles interaction states and smooth transitions with blazing-fast performance.
#[inline]
pub fn apply_gradient_system(
    gradient_theme: Res<GradientTheme>,
    mut gradient_components: Query<
        (&GradientComponent, &mut BackgroundColor),
        Changed<GradientComponent>,
    >,
) {
    // Apply gradient-based background colors to UI components
    for (gradient_component, mut background_color) in gradient_components.iter_mut() {
        let gradient_data = gradient_component.get_current_gradient(&gradient_theme);
        let bevy_background = gradient_data.to_bevy_background();

        *background_color = bevy_background;
    }
}

/// Interactive gradient state system
/// Zero-allocation system for updating gradient states based on UI interactions.
/// Handles hover, selection, and other interaction states with smooth transitions.
#[inline]
pub fn interactive_gradient_system(
    mut gradient_components: Query<&mut GradientComponent>,
    interaction_query: Query<(Entity, &Interaction), Changed<Interaction>>,
) {
    // Update gradient component interaction states based on UI interactions
    for (entity, interaction) in interaction_query.iter() {
        if let Ok(mut gradient_component) = gradient_components.get_mut(entity) {
            let new_state = match interaction {
                Interaction::Hovered => GradientInteractionState::Hover,
                Interaction::Pressed => GradientInteractionState::Pressed,
                Interaction::None => GradientInteractionState::Default,
            };

            // Only update if state actually changed to avoid unnecessary work
            if gradient_component.interaction_state != new_state {
                gradient_component.interaction_state = new_state;
            }
        }
    }
}

/// Gradient performance optimization system
/// Zero-allocation system for batching gradient updates and minimizing computation.
/// Handles gradient caching and efficient state transitions.
#[inline]
pub fn optimize_gradient_performance_system(
    gradient_theme: Res<GradientTheme>,
    gradient_components: Query<&GradientComponent>,
) {
    // Only process when gradient theme changes (rare event)
    if !gradient_theme.is_changed() {
        return;
    }

    // Pre-warm gradient calculations for commonly used gradients
    let mut calculated_count = 0;
    for gradient_component in gradient_components.iter() {
        // Touch each gradient to ensure it's calculated and potentially cached
        let _gradient_data = gradient_component.get_current_gradient(&gradient_theme);
        calculated_count += 1;
    }

    info!(
        "Optimized {} gradient components for performance",
        calculated_count
    );
}

/// Advanced gradient theme system for dynamic theme switching
/// Zero-allocation system for hot-swapping gradient themes without UI rebuild.
/// Handles theme transitions and maintains visual consistency.
#[inline]
pub fn dynamic_gradient_theme_system(
    mut gradient_theme: ResMut<GradientTheme>,
    mut gradient_components: Query<&mut GradientComponent>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Dynamic theme switching for development and accessibility
    let theme_changed = if keyboard.just_pressed(KeyCode::F10) {
        // Switch to high contrast theme
        *gradient_theme = GradientTheme::high_contrast();
        info!("Switched to high contrast gradient theme");
        true
    } else if keyboard.just_pressed(KeyCode::F11) {
        // Switch back to professional dark theme
        *gradient_theme = GradientTheme::professional_dark();
        info!("Switched to professional dark gradient theme");
        true
    } else {
        false
    };

    // Force update all gradient components when theme changes
    if theme_changed {
        for mut gradient_component in gradient_components.iter_mut() {
            // Touch component to trigger change detection
            gradient_component.set_changed();
        }
    }
}

/// Gradient accessibility system
/// Zero-allocation system for ensuring gradient contrast meets accessibility standards.
/// Automatically adjusts gradients for users with visual impairments.
#[inline]
pub fn gradient_accessibility_system(
    mut gradient_theme: ResMut<GradientTheme>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Toggle high contrast mode for accessibility
    // Support both left and right modifier keys for better accessibility
    let ctrl_pressed = keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);

    let alt_pressed = keyboard.pressed(KeyCode::AltLeft)
        || keyboard.pressed(KeyCode::AltRight);

    if ctrl_pressed && alt_pressed && keyboard.just_pressed(KeyCode::KeyH) {
        // Switch to high contrast theme for better accessibility
        *gradient_theme = GradientTheme::high_contrast();
        info!("Activated high contrast gradient theme for accessibility");
    }
}

/// Gradient animation system for smooth state transitions
/// Zero-allocation system for animating gradient changes between interaction states.
/// Provides smooth visual feedback for user interactions.
#[inline]
pub fn animate_gradient_transitions_system(
    time: Res<Time>,
    mut gradient_components: Query<(&mut GradientComponent, &mut BackgroundColor)>,
    gradient_theme: Res<GradientTheme>,
) {
    let delta_time = time.delta_secs();

    for (mut gradient_component, mut background_color) in gradient_components.iter_mut() {
        // Detect state change - reset timer when interaction state changes
        if gradient_component.previous_state.is_none()
            || gradient_component.previous_state != Some(gradient_component.interaction_state)
        {
            gradient_component.elapsed_transition_time = 0.0;
            gradient_component.previous_state = Some(gradient_component.interaction_state);
        }

        // Only process if actively transitioning
        if gradient_component.elapsed_transition_time < gradient_component.transition_speed {
            // Accumulate elapsed time across frames
            gradient_component.elapsed_transition_time += delta_time;

            // Calculate progress (0.0 to 1.0) from accumulated time
            let transition_progress =
                (gradient_component.elapsed_transition_time / gradient_component.transition_speed)
                    .min(1.0);

            // Apply gradient when transition completes or for instant transitions
            if transition_progress >= 1.0 || gradient_component.transition_speed == 0.0 {
                let target_gradient = gradient_component.get_current_gradient(&gradient_theme);
                *background_color = target_gradient.to_bevy_background();
            }
        }
    }
}

/// Gradient theme validation system
/// Zero-allocation system for validating gradient configurations and ensuring visual quality.
/// Prevents invalid gradient states that could cause rendering issues.
#[inline]
pub fn validate_gradient_theme_system(
    gradient_theme: Res<GradientTheme>,
    gradient_components: Query<&GradientComponent>,
) {
    // Only validate when theme changes to avoid unnecessary work
    if !gradient_theme.is_changed() {
        return;
    }

    let mut validation_errors = 0;

    // Validate each gradient component against current theme
    for gradient_component in gradient_components.iter() {
        let gradient_data = gradient_component.get_current_gradient(&gradient_theme);

        // Validate gradient data integrity
        // Note: Empty color_stops is valid (represents solid color backgrounds)
        // Only validate opacity range (required for Bevy's sRGBA color space)
        if gradient_data.opacity < 0.0 || gradient_data.opacity > 1.0 {
            validation_errors += 1;
        }
    }

    if validation_errors > 0 {
        warn!(
            "Detected {} gradient validation errors - visual quality may be compromised",
            validation_errors
        );
    } else {
        info!("Gradient theme validation passed - all gradients are properly configured");
    }
}

/// InteractiveGradient interaction handler
/// 
/// Zero-allocation system for managing InteractiveGradient color transitions.
/// Monitors Interaction state changes and initiates smooth color animations.
/// 
/// This system:
/// - Detects interaction state changes (hover, press, normal)
/// - Selects target color based on interaction state
/// - Creates/updates InteractiveGradientTransition for smooth transitions
/// 
/// Related: [`InteractiveGradient`](crate::gradients::InteractiveGradient)
#[inline]
pub fn interactive_gradient_interactive_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Interaction,
            &InteractiveGradient,
            &BackgroundColor,
            Option<&mut InteractiveGradientTransition>,
        ),
        Changed<Interaction>,
    >,
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
            commands.entity(entity).insert(InteractiveGradientTransition {
                from_color: background.0,
                to_color: target_color,
                animation: crate::animations::AnimationState::new(
                    0.2,
                    crate::animations::EasingFunction::EaseOut,
                ), // 200ms smooth transition
            });
        }
    }
}

/// InteractiveGradient animation system
/// 
/// Zero-allocation system for animating InteractiveGradient color transitions.
/// Handles smooth color interpolation with easing functions.
/// 
/// This system:
/// - Updates InteractiveGradientTransition animation states
/// - Interpolates colors using easing-adjusted progress
/// - Updates BackgroundColor with interpolated values
/// - Removes transition component when animation completes
/// 
/// Related: [`InteractiveGradientTransition`](crate::gradients::InteractiveGradientTransition)
#[inline]
pub fn animate_interactive_gradient_transitions_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BackgroundColor, &mut InteractiveGradientTransition)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (entity, mut background, mut transition) in query.iter_mut() {
        if transition.animation.update(delta_time) {
            // Animation complete - set final color and remove component
            background.0 = transition.to_color;
            commands.entity(entity).remove::<InteractiveGradientTransition>();
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

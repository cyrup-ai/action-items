//! Gradient system plugin for Bevy integration

use bevy::prelude::*;
use super::theme::GradientTheme;
use super::systems::*;

/// Plugin for gradient theming system
///
/// Registers GradientTheme resource and gradient systems.
/// Provides complete gradient functionality including:
/// - Theme management (professional dark, high contrast)
/// - Interactive state handling (hover, press, selection)
/// - Smooth animations and transitions
/// - Accessibility features (Ctrl+Alt+H for high contrast)
/// - Dynamic theme switching (F10/F11 keys)
/// - Performance optimization and validation
///
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use action_items_ecs_ui::gradients::GradientPlugin;
///
/// App::new()
///     .add_plugins(GradientPlugin)
///     .run();
/// ```
#[derive(Debug, Default, Clone)]
pub struct GradientPlugin;

impl Plugin for GradientPlugin {
    fn build(&self, app: &mut App) {
        app
            // Insert default gradient theme (professional dark)
            .insert_resource(GradientTheme::professional_dark())
            // Core gradient systems (run every frame)
            .add_systems(
                Update,
                (
                    apply_gradient_system,
                    interactive_gradient_system,
                    animate_gradient_transitions_system,
                    interactive_gradient_interactive_system,
                    animate_interactive_gradient_transitions_system,
                ),
            )
            // Performance and validation systems (run when theme changes)
            .add_systems(
                Update,
                (
                    optimize_gradient_performance_system,
                    validate_gradient_theme_system,
                )
                    .run_if(resource_changed::<GradientTheme>),
            )
            // User interaction systems (theme switching, accessibility)
            .add_systems(
                Update,
                (dynamic_gradient_theme_system, gradient_accessibility_system),
            );
    }
}

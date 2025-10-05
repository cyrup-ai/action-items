//! Responsive layout plugin for Bevy integration

use bevy::prelude::*;
use super::systems::*;

/// Plugin for responsive layout system
///
/// Registers responsive layout systems for viewport-responsive containers,
/// content constraints, and text truncation.
///
/// # Features
/// - Viewport-responsive container sizing with Vw/Vh units
/// - Automatic content height constraints based on child count
/// - Pixel-perfect text truncation using Bevy's TextLayoutInfo measurements
/// - Change-detection optimized for zero-allocation performance
///
/// # Systems Registered
/// - `update_viewport_responsive_container_system` - Syncs ViewportResponsiveContainer to Node
/// - `content_constraints_system` - Enforces max height based on visible item limits
/// - `text_truncation_system` - Iterative truncation with real pixel measurements
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use action_items_ecs_ui::responsive::ResponsivePlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(ResponsivePlugin)
///         .run();
/// }
/// ```
#[derive(Debug, Default, Clone)]
pub struct ResponsivePlugin;

impl Plugin for ResponsivePlugin {
    fn build(&self, app: &mut App) {
        app
            // Responsive layout systems (run on component changes only)
            .add_systems(
                Update,
                (
                    update_viewport_responsive_container_system,
                    content_constraints_system,
                    text_truncation_system,
                ),
            );
    }
}

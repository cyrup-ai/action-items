//! Monitor-aware constraint system for flex layout with screen-relative max-width/max-height
//!
//! This system fixes the critical bug where Val::Percent is resolved against window dimensions
//! instead of actual screen dimensions. Provides max-width/max-height constraints based on
//! the active monitor's physical dimensions for proper flex layout behavior.

use bevy::prelude::*;
use bevy::window::{Monitor, PrimaryMonitor};
use tracing::info;

/// Resource containing the active monitor's dimensions for constraint calculations
#[derive(Resource, Debug, Clone)]
pub struct MonitorConstraints {
    /// Physical width of the active monitor in pixels
    pub screen_width: f32,
    /// Physical height of the active monitor in pixels  
    pub screen_height: f32,
    /// DPI scale factor for proper scaling
    pub scale_factor: f32,
    /// Logical width (physical_width / scale_factor)
    pub logical_width: f32,
    /// Logical height (physical_height / scale_factor)
    pub logical_height: f32,
}

impl Default for MonitorConstraints {
    fn default() -> Self {
        Self {
            screen_width: 1920.0,  // Default fallback
            screen_height: 1080.0, // Default fallback
            scale_factor: 1.0,
            logical_width: 1920.0,
            logical_height: 1080.0,
        }
    }
}

impl MonitorConstraints {
    /// Calculate max-width as percentage of screen width
    #[inline]
    pub fn max_width_percent(&self, percent: f32) -> Val {
        Val::Px(self.logical_width * (percent / 100.0))
    }

    /// Calculate max-height as percentage of screen height
    #[inline]
    pub fn max_height_percent(&self, percent: f32) -> Val {
        Val::Px(self.logical_height * (percent / 100.0))
    }

    /// Get full screen width constraint
    #[inline]
    pub fn max_width_full(&self) -> Val {
        Val::Px(self.logical_width)
    }

    /// Get full screen height constraint
    #[inline]
    pub fn max_height_full(&self) -> Val {
        Val::Px(self.logical_height)
    }

    /// Calculate width for launcher centering (85% of screen, max 800px)
    #[inline]
    pub fn launcher_max_width(&self) -> Val {
        Val::Px((self.logical_width * 0.85).min(800.0)) // Max 85% screen or 800px
    }

    /// Calculate height for launcher (content-driven, max 600px, but constrained by screen height
    /// with margin)
    #[inline]
    pub fn launcher_max_height(&self) -> Val {
        // Leave 30px margin (15px top + 15px bottom) for standard modal sizing
        let screen_constrained_height = (self.logical_height - 30.0).max(200.0); // Minimum 200px
        let max_height = screen_constrained_height.min(600.0); // Max 600px, but respect screen constraints
        Val::Px(max_height)
    }
}

/// Component marker for UI elements that need monitor-based constraints
#[derive(Component, Debug, Default)]
pub struct MonitorConstrained {
    /// Width percentage (0.0 - 100.0) of screen width
    pub width_percent: Option<f32>,
    /// Height percentage (0.0 - 100.0) of screen height  
    pub height_percent: Option<f32>,
    /// Whether this is a launcher container (uses launcher-specific constraints)
    pub is_launcher: bool,
}

impl MonitorConstrained {
    /// Create constraint for full screen coverage
    pub fn full_screen() -> Self {
        Self {
            width_percent: Some(100.0),
            height_percent: Some(100.0),
            is_launcher: false,
        }
    }

    /// Create constraint for width only
    pub fn width_percent(width_percent: f32) -> Self {
        Self {
            width_percent: Some(width_percent),
            height_percent: None,
            is_launcher: false,
        }
    }
}

/// System to update monitor constraints when monitor configuration changes
pub fn update_monitor_constraints_system(
    monitors: Query<&Monitor, With<PrimaryMonitor>>,
    all_monitors: Query<&Monitor>,
    mut constraints: ResMut<MonitorConstraints>,
) {
    let monitor = match monitors.single() {
        Ok(primary) => Some(primary),
        Err(_) => {
            // Fallback: try to find any available monitor if primary not found
            all_monitors.iter().next()
        },
    };

    if let Some(monitor) = monitor {
        // Validate monitor data before using it
        let physical_width = monitor.physical_width.max(1); // Ensure minimum size
        let physical_height = monitor.physical_height.max(1);
        let scale_factor = monitor.scale_factor.clamp(0.1, 10.0); // Reasonable scale factor range

        // Ensure reasonable minimum dimensions (at least 320x240 logical pixels)
        let logical_width = (physical_width as f32 / scale_factor as f32).max(320.0);
        let logical_height = (physical_height as f32 / scale_factor as f32).max(240.0);

        // Ensure maximum reasonable dimensions (prevent overflow in calculations)
        let screen_width = (physical_width as f32).min(32768.0);
        let screen_height = (physical_height as f32).min(32768.0);

        let new_constraints = MonitorConstraints {
            screen_width,
            screen_height,
            scale_factor: scale_factor as f32,
            logical_width,
            logical_height,
        };

        // Only update if dimensions actually changed to avoid unnecessary recalculations
        if (constraints.screen_width - new_constraints.screen_width).abs() > 1.0
            || (constraints.screen_height - new_constraints.screen_height).abs() > 1.0
            || (constraints.scale_factor - new_constraints.scale_factor).abs() > 0.01
        {
            *constraints = new_constraints;
            info!(
                "Updated monitor constraints: {}x{} (logical: {:.1}x{:.1}, scale: {:.2})",
                constraints.screen_width,
                constraints.screen_height,
                constraints.logical_width,
                constraints.logical_height,
                constraints.scale_factor
            );
        }
    } else if constraints.screen_width == 0.0 || constraints.screen_height == 0.0 {
        // No monitors detected - check if we need to use fallback values
        info!("No monitors detected, using fallback constraints: 1920x1080");
        *constraints = MonitorConstraints::default();
    }
}

/// System to apply monitor-based constraints to UI nodes
pub fn apply_monitor_constraints_system(
    constraints: Res<MonitorConstraints>,
    mut constrained_nodes: Query<(&mut Node, &MonitorConstrained)>,
    changed_components: Query<Entity, Changed<MonitorConstrained>>,
) {
    // Only process if monitor constraints changed OR any MonitorConstrained components changed
    let constraints_changed = constraints.is_changed();
    let components_changed = !changed_components.is_empty();

    if !constraints_changed && !components_changed {
        return;
    }

    // Process nodes based on what changed
    if constraints_changed {
        // Update all constrained nodes when constraints change
        for (mut node, constraint) in constrained_nodes.iter_mut() {
            apply_constraint_to_node(&mut node, constraint, &constraints);
        }
    } else {
        // Update only changed components
        for entity in changed_components.iter() {
            if let Ok((mut node, constraint)) = constrained_nodes.get_mut(entity) {
                apply_constraint_to_node(&mut node, constraint, &constraints);
            }
        }
    }
}

fn apply_constraint_to_node(
    node: &mut Node,
    constraint: &MonitorConstrained,
    constraints: &MonitorConstraints,
) {
    if constraint.is_launcher {
        // Apply launcher-specific constraints with validation
        let max_width = constraints.launcher_max_width();
        let max_height = constraints.launcher_max_height();

        // Validate constraint values are reasonable (minimum 200px, maximum 8192px)
        if let Val::Px(width) = max_width
            && (200.0..=8192.0).contains(&width)
        {
            node.max_width = max_width;
        }
        if let Val::Px(height) = max_height
            && (150.0..=8192.0).contains(&height)
        {
            node.max_height = max_height;
        }
    } else {
        // Apply percentage-based constraints with validation
        if let Some(width_percent) = constraint.width_percent {
            // Validate percentage is reasonable (0.1% to 100%)
            if (0.1..=100.0).contains(&width_percent) {
                let max_width = constraints.max_width_percent(width_percent);
                if let Val::Px(width) = max_width
                    && (50.0..=8192.0).contains(&width)
                {
                    node.max_width = max_width;
                }
            }
        }
        if let Some(height_percent) = constraint.height_percent {
            // Validate percentage is reasonable (0.1% to 100%)
            if (0.1..=100.0).contains(&height_percent) {
                let max_height = constraints.max_height_percent(height_percent);
                if let Val::Px(height) = max_height
                    && (30.0..=8192.0).contains(&height)
                {
                    node.max_height = max_height;
                }
            }
        }
    }
}

/// Plugin to register monitor constraint systems
pub struct MonitorConstraintsPlugin;

impl Plugin for MonitorConstraintsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MonitorConstraints>().add_systems(
            Update,
            (
                update_monitor_constraints_system,
                apply_monitor_constraints_system.after(update_monitor_constraints_system),
            ),
        );
    }
}

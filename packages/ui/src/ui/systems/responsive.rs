//! Application-specific responsive container system for Action Items launcher
//!
//! This module contains responsive systems specific to the Action Items application.
//! Generic responsive systems are provided by the ecs-ui crate.

use bevy::prelude::*;
use tracing::info;

use crate::ui::components::{LauncherContainer, ViewportResponsiveContainer};
use crate::ui::systems::monitor_constraints::MonitorConstraints;

/// Adaptive container system for extreme screen configurations
/// Automatically adjusts ViewportResponsiveContainer settings based on screen aspect ratio and size
#[inline]
pub fn adaptive_container_system(
    monitor_constraints: Res<MonitorConstraints>,
    mut containers: Query<&mut ViewportResponsiveContainer, With<LauncherContainer>>,
) {
    // Only process when monitor constraints change
    if !monitor_constraints.is_changed() {
        return;
    }

    let screen_width = monitor_constraints.logical_width;
    let screen_height = monitor_constraints.logical_height;
    let aspect_ratio = screen_width / screen_height.max(1.0);

    for mut container in containers.iter_mut() {
        // Adapt container settings based on screen characteristics
        match aspect_ratio {
            // Ultra-wide screens (>2.0 aspect ratio): use compact width
            ratio if ratio > 2.0 => {
                container.width_vw = container.width_vw.min(45.0);
                container.max_width_vw = container.max_width_vw.min(60.0);
            },
            // Very narrow screens (<1.2 aspect ratio): expand vertically
            ratio if ratio < 1.2 => {
                container.height_vh = container.height_vh.max(60.0);
                container.max_height_vh = container.max_height_vh.max(85.0);
            },
            // Standard screens: use default settings
            _ => {}, // No adjustment needed
        }

        // Adapt based on absolute screen size
        match screen_width {
            // Very small screens (<1024px): use compact variant
            width if width < 1024.0 => {
                let compact = ViewportResponsiveContainer::compact();
                container.width_vw = compact.width_vw;
                container.height_vh = compact.height_vh;
                container.max_width_vw = compact.max_width_vw;
                container.max_height_vh = compact.max_height_vh;
            },
            // Very large screens (>2560px): use expanded variant
            width if width > 2560.0 => {
                let expanded = ViewportResponsiveContainer::expanded();
                container.width_vw = expanded.width_vw;
                container.height_vh = expanded.height_vh;
                container.max_width_vw = expanded.max_width_vw;
                container.max_height_vh = expanded.max_height_vh;
            },
            // Standard screens: keep current settings
            _ => {},
        }

        info!(
            "Adapted container for screen {}x{} (aspect: {:.2}): {}vw x {}vh",
            screen_width, screen_height, aspect_ratio, container.width_vw, container.height_vh
        );
    }
}

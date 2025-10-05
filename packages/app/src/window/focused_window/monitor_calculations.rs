//! Monitor calculations for focused window positioning

use bevy::prelude::*;
use bevy::window::Monitor;

use super::types::WindowBounds;

/// Calculate the best monitor for a window based on overlap percentage
/// Returns the monitor entity with the highest overlap, or None if no overlap
#[inline]
pub fn calculate_best_monitor_for_window(
    window_bounds: &WindowBounds,
    monitors: &Query<(Entity, &Monitor)>,
) -> Option<(Entity, f64)> {
    let mut best_monitor = None;
    let mut best_overlap = 0.0;

    for (entity, monitor) in monitors.iter() {
        let monitor_bounds = WindowBounds::from(monitor);
        let overlap = window_bounds.overlap_percentage(&monitor_bounds);

        if overlap > best_overlap {
            best_overlap = overlap;
            best_monitor = Some((entity, overlap));
        }
    }

    best_monitor
}

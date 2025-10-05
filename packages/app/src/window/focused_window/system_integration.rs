//! Bevy system integration for focused window detection

use bevy::prelude::*;
use bevy::window::Monitor;
use tracing::{debug, warn};

use super::monitor_calculations::calculate_best_monitor_for_window;
use super::platform::get_focused_window_bounds;
use super::types::FocusedWindowError;

/// Professional system to detect focused window and update active monitor
/// Zero allocation, blazing-fast implementation with comprehensive error handling
#[inline]
pub fn detect_focused_window_monitor_system(
    mut active_monitor: ResMut<crate::window::positioning::ActiveMonitor>,
    monitors_query: Query<(Entity, &Monitor)>,
) {
    // Only run detection if we have monitors to work with
    if monitors_query.is_empty() {
        return;
    }

    match get_focused_window_bounds() {
        Ok(focused_window_bounds) => {
            // Calculate which monitor has the best overlap with the focused window
            if let Some((best_monitor_entity, overlap_percentage)) =
                calculate_best_monitor_for_window(&focused_window_bounds, &monitors_query)
            {
                // Only update if this is a significant change (avoid thrashing)
                if active_monitor.target != Some(best_monitor_entity) {
                    active_monitor.target = Some(best_monitor_entity);

                    // Get monitor info for logging
                    if let Ok((_, monitor)) = monitors_query.get(best_monitor_entity) {
                        let monitor_name = monitor.name.as_deref().unwrap_or("Unknown Display");
                        debug!(
                            "Active monitor updated to: {} ({:.1}% overlap with focused window at \
                             {}x{} +{},+{})",
                            monitor_name,
                            overlap_percentage * 100.0,
                            focused_window_bounds.width,
                            focused_window_bounds.height,
                            focused_window_bounds.x,
                            focused_window_bounds.y
                        );
                    }
                }
            } else if active_monitor.target != active_monitor.primary {
                // No good overlap found, fall back to primary monitor
                active_monitor.target = active_monitor.primary;
                debug!(
                    "No monitor overlap found for focused window, using primary monitor fallback"
                );
            }
        },
        Err(FocusedWindowError::NoFocusedWindow) => {
            // No focused window detected, use primary monitor
            if active_monitor.target != active_monitor.primary {
                active_monitor.target = active_monitor.primary;
                debug!("No focused window detected, using primary monitor");
            }
        },
        Err(FocusedWindowError::DisplayServerNotFound) => {
            // Display server not found, use primary monitor
            if active_monitor.target != active_monitor.primary {
                active_monitor.target = active_monitor.primary;
                debug!("Display server not found, using primary monitor");
            }
        },
        Err(FocusedWindowError::CompositorNotSupported(_)) => {
            // Wayland compositor doesn't support required protocol, use primary monitor
            if active_monitor.target != active_monitor.primary {
                active_monitor.target = active_monitor.primary;
                debug!("Wayland compositor not supported, using primary monitor");
            }
        },
        Err(e) => {
            // Other errors, log and use primary monitor as fallback
            warn!(
                "Focused window detection error: {}, using primary monitor fallback",
                e
            );
            if active_monitor.target != active_monitor.primary {
                active_monitor.target = active_monitor.primary;
            }
        },
    }
}

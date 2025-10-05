use bevy::prelude::*;

use crate::window::{ActiveMonitor, LauncherState};

/// System to handle window resize events for responsive UI scaling
/// Updates UiScale and launcher state when window is resized following window_resizing.rs pattern
#[inline]
pub fn handle_window_resized_system(
    mut resize_events: EventReader<bevy::window::WindowResized>,
    mut ui_scale: ResMut<UiScale>,
    mut launcher_state: ResMut<LauncherState>,
    monitors_query: Query<&bevy::window::Monitor>,
    active_monitor: Res<ActiveMonitor>,
) {
    for resize_event in resize_events.read() {
        // Window resize handling

        // Recalculate UI scaling based on new window size relative to monitor
        if let Some(monitor_entity) = active_monitor.target.or(active_monitor.primary)
            && let Ok(monitor) = monitors_query.get(monitor_entity)
        {
            let monitor_width = monitor.physical_width as f32;
            let monitor_height = monitor.physical_height as f32;

            // Calculate scale factor based on window size relative to monitor
            let width_ratio = resize_event.width / monitor_width;
            let height_ratio = resize_event.height / monitor_height;
            let scale_factor = (width_ratio + height_ratio) * 0.5;

            // Update UI scale to maintain proportional elements
            ui_scale.0 = scale_factor.clamp(0.5, 2.0);

            // Update launcher state dimensions proportionally
            launcher_state.current_height = resize_event.height;
            launcher_state.target_height = resize_event.height;
        }
    }
}

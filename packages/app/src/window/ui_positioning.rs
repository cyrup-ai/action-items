//! UI positioning integration with multi-monitor system
//!
//! Connects the ActiveMonitor with UI positioning for proper screen placement.

use action_items_ui::prelude::*;
use bevy::prelude::*;
use tracing::info;

use crate::window::positioning::ActiveMonitor;

/// System to position UI elements based on multi-monitor configuration
/// Integrates ActiveMonitor with UI root positioning
#[inline]
pub fn position_ui_on_correct_monitor(
    active_monitor: Res<ActiveMonitor>,
    mut ui_root_query: Query<&mut Node, With<UiRoot>>,
    mut launcher_container_query: Query<&mut Node, (With<LauncherContainer>, Without<UiRoot>)>,
) {
    // Only run when ActiveMonitor changes, not every frame!
    if !active_monitor.is_changed() {
        return;
    }

    // Only reposition if we have monitor information
    if active_monitor.target.is_none() && active_monitor.primary.is_none() {
        return;
    }

    // Get target monitor (cursor-based or primary fallback)
    let target_monitor = active_monitor.target.or(active_monitor.primary);

    if let Some(_monitor_entity) = target_monitor {
        // Position UI root to take advantage of monitor positioning
        if let Ok(mut ui_root_node) = ui_root_query.single_mut() {
            // Maintain full screen coverage but position according to monitor
            ui_root_node.width = Val::Percent(100.0);
            ui_root_node.height = Val::Percent(100.0);
            ui_root_node.justify_content = JustifyContent::Center;
            ui_root_node.align_items = AlignItems::Center;
        }

        // Adjust launcher container positioning for optimal monitor placement
        if let Ok(mut launcher_container_node) = launcher_container_query.single_mut() {
            // Use responsive viewport units instead of hardcoded calculations
            launcher_container_node.width = Val::Vw(45.0); // 45% viewport width
            launcher_container_node.height = Val::Vh(35.0); // 35% viewport height

            // Removed excessive logging
        }
    }
}

/// System to update UI positioning when monitor configuration changes
/// Responds to monitor detection and cursor movement events  
#[inline]
pub fn update_ui_monitor_positioning(
    active_monitor: Res<ActiveMonitor>,
    mut ui_root_query: Query<&mut Node, With<UiRoot>>,
) {
    if active_monitor.is_changed()
        && let Ok(mut ui_root_node) = ui_root_query.single_mut()
    {
        // Force UI repositioning when monitor config changes
        ui_root_node.justify_content = JustifyContent::Center;
        ui_root_node.align_items = AlignItems::Center;

        info!("UI repositioning triggered by monitor configuration change");
    }
}

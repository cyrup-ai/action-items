//! Cleanup and metrics systems with high-performance batch processing

use bevy::prelude::*;
use tracing::debug;

use crate::components::*;
use crate::events::*;
use crate::resources::*;

/// Clean up completed operations with high-performance batch processing
#[inline(always)]
pub fn cleanup_completed_operations_system(
    mut commands: Commands,
    completed_actions: Query<(Entity, &ActionExecution)>,
    completed_searches: Query<(Entity, &SearchOperation), Without<ActionExecution>>,
    config: Res<LauncherConfig>,
) {
    let now = std::time::Instant::now();
    let cleanup_threshold = std::time::Duration::from_secs(300); // 5 minutes

    // Clean up completed actions
    for (entity, action) in completed_actions.iter() {
        if matches!(
            action.status,
            ExecutionStatus::Completed | ExecutionStatus::Failed
        )
            && let Some(completed_at) = action.completed_at
                && now.duration_since(completed_at) > cleanup_threshold {
                    if config.enable_debug_logging {
                        debug!("Cleaning up completed action: {}", action.action_id);
                    }
                    commands.entity(entity).despawn();
                }
    }

    // Clean up completed searches
    for (entity, search) in completed_searches.iter() {
        if matches!(
            search.status,
            SearchStatus::Completed | SearchStatus::Failed
        )
            && let Some(completed_at) = search.completed_at
                && now.duration_since(completed_at) > cleanup_threshold {
                    if config.enable_debug_logging {
                        debug!("Cleaning up completed search: {}", search.query);
                    }
                    commands.entity(entity).despawn();
                }
    }
}

/// Update launcher metrics with blazing-fast atomic operations
#[inline(always)]
pub fn update_launcher_metrics_system(
    search_events: EventReader<SearchCompleted>,
    mut action_events: EventReader<ActionExecuteCompleted>,
    mut launcher_metrics: ResMut<LauncherMetrics>,
    plugin_registry: Res<PluginRegistry>,
) {
    // Count searches
    let search_count = search_events.len();
    if search_count > 0 {
        launcher_metrics.total_searches += search_count as u64;
    }

    // Count actions
    for action_event in action_events.read() {
        if action_event.success {
            launcher_metrics.successful_actions += 1;
        } else {
            launcher_metrics.failed_actions += 1;
        }
    }

    // Update plugin count
    launcher_metrics.plugin_count = plugin_registry.discovered_plugins.len();

    launcher_metrics.last_updated = Some(std::time::Instant::now());
}

use bevy::prelude::*;
use log::{debug, error, info, warn};

use super::counter::PluginCounter;
use super::names::PluginNames;
// Note: Plugin components will be used when ECS query system is fully implemented

/// System to handle plugin queries and management
pub fn plugin_query_system(plugin_counter: PluginCounter, plugin_names: PluginNames) {
    let total_count = plugin_counter.count();

    if total_count > 0 {
        let grouped_names = plugin_names.grouped_by_type();

        debug!(
            "Plugin query system processing {} plugins: {} native, {} extism, {} raycast",
            total_count,
            grouped_names.native.len(),
            grouped_names.extism.len(),
            grouped_names.raycast.len()
        );

        // Log individual plugin names for detailed debugging
        if !grouped_names.native.is_empty() {
            debug!("Native plugins: {}", grouped_names.native.join(", "));
        }
        if !grouped_names.extism.is_empty() {
            debug!("Extism plugins: {}", grouped_names.extism.join(", "));
        }
        if !grouped_names.raycast.is_empty() {
            debug!("Raycast plugins: {}", grouped_names.raycast.join(", "));
        }

        // Validate plugin name consistency
        let expected_count = grouped_names.total_count();
        if expected_count != total_count {
            warn!(
                "Plugin count mismatch: counter reports {}, names report {}",
                total_count, expected_count
            );
        }
    } else {
        debug!("Plugin query system: no plugins loaded");
    }
}

/// System to log all loaded plugins (useful for debugging)
pub fn log_plugins_system(plugin_counter: PluginCounter, plugin_names: PluginNames) {
    let total_count = plugin_counter.count();
    let native_count = plugin_counter.native_count();
    let extism_count = plugin_counter.extism_count();
    let raycast_count = plugin_counter.raycast_count();

    info!(
        "Total plugins: {} (Native: {}, WASM: {}, Raycast: {})",
        total_count, native_count, extism_count, raycast_count
    );

    if total_count == 0 {
        info!(
            "No plugins loaded. The async plugin loading system will discover plugins at startup."
        );
        return;
    }

    let mut index = 1;

    // Log native plugins
    for name in plugin_names.native_names() {
        info!("  {}. {} (Native)", index, name);
        index += 1;
    }

    // Log extism plugins
    for name in plugin_names.extism_names() {
        info!("  {}. {} (WASM)", index, name);
        index += 1;
    }

    // Log raycast plugins
    for name in plugin_names.raycast_names() {
        info!("  {}. {} (Raycast)", index, name);
        index += 1;
    }
}

/// System to print plugin statistics (useful for debugging)
pub fn print_plugin_stats(plugin_counter: PluginCounter) {
    let total = plugin_counter.count();
    if total > 0 {
        debug!(
            "Plugin Stats: {} total ({} Native, {} WASM, {} Raycast)",
            total,
            plugin_counter.native_count(),
            plugin_counter.extism_count(),
            plugin_counter.raycast_count()
        );
    }
}

/// ECS-based system to handle search results
pub fn handle_search_results_system_ecs(
    mut commands: Commands,
    mut pending_searches: Query<(Entity, &mut crate::plugins::core::PendingActionResult)>,
    action_map: Res<ActionMap>,
    mut current_search_results: ResMut<crate::plugins::core::CurrentSearchResults>,
) {
    let mut all_new_results: Vec<crate::plugins::core::ActionItem> = Vec::new();
    let mut any_task_completed_or_failed = false;

    for (entity, mut pending_search) in pending_searches.iter_mut() {
        match bevy::tasks::block_on(bevy::tasks::futures_lite::future::poll_once(
            &mut pending_search.task,
        )) {
            Some(Ok(interface_results_vec)) => {
                any_task_completed_or_failed = true;
                for interface_result in interface_results_vec {
                    let local_action_item: crate::plugins::core::ActionItem = interface_result;

                    if let Err(e) = action_map.insert(
                        local_action_item.action.clone(),
                        pending_search.plugin_id.clone(),
                    ) {
                        error!("Failed to insert action mapping: {}", e);
                    }
                    all_new_results.push(local_action_item);
                }
                commands.entity(entity).despawn();
            },
            Some(Err(e)) => {
                any_task_completed_or_failed = true;
                error!(
                    "Search task for plugin {} failed: {}",
                    pending_search.plugin_id, e
                );
                commands.entity(entity).despawn();
            },
            None => {
                // Task is not yet complete
            },
        }
    }

    // Update CurrentSearchResults if any task finished (successfully or with error)
    // This ensures results are cleared if all tasks finish and yield no new items.
    if any_task_completed_or_failed {
        all_new_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        current_search_results.results = all_new_results;
        debug!(
            "Updated search results with {} items",
            current_search_results.results.len()
        );
    }
}

/// ECS-based action mapping resource to replace PluginRegistry.action_map
#[derive(Resource, Default)]
pub struct ActionMap {
    pub map: std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<String, String>>>,
}

impl ActionMap {
    pub fn new() -> Self {
        Self {
            map: std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn insert(&self, action_id: String, plugin_id: String) -> crate::error::Result<()> {
        let mut map = self.map.write();
        map.insert(action_id, plugin_id);
        Ok(())
    }

    pub fn get(&self, action_id: &str) -> Option<String> {
        let map = self.map.read();
        map.get(action_id).cloned()
    }

    pub fn clear(&self) {
        let mut map = self.map.write();
        map.clear();
    }

    pub fn remove(&self, action_id: &str) -> Option<String> {
        let mut map = self.map.write();
        map.remove(action_id)
    }

    pub fn contains_key(&self, action_id: &str) -> bool {
        let map = self.map.read();
        map.contains_key(action_id)
    }

    pub fn len(&self) -> usize {
        let map = self.map.read();
        map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// System to periodically clean up stale action mappings
pub fn cleanup_action_mappings_system(action_map: Res<ActionMap>, plugin_counter: PluginCounter) {
    // Only run cleanup occasionally
    static mut CLEANUP_COUNTER: u32 = 0;
    unsafe {
        CLEANUP_COUNTER += 1;
        if !CLEANUP_COUNTER.is_multiple_of(1000) {
            return;
        }
    }

    let plugin_count = plugin_counter.count();
    let mapping_count = action_map.len();

    debug!(
        "Action mapping cleanup: {} mappings for {} plugins",
        mapping_count, plugin_count
    );

    // If we have significantly more mappings than plugins, consider cleanup
    if mapping_count > plugin_count * 10 {
        warn!(
            "Large number of action mappings ({}) relative to plugins ({}). Consider cleanup.",
            mapping_count, plugin_count
        );
    }
}

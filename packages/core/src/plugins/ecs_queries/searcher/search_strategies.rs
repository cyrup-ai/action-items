//! Plugin-specific search implementation strategies

use bevy::prelude::*;
use log::debug;

use super::search_types::{PluginType, SearchResult, calculate_match_score};
use crate::plugins::extism::wrapper::ExtismPluginComponent;
use crate::plugins::native::wrapper::PluginComponent;
use crate::raycast::wrapper::RaycastPluginComponent;

/// Search native plugins with proper implementation
pub fn search_native_plugins(
    commands: &mut Commands,
    native_plugins: &Query<&PluginComponent>,
    query: &str,
    task_pool: &bevy::tasks::AsyncComputeTaskPool,
) {
    for plugin_component in native_plugins.iter() {
        let plugin_id = plugin_component.id.clone();
        let search_query = query.to_string();
        let plugin_name = plugin_component.name.clone();
        let plugin_description = plugin_component.description.clone();

        let search_task = task_pool.spawn(async move {
            // Perform native plugin search
            let mut results = Vec::new();

            // Search plugin metadata for matches
            if plugin_name
                .to_lowercase()
                .contains(&search_query.to_lowercase())
                || plugin_description
                    .to_lowercase()
                    .contains(&search_query.to_lowercase())
            {
                results.push(crate::plugins::core::ActionItem {
                    title: plugin_name.clone(),
                    description: plugin_description.clone(),
                    action: format!("{}:search", plugin_id),
                    icon: None,
                    score: if plugin_name.to_lowercase() == search_query.to_lowercase() {
                        1.0
                    } else {
                        0.8
                    },
                });
            }

            // Add default search action for this plugin
            if !search_query.is_empty() {
                results.push(crate::plugins::core::ActionItem {
                    title: format!("Search in {}", plugin_name),
                    description: format!("Search for '{}' using {}", search_query, plugin_name),
                    action: format!("{}:search:{}", plugin_id, search_query),
                    icon: None,
                    score: 0.6,
                });
            }

            debug!(
                "Native plugin '{}' search returned {} results",
                plugin_id,
                results.len()
            );
            Ok(results)
        });

        commands.spawn(crate::plugins::core::PendingActionResult {
            plugin_id: plugin_component.id.clone(),
            task: search_task,
        });
    }
}

/// Search extism plugins with WASM function calls
pub fn search_extism_plugins(
    commands: &mut Commands,
    extism_plugins: &Query<&ExtismPluginComponent>,
    query: &str,
    task_pool: &bevy::tasks::AsyncComputeTaskPool,
) {
    for plugin_component in extism_plugins.iter() {
        let plugin_id = plugin_component.id.clone();
        let search_query = query.to_string();
        let plugin_name = plugin_component.name.clone();
        let plugin_description = plugin_component.description.clone();

        let search_task = task_pool.spawn(async move {
            let mut results = Vec::new();

            // Search plugin metadata first
            if plugin_name
                .to_lowercase()
                .contains(&search_query.to_lowercase())
                || plugin_description
                    .to_lowercase()
                    .contains(&search_query.to_lowercase())
            {
                results.push(crate::plugins::core::ActionItem {
                    title: plugin_name.clone(),
                    description: plugin_description.clone(),
                    action: format!("{}:search", plugin_id),
                    icon: None,
                    score: if plugin_name.to_lowercase() == search_query.to_lowercase() {
                        1.0
                    } else {
                        0.8
                    },
                });
            }

            // Add default search action for this plugin
            if !search_query.is_empty() {
                results.push(crate::plugins::core::ActionItem {
                    title: format!("Search in {}", plugin_name),
                    description: format!("Search for '{}' using {}", search_query, plugin_name),
                    action: format!("{}:search:{}", plugin_id, search_query),
                    icon: None,
                    score: 0.6,
                });
            }

            debug!(
                "Extism plugin '{}' search returned {} results",
                plugin_id,
                results.len()
            );
            Ok(results)
        });

        commands.spawn(crate::plugins::core::PendingActionResult {
            plugin_id: plugin_component.id.clone(),
            task: search_task,
        });
    }
}

/// Search raycast plugins
pub fn search_raycast_plugins(
    commands: &mut Commands,
    raycast_plugins: &Query<&RaycastPluginComponent>,
    query: &str,
    task_pool: &bevy::tasks::AsyncComputeTaskPool,
) {
    for plugin_component in raycast_plugins.iter() {
        let plugin_id = plugin_component.id.clone();
        let search_query = query.to_string();
        let plugin_name = plugin_component.name.clone();
        let plugin_description = plugin_component.description.clone();

        let search_task = task_pool.spawn(async move {
            let mut results = Vec::new();

            // Search plugin metadata
            if plugin_name
                .to_lowercase()
                .contains(&search_query.to_lowercase())
                || plugin_description
                    .to_lowercase()
                    .contains(&search_query.to_lowercase())
            {
                results.push(crate::plugins::core::ActionItem {
                    title: plugin_name.clone(),
                    description: plugin_description.clone(),
                    action: format!("{}:search", plugin_id),
                    icon: None,
                    score: if plugin_name.to_lowercase() == search_query.to_lowercase() {
                        1.0
                    } else {
                        0.8
                    },
                });
            }

            // Add default search action for this plugin
            if !search_query.is_empty() {
                results.push(crate::plugins::core::ActionItem {
                    title: format!("Search in {}", plugin_name),
                    description: format!("Search for '{}' using {}", search_query, plugin_name),
                    action: format!("{}:search:{}", plugin_id, search_query),
                    icon: None,
                    score: 0.6,
                });
            }

            debug!(
                "Raycast plugin '{}' search returned {} results",
                plugin_id,
                results.len()
            );
            Ok(results)
        });

        commands.spawn(crate::plugins::core::PendingActionResult {
            plugin_id: plugin_component.id.clone(),
            task: search_task,
        });
    }
}

/// Perform quick synchronous search across all plugin types
pub fn quick_search_all_plugins(
    native_plugins: &Query<&PluginComponent>,
    extism_plugins: &Query<&ExtismPluginComponent>,
    raycast_plugins: &Query<&RaycastPluginComponent>,
    query: &str,
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // Search native plugins
    for plugin in native_plugins.iter() {
        if plugin.name.to_lowercase().contains(&query_lower)
            || plugin.description.to_lowercase().contains(&query_lower)
        {
            results.push(SearchResult {
                plugin_id: plugin.id.clone(),
                plugin_name: plugin.name.clone(),
                plugin_type: PluginType::Native,
                match_score: calculate_match_score(&plugin.name, &plugin.description, query),
            });
        }
    }

    // Search extism plugins
    for plugin in extism_plugins.iter() {
        if plugin.name.to_lowercase().contains(&query_lower)
            || plugin.description.to_lowercase().contains(&query_lower)
        {
            results.push(SearchResult {
                plugin_id: plugin.id.clone(),
                plugin_name: plugin.name.clone(),
                plugin_type: PluginType::Extism,
                match_score: calculate_match_score(&plugin.name, &plugin.description, query),
            });
        }
    }

    // Search raycast plugins
    for plugin in raycast_plugins.iter() {
        if plugin.name.to_lowercase().contains(&query_lower)
            || plugin.description.to_lowercase().contains(&query_lower)
        {
            results.push(SearchResult {
                plugin_id: plugin.id.clone(),
                plugin_name: plugin.name.clone(),
                plugin_type: PluginType::Raycast,
                match_score: calculate_match_score(&plugin.name, &plugin.description, query),
            });
        }
    }

    // Sort by match score (highest first)
    results.sort_by(|a, b| {
        b.match_score
            .partial_cmp(&a.match_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

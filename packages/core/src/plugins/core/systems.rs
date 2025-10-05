use action_items_common::plugin_interface::ActionType;
use bevy::prelude::*;
use log::{debug, error};

use super::components::PendingActionResult;
use super::metadata::ActionItem;
use super::resources::CurrentSearchResults;
use crate::events::ActionMap;

/// System to poll pending search tasks, process their results, and update the UI-facing resource.
pub fn handle_search_results_system(
    mut commands: Commands,
    mut pending_searches: Query<(Entity, &mut PendingActionResult)>,
    mut action_map: ResMut<ActionMap>,
    mut current_search_results: ResMut<CurrentSearchResults>,
) {
    let mut all_new_results: Vec<ActionItem> = Vec::new();
    let mut any_task_completed_or_failed = false;

    for (entity, mut pending_search) in pending_searches.iter_mut() {
        match bevy::tasks::block_on(bevy::tasks::futures_lite::future::poll_once(
            &mut pending_search.task,
        )) {
            Some(Ok(interface_results_vec)) => {
                any_task_completed_or_failed = true;
                for interface_result in interface_results_vec {
                    let local_action_item: ActionItem = interface_result;

                    // Convert to common ActionItem format for ActionMap
                    let common_action_item = action_items_common::plugin_interface::ActionItem {
                        id: local_action_item.action.clone(),
                        title: local_action_item.title.clone(),
                        subtitle: Some(local_action_item.description.clone()),
                        icon: local_action_item.icon.as_ref().map(|icon_str| {
                            action_items_common::plugin_interface::Icon::BuiltIn(icon_str.clone())
                        }),
                        actions: vec![action_items_common::plugin_interface::ItemAction {
                            id: local_action_item.action.clone(),
                            title: "Execute".to_string(),
                            icon: None,
                            shortcut: None,
                            action_type: ActionType::Custom(local_action_item.action.clone()),
                        }],
                        item_badges: vec![],
                        metadata: None,
                        score: local_action_item.score,
                        description: Some(local_action_item.description.clone()),
                        tags: vec![],
                        created_at: Some(chrono::Utc::now()),
                        updated_at: Some(chrono::Utc::now()),
                    };

                    // Add to ECS ActionMap resource for UI access
                    action_map.insert(local_action_item.action.clone(), common_action_item);

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
            "Search results updated. Total: {}",
            current_search_results.results.len()
        );
    }
}

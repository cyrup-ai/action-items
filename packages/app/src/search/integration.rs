//! Direct ECS search integration for Action Items launcher
//!
//! Simplified integration using ecs-search-aggregator directly without wrapper systems

use action_items_ecs_search_aggregator::events::{SearchCompleted, SearchResultReceived};
use action_items_ecs_search_aggregator::types::{AggregatedSearchResults, CurrentQuery};
use bevy::prelude::*;
use tracing::{debug, info};

use crate::input::TextInputChanged;

/// Direct text input to search query system - no debouncing wrapper needed
/// ECS search aggregator handles debouncing internally
#[inline]
pub fn handle_text_input_to_search(
    mut text_changed_events: EventReader<TextInputChanged>,
    mut current_query: ResMut<CurrentQuery>,
) {
    for event in text_changed_events.read() {
        // Direct update to CurrentQuery - SearchAggregatorPlugin handles debouncing
        current_query.0 = event.text.clone();

        debug!(
            "Updated search query: '{}' (cursor at position {})",
            event.text, event.cursor_position
        );
    }
}

/// Direct UI update system using ECS search aggregator results
#[inline]
pub fn update_search_ui(
    aggregated_results: Res<AggregatedSearchResults>,
    current_query: Res<CurrentQuery>,
    mut results_container_query: Query<
        &mut Visibility,
        With<action_items_ui::prelude::ResultsContainer>,
    >,
) {
    if aggregated_results.is_changed() || current_query.is_changed() {
        for mut visibility in results_container_query.iter_mut() {
            if current_query.0.trim().is_empty() || aggregated_results.results.is_empty() {
                *visibility = Visibility::Hidden;
            } else {
                *visibility = Visibility::Visible;
            }
        }
    }
}

/// Direct search event logging - UI updates handled by update_search_ui system
#[inline]
pub fn log_search_events(
    mut search_completed_events: EventReader<SearchCompleted>,
    mut search_result_events: EventReader<SearchResultReceived>,
) {
    // Log individual plugin results
    for result_event in search_result_events.read() {
        debug!(
            "Plugin '{}' returned {} results in {}ms for search {:?}",
            result_event.plugin_id,
            result_event.results.len(),
            result_event.execution_time_ms,
            result_event.search_id
        );
    }

    // Log completed searches
    for completion_event in search_completed_events.read() {
        info!(
            "Search {:?} completed: {} total results from {} plugins ({} failed) in {}ms",
            completion_event.search_id,
            completion_event.total_results,
            completion_event.responding_plugins.len(),
            completion_event.failed_plugins.len(),
            completion_event.execution_time_ms
        );
    }
}

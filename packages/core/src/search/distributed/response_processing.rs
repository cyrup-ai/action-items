use bevy::prelude::*;
use ecs_service_bridge::events::PluginMessageEvent;
use ecs_service_bridge::resources::ServiceBridgeResource;
use log::{debug, error, info, warn};

use super::message_handling::parse_search_response_results;
use super::types::DistributedSearchManager;
use crate::plugins::core::CurrentSearchResults;

// NOTE: Security limits are now managed by dedicated ecs-* packages

// NOTE: Validation functions are now handled by dedicated ecs-* packages
// Each service package (ecs-clipboard, ecs-notifications, etc.) handles its own validation

// NOTE: Error response handling is now managed by dedicated ecs-* packages

// NOTE: Service handling is now managed by dedicated ecs-* packages
// This function has been removed - services should send events to appropriate ECS packages:
// - ecs-clipboard for clipboard operations
// - ecs-notifications for notifications
// - ecs-http for HTTP requests (if exists)
// - ecs-storage for storage operations (if exists)

/// System to process distributed search responses
pub fn process_distributed_search_responses(
    _service_bridge: Res<ServiceBridgeResource>,
    mut search_manager: ResMut<DistributedSearchManager>,
    mut current_search_results: ResMut<CurrentSearchResults>,
    mut message_events: EventReader<PluginMessageEvent>,
) {
    // Process incoming messages from service bridge via events
    debug!("Processing distributed search responses with service bridge");

    // Process plugin message events for search responses
    for event in message_events.read() {
        if event.message_type == "search_response" {
            // Service messages are now handled by dedicated ecs-* packages
            // Handle search response
            if let Some(correlation_id) = &event.correlation_id {
                // Check if this is a search response we're waiting for
                if search_manager
                    .active_searches()
                    .contains_key(correlation_id)
                {
                    info!(
                        "Received search response from plugin for correlation: {}",
                        correlation_id
                    );

                    // Parse the search results from payload
                    if let Some(result_value) = event.payload.get("result") {
                        match parse_search_response_results(result_value) {
                            Ok(search_results) => {
                                // Directly merge results into current search results
                                current_search_results
                                    .results
                                    .extend(search_results.clone());

                                // Update the active search with received results
                                if let Some(active_search) =
                                    search_manager.active_searches_mut().get_mut(correlation_id)
                                {
                                    active_search.received_responses += 1;
                                    active_search.results.extend(search_results);
                                }

                                info!(
                                    "Successfully processed {} search results for correlation: {}",
                                    current_search_results.results.len(),
                                    correlation_id
                                );
                            },
                            Err(e) => {
                                error!(
                                    "Failed to parse search results for correlation {}: {}",
                                    correlation_id, e
                                );
                            },
                        }
                    }
                } else {
                    debug!(
                        "Received search response for unknown correlation: {}",
                        correlation_id
                    );
                }
            } else {
                warn!("Received search response without correlation ID");
            }
        } else {
            // Ignore non-response messages for search processing
            debug!("Ignoring non-response message in search processing");
        }
    }

    // Check for completed or timed out searches
    let timeout_ms = search_manager.search_timeout_ms();
    let mut timed_out_searches = Vec::new();

    for (correlation_id, query) in search_manager.active_searches().iter() {
        if query.is_timed_out(timeout_ms) {
            timed_out_searches.push(correlation_id.clone());
            warn!(
                "Search timed out for correlation: {} after {}ms",
                correlation_id, timeout_ms
            );
        } else if query.is_complete() {
            debug!(
                "Search completed for correlation: {} with {} responses",
                correlation_id, query.received_responses
            );
        } else {
            debug!(
                "Active search correlation: {} for query: '{}' ({}/{} responses)",
                correlation_id, query.query, query.received_responses, query.expected_responses
            );
        }
    }

    // Remove timed out searches from active searches
    for correlation_id in timed_out_searches {
        search_manager.active_searches_mut().remove(&correlation_id);
        info!("Removed timed out search: {}", correlation_id);
    }
}

/// System to handle search result aggregation and filtering
pub fn aggregate_search_results(
    _service_bridge: Res<ServiceBridgeResource>,
    mut current_search_results: ResMut<CurrentSearchResults>,
) {
    // This system can be used to perform post-processing on search results
    // such as deduplication, ranking, filtering, etc.

    debug!(
        "Aggregating {} search results with service bridge",
        current_search_results.results.len()
    );

    // Remove duplicates based on action (assuming action is unique identifier)
    current_search_results
        .results
        .dedup_by(|a, b| a.action == b.action);

    // Example: Remove duplicate results based on title or action
    let initial_count = current_search_results.results.len();
    current_search_results
        .results
        .dedup_by(|a, b| a.action == b.action || a.title == b.title);
    let final_count = current_search_results.results.len();

    if initial_count != final_count {
        info!(
            "Removed {} duplicate search results, {} remaining",
            initial_count - final_count,
            final_count
        );
    }

    // Example: Sort results by score (relevance)
    current_search_results.results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    debug!("Search result aggregation completed");
}

/// System to handle search response messages from plugins using async tasks
pub fn handle_search_response_messages(
    mut commands: Commands,
    search_manager: ResMut<DistributedSearchManager>,
    _current_search_results: ResMut<CurrentSearchResults>,
    mut message_events: EventReader<PluginMessageEvent>,
) {
    use bevy::tasks::AsyncComputeTaskPool;

    let thread_pool = AsyncComputeTaskPool::get();

    // Process plugin message events for search responses
    for event in message_events.read() {
        if event.message_type == "search_response"
            && let Some(correlation_id) = &event.correlation_id
                && search_manager
                    .active_searches()
                    .contains_key(correlation_id)
                {
                    // Spawn async task to process search response
                    let correlation_id = correlation_id.clone();
                    let event_payload = event.payload.clone();

                    let task = thread_pool.spawn(async move {
                        let mut command_queue = bevy::ecs::world::CommandQueue::default();

                        // Parse search results asynchronously
                        if let Some(result_value) = event_payload.get("result") {
                            match parse_search_response_results(result_value) {
                                Ok(search_results) => {
                                    command_queue.push(move |world: &mut World| {
                                        // Update search manager and results
                                        if let Some(mut search_manager) =
                                            world.get_resource_mut::<DistributedSearchManager>()
                                            && let Some(active_search) = search_manager
                                                .active_searches_mut()
                                                .get_mut(&correlation_id)
                                            {
                                                active_search.received_responses += 1;
                                                active_search
                                                    .results
                                                    .extend(search_results.clone());
                                            }

                                        // Update current search results
                                        if let Some(mut current_results) =
                                            world.get_resource_mut::<CurrentSearchResults>()
                                        {
                                            current_results.results.extend(search_results);
                                        }
                                    });
                                },
                                Err(e) => {
                                    error!(
                                        "Failed to parse search results for correlation {}: {}",
                                        correlation_id, e
                                    );
                                },
                            }
                        }

                        command_queue
                    });

                    // Spawn entity with async task component
                    commands.spawn(SearchResponseTask(task));
                }
    }
}

#[derive(Component)]
pub struct SearchResponseTask(bevy::tasks::Task<bevy::ecs::world::CommandQueue>);

/// System to handle completed search response tasks
pub fn handle_search_response_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SearchResponseTask)>,
) {
    use bevy::tasks::block_on;
    use bevy::tasks::futures_lite::future;

    for (entity, mut task) in &mut tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            // Apply the command queue to execute world updates
            commands.append(&mut commands_queue);
            // Remove the completed task entity
            commands.entity(entity).despawn();
        }
    }
}

// NOTE: Command execution is now managed by dedicated ecs-* packages
// This function has been removed - commands should send events to appropriate ECS packages

// NOTE: Broadcast message handling is now managed by dedicated ecs-* packages
// This function has been removed - broadcast events should be handled by appropriate ECS systems

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, block_on};
use ecs_service_bridge::PluginRegistryResource;
use ecs_service_bridge::events::MessagePriority;
use ecs_service_bridge::messaging::{MessageEnvelope, MessageInfrastructure};
use ecs_service_bridge::resources::PluginInfo;
// Full production service bridge integration
use ecs_service_bridge::systems::plugin_management::PluginCapabilityIndex;
use ecs_service_bridge::types::MessageAddress;
use tracing::{debug, info, warn};

use crate::components::*;
use crate::events::*;
use crate::manager::SearchAggregatorManager;
use crate::types::*;

/// Bevy plugin for search aggregation - coordinates searches across multiple plugins
pub struct SearchAggregatorPlugin;

impl Plugin for SearchAggregatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SearchAggregator>()
            .init_resource::<AggregatedSearchResults>()
            .init_resource::<SearchConfig>()
            .init_resource::<CurrentQuery>()
            .add_event::<SearchRequested>()
            .add_event::<SearchResultReceived>()
            .add_event::<SearchFailed>()
            .add_event::<SearchCompleted>()
            .add_event::<SearchCancelled>()
            .add_systems(
                Update,
                (
                    query_change_detection_system,
                    spawn_plugin_search_tasks_system,
                    handle_plugin_search_tasks_system,
                    aggregate_search_results_system,
                    search_timeout_system,
                    search_cancellation_system,
                    search_cleanup_system,
                )
                    .chain(),
            );
    }
}
/// System to detect query changes and initiate new searches
fn query_change_detection_system(
    current_query: Res<CurrentQuery>,
    mut search_events: EventWriter<SearchRequested>,
    mut cancel_events: EventWriter<SearchCancelled>,
    mut search_aggregator: ResMut<SearchAggregator>,
    mut aggregated_results: ResMut<AggregatedSearchResults>,
    search_config: Res<SearchConfig>,
    capability_index: Res<PluginCapabilityIndex>,
) {
    // Only trigger on actual query changes
    if !current_query.is_changed() {
        return;
    }

    let query = current_query.0.trim().to_string();

    // Clear results immediately when query changes
    aggregated_results.clear();

    // Validate query
    if let Err(e) = SearchAggregatorManager::validate_query(&query, &search_config) {
        debug!("Invalid search query '{}': {}", query, e);
        return;
    }

    // Cancel any active searches with proper event emission
    for (search_id, active_search) in search_aggregator.active_searches.iter() {
        let cancellation_event = SearchCancelled {
            search_id: *search_id,
            reason: format!(
                "Query changed from '{}' to '{}'",
                active_search.query, query
            ),
        };
        cancel_events.write(cancellation_event);
        debug!("Sent cancellation event for search: {:?}", search_id);
    }
    search_aggregator.active_searches.clear();

    // Find plugins with search capability via service bridge integration
    let search_capable_plugins: Vec<String> = discover_search_capable_plugins(&capability_index);

    if search_capable_plugins.is_empty() {
        debug!("No search-capable plugins found");
        return;
    }

    // Create and send search request event
    let search_request = SearchRequested::new(query.clone(), search_capable_plugins.clone());
    let search_id = search_request.search_id;

    // Track this search
    let expected_plugins: HashSet<String> = search_capable_plugins.into_iter().collect();
    let active_search = ActiveSearch::new(query, search_id, expected_plugins);
    search_aggregator
        .active_searches
        .insert(search_id, active_search);

    // Start loading state
    aggregated_results.start_search(search_id);

    info!(
        "Starting search '{}' with ID {:?}",
        current_query.0, search_id
    );
    search_events.write(search_request);
}
/// System to spawn async search tasks for each capable plugin
fn spawn_plugin_search_tasks_system(
    mut commands: Commands,
    mut search_events: EventReader<SearchRequested>,
    search_config: Res<SearchConfig>,
    plugin_registry: Res<PluginRegistryResource>,
    message_infrastructure: Res<MessageInfrastructure>,
) {
    for search_event in search_events.read() {
        let search_id = search_event.search_id;
        let query = search_event.query.clone();

        // Log message infrastructure stats (this uses the parameter)
        debug!(
            "Processing search {} with message infrastructure config: {:?}",
            search_id, message_infrastructure.config
        );

        // Create timeout tracking entity
        let timeout_duration = std::time::Duration::from_millis(search_config.timeout_ms);
        commands.spawn(SearchTimeout::new(search_id, timeout_duration));

        // Spawn search task for each plugin
        for plugin_id in &search_event.requesting_plugins {
            let plugin_id_owned = plugin_id.clone();
            let query_owned = query.clone();
            let _max_results = search_config.max_results_per_plugin;
            let plugin_registry_data = plugin_registry.plugins.clone();

            // Create async task for real plugin search using the execute_plugin_search function
            let task = AsyncComputeTaskPool::get().spawn(async move {
                // Execute actual plugin search via service bridge
                info!(
                    "Searching plugin '{}' for query '{}'",
                    plugin_id_owned, query_owned
                );

                // Use the plugin registry data to execute the search
                match execute_plugin_search(
                    &plugin_id_owned,
                    &query_owned,
                    _max_results,
                    &plugin_registry_data,
                    &mut MessageInfrastructure::default(), /* Create a local instance for async
                                                            * context */
                )
                .await
                {
                    Ok(results) => {
                        info!(
                            "Plugin '{}' returned {} results",
                            plugin_id_owned,
                            results.len()
                        );
                        Ok::<Vec<SearchResult>, SearchError>(results)
                    },
                    Err(e) => {
                        error!("Plugin search failed for '{}': {}", plugin_id_owned, e);
                        Err(SearchError::PluginSearchFailed(
                            plugin_id_owned,
                            e.to_string(),
                        ))
                    },
                }
            });

            // Spawn entity with search task component
            let plugin_search_task = PluginSearchTask::new(search_id, plugin_id.clone(), task);
            commands.spawn(plugin_search_task);

            debug!(
                "Spawned search task for plugin '{}' in search {:?}",
                plugin_id, search_id
            );
        }

        info!(
            "Spawned {} search tasks for search {:?}",
            search_event.requesting_plugins.len(),
            search_id
        );
    }
}
/// System to poll completed plugin search tasks - following async_compute.rs pattern exactly
fn handle_plugin_search_tasks_system(
    mut commands: Commands,
    mut search_tasks: Query<(Entity, &mut PluginSearchTask)>,
    mut result_events: EventWriter<SearchResultReceived>,
    mut failure_events: EventWriter<SearchFailed>,
) {
    for (entity, mut task) in &mut search_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.task)) {
            let execution_time = task.execution_time_ms();

            match result {
                Ok(results) => {
                    debug!(
                        "Plugin '{}' returned {} results in {}ms",
                        task.plugin_id,
                        results.len(),
                        execution_time
                    );

                    result_events.write(SearchResultReceived {
                        search_id: task.search_id,
                        plugin_id: task.plugin_id.clone(),
                        results,
                        execution_time_ms: execution_time,
                    });
                },
                Err(error) => {
                    warn!(
                        "Plugin '{}' search failed after {}ms: {:?}",
                        task.plugin_id, execution_time, error
                    );

                    failure_events.write(SearchFailed {
                        search_id: task.search_id,
                        plugin_id: task.plugin_id.clone(),
                        error,
                    });
                },
            }

            // Task is complete, remove the entity
            commands.entity(entity).despawn();
        }
    }
}
/// System to aggregate results from multiple plugins
fn aggregate_search_results_system(
    mut result_events: EventReader<SearchResultReceived>,
    mut failure_events: EventReader<SearchFailed>,
    mut search_aggregator: ResMut<SearchAggregator>,
    mut aggregated_results: ResMut<AggregatedSearchResults>,
    mut completion_events: EventWriter<SearchCompleted>,
    search_config: Res<SearchConfig>,
) {
    // Handle successful results
    for result_event in result_events.read() {
        let search_id = result_event.search_id;

        if let Some(active_search) = search_aggregator.active_searches.get_mut(&search_id) {
            // Add results to the active search
            active_search.add_results(result_event.plugin_id.clone(), result_event.results.clone());

            // Update aggregated results if this is the current search
            if aggregated_results.search_id == Some(search_id) {
                aggregated_results
                    .completed_plugins
                    .insert(result_event.plugin_id.clone());

                // Re-process all results with scoring and deduplication
                let all_plugin_results: Vec<(String, Vec<SearchResult>)> = active_search
                    .expected_plugins
                    .iter()
                    .filter_map(|pid| {
                        if active_search.completed_plugins.contains(pid) {
                            let plugin_results: Vec<SearchResult> = active_search
                                .results
                                .iter()
                                .filter(|r| r.plugin_id == *pid)
                                .cloned()
                                .collect();
                            Some((pid.clone(), plugin_results))
                        } else {
                            None
                        }
                    })
                    .collect();

                aggregated_results.results = SearchAggregatorManager::merge_plugin_results(
                    all_plugin_results,
                    &active_search.query,
                    &search_config,
                );
            }

            // Check if search is complete
            if active_search.is_complete() {
                let execution_time = active_search.started_at.elapsed().as_millis() as u64;

                completion_events.write(SearchCompleted {
                    search_id,
                    total_results: active_search.results.len(),
                    responding_plugins: active_search.completed_plugins.iter().cloned().collect(),
                    failed_plugins: active_search.failed_plugins.keys().cloned().collect(),
                    execution_time_ms: execution_time,
                });
            }
        }
    }

    // Handle failures
    for failure_event in failure_events.read() {
        let search_id = failure_event.search_id;

        if let Some(active_search) = search_aggregator.active_searches.get_mut(&search_id) {
            active_search.mark_failed(
                failure_event.plugin_id.clone(),
                format!("{:?}", failure_event.error),
            );

            // Update aggregated results if this is the current search
            if aggregated_results.search_id == Some(search_id) {
                aggregated_results.failed_plugins.push((
                    failure_event.plugin_id.clone(),
                    format!("{:?}", failure_event.error),
                ));
            }

            // Check if search is complete
            if active_search.is_complete() {
                let execution_time = active_search.started_at.elapsed().as_millis() as u64;

                completion_events.write(SearchCompleted {
                    search_id,
                    total_results: active_search.results.len(),
                    responding_plugins: active_search.completed_plugins.iter().cloned().collect(),
                    failed_plugins: active_search.failed_plugins.keys().cloned().collect(),
                    execution_time_ms: execution_time,
                });
            }
        }
    }
}
/// System to handle search timeouts
fn search_timeout_system(
    mut commands: Commands,
    timeout_query: Query<(Entity, &SearchTimeout)>,
    mut failure_events: EventWriter<SearchFailed>,
    search_aggregator: Res<SearchAggregator>,
) {
    for (entity, timeout) in timeout_query.iter() {
        if timeout.is_expired() {
            // Find which plugins haven't responded yet for this search
            if let Some(active_search) = search_aggregator.active_searches.get(&timeout.search_id) {
                let pending_plugins: Vec<String> = active_search
                    .expected_plugins
                    .iter()
                    .filter(|&plugin_id| {
                        !active_search.completed_plugins.contains(plugin_id)
                            && !active_search.failed_plugins.contains_key(plugin_id)
                    })
                    .cloned()
                    .collect();

                // Send timeout failure events for all pending plugins
                for plugin_id in pending_plugins {
                    failure_events.write(SearchFailed {
                        search_id: timeout.search_id,
                        plugin_id,
                        error: SearchError::Timeout,
                    });
                }
            }

            // Remove the timeout entity
            commands.entity(entity).despawn();
            debug!("Search {:?} timed out", timeout.search_id);
        }
    }
}

/// System to handle search cancellation
fn search_cancellation_system(
    mut cancel_events: EventReader<SearchCancelled>,
    mut commands: Commands,
    active_tasks: Query<(Entity, &PluginSearchTask)>,
    timeout_entities: Query<(Entity, &SearchTimeout)>,
) {
    for cancel_event in cancel_events.read() {
        let search_id = cancel_event.search_id;

        // Cancel all active tasks for this search
        for (entity, task) in active_tasks.iter() {
            if task.search_id == search_id {
                commands.entity(entity).despawn();
            }
        }

        // Cancel timeout entities for this search
        for (entity, timeout) in timeout_entities.iter() {
            if timeout.search_id == search_id {
                commands.entity(entity).despawn();
            }
        }

        info!("Cancelled search {:?}: {}", search_id, cancel_event.reason);
    }
}
/// System to cleanup completed searches
fn search_cleanup_system(
    mut completion_events: EventReader<SearchCompleted>,
    mut search_aggregator: ResMut<SearchAggregator>,
    mut aggregated_results: ResMut<AggregatedSearchResults>,
) {
    for completion_event in completion_events.read() {
        let search_id = completion_event.search_id;

        info!(
            "Search {:?} completed: {} results from {} plugins ({} failed) in {}ms",
            search_id,
            completion_event.total_results,
            completion_event.responding_plugins.len(),
            completion_event.failed_plugins.len(),
            completion_event.execution_time_ms
        );

        // Update aggregated results if this is the current search
        if aggregated_results.search_id == Some(search_id) {
            aggregated_results.finish_search(completion_event.execution_time_ms);
        }

        // Clean up the active search after a delay to allow for any final processing
        // In a real implementation, you might want to keep recent searches for caching
        search_aggregator.active_searches.remove(&search_id);

        debug!("Cleaned up search {:?}", search_id);
    }
}

/// Discover search-capable plugins from the service bridge registry
fn discover_search_capable_plugins(capability_index: &PluginCapabilityIndex) -> Vec<String> {
    let search_plugins = capability_index
        .capability_to_plugins
        .get("search")
        .map(|plugin_vec| plugin_vec.to_vec())
        .unwrap_or_default();

    if search_plugins.is_empty() {
        debug!("No healthy search-capable plugins found");
    } else {
        debug!(
            "Found {} healthy search-capable plugins: {:?}",
            search_plugins.len(),
            search_plugins
        );
    }

    search_plugins
}

/// Execute search request on specific plugin via service bridge
async fn execute_plugin_search(
    plugin_id: &str,
    query: &str,
    max_results: usize,
    plugin_registry: &HashMap<String, PluginInfo>,
    message_infrastructure: &mut MessageInfrastructure,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>> {
    // Validate plugin exists and is active
    let plugin_info = plugin_registry
        .get(plugin_id)
        .ok_or_else(|| format!("Plugin {} not found in registry", plugin_id))?;

    if !matches!(
        plugin_info.status,
        ecs_service_bridge::components::PluginStatus::Active
    ) {
        return Err(format!("Plugin {} is not active", plugin_id).into());
    }

    // Create structured search message using Service Bridge messaging
    let search_payload = serde_json::json!({
        "query": query,
        "max_results": max_results,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    });

    // Create message envelope for Service Bridge
    let from_address = MessageAddress::new("search_aggregator")?;
    let to_address = MessageAddress::new(plugin_id)?;
    let envelope = MessageEnvelope::new(
        from_address,
        to_address,
        "search_request".to_string(),
        search_payload,
        MessagePriority::High,
    )?;

    info!("Sending search request to plugin {}", plugin_id);

    // Send message via Service Bridge infrastructure
    let timeout = std::time::Duration::from_millis(5000);
    match send_message_with_timeout(message_infrastructure, envelope, timeout).await {
        Ok(response_envelope) => {
            // Parse response payload for search results
            match parse_search_response(&response_envelope.payload.content, plugin_id, max_results)
            {
                Ok(results) => {
                    info!("Plugin '{}' returned {} results", plugin_id, results.len());
                    Ok(results)
                },
                Err(e) => {
                    warn!(
                        "Failed to parse search response from plugin '{}': {}",
                        plugin_id, e
                    );
                    // Fall back to empty results rather than failing completely
                    Ok(vec![])
                },
            }
        },
        Err(e) => {
            warn!("Search request to plugin '{}' failed: {}", plugin_id, e);
            // Fall back to empty results for graceful degradation
            Ok(vec![])
        },
    }
}

/// Send message with timeout using Service Bridge infrastructure
async fn send_message_with_timeout(
    message_infrastructure: &mut MessageInfrastructure,
    envelope: MessageEnvelope,
    timeout: std::time::Duration,
) -> Result<MessageEnvelope, Box<dyn std::error::Error + Send + Sync>> {
    // Send via priority queue system
    message_infrastructure
        .priority_queues
        .send(envelope.clone())?;

    // Poll for response with timeout (simplified implementation)
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < timeout {
        if let Some(response) = message_infrastructure.priority_queues.try_recv() {
            // Check if this is a response to our message
            if response.metadata.correlation_id == envelope.metadata.correlation_id
                && response.metadata.message_type == "search_response"
            {
                return Ok(response);
            }
        }

        // Small delay to avoid busy waiting
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    Err("Message timeout".into())
}

/// Parse search response from plugin message payload
fn parse_search_response(
    payload: &serde_json::Value,
    plugin_id: &str,
    max_results: usize,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error + Send + Sync>> {
    let results_array = payload
        .get("results")
        .and_then(|v| v.as_array())
        .ok_or("Missing or invalid 'results' array in response")?;

    let mut search_results = Vec::new();

    for (i, result_value) in results_array.iter().enumerate() {
        if i >= max_results {
            break; // Respect max_results limit
        }

        let title = result_value
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'title' field in search result")?
            .to_string();

        let description = result_value
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let action = result_value
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'action' field in search result")?
            .to_string();

        let icon = result_value
            .get("icon")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let score = result_value
            .get("score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        search_results.push(SearchResult {
            title,
            description,
            action,
            icon,
            score,
            plugin_id: plugin_id.to_string(),
        });
    }

    Ok(search_results)
}

// get_plugins_with_capability function removed - replaced by PluginCapabilityIndex integration

// send_search_request_to_plugin function removed - functionality integrated into
// execute_plugin_search

/// Plugin search request structure for service bridge communication
#[derive(Debug, Clone)]
pub struct PluginSearchRequest {
    pub plugin_id: String,
    pub query: String,
    pub max_results: usize,
    pub timeout_ms: u64,
}

// PluginInfo already imported at top of file

/// Plugin search response structure
#[derive(Debug, Clone)]
pub struct PluginSearchResponse {
    pub results: Vec<PluginSearchResult>,
}

/// Individual plugin search result
#[derive(Debug, Clone)]
pub struct PluginSearchResult {
    pub title: String,
    pub description: String,
    pub action: String,
    pub icon: Option<String>,
    pub relevance_score: f32,
}

use std::collections::HashMap;

use bevy::prelude::*;
use ecs_service_bridge::events::{MessagePriority, PluginMessageEvent};
use ecs_service_bridge::resources::{PluginRegistryResource, ServiceBridgeResource};
use log::{debug, error, info};
use serde_json::Value;

use super::types::{DistributedSearchManager, DistributedSearchQuery};
use crate::plugins::core::CurrentSearchResults;

/// System to perform distributed search using the service bridge
pub fn distributed_search_system(
    _service_bridge: Res<ServiceBridgeResource>,
    plugin_registry: Res<PluginRegistryResource>,
    mut search_manager: ResMut<DistributedSearchManager>,
    mut current_search_results: ResMut<CurrentSearchResults>,
    mut message_events: EventWriter<PluginMessageEvent>,
    current_query: Res<crate::CurrentQuery>,
) {
    if current_query.is_changed() && !current_query.0.is_empty() {
        let query = &current_query.0;
        info!("Starting distributed search for query: '{}'", query);

        // Clear previous search results when starting new search
        current_search_results.results.clear();

        // Find plugins with search capability from plugin registry
        let search_capable_plugins: Vec<String> = plugin_registry
            .capabilities
            .get("search")
            .cloned()
            .unwrap_or_default();

        if search_capable_plugins.is_empty() {
            debug!("No search-capable plugins found for distributed search");
            return;
        }

        // Generate correlation ID for this search
        let correlation_id = uuid::Uuid::new_v4().to_string();

        // Create distributed search query
        let distributed_query = DistributedSearchQuery::new(
            query.clone(),
            correlation_id.clone(),
            search_capable_plugins.len(),
        );

        // Send search requests to all search-capable plugins
        let mut successful_requests = 0;
        for plugin_id in &search_capable_plugins {
            let search_message = create_search_message(query, &correlation_id);

            let mut params = std::collections::HashMap::new();
            params.insert(
                "query".to_string(),
                serde_json::to_value(&search_message).unwrap_or_default(),
            );

            // Serialize params with error handling
            let payload = match serde_json::to_value(params) {
                Ok(value) => value,
                Err(e) => {
                    error!("Failed to serialize search params for plugin '{}': {}", plugin_id, e);
                    continue; // Skip this plugin and continue with others
                }
            };

            // Send search message as PluginMessageEvent
            let message_event = PluginMessageEvent {
                from: "search_orchestrator".to_string(),
                to: plugin_id.clone(),
                plugin_id: plugin_id.clone(),
                message_type: "search".to_string(),
                payload,
                priority: MessagePriority::Normal,
                timestamp: ecs_service_bridge::types::TimeStamp::now(),
                request_id: Some(correlation_id.clone()),
                correlation_id: Some(correlation_id.clone()),
            };

            // Send the message event - EventWriter operations are infallible
            message_events.write(message_event);
            successful_requests += 1;
            debug!("Sent search request to plugin '{}'", plugin_id);
        }

        if successful_requests > 0 {
            // Update expected responses to match successful requests
            let mut updated_query = distributed_query;
            updated_query.expected_responses = successful_requests;

            search_manager
                .active_searches_mut()
                .insert(correlation_id, updated_query);
            info!(
                "Distributed search initiated with {} plugins",
                successful_requests
            );
        } else {
            error!("Failed to send search requests to any plugins");
        }
    }
}

/// Create a search message to send to plugins
fn create_search_message(query: &str, _correlation_id: &str) -> HashMap<String, Value> {
    let mut params = HashMap::new();
    params.insert("query".to_string(), Value::String(query.to_string()));
    params.insert(
        "max_results".to_string(),
        Value::Number(serde_json::Number::from(10)),
    );
    params.insert(
        "timeout_ms".to_string(),
        Value::Number(serde_json::Number::from(3000)),
    );
    params
}

use std::collections::HashMap;

use bevy::prelude::*;

use crate::plugins::core::ActionItem;

/// Resource for managing distributed search across plugins via service bridge
#[derive(Resource)]
pub struct DistributedSearchManager {
    /// Active search queries and their correlation IDs
    pub(super) active_searches: HashMap<String, DistributedSearchQuery>,
    /// Search timeout in milliseconds
    pub(super) search_timeout_ms: u64,
}

impl Default for DistributedSearchManager {
    fn default() -> Self {
        Self {
            active_searches: HashMap::new(),
            search_timeout_ms: 5000, // 5 second timeout
        }
    }
}

impl DistributedSearchManager {
    /// Get a reference to active searches
    pub fn active_searches(&self) -> &HashMap<String, DistributedSearchQuery> {
        &self.active_searches
    }

    /// Get a mutable reference to active searches
    pub fn active_searches_mut(&mut self) -> &mut HashMap<String, DistributedSearchQuery> {
        &mut self.active_searches
    }

    /// Get the search timeout in milliseconds
    pub fn search_timeout_ms(&self) -> u64 {
        self.search_timeout_ms
    }
}

/// Represents an active distributed search query
#[derive(Debug, Clone)]
pub struct DistributedSearchQuery {
    pub query: String,
    pub correlation_id: String,
    pub started_at: std::time::Instant,
    pub expected_responses: usize,
    pub received_responses: usize,
    pub results: Vec<ActionItem>,
    pub responding_plugins: Vec<String>,
}

impl DistributedSearchQuery {
    /// Create a new distributed search query
    pub fn new(query: String, correlation_id: String, expected_responses: usize) -> Self {
        Self {
            query,
            correlation_id,
            started_at: std::time::Instant::now(),
            expected_responses,
            received_responses: 0,
            results: Vec::new(),
            responding_plugins: Vec::new(),
        }
    }

    /// Check if the search query has received all expected responses
    pub fn is_complete(&self) -> bool {
        self.received_responses >= self.expected_responses
    }

    /// Check if the search query has timed out
    pub fn is_timed_out(&self, timeout_ms: u64) -> bool {
        self.started_at.elapsed().as_millis() > timeout_ms as u128
    }

    /// Add a response from a plugin to this search query
    pub fn add_response(&mut self, plugin_id: String, results: Vec<ActionItem>) {
        self.received_responses += 1;
        self.responding_plugins.push(plugin_id);
        self.results.extend(results);
    }
}

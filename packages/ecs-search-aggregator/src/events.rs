use bevy::prelude::*;

use crate::types::{SearchError, SearchId, SearchResult};

/// Event to initiate search across all capable plugins
#[derive(Event, Debug, Clone)]
pub struct SearchRequested {
    pub query: String,
    pub search_id: SearchId,
    pub requesting_plugins: Vec<String>,
}

impl SearchRequested {
    pub fn new(query: String, requesting_plugins: Vec<String>) -> Self {
        Self {
            query,
            search_id: SearchId::new_v4(),
            requesting_plugins,
        }
    }
}

/// Event when a plugin returns search results
#[derive(Event, Debug, Clone)]
pub struct SearchResultReceived {
    pub search_id: SearchId,
    pub plugin_id: String,
    pub results: Vec<SearchResult>,
    pub execution_time_ms: u64,
}

/// Event when a plugin search fails or times out
#[derive(Event, Debug, Clone)]
pub struct SearchFailed {
    pub search_id: SearchId,
    pub plugin_id: String,
    pub error: SearchError,
}

/// Event when all search results are aggregated and ready
#[derive(Event, Debug, Clone)]
pub struct SearchCompleted {
    pub search_id: SearchId,
    pub total_results: usize,
    pub responding_plugins: Vec<String>,
    pub failed_plugins: Vec<String>,
    pub execution_time_ms: u64,
}

/// Event to cancel active searches
#[derive(Event, Debug, Clone)]
pub struct SearchCancelled {
    pub search_id: SearchId,
    pub reason: String,
}

impl SearchCancelled {
    pub fn new_query(search_id: SearchId) -> Self {
        Self {
            search_id,
            reason: "New query started".to_string(),
        }
    }

    pub fn timeout(search_id: SearchId) -> Self {
        Self {
            search_id,
            reason: "Search timeout".to_string(),
        }
    }
}

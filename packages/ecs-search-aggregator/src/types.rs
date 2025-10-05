use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Unique identifier for search sessions
pub type SearchId = uuid::Uuid;

/// Search result from a plugin - matches ActionItem from core
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub title: String,
    pub description: String,
    pub action: String,
    pub icon: Option<String>,
    pub score: f32,
    pub plugin_id: String,
}

/// Main resource for managing search aggregation
#[derive(Resource)]
pub struct SearchAggregator {
    pub active_searches: HashMap<SearchId, ActiveSearch>,
    pub search_timeout: Duration,
    pub max_concurrent_searches: usize,
}

impl Default for SearchAggregator {
    fn default() -> Self {
        Self {
            active_searches: HashMap::new(),
            search_timeout: Duration::from_secs(5),
            max_concurrent_searches: 10,
        }
    }
}

/// Information about an active search session
#[derive(Debug, Clone)]
pub struct ActiveSearch {
    pub query: String,
    pub search_id: SearchId,
    pub started_at: Instant,
    pub expected_plugins: HashSet<String>,
    pub completed_plugins: HashSet<String>,
    pub failed_plugins: HashMap<String, String>, // plugin_id -> error_message
    pub results: Vec<SearchResult>,
}

impl ActiveSearch {
    pub fn new(query: String, search_id: SearchId, expected_plugins: HashSet<String>) -> Self {
        Self {
            query,
            search_id,
            started_at: Instant::now(),
            expected_plugins,
            completed_plugins: HashSet::new(),
            failed_plugins: HashMap::new(),
            results: Vec::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.completed_plugins.len() + self.failed_plugins.len() >= self.expected_plugins.len()
    }

    pub fn add_results(&mut self, plugin_id: String, results: Vec<SearchResult>) {
        self.completed_plugins.insert(plugin_id);
        self.results.extend(results);
    }

    pub fn mark_failed(&mut self, plugin_id: String, error: String) {
        self.failed_plugins.insert(plugin_id, error);
    }
}
/// Resource holding aggregated search results for UI display
#[derive(Resource, Default)]
pub struct AggregatedSearchResults {
    pub results: Vec<SearchResult>,
    pub search_id: Option<SearchId>,
    pub is_loading: bool,
    pub completed_plugins: HashSet<String>,
    pub failed_plugins: Vec<(String, String)>, // plugin_id, error
    pub total_execution_time_ms: u64,
}

impl AggregatedSearchResults {
    pub fn clear(&mut self) {
        self.results.clear();
        self.search_id = None;
        self.is_loading = false;
        self.completed_plugins.clear();
        self.failed_plugins.clear();
        self.total_execution_time_ms = 0;
    }

    pub fn start_search(&mut self, search_id: SearchId) {
        self.clear();
        self.search_id = Some(search_id);
        self.is_loading = true;
    }

    pub fn finish_search(&mut self, execution_time_ms: u64) {
        self.is_loading = false;
        self.total_execution_time_ms = execution_time_ms;
        // Sort results by score descending
        self.results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

/// Configuration resource for search behavior
#[derive(Resource)]
pub struct SearchConfig {
    pub timeout_ms: u64,
    pub max_results_per_plugin: usize,
    pub debounce_delay_ms: u64,
    pub min_query_length: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            max_results_per_plugin: 20,
            debounce_delay_ms: 150,
            min_query_length: 1,
        }
    }
}

/// Search error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchError {
    Timeout,
    PluginNotResponding,
    InvalidQuery,
    ServiceUnavailable,
    InternalError(String),
    PluginSearchFailed(String, String), // plugin_id, error_message
}

/// Resource to track the current search query
#[derive(Resource, Debug, Clone, Default)]
pub struct CurrentQuery(pub String);

impl CurrentQuery {
    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

/// Search session data
#[derive(Debug, Clone)]
pub struct SearchSession {
    pub query: String,
    pub started_at: Instant,
    pub requesting_plugins: Vec<String>,
    pub responding_plugins: HashSet<String>,
    pub failed_plugins: HashSet<String>,
    pub results: Vec<SearchResult>,
}

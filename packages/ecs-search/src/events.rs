//! Search events for request/response pattern
//!
//! Follows the standard ECS event-driven architecture pattern.

use bevy::prelude::*;
use crate::components::SearchResult;

/// Request a search operation
#[derive(Event, Debug, Clone)]
pub struct SearchRequested {
    /// The search query string
    pub query: String,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Identifier of the requester (for tracking)
    pub requester: String,
}

impl SearchRequested {
    /// Create a new search request
    pub fn new(query: impl Into<String>, max_results: usize, requester: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            max_results,
            requester: requester.into(),
        }
    }
}

/// Search operation completed
#[derive(Event, Debug, Clone)]
pub struct SearchCompleted {
    /// The search query that was executed
    pub query: String,
    /// The search results
    pub results: Vec<SearchResult>,
    /// Time taken for the search in milliseconds
    pub duration_ms: u64,
    /// Requester identifier
    pub requester: String,
}

impl SearchCompleted {
    /// Create a new search completed event
    pub fn new(query: String, results: Vec<SearchResult>, duration_ms: u64, requester: String) -> Self {
        Self {
            query,
            results,
            duration_ms,
            requester,
        }
    }
}

/// Search query changed (for live search)
#[derive(Event, Debug, Clone)]
pub struct SearchQueryChanged {
    /// The new search query
    pub query: String,
    /// Identifier of the requester
    pub requester: String,
}

impl SearchQueryChanged {
    /// Create a new query changed event
    pub fn new(query: impl Into<String>, requester: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            requester: requester.into(),
        }
    }
}

/// Search error occurred
#[derive(Event, Debug, Clone)]
pub struct SearchError {
    /// The search query that caused the error
    pub query: String,
    /// Error message
    pub error: String,
    /// Requester identifier
    pub requester: String,
}

impl SearchError {
    /// Create a new search error event
    pub fn new(query: String, error: String, requester: String) -> Self {
        Self {
            query,
            error,
            requester,
        }
    }
}

/// Search cache cleared event
#[derive(Event, Debug, Clone)]
pub struct SearchCacheCleared {
    /// Number of cached entries cleared
    pub entries_cleared: usize,
}

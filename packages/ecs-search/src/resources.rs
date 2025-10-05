//! Search resource for global search state
//!
//! Manages search configuration and caching.

use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::SearchResult;
use crate::systems::filtering::FilterState;

/// Global search resource
#[derive(Resource)]
pub struct SearchResource {
    /// Current search query
    pub current_query: String,
    /// Cached search results
    pub cached_results: HashMap<String, CachedSearchResults>,
    /// Search configuration
    pub config: SearchConfig,
    /// Filter state
    pub filter_state: FilterState,
}

impl Default for SearchResource {
    fn default() -> Self {
        Self {
            current_query: String::new(),
            cached_results: HashMap::new(),
            config: SearchConfig::default(),
            filter_state: FilterState::new(),
        }
    }
}

impl SearchResource {
    /// Create a new search resource
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom config
    pub fn with_config(config: SearchConfig) -> Self {
        Self {
            current_query: String::new(),
            cached_results: HashMap::new(),
            config,
            filter_state: FilterState::new(),
        }
    }

    /// Get cached results for a query if available
    pub fn get_cached(&self, query: &str) -> Option<&Vec<SearchResult>> {
        self.cached_results.get(query).and_then(|cached| {
            if cached.is_expired(self.config.cache_ttl_seconds) {
                None
            } else {
                Some(&cached.results)
            }
        })
    }

    /// Cache search results for a query
    pub fn cache_results(&mut self, query: String, results: Vec<SearchResult>) {
        self.cached_results.insert(
            query,
            CachedSearchResults {
                results,
                cached_at: std::time::Instant::now(),
            },
        );

        // Limit cache size
        if self.cached_results.len() > self.config.max_cache_size {
            // Remove oldest entry
            if let Some(oldest_key) = self.find_oldest_cache_key() {
                self.cached_results.remove(&oldest_key);
            }
        }
    }

    /// Clear all cached results
    pub fn clear_cache(&mut self) -> usize {
        let count = self.cached_results.len();
        self.cached_results.clear();
        count
    }

    /// Find the oldest cache entry key
    fn find_oldest_cache_key(&self) -> Option<String> {
        self.cached_results
            .iter()
            .min_by_key(|(_, cached)| cached.cached_at)
            .map(|(key, _)| key.clone())
    }
}

/// Cached search results with timestamp
#[derive(Debug, Clone)]
pub struct CachedSearchResults {
    /// The cached results
    pub results: Vec<SearchResult>,
    /// When these results were cached
    pub cached_at: std::time::Instant,
}

impl CachedSearchResults {
    /// Check if this cache entry is expired
    pub fn is_expired(&self, ttl_seconds: u64) -> bool {
        self.cached_at.elapsed().as_secs() > ttl_seconds
    }
}

/// Search configuration
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Maximum number of results to return
    pub max_results: usize,
    /// Minimum score threshold (0.0 to 1.0)
    pub score_threshold: f32,
    /// Enable fuzzy matching
    pub enable_fuzzy: bool,
    /// Enable result caching
    pub enable_cache: bool,
    /// Cache time-to-live in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum cache size (number of queries)
    pub max_cache_size: usize,
    /// Search timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 50,
            score_threshold: 0.1,
            enable_fuzzy: true,
            enable_cache: true,
            cache_ttl_seconds: 300, // 5 minutes
            max_cache_size: 100,
            timeout_ms: 5000, // 5 seconds
        }
    }
}

impl SearchConfig {
    /// Create a new search config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of results
    pub fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = max_results;
        self
    }

    /// Set score threshold
    pub fn with_score_threshold(mut self, threshold: f32) -> Self {
        self.score_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Enable or disable caching
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }

    /// Set cache TTL in seconds
    pub fn with_cache_ttl(mut self, seconds: u64) -> Self {
        self.cache_ttl_seconds = seconds;
        self
    }
}

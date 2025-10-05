use std::collections::HashSet;

use crate::types::*;

/// Search aggregator manager for coordinating multi-plugin searches
pub struct SearchAggregatorManager;

impl SearchAggregatorManager {
    /// Validate search query
    pub fn validate_query(query: &str, config: &SearchConfig) -> Result<(), String> {
        if query.trim().is_empty() {
            return Err("Search query cannot be empty".to_string());
        }

        if query.len() < config.min_query_length {
            return Err(format!(
                "Search query must be at least {} characters",
                config.min_query_length
            ));
        }

        // Check for potentially problematic queries
        if query.len() > 500 {
            return Err("Search query too long".to_string());
        }

        Ok(())
    }

    /// Filter and score search results
    pub fn process_results(
        results: Vec<SearchResult>,
        query: &str,
        max_results: usize,
    ) -> Vec<SearchResult> {
        let mut filtered_results = results;

        // Remove duplicates based on title and action
        filtered_results.sort_by(|a, b| match a.title.cmp(&b.title) {
            std::cmp::Ordering::Equal => a.action.cmp(&b.action),
            other => other,
        });
        filtered_results.dedup_by(|a, b| a.title == b.title && a.action == b.action);

        // Sort by score descending
        filtered_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply basic relevance boosting based on query matching
        for result in &mut filtered_results {
            if result.title.to_lowercase().contains(&query.to_lowercase()) {
                result.score += 0.2; // Boost score for title matches
            }
            if result
                .description
                .to_lowercase()
                .contains(&query.to_lowercase())
            {
                result.score += 0.1; // Smaller boost for description matches
            }
        }

        // Re-sort after scoring adjustments
        filtered_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        filtered_results.truncate(max_results);

        filtered_results
    }

    /// Merge results from multiple plugins, removing duplicates and applying scoring
    pub fn merge_plugin_results(
        plugin_results: Vec<(String, Vec<SearchResult>)>,
        query: &str,
        config: &SearchConfig,
    ) -> Vec<SearchResult> {
        let mut all_results = Vec::new();

        // Collect all results and tag them with plugin_id
        for (plugin_id, mut results) in plugin_results {
            for result in &mut results {
                result.plugin_id = plugin_id.clone();
            }
            all_results.extend(results);
        }

        Self::process_results(all_results, query, config.max_results_per_plugin * 5)
    }

    /// Check if enough plugins have responded to consider search complete
    pub fn should_complete_early(
        completed_plugins: &HashSet<String>,
        expected_plugins: &HashSet<String>,
        min_response_threshold: f32,
    ) -> bool {
        let completion_ratio = completed_plugins.len() as f32 / expected_plugins.len() as f32;
        completion_ratio >= min_response_threshold
    }
}

use std::collections::HashSet;

use log::info;

use super::types::DistributedSearchQuery;
use crate::plugins::core::CurrentSearchResults;

/// Merge distributed search results with current search results
pub fn merge_distributed_results(
    completed_query: &DistributedSearchQuery,
    current_results: &mut CurrentSearchResults,
) {
    // Add distributed results to current results
    current_results
        .results
        .extend(completed_query.results.clone());

    // Sort by score (highest first)
    current_results.results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Remove duplicates based on action ID
    let mut seen_actions = HashSet::new();
    current_results.results.retain(|item| {
        if seen_actions.contains(&item.action) {
            false
        } else {
            seen_actions.insert(item.action.clone());
            true
        }
    });

    // Limit to top results
    current_results.results.truncate(20);

    info!(
        "Merged distributed search results: {} total results after deduplication",
        current_results.results.len()
    );
}

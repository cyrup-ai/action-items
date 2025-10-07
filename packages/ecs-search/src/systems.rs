//! Core search systems
//!
//! Provides systems for processing search requests using SearchIndex.

pub mod filtering;

use bevy::prelude::*;

use action_items_core::search::{SearchIndex, SearchItem, SearchItemType};
use crate::{components::*, events::*, resources::*, scoring::SearchScore};

/// Process search requests using SearchIndex
pub fn process_search_requests(
    mut events: EventReader<SearchRequested>,
    mut resource: ResMut<SearchResource>,
    mut completed_events: EventWriter<SearchCompleted>,
    search_index: Res<SearchIndex>,
) {
    for event in events.read() {
        tracing::debug!("Processing search request: {}", event.query);

        // Check cache first if enabled
        if resource.config.enable_cache
            && let Some(cached) = resource.get_cached(&event.query)
        {
            tracing::debug!("Returning cached results for: {}", event.query);
            completed_events.write(SearchCompleted::new(
                event.query.clone(),
                cached.clone(),
                0,
                event.requester.clone(),
            ));
            continue;
        }

        // Update current query
        resource.current_query = event.query.clone();

        // Perform REAL search using SearchIndex
        let search_items = search_index.search(&event.query);
        
        // Convert SearchItem -> SearchResult  
        let results: Vec<SearchResult> = search_items
            .into_iter()
            .take(event.max_results)
            .filter(|item| item.score >= resource.config.score_threshold)
            .map(convert_search_item_to_result)
            .collect();

        let duration_ms = 0; // Synchronous search, instant

        tracing::debug!(
            "Search completed for '{}': {} results",
            event.query,
            results.len()
        );

        // Cache results if caching is enabled
        if resource.config.enable_cache {
            resource.cache_results(event.query.clone(), results.clone());
        }

        // Emit completion event
        completed_events.write(SearchCompleted::new(
            event.query.clone(),
            results,
            duration_ms,
            event.requester.clone(),
        ));
    }
}

/// Update search cache when results complete
pub fn update_search_cache(
    mut events: EventReader<SearchCompleted>,
    mut resource: ResMut<SearchResource>,
) {
    for event in events.read() {
        if resource.config.enable_cache {
            tracing::trace!("Caching {} results for query: {}", event.results.len(), event.query);
            resource.cache_results(event.query.clone(), event.results.clone());
        }
    }
}

/// Handle search query changes for live search
pub fn handle_query_changes(
    mut events: EventReader<SearchQueryChanged>,
    mut search_requests: EventWriter<SearchRequested>,
    resource: Res<SearchResource>,
) {
    for event in events.read() {
        // Debounce logic could go here
        tracing::trace!("Query changed to: {}", event.query);

        // Emit search request for the new query
        search_requests.write(SearchRequested::new(
            event.query.clone(),
            resource.config.max_results,
            event.requester.clone(),
        ));
    }
}

/// Convert SearchItem to SearchResult
fn convert_search_item_to_result(item: SearchItem) -> SearchResult {
    use crate::systems::filtering::FilterCategory;
    
    // Map icon path to string, or use emoji fallback
    let icon = item.icon_path
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| match item.item_type {
            SearchItemType::Application => "🚀".to_string(),
            SearchItemType::File => "📄".to_string(),
            SearchItemType::Directory => "📁".to_string(),
            SearchItemType::Command => "⚡".to_string(),
            SearchItemType::Plugin => "🔌".to_string(),
            SearchItemType::ActionItem => "✨".to_string(),
        });
    
    // Map item type to filter categories
    let categories = match item.item_type {
        SearchItemType::Application => vec![FilterCategory::Applications],
        SearchItemType::File | SearchItemType::Directory => vec![FilterCategory::Documents],
        SearchItemType::Command => vec![FilterCategory::System],
        SearchItemType::Plugin => vec![FilterCategory::Plugin("unknown".to_string())],
        SearchItemType::ActionItem => vec![FilterCategory::Custom("action-items".to_string())],
    };
    
    // Use existing score from SearchIndex
    let score = item.score;
    
    // Create SearchScore with proper weighting
    let score_details = SearchScore::new(
        score / 100.0,      // text_match (SearchIndex uses 0-100, normalize to 0-1)
        0.5,                 // frequency (default)
        0.5,                 // recency (default)
        0.7,                 // relevance (based on match quality)
    );
    
    SearchResult::new(
        item.id,
        item.title,
        item.description,
        icon,
        score,
    )
    .with_categories(categories)
    .with_score_details(score_details)
}

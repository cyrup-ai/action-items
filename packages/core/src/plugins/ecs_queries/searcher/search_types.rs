//! Search result types and data structures

/// Search result for plugin queries
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub plugin_id: String,
    pub plugin_name: String,
    pub plugin_type: PluginType,
    pub match_score: f32,
}

/// Plugin type enumeration for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    Native,
    Extism,
    Raycast,
}

/// Calculate match score for search relevance
pub fn calculate_match_score(name: &str, description: &str, query: &str) -> f32 {
    let query_lower = query.to_lowercase();
    let name_lower = name.to_lowercase();
    let description_lower = description.to_lowercase();

    // Exact name match gets highest score
    if name_lower == query_lower {
        return 1.0;
    }

    // Name starts with query gets high score
    if name_lower.starts_with(&query_lower) {
        return 0.9;
    }

    // Name contains query gets medium score
    if name_lower.contains(&query_lower) {
        return 0.7;
    }

    // Description contains query gets lower score
    if description_lower.contains(&query_lower) {
        return 0.5;
    }

    // No match
    0.0
}

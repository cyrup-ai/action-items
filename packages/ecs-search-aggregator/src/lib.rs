//! Action Items ECS Search Aggregator
//!
//! Bevy ECS plugin for aggregating search results from multiple plugins.
//! Coordinates distributed search across the plugin ecosystem for the
//! Raycast/Alfred-style launcher interface.

use bevy::prelude::*;

pub mod components;
pub mod events;
pub mod manager;
pub mod plugin;
pub mod types;

// Re-export core types
pub use components::*;
pub use events::*;
pub use manager::SearchAggregatorManager;
pub use plugin::SearchAggregatorPlugin;
pub use types::*;

/// Convenience function to add the search aggregator system to a Bevy app
pub fn add_search_aggregator(app: &mut App) {
    app.add_plugins(SearchAggregatorPlugin);
}

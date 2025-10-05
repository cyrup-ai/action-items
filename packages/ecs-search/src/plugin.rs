//! Search plugin for Bevy ECS
//!
//! Provides the main SearchPlugin that registers all search functionality.

use bevy::prelude::*;
use crate::{resources::*, events::*, systems::*};

/// Main search plugin
///
/// Provides core search functionality with event-driven architecture.
/// Can be used headless (without UI) for search operations.
///
/// # Example
///
/// ```no_run
/// use bevy::prelude::*;
/// use action_items_ecs_search::SearchPlugin;
///
/// App::new()
///     .add_plugins(SearchPlugin)
///     .run();
/// ```
pub struct SearchPlugin;

impl Plugin for SearchPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register resources
            .insert_resource(SearchResource::default())
            
            // Register events
            .add_event::<SearchRequested>()
            .add_event::<SearchCompleted>()
            .add_event::<SearchQueryChanged>()
            .add_event::<SearchError>()
            .add_event::<SearchCacheCleared>()
            
            // Register systems - SIMPLIFIED
            .add_systems(Update, (
                process_search_requests,
                handle_query_changes,
            ).chain());

        tracing::info!("SearchPlugin initialized");
    }
}

/// Search plugin with custom configuration
pub struct SearchPluginWithConfig {
    /// Custom search configuration
    pub config: SearchConfig,
}

impl SearchPluginWithConfig {
    /// Create a new plugin with custom configuration
    pub fn new(config: SearchConfig) -> Self {
        Self { config }
    }
}

impl Plugin for SearchPluginWithConfig {
    fn build(&self, app: &mut App) {
        app
            // Register resources with custom config
            .insert_resource(SearchResource::with_config(self.config.clone()))
            
            // Register events
            .add_event::<SearchRequested>()
            .add_event::<SearchCompleted>()
            .add_event::<SearchQueryChanged>()
            .add_event::<SearchError>()
            .add_event::<SearchCacheCleared>()
            
            // Register systems - SIMPLIFIED
            .add_systems(Update, (
                process_search_requests,
                handle_query_changes,
            ).chain());

        tracing::info!("SearchPlugin initialized with custom config");
    }
}

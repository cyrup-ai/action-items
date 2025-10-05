//! Main PluginSearcher SystemParam and core search logic

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use log::info;

use super::search_strategies::{
    quick_search_all_plugins, search_extism_plugins, search_native_plugins, search_raycast_plugins,
};
use super::search_types::SearchResult;
use crate::plugins::extism::wrapper::ExtismPluginComponent;
use crate::plugins::native::wrapper::PluginComponent;
use crate::raycast::wrapper::RaycastPluginComponent;

/// ECS-based plugin search functionality
///
/// This SystemParam provides search capabilities across all plugin types.
#[derive(SystemParam)]
pub struct PluginSearcher<'w, 's> {
    native_plugins: Query<'w, 's, &'static PluginComponent>,
    extism_plugins: Query<'w, 's, &'static ExtismPluginComponent>,
    raycast_plugins: Query<'w, 's, &'static RaycastPluginComponent>,
}

impl<'w, 's> PluginSearcher<'w, 's> {
    /// Search all plugins asynchronously and spawn ECS entities with search tasks
    pub fn search_plugins_ecs(
        &self,
        commands: &mut Commands,
        query: &str,
        task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) {
        // Search native plugins with proper implementation
        search_native_plugins(commands, &self.native_plugins, query, task_pool);

        // Search extism plugins with WASM function calls
        search_extism_plugins(commands, &self.extism_plugins, query, task_pool);

        // Search raycast plugins
        search_raycast_plugins(commands, &self.raycast_plugins, query, task_pool);

        info!(
            "Started search tasks for {} plugins",
            self.native_plugins.iter().count()
                + self.extism_plugins.iter().count()
                + self.raycast_plugins.iter().count()
        );
    }

    /// Synchronous search for quick metadata-based results
    pub fn quick_search(&self, query: &str) -> Vec<SearchResult> {
        quick_search_all_plugins(
            &self.native_plugins,
            &self.extism_plugins,
            &self.raycast_plugins,
            query,
        )
    }
}

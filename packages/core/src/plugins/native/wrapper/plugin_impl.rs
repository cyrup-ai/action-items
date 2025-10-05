//! Bevy Plugin trait implementation

use bevy::prelude::*;
use log::info;

use super::types::{NativePluginWrapper, PluginComponent};
use crate::search::{SearchIndex, SearchItem, SearchItemType};

impl Plugin for NativePluginWrapper {
    fn build(&self, app: &mut App) {
        let metadata = self.metadata.clone();
        let plugin = self.plugin.clone();
        
        // Use startup system to spawn entity instead of direct world spawning
        app.add_systems(Startup, move |mut commands: Commands| {
            commands.spawn(PluginComponent {
                id: metadata.id.clone(),
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                capabilities: metadata.capabilities.clone(),
                config: metadata.clone(),
                plugin: plugin.clone(),
            });
        });

        // Move SearchIndex operations to startup system to avoid CommandQueue issues
        let metadata_for_search = self.metadata.clone();
        
        app.add_systems(Startup, move |mut search_index: ResMut<SearchIndex>| {
            // Create search items from plugin manifest
            let manifest = &metadata_for_search.manifest;

            // Add main plugin entry
            let plugin_item = SearchItem::new(
                format!("native:{}", metadata_for_search.id),
                metadata_for_search.name.clone(),
                manifest.description.clone(),
                SearchItemType::Plugin,
            )
            .with_keywords(manifest.keywords.clone());

            search_index.add_item(plugin_item);

            // Add action entries
            for action in &manifest.actions {
                let action_item = SearchItem::new(
                    format!("native:{}:{}", metadata_for_search.id, action.id),
                    action.title.clone(),
                    action.description.clone().unwrap_or_default(),
                    SearchItemType::ActionItem,
                );

                search_index.add_item(action_item);
            }

            info!(
                "Added native plugin '{}' with {} actions to search index",
                metadata_for_search.name,
                manifest.actions.len()
            );
        });
    }
}

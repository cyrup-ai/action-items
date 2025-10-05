use std::path::PathBuf;

use bevy::prelude::*;
use log::info;

use super::core::DenoPluginWrapper;
use crate::plugins::services::PluginId;
use crate::search::{SearchIndex, SearchItem, SearchItemType};

/// Component that holds Deno plugin instance and metadata
#[derive(Component)]
pub struct DenoPluginComponent {
    pub plugin_id: PluginId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub entry_point: PathBuf,
}

impl Plugin for DenoPluginWrapper {
    fn build(&self, app: &mut App) {
        let plugin_id = self.plugin_id.clone();
        let metadata = self.metadata.clone();
        
        // Use startup system to spawn entity instead of direct world spawning
        app.add_systems(Startup, move |mut commands: Commands| {
            commands.spawn(DenoPluginComponent {
                plugin_id: plugin_id.clone(),
                name: metadata.name.clone(),
                version: metadata.manifest.version.clone(),
                description: metadata.manifest.description.clone(),
                entry_point: metadata.path.clone(),
            });
        });

        // Move SearchIndex operations to startup system to avoid CommandQueue issues
        let metadata_for_search = self.metadata.clone();
        
        app.add_systems(Startup, move |mut search_index: ResMut<SearchIndex>| {
            // Add main plugin entry
            let plugin_item = SearchItem::new(
                format!("deno:{}", metadata_for_search.name),
                metadata_for_search.name.clone(),
                metadata_for_search.manifest.description.clone(),
                SearchItemType::Plugin,
            );

            search_index.add_item(plugin_item);

            info!(
                "Added Deno plugin '{}' to search index",
                metadata_for_search.name
            );
        });

        info!(
            "Registered Deno plugin: {} (v{}) from {:?}",
            self.metadata.name, self.metadata.manifest.version, self.metadata.path
        );
    }
}

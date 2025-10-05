use bevy::prelude::*;

use super::metadata::ActionItem;

/// Resource to hold the latest aggregated search results from all plugins.
#[derive(Resource, Default, Debug, Clone)]
pub struct CurrentSearchResults {
    pub results: Vec<ActionItem>,
}

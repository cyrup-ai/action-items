use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Generic icon cache resource for managing loaded icons
///
/// Tracks successfully loaded icons and failed load attempts to prevent
/// redundant loading operations.
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use action_items_ecs_ui::icons::IconCache;
///
/// fn setup(mut commands: Commands) {
///     commands.insert_resource(IconCache::new());
/// }
/// ```
#[derive(Resource, Default, Debug)]
pub struct IconCache {
    /// Map of icon IDs to loaded image handles
    ///
    /// Icon IDs are typically file paths or generated identifiers.
    pub loaded_icons: HashMap<String, Handle<Image>>,
    
    /// Set of icon IDs that failed to load
    ///
    /// Prevents repeated load attempts for missing or invalid icons.
    pub failed_to_load: HashSet<String>,
}

impl IconCache {
    /// Create a new empty icon cache
    pub fn new() -> Self {
        Self::default()
    }
}

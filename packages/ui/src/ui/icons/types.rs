use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use bevy::prelude::*;

// Import generic types from ecs-ui
pub use action_items_ecs_ui::icons::{IconSize, IconType, ThemeColors, IconTheme};

// App-specific icon extraction events
#[derive(Event, Clone)]
pub struct IconExtractionRequest {
    pub id: String,
    pub path: PathBuf,
    pub icon_type: IconType,
    pub size: IconSize,
}

#[derive(Event)]
pub struct IconExtractionResult {
    pub id: String,
    pub icon_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Resource for managing pending icon extraction requests
#[derive(Resource, Default)]
pub struct IconExtractionQueue {
    pub pending: Vec<String>,
}

// Extended icon cache for launcher-specific functionality
// Wraps ecs-ui IconCache and adds generic icon fallback system
#[derive(Resource, Default)]
pub struct LauncherIconCache {
    /// Base cache from ecs-ui
    pub base: action_items_ecs_ui::icons::IconCache,
    /// Launcher-specific: fallback icons by type
    pub generic_icons: HashMap<IconType, Handle<Image>>,
}

impl LauncherIconCache {
    pub fn new() -> Self {
        Self::default()
    }
    
    // Convenience accessors that delegate to base
    pub fn loaded_icons(&self) -> &HashMap<String, Handle<Image>> {
        &self.base.loaded_icons
    }
    
    pub fn loaded_icons_mut(&mut self) -> &mut HashMap<String, Handle<Image>> {
        &mut self.base.loaded_icons
    }
    
    pub fn failed_to_load(&self) -> &HashSet<String> {
        &self.base.failed_to_load
    }
    
    pub fn failed_to_load_mut(&mut self) -> &mut HashSet<String> {
        &mut self.base.failed_to_load
    }
}

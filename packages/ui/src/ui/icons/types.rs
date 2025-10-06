use std::collections::HashMap;
use bevy::prelude::*;

// Import base types from ecs-ui (used by LauncherIconCache)
use action_items_ecs_ui::icons::{IconCache, IconType};

// Re-export ecs-ui events (remove local duplicates)
pub use action_items_ecs_ui::icons::{IconExtractionRequest, IconExtractionResult};

/// Launcher-specific icon cache with generic fallback system
///
/// Wraps ecs-ui's IconCache and adds launcher-specific fallback icons.
/// When an icon can't be loaded (app deleted, permission denied, etc.),
/// the launcher shows a generic icon based on type (folder, app, document).
///
/// # Architecture
/// - `base`: Standard icon cache from ecs-ui (loaded_icons, failed_to_load)
/// - `generic_icons`: Launcher fallback system (IconType → generic Handle<Image>)
///
/// # Example
/// ```rust
/// // Try loaded icon first, fallback to generic
/// let icon = cache.loaded_icons().get(&app_path)
///     .or_else(|| cache.generic_icons.get(&IconType::Application))
///     .cloned()
///     .unwrap_or_default();
/// ```
#[derive(Resource, Default)]
pub struct LauncherIconCache {
    /// Base cache from ecs-ui
    pub base: IconCache,
    /// Launcher-specific: fallback icons by type
    pub generic_icons: HashMap<IconType, Handle<Image>>,
}

impl LauncherIconCache {
    pub fn new() -> Self {
        Self::default()
    }
    
    // Delegate to base for standard operations
    pub fn loaded_icons(&self) -> &std::collections::HashMap<String, Handle<Image>> {
        &self.base.loaded_icons
    }
    
    pub fn loaded_icons_mut(&mut self) -> &mut std::collections::HashMap<String, Handle<Image>> {
        &mut self.base.loaded_icons
    }
    
    pub fn failed_to_load(&self) -> &std::collections::HashSet<String> {
        &self.base.failed_to_load
    }
    
    pub fn failed_to_load_mut(&mut self) -> &mut std::collections::HashSet<String> {
        &mut self.base.failed_to_load
    }
}

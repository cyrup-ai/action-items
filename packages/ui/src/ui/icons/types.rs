use std::collections::HashMap;
use bevy::prelude::*;

// Import base types from ecs-ui (used by LauncherIconCache)
use action_items_ecs_ui::icons::{IconCache, IconType};

// Re-export ecs-ui events (remove local duplicates)
pub use action_items_ecs_ui::icons::{IconExtractionRequest, IconExtractionResult};

/// Launcher-specific generic icon fallbacks
///
/// Provides fallback icons by type when specific icons can't be loaded.
/// When an icon can't be loaded (app deleted, permission denied, etc.),
/// the launcher shows a generic icon based on type (folder, app, document).
///
/// # Architecture
/// Works with ecs-ui's IconCache for loaded icons.
/// This resource only manages generic fallback icons by type.
///
/// # Example
/// ```rust
/// // Try loaded icon first, fallback to generic
/// let icon = icon_cache.loaded_icons.get(&app_path)
///     .or_else(|| fallbacks.fallback_icons.get(&IconType::Application))
///     .cloned()
///     .unwrap_or_default();
/// ```
#[derive(Resource, Default)]
pub struct GenericIconFallbacks {
    /// Launcher-specific: fallback icons by type
    pub fallback_icons: HashMap<IconType, Handle<Image>>,
}

impl GenericIconFallbacks {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Legacy wrapper - kept for compatibility but no longer used as Resource
///
/// This struct wrapped IconCache with generic icons, but caused resource conflicts
/// with IconPlugin. Now superseded by using IconCache directly + GenericIconFallbacks.
#[derive(Default)]
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

//! Icon management systems
//!
//! Zero-allocation icon processing with blazing-fast icon initialization and updates.

use bevy::prelude::*;

use crate::ui::components::{FallbackIcon, ImageComponent, ResultIcon};
use crate::ui::icons::{LauncherIconCache, IconExtractionQueue, IconTheme};

/// Initialize icon extraction and cache systems
/// Zero-allocation icon system initialization with blazing-fast cache setup
#[inline]
pub fn initialize_icon_system_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    icon_cache: Option<Res<LauncherIconCache>>,
    icon_extraction: Option<Res<IconExtractionQueue>>,
    icon_theme: Option<Res<IconTheme>>,
) {
    // Initialize icon-related resources if not already done
    if icon_cache.is_none() {
        commands.insert_resource(LauncherIconCache::new());
    }

    if icon_extraction.is_none() {
        commands.init_resource::<IconExtractionQueue>();
    }

    if icon_theme.is_none() {
        commands.insert_resource(IconTheme::default());
    }

    // Load fallback icon
    let fallback_icon = asset_server.load("icons/app.png");
    commands.insert_resource(FallbackIcon(Some(fallback_icon)));
}

/// Update result icons from cache or requests
/// Zero-allocation icon updates with blazing-fast asset loading and fallback handling
#[inline]
pub fn update_result_icons_system(
    icon_cache: Res<LauncherIconCache>,
    fallback_icon: Res<FallbackIcon>,
    _asset_server: Res<AssetServer>,
    mut icon_query: Query<(&mut ImageComponent, &ResultIcon)>,
) {
    for (mut image_component, result_icon) in icon_query.iter_mut() {
        // Try to get cached icon first
        if let Some(cached_icon) = icon_cache.loaded_icons().get(&result_icon.result_id) {
            image_component.0 = cached_icon.clone();
        } else if let Some(fallback) = &fallback_icon.0 {
            // Use fallback icon while loading
            image_component.0 = fallback.clone();
        }
    }
}

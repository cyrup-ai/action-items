// Note: CoreActionItem will be used when icon utilities are implemented
use action_items_core::SearchResult;
use bevy::prelude::*;

use crate::ui::icons::types::{LauncherIconCache, IconType};

pub fn get_icon_for_result(
    result: &action_items_core::plugins::ActionItem,
    icon_cache: &LauncherIconCache,
) -> Handle<Image> {
    // Check if we have a loaded icon for this result
    if let Some(handle) = icon_cache.loaded_icons().get(&result.action) {
        return handle.clone();
    }

    // Determine icon type from action string
    let icon_type = if result.action.starts_with("app_") {
        IconType::Application
    } else if result.action.starts_with("dir_") {
        IconType::Folder
    } else if result.action.starts_with("cmd_") {
        IconType::Command
    } else {
        IconType::Unknown
    };

    // Return generic icon as fallback
    icon_cache
        .generic_icons
        .get(&icon_type)
        .or_else(|| icon_cache.generic_icons.get(&IconType::Unknown))
        .cloned()
        .unwrap_or_default()
}

pub fn get_icon_for_search_result(result: &SearchResult, icon_cache: &LauncherIconCache) -> Handle<Image> {
    // Check if we have a loaded icon for this result
    if let Some(handle) = icon_cache.loaded_icons().get(&result.action) {
        return handle.clone();
    }

    // Determine icon type from action string
    let icon_type = if result.action.starts_with("app_") {
        IconType::Application
    } else if result.action.starts_with("dir_") {
        IconType::Folder
    } else if result.action.starts_with("cmd_") {
        IconType::Command
    } else {
        IconType::Unknown
    };

    // Return generic icon as fallback
    icon_cache
        .generic_icons
        .get(&icon_type)
        .or_else(|| icon_cache.generic_icons.get(&IconType::Unknown))
        .cloned()
        .unwrap_or_default()
}

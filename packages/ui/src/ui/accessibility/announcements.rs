use bevy::prelude::*;

use action_items_ecs_ui::accessibility::AccessibilityManager;

/// System to update accessibility announcements
pub fn update_accessibility_announcements(
    mut accessibility_manager: ResMut<AccessibilityManager>,
    search_results: Res<action_items_core::CurrentSearchResults>,
) {
    // Announce search result updates
    if search_results.is_changed() && !search_results.results.is_empty() {
        let count = search_results.results.len();
        let announcement = if count == 1 {
            "1 search result found".to_string()
        } else {
            format!("{count} search results found")
        };
        accessibility_manager.announcements.push(announcement);
    }
}

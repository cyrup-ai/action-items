use bevy::prelude::*;

use action_items_ecs_ui::accessibility::{AccessibleElement, FocusStyle, FocusableElement, LiveRegion};
use crate::ui::components::{ActionResultItem, LauncherContainer, SearchInput};

/// System to setup accessibility for launcher components
pub fn setup_accessibility(
    mut commands: Commands,
    launcher_query: Query<Entity, (With<LauncherContainer>, Without<AccessibleElement>)>,
    search_query: Query<Entity, (With<SearchInput>, Without<AccessibleElement>)>,
    result_query: Query<Entity, (With<ActionResultItem>, Without<AccessibleElement>)>,
) {
    // Setup launcher container accessibility
    for entity in launcher_query.iter() {
        commands.entity(entity).insert((
            AccessibleElement {
                role: accesskit::Role::Application,
                name: "Action Items Launcher".to_string(),
                description: Some(
                    "Quick application launcher with search functionality".to_string(),
                ),
                focusable: false,
                live_region: Some(LiveRegion::Polite),
                ..default()
            },
            // AccessibilityNode integration removed - not available in Bevy 0.16.1
        ));
    }

    // Setup search input accessibility
    for entity in search_query.iter() {
        commands.entity(entity).insert((
            AccessibleElement {
                role: accesskit::Role::TextInput,
                name: "Search for applications and actions".to_string(),
                description: Some(
                    "Type to search for applications, files, and actions".to_string(),
                ),
                focusable: true,
                tab_index: Some(0),
                ..default()
            },
            FocusableElement {
                tab_order: 1,
                focus_style: FocusStyle::Combined,
                ..default()
            },
            // AccessibilityNode integration removed - not available in Bevy 0.16.1
        ));
    }

    // Setup result item accessibility
    for (index, entity) in result_query.iter().enumerate() {
        commands.entity(entity).insert((
            AccessibleElement {
                role: accesskit::Role::Button,
                name: {
                    let result_number = index + 1;
                    format!("Search result {result_number}")
                },
                description: Some("Press Enter to execute this action".to_string()),
                focusable: true,
                tab_index: Some(index as i32 + 2),
                ..default()
            },
            FocusableElement {
                tab_order: index as u32 + 2,
                focus_style: FocusStyle::Combined,
                ..default()
            },
            // AccessibilityNode integration removed - not available in Bevy 0.16.1
        ));
    }
}

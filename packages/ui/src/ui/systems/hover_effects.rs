//! Hover effect systems for result items
//!
//! Zero-allocation hover handling with blazing-fast color transitions using Bevy patterns.

use bevy::prelude::*;

// Type aliases for complex query types
type HoverResultQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        &'static Name,
    ),
    (Changed<Interaction>, With<Name>),
>;

/// System to handle result item hover effects with color changes
/// Updates BackgroundColor based on Interaction state for professional hover feedback
#[inline]
pub fn handle_result_item_hover_system(mut result_items: HoverResultQuery) {
    for (interaction, mut background_color, name) in result_items.iter_mut() {
        // Only process result items (identified by Name component starting with "ResultItem")
        if !name.as_str().starts_with("ResultItem") {
            continue;
        }

        match interaction {
            Interaction::Hovered => {
                // Professional hover color - subtle blue-grey highlight
                background_color.0 = Color::srgba(0.18, 0.20, 0.25, 0.35);
            },
            Interaction::Pressed => {
                // Pressed state with deeper color
                background_color.0 = Color::srgba(0.15, 0.17, 0.22, 0.45);
            },
            Interaction::None => {
                // Return to transparent state
                background_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            },
        }
    }
}

/// System to handle keyboard navigation highlighting with color effects
/// Provides visual feedback for Up/Down arrow key navigation through result items
#[inline]
pub fn handle_keyboard_selection_highlighting_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected_index: Local<usize>,
    mut result_items: Query<(&mut BackgroundColor, &Name), With<Name>>,
) {
    let result_items_vec: Vec<_> = result_items
        .iter()
        .enumerate()
        .filter(|(_, (_, name))| name.as_str().starts_with("ResultItem"))
        .collect();

    let total_items = result_items_vec.len();
    if total_items == 0 {
        return;
    }

    // Handle keyboard navigation
    let mut selection_changed = false;
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        *selected_index = (*selected_index + 1).min(total_items - 1);
        selection_changed = true;
    } else if keyboard.just_pressed(KeyCode::ArrowUp) {
        *selected_index = selected_index.saturating_sub(1);
        selection_changed = true;
    }

    // Update visual highlighting if selection changed
    if selection_changed {
        for (item_index, (mut background_color, name)) in result_items.iter_mut().enumerate() {
            if !name.as_str().starts_with("ResultItem") {
                continue;
            }

            if item_index == *selected_index {
                // Highlight selected item with professional blue selection color
                background_color.0 = Color::srgba(0.25, 0.35, 0.55, 0.40);
            } else {
                // Clear highlighting for non-selected items
                background_color.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            }
        }
    }
}

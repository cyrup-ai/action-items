//! Application-specific gradient system for Action Items
//!
//! This module contains gradient systems specific to the Action Items application.
//! Generic gradient systems are provided by the ecs-ui crate.

use bevy::prelude::*;
use action_items_ecs_ui::gradients::{GradientComponent, GradientInteractionState};
use crate::ui::components::{ActionItemsSearchResultItem, UiState};

/// Gradient selection system for Action Items search results
///
/// Application-specific system that updates gradient states based on ActionItemsSearchResultItem selection.
/// Integrates with the Action Items UI state to provide visual feedback for the selected result.
///
/// This is application-specific because it uses:
/// - `ActionItemsSearchResultItem` - Action Items specific component
/// - `UiState.selected_index` - Action Items specific state management
#[inline]
pub fn gradient_selection_system(
    mut gradient_components: Query<(Entity, &mut GradientComponent)>,
    result_items: Query<(Entity, &ActionItemsSearchResultItem)>,
    ui_state: Res<UiState>,
) {
    // Update gradient components based on selection state
    for (entity, mut gradient_component) in gradient_components.iter_mut() {
        // Find if this entity has ActionItemsSearchResultItem with index matching selected_index
        let is_selected = result_items
            .iter()
            .any(|(e, item)| e == entity && item.index == ui_state.selected_index);

        // Update gradient state based on selection
        let new_state = if is_selected {
            GradientInteractionState::Selected
        } else {
            match gradient_component.interaction_state {
                GradientInteractionState::Selected => GradientInteractionState::Default,
                current_state => current_state, // Preserve other states (Hover, Pressed)
            }
        };

        // Only update if state changed to avoid unnecessary work
        if gradient_component.interaction_state != new_state {
            gradient_component.interaction_state = new_state;
        }
    }
}

use bevy::prelude::*;

use super::components::{AccessibleElement, FocusableElement};
use super::manager::AccessibilityManager;
use super::events::FocusChanged;

/// Keyboard navigation system for accessibility
pub fn handle_accessibility_navigation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut accessibility_manager: ResMut<AccessibilityManager>,
    mut focusable_query: Query<(Entity, &mut FocusableElement)>,
    mut _accessible_query: Query<&mut AccessibleElement>,
    mut focus_events: EventWriter<FocusChanged>,
) {
    // Tab navigation
    if keyboard_input.just_pressed(KeyCode::Tab) {
        let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft)
            || keyboard_input.pressed(KeyCode::ShiftRight);

        navigate_focus(
            &mut accessibility_manager,
            &mut focusable_query,
            !shift_pressed,
            &mut focus_events,
        );
    }

    // Escape to reset focus
    if keyboard_input.just_pressed(KeyCode::Escape) {
        clear_focus(&mut accessibility_manager, &mut focusable_query);
    }

    // Enter to activate focused element
    if keyboard_input.just_pressed(KeyCode::Enter)
        && let Some(_focused_entity) = accessibility_manager.focused_element
    {
        // Trigger action on focused element
        accessibility_manager
            .announcements
            .push("Action executed".to_string());
    }
}

/// Focus navigation helper
pub fn navigate_focus(
    accessibility_manager: &mut AccessibilityManager,
    focusable_query: &mut Query<(Entity, &mut FocusableElement)>,
    forward: bool,
    focus_events: &mut EventWriter<FocusChanged>,
) {
    // Collect entities and tab orders for sorting (immutable data only)
    let mut focusable_entities: Vec<(Entity, u32)> = focusable_query
        .iter()
        .map(|(entity, element)| (entity, element.tab_order))
        .collect();

    // Sort by tab order
    focusable_entities.sort_by_key(|(_, tab_order)| *tab_order);

    let current_index = if let Some(focused_entity) = accessibility_manager.focused_element {
        focusable_entities
            .iter()
            .position(|(entity, _)| *entity == focused_entity)
    } else {
        None
    };

    let next_index = match current_index {
        Some(index) => {
            if forward {
                (index + 1) % focusable_entities.len()
            } else if index == 0 {
                focusable_entities.len() - 1
            } else {
                index - 1
            }
        },
        None => 0, // Focus first element if none focused
    };

    let old_focus = accessibility_manager.focused_element;

    // Clear previous focus
    for (_, mut element) in focusable_query.iter_mut() {
        element.focused = false;
    }

    // Set new focus
    if let Some((target_entity, _)) = focusable_entities.get(next_index)
        && let Ok((_, mut element)) = focusable_query.get_mut(*target_entity)
    {
        element.focused = true;
        accessibility_manager.focused_element = Some(*target_entity);
        let element_number = next_index + 1;
        accessibility_manager
            .announcements
            .push(format!("Focused on element {element_number}"));
        
        // Emit focus changed event
        focus_events.write(FocusChanged {
            old_focus,
            new_focus: accessibility_manager.focused_element,
        });
    }
}

/// Clear all focus states
pub fn clear_focus(
    accessibility_manager: &mut AccessibilityManager,
    focusable_query: &mut Query<(Entity, &mut FocusableElement)>,
) {
    for (_, mut element) in focusable_query.iter_mut() {
        element.focused = false;
    }
    accessibility_manager.focused_element = None;
}

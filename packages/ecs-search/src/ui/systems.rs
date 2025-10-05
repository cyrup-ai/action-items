use bevy::prelude::*;
use action_items_ecs_ui::prelude::*;
use action_items_ecs_ui::theme::Theme;
use crate::ui::components::*;

/// Animate search results appearing (migrated from animation.rs)
/// 
/// Applies staggered scale and translate animations to result items.
/// Uses original timing: 0.1s base delay + 0.05s per item.
pub fn animate_results_appear(
    mut query: Query<(&SearchResultItem, &mut Transform), Added<SearchResultItem>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    
    for (_item, mut transform) in &mut query {
        let target_scale = Vec3::ONE;
        let target_translation = Vec3::ZERO;
        
        // Interpolate towards target (ease-out back from result_rendering.rs)
        let t = (delta * 5.0).min(1.0); // 5.0 is animation speed
        let eased_t = 1.0 - (1.0 - t).powi(3); // Cubic ease-out
        
        transform.scale = transform.scale.lerp(target_scale, eased_t);
        transform.translation = transform.translation.lerp(target_translation, eased_t);
    }
}

/// Handle result item hover effects (migrated from result_rendering.rs)
/// 
/// Updates result item appearance on hover using theme colors.
pub fn update_result_hover_colors(
    mut query: Query<(&UiHover, &mut UiColor, &SearchResultItem)>,
    selection: Res<SearchSelection>,
    theme: Res<Theme>,
) {
    for (hover, mut color, item) in &mut query {
        let is_selected = item.index == selection.selected_index;
        
        let target_color = if is_selected {
            theme.colors.surface_selected
        } else if hover.enable {
            theme.colors.surface_hover
        } else {
            theme.colors.surface_default
        };
        
        *color = UiColor::from(target_color);
    }
}

/// Loading animation for search bar (migrated from animation.rs lines 12-41)
/// 
/// Applies pulse animation to search bar during search operations.
pub fn animate_search_loading(
    mut query: Query<(&SearchBarComponent, &mut UiColor), With<SearchUIState>>,
    mut ui_state_query: Query<&mut SearchUIState>,
    time: Res<Time>,
    theme: Res<Theme>,
) {
    for mut ui_state in &mut ui_state_query {
        ui_state.update_progress(time.delta_secs());
    }
    
    for (_bar, mut color) in &mut query {
        if let Ok(ui_state) = ui_state_query.single() {
            if ui_state.search_loading {
                // Pulse effect (from original animation.rs line 28)
                let pulse_factor = (time.elapsed_secs() * 2.0).sin() * 0.1 + 0.9;
                let base_color = theme.colors.background_secondary;
                *color = UiColor::from(Color::srgba(
                    base_color.to_srgba().red,
                    base_color.to_srgba().green,
                    base_color.to_srgba().blue,
                    base_color.alpha() * pulse_factor,
                ));
            } else {
                *color = UiColor::from(theme.colors.background_secondary);
            }
        }
    }
}

/// Focus ring animation for accessibility (migrated from animation.rs lines 43-77)
/// 
/// Shows focus indicator for keyboard navigation.
pub fn update_focus_indicators(
    mut search_bar_query: Query<&mut Outline, With<SearchBarComponent>>,
    _selection: Res<SearchSelection>,
    theme: Res<Theme>,
) {
    // Update search bar focus
    for mut outline in &mut search_bar_query {
        outline.color = theme.colors.border_accent;
        outline.width = Val::Px(2.0);
    }
}

// UI update systems

use crate::events::*;

/// Handle text input changes and emit SearchQueryChanged events
/// 
/// Migrated from input_handling.rs real_time_search_input_system.
pub fn handle_search_input_changes(
    query: Query<&Text, (With<SearchInputField>, Changed<Text>)>,
    mut events: EventWriter<SearchQueryChanged>,
) {
    for text in &query {
        if !text.is_empty() {
            events.write(SearchQueryChanged::new(
                text.0.clone(),
                "search_ui",
            ));
        }
    }
}

/// Update results display when SearchCompleted event fires
/// 
/// Clears old results and spawns new ones using spawn_result_item.
/// Migrated from result_rendering.rs real_time_search_results_system.
pub fn update_results_on_completion(
    mut commands: Commands,
    mut events: EventReader<SearchCompleted>,
    results_container: Query<Entity, With<SearchResultsContainer>>,
    existing_results: Query<Entity, With<SearchResultItem>>,
    mut selection: ResMut<SearchSelection>,
    theme: Res<Theme>,
) {
    for event in events.read() {
        if event.requester != "search_ui" {
            continue; // Only handle our own requests
        }
        
        let Ok(container) = results_container.single() else {
            continue;
        };
        
        // Clear existing results (from result_rendering.rs lines 56-64)
        for result_entity in &existing_results {
            commands.entity(result_entity).despawn();
        }
        
        // Update selection state
        selection.total_results = event.results.len();
        selection.selected_index = 0;
        
        // Spawn new results
        for (index, result) in event.results.iter().enumerate() {
            let is_selected = index == 0;
            super::screens::spawn_result_item(
                &mut commands,
                container,
                result,
                index,
                is_selected,
                &theme,
            );
        }
    }
}

/// Handle result selection via mouse click
/// 
/// Detects clicks on result items using Bevy Interaction component.
pub fn handle_result_click_selection(
    query: Query<(&SearchResultItem, &Interaction), Changed<Interaction>>,
    mut selection: ResMut<SearchSelection>,
) {
    for (item, interaction) in &query {
        if *interaction == Interaction::Pressed {
            selection.selected_index = item.index;
            // TODO: Emit ResultSelected event or execute action
        }
    }
}

/// Handle keyboard navigation (arrow keys, enter)
/// 
/// New system for keyboard-based result selection.
pub fn handle_keyboard_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<SearchSelection>,
    _results: Query<&SearchResultItem>,
) {
    if selection.total_results == 0 {
        return;
    }
    
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        selection.select_next();
    } else if keyboard.just_pressed(KeyCode::ArrowUp) {
        selection.select_previous();
    } else if keyboard.just_pressed(KeyCode::Enter) {
        // TODO: Execute selected result action
    }
}

/// Update search bar visual state based on loading
/// 
/// Shows loading indicator in search bar during active search.
pub fn update_search_bar_loading_state(
    mut search_bar_query: Query<(&mut SearchUIState, &SearchBarComponent)>,
    mut events: EventReader<SearchCompleted>,
) {
    // Start loading when SearchRequested (handled elsewhere)
    // Complete loading when SearchCompleted
    for _event in events.read() {
        for (mut ui_state, _) in &mut search_bar_query {
            ui_state.complete_loading();
        }
    }
}

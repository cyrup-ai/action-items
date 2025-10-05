use action_items_ui::prelude::{
    ApplyHotkeyButton, HotkeyInputField, PreferencesCloseButton, TestHotkeyButton,
};
use bevy::prelude::*;

use crate::events::PreferencesEvent;
use action_items_ecs_preferences::PreferencesResource;

/// System to handle preferences UI interactions - REAL implementation
/// Zero allocation UI interaction processing for blazing-fast response
#[inline]
pub fn handle_preferences_ui_interactions(
    mut prefs_events: EventWriter<PreferencesEvent>,
    close_button_query: Query<&Interaction, (Changed<Interaction>, With<PreferencesCloseButton>)>,
    input_field_query: Query<&Interaction, (Changed<Interaction>, With<HotkeyInputField>)>,
    test_button_query: Query<&Interaction, (Changed<Interaction>, With<TestHotkeyButton>)>,
    apply_button_query: Query<&Interaction, (Changed<Interaction>, With<ApplyHotkeyButton>)>,
    prefs_state: Res<PreferencesResource>,
) {
    // Handle close button - zero allocation interaction processing
    for interaction in close_button_query.iter() {
        if *interaction == Interaction::Pressed {
            prefs_events.write(PreferencesEvent::Close);
        }
    }

    // Handle input field clicks to start capture - REAL functionality
    for interaction in input_field_query.iter() {
        if *interaction == Interaction::Pressed && !prefs_state.capturing {
            prefs_events.write(PreferencesEvent::StartCapture);
        }
    }

    // Handle test button - REAL functionality with zero allocation
    for interaction in test_button_query.iter() {
        if *interaction == Interaction::Pressed
            && let Some(hotkey_def) = &prefs_state.captured_hotkey
        {
            prefs_events.write(PreferencesEvent::TestHotkey(hotkey_def.clone()));
        }
    }

    // Handle apply button - REAL functionality with zero allocation
    for interaction in apply_button_query.iter() {
        if *interaction == Interaction::Pressed
            && let Some(hotkey_def) = &prefs_state.captured_hotkey
        {
            prefs_events.write(PreferencesEvent::ApplyHotkey(hotkey_def.clone()));
        }
    }
}

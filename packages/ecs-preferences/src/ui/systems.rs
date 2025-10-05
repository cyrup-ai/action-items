use bevy::prelude::*;
use crate::{events::*, resources::*};
use crate::ui::components::*;
use crate::ui::screens::*;
use action_items_ecs_ui::theme::Theme;

/// Manage preferences window visibility
pub fn manage_preferences_visibility(
    mut commands: Commands,
    mut show_events: EventReader<PreferencesShowRequested>,
    mut hide_events: EventReader<PreferencesHideRequested>,
    mut resource: ResMut<PreferencesResource>,
    existing_windows: Query<Entity, With<PreferencesContainer>>,
    theme: Res<Theme>,
) {
    // Handle show requests
    for _ in show_events.read() {
        if !resource.is_visible {
            resource.is_visible = true;
            let window = create_preferences_window(&mut commands, &theme);
            commands.entity(window).insert(Visibility::Visible);
        }
    }
    
    // Handle hide requests
    for _ in hide_events.read() {
        if resource.is_visible {
            resource.is_visible = false;
            for entity in existing_windows.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Handle close button clicks
pub fn handle_close_button(
    button_query: Query<&Interaction, (With<PreferencesCloseButton>, Changed<Interaction>)>,
    mut hide_events: EventWriter<PreferencesHideRequested>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            hide_events.write(PreferencesHideRequested);
        }
    }
}

/// Handle recorder button clicks
pub fn handle_recorder_button(
    button_query: Query<&Interaction, (With<HotkeyRecorderButton>, Changed<Interaction>)>,
    mut recording_events: EventWriter<HotkeyRecordingStarted>,
    mut resource: ResMut<PreferencesResource>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            resource.capturing = true;
            recording_events.write(HotkeyRecordingStarted);
        }
    }
}

/// Handle save button clicks
pub fn handle_save_button(
    button_query: Query<&Interaction, (With<PreferencesSaveButton>, Changed<Interaction>)>,
    mut save_events: EventWriter<PreferencesSaveRequested>,
    resource: Res<PreferencesResource>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Some(ref hotkey) = resource.captured_hotkey {
                save_events.write(PreferencesSaveRequested {
                    hotkey: hotkey.clone(),
                });
            }
        }
    }
}

/// Handle cancel button clicks
pub fn handle_cancel_button(
    button_query: Query<&Interaction, (With<PreferencesCancelButton>, Changed<Interaction>)>,
    mut hide_events: EventWriter<PreferencesHideRequested>,
) {
    for interaction in button_query.iter() {
        if *interaction == Interaction::Pressed {
            hide_events.write(PreferencesHideRequested);
        }
    }
}

/// Update hotkey display text
pub fn update_hotkey_display(
    resource: Res<PreferencesResource>,
    mut display_query: Query<(&mut Text, &mut HotkeyDisplay)>,
) {
    if !resource.is_changed() {
        return;
    }
    
    for (mut text, mut display) in display_query.iter_mut() {
        let hotkey_text = match &resource.captured_hotkey {
            Some(hotkey) => format!("Current: {:?}+{:?}", hotkey.modifiers, hotkey.code),
            None => "Current: None".to_string(),
        };
        
        **text = hotkey_text.clone();
        display.current_hotkey = Some(hotkey_text);
    }
}

/// Show/hide recording overlay
pub fn manage_recording_overlay(
    mut commands: Commands,
    resource: Res<PreferencesResource>,
    mut last_capturing: Local<bool>,
    overlay_query: Query<Entity, With<HotkeyRecordingOverlay>>,
    theme: Res<Theme>,
) {
    if resource.capturing != *last_capturing {
        if resource.capturing {
            // Show overlay
            let overlay = create_recording_overlay(&mut commands, &theme);
            commands.entity(overlay).insert(Visibility::Visible);
        } else {
            // Hide overlay
            for entity in overlay_query.iter() {
                commands.entity(entity).despawn();
            }
        }
        *last_capturing = resource.capturing;
    }
}

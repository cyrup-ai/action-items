use bevy::prelude::*;
use crate::{events::*, resources::*};
use global_hotkey::hotkey::Modifiers;

/// Process save requests
pub fn process_save_requests(
    mut events: EventReader<PreferencesSaveRequested>,
    mut saved_events: EventWriter<PreferencesSaved>,
    mut resource: ResMut<PreferencesResource>,
) {
    for event in events.read() {
        // Store the hotkey (persistence handled separately)
        resource.captured_hotkey = Some(event.hotkey.clone());
        resource.last_save_success = Some(std::time::SystemTime::now());
        saved_events.write(PreferencesSaved);
    }
}

/// Process recording started events
pub fn process_recording_started(
    mut events: EventReader<HotkeyRecordingStarted>,
    mut resource: ResMut<PreferencesResource>,
) {
    for _ in events.read() {
        resource.capturing = true;
        resource.held_modifiers = Modifiers::empty();
        resource.captured_key = None;
    }
}

/// Process recording cancelled events
pub fn process_recording_cancelled(
    mut events: EventReader<HotkeyRecordingCancelled>,
    mut resource: ResMut<PreferencesResource>,
) {
    for _ in events.read() {
        resource.capturing = false;
        resource.held_modifiers = Modifiers::empty();
        resource.captured_key = None;
    }
}

/// Process hotkey recorded events
pub fn process_hotkey_recorded(
    mut events: EventReader<HotkeyRecorded>,
    mut resource: ResMut<PreferencesResource>,
) {
    for event in events.read() {
        resource.captured_hotkey = Some(event.hotkey.clone());
        resource.capturing = false;
        
        if event.has_conflict {
            resource.current_status = HotkeyStatus::Conflict(
                event.conflict_with.clone().unwrap_or_default()
            );
        } else {
            resource.current_status = HotkeyStatus::Valid;
        }
    }
}

use bevy::prelude::*;
use crate::{resources::*, events::*, systems::*};

/// Core preferences plugin (headless)
pub struct PreferencesPlugin;

impl Plugin for PreferencesPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.init_resource::<PreferencesResource>();
        
        // Events
        app.add_event::<PreferencesShowRequested>()
            .add_event::<PreferencesHideRequested>()
            .add_event::<PreferencesVisibilityChanged>()
            .add_event::<PreferencesSaveRequested>()
            .add_event::<PreferencesSaved>()
            .add_event::<HotkeyRecordingStarted>()
            .add_event::<HotkeyRecorded>()
            .add_event::<HotkeyRecordingCancelled>();
        
        // Systems
        app.add_systems(Update, (
            process_save_requests,
            process_recording_started,
            process_recording_cancelled,
            process_hotkey_recorded,
        ));
    }
}

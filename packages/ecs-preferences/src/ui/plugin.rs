use bevy::prelude::*;
use crate::ui::systems::*;

/// Preferences UI plugin
#[derive(Default)]
pub struct PreferencesUIPlugin;

impl Plugin for PreferencesUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            manage_preferences_visibility,
            handle_close_button,
            handle_recorder_button,
            handle_save_button,
            handle_cancel_button,
            update_hotkey_display,
            manage_recording_overlay,
        ));
    }
}

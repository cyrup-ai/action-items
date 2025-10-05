use action_items_ui::{UiComponentTarget, UiVisibilityEvent};
use bevy::prelude::*;
use tracing::info;

use crate::events::PreferencesEvent;

/// Bridge system to connect PreferencesEvent to UiVisibilityEvent
/// This system listens for preferences events and translates them to settings UI events
pub fn bridge_preferences_to_settings(
    mut preferences_events: EventReader<PreferencesEvent>,
    mut ui_visibility_events: EventWriter<UiVisibilityEvent>,
) {
    for event in preferences_events.read() {
        match event {
            PreferencesEvent::Open => {
                ui_visibility_events.write(UiVisibilityEvent::immediate(true, UiComponentTarget::Panel));
                info!("🔗 Bridged PreferencesEvent::Open -> UiVisibilityEvent (Settings visible)");
            },
            PreferencesEvent::Close => {
                ui_visibility_events.write(UiVisibilityEvent::immediate(false, UiComponentTarget::Panel));
                info!("🔗 Bridged PreferencesEvent::Close -> UiVisibilityEvent (Settings hidden)");
            },
            _ => {
                // Other preference events don't affect settings visibility
            },
        }
    }
}

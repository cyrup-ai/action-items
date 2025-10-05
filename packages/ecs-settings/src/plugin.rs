use bevy::prelude::*;
use crate::{resources::*, events::*, systems::*, persistence};

/// Settings management plugin
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.init_resource::<SettingsResource>();
        
        // Events
        app.add_event::<TabChangeRequested>()
            .add_event::<TabChanged>()
            .add_event::<SettingUpdateRequested>()
            .add_event::<SettingUpdated>()
            .add_event::<SettingValidationFailed>()
            .add_event::<SearchQueryChanged>()
            .add_event::<FilterChanged>()
            .add_event::<ExtensionSelected>()
            .add_event::<ExtensionToggled>()
            .add_event::<ExtensionConfigChanged>()
            .add_event::<OpenExtensionStore>()
            // NEW modal events
            .add_event::<SettingsOpenRequested>()
            .add_event::<SettingsCloseRequested>()
            .add_event::<SettingsWindowOpened>()
            .add_event::<SettingsWindowClosed>()
            // Keep for backward compatibility
            .add_event::<SettingsVisibilityEvent>();
        
        // Load settings on startup
        app.add_systems(PostStartup, (
            persistence::load_settings_on_startup,
            persistence::apply_loaded_settings,
        ).chain());
        
        // Systems
        app.add_systems(Update, (
            process_tab_changes,
            process_setting_updates,
            process_search_changes,
            process_filter_changes,
            process_extension_selection,
            process_visibility_events,  // Keep for backward compatibility
            handle_settings_open,        // NEW
            handle_settings_close,       // NEW
            handle_close_button,         // NEW
            persistence::apply_loaded_settings,  // Also apply during runtime
        ));
    }
}

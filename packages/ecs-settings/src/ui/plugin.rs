use bevy::prelude::*;
use crate::navigation::SettingsTab;
use super::{screens::*, systems::*};

pub struct SettingsUIPlugin {
    pub default_tab: SettingsTab,
    pub show_on_startup: bool,
}

impl Default for SettingsUIPlugin {
    fn default() -> Self {
        Self {
            default_tab: SettingsTab::General,
            show_on_startup: false,
        }
    }
}

impl Plugin for SettingsUIPlugin {
    fn build(&self, app: &mut App) {
        app
            // REPLACE setup_settings_ui with infrastructure setup
            .add_systems(Startup, setup_settings_infrastructure)
            .add_systems(Update, (
                handle_tab_clicks,
                update_tab_states,
                switch_tab_visibility,          // REPLACES update_content_area
                handle_keyboard_tab_navigation, // NEW
                handle_escape_close,            // NEW
                handle_checkbox_changes,
                handle_text_input_changes,
                handle_dropdown_changes,
                populate_extension_table,
                handle_extension_toggle,
                handle_extension_search,
                handle_extension_filters,
                handle_extension_store_button,
                handle_extension_settings_button,
                display_setting_errors,
                auto_hide_errors,
                setting_save_feedback,
                animate_save_feedback,
                handle_window_mode_selection,
                handle_theme_studio_click,
            ));
    }
}

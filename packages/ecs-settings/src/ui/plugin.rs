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
            // Core UI systems
            .add_systems(Update, (
                handle_tab_clicks,
                update_tab_states,
                switch_tab_visibility,
                handle_keyboard_tab_navigation,
                handle_escape_close,
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
            ))
            // Button click handlers
            .add_systems(Update, (
                handle_theme_studio_click,
                handle_log_out_click,
                handle_manage_subscription_click,
                handle_visit_website_click,
                handle_send_feedback_click,
                handle_acknowledgements_click,
                handle_org_selection_click,
                handle_manage_org_click,
                handle_edit_org_click,
                handle_open_store_click,
                handle_leave_org_click,
                handle_manage_org_subscription_click,
                handle_create_org_click,
            ));
    }
}

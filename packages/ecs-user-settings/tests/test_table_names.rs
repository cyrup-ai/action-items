//! Tests for table_names.rs

use action_items_ecs_user_settings::table_names::*;
use action_items_ecs_user_settings::types::VALID_TABLES;

#[test]
fn all_constants_match_valid_tables() {
    // Ensure all const values are in VALID_TABLES
    let constants = [
        STARTUP_SETTINGS,
        APPEARANCE_SETTINGS,
        AI_SETTINGS,
        CLOUD_SYNC_SETTINGS,
        ACCOUNT_SETTINGS,
        ORGANIZATION_SETTINGS,
        ADVANCED_SETTINGS,
        HOTKEY_SETTINGS,
        USER_PREFERENCES,
        PLUGIN_CONFIGS,
        UI_STATE,
        SETTINGS_HISTORY,
    ];

    for constant in &constants {
        assert!(
            VALID_TABLES.contains(constant),
            "Constant '{}' not in VALID_TABLES",
            constant
        );
    }
}

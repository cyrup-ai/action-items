//! Type-safe table name constants
//!
//! These constants provide compile-time verification that table names
//! are correct. The compiler will catch typos immediately.

/// Startup and system behavior settings
pub const STARTUP_SETTINGS: &str = "startup_settings";

/// Visual appearance and theme settings
pub const APPEARANCE_SETTINGS: &str = "appearance_settings";

/// AI assistant configuration
pub const AI_SETTINGS: &str = "ai_settings";

/// Cloud synchronization preferences
pub const CLOUD_SYNC_SETTINGS: &str = "cloud_sync_settings";

/// User account and subscription settings
pub const ACCOUNT_SETTINGS: &str = "account_settings";

/// Organization and team settings
pub const ORGANIZATION_SETTINGS: &str = "organization_settings";

/// Advanced user preferences
pub const ADVANCED_SETTINGS: &str = "advanced_settings";

/// Hotkey and keyboard shortcut settings
pub const HOTKEY_SETTINGS: &str = "hotkey_settings";

/// Generic user preferences (avoid using in new code)
pub const USER_PREFERENCES: &str = "user_preferences";

/// Plugin-specific configuration (managed by plugins)
pub const PLUGIN_CONFIGS: &str = "plugin_configs";

/// UI state and layout (managed by ecs-ui)
pub const UI_STATE: &str = "ui_state";

/// Audit trail (read-only, managed by system)
pub const SETTINGS_HISTORY: &str = "settings_history";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VALID_TABLES;

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
}

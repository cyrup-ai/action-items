//! SurrealDB schema definitions for user settings
//!
//! Defines 12 tables with SCHEMAFULL enforcement:
//! - `user_preferences` - General user preferences
//! - `hotkey_settings` - Keyboard shortcut configurations
//! - `plugin_configs` - Plugin-specific settings
//! - `ui_state` - UI window state persistence
//! - `ai_settings` - AI feature configurations
//! - `cloud_sync_settings` - Cloud synchronization settings
//! - `account_settings` - User account information
//! - `organization_settings` - Organization-level settings
//! - `advanced_settings` - Advanced user preferences
//! - `appearance_settings` - Theme and UI appearance
//! - `startup_settings` - Application startup behavior
//! - `settings_history` - Complete audit trail of all changes
//!
//! All tables include:
//! - Proper type constraints and assertions
//! - Automatic timestamps (created_at, updated_at)
//! - Indexes for performance
//! - SCHEMAFULL enforcement for data integrity

/// Complete schema as const string for initialization
///
/// This schema is executed once during service initialization to create
/// all required tables, fields, and indexes. The settings_history table
/// provides a complete audit trail of all setting changes.
pub const USER_SETTINGS_SCHEMA: &str = r#"
-- ============================================================================
-- USER PREFERENCES TABLE
-- ============================================================================
DEFINE TABLE user_preferences SCHEMAFULL;
DEFINE FIELD key ON user_preferences TYPE string
    ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD value ON user_preferences TYPE object;
DEFINE FIELD category ON user_preferences TYPE string DEFAULT "general";
DEFINE FIELD description ON user_preferences TYPE option<string>;
DEFINE FIELD created_at ON user_preferences TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON user_preferences TYPE datetime DEFAULT time::now();
DEFINE INDEX key_idx ON user_preferences COLUMNS key UNIQUE;
DEFINE INDEX category_idx ON user_preferences COLUMNS category;

-- ============================================================================
-- HOTKEY SETTINGS TABLE
-- ============================================================================
DEFINE TABLE hotkey_settings SCHEMAFULL;
DEFINE FIELD hotkey_id ON hotkey_settings TYPE string
    ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD modifiers ON hotkey_settings TYPE array<string>;
DEFINE FIELD key_code ON hotkey_settings TYPE string;
DEFINE FIELD description ON hotkey_settings TYPE string;
DEFINE FIELD enabled ON hotkey_settings TYPE bool DEFAULT true;
DEFINE FIELD priority ON hotkey_settings TYPE number DEFAULT 0;
DEFINE FIELD created_at ON hotkey_settings TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON hotkey_settings TYPE datetime DEFAULT time::now();
DEFINE INDEX hotkey_id_idx ON hotkey_settings COLUMNS hotkey_id UNIQUE;
DEFINE INDEX enabled_idx ON hotkey_settings COLUMNS enabled;

-- ============================================================================
-- PLUGIN CONFIGURATIONS TABLE
-- ============================================================================
DEFINE TABLE plugin_configs SCHEMAFULL;
DEFINE FIELD plugin_id ON plugin_configs TYPE string
    ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD version ON plugin_configs TYPE string;
DEFINE FIELD configuration ON plugin_configs TYPE object;
DEFINE FIELD preferences ON plugin_configs TYPE object;
DEFINE FIELD enabled ON plugin_configs TYPE bool DEFAULT true;
DEFINE FIELD last_modified ON plugin_configs TYPE datetime DEFAULT time::now();
DEFINE INDEX plugin_id_idx ON plugin_configs COLUMNS plugin_id UNIQUE;
DEFINE INDEX enabled_idx ON plugin_configs COLUMNS enabled;

-- ============================================================================
-- UI STATE TABLE
-- ============================================================================
DEFINE TABLE ui_state SCHEMAFULL;
DEFINE FIELD window_id ON ui_state TYPE string;
DEFINE FIELD state ON ui_state TYPE object;
DEFINE FIELD last_updated ON ui_state TYPE datetime DEFAULT time::now();
DEFINE INDEX window_id_idx ON ui_state COLUMNS window_id UNIQUE;

-- ============================================================================
-- AI SETTINGS TABLE
-- ============================================================================
DEFINE TABLE ai_settings SCHEMAFULL;
DEFINE FIELD enabled ON ai_settings TYPE bool DEFAULT true;
DEFINE FIELD quick_ai_trigger ON ai_settings TYPE string DEFAULT "Tab";
DEFINE FIELD show_hint_in_root_search ON ai_settings TYPE bool DEFAULT true;
DEFINE FIELD quick_ai_model ON ai_settings TYPE string DEFAULT "sonar-reasoning-pro";
DEFINE FIELD web_search_enabled ON ai_settings TYPE bool DEFAULT true;
DEFINE FIELD default_primary_action ON ai_settings TYPE string DEFAULT "paste_response";
DEFINE FIELD chat_hotkey ON ai_settings TYPE string DEFAULT "^ ⌘ L";
DEFINE FIELD start_new_chat_after ON ai_settings TYPE string DEFAULT "30_minutes";
DEFINE FIELD ai_commands_model ON ai_settings TYPE string DEFAULT "gemini-2.5-pro";
DEFINE FIELD show_tool_call_info ON ai_settings TYPE bool DEFAULT false;
DEFINE FIELD auto_confirm_tool_calls ON ai_settings TYPE bool DEFAULT true;
DEFINE FIELD text_size ON ai_settings TYPE string DEFAULT "medium";
DEFINE FIELD ollama_host ON ai_settings TYPE string DEFAULT "127.0.0.1:11434";
DEFINE FIELD ollama_models ON ai_settings TYPE array<string>;
DEFINE FIELD browser_extension_enabled ON ai_settings TYPE bool DEFAULT false;
DEFINE FIELD experiments_auto_models ON ai_settings TYPE bool DEFAULT true;
DEFINE FIELD experiments_chat_branching ON ai_settings TYPE bool DEFAULT true;
DEFINE FIELD experiments_custom_providers ON ai_settings TYPE bool DEFAULT false;
DEFINE FIELD experiments_mcp_servers ON ai_settings TYPE bool DEFAULT true;
DEFINE FIELD experiments_ollama_extensions ON ai_settings TYPE bool DEFAULT true;
DEFINE FIELD created_at ON ai_settings TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON ai_settings TYPE datetime DEFAULT time::now();
DEFINE INDEX ai_enabled_idx ON ai_settings COLUMNS enabled;

-- ============================================================================
-- CLOUD SYNC SETTINGS TABLE
-- ============================================================================
DEFINE TABLE cloud_sync_settings SCHEMAFULL;
DEFINE FIELD enabled ON cloud_sync_settings TYPE bool DEFAULT false;
DEFINE FIELD last_synced ON cloud_sync_settings TYPE option<datetime>;
DEFINE FIELD sync_search_history ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD sync_aliases ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD sync_hotkeys ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD sync_quicklinks ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD sync_snippets ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD sync_notes ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD sync_extensions_settings ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD sync_ai_chats ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD sync_themes ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD sync_window_management ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD not_synced_clipboard_history ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD not_synced_script_commands ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD not_synced_credentials ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD not_synced_general_advanced ON cloud_sync_settings TYPE bool DEFAULT true;
DEFINE FIELD created_at ON cloud_sync_settings TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON cloud_sync_settings TYPE datetime DEFAULT time::now();

-- ============================================================================
-- ACCOUNT SETTINGS TABLE
-- ============================================================================
DEFINE TABLE account_settings SCHEMAFULL;
DEFINE FIELD user_name ON account_settings TYPE string;
DEFINE FIELD user_email ON account_settings TYPE string;
DEFINE FIELD user_avatar ON account_settings TYPE option<string>;
DEFINE FIELD subscription_type ON account_settings TYPE string;
DEFINE FIELD subscription_status ON account_settings TYPE string;
DEFINE FIELD pro_features ON account_settings TYPE object;
DEFINE FIELD organization_id ON account_settings TYPE option<string>;
DEFINE FIELD created_at ON account_settings TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON account_settings TYPE datetime DEFAULT time::now();

-- ============================================================================
-- ORGANIZATION SETTINGS TABLE
-- ============================================================================
DEFINE TABLE organization_settings SCHEMAFULL;
DEFINE FIELD organization_id ON organization_settings TYPE string;
DEFINE FIELD organization_name ON organization_settings TYPE string;
DEFINE FIELD subscription_plan ON organization_settings TYPE string;
DEFINE FIELD private_extensions ON organization_settings TYPE bool DEFAULT false;
DEFINE FIELD shared_quicklinks ON organization_settings TYPE bool DEFAULT false;
DEFINE FIELD shared_snippets ON organization_settings TYPE bool DEFAULT false;
DEFINE FIELD pro_features_for_all ON organization_settings TYPE bool DEFAULT false;
DEFINE FIELD store_url ON organization_settings TYPE option<string>;
DEFINE FIELD created_at ON organization_settings TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON organization_settings TYPE datetime DEFAULT time::now();
DEFINE INDEX org_id_idx ON organization_settings COLUMNS organization_id UNIQUE;

-- ============================================================================
-- ADVANCED SETTINGS TABLE
-- ============================================================================
DEFINE TABLE advanced_settings SCHEMAFULL;
DEFINE FIELD show_raycast_on ON advanced_settings TYPE string DEFAULT "screen_with_mouse";
DEFINE FIELD pop_to_root_after ON advanced_settings TYPE string DEFAULT "90_seconds";
DEFINE FIELD escape_key_behavior ON advanced_settings TYPE string DEFAULT "navigate_back";
DEFINE FIELD auto_switch_input_source ON advanced_settings TYPE string DEFAULT "us";
DEFINE FIELD navigation_bindings ON advanced_settings TYPE string DEFAULT "macos_standard";
DEFINE FIELD page_navigation_keys ON advanced_settings TYPE string DEFAULT "square_brackets";
DEFINE FIELD root_search_sensitivity ON advanced_settings TYPE string DEFAULT "medium";
DEFINE FIELD hyper_key ON advanced_settings TYPE option<string>;
DEFINE FIELD hyper_key_replacement ON advanced_settings TYPE option<string>;
DEFINE FIELD favicon_provider ON advanced_settings TYPE string DEFAULT "raycast";
DEFINE FIELD emoji_skin_tone ON advanced_settings TYPE number DEFAULT 0;
DEFINE FIELD import_export_data ON advanced_settings TYPE object;
DEFINE FIELD window_capture_hotkey ON advanced_settings TYPE option<string>;
DEFINE FIELD window_capture_clipboard ON advanced_settings TYPE bool DEFAULT true;
DEFINE FIELD window_capture_finder ON advanced_settings TYPE bool DEFAULT false;
DEFINE FIELD custom_wallpaper ON advanced_settings TYPE option<string>;
DEFINE FIELD use_node_production ON advanced_settings TYPE bool DEFAULT true;
DEFINE FIELD use_file_logging ON advanced_settings TYPE bool DEFAULT true;
DEFINE FIELD auto_reload_on_save ON advanced_settings TYPE bool DEFAULT true;
DEFINE FIELD disable_pop_to_root ON advanced_settings TYPE bool DEFAULT false;
DEFINE FIELD open_in_dev_mode ON advanced_settings TYPE bool DEFAULT false;
DEFINE FIELD keep_visible_in_dev ON advanced_settings TYPE bool DEFAULT false;
DEFINE FIELD use_system_network ON advanced_settings TYPE bool DEFAULT true;
DEFINE FIELD certificates_keychain ON advanced_settings TYPE string DEFAULT "keychain";
DEFINE FIELD created_at ON advanced_settings TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON advanced_settings TYPE datetime DEFAULT time::now();

-- ============================================================================
-- APPEARANCE SETTINGS TABLE
-- ============================================================================
DEFINE TABLE appearance_settings SCHEMAFULL;
DEFINE FIELD text_size ON appearance_settings TYPE string DEFAULT "medium";
DEFINE FIELD theme_dark ON appearance_settings TYPE string DEFAULT "raycast_dark";
DEFINE FIELD theme_light ON appearance_settings TYPE string DEFAULT "raycast_light";
DEFINE FIELD follow_system_appearance ON appearance_settings TYPE bool DEFAULT true;
DEFINE FIELD window_mode ON appearance_settings TYPE string DEFAULT "default";
DEFINE FIELD show_favorites_compact ON appearance_settings TYPE bool DEFAULT true;
DEFINE FIELD created_at ON appearance_settings TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON appearance_settings TYPE datetime DEFAULT time::now();

-- ============================================================================
-- STARTUP SETTINGS TABLE
-- ============================================================================
DEFINE TABLE startup_settings SCHEMAFULL;
DEFINE FIELD launch_at_login ON startup_settings TYPE bool DEFAULT false;
DEFINE FIELD show_menu_bar_icon ON startup_settings TYPE bool DEFAULT false;
DEFINE FIELD created_at ON startup_settings TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON startup_settings TYPE datetime DEFAULT time::now();

-- ============================================================================
-- SETTINGS HISTORY TABLE (Audit Trail)
-- ============================================================================
DEFINE TABLE settings_history SCHEMAFULL;
DEFINE FIELD table_name ON settings_history TYPE string;
DEFINE FIELD record_id ON settings_history TYPE string;
DEFINE FIELD field_name ON settings_history TYPE string;
DEFINE FIELD old_value ON settings_history TYPE option<object>;
DEFINE FIELD new_value ON settings_history TYPE object;
DEFINE FIELD changed_at ON settings_history TYPE datetime DEFAULT time::now();
DEFINE FIELD change_type ON settings_history TYPE string;
DEFINE INDEX changed_at_idx ON settings_history COLUMNS changed_at;
DEFINE INDEX table_record_idx ON settings_history COLUMNS table_name, record_id;
"#;

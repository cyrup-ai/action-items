/// Extism plugin interface (for WASM plugins)
///
/// Functions that Extism plugins must export
pub const REQUIRED_EXPORTS: &[&str] = &[
    "plugin_manifest",
    "plugin_initialize",
    "plugin_search",
    "plugin_execute_command",
    "plugin_execute_action",
];

/// Optional exports
pub const OPTIONAL_EXPORTS: &[&str] = &[
    "plugin_background_refresh",
    "plugin_cleanup",
    "plugin_handle_deep_link",
    "plugin_handle_notification",
];

/// Host functions provided to plugins
pub const HOST_FUNCTIONS: &[&str] = &[
    // Storage
    "storage_get",
    "storage_set",
    "storage_delete",
    "storage_list",
    // Cache
    "cache_get",
    "cache_set",
    "cache_delete",
    "cache_clear",
    // Clipboard
    "clipboard_read",
    "clipboard_write",
    "clipboard_clear",
    // Notifications
    "notification_show",
    "notification_clear",
    "notification_clear_all",
    // HTTP
    "http_get",
    "http_post",
    "http_put",
    "http_delete",
    // System
    "system_open_url",
    "system_reveal_file",
    "system_get_env",
    "system_exec_command",
    // UI
    "ui_show_hud",
    "ui_show_toast",
    "ui_close_window",
    "ui_refresh_view",
];

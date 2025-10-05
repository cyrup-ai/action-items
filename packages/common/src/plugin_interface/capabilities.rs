use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginCapabilities {
    pub search: bool,
    pub background_refresh: bool,
    pub notifications: bool,
    pub shortcuts: bool,
    pub deep_links: bool,
    pub clipboard_access: bool,
    pub file_system_access: bool,
    pub network_access: bool,
    pub system_commands: bool,
    pub ui_extensions: bool,
    pub context_menu: bool,
    pub quick_actions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginPermissions {
    pub read_clipboard: bool,
    pub write_clipboard: bool,
    pub read_files: Vec<PathBuf>,
    pub write_files: Vec<PathBuf>,
    pub execute_commands: Vec<String>,
    pub network_hosts: Vec<String>,
    pub environment_variables: Vec<String>,
    pub system_notifications: bool,
    pub accessibility: bool,
    pub camera: bool,
    pub microphone: bool,
    pub location: bool,
    pub contacts: bool,
    pub calendar: bool,
}

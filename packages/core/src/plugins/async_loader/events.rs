use std::path::PathBuf;

use bevy::prelude::*;

/// Event fired when plugin loading starts
#[derive(Event)]
pub struct PluginLoadingStarted {
    pub total_plugins: usize,
}

/// Event fired when a plugin is loaded successfully
#[derive(Event)]
pub struct PluginLoaded {
    pub plugin_id: String,
    pub plugin_name: String,
}

/// Event fired when a plugin fails to load
#[derive(Event)]
pub struct PluginLoadFailed {
    pub path: PathBuf,
    pub error: String,
}

/// Event fired when all plugin loading is complete
#[derive(Event)]
pub struct PluginLoadingComplete {
    pub loaded_count: usize,
    pub failed_count: usize,
}

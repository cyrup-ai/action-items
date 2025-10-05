use action_items_common::plugin_interface::PluginManifest;
use bevy::prelude::*;

/// Events for plugin lifecycle
#[derive(Event)]
pub struct PluginLoadedEvent {
    pub plugin_id: String,
    pub manifest: PluginManifest,
}

#[derive(Event)]
pub struct PluginUnloadedEvent {
    pub plugin_id: String,
    pub reason: String,
}

#[derive(Event)]
pub struct PluginErrorEvent {
    pub plugin_id: String,
    pub error: String,
    pub recoverable: bool,
}

#[derive(Event)]
pub struct PluginBackgroundTaskCompleted {
    pub plugin_id: String,
    pub task_id: String,
    pub result: Result<serde_json::Value, String>,
}

pub mod performance;
use std::collections::HashMap;

use bevy::prelude::*;
pub use performance::EventPerformanceMonitor;
use serde_json::Value;

/// Core launcher event for system-wide communication
#[derive(Event, Debug, Clone)]
pub struct LauncherEvent {
    pub event_type: LauncherEventType,
    pub data: Option<Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub enum LauncherEventType {
    PluginLoaded(String),
    PluginUnloaded(String),
    SearchStarted(String),
    SearchCompleted,
    ActionExecuted(String),
    SystemShutdown,
    Execute(String),
    // Additional variants needed for UI coordination
    ShowLauncher,
    HideLauncher,
    SearchQuery(String),
    ActionSelected(String),
    PreferencesOpened,
    PreferencesClosed,
}

impl LauncherEvent {
    pub fn new(event_type: LauncherEventType) -> Self {
        Self {
            event_type,
            data: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_data(event_type: LauncherEventType, data: Value) -> Self {
        Self {
            event_type,
            data: Some(data),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Event for WASM plugin callbacks
#[derive(Event, Debug, Clone)]
pub struct WasmCallbackEvent {
    pub plugin_id: String,
    pub callback_fn_name: String,
    pub request_id: String,
    pub result: Result<serde_json::Value, String>,
}

/// Resource for mapping action IDs to action items
#[derive(Resource, Default)]
pub struct ActionMap {
    actions: HashMap<String, action_items_common::plugin_interface::ActionItem>,
    reverse_lookup: HashMap<String, String>, // title -> id mapping for fast lookup
}

impl ActionMap {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            reverse_lookup: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        id: String,
        action: action_items_common::plugin_interface::ActionItem,
    ) {
        self.reverse_lookup.insert(action.title.clone(), id.clone());
        self.actions.insert(id, action);
    }

    pub fn get(&self, id: &str) -> Option<&action_items_common::plugin_interface::ActionItem> {
        self.actions.get(id)
    }

    pub fn get_by_title(
        &self,
        title: &str,
    ) -> Option<&action_items_common::plugin_interface::ActionItem> {
        self.reverse_lookup
            .get(title)
            .and_then(|id| self.actions.get(id))
    }

    pub fn remove(
        &mut self,
        id: &str,
    ) -> Option<action_items_common::plugin_interface::ActionItem> {
        if let Some(action) = self.actions.remove(id) {
            self.reverse_lookup.remove(&action.title);
            Some(action)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.actions.clear();
        self.reverse_lookup.clear();
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&String, &action_items_common::plugin_interface::ActionItem)> {
        self.actions.iter()
    }
}

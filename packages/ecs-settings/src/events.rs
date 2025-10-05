use bevy::prelude::*;
use uuid::Uuid;
use serde_json::Value;
use crate::navigation::{SettingsTab, ExtensionFilter};

pub type OperationId = Uuid;

// ========== TAB NAVIGATION ==========

/// Request to change settings tab
#[derive(Event, Debug, Clone)]
pub struct TabChangeRequested {
    pub operation_id: OperationId,
    pub from: SettingsTab,
    pub to: SettingsTab,
    pub requester: String,
}

impl TabChangeRequested {
    pub fn new(from: SettingsTab, to: SettingsTab, requester: impl Into<String>) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            from,
            to,
            requester: requester.into(),
        }
    }
}

/// Tab change completed
#[derive(Event, Debug, Clone)]
pub struct TabChanged {
    pub operation_id: OperationId,
    pub tab: SettingsTab,
}

// ========== SETTING VALUES ==========

/// Request to update a setting value
#[derive(Event, Debug, Clone)]
pub struct SettingUpdateRequested {
    pub operation_id: OperationId,
    pub tab: SettingsTab,
    pub table: String,
    pub field_name: String,
    pub new_value: Value,
    pub requester: String,
}

impl SettingUpdateRequested {
    pub fn new(
        tab: SettingsTab,
        table: impl Into<String>,
        field_name: impl Into<String>,
        new_value: Value,
        requester: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            tab,
            table: table.into(),
            field_name: field_name.into(),
            new_value,
            requester: requester.into(),
        }
    }
}

/// Setting update completed
#[derive(Event, Debug, Clone)]
pub struct SettingUpdated {
    pub operation_id: OperationId,
    pub tab: SettingsTab,
    pub field_name: String,
    pub old_value: Value,
    pub new_value: Value,
}

/// Setting validation failed
#[derive(Event, Debug, Clone)]
pub struct SettingValidationFailed {
    pub operation_id: OperationId,
    pub field_name: String,
    pub error: String,
}

// ========== SEARCH & FILTER ==========

/// Search query changed
#[derive(Event, Debug, Clone)]
pub struct SearchQueryChanged {
    pub operation_id: OperationId,
    pub query: String,
    pub requester: String,
}

impl SearchQueryChanged {
    pub fn new(query: impl Into<String>, requester: impl Into<String>) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            query: query.into(),
            requester: requester.into(),
        }
    }
}

/// Extension filter changed
#[derive(Event, Debug, Clone)]
pub struct FilterChanged {
    pub operation_id: OperationId,
    pub filter: ExtensionFilter,
    pub requester: String,
}

impl FilterChanged {
    pub fn new(filter: ExtensionFilter, requester: impl Into<String>) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            filter,
            requester: requester.into(),
        }
    }
}

// ========== EXTENSION MANAGEMENT ==========

/// Extension selected
#[derive(Event, Debug, Clone)]
pub struct ExtensionSelected {
    pub operation_id: OperationId,
    pub extension_id: String,
    pub requester: String,
}

impl ExtensionSelected {
    pub fn new(extension_id: impl Into<String>, requester: impl Into<String>) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            extension_id: extension_id.into(),
            requester: requester.into(),
        }
    }
}

/// Extension enabled/disabled
#[derive(Event, Debug, Clone)]
pub struct ExtensionToggled {
    pub operation_id: OperationId,
    pub extension_id: String,
    pub enabled: bool,
    pub requester: String,
}

impl ExtensionToggled {
    pub fn new(
        extension_id: impl Into<String>,
        enabled: bool,
        requester: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            extension_id: extension_id.into(),
            enabled,
            requester: requester.into(),
        }
    }
}

/// Extension configuration changed
#[derive(Event, Debug, Clone)]
pub struct ExtensionConfigChanged {
    pub operation_id: OperationId,
    pub extension_id: String,
    pub field_name: String,
    pub new_value: String,
    pub requester: String,
}

impl ExtensionConfigChanged {
    pub fn new(
        extension_id: impl Into<String>,
        field_name: impl Into<String>,
        new_value: impl Into<String>,
        requester: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            extension_id: extension_id.into(),
            field_name: field_name.into(),
            new_value: new_value.into(),
            requester: requester.into(),
        }
    }
}

// ========== EXTENSION STORE ==========

/// Extension store open event
#[derive(Event, Debug, Clone)]
pub struct OpenExtensionStore;

// ========== VISIBILITY ==========

/// Settings visibility event
#[derive(Event, Debug, Clone)]
pub enum SettingsVisibilityEvent {
    Show,
    Hide,
    Toggle,
}


// ========== MODAL OPEN/CLOSE (SPECIFIC EVENTS) ==========

/// Request to open settings window with optional initial tab
#[derive(Event, Debug, Clone)]
pub struct SettingsOpenRequested {
    pub operation_id: OperationId,
    pub initial_tab: Option<SettingsTab>,
}

impl SettingsOpenRequested {
    pub fn new(initial_tab: Option<SettingsTab>) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            initial_tab,
        }
    }
}

/// Request to close settings window
#[derive(Event, Debug, Clone)]
pub struct SettingsCloseRequested {
    pub operation_id: OperationId,
}

impl SettingsCloseRequested {
    pub fn new() -> Self {
        Self {
            operation_id: Uuid::new_v4(),
        }
    }
}

/// Settings window opened and visible
#[derive(Event, Debug, Clone)]
pub struct SettingsWindowOpened {
    pub operation_id: OperationId,
}

/// Settings window closed and hidden
#[derive(Event, Debug, Clone)]
pub struct SettingsWindowClosed {
    pub operation_id: OperationId,
}

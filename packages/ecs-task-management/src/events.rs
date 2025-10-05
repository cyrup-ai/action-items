//! Task management events following ARCHITECTURE.md request/response pattern

use std::path::PathBuf;

use bevy::prelude::*;
use uuid::Uuid;

use crate::components::*;

/// Events for hotkey preferences operations
#[derive(Event)]
pub struct HotkeyPreferencesLoadRequested {
    pub request_id: Uuid,
    pub config_path: Option<PathBuf>,
}

#[derive(Event)]
pub struct HotkeyPreferencesLoadCompleted {
    pub request_id: Uuid,
    pub result: Result<HotkeyPreferencesResult, String>,
}

#[derive(Event)]
pub struct HotkeyPreferencesPersistRequested {
    pub request_id: Uuid,
    pub preferences: HotkeyPreferencesResult,
    pub config_path: Option<PathBuf>,
}

#[derive(Event)]
pub struct HotkeyPreferencesPersistCompleted {
    pub request_id: Uuid,
    pub result: Result<PathBuf, String>,
}
/// Generic task management events
#[derive(Event)]
pub struct TaskSpawnedEvent {
    pub id: Uuid,
    pub operation_type: String,
}

#[derive(Event)]
pub struct TaskStarted {
    pub task_id: Uuid,
    pub operation_type: String,
}

#[derive(Event)]
pub struct TaskFailed {
    pub task_id: Uuid,
    pub operation_type: String,
    pub error: String,
}

#[derive(Event)]
pub struct TaskExpired {
    pub task_id: Uuid,
    pub operation_type: String,
    pub duration: std::time::Duration,
}

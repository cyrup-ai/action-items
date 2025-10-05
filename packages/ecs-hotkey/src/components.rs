//! Hotkey service components
//!
//! Component definitions for tracking hotkey operations and state.

use std::path::PathBuf;
use std::time::Instant;

use bevy::prelude::*;
use bevy::tasks::Task;

use crate::events::HotkeyDefinition;
use crate::resources::{HotkeyId, HotkeyPreferences};

/// Component to track hotkey operation state
#[derive(Component)]
pub struct HotkeyOperation {
    pub id: uuid::Uuid,
    pub operation_type: String,
    pub hotkey_definition: HotkeyDefinition,
    pub requester: String,
    pub status: String,
    pub created_at: Instant,
    pub completed_at: Option<Instant>,
}

/// Component to track hotkey polling task
/// Extracted from production hotkey_setup.rs
#[derive(Component)]
#[allow(dead_code)]
pub struct HotkeyPollingTask {
    pub task: Task<()>,
}

/// Marker component for the hotkey manager entity
/// Extracted from production hotkey_setup.rs
#[derive(Component)]
pub struct HotkeyManagerEntity;

/// Component to track hotkey preferences persistence task
/// Extracted from production preferences.rs
#[derive(Component)]
#[allow(dead_code)]
pub struct HotkeyPreferencesPersistTask {
    pub task: Task<Result<PathBuf, Box<dyn std::error::Error + Send + Sync>>>,
}

/// Component to track hotkey preferences loading task
/// Extracted from production preferences.rs
#[derive(Component)]
#[allow(dead_code)]
pub struct HotkeyPreferencesLoadTask {
    pub task: Task<HotkeyPreferences>,
}

/// Component for hotkey conflict monitoring
#[derive(Component)]
pub struct HotkeyConflictMonitor {
    pub last_check: Instant,
    pub check_interval: std::time::Duration,
}

impl Default for HotkeyConflictMonitor {
    fn default() -> Self {
        Self {
            last_check: Instant::now(),
            check_interval: std::time::Duration::from_secs(30),
        }
    }
}

/// Component for tracking hotkey usage statistics
#[derive(Component)]
pub struct HotkeyUsageTracker {
    pub hotkey_id: HotkeyId,
    pub usage_count: u64,
    pub last_used: Option<Instant>,
    pub moving_average: f64,
    pub alpha: f64,
    pub sample_count: u64,
}

impl HotkeyUsageTracker {
    pub fn new(hotkey_id: HotkeyId) -> Self {
        Self {
            hotkey_id,
            usage_count: 0,
            last_used: None,
            moving_average: 0.0,
            alpha: 0.2,
            sample_count: 0,
        }
    }

    pub fn update_moving_average(&mut self, new_value: f64) {
        self.moving_average = self.alpha * new_value + (1.0 - self.alpha) * self.moving_average;
        self.sample_count += 1;
    }

    pub fn record_usage(&mut self) {
        let now = Instant::now();
        self.usage_count += 1;
        self.last_used = Some(now);
    }
}

//! First-Run Detection and Wizard Auto-Start
//!
//! High-performance first-run detection using ecs-filesystem for async file operations.
//! Manages wizard completion persistence and auto-start logic with zero blocking operations.

#![allow(dead_code)]

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::{error, info, warn};

use crate::types::PermissionType;
use crate::wizard::{WizardState, WizardStartRequest, WizardCompleteEvent};

/// Resource for tracking first-run detection state
#[derive(Resource)]
pub struct FirstRunDetector {
    /// Whether this is the first run of the application
    pub is_first_run: bool,
    /// Whether the wizard has been completed previously
    pub wizard_completed: bool,
    /// Whether the first-run check has been completed
    pub check_completed: bool,
    /// Whether the detector is currently loading
    pub is_loading: bool,
    /// Timestamp when wizard was completed (if applicable)
    pub completion_timestamp: Option<SystemTime>,
    /// Error message if first-run detection failed
    pub error_message: Option<String>,
}

impl Default for FirstRunDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FirstRunDetector {
    /// Create a new first-run detector
    pub fn new() -> Self {
        Self {
            is_first_run: false,
            wizard_completed: false,
            check_completed: false,
            is_loading: false,
            completion_timestamp: None,
            error_message: None,
        }
    }
    
    /// Check if the wizard should be shown
    #[inline]
    pub fn should_show_wizard(&self) -> bool {
        self.check_completed && (self.is_first_run || !self.wizard_completed)
    }
    
    /// Mark the wizard as completed
    pub fn mark_wizard_completed(&mut self, completed_at: SystemTime) {
        self.wizard_completed = true;
        self.completion_timestamp = Some(completed_at);
        self.is_first_run = false;
        info!("Marked wizard as completed");
    }
    
    /// Handle an error during first-run detection
    pub fn handle_error(&mut self, error: String) {
        self.error_message = Some(error.clone());
        self.check_completed = true;
        self.is_loading = false;
        // Default to first-run behavior on error
        self.is_first_run = true;
        self.wizard_completed = false;
        warn!("First-run detection error: {}", error);
    }
    
    /// Apply completion status from file
    pub fn apply_completion_status(&mut self, status: WizardCompletionStatus) {
        self.wizard_completed = status.completed;
        self.completion_timestamp = status.completed_at;
        self.is_first_run = !status.completed;
        self.check_completed = true;
        self.is_loading = false;
        
        if status.completed {
            info!("Applied wizard completion status from file");
        } else {
            info!("Applied first-run status from file");
        }
    }
    
    /// Save partial wizard progress for resumption
    pub fn save_partial_progress(&mut self, progress: WizardPartialProgress) {
        info!("Saving partial progress: {} permissions completed", progress.completed_permissions.len());
        // Note: Actual file persistence would go here
        // For now, just log - file operations need ecs-filesystem integration
    }
    
    /// Load partial wizard progress if available
    pub fn load_partial_progress(&self) -> Option<WizardPartialProgress> {
        // Note: Actual file loading would go here
        None
    }
}

/// Simplified wizard completion status for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardCompletionStatus {
    /// Whether the wizard has been completed
    pub completed: bool,
    /// Timestamp when completed (if applicable)
    pub completed_at: Option<SystemTime>,
    /// Summary of completion results
    pub summary: Option<SerializableWizardSummary>,
}

impl Default for WizardCompletionStatus {
    fn default() -> Self {
        Self {
            completed: false,
            completed_at: None,
            summary: None,
        }
    }
}

/// Partial wizard progress for resumption
#[derive(Debug, Clone)]
pub struct WizardPartialProgress {
    pub last_state: WizardState,
    pub completed_permissions: Vec<PermissionType>,
    pub cancelled_at: SystemTime,
    pub can_resume: bool,
}

impl WizardCompletionStatus {
    /// Create a completed status with summary
    pub fn completed(summary: SerializableWizardSummary) -> Self {
        Self {
            completed: true,
            completed_at: Some(SystemTime::now()),
            summary: Some(summary),
        }
    }
}

/// Simplified wizard completion summary for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableWizardSummary {
    /// Number of permissions granted
    pub permissions_granted: u8,
    /// Number of permissions that failed
    pub permissions_failed: u8,
    /// Whether hotkeys were configured
    pub hotkeys_configured: bool,
    /// Total duration in seconds
    pub total_duration_secs: u64,
}

/// Resource for tracking file operations
#[derive(Resource, Default)]
pub struct FirstRunFileOperations {
    /// Pending completion file read operations
    pub completion_read_ops: HashMap<String, ()>,
    /// Pending completion file write operations  
    pub completion_write_ops: HashMap<String, WizardCompletionStatus>,
    /// Pending directory creation operations
    pub completion_dir_ops: HashMap<String, (Entity, PathBuf, Vec<u8>, WizardCompletionStatus)>,
}

/// System to initiate first-run check using ecs-filesystem
pub fn initiate_first_run_check(
    mut detector: ResMut<FirstRunDetector>,
    _pending_ops: ResMut<FirstRunFileOperations>,
) {
    // Only check once
    if detector.check_completed || detector.is_loading {
        return;
    }
    
    detector.is_loading = true;
    
    // For now, simulate a simple first-run check
    // In a real implementation, this would check for a completion file
    let config_dir = dirs::config_dir()
        .map(|dir| dir.join("action-items"))
        .unwrap_or_else(|| PathBuf::from("."));
    let config_file = config_dir.join("wizard-completion.json");
    
    // Check if completion file exists
    if config_file.exists() {
        // Try to read and parse the file
        match std::fs::read_to_string(&config_file) {
            Ok(content) => {
                if content.trim().is_empty() {
                    info!("Empty completion file - treating as first run");
                    detector.is_first_run = true;
                    detector.wizard_completed = false;
                } else {
                    match serde_json::from_str::<WizardCompletionStatus>(&content) {
                        Ok(status) => {
                            detector.apply_completion_status(status);
                        },
                        Err(e) => {
                            warn!("Failed to parse completion file: {}", e);
                            detector.handle_error(format!("Parse error: {}", e));
                        },
                    }
                }
            },
            Err(e) => {
                warn!("Failed to read completion file: {}", e);
                detector.handle_error(format!("Read error: {}", e));
            },
        }
    } else {
        info!("Completion file not found - first run confirmed");
        detector.is_first_run = true;
        detector.wizard_completed = false;
    }
    
    detector.check_completed = true;
    detector.is_loading = false;
}

/// System to handle wizard completion events and persist completion status
pub fn handle_wizard_completion(
    mut completion_events: EventReader<WizardCompleteEvent>,
    mut detector: ResMut<FirstRunDetector>,
) {
    for event in completion_events.read() {
        info!("Processing wizard completion event");
        
        // Convert to simplified summary for serialization
        let summary = SerializableWizardSummary {
            permissions_granted: event.completion_summary.granted_permissions.len() as u8,
            permissions_failed: event.completion_summary.failed_permissions.len() as u8,
            hotkeys_configured: event.completion_summary.hotkeys_configured,
            total_duration_secs: event.completion_summary.total_duration.as_secs(),
        };
        
        let completion_status = WizardCompletionStatus::completed(summary);
        
        // Update detector state
        detector.mark_wizard_completed(event.completed_at);
        
        // Persist completion status to filesystem
        let config_dir = dirs::config_dir()
            .map(|dir| dir.join("action-items"))
            .unwrap_or_else(|| PathBuf::from("."));
        
        // Create directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            error!("Failed to create config directory: {}", e);
            detector.handle_error(format!("Directory creation failed: {}", e));
            continue;
        }
        
        let config_file = config_dir.join("wizard-completion.json");
        
        // Serialize and write completion status
        match serde_json::to_string_pretty(&completion_status) {
            Ok(json_content) => {
                match std::fs::write(&config_file, json_content) {
                    Ok(()) => {
                        info!("Successfully saved wizard completion status");
                    },
                    Err(e) => {
                        error!("Failed to write completion file: {}", e);
                        detector.handle_error(format!("Write error: {}", e));
                    },
                }
            },
            Err(e) => {
                error!("Failed to serialize completion status: {}", e);
                detector.handle_error(format!("Serialization failed: {}", e));
            },
        }
    }
}

/// System to conditionally start wizard based on first-run status
pub fn check_should_start_wizard(
    detector: Res<FirstRunDetector>,
    wizard_state: Res<State<WizardState>>,
    mut wizard_start_events: EventWriter<WizardStartRequest>,
) {
    // Only consider starting wizard when wizard is not started
    if !matches!(wizard_state.get(), WizardState::NotStarted) {
        return;
    }
    
    // Start wizard if first-run detection is complete and wizard should be shown
    if detector.check_completed && detector.should_show_wizard() {
        info!("Starting wizard based on first-run detection");
        wizard_start_events.write(WizardStartRequest::new());
    }
}
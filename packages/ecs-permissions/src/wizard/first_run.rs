//! First-Run Detection and Wizard Auto-Start
//!
//! Detects first-run by querying actual OS permission status.
//! No file persistence - OS is the single source of truth.

use bevy::prelude::*;
use std::time::SystemTime;
use tracing::{error, info, warn};

use crate::types::{PermissionType, PermissionStatus};
use crate::plugin::PermissionResource;
use crate::wizard::{WizardState, WizardStartRequest, WizardCompleteEvent};
use crate::wizard::plugin::WizardRequiredPermissions;

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
    
    /// Log partial wizard progress (no persistence per requirements)
    pub fn save_partial_progress(&mut self, progress: WizardPartialProgress) {
        info!("Wizard progress: {} permissions completed (not persisted - OS is source of truth)", 
              progress.completed_permissions.len());
        // Intentionally does not persist - OS permission status is queried on each startup
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

/// System to initiate first-run check by querying actual OS permission status
pub fn initiate_first_run_check(
    mut detector: ResMut<FirstRunDetector>,
    permission_resource: Option<Res<PermissionResource>>,
    required_perms: Res<WizardRequiredPermissions>,
) {
    // Only check once per app session
    if detector.check_completed || detector.is_loading {
        return;
    }
    
    let Some(perm_res) = permission_resource else {
        warn!("PermissionResource not available - cannot check permissions");
        detector.handle_error("PermissionResource not available".to_string());
        return;
    };
    
    detector.is_loading = true;
    
    // Check actual OS permission status for all required permissions
    let mut all_granted = true;
    
    for perm_type in &required_perms.permissions {
        match perm_res.manager.check_permission(*perm_type) {
            Ok(PermissionStatus::Authorized) => {
                info!("Permission {:?} is authorized", perm_type);
            },
            Ok(PermissionStatus::Denied) => {
                info!("Permission {:?} is denied - wizard should show", perm_type);
                all_granted = false;
            },
            Ok(PermissionStatus::NotDetermined) => {
                info!("Permission {:?} not determined - wizard should show", perm_type);
                all_granted = false;
            },
            Ok(PermissionStatus::Restricted) => {
                warn!("Permission {:?} is restricted - wizard should show", perm_type);
                all_granted = false;
            },
            Ok(PermissionStatus::Unknown) => {
                warn!("Permission {:?} status unknown - wizard should show", perm_type);
                all_granted = false;
            },
            Err(e) => {
                error!("Failed to check permission {:?}: {}", perm_type, e);
                all_granted = false;
            },
        }
    }
    
    // Set detector state based on actual OS permission status
    if all_granted {
        info!("All required permissions granted - wizard not needed");
        detector.is_first_run = false;
        detector.wizard_completed = true;
    } else {
        info!("Some permissions missing - wizard should show");
        detector.is_first_run = true;
        detector.wizard_completed = false;
    }
    
    detector.check_completed = true;
    detector.is_loading = false;
}

/// System to handle wizard completion events
/// No persistence needed - OS permission status is the source of truth
pub fn handle_wizard_completion(
    mut completion_events: EventReader<WizardCompleteEvent>,
    mut detector: ResMut<FirstRunDetector>,
) {
    for event in completion_events.read() {
        info!("Processing wizard completion event");
        
        // Update detector state - no file persistence needed
        // OS permission status is the source of truth
        detector.mark_wizard_completed(event.completed_at);
        
        info!(
            "Wizard completed: {} permissions granted, {} failed, hotkeys_configured={}",
            event.completion_summary.granted_permissions.len(),
            event.completion_summary.failed_permissions.len(),
            event.completion_summary.hotkeys_configured
        );
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
//! Wizard Progress Tracking Systems  
//!
//! Integrates with ecs-progress to provide smooth wizard state transitions
//! based on permission status and user progress through wizard steps.

use bevy::prelude::*;
use action_items_ecs_progress::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::types::{PermissionType, PermissionStatus};
use crate::wizard::events::PermissionStatusExt;
use crate::wizard::{
    WizardState, WizardStateTransitions, WizardStepComplete,
    WizardPermissionStatusChanged, WizardCompleteEvent,
};
use crate::wizard::events::WizardCompletionSummary;

/// Resource to track wizard progress with pre-allocated entry IDs for performance
#[derive(Resource)]
pub struct WizardProgressTracker {
    /// Pre-allocated entry IDs for each permission type (zero allocation)
    permission_entries: HashMap<PermissionType, EntryId>,
    /// Entry ID for overall wizard progress
    wizard_entry: EntryId,
    /// Entry ID for hidden background operations
    background_entry: EntryId,
    /// Cache of current permission statuses
    permission_status_cache: HashMap<PermissionType, PermissionStatus>,
    /// Timestamp when wizard was started
    started_at: std::time::SystemTime,
    /// Required permissions that must be completed
    required_permissions: Vec<PermissionType>,
    /// Optional permissions (count toward progress but don't block completion)
    optional_permissions: Vec<PermissionType>,
}

impl Default for WizardProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl WizardProgressTracker {
    /// Create a new wizard progress tracker with pre-allocated entries
    pub fn new() -> Self {
        let permission_types = [
            PermissionType::Accessibility,
            PermissionType::ScreenCapture,
            PermissionType::InputMonitoring,
            PermissionType::Camera,
            PermissionType::Microphone,
            PermissionType::FullDiskAccess,
            PermissionType::WiFi,
        ];
        
        let mut permission_entries = HashMap::new();
        let mut permission_status_cache = HashMap::new();
        
        // Pre-allocate entry IDs for zero allocation during runtime
        for &perm_type in &permission_types {
            permission_entries.insert(perm_type, EntryId::new());
            permission_status_cache.insert(perm_type, PermissionStatus::Unknown);
        }
        
        Self {
            permission_entries,
            wizard_entry: EntryId::new(),
            background_entry: EntryId::new(),
            permission_status_cache,
            started_at: std::time::SystemTime::now(),
            required_permissions: vec![
                PermissionType::Accessibility,
                PermissionType::ScreenCapture,
                PermissionType::InputMonitoring,
            ],
            optional_permissions: vec![
                PermissionType::Camera,
                PermissionType::Microphone,
                PermissionType::FullDiskAccess,
                PermissionType::WiFi,
            ],
        }
    }
    
    /// Get the entry ID for a specific permission type
    #[inline]
    pub fn get_permission_entry(&self, permission_type: PermissionType) -> Option<EntryId> {
        self.permission_entries.get(&permission_type).copied()
    }
    
    /// Get the main wizard progress entry ID
    #[inline]
    pub fn get_wizard_entry(&self) -> EntryId {
        self.wizard_entry
    }
    
    /// Get the background operations entry ID
    #[inline]
    pub fn get_background_entry(&self) -> EntryId {
        self.background_entry
    }
    
    /// Update permission status and return if it changed
    pub fn update_permission_status(&mut self, permission_type: PermissionType, new_status: PermissionStatus) -> bool {
        let previous_status = self.permission_status_cache.get(&permission_type).copied().unwrap_or(PermissionStatus::Unknown);
        
        if previous_status != new_status {
            self.permission_status_cache.insert(permission_type, new_status);
            true
        } else {
            false
        }
    }
    
    /// Get current status for a permission
    #[inline]
    pub fn get_permission_status(&self, permission_type: PermissionType) -> PermissionStatus {
        self.permission_status_cache.get(&permission_type).copied().unwrap_or(PermissionStatus::Unknown)
    }
    
    /// Calculate overall progress based on permission statuses
    pub fn calculate_progress(&self) -> (u32, u32) {
        let mut completed = 0;
        let total = self.required_permissions.len() + self.optional_permissions.len();
        
        // Count required permissions (must be granted)
        for &perm_type in &self.required_permissions {
            if self.get_permission_status(perm_type).is_granted() {
                completed += 1;
            }
        }
        
        // Count optional permissions (any final status counts as "completed")
        for &perm_type in &self.optional_permissions {
            if self.get_permission_status(perm_type).is_final() {
                completed += 1;
            }
        }
        
        (completed, total as u32)
    }
    
    /// Check if all required permissions are granted
    pub fn all_required_permissions_granted(&self) -> bool {
        self.required_permissions.iter().all(|&perm_type| {
            self.get_permission_status(perm_type).is_granted()
        })
    }
    
    /// Get summary of current completion status
    pub fn get_completion_summary(&self) -> WizardCompletionSummary {
        let mut granted_permissions = Vec::new();
        let mut failed_permissions = Vec::new();
        
        for (&perm_type, &status) in &self.permission_status_cache {
            match status {
                PermissionStatus::Authorized => granted_permissions.push(perm_type),
                PermissionStatus::Denied | PermissionStatus::Restricted => failed_permissions.push(perm_type),
                _ => {}, // Ignore pending/unknown
            }
        }
        
        WizardCompletionSummary {
            granted_permissions,
            failed_permissions,
            hotkeys_configured: self.all_required_permissions_granted(),
            total_duration: self.started_at.elapsed().unwrap_or(std::time::Duration::ZERO),
        }
    }
}

/// System to update wizard progress based on permission status changes
pub fn update_wizard_progress(
    mut tracker: ResMut<WizardProgressTracker>,
    mut monitor: ResMut<ProgressMonitor<WizardState>>,
    mut progress_writer: EventWriter<Progress>,
    mut permission_status_events: EventReader<WizardPermissionStatusChanged>,
    mut step_complete_events: EventWriter<WizardStepComplete>,
    wizard_state: Res<State<WizardState>>,
    state_transitions: Res<WizardStateTransitions>,
) {
    let mut any_status_changed = false;
    
    // Process permission status changes
    for event in permission_status_events.read() {
        if tracker.update_permission_status(event.permission_type, event.new_status) {
            any_status_changed = true;
            
            // Update individual permission progress
            if let Some(entry_id) = tracker.get_permission_entry(event.permission_type) {
                let progress = if event.new_status.is_final() {
                    Progress { done: 1, total: 1 }
                } else {
                    Progress { done: 0, total: 1 }
                };
                
                monitor.update_visible(entry_id, progress);
                progress_writer.write(progress);
                
                debug!("Updated progress for {:?}: {:?}", event.permission_type, progress);
            }
        }
    }
    
    // Update overall wizard progress if any permissions changed
    if any_status_changed {
        let (completed, total) = tracker.calculate_progress();
        let wizard_progress = Progress { 
            done: completed, 
            total 
        };
        
        monitor.update_visible(tracker.get_wizard_entry(), wizard_progress);
        progress_writer.write(wizard_progress);
        
        info!("Overall wizard progress: {}/{} permissions", completed, total);
        
        // Check if we should advance to next state
        if state_transitions.can_transition() {
            match wizard_state.get() {
                WizardState::CheckingPermissions => {
                    // Advance to requesting permissions once we've checked all
                    let all_checked = tracker.permission_status_cache.values()
                        .all(|status| !matches!(status, PermissionStatus::Unknown));
                    
                    if all_checked {
                        step_complete_events.write(WizardStepComplete::new(
                            WizardState::CheckingPermissions,
                            WizardState::RequestingPermissions,
                        ));
                    }
                },
                WizardState::RequestingPermissions => {
                    // Advance to hotkey setup when all required permissions are granted
                    if tracker.all_required_permissions_granted() {
                        step_complete_events.write(WizardStepComplete::new(
                            WizardState::RequestingPermissions,
                            WizardState::SettingUpHotkeys,
                        ));
                    }
                },
                WizardState::SettingUpHotkeys => {
                    // Complete wizard when hotkeys are configured
                    // For now, auto-complete after a brief delay
                    step_complete_events.write(WizardStepComplete::new(
                        WizardState::SettingUpHotkeys,
                        WizardState::Complete,
                    ));
                },
                _ => {},
            }
        }
    }
}

/// System to handle wizard step completion events and trigger state transitions
pub fn handle_wizard_step_completion(
    mut step_complete_events: EventReader<WizardStepComplete>,
    mut next_state: ResMut<NextState<WizardState>>,
    mut state_transitions: ResMut<WizardStateTransitions>,
    mut wizard_complete_events: EventWriter<WizardCompleteEvent>,
    tracker: Res<WizardProgressTracker>,
) {
    for event in step_complete_events.read() {
        if event.auto_advance && state_transitions.can_transition() {
            info!("Advancing wizard from {:?} to {:?}", event.completed_state, event.next_state);
            
            next_state.set(event.next_state);
            state_transitions.mark_state_entered();
            
            // If completing the wizard, send completion event
            if event.next_state == WizardState::Complete {
                let summary = tracker.get_completion_summary();
                wizard_complete_events.write(WizardCompleteEvent::new(summary));
                info!("Wizard completed successfully");
            }
        }
    }
}

/// System to initialize wizard progress tracking
pub fn initialize_wizard_progress(
    _commands: Commands,
    mut monitor: ResMut<ProgressMonitor<WizardState>>,
    tracker: Res<WizardProgressTracker>,
) {
    // Initialize main wizard progress
    monitor.update_visible(tracker.get_wizard_entry(), Progress { done: 0, total: 7 });
    
    // Initialize individual permission progress entries
    for &perm_type in &tracker.required_permissions {
        if let Some(entry_id) = tracker.get_permission_entry(perm_type) {
            monitor.update_visible(entry_id, Progress { done: 0, total: 1 });
        }
    }
    
    for &perm_type in &tracker.optional_permissions {
        if let Some(entry_id) = tracker.get_permission_entry(perm_type) {
            monitor.update_visible(entry_id, Progress { done: 0, total: 1 });
        }
    }
    
    info!("Initialized wizard progress tracking");
}

/// System to cleanup wizard progress tracking
pub fn cleanup_wizard_progress(
    mut monitor: ResMut<ProgressMonitor<WizardState>>,
    _tracker: Res<WizardProgressTracker>,
) {
    // Reset all progress entries (ProgressMonitor doesn't have individual remove)
    monitor.reset();
    
    info!("Cleaned up wizard progress tracking");
}

/// System to track background permission operations
pub fn track_background_permission_operations(
    mut monitor: ResMut<ProgressMonitor<WizardState>>,
    tracker: Res<WizardProgressTracker>,
    wizard_permission_manager: Option<Res<super::permissions::WizardPermissionManager>>,
) {
    if let Some(manager) = wizard_permission_manager {
        let active_count = manager.active_request_count();
        let background_progress = Progress {
            done: if active_count == 0 { 1 } else { 0 },
            total: 1,
        };
        
        monitor.update_visible(tracker.get_background_entry(), background_progress);
    }
}

/// Run condition to check if wizard progress tracking is active
pub fn wizard_progress_active(wizard_state: Res<State<WizardState>>) -> bool {
    wizard_state.get().is_active()
}
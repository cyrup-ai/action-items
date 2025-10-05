//! Wizard Event System
//!
//! Defines all events used throughout the wizard system for communication
//! between UI, systems, and state management. Designed for zero allocation
//! and optimal performance with Bevy's event system.

#![allow(dead_code)]

use bevy::prelude::*;
use crate::types::{PermissionType, PermissionStatus};
use crate::wizard::WizardState;

/// Actions that can be triggered by wizard UI buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WizardAction {
    /// Move to the next step in the wizard
    Next,
    /// Go back to the previous step
    Back,
    /// Skip the current step (if allowed)
    Skip,
    /// Cancel the wizard and exit
    Cancel,
}

#[allow(dead_code)] // Public API methods for wizard actions
impl WizardAction {
    /// Get a human-readable description of this action
    #[inline]
    pub fn description(self) -> &'static str {
        match self {
            WizardAction::Next => "Continue to next step",
            WizardAction::Back => "Return to previous step", 
            WizardAction::Skip => "Skip this step",
            WizardAction::Cancel => "Cancel wizard setup",
        }
    }
    
    /// Check if this action is considered destructive (requires confirmation)
    #[inline]
    pub fn is_destructive(self) -> bool {
        matches!(self, WizardAction::Cancel)
    }
}

/// Request to start the wizard from the specified state
#[derive(Event, Debug, Clone, Copy)]
pub struct WizardStartRequest {
    /// State to start the wizard from (typically NotStarted)
    pub from_state: WizardState,
    /// Whether this is a forced restart (skip checks)
    pub force_restart: bool,
}

impl WizardStartRequest {
    /// Create a new wizard start request from the beginning
    #[inline]
    pub fn new() -> Self {
        Self {
            from_state: WizardState::NotStarted,
            force_restart: false,
        }
    }
    
    /// Create a forced restart request (skip first-run checks)
    #[inline]
    pub fn forced() -> Self {
        Self {
            from_state: WizardState::NotStarted,
            force_restart: true,
        }
    }
    
    /// Create a restart from specific state
    #[inline]
    pub fn from_state(state: WizardState) -> Self {
        Self {
            from_state: state,
            force_restart: false,
        }
    }
}

impl Default for WizardStartRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Event fired when a wizard step is completed
#[derive(Event, Debug, Clone, Copy)]
pub struct WizardStepComplete {
    /// The state that was just completed
    pub completed_state: WizardState,
    /// The next state to transition to
    pub next_state: WizardState,
    /// Whether this completion should trigger immediate transition
    pub auto_advance: bool,
}

impl WizardStepComplete {
    /// Create a step completion with automatic advancement
    #[inline]
    pub fn new(completed: WizardState, next: WizardState) -> Self {
        Self {
            completed_state: completed,
            next_state: next,
            auto_advance: true,
        }
    }
    
    /// Create a step completion without automatic advancement
    #[inline]
    pub fn manual(completed: WizardState, next: WizardState) -> Self {
        Self {
            completed_state: completed,
            next_state: next,
            auto_advance: false,
        }
    }
}

/// Event fired when a permission status changes during the wizard
#[derive(Event, Debug, Clone, Copy)]
pub struct WizardPermissionStatusChanged {
    /// The permission type that changed
    pub permission_type: PermissionType,
    /// The previous status (for comparison)
    pub previous_status: PermissionStatus,
    /// The new status
    pub new_status: PermissionStatus,
    /// Whether this change contributes to wizard progress
    pub affects_progress: bool,
}

// Use the main PermissionStatus type from types module
// Add wizard-specific extension methods
pub trait PermissionStatusExt {
    /// Get a user-friendly description of this status
    fn description(self) -> &'static str;
    
    /// Get the color associated with this status for UI display
    fn color(self) -> Color;
    
    /// Check if this status represents a final decision (not pending)
    fn is_final(self) -> bool;
    
    /// Check if this status represents an active operation
    fn is_active(self) -> bool;
    
    /// Check if this status represents a successful grant
    fn is_granted(self) -> bool;
}

impl PermissionStatusExt for PermissionStatus {
    #[inline]
    fn description(self) -> &'static str {
        match self {
            PermissionStatus::NotDetermined => "Not Determined",
            PermissionStatus::Authorized => "Granted",
            PermissionStatus::Denied => "Denied",
            PermissionStatus::Restricted => "Restricted",
            PermissionStatus::Unknown => "Checking...",
        }
    }
    
    #[inline]
    fn color(self) -> Color {
        match self {
            PermissionStatus::NotDetermined => Color::srgb(0.8, 0.6, 0.2), // Orange
            PermissionStatus::Authorized => Color::srgb(0.2, 0.8, 0.2), // Green
            PermissionStatus::Denied => Color::srgb(0.8, 0.2, 0.2),  // Red
            PermissionStatus::Restricted => Color::srgb(0.8, 0.2, 0.2), // Red
            PermissionStatus::Unknown => Color::srgb(0.7, 0.7, 0.7), // Gray
        }
    }
    
    #[inline]
    fn is_final(self) -> bool {
        matches!(
            self,
            PermissionStatus::Authorized | PermissionStatus::Denied | PermissionStatus::Restricted
        )
    }
    
    #[inline]
    fn is_active(self) -> bool {
        matches!(self, PermissionStatus::NotDetermined)
    }
    
    #[inline]
    fn is_granted(self) -> bool {
        matches!(self, PermissionStatus::Authorized)
    }
}

/// Request to check a specific permission status
#[derive(Event, Debug, Clone, Copy)]
pub struct WizardPermissionCheckRequest {
    /// The permission to check
    pub permission_type: PermissionType,
    /// Whether to force a fresh check (ignore cache)
    pub force_refresh: bool,
}

impl WizardPermissionCheckRequest {
    /// Create a new permission check request
    #[inline]
    pub fn new(permission_type: PermissionType) -> Self {
        Self {
            permission_type,
            force_refresh: false,
        }
    }
    
    /// Create a forced refresh request
    #[inline]
    pub fn forced(permission_type: PermissionType) -> Self {
        Self {
            permission_type,
            force_refresh: true,
        }
    }
}

/// Request to request a specific permission from the system
#[derive(Event, Debug, Clone, Copy)]
pub struct WizardPermissionRequest {
    /// The permission to request
    pub permission_type: PermissionType,
    /// Whether to show user explanation before requesting
    pub show_explanation: bool,
}

impl WizardPermissionRequest {
    /// Create a new permission request
    #[inline]
    pub fn new(permission_type: PermissionType) -> Self {
        Self {
            permission_type,
            show_explanation: true,
        }
    }
    
    /// Create a direct permission request (no explanation)
    #[inline]
    pub fn direct(permission_type: PermissionType) -> Self {
        Self {
            permission_type,
            show_explanation: false,
        }
    }
}

/// Event fired when the wizard should be canceled or skipped
#[derive(Event, Debug, Clone, Copy)]
pub struct WizardCancelRequest {
    /// The reason for cancellation
    pub reason: WizardCancelReason,
    /// Whether to save partial progress
    pub save_progress: bool,
}

/// Reasons why the wizard might be canceled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardCancelReason {
    /// User explicitly canceled the wizard
    UserCanceled,
    /// User chose to skip the wizard
    UserSkipped,
    /// System error forced cancellation
    SystemError,
    /// Permissions already configured
    AlreadyConfigured,
}

impl WizardCancelRequest {
    /// Create a user cancellation request
    #[inline]
    pub fn user_canceled() -> Self {
        Self {
            reason: WizardCancelReason::UserCanceled,
            save_progress: true,
        }
    }
    
    /// Create a user skip request
    #[inline]
    pub fn user_skipped() -> Self {
        Self {
            reason: WizardCancelReason::UserSkipped,
            save_progress: false,
        }
    }
    
    /// Create a system error cancellation
    #[inline]
    pub fn system_error() -> Self {
        Self {
            reason: WizardCancelReason::SystemError,
            save_progress: true,
        }
    }
}

/// Event fired when wizard navigation is requested (back/next)
#[derive(Event, Debug, Clone, Copy)]
pub struct WizardNavigationRequest {
    /// The direction of navigation
    pub direction: WizardNavigationDirection,
    /// Whether to validate the current step before navigating
    pub validate_current: bool,
}

/// Navigation directions within the wizard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardNavigationDirection {
    /// Go to the previous step
    Back,
    /// Go to the next step
    Next,
    /// Skip to a specific step
    SkipTo(WizardState),
}

impl WizardNavigationRequest {
    /// Create a navigation request from a wizard action
    #[inline]
    pub fn new(action: WizardAction) -> Self {
        match action {
            WizardAction::Next => Self::next(),
            WizardAction::Back => Self::back(),
            WizardAction::Skip => Self::back(), // For now, treat skip as back
            WizardAction::Cancel => Self::back(), // For now, treat cancel as back
        }
    }
    
    /// Create a back navigation request
    #[inline]
    pub fn back() -> Self {
        Self {
            direction: WizardNavigationDirection::Back,
            validate_current: false,
        }
    }
    
    /// Create a next navigation request with validation
    #[inline]
    pub fn next() -> Self {
        Self {
            direction: WizardNavigationDirection::Next,
            validate_current: true,
        }
    }
    
    /// Create a skip-to navigation request
    #[inline]
    pub fn skip_to(state: WizardState) -> Self {
        Self {
            direction: WizardNavigationDirection::SkipTo(state),
            validate_current: false,
        }
    }
}

/// Event fired when the wizard completes successfully
#[derive(Event, Debug, Clone)]
pub struct WizardCompleteEvent {
    /// Timestamp when the wizard was completed
    pub completed_at: std::time::SystemTime,
    /// Summary of what was accomplished
    pub completion_summary: WizardCompletionSummary,
}

/// Summary of wizard completion results
#[derive(Debug, Clone)]
pub struct WizardCompletionSummary {
    /// Permissions that were successfully granted
    pub granted_permissions: Vec<PermissionType>,
    /// Permissions that were denied or failed
    pub failed_permissions: Vec<PermissionType>,
    /// Whether hotkeys were configured
    pub hotkeys_configured: bool,
    /// Total time spent in wizard
    pub total_duration: std::time::Duration,
}

impl WizardCompleteEvent {
    /// Create a new completion event
    pub fn new(summary: WizardCompletionSummary) -> Self {
        Self {
            completed_at: std::time::SystemTime::now(),
            completion_summary: summary,
        }
    }
}

impl Default for WizardCompletionSummary {
    fn default() -> Self {
        Self {
            granted_permissions: Vec::new(),
            failed_permissions: Vec::new(),
            hotkeys_configured: false,
            total_duration: std::time::Duration::ZERO,
        }
    }
}

/// Event to request batch permission checking for performance optimization
#[derive(Event, Debug, Clone)]
pub struct WizardBatchPermissionCheck {
    /// List of permissions to check in batch
    pub permission_types: Vec<PermissionType>,
    /// Whether to force refresh of cached values
    pub force_refresh: bool,
}

impl WizardBatchPermissionCheck {
    /// Create a new batch check request
    pub fn new(permission_types: Vec<PermissionType>) -> Self {
        Self {
            permission_types,
            force_refresh: false,
        }
    }
    
    /// Create a forced refresh batch check
    pub fn forced(permission_types: Vec<PermissionType>) -> Self {
        Self {
            permission_types,
            force_refresh: true,
        }
    }
    
    /// Create a batch check for all wizard permissions
    pub fn all_wizard_permissions() -> Self {
        Self::new(vec![
            PermissionType::Accessibility,
            PermissionType::ScreenCapture,
            PermissionType::InputMonitoring,
            PermissionType::Camera,
            PermissionType::Microphone,
            PermissionType::FullDiskAccess,
            PermissionType::WiFi,
        ])
    }
}

/// Event fired when a batch permission check completes
#[derive(Event, Debug, Clone)]
pub struct WizardPermissionCheckComplete {
    /// The permissions that were checked
    pub checked_permissions: Vec<PermissionType>,
    /// Number of permissions that were successfully checked
    pub success_count: usize,
}

impl WizardPermissionCheckComplete {
    /// Create a new check complete event
    pub fn new(checked_permissions: Vec<PermissionType>, success_count: usize) -> Self {
        Self {
            checked_permissions,
            success_count,
        }
    }
}

/// Response event for permission set requests that were cancelled or completed
#[derive(Event, Debug, Clone)]
pub struct PermissionSetResponse {
    /// Original request ID if tracked
    pub request_id: Option<u64>,
    /// Whether wizard completed successfully
    pub completed: bool,
    /// Permissions that were granted
    pub granted_permissions: Vec<PermissionType>,
    /// Permissions that were denied
    pub denied_permissions: Vec<PermissionType>,
    /// Reason if not completed
    pub cancellation_reason: Option<WizardCancelReason>,
    /// Whether partial progress was saved
    pub progress_saved: bool,
}
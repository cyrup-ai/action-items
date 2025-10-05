//! Permission Set Request Events
//!
//! High-performance, zero-allocation events for requesting permission sets
//! and triggering wizard UI when permissions are missing.

use bevy::prelude::*;
use std::collections::HashSet;
use crate::types::{PermissionType, PermissionStatus};

/// Request for a specific set of permissions with automatic wizard fallback
///
/// This event allows any system to request a set of permissions. If any required
/// permissions are missing, the wizard UI will automatically appear to guide the
/// user through granting them.
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use action_items_ecs_permissions::{PermissionSetRequest, PermissionType};
///
/// fn request_camera_permissions(mut events: EventWriter<PermissionSetRequest>) {
///     events.send(PermissionSetRequest::new("camera_service")
///         .with_required(PermissionType::Camera)
///         .with_optional(PermissionType::Microphone)
///         .with_reason("Camera access is needed for profile pictures"));
/// }
/// ```
#[derive(Event, Debug, Clone)]
pub struct PermissionSetRequest {
    /// Unique identifier for this request (for tracking responses)
    pub request_id: String,
    /// Service or component making the request
    pub requester: String,
    /// Permissions that are absolutely required
    pub required_permissions: HashSet<PermissionType>,
    /// Permissions that would be nice to have but aren't required
    pub optional_permissions: HashSet<PermissionType>,
    /// Human-readable reason for requesting these permissions
    pub reason: String,
    /// Whether to show wizard UI if permissions are missing
    pub show_wizard_if_missing: bool,
    /// Whether to force re-checking permissions (ignore cache)
    pub force_recheck: bool,
    /// Priority level for this request (higher = more important)
    pub priority: RequestPriority,
}

/// Priority levels for permission requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    /// Low priority - can be deferred
    Low = 1,
    /// Normal priority - standard requests
    Normal = 2,
    /// High priority - important for app functionality
    High = 3,
    /// Critical priority - app cannot function without these
    Critical = 4,
}

impl Default for RequestPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl PermissionSetRequest {
    /// Create a new permission set request
    #[inline]
    pub fn new(requester: impl Into<String>) -> Self {
        let requester = requester.into();
        let request_id = format!("{}_{}", requester, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0));
        
        Self {
            request_id,
            requester,
            required_permissions: HashSet::new(),
            optional_permissions: HashSet::new(),
            reason: String::new(),
            show_wizard_if_missing: true,
            force_recheck: false,
            priority: RequestPriority::default(),
        }
    }
    
    /// Add a required permission to this request
    #[inline]
    pub fn with_required(mut self, permission: PermissionType) -> Self {
        self.required_permissions.insert(permission);
        self
    }
    
    /// Add multiple required permissions to this request
    #[inline]
    pub fn with_required_permissions(mut self, permissions: impl IntoIterator<Item = PermissionType>) -> Self {
        self.required_permissions.extend(permissions);
        self
    }
    
    /// Add an optional permission to this request
    #[inline]
    pub fn with_optional(mut self, permission: PermissionType) -> Self {
        self.optional_permissions.insert(permission);
        self
    }
    
    /// Add multiple optional permissions to this request
    #[inline]
    pub fn with_optional_permissions(mut self, permissions: impl IntoIterator<Item = PermissionType>) -> Self {
        self.optional_permissions.extend(permissions);
        self
    }
    
    /// Set the reason for requesting these permissions
    #[inline]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }
    
    /// Set whether to show wizard UI if permissions are missing
    #[inline]
    pub fn with_wizard_fallback(mut self, show_wizard: bool) -> Self {
        self.show_wizard_if_missing = show_wizard;
        self
    }
    
    /// Set whether to force re-checking permissions
    #[inline]
    pub fn with_force_recheck(mut self, force: bool) -> Self {
        self.force_recheck = force;
        self
    }
    
    /// Set the priority level for this request
    #[inline]
    pub fn with_priority(mut self, priority: RequestPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Get all permissions (required + optional) in this request
    #[inline]
    pub fn all_permissions(&self) -> HashSet<PermissionType> {
        self.required_permissions.union(&self.optional_permissions).copied().collect()
    }
    
    /// Check if this request has any permissions
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.required_permissions.is_empty() && self.optional_permissions.is_empty()
    }
    
    /// Get the total number of permissions requested
    #[inline]
    pub fn permission_count(&self) -> usize {
        self.required_permissions.len() + self.optional_permissions.len()
    }
}

/// Response to a permission set request
///
/// Sent after processing a PermissionSetRequest to indicate success or failure.
#[derive(Event, Debug, Clone)]
pub struct PermissionSetResponse {
    /// The request ID this response corresponds to
    pub request_id: String,
    /// Whether the request was successful
    pub success: bool,
    /// Permissions that were successfully granted
    pub granted_permissions: HashSet<PermissionType>,
    /// Permissions that were denied or failed
    pub denied_permissions: HashSet<PermissionType>,
    /// Permissions that are still pending (wizard in progress)
    pub pending_permissions: HashSet<PermissionType>,
    /// Error message if the request failed
    pub error_message: Option<String>,
    /// Whether the wizard was shown for this request
    pub wizard_shown: bool,
}

impl PermissionSetResponse {
    /// Create a successful response
    #[inline]
    pub fn success(request_id: String, granted: HashSet<PermissionType>) -> Self {
        Self {
            request_id,
            success: true,
            granted_permissions: granted,
            denied_permissions: HashSet::new(),
            pending_permissions: HashSet::new(),
            error_message: None,
            wizard_shown: false,
        }
    }
    
    /// Create a failed response
    #[inline]
    pub fn failure(request_id: String, error: String) -> Self {
        Self {
            request_id,
            success: false,
            granted_permissions: HashSet::new(),
            denied_permissions: HashSet::new(),
            pending_permissions: HashSet::new(),
            error_message: Some(error),
            wizard_shown: false,
        }
    }
    
    /// Create a partial response (some granted, some denied)
    #[inline]
    pub fn partial(
        request_id: String,
        granted: HashSet<PermissionType>,
        denied: HashSet<PermissionType>,
    ) -> Self {
        let success = !granted.is_empty();
        Self {
            request_id,
            success,
            granted_permissions: granted,
            denied_permissions: denied,
            pending_permissions: HashSet::new(),
            error_message: None,
            wizard_shown: false,
        }
    }
    
    /// Mark that the wizard was shown for this response
    #[inline]
    pub fn with_wizard_shown(mut self) -> Self {
        self.wizard_shown = true;
        self
    }
    
    /// Add pending permissions (wizard in progress)
    #[inline]
    pub fn with_pending(mut self, pending: HashSet<PermissionType>) -> Self {
        self.pending_permissions = pending;
        self
    }
}

/// Request to explicitly show the permission wizard
///
/// This event can be used to manually trigger the wizard UI, even if no
/// specific permissions are being requested.
#[derive(Event, Debug, Clone)]
pub struct PermissionWizardRequest {
    /// Service or component requesting the wizard
    pub requester: String,
    /// Optional set of permissions to focus on in the wizard
    pub focus_permissions: Option<HashSet<PermissionType>>,
    /// Reason for showing the wizard
    pub reason: String,
    /// Whether to force showing the wizard even if permissions are granted
    pub force_show: bool,
}

impl PermissionWizardRequest {
    /// Create a new wizard request
    #[inline]
    pub fn new(requester: impl Into<String>) -> Self {
        Self {
            requester: requester.into(),
            focus_permissions: None,
            reason: String::new(),
            force_show: false,
        }
    }
    
    /// Set specific permissions to focus on in the wizard
    #[inline]
    pub fn with_focus_permissions(mut self, permissions: HashSet<PermissionType>) -> Self {
        self.focus_permissions = Some(permissions);
        self
    }
    
    /// Set the reason for showing the wizard
    #[inline]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }
    
    /// Force showing the wizard even if permissions are already granted
    #[inline]
    pub fn with_force_show(mut self, force: bool) -> Self {
        self.force_show = force;
        self
    }
}

/// Event fired when the permission wizard completes
#[derive(Event, Debug, Clone)]
pub struct PermissionWizardComplete {
    /// The requester that triggered the wizard
    pub requester: String,
    /// Whether the wizard completed successfully
    pub success: bool,
    /// Permissions that were granted during the wizard
    pub granted_permissions: HashSet<PermissionType>,
    /// Permissions that were denied during the wizard
    pub denied_permissions: HashSet<PermissionType>,
    /// Whether the user canceled the wizard
    pub was_canceled: bool,
    /// Total time spent in the wizard
    pub duration: std::time::Duration,
}

impl PermissionWizardComplete {
    /// Create a successful completion event
    #[inline]
    pub fn success(
        requester: String,
        granted: HashSet<PermissionType>,
        duration: std::time::Duration,
    ) -> Self {
        Self {
            requester,
            success: true,
            granted_permissions: granted,
            denied_permissions: HashSet::new(),
            was_canceled: false,
            duration,
        }
    }
    
    /// Create a canceled completion event
    #[inline]
    pub fn canceled(requester: String, duration: std::time::Duration) -> Self {
        Self {
            requester,
            success: false,
            granted_permissions: HashSet::new(),
            denied_permissions: HashSet::new(),
            was_canceled: true,
            duration,
        }
    }
    
    /// Create a failed completion event
    #[inline]
    pub fn failed(
        requester: String,
        denied: HashSet<PermissionType>,
        duration: std::time::Duration,
    ) -> Self {
        Self {
            requester,
            success: false,
            granted_permissions: HashSet::new(),
            denied_permissions: denied,
            was_canceled: false,
            duration,
        }
    }
}

/// Batch permission status update event
///
/// Fired when multiple permission statuses change, typically after
/// a batch check or wizard completion.
#[derive(Event, Debug, Clone)]
pub struct PermissionBatchStatusUpdate {
    /// Map of permission types to their new statuses
    pub status_updates: std::collections::HashMap<PermissionType, PermissionStatus>,
    /// The source of this batch update
    pub source: BatchUpdateSource,
    /// Timestamp when the update occurred
    pub timestamp: std::time::SystemTime,
}

/// Source of a batch permission status update
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchUpdateSource {
    /// Update from wizard completion
    WizardCompletion,
    /// Update from batch permission check
    BatchCheck,
    /// Update from system permission change notification
    SystemNotification,
    /// Update from manual refresh
    ManualRefresh,
}

impl PermissionBatchStatusUpdate {
    /// Create a new batch status update
    #[inline]
    pub fn new(
        updates: std::collections::HashMap<PermissionType, PermissionStatus>,
        source: BatchUpdateSource,
    ) -> Self {
        Self {
            status_updates: updates,
            source,
            timestamp: std::time::SystemTime::now(),
        }
    }
    
    /// Get the number of permissions updated
    #[inline]
    pub fn update_count(&self) -> usize {
        self.status_updates.len()
    }
    
    /// Check if any permissions were granted in this update
    #[inline]
    pub fn has_granted_permissions(&self) -> bool {
        self.status_updates.values().any(|status| matches!(status, PermissionStatus::Authorized))
    }
    
    /// Check if any permissions were denied in this update
    #[inline]
    pub fn has_denied_permissions(&self) -> bool {
        self.status_updates.values().any(|status| matches!(status, PermissionStatus::Denied))
    }
}
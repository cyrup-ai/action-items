//! Wizard Permission Systems
//!
//! High-performance bridge between wizard UI and ecs-permissions API providing
//! batch operations, caching, real-time monitoring, and permission set request handling.

#![allow(dead_code)]

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tracing::{debug, info, error};

use crate::{
    PermissionChanged, PermissionRequest, PermissionRequestError, PermissionResource,
    PermissionType,
    PermissionSetRequest, PermissionSetResponse,
    PermissionBatchStatusUpdate, BatchUpdateSource,
};
use crate::wizard::{
    WizardState, WizardPermissionStatusChanged, WizardBatchPermissionCheck,
    WizardPermissionRequest, PermissionStatusExt,
    WizardStartRequest, WizardStepComplete,
    WizardCompleteEvent, WizardCompletionSummary, WizardRoot,
};
use crate::types::PermissionStatus;
use crate::wizard::components::PermissionCard;

/// Resource for tracking permission error messages
#[derive(Resource, Default)]
pub struct PermissionErrorMessages {
    messages: HashMap<PermissionType, ErrorMessage>,
}

impl PermissionErrorMessages {
    pub fn add(&mut self, perm_type: PermissionType, message: ErrorMessage) {
        self.messages.insert(perm_type, message);
    }
    
    pub fn get(&self, perm_type: PermissionType) -> Option<&ErrorMessage> {
        self.messages.get(&perm_type)
    }
    
    pub fn clear(&mut self, perm_type: PermissionType) {
        self.messages.remove(&perm_type);
    }
}

/// User-friendly error message for permission failures
#[derive(Debug, Clone)]
pub struct ErrorMessage {
    pub title: String,
    pub reason: String,
    pub recovery: String,
    pub is_critical: bool,
}

/// Generate user-friendly error message for a permission failure
fn generate_error_message(perm_type: PermissionType, _status: PermissionStatus) -> ErrorMessage {
    match perm_type {
        PermissionType::Accessibility => ErrorMessage {
            title: "Accessibility Access Denied".to_string(),
            reason: "Action Items needs Accessibility permissions to monitor keyboard shortcuts and automate tasks.".to_string(),
            recovery: "Open System Preferences → Security & Privacy → Privacy → Accessibility, then enable Action Items.".to_string(),
            is_critical: true,
        },
        PermissionType::ScreenCapture => ErrorMessage {
            title: "Screen Recording Denied".to_string(),
            reason: "Screen recording is required for visual search and screenshot features.".to_string(),
            recovery: "Open System Preferences → Security & Privacy → Privacy → Screen Recording, then enable Action Items.".to_string(),
            is_critical: true,
        },
        PermissionType::InputMonitoring => ErrorMessage {
            title: "Input Monitoring Denied".to_string(),
            reason: "Input monitoring is needed to detect keyboard shortcuts globally.".to_string(),
            recovery: "Open System Preferences → Security & Privacy → Privacy → Input Monitoring, then enable Action Items.".to_string(),
            is_critical: true,
        },
        _ => ErrorMessage {
            title: format!("{:?} Permission Denied", perm_type),
            reason: "This permission is required for full functionality.".to_string(),
            recovery: "Please grant this permission in System Preferences.".to_string(),
            is_critical: false,
        },
    }
}

/// Event for real-time permission card updates
#[derive(Event, Debug, Clone)]
pub struct PermissionCardUpdateEvent {
    pub permission_type: PermissionType,
    pub new_status: PermissionStatus,
    pub previous_status: PermissionStatus,
}

/// Resource for wizard permission management with caching and batch operations
#[derive(Resource)]
pub struct WizardPermissionManager {
    /// Cache of permission statuses with timestamps
    status_cache: HashMap<PermissionType, (PermissionStatus, Instant)>,
    /// Set of permissions currently being requested with timestamps for timeout tracking
    requesting_permissions: HashMap<PermissionType, Instant>,
    /// Set of permissions queued for checking
    check_queue: HashSet<PermissionType>,
    /// Cache expiration duration (avoid repeated API calls)
    cache_duration: Duration,
    /// Required permissions for wizard completion
    required_permissions: Vec<PermissionType>,
    /// Optional permissions (improve user experience but not required)
    optional_permissions: Vec<PermissionType>,
    /// Batch check throttling
    last_batch_check: Option<Instant>,
    batch_throttle_duration: Duration,
    /// Active permission set requests being processed
    active_set_requests: HashMap<String, PermissionSetRequestState>,
    /// Caller requests waiting for wizard completion
    pending_caller_requests: HashMap<String, CallerRequestState>,
}

/// State tracking for permission set requests
#[derive(Debug, Clone)]
pub struct PermissionSetRequestState {
    /// The original request
    request: PermissionSetRequest,
    /// Timestamp when request was received
    received_at: Instant,
    /// Permissions still being checked/requested
    pending_permissions: HashSet<PermissionType>,
    /// Permissions that have been granted
    granted_permissions: HashSet<PermissionType>,
    /// Permissions that have been denied
    denied_permissions: HashSet<PermissionType>,
    /// Whether wizard was triggered for this request
    wizard_triggered: bool,
}

/// State tracking for caller requests waiting for wizard completion
#[derive(Debug, Clone)]
pub struct CallerRequestState {
    /// The original request
    request: PermissionSetRequest,
    /// Timestamp when request was received
    received_at: Instant,
    /// Whether wizard was shown for this request
    wizard_shown: bool,
}

impl Default for WizardPermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WizardPermissionManager {
    /// Create new wizard permission manager with performance optimizations
    pub fn new() -> Self {
        Self {
            status_cache: HashMap::with_capacity(32),
            requesting_permissions: HashMap::with_capacity(16),
            check_queue: HashSet::with_capacity(16),
            cache_duration: Duration::from_secs(30),
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
            last_batch_check: None,
            batch_throttle_duration: Duration::from_millis(100),
            active_set_requests: HashMap::with_capacity(8),
            pending_caller_requests: HashMap::with_capacity(8),
        }
    }
    
    /// Get cached permission status if available and not expired
    #[inline]
    pub fn get_cached_status(&self, permission_type: PermissionType) -> Option<PermissionStatus> {
        self.status_cache.get(&permission_type)
            .filter(|(_, timestamp)| timestamp.elapsed() < self.cache_duration)
            .map(|(status, _)| *status)
    }
    
    /// Update cached permission status
    #[inline]
    pub fn update_cache(&mut self, permission_type: PermissionType, status: PermissionStatus) {
        self.status_cache.insert(permission_type, (status, Instant::now()));
    }
    
    /// Check if permission is currently being requested
    #[inline]
    pub fn is_requesting(&self, permission_type: PermissionType) -> bool {
        self.requesting_permissions.contains_key(&permission_type)
    }
    
    /// Add permission to active requests with timestamp
    #[inline]
    pub fn mark_requesting(&mut self, permission_type: PermissionType) {
        self.requesting_permissions.insert(permission_type, Instant::now());
    }
    
    /// Remove permission from active requests
    #[inline]
    pub fn mark_completed(&mut self, permission_type: PermissionType) {
        self.requesting_permissions.remove(&permission_type);
    }
    
    /// Get count of active requests
    #[inline]
    pub fn active_request_count(&self) -> usize {
        self.requesting_permissions.len()
    }
    
    /// Check if batch check throttling allows a new check
    #[inline]
    pub fn can_batch_check(&self) -> bool {
        self.last_batch_check
            .map(|last| last.elapsed() >= self.batch_throttle_duration)
            .unwrap_or(true)
    }
    
    /// Mark that a batch check was performed
    #[inline]
    pub fn mark_batch_check(&mut self) {
        self.last_batch_check = Some(Instant::now());
    }
    
    /// Check if all required permissions are granted
    pub fn all_required_permissions_granted(&self) -> bool {
        self.required_permissions.iter().all(|&perm_type| {
            self.get_cached_status(perm_type)
                .map(|status| status.is_granted())
                .unwrap_or(false)
        })
    }
    
    /// Calculate overall progress based on permission statuses
    pub fn calculate_progress(&self) -> (u32, u32) {
        let mut completed = 0;
        let total = self.required_permissions.len() + self.optional_permissions.len();
        
        // Count required permissions (must be granted)
        for &perm_type in &self.required_permissions {
            if let Some(status) = self.get_cached_status(perm_type) {
                if status.is_granted() {
                    completed += 1;
                }
            }
        }
        
        // Count optional permissions (any final status counts as "completed")
        for &perm_type in &self.optional_permissions {
            if let Some(status) = self.get_cached_status(perm_type) {
                if status.is_final() {
                    completed += 1;
                }
            }
        }
        
        (completed, total as u32)
    }
    
    /// Add permission set request to tracking
    pub fn add_set_request(&mut self, request: PermissionSetRequest) {
        let all_permissions = request.all_permissions();
        let state = PermissionSetRequestState {
            request: request.clone(),
            received_at: Instant::now(),
            pending_permissions: all_permissions,
            granted_permissions: HashSet::new(),
            denied_permissions: HashSet::new(),
            wizard_triggered: false,
        };
        self.active_set_requests.insert(request.request_id.clone(), state);
    }
    
    /// Update permission set request with status change
    pub fn update_set_request_status(
        &mut self,
        permission_type: PermissionType,
        status: PermissionStatus,
    ) -> Vec<String> {
        let mut completed_requests = Vec::new();
        
        for (request_id, state) in self.active_set_requests.iter_mut() {
            if state.pending_permissions.contains(&permission_type) {
                state.pending_permissions.remove(&permission_type);
                
                match status {
                    PermissionStatus::Authorized => {
                        state.granted_permissions.insert(permission_type);
                    },
                    PermissionStatus::Denied | PermissionStatus::Restricted => {
                        state.denied_permissions.insert(permission_type);
                    },
                    _ => {
                        // Still pending, add back to pending
                        state.pending_permissions.insert(permission_type);
                    }
                }
                
                // Check if request is complete
                if state.pending_permissions.is_empty() {
                    completed_requests.push(request_id.clone());
                }
            }
        }
        
        completed_requests
    }
    
    /// Get permission set request state
    pub fn get_set_request(&self, request_id: &str) -> Option<&PermissionSetRequestState> {
        self.active_set_requests.get(request_id)
    }
    
    /// Remove completed permission set request
    pub fn remove_set_request(&mut self, request_id: &str) -> Option<PermissionSetRequestState> {
        self.active_set_requests.remove(request_id)
    }
    
    /// Mark wizard as triggered for a permission set request
    pub fn mark_wizard_triggered(&mut self, request_id: &str) {
        if let Some(state) = self.active_set_requests.get_mut(request_id) {
            state.wizard_triggered = true;
        }
    }
    
    /// Add caller request waiting for wizard completion
    pub fn add_caller_request(&mut self, request: PermissionSetRequest, wizard_shown: bool) {
        let state = CallerRequestState {
            request: request.clone(),
            received_at: Instant::now(),
            wizard_shown,
        };
        self.pending_caller_requests.insert(request.request_id.clone(), state);
    }
    
    /// Get all pending caller requests
    pub fn get_pending_caller_requests(&self) -> &HashMap<String, CallerRequestState> {
        &self.pending_caller_requests
    }
    
    /// Remove completed caller request
    pub fn remove_caller_request(&mut self, request_id: &str) -> Option<CallerRequestState> {
        self.pending_caller_requests.remove(request_id)
    }
    
    /// Clean up timed out caller requests (5 minute timeout)
    pub fn cleanup_timed_out_caller_requests(&mut self) -> Vec<CallerRequestState> {
        let timeout_duration = Duration::from_secs(300); // 5 minutes
        let mut timed_out = Vec::new();
        
        self.pending_caller_requests.retain(|_, state| {
            if state.received_at.elapsed() > timeout_duration {
                timed_out.push(state.clone());
                false
            } else {
                true
            }
        });
        
        timed_out
    }
    
    /// Clear all pending permission requests
    pub fn clear_pending_requests(&mut self) {
        let count = self.check_queue.len();
        self.check_queue.clear();
        info!("Cleared {} pending permission requests", count);
    }
    
    /// Reset internal state for wizard restart
    pub fn reset_for_cancellation(&mut self) {
        self.requesting_permissions.clear();
        self.check_queue.clear();
        self.active_set_requests.clear();
        info!("Reset wizard permission manager for cancellation");
    }
}

/// Convert ECS permission status to wizard permission status
#[inline]
fn convert_permission_status(ecs_status: PermissionStatus) -> PermissionStatus {
    // EcsPermissionStatus and PermissionStatus are the same type now
    ecs_status
}

/// System to handle permission set requests and trigger wizard if needed
pub fn handle_permission_set_requests(
    mut permission_set_requests: EventReader<PermissionSetRequest>,
    mut permission_set_responses: EventWriter<PermissionSetResponse>,
    mut wizard_start_requests: EventWriter<WizardStartRequest>,
    mut wizard_permission_manager: ResMut<WizardPermissionManager>,
    _permission_resource: Res<PermissionResource>,
    wizard_state: Res<State<WizardState>>,
) {
    for request in permission_set_requests.read() {
        debug!("Processing permission set request from {}: {} permissions", 
               request.requester, request.permission_count());
        
        // Add to tracking
        wizard_permission_manager.add_set_request(request.clone());
        
        // Check current status of all requested permissions
        let mut granted = HashSet::new();
        let mut denied = HashSet::new();
        let mut needs_checking = HashSet::new();
        
        for &permission_type in &request.required_permissions {
            if let Some(cached_status) = wizard_permission_manager.get_cached_status(permission_type) {
                match cached_status {
                    PermissionStatus::Authorized => {
                        granted.insert(permission_type);
                    },
                    PermissionStatus::Denied | PermissionStatus::Restricted => {
                        denied.insert(permission_type);
                    },
                    _ => {
                        needs_checking.insert(permission_type);
                    }
                }
            } else {
                needs_checking.insert(permission_type);
            }
        }
        
        // Check optional permissions too
        for &permission_type in &request.optional_permissions {
            if let Some(cached_status) = wizard_permission_manager.get_cached_status(permission_type) {
                match cached_status {
                    PermissionStatus::Authorized => {
                        granted.insert(permission_type);
                    },
                    PermissionStatus::Denied | PermissionStatus::Restricted => {
                        denied.insert(permission_type);
                    },
                    _ => {
                        needs_checking.insert(permission_type);
                    }
                }
            } else {
                needs_checking.insert(permission_type);
            }
        }
        
        // If we need to check permissions or some required ones are denied, potentially trigger wizard
        let missing_required: HashSet<_> = request.required_permissions
            .difference(&granted)
            .copied()
            .collect();
        
        if !missing_required.is_empty() && request.show_wizard_if_missing {
            // Check if wizard is already active
            if !wizard_state.get().is_active() {
                info!("Triggering wizard for missing required permissions: {:?}", missing_required);
                wizard_start_requests.write(WizardStartRequest::new());
                wizard_permission_manager.mark_wizard_triggered(&request.request_id);
            }
            
            // Add caller request for completion callback
            wizard_permission_manager.add_caller_request(request.clone(), true);
            
            // Send pending response
            let response = PermissionSetResponse {
                request_id: request.request_id.clone(),
                success: false,
                granted_permissions: granted,
                denied_permissions: denied,
                pending_permissions: missing_required,
                error_message: None,
                wizard_shown: true,
            };
            permission_set_responses.write(response);
        } else if missing_required.is_empty() {
            // All required permissions are granted
            let response = PermissionSetResponse::success(request.request_id.clone(), granted);
            permission_set_responses.write(response);
            wizard_permission_manager.remove_set_request(&request.request_id);
        } else {
            // Some required permissions missing but no wizard requested
            let response = PermissionSetResponse::partial(
                request.request_id.clone(),
                granted,
                missing_required,
            );
            permission_set_responses.write(response);
        }
    }
}

/// System to handle wizard permission checks with batch optimization
pub fn handle_wizard_permission_checks(
    mut batch_check_events: EventReader<WizardBatchPermissionCheck>,
    _permission_requests: EventWriter<PermissionRequest>,
    mut wizard_permission_manager: ResMut<WizardPermissionManager>,
    _permission_resource: Res<PermissionResource>,
) {
    for batch_check in batch_check_events.read() {
        if !wizard_permission_manager.can_batch_check() && !batch_check.force_refresh {
            debug!("Skipping batch check due to throttling");
            continue;
        }
        
        wizard_permission_manager.mark_batch_check();
        
        for &permission_type in &batch_check.permission_types {
            // Skip if we have fresh cached data and not forcing refresh
            if !batch_check.force_refresh {
                if let Some(_cached_status) = wizard_permission_manager.get_cached_status(permission_type) {
                    continue;
                }
            }
            
            // Skip if already requesting
            if wizard_permission_manager.is_requesting(permission_type) {
                continue;
            }
            
            // Check permission status through ecs-permissions
            let current_status = _permission_resource.manager.check_permission(permission_type);
            let wizard_status = match current_status {
                Ok(status) => status,
                Err(_) => PermissionStatus::Denied,
            };
            wizard_permission_manager.update_cache(permission_type, wizard_status);
        }
    }
}

/// System to handle individual wizard permission requests with native dialog integration
pub fn handle_wizard_permission_requests(
    mut permission_requests: EventReader<WizardPermissionRequest>,
    mut ecs_permission_requests: EventWriter<PermissionRequest>,
    mut wizard_permission_manager: ResMut<WizardPermissionManager>,
    time: Res<Time>,
) {
    let _now = time.elapsed();
    
    // Check for timed out requests (30 second timeout)
    let timed_out_permissions: Vec<PermissionType> = wizard_permission_manager
        .requesting_permissions
        .iter()
        .filter_map(|(perm_type, start_time)| {
            if start_time.elapsed() > Duration::from_secs(30) {
                Some(*perm_type)
            } else {
                None
            }
        })
        .collect();
    
    // Clean up timed out requests
    for permission_type in timed_out_permissions {
        wizard_permission_manager.mark_completed(permission_type);
        error!("Permission request timed out after 30 seconds: {}", permission_type);
    }
    
    for request in permission_requests.read() {
        if wizard_permission_manager.is_requesting(request.permission_type) {
            debug!("Permission {} already being requested", request.permission_type);
            continue;
        }
        
        // Platform-specific API validation and logging
        info!("Triggering native permission dialog for: {}", request.permission_type);
        
        #[cfg(target_os = "macos")]
        {
            info!("macOS: Will open Security & Privacy settings for permission: {}", request.permission_type);
        }
        
        #[cfg(target_os = "windows")]
        {
            info!("Windows: Will open Settings app for permission: {}", request.permission_type);
        }
        
        #[cfg(target_os = "linux")]
        {
            info!("Linux: Will use platform-specific permission system for: {}", request.permission_type);
        }
        
        // Mark as requesting with timestamp for timeout tracking
        wizard_permission_manager.mark_requesting(request.permission_type);
        
        // Send request to ecs-permissions which handles native dialog triggering
        ecs_permission_requests.write(PermissionRequest {
            typ: request.permission_type,
        });
        info!("Successfully sent permission request to native API: {}", request.permission_type);
    }
}

/// SystemParam grouping wizard event writers to reduce function parameter count
#[derive(SystemParam)]
pub struct WizardEventWriters<'w> {
    wizard_status_changed: EventWriter<'w, WizardPermissionStatusChanged>,
    permission_set_responses: EventWriter<'w, PermissionSetResponse>,
    batch_status_updates: EventWriter<'w, PermissionBatchStatusUpdate>,
    wizard_step_complete: EventWriter<'w, WizardStepComplete>,
}

/// System to monitor ecs-permissions events and update wizard state with real-time updates
pub fn monitor_ecs_permission_events(
    mut permission_changed_events: EventReader<PermissionChanged>,
    mut permission_error_events: EventReader<PermissionRequestError>,
    mut events: WizardEventWriters,
    mut wizard_permission_manager: ResMut<WizardPermissionManager>,
    mut commands: Commands,
) {
    let mut status_updates = HashMap::new();
    
    // Handle permission changes
    for event in permission_changed_events.read() {
        let wizard_status = convert_permission_status(event.status);
        let previous_status = wizard_permission_manager
            .get_cached_status(event.typ)
            .unwrap_or(PermissionStatus::Unknown);
        
        wizard_permission_manager.update_cache(event.typ, wizard_status);
        wizard_permission_manager.mark_completed(event.typ);
        
        status_updates.insert(event.typ, wizard_status);
        
        // Send wizard-specific status change event
        events.wizard_status_changed.write(WizardPermissionStatusChanged {
            permission_type: event.typ,
            previous_status,
            new_status: wizard_status,
            affects_progress: wizard_permission_manager.required_permissions.contains(&event.typ),
        });
        
        // Update any active permission set requests
        let completed_requests = wizard_permission_manager
            .update_set_request_status(event.typ, wizard_status);
        
        for request_id in completed_requests {
            if let Some(state) = wizard_permission_manager.remove_set_request(&request_id) {
                let response = if state.denied_permissions.is_empty() || 
                    !state.request.required_permissions.iter().any(|p| state.denied_permissions.contains(p)) {
                    PermissionSetResponse::success(request_id, state.granted_permissions)
                } else {
                    PermissionSetResponse::partial(request_id, state.granted_permissions, state.denied_permissions)
                }.with_wizard_shown();
                
                events.permission_set_responses.write(response);
            }
        }
        
        info!("Permission {} status changed to {:?}", event.typ, wizard_status);
        
        // Real-time permission card updates - trigger UI component updates
        commands.trigger(PermissionCardUpdateEvent {
            permission_type: event.typ,
            new_status: wizard_status,
            previous_status,
        });
    }
    
    // Handle permission errors
    for event in permission_error_events.read() {
        let wizard_status = PermissionStatus::Denied;
        let previous_status = wizard_permission_manager
            .get_cached_status(event.typ)
            .unwrap_or(PermissionStatus::Unknown);
        
        wizard_permission_manager.update_cache(event.typ, wizard_status);
        wizard_permission_manager.mark_completed(event.typ);
        
        status_updates.insert(event.typ, wizard_status);
        
        events.wizard_status_changed.write(WizardPermissionStatusChanged {
            permission_type: event.typ,
            previous_status,
            new_status: wizard_status,
            affects_progress: wizard_permission_manager.required_permissions.contains(&event.typ),
        });
        
        error!("Permission {} request failed: {:?}", event.typ, event.error);
    }
    
    // Check if all required permissions are granted and trigger wizard step completion
    if !status_updates.is_empty() {
        let all_required_granted = wizard_permission_manager
            .required_permissions
            .iter()
            .all(|&perm_type| {
                wizard_permission_manager
                    .get_cached_status(perm_type)
                    .map(|status| status.is_granted())
                    .unwrap_or(false)
            });
        
        if all_required_granted {
            info!("All required permissions granted - triggering wizard step completion");
            events.wizard_step_complete.write(WizardStepComplete {
                completed_state: WizardState::CheckingPermissions,
                next_state: WizardState::SettingUpHotkeys,
                auto_advance: true,
            });
        }
        
        // Send batch status update if any changes occurred
        events.batch_status_updates.write(PermissionBatchStatusUpdate::new(
            status_updates,
            BatchUpdateSource::SystemNotification,
        ));
    }
}

/// System to handle caller callbacks when wizard completes
pub fn handle_caller_callback_system(
    mut permission_set_responses: EventWriter<PermissionSetResponse>,
    mut wizard_permission_manager: ResMut<WizardPermissionManager>,
    wizard_state: Res<State<WizardState>>,
    mut last_state: Local<Option<WizardState>>,
) {
    let current_state = *wizard_state.get();
    
    // Check if wizard state changed to Complete
    let wizard_completed = last_state.map(|s| s != current_state && current_state == WizardState::Complete).unwrap_or(false);
    *last_state = Some(current_state);
    
    // Handle wizard completion callbacks
    if wizard_completed {
        info!("Wizard completed - processing caller callbacks");
        
        // Get all pending caller requests
        let pending_requests: Vec<_> = wizard_permission_manager
            .get_pending_caller_requests()
            .values()
            .cloned()
            .collect();
        
        // Send responses for each pending request
        for caller_state in pending_requests {
            let request = &caller_state.request;
            
            // Collect current permission statuses
            let mut granted_permissions = HashSet::new();
            let mut denied_permissions = HashSet::new();
            let mut pending_permissions = HashSet::new();
            
            // Check all requested permissions
            for &permission_type in &request.required_permissions {
                match wizard_permission_manager.get_cached_status(permission_type) {
                    Some(status) if status.is_granted() => {
                        granted_permissions.insert(permission_type);
                    }
                    Some(PermissionStatus::Denied) | Some(PermissionStatus::Restricted) => {
                        denied_permissions.insert(permission_type);
                    }
                    _ => {
                        pending_permissions.insert(permission_type);
                    }
                }
            }
            
            for &permission_type in &request.optional_permissions {
                match wizard_permission_manager.get_cached_status(permission_type) {
                    Some(status) if status.is_granted() => {
                        granted_permissions.insert(permission_type);
                    }
                    Some(PermissionStatus::Denied) | Some(PermissionStatus::Restricted) => {
                        denied_permissions.insert(permission_type);
                    }
                    _ => {
                        pending_permissions.insert(permission_type);
                    }
                }
            }
            
            // Determine success based on required permissions
            let success = request.required_permissions.iter().all(|&perm_type| {
                granted_permissions.contains(&perm_type)
            });
            
            // Create response
            let response = PermissionSetResponse {
                request_id: request.request_id.clone(),
                success,
                granted_permissions,
                denied_permissions,
                pending_permissions,
                error_message: None,
                wizard_shown: caller_state.wizard_shown,
            };
            
            permission_set_responses.write(response);
            info!("Sent callback response for request: {}", request.request_id);
        }
        
        // Clear all pending caller requests
        wizard_permission_manager.pending_caller_requests.clear();
    }
    
    // Handle timeout cleanup (5 minute timeout)
    let timed_out_requests = wizard_permission_manager.cleanup_timed_out_caller_requests();
    for caller_state in timed_out_requests {
        let request = &caller_state.request;
        
        // Send timeout response
        let response = PermissionSetResponse {
            request_id: request.request_id.clone(),
            success: false,
            granted_permissions: HashSet::new(),
            denied_permissions: HashSet::new(),
            pending_permissions: request.required_permissions.iter().chain(request.optional_permissions.iter()).copied().collect(),
            error_message: Some("Wizard session timed out after 5 minutes".to_string()),
            wizard_shown: caller_state.wizard_shown,
        };
        
        permission_set_responses.write(response);
        info!("Sent timeout response for request: {}", request.request_id);
    }
}

/// System to handle hotkey registration during wizard SettingUpHotkeys state
///
/// Integrates with ecs-hotkey service to register configured hotkeys and handle
/// registration responses, storing results in wizard completion summary.
pub(crate) fn handle_hotkey_registration_system(
    mut commands: Commands,
    wizard_state: Res<State<WizardState>>,
    mut last_state: Local<Option<WizardState>>,
    mut hotkey_manager: Local<HotkeyWizardManager>,
) {
    let current_state = *wizard_state.get();
    
    // Check if entering SettingUpHotkeys state
    let entering_hotkey_setup = last_state.map(|s| s != current_state && current_state == WizardState::SettingUpHotkeys).unwrap_or(false);
    *last_state = Some(current_state);
    
    if entering_hotkey_setup {
        info!("Entered hotkey setup state - initializing hotkey configuration");
        hotkey_manager.initialize_default_suggestions();
        
        // Trigger conflict detection for default suggestions
        commands.trigger(HotkeyConflictCheckRequested {
            requester: "wizard".to_string(),
        });
    }
    
    // Check if leaving SettingUpHotkeys state
    if last_state.map(|s| s == WizardState::SettingUpHotkeys && current_state != WizardState::SettingUpHotkeys).unwrap_or(false) {
        info!("Leaving hotkey setup state - finalizing hotkey configuration");
        
        // Register configured hotkeys
        for (action, hotkey_config) in hotkey_manager.configured_hotkeys.iter() {
            commands.trigger(HotkeyRegistrationRequested {
                action: action.clone(),
                configuration: hotkey_config.clone(),
                requester: "wizard".to_string(),
            });
        }
    }
}

/// System to handle hotkey registration responses from ecs-hotkey service
///
/// Processes registration success/failure events and updates wizard progress.
pub(crate) fn handle_hotkey_registration_responses_system(
    mut registration_events: EventReader<HotkeyRegistrationCompleted>,
    mut conflict_events: EventReader<HotkeyConflictDetected>,
    mut validation_events: EventReader<HotkeyValidationResult>,
    mut hotkey_manager: Local<HotkeyWizardManager>,
    mut commands: Commands,
) {
    // Handle registration completion events
    for event in registration_events.read() {
        if event.success {
            info!("Hotkey registration successful: {}", event.action);
            hotkey_manager.mark_registration_complete(&event.action, true, None);
        } else {
            error!("Hotkey registration failed: {} - {:?}", event.action, event.error_message);
            hotkey_manager.mark_registration_complete(&event.action, false, event.error_message.clone());
        }
    }
    
    // Handle conflict detection events
    for event in conflict_events.read() {
        info!("Hotkey conflict detected: {:?}", event);
        hotkey_manager.add_conflict_warning(event.action.clone(), event.conflict_description.clone());
        
        // Update conflict warning UI
        commands.trigger(HotkeyConflictWarningUpdate {
            action: event.action.clone(),
            conflict: event.conflict_description.clone(),
            suggestions: event.suggested_alternatives.clone(),
        });
    }
    
    // Handle validation results
    for event in validation_events.read() {
        if event.is_valid {
            info!("Hotkey validation passed: {}", event.action);
        } else {
            info!("Hotkey validation failed: {} - {}", event.action, event.validation_message);
        }
        
        hotkey_manager.update_validation_status(&event.action, event.is_valid, event.validation_message.clone());
        
        // Update validation feedback UI
        commands.trigger(HotkeyValidationFeedbackUpdate {
            action: event.action.clone(),
            is_valid: event.is_valid,
            message: event.validation_message.clone(),
        });
    }
}

/// Local resource to manage hotkey configuration during wizard
#[derive(Default)]
pub struct HotkeyWizardManager {
    configured_hotkeys: HashMap<String, HotkeyConfiguration>,
    registration_results: HashMap<String, HotkeyRegistrationResult>,
    conflict_warnings: HashMap<String, String>,
    validation_statuses: HashMap<String, ValidationStatus>,
    setup_skipped: bool,
}

impl HotkeyWizardManager {
    fn initialize_default_suggestions(&mut self) {
        // Platform-specific defaults
        #[cfg(target_os = "macos")]
        let defaults = vec![
            ("Show/Hide Window", "Cmd+Shift+Space"),
            ("Quick Add", "Cmd+Shift+A"),
            ("Search", "Cmd+Shift+S"),
            ("Today's Tasks", "Cmd+Shift+T"),
        ];
        
        #[cfg(target_os = "windows")]
        let defaults = vec![
            ("Show/Hide Window", "Ctrl+Shift+Space"),
            ("Quick Add", "Ctrl+Shift+A"),
            ("Search", "Ctrl+Shift+S"),
            ("Today's Tasks", "Ctrl+Shift+T"),
        ];
        
        #[cfg(target_os = "linux")]
        let defaults = vec![
            ("Show/Hide Window", "Super+Shift+Space"),
            ("Quick Add", "Super+Shift+A"),
            ("Search", "Super+Shift+S"),
            ("Today's Tasks", "Super+Shift+T"),
        ];
        
        for (action, hotkey) in defaults {
            self.configured_hotkeys.insert(
                action.to_string(),
                HotkeyConfiguration {
                    hotkey: hotkey.to_string(),
                    enabled: true,
                    custom: false,
                },
            );
        }
    }
    
    fn mark_registration_complete(&mut self, action: &str, success: bool, error: Option<String>) {
        self.registration_results.insert(
            action.to_string(),
            HotkeyRegistrationResult { success, error },
        );
    }
    
    fn add_conflict_warning(&mut self, action: String, warning: String) {
        self.conflict_warnings.insert(action, warning);
    }
    
    fn update_validation_status(&mut self, action: &str, is_valid: bool, message: String) {
        self.validation_statuses.insert(
            action.to_string(),
            ValidationStatus { is_valid, message },
        );
    }
    
    fn get_completion_summary(&self) -> HotkeyCompletionSummary {
        HotkeyCompletionSummary {
            configured_hotkeys: self.configured_hotkeys.clone(),
            registration_results: self.registration_results.clone(),
            setup_skipped: self.setup_skipped,
        }
    }
}

/// Hotkey configuration details
#[derive(Clone, Debug)]
pub(crate) struct HotkeyConfiguration {
    hotkey: String,
    enabled: bool,
    custom: bool,
}

/// Hotkey registration result
#[derive(Clone, Debug)]
pub(crate) struct HotkeyRegistrationResult {
    success: bool,
    error: Option<String>,
}

/// Validation status for hotkey
#[derive(Clone, Debug)]
struct ValidationStatus {
    is_valid: bool,
    message: String,
}

/// Hotkey completion summary for wizard
#[derive(Clone, Debug)]
pub(crate) struct HotkeyCompletionSummary {
    pub configured_hotkeys: HashMap<String, HotkeyConfiguration>,
    pub registration_results: HashMap<String, HotkeyRegistrationResult>,
    pub setup_skipped: bool,
}

/// Event for requesting hotkey registration
#[derive(Event, Debug, Clone)]
struct HotkeyRegistrationRequested {
    action: String,
    configuration: HotkeyConfiguration,
    requester: String,
}

/// Event for hotkey registration completion
#[derive(Event, Debug, Clone)]
pub(crate) struct HotkeyRegistrationCompleted {
    action: String,
    success: bool,
    error_message: Option<String>,
}

/// Event for conflict check request
#[derive(Event, Debug, Clone)]
struct HotkeyConflictCheckRequested {
    requester: String,
}

/// Event for conflict detection
#[derive(Event, Debug, Clone)]
pub(crate) struct HotkeyConflictDetected {
    action: String,
    conflict_description: String,
    suggested_alternatives: Vec<String>,
}

/// Event for conflict warning UI update
#[derive(Event, Debug, Clone)]
struct HotkeyConflictWarningUpdate {
    action: String,
    conflict: String,
    suggestions: Vec<String>,
}

/// Event for hotkey validation result
#[derive(Event, Debug, Clone)]
pub(crate) struct HotkeyValidationResult {
    action: String,
    is_valid: bool,
    validation_message: String,
}

/// Event for validation feedback UI update
#[derive(Event, Debug, Clone)]
struct HotkeyValidationFeedbackUpdate {
    action: String,
    is_valid: bool,
    message: String,
}

/// System to auto-check permissions when entering checking state
pub fn auto_check_permissions_on_state_enter(
    mut batch_check_events: EventWriter<WizardBatchPermissionCheck>,
    wizard_state: Res<State<WizardState>>,
    mut last_state: Local<Option<WizardState>>,
) {
    let current_state = *wizard_state.get();
    
    if last_state.map(|s| s != current_state).unwrap_or(true) {
        if current_state == WizardState::CheckingPermissions {
            batch_check_events.write(WizardBatchPermissionCheck::all_wizard_permissions());
            info!("Auto-triggered batch permission check on state enter");
        }
        *last_state = Some(current_state);
    }
}

/// System to refresh expired permission cache entries
pub fn refresh_expired_permission_cache(
    mut batch_check_events: EventWriter<WizardBatchPermissionCheck>,
    wizard_permission_manager: Res<WizardPermissionManager>,
    mut last_refresh: Local<Option<Instant>>,
) {
    const REFRESH_INTERVAL: Duration = Duration::from_secs(60);
    
    if last_refresh.map(|t| t.elapsed() >= REFRESH_INTERVAL).unwrap_or(true) {
        let expired_permissions: Vec<_> = wizard_permission_manager.required_permissions
            .iter()
            .chain(wizard_permission_manager.optional_permissions.iter())
            .filter(|&&perm| wizard_permission_manager.get_cached_status(perm).is_none())
            .copied()
            .collect();
        
        if !expired_permissions.is_empty() {
            batch_check_events.write(WizardBatchPermissionCheck::new(expired_permissions));
            debug!("Refreshing expired permission cache entries");
        }
        
        *last_refresh = Some(Instant::now());
    }
}

/// System to monitor permission status changes and log progress
pub fn monitor_permission_status_changes(
    mut status_changed_events: EventReader<WizardPermissionStatusChanged>,
) {
    for event in status_changed_events.read() {
        if event.affects_progress {
            info!("Progress-affecting permission {} changed from {:?} to {:?}",
                  event.permission_type, event.previous_status, event.new_status);
        }
    }
}

/// System to generate wizard completion summary when wizard finishes
///
/// Collects all granted/denied permissions, hotkey configuration status,
/// wizard duration, and generates WizardCompleteEvent with comprehensive summary.
pub(crate) fn generate_wizard_completion_summary_system(
    mut wizard_complete_events: EventWriter<WizardCompleteEvent>,
    wizard_permission_manager: Res<WizardPermissionManager>,
    hotkey_manager: Local<HotkeyWizardManager>,
    wizard_root_query: Query<&WizardRoot>,
    wizard_state: Res<State<WizardState>>,
    mut last_state: Local<Option<WizardState>>,
) {
    let current_state = *wizard_state.get();
    
    // Check if wizard just transitioned to Complete state
    let wizard_completed = last_state.map(|s| s != current_state && current_state == WizardState::Complete).unwrap_or(false);
    *last_state = Some(current_state);
    
    if wizard_completed {
        info!("Wizard completed - generating comprehensive completion summary");
        
        // SUBTASK2: Collect all granted and denied permissions
        let mut granted_permissions = Vec::new();
        let mut denied_permissions = Vec::new();
        
        // Collect from required permissions
        for &permission_type in &wizard_permission_manager.required_permissions {
            if let Some(status) = wizard_permission_manager.get_cached_status(permission_type) {
                if status.is_granted() {
                    granted_permissions.push(permission_type);
                } else {
                    denied_permissions.push(permission_type);
                }
            } else {
                denied_permissions.push(permission_type);
            }
        }
        
        // Collect from optional permissions
        for &permission_type in &wizard_permission_manager.optional_permissions {
            if let Some(status) = wizard_permission_manager.get_cached_status(permission_type) {
                if status.is_granted() {
                    granted_permissions.push(permission_type);
                } else {
                    denied_permissions.push(permission_type);
                }
            }
        }
        
        info!("Collected {} granted permissions and {} denied permissions",
              granted_permissions.len(), denied_permissions.len());
        
        // SUBTASK3: Include hotkey configuration status
        let hotkeys_configured = !hotkey_manager.setup_skipped && !hotkey_manager.configured_hotkeys.is_empty();
        info!("Hotkeys configured: {}", hotkeys_configured);
        
        // SUBTASK4: Calculate wizard duration
        let total_duration = if let Ok(wizard_root) = wizard_root_query.single() {
            match wizard_root.started_at.elapsed() {
                Ok(duration) => duration,
                Err(_) => Duration::ZERO,
            }
        } else {
            Duration::ZERO
        };
        
        info!("Wizard duration: {:?}", total_duration);
        
        // SUBTASK5: Generate WizardCompleteEvent with full summary
        let completion_summary = WizardCompletionSummary {
            granted_permissions,
            failed_permissions: denied_permissions,
            hotkeys_configured,
            total_duration,
        };
        
        let complete_event = WizardCompleteEvent::new(completion_summary);
        wizard_complete_events.write(complete_event);
        
        info!("Wizard completion summary generated and event sent");
    }
}

/// System to monitor permission failures and provide recovery guidance
///
/// Detects denied/restricted permissions and generates user-friendly error messages
/// with actionable recovery instructions.
pub fn handle_permission_error_recovery(
    card_query: Query<&PermissionCard, Changed<PermissionCard>>,
    mut error_messages: ResMut<PermissionErrorMessages>,
) {
    for card in card_query.iter() {
        if matches!(card.status, PermissionStatus::Denied | PermissionStatus::Restricted) {
            let message = generate_error_message(card.permission_type, card.status);
            error_messages.add(card.permission_type, message);
            if let Some(msg) = error_messages.get(card.permission_type) {
                info!("Permission error: {:?} - {}", card.permission_type, msg.title);
            }
        } else if card.status.is_granted() {
            // Clear error message if permission granted
            error_messages.clear(card.permission_type);
        }
    }
}

/// System to cleanup wizard permissions on exit
pub fn cleanup_wizard_permissions(
    mut wizard_permission_manager: ResMut<WizardPermissionManager>,
) {
    wizard_permission_manager.requesting_permissions.clear();
    wizard_permission_manager.check_queue.clear();
    wizard_permission_manager.active_set_requests.clear();
    info!("Cleaned up wizard permission state");
}

/// Run condition to check if wizard permissions are active
pub fn wizard_permissions_active(wizard_state: Res<State<WizardState>>) -> bool {
    wizard_state.get().is_permission_phase()
}
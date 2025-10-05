//! Wizard Navigation Systems
//!
//! Handles wizard navigation flow including back/next/skip buttons,
//! state transitions, and validation logic for smooth user experience.

use bevy::prelude::*;
use std::time::SystemTime;
use tracing::{debug, info, warn};

use crate::types::{PermissionType, PermissionStatus};
use crate::wizard::{
    WizardState, WizardNavigationRequest, WizardNavigationDirection,
    WizardStepComplete, WizardCancelRequest, WizardStateTransitions,
    WizardCancelReason, PermissionStatusExt, NavigationAction,
};
use crate::wizard::events::PermissionSetResponse;
use crate::wizard::components::PermissionCard;
use crate::wizard::first_run::{FirstRunDetector, WizardPartialProgress};
use crate::wizard::systems::permissions::WizardPermissionManager;

/// System to handle wizard navigation requests (back/next/skip)
pub fn handle_wizard_navigation(
    mut navigation_events: EventReader<WizardNavigationRequest>,
    mut next_wizard_state: ResMut<NextState<WizardState>>,
    mut step_complete_events: EventWriter<WizardStepComplete>,
    wizard_state: Res<State<WizardState>>,
    permission_manager: Option<Res<WizardPermissionManager>>,
    _state_transitions: Res<WizardStateTransitions>,
) {
    for event in navigation_events.read() {
        let current_state = *wizard_state.get();
        
        // Validate current step if requested
        if event.validate_current && !validate_current_step(
            current_state, 
            permission_manager.as_deref(),
        ) {
            warn!("Current wizard step validation failed: {:?}", current_state);
            continue;
        }
        
        match event.direction {
            WizardNavigationDirection::Back => {
                if let Some(previous_state) = current_state.previous_state() {
                    info!("Navigating back: {:?} -> {:?}", current_state, previous_state);
                    next_wizard_state.set(previous_state);
                } else {
                    debug!("Cannot navigate back from {:?} - already at beginning", current_state);
                }
            },
            WizardNavigationDirection::Next => {
                if let Some(next_state) = current_state.next_state() {
                    info!("Navigating forward: {:?} -> {:?}", current_state, next_state);
                    
                    // Emit step completion event
                    step_complete_events.write(WizardStepComplete::new(current_state, next_state));
                } else {
                    debug!("Cannot navigate forward from {:?} - already at end", current_state);
                }
            },
            WizardNavigationDirection::SkipTo(target_state) => {
                if can_skip_to_state(current_state, target_state) {
                    info!("Skipping to state: {:?} -> {:?}", current_state, target_state);
                    next_wizard_state.set(target_state);
                } else {
                    warn!("Cannot skip to {:?} from {:?} - transition not allowed", 
                          target_state, current_state);
                }
            },
        }
    }
}

/// System to handle wizard cancellation requests
pub fn handle_wizard_cancellation(
    mut cancel_events: EventReader<WizardCancelRequest>,
    mut next_wizard_state: ResMut<NextState<WizardState>>,
    wizard_state: Res<State<WizardState>>,
    mut permission_manager: ResMut<WizardPermissionManager>,
    mut first_run: ResMut<FirstRunDetector>,
    card_query: Query<&PermissionCard>,
) {
    for event in cancel_events.read() {
        let current_state = *wizard_state.get();
        
        info!("Wizard cancellation: reason={:?}, save_progress={}", event.reason, event.save_progress);
        
        // CLEANUP: Reset permission manager state
        permission_manager.reset_for_cancellation();
        
        // SAVE PROGRESS if requested
        if event.save_progress {
            let completed_permissions: Vec<PermissionType> = card_query
                .iter()
                .filter(|card| card.status.is_granted())
                .map(|card| card.permission_type)
                .collect();
            
            info!("Saving partial progress: {} permissions granted", completed_permissions.len());
            
            let progress = WizardPartialProgress {
                last_state: current_state,
                completed_permissions,
                cancelled_at: SystemTime::now(),
                can_resume: true,
            };
            
            first_run.save_partial_progress(progress);
        }
        
        match event.reason {
            WizardCancelReason::UserCanceled => {
                info!("User canceled wizard from state: {:?}", current_state);
                next_wizard_state.set(WizardState::NotStarted);
            },
            WizardCancelReason::UserSkipped => {
                info!("User skipped wizard from state: {:?}", current_state);
                next_wizard_state.set(WizardState::Complete);
            },
            WizardCancelReason::SystemError => {
                warn!("System error forced wizard cancellation from state: {:?}", current_state);
                next_wizard_state.set(WizardState::NotStarted);
            },
            WizardCancelReason::AlreadyConfigured => {
                info!("Wizard canceled - permissions already configured");
                next_wizard_state.set(WizardState::Complete);
            },
        }
    }
}

/// Send PermissionSetResponse when wizard is cancelled
///
/// Collects granted and denied permissions and sends response event
/// so calling services know the wizard outcome.
pub fn send_cancellation_response(
    mut cancel_events: EventReader<WizardCancelRequest>,
    card_query: Query<&PermissionCard>,
    mut response_events: EventWriter<PermissionSetResponse>,
) {
    for event in cancel_events.read() {
        let granted: Vec<PermissionType> = card_query
            .iter()
            .filter(|c| c.status.is_granted())
            .map(|c| c.permission_type)
            .collect();
            
        let denied: Vec<PermissionType> = card_query
            .iter()
            .filter(|c| matches!(c.status, PermissionStatus::Denied | PermissionStatus::Restricted))
            .map(|c| c.permission_type)
            .collect();
        
        response_events.write(PermissionSetResponse {
            request_id: None,
            completed: false,
            granted_permissions: granted.clone(),
            denied_permissions: denied.clone(),
            cancellation_reason: Some(event.reason),
            progress_saved: event.save_progress,
        });
        
        info!("Sent cancellation response: {} granted, {} denied, reason: {:?}", 
              granted.len(), denied.len(), event.reason);
    }
}

/// System to provide automatic navigation hints based on current state
pub fn provide_navigation_hints(
    wizard_state: Res<State<WizardState>>,
    permission_manager: Option<Res<WizardPermissionManager>>,
    mut last_hint_state: Local<Option<WizardState>>,
) {
    let current_state = *wizard_state.get();
    
    // Only provide hints when state changes
    if last_hint_state.map(|s| s != current_state).unwrap_or(true) {
        match current_state {
            WizardState::Welcome => {
                info!("Navigation hint: Click 'Get Started' to begin permission setup");
            },
            WizardState::CheckingPermissions => {
                info!("Navigation hint: Checking permissions automatically...");
            },
            WizardState::RequestingPermissions => {
                if let Some(manager) = &permission_manager {
                    let (completed, total) = manager.calculate_progress();
                    info!("Navigation hint: Grant permissions ({}/{} complete)", completed, total);
                } else {
                    info!("Navigation hint: Grant the required permissions to continue");
                }
            },
            WizardState::SettingUpHotkeys => {
                info!("Navigation hint: Configuring global hotkeys...");
            },
            WizardState::Complete => {
                info!("Navigation hint: Setup complete! Click 'Finish' to start using Action Items");
            },
            WizardState::NotStarted => {
                // No hint needed for not started state
            },
        }
        
        *last_hint_state = Some(current_state);
    }
}

/// System to handle timed transitions (e.g., welcome screen auto-advance)
pub fn handle_timed_transitions(
    mut step_complete_events: EventWriter<WizardStepComplete>,
    wizard_state: Res<State<WizardState>>,
    state_transitions: Res<WizardStateTransitions>,
    mut welcome_timer: Local<Option<std::time::Instant>>,
) {
    let current_state = *wizard_state.get();
    
    match current_state {
        WizardState::Welcome => {
            // Initialize timer if not set
            if welcome_timer.is_none() {
                *welcome_timer = Some(std::time::Instant::now());
            }
            
            // Auto-advance after 3 seconds if auto-transitions are enabled
            if let Some(start_time) = *welcome_timer {
                if state_transitions.auto_transitions && 
                   start_time.elapsed() >= std::time::Duration::from_secs(3) {
                    step_complete_events.write(WizardStepComplete::new(
                        WizardState::Welcome,
                        WizardState::CheckingPermissions,
                    ));
                    *welcome_timer = None;
                }
            }
        },
        _ => {
            // Reset timer for non-welcome states
            *welcome_timer = None;
        },
    }
}

/// System to update auto-advance timers
pub fn update_auto_advance_timers(
    time: Res<Time>,
    mut timer_resource: Local<Option<Timer>>,
) {
    if let Some(ref mut timer) = timer_resource.as_mut() {
        timer.tick(time.delta());
    }
}

/// System to spawn welcome screen auto-advance timer
pub fn spawn_welcome_auto_advance_timer(
    mut commands: Commands,
    wizard_state: Res<State<WizardState>>,
    mut timer_spawned: Local<bool>,
) {
    if *wizard_state.get() == WizardState::Welcome && !*timer_spawned {
        commands.spawn((
            WelcomeTimer(Timer::from_seconds(3.0, TimerMode::Once)), 
            Name::new("WelcomeAutoAdvanceTimer")
        ));
        *timer_spawned = true;
        debug!("Spawned welcome auto-advance timer");
    } else if *wizard_state.get() != WizardState::Welcome {
        *timer_spawned = false;
    }
}

/// Validate that the current wizard step can be advanced
fn validate_current_step(
    current_state: WizardState,
    permission_manager: Option<&WizardPermissionManager>,
) -> bool {
    match current_state {
        WizardState::NotStarted => true, // Can always start
        WizardState::Welcome => true,    // Can always advance from welcome
        WizardState::CheckingPermissions => {
            // Can advance once all permissions have been checked
            if let Some(manager) = permission_manager {
                // Check if all required permissions have been checked (not unknown)
                let required_permissions = [
                    PermissionType::Accessibility,
                    PermissionType::ScreenCapture,
                    PermissionType::InputMonitoring,
                ];
                
                required_permissions.iter().all(|&perm| {
                    manager.get_cached_status(perm)
                        .map(|status| !matches!(status, PermissionStatus::Unknown))
                        .unwrap_or(false)
                })
            } else {
                true // Allow advance if no manager available
            }
        },
        WizardState::RequestingPermissions => {
            // Can advance when all required permissions are granted
            if let Some(manager) = permission_manager {
                manager.all_required_permissions_granted()
            } else {
                true // Allow advance if no manager available
            }
        },
        WizardState::SettingUpHotkeys => {
            // For now, always allow advance from hotkey setup
            // In the future, this could check if hotkeys are properly configured
            true
        },
        WizardState::Complete => false, // Cannot advance from complete state
    }
}

/// Enhanced validation result with detailed feedback
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub can_advance: bool,
    pub missing_requirements: Vec<PermissionType>,
    pub validation_message: String,
}

/// Validate wizard step with detailed feedback for users
pub fn validate_wizard_step(
    current_state: WizardState,
    permission_manager: Option<&WizardPermissionManager>,
) -> ValidationResult {
    match current_state {
        WizardState::NotStarted => ValidationResult {
            can_advance: true,
            missing_requirements: Vec::new(),
            validation_message: "Ready to start wizard".to_string(),
        },
        WizardState::Welcome => ValidationResult {
            can_advance: true,
            missing_requirements: Vec::new(),
            validation_message: "Welcome! Click Next to continue".to_string(),
        },
        WizardState::CheckingPermissions => {
            if let Some(manager) = permission_manager {
                let required_permissions = [
                    PermissionType::Accessibility,
                    PermissionType::ScreenCapture,
                    PermissionType::InputMonitoring,
                ];
                
                let missing: Vec<PermissionType> = required_permissions
                    .iter()
                    .filter(|&&perm| {
                        manager.get_cached_status(perm)
                            .map(|status| matches!(status, PermissionStatus::Unknown))
                            .unwrap_or(true)
                    })
                    .copied()
                    .collect();
                
                if missing.is_empty() {
                    ValidationResult {
                        can_advance: true,
                        missing_requirements: Vec::new(),
                        validation_message: "All permissions checked!".to_string(),
                    }
                } else {
                    ValidationResult {
                        can_advance: false,
                        missing_requirements: missing.clone(),
                        validation_message: format!("Checking {} permission(s)...", missing.len()),
                    }
                }
            } else {
                ValidationResult {
                    can_advance: true,
                    missing_requirements: Vec::new(),
                    validation_message: String::new(),
                }
            }
        },
        WizardState::RequestingPermissions => {
            if let Some(manager) = permission_manager {
                let required_permissions = [
                    PermissionType::Accessibility,
                    PermissionType::ScreenCapture,
                    PermissionType::InputMonitoring,
                ];
                
                let missing: Vec<PermissionType> = required_permissions
                    .iter()
                    .filter(|&&perm| {
                        manager.get_cached_status(perm)
                            .map(|status| !status.is_granted())
                            .unwrap_or(true)
                    })
                    .copied()
                    .collect();
                
                if missing.is_empty() {
                    ValidationResult {
                        can_advance: true,
                        missing_requirements: Vec::new(),
                        validation_message: "All required permissions granted!".to_string(),
                    }
                } else {
                    ValidationResult {
                        can_advance: false,
                        missing_requirements: missing.clone(),
                        validation_message: format!("{} required permission(s) still needed", missing.len()),
                    }
                }
            } else {
                ValidationResult {
                    can_advance: true,
                    missing_requirements: Vec::new(),
                    validation_message: String::new(),
                }
            }
        },
        WizardState::SettingUpHotkeys => ValidationResult {
            can_advance: true,
            missing_requirements: Vec::new(),
            validation_message: "Configure hotkeys or skip to continue".to_string(),
        },
        WizardState::Complete => ValidationResult {
            can_advance: false,
            missing_requirements: Vec::new(),
            validation_message: "Wizard complete!".to_string(),
        },
    }
}

/// System to manage navigation button states based on validation
pub fn update_navigation_button_states(
    mut button_query: Query<(&mut Visibility, &crate::wizard::WizardNavigationButton)>,
    wizard_state: Res<State<WizardState>>,
    permission_manager: Option<Res<WizardPermissionManager>>,
) {
    let validation_result = validate_wizard_step(
        *wizard_state.get(),
        permission_manager.as_deref(),
    );
    
    for (mut visibility, button) in button_query.iter_mut() {
        match button.action {
            NavigationAction::Next => {
                // Enable/disable next button based on validation
                if validation_result.can_advance {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            },
            NavigationAction::Back => {
                // Back button always visible except on first screen
                if *wizard_state.get() == WizardState::NotStarted || 
                   *wizard_state.get() == WizardState::Welcome {
                    *visibility = Visibility::Hidden;
                } else {
                    *visibility = Visibility::Visible;
                }
            },
            NavigationAction::Skip => {
                // Skip button visible for optional steps
                if *wizard_state.get() == WizardState::SettingUpHotkeys {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            },
            _ => {}
        }
    }
}

/// System to show visual feedback for validation status
pub fn show_validation_feedback(
    mut text_query: Query<&mut Text, With<ValidationFeedbackText>>,
    wizard_state: Res<State<WizardState>>,
    permission_manager: Option<Res<WizardPermissionManager>>,
) {
    let validation_result = validate_wizard_step(
        *wizard_state.get(),
        permission_manager.as_deref(),
    );
    
    for mut text in text_query.iter_mut() {
        text.0 = validation_result.validation_message.clone();
    }
}

/// Component marker for validation feedback text
#[derive(Component)]
pub struct ValidationFeedbackText;

/// System to handle skip confirmation for optional permissions
pub fn handle_skip_confirmation(
    mut navigation_events: EventReader<WizardNavigationRequest>,
    mut next_wizard_state: ResMut<NextState<WizardState>>,
    wizard_state: Res<State<WizardState>>,
) {
    for event in navigation_events.read() {
        if let WizardNavigationDirection::SkipTo(target_state) = event.direction {
            match *wizard_state.get() {
                WizardState::SettingUpHotkeys => {
                    info!("User confirmed skipping hotkey setup");
                    next_wizard_state.set(target_state);
                },
                _ => {
                    debug!("Skip not available for current state: {:?}", wizard_state.get());
                }
            }
        }
    }
}

/// Event for retrying a failed permission request
#[derive(Event, Debug, Clone)]
pub struct PermissionRetryRequest {
    pub permission_type: PermissionType,
}

/// System to handle retry requests for failed permissions
///
/// Enforces maximum retry limit (3 attempts) per permission to prevent
/// infinite retry loops. Tracks retry count in PermissionCard component.
pub fn handle_permission_retry(
    mut retry_events: EventReader<PermissionRetryRequest>,
    mut permission_requests: EventWriter<crate::wizard::WizardPermissionRequest>,
    mut card_query: Query<&mut PermissionCard>,
) {
    for retry_event in retry_events.read() {
        info!("Retry requested for: {:?}", retry_event.permission_type);
        
        // Find matching permission card and check retry limit
        for mut card in card_query.iter_mut() {
            if card.permission_type == retry_event.permission_type {
                if card.can_retry() {
                    card.increment_retry();
                    
                    permission_requests.write(crate::wizard::WizardPermissionRequest {
                        permission_type: retry_event.permission_type,
                        show_explanation: true,
                    });
                    
                    info!(
                        "Retry {}/{} for {:?}", 
                        card.retry_count, 
                        card.max_retries, 
                        retry_event.permission_type
                    );
                } else {
                    warn!(
                        "Max retries ({}) exceeded for {:?} - retry blocked", 
                        card.max_retries, 
                        retry_event.permission_type
                    );
                }
                break; // Found matching card, exit loop
            }
        }
    }
}

/// Component for retry button
#[derive(Component)]
pub struct PermissionRetryButton {
    pub permission_type: PermissionType,
}

/// Check if we can skip to a specific state from the current state
fn can_skip_to_state(current_state: WizardState, target_state: WizardState) -> bool {
    use WizardState::*;
    
    match (current_state, target_state) {
        // Can skip forward but not backward (except to complete)
        (Welcome, CheckingPermissions | RequestingPermissions | SettingUpHotkeys | Complete) => true,
        (CheckingPermissions, RequestingPermissions | SettingUpHotkeys | Complete) => true,
        (RequestingPermissions, SettingUpHotkeys | Complete) => true,
        (SettingUpHotkeys, Complete) => true,
        
        // Can skip to complete from any state
        (_, Complete) => true,
        
        // Cannot skip backward or to NotStarted
        _ => false,
    }
}

/// Component wrapper for Timer to make it a valid Bundle
#[derive(Component)]
pub struct WelcomeTimer(pub Timer);

/// Run condition to check if wizard navigation is active
pub fn wizard_navigation_active(wizard_state: Res<State<WizardState>>) -> bool {
    wizard_state.get().is_active()
}
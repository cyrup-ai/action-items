//! Wizard Plugin
//!
//! Main plugin that integrates all wizard functionality with Bevy ECS,
//! ecs-progress, and ecs-permissions for a complete permission setup wizard.

#![allow(dead_code)]

use bevy::prelude::*;
use action_items_ecs_progress::prelude::*;
use action_items_ecs_ui::prelude::*;

use crate::wizard::{
    // States
    WizardState, WizardStateTransitions,
    // Events
    WizardStartRequest, WizardStepComplete, WizardPermissionStatusChanged,
    WizardPermissionRequest, WizardBatchPermissionCheck, WizardPermissionCheckComplete,
    WizardCancelRequest, WizardNavigationRequest, WizardCompleteEvent,
    PermissionSetResponse,
    // Components and Resources
    WizardProgressTracker, FirstRunDetector,
    // Systems
    systems::{
        // Progress systems
        update_wizard_progress, handle_wizard_step_completion,
        initialize_wizard_progress, cleanup_wizard_progress,
        track_background_permission_operations, wizard_progress_active,
        // Permission systems
        handle_wizard_permission_checks, handle_wizard_permission_requests,
        monitor_ecs_permission_events, auto_check_permissions_on_state_enter,
        refresh_expired_permission_cache, monitor_permission_status_changes,
        cleanup_wizard_permissions, wizard_permissions_active,
        handle_permission_set_requests, WizardPermissionManager,
        handle_permission_error_recovery, PermissionErrorMessages,
        // Navigation systems
        handle_wizard_navigation, handle_wizard_cancellation, provide_navigation_hints,
        handle_timed_transitions, update_auto_advance_timers, spawn_welcome_auto_advance_timer,
        send_cancellation_response, wizard_navigation_active,
        // UI update systems
        update_permission_cards_with_animations, update_navigation_buttons_with_states,
        update_wizard_modal_theming, update_wizard_progress_with_layout,
        update_wizard_status_text, handle_permission_card_hover_effects,
        update_wizard_button_text, animate_ui_element_fade_in,
        update_permission_card_grid_positions, wizard_ui_active,
        handle_permission_card_interactions_system, populate_permission_card_content_system,
        integrate_icons_into_permission_cards, add_hover_effects_to_permission_card_buttons,
        // Responsive systems
        update_modal_responsiveness, update_permission_grid_layout,
        update_navigation_responsiveness, update_progress_indicator_responsiveness,
        handle_window_resize_events, process_responsive_updates,
        update_responsive_font_sizing,
        // UI systems
        setup_wizard_ui, show_wizard_ui, cleanup_wizard_ui,
        // System sets
        WizardCheckSet, WizardUISet, WizardProgressSet,
        WizardPermissionSet, WizardNavigationSet,
    },
};
use crate::wizard::first_run::{
    FirstRunFileOperations,
    // First-run system function imports
    initiate_first_run_check, handle_wizard_completion, check_should_start_wizard,
};
use crate::wizard::ui::theme::{
    WizardTheme, ThemeAnimationState, ThemePresetChangeEvent,
    // Theme system function imports
    apply_wizard_theme, update_permission_card_colors, update_animation_speeds,
    switch_theme_preset,
};
// Conditional import for macOS-specific theme function
#[cfg(target_os = "macos")]
use crate::wizard::ui::theme::detect_system_theme_changes;
// Observer system function imports
use crate::wizard::ui::observers::{register_wizard_observers, handle_auto_remove_entities};

/// Resource to store permissions that should be requested on first run
#[derive(Resource, Debug, Clone)]
pub struct WizardRequiredPermissions {
    pub permissions: Vec<crate::PermissionType>,
    pub reason: String,
}

impl Default for WizardRequiredPermissions {
    #[inline]
    fn default() -> Self {
        Self {
            permissions: Vec::new(),
            reason: String::from("Application requires these permissions to function properly"),
        }
    }
}

/// Main plugin for the permission setup wizard
/// 
/// Provides a complete permission setup experience integrated with:
/// - ecs-progress for smooth state transitions
/// - ecs-permissions for system permission management
/// - ecs-ui for responsive user interface
/// 
/// # Usage
/// 
/// ```rust
/// use bevy::prelude::*;
/// use action_items_ecs_permissions::wizard::PermissionWizardPlugin;
/// use action_items_ecs_permissions::PermissionType;
/// 
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(
///         PermissionWizardPlugin::default()
///             .with_required_permissions(vec![
///                 PermissionType::Accessibility,
///                 PermissionType::Camera,
///             ])
///     )
///     .run();
/// ```
pub struct PermissionWizardPlugin {
    /// Whether to auto-start wizard on first run
    auto_start_on_first_run: bool,
    /// Whether to show wizard in debug builds even if permissions exist
    debug_force_show: bool,
    /// Whether to enable wizard state transitions
    enable_state_transitions: bool,
    /// Whether to enable progress tracking integration
    enable_progress_tracking: bool,
    /// Permissions to request on first run
    required_permissions: Option<Vec<crate::PermissionType>>,
    /// Reason message for permission requests
    permission_reason: Option<String>,
}

impl Default for PermissionWizardPlugin {
    #[inline]
    fn default() -> Self {
        Self {
            auto_start_on_first_run: true,
            debug_force_show: cfg!(debug_assertions),
            enable_state_transitions: true,
            enable_progress_tracking: true,
            required_permissions: None,
            permission_reason: None,
        }
    }
}

impl PermissionWizardPlugin {
    /// Create a new wizard plugin with default settings
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Configure auto-start behavior on first run
    #[inline]
    pub fn with_auto_start(mut self, auto_start: bool) -> Self {
        self.auto_start_on_first_run = auto_start;
        self
    }
    
    /// Configure debug force show (useful for testing)
    #[inline]
    pub fn with_debug_force_show(mut self, force_show: bool) -> Self {
        self.debug_force_show = force_show;
        self
    }
    
    /// Enable or disable state transitions
    #[inline]
    pub fn with_state_transitions(mut self, enable: bool) -> Self {
        self.enable_state_transitions = enable;
        self
    }
    
    /// Enable or disable progress tracking
    #[inline]
    pub fn with_progress_tracking(mut self, enable: bool) -> Self {
        self.enable_progress_tracking = enable;
        self
    }
    
    /// Configure permissions to request on first run
    #[inline]
    pub fn with_required_permissions(mut self, permissions: Vec<crate::PermissionType>) -> Self {
        self.required_permissions = Some(permissions);
        self
    }
    
    /// Set the reason message shown to users for permission requests
    #[inline]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.permission_reason = Some(reason.into());
        self
    }
}

impl Plugin for PermissionWizardPlugin {
    fn build(&self, app: &mut App) {
        // Add ecs-ui plugin group FIRST
        app.add_plugins(UiLunexPlugins);
        
        // Initialize wizard state machine
        app.init_state::<WizardState>();
        
        // Add wizard events
        app.add_event::<WizardStartRequest>()
           .add_event::<WizardStepComplete>()
           .add_event::<WizardPermissionStatusChanged>()
           .add_event::<WizardPermissionRequest>()
           .add_event::<WizardBatchPermissionCheck>()
           .add_event::<WizardPermissionCheckComplete>()
           .add_event::<WizardCancelRequest>()
           .add_event::<WizardNavigationRequest>()
           .add_event::<WizardCompleteEvent>()
           .add_event::<ThemePresetChangeEvent>()
           .add_event::<PermissionSetResponse>();
        
        // Add wizard resources
        app.init_resource::<WizardProgressTracker>()
           .init_resource::<WizardPermissionManager>()
           .init_resource::<PermissionErrorMessages>()
           .init_resource::<FirstRunDetector>()
           .init_resource::<WizardStateTransitions>()
           .init_resource::<WizardTheme>()
           .init_resource::<ThemeAnimationState>();
        
        // Initialize required permissions resource with configured values
        let required_perms = WizardRequiredPermissions {
            permissions: self.required_permissions.clone().unwrap_or_default(),
            reason: self.permission_reason.clone().unwrap_or_else(|| 
                String::from("Application requires these permissions to function properly")
            ),
        };
        app.insert_resource(required_perms);
        
        // Configure system sets with ecs-ui integration
        app.configure_sets(
            Update,
            (
                WizardCheckSet,
                WizardPermissionSet,
                WizardProgressSet,
                WizardUISet.after(UiSystems::PostCompute), // Order after ecs-ui
                WizardNavigationSet,
            )
                .chain()
                .run_if(wizard_active),
        );
        
        // Wizard initialization and lifecycle systems
        app.add_systems(
            OnEnter(WizardState::Welcome),
            (initialize_wizard_progress, setup_wizard_ui)
        );
        
        app.add_systems(
            OnExit(WizardState::Complete),
            (cleanup_wizard_progress, cleanup_wizard_permissions)
        );
        
        app.add_systems(
            OnExit(WizardState::NotStarted),
            cleanup_wizard_progress
        );
        
        // Permission checking and management systems
        if self.enable_state_transitions {
            app.add_systems(
                Update,
                (
                    // Handle permission set requests (NEW)
                    handle_permission_set_requests,
                    // Auto-check permissions when entering checking state
                    auto_check_permissions_on_state_enter,
                    // Handle batch permission checks with caching
                    handle_wizard_permission_checks,
                    // Handle individual permission requests
                    handle_wizard_permission_requests,
                    // Monitor ecs-permissions events and update wizard state
                    monitor_ecs_permission_events,
                    // Refresh expired cache entries
                    refresh_expired_permission_cache,
                    // Monitor and log permission status changes
                    monitor_permission_status_changes,
                    // Handle permission error recovery with user-friendly messages
                    handle_permission_error_recovery,
                )
                    .in_set(WizardPermissionSet)
                    .run_if(wizard_permissions_active),
            );
            
            // Navigation systems for wizard flow control
            app.add_systems(
                Update,
                (
                    // Handle back/next/skip navigation requests
                    handle_wizard_navigation,
                    // Handle wizard cancellation requests
                    handle_wizard_cancellation,
                    // Send cancellation response events
                    send_cancellation_response,
                    // Provide automatic navigation hints
                    provide_navigation_hints,
                    // Handle timed transitions (e.g. welcome screen auto-advance)
                    handle_timed_transitions,
                    // Update auto-advance timers
                    update_auto_advance_timers,
                    // Spawn welcome screen auto-advance timer
                    spawn_welcome_auto_advance_timer,
                )
                    .in_set(WizardNavigationSet)
                    .run_if(wizard_navigation_active),
            );
        }
        
        // Progress tracking systems  
        if self.enable_progress_tracking {
            app.add_systems(
                Update,
                (
                    // Update overall wizard progress based on permission changes
                    update_wizard_progress,
                    // Handle step completion events and trigger transitions
                    handle_wizard_step_completion,
                    // Track background permission operations
                    track_background_permission_operations,
                )
                    .in_set(WizardProgressSet)
                    .run_if(wizard_progress_active),
            );
            
            // Integrate with ecs-progress for state transitions
            if self.enable_state_transitions {
                app.add_plugins(
                    ProgressPlugin::<WizardState>::new()
                        .with_transition(WizardState::Welcome, WizardState::CheckingPermissions)
                        .with_transition(WizardState::CheckingPermissions, WizardState::RequestingPermissions)
                        .with_transition(WizardState::RequestingPermissions, WizardState::SettingUpHotkeys)
                        .with_transition(WizardState::SettingUpHotkeys, WizardState::Complete)
                        .auto_clear(true, false) // Clear on enter, keep on exit
                        .check_in(PostUpdate)
                );
            }
        }
        
        // UI update and animation systems
        app.add_systems(
            Update,
            (
                // Show/hide wizard UI based on state
                show_wizard_ui,
                // Update permission cards with animations
                update_permission_cards_with_animations,
                // Integrate icons into permission cards
                integrate_icons_into_permission_cards,
                // Add hover effects to permission card buttons
                add_hover_effects_to_permission_card_buttons,
                // Update navigation buttons with current state
                update_navigation_buttons_with_states,
                // Update wizard modal theming
                update_wizard_modal_theming,
                // Update wizard progress with responsive layout
                update_wizard_progress_with_layout,
                // Update wizard status text
                update_wizard_status_text,
                // Handle permission card interactions
                handle_permission_card_interactions_system,
                // Populate permission card content
                populate_permission_card_content_system,
                // Handle permission card hover effects
                handle_permission_card_hover_effects,
                // Update wizard button text based on state
                update_wizard_button_text,
                // Animate UI element fade-in effects
                animate_ui_element_fade_in,
                // Update permission card grid positions
                update_permission_card_grid_positions,
            )
                .in_set(WizardUISet)
                .run_if(wizard_ui_active),
        );
        
        // Responsive UI systems
        app.add_systems(
            Update,
            (
                // Update modal responsiveness
                update_modal_responsiveness,
                // Update permission grid layout
                update_permission_grid_layout,
                // Update navigation responsiveness
                update_navigation_responsiveness,
                // Update progress indicator responsiveness
                update_progress_indicator_responsiveness,
                // Handle window resize events
                handle_window_resize_events,
                // Process responsive updates
                process_responsive_updates,
                // Update responsive font sizing
                update_responsive_font_sizing,
            )
                .in_set(WizardUISet)
                .run_if(wizard_ui_active),
        );
        
        // Theme systems
        app.add_systems(
            OnEnter(WizardState::Welcome),
            apply_wizard_theme
        );

        app.add_systems(
            Update,
            (
                update_permission_card_colors,
                update_animation_speeds,
                switch_theme_preset,
                #[cfg(target_os = "macos")]
                detect_system_theme_changes,
            )
                .in_set(WizardUISet)
                .run_if(wizard_ui_active),
        );
        
        // First-run detection and auto-start systems
        if self.auto_start_on_first_run {
            app.add_systems(
                PreUpdate,
                (
                    initiate_first_run_check,
                    check_first_run_and_auto_start,
                    check_should_start_wizard,
                )
                    .in_set(WizardCheckSet)
                    .run_if(not(wizard_active))
            );
            
            app.add_systems(
                OnExit(WizardState::Complete),
                handle_wizard_completion
            );
            
            // Add missing resource
            app.init_resource::<FirstRunFileOperations>();
        }
        
        // Observer system registrations
        app.add_systems(
            OnEnter(WizardState::Welcome),
            register_wizard_observers
        );

        app.add_systems(
            Update,
            handle_auto_remove_entities
                .in_set(WizardUISet)
                .run_if(wizard_ui_active)
        );
        
        // Cleanup systems
        app.add_systems(
            OnExit(WizardState::Complete),
            cleanup_wizard_ui
        );
        
        // Debug systems
        #[cfg(debug_assertions)]
        if self.debug_force_show {
            app.add_systems(
                Update,
                debug_wizard_state_changes
                    .run_if(resource_changed::<State<WizardState>>)
            );
        }
    }
}

/// Run condition to check if wizard is currently active
fn wizard_active(wizard_state: Res<State<WizardState>>) -> bool {
    wizard_state.get().is_active()
}

/// System to check for first run and auto-start wizard with configured permissions
fn check_first_run_and_auto_start(
    mut first_run_detector: ResMut<FirstRunDetector>,
    mut permission_requests: EventWriter<crate::PermissionSetRequest>,
    wizard_state: Res<State<WizardState>>,
    required_perms: Res<WizardRequiredPermissions>,
) {
    // Only check on first run
    if first_run_detector.check_completed {
        return;
    }
    
    // Only auto-start if wizard is not already active
    if wizard_state.get().is_active() {
        return;
    }
    
    // Check if this is first run and permissions are configured
    if first_run_detector.is_first_run && !first_run_detector.wizard_completed {
        if !required_perms.permissions.is_empty() {
            info!("First run detected - requesting {} permissions through wizard", required_perms.permissions.len());
            
            // Build permission request with all configured permissions
            let mut request = crate::PermissionSetRequest::new("first_run_wizard");
            for perm in &required_perms.permissions {
                request = request.with_required(*perm);
            }
            request = request
                .with_reason(&required_perms.reason)
                .with_wizard_fallback(true);
            
            permission_requests.write(request);
        } else {
            // No permissions configured - wizard will not auto-start
            info!("First run detected but no permissions configured - skipping wizard auto-start");
        }
        
        first_run_detector.check_completed = true;
    }
}

/// Debug system to log wizard state changes
#[cfg(debug_assertions)]
fn debug_wizard_state_changes(
    wizard_state: Res<State<WizardState>>,
    progress_tracker: Option<Res<WizardProgressTracker>>,
    permission_manager: Option<Res<WizardPermissionManager>>,
) {
    let current_state = wizard_state.get();
    debug!("Wizard state changed to: {:?}", current_state);
    
    if let Some(tracker) = progress_tracker {
        let (completed, total) = tracker.calculate_progress();
        debug!("Wizard progress: {}/{} permissions", completed, total);
    }
    
    if let Some(manager) = permission_manager {
        let active_requests = manager.active_request_count();
        if active_requests > 0 {
            debug!("Active permission requests: {}", active_requests);
        }
    }
}
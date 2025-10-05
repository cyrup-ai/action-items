//! Wizard State Management
//!
//! Defines the core state machine for the permission setup wizard using Bevy's
//! built-in state system for zero-allocation state transitions.

use bevy::prelude::*;

/// Core wizard state machine with optimized transitions
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum WizardState {
    /// Wizard has not been started (default state)
    #[default]
    NotStarted,
    
    /// Welcome screen introducing the wizard
    Welcome,
    
    /// Background permission status checking
    CheckingPermissions,
    
    /// Active permission requesting phase
    RequestingPermissions,
    
    /// Setting up hotkeys and preferences
    SettingUpHotkeys,
    
    /// Wizard completed successfully
    Complete,
}

#[allow(dead_code)] // Public API methods for wizard state management
impl WizardState {
    /// Check if the wizard is currently active (not NotStarted or Complete)
    #[inline]
    pub fn is_active(self) -> bool {
        !matches!(self, WizardState::NotStarted | WizardState::Complete)
    }
    
    /// Check if the wizard is in a permission-related state
    #[inline]
    pub fn is_permission_phase(self) -> bool {
        matches!(
            self,
            WizardState::CheckingPermissions | WizardState::RequestingPermissions
        )
    }
    
    /// Get the next logical state in the wizard flow
    #[inline]
    pub fn next_state(self) -> Option<Self> {
        match self {
            WizardState::NotStarted => Some(WizardState::Welcome),
            WizardState::Welcome => Some(WizardState::CheckingPermissions),
            WizardState::CheckingPermissions => Some(WizardState::RequestingPermissions),
            WizardState::RequestingPermissions => Some(WizardState::SettingUpHotkeys),
            WizardState::SettingUpHotkeys => Some(WizardState::Complete),
            WizardState::Complete => None,
        }
    }
    
    /// Get the previous logical state in the wizard flow
    #[inline]
    pub fn previous_state(self) -> Option<Self> {
        match self {
            WizardState::NotStarted => None,
            WizardState::Welcome => Some(WizardState::NotStarted),
            WizardState::CheckingPermissions => Some(WizardState::Welcome),
            WizardState::RequestingPermissions => Some(WizardState::CheckingPermissions),
            WizardState::SettingUpHotkeys => Some(WizardState::RequestingPermissions),
            WizardState::Complete => Some(WizardState::SettingUpHotkeys),
        }
    }
    
    /// Get human-readable description of the current state
    #[inline]
    pub fn description(self) -> &'static str {
        match self {
            WizardState::NotStarted => "Wizard not started",
            WizardState::Welcome => "Welcome to Action Items",
            WizardState::CheckingPermissions => "Checking system permissions",
            WizardState::RequestingPermissions => "Setting up permissions",
            WizardState::SettingUpHotkeys => "Configuring hotkeys",
            WizardState::Complete => "Setup complete",
        }
    }
    
    /// Get the progress percentage for this state (0.0 to 1.0)
    #[inline]
    pub fn progress_percentage(self) -> f32 {
        match self {
            WizardState::NotStarted => 0.0,
            WizardState::Welcome => 0.1,
            WizardState::CheckingPermissions => 0.3,
            WizardState::RequestingPermissions => 0.7,
            WizardState::SettingUpHotkeys => 0.9,
            WizardState::Complete => 1.0,
        }
    }
}

/// State transition resource for managing wizard flow
#[derive(Resource, Debug, Clone)]
pub struct WizardStateTransitions {
    /// Whether automatic state transitions are enabled
    pub auto_transitions: bool,
    
    /// Minimum time to spend in each state (for UX smoothness)
    pub minimum_state_duration: std::time::Duration,
    
    /// Timestamp when current state was entered
    pub state_entered_at: std::time::Instant,
}

impl Default for WizardStateTransitions {
    fn default() -> Self {
        Self {
            auto_transitions: true,
            minimum_state_duration: std::time::Duration::from_millis(500),
            state_entered_at: std::time::Instant::now(),
        }
    }
}

#[allow(dead_code)] // Public API methods for wizard state transitions
impl WizardStateTransitions {
    /// Check if enough time has passed to allow state transition
    #[inline]
    pub fn can_transition(&self) -> bool {
        self.auto_transitions && 
        self.state_entered_at.elapsed() >= self.minimum_state_duration
    }
    
    /// Mark that a new state has been entered
    #[inline]
    pub fn mark_state_entered(&mut self) {
        self.state_entered_at = std::time::Instant::now();
    }
    
    /// Disable automatic transitions (for manual control)
    #[inline]
    pub fn disable_auto_transitions(&mut self) {
        self.auto_transitions = false;
    }
    
    /// Enable automatic transitions
    #[inline]
    pub fn enable_auto_transitions(&mut self) {
        self.auto_transitions = true;
    }
}
//! Privacy indicators component system for AI menu
//!
//! Zero-allocation components for blazing-fast privacy status display with real-time updates.

use std::time::Duration;

use bevy::prelude::*;

use super::privacy_events::IndicatorType;
use action_items_ecs_ui::{
    UiComponentTarget, UiVisibilityAnimationType, UiVisibilityEvent,
};

/// Core privacy indicators component for AI menu display
/// Tracks all privacy states with efficient change detection
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct PrivacyIndicators {
    /// Full user control over AI interactions
    pub full_control: bool,
    /// No data collection by AI providers
    pub no_collection: bool,
    /// End-to-end encryption active for AI communications
    pub encrypted: bool,
    /// Info panel expansion state
    pub info_expanded: bool,
    /// Cache for avoiding unnecessary UI updates
    pub last_update: std::time::Instant,
}

impl Default for PrivacyIndicators {
    fn default() -> Self {
        Self {
            full_control: false,
            no_collection: false,
            encrypted: false,
            info_expanded: false,
            last_update: std::time::Instant::now(),
        }
    }
}

impl PrivacyIndicators {
    /// Create new privacy indicators with specified states
    #[inline]
    pub fn new(full_control: bool, no_collection: bool, encrypted: bool) -> Self {
        Self {
            full_control,
            no_collection,
            encrypted,
            info_expanded: false,
            last_update: std::time::Instant::now(),
        }
    }

    /// Update privacy states and return true if any changed
    #[inline]
    pub fn update_states(
        &mut self,
        full_control: bool,
        no_collection: bool,
        encrypted: bool,
    ) -> bool {
        let changed = self.full_control != full_control
            || self.no_collection != no_collection
            || self.encrypted != encrypted;

        if changed {
            self.full_control = full_control;
            self.no_collection = no_collection;
            self.encrypted = encrypted;
            self.last_update = std::time::Instant::now();
        }

        changed
    }

    /// Toggle info panel expansion state
    #[inline]
    pub fn toggle_info(&mut self, ui_events: &mut EventWriter<UiVisibilityEvent>) {
        let target_expanded = !self.info_expanded;
        self.info_expanded = target_expanded;
        self.last_update = std::time::Instant::now();

        // Send animated expand/collapse event for privacy indicator
        ui_events.write(UiVisibilityEvent::with_animation(
            target_expanded,
            Duration::from_millis(180),
            UiVisibilityAnimationType::Scale,
            UiComponentTarget::SecondaryPanel,
        ));
    }
}

/// Interactive privacy icon button component
/// Handles hover states and click interactions for blazing-fast UI responsiveness
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct PrivacyIconButton {
    /// Type of privacy indicator this button represents
    pub indicator_type: IndicatorType,
    /// Current hover state for visual feedback
    pub hover_state: HoverState,
    /// Button interaction state
    pub interaction_state: InteractionState,
    /// Cache for efficient hover state transitions
    pub last_hover_change: std::time::Instant,
}

impl PrivacyIconButton {
    /// Create new privacy icon button for specified indicator type
    /// Part of public API - used in privacy_icons.rs for UI setup
    #[inline]
    pub fn new(indicator_type: IndicatorType) -> Self {
        Self {
            indicator_type,
            hover_state: HoverState::Normal,
            interaction_state: InteractionState::None,
            last_hover_change: std::time::Instant::now(),
        }
    }

    /// Update hover state and return true if changed
    #[inline]
    pub fn set_hover_state(&mut self, new_state: HoverState) -> bool {
        if self.hover_state != new_state {
            self.hover_state = new_state;
            self.last_hover_change = std::time::Instant::now();
            true
        } else {
            false
        }
    }
}

/// Hover states for privacy icon buttons with efficient transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum HoverState {
    /// Default state - no hover interaction
    Normal,
    /// Mouse hovering over button
    Hovered,
    /// Button being pressed
    Pressed,
}

/// Button interaction states for comprehensive user feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum InteractionState {
    /// No interaction occurring
    None,
    /// Button is being clicked/pressed
    Clicked,
    /// Button click was completed
    Released,
}

/// Privacy configuration resource for application-wide privacy settings
/// Provides centralized source of truth for privacy indicator calculations
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct PrivacyConfiguration {
    /// Whether data collection is enabled by any AI provider
    pub data_collection_enabled: bool,
    /// Whether user has full control over AI interactions
    pub full_user_control: bool,
    /// Whether encryption is active for AI communications
    pub encryption_active: bool,
    /// Current privacy level based on active providers
    pub provider_privacy_level: PrivacyLevel,
    /// Last configuration change timestamp for change detection
    pub last_modified: std::time::Instant,
}

impl Default for PrivacyConfiguration {
    fn default() -> Self {
        Self::secure_default()
    }
}

impl PrivacyConfiguration {
    /// Create new privacy configuration with secure defaults
    #[inline]
    pub fn secure_default() -> Self {
        Self {
            data_collection_enabled: false,
            full_user_control: true,
            encryption_active: true,
            provider_privacy_level: PrivacyLevel::Maximum,
            last_modified: std::time::Instant::now(),
        }
    }

    /// Update configuration and return true if any values changed
    #[inline]
    pub fn update_config(
        &mut self,
        data_collection: bool,
        full_control: bool,
        encryption: bool,
        privacy_level: PrivacyLevel,
    ) -> bool {
        let changed = self.data_collection_enabled != data_collection
            || self.full_user_control != full_control
            || self.encryption_active != encryption
            || self.provider_privacy_level != privacy_level;

        if changed {
            self.data_collection_enabled = data_collection;
            self.full_user_control = full_control;
            self.encryption_active = encryption;
            self.provider_privacy_level = privacy_level;
            self.last_modified = std::time::Instant::now();
        }

        changed
    }

    /// Calculate privacy indicators based on current configuration
    #[inline]
    pub fn calculate_indicators(&self) -> (bool, bool, bool) {
        let full_control =
            self.full_user_control && self.provider_privacy_level != PrivacyLevel::None;
        let no_collection = !self.data_collection_enabled;
        let encrypted =
            self.encryption_active && self.provider_privacy_level >= PrivacyLevel::Standard;

        (full_control, no_collection, encrypted)
    }
}

/// Privacy levels for different AI provider configurations
/// Enables granular privacy control with efficient comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
pub enum PrivacyLevel {
    /// No privacy protections
    None = 0,
    /// Basic privacy protections
    Basic = 1,
    /// Standard privacy protections
    Standard = 2,
    /// Enhanced privacy protections
    Enhanced = 3,
    /// Maximum privacy protections
    Maximum = 4,
}

impl Default for PrivacyLevel {
    fn default() -> Self {
        Self::Maximum
    }
}

/// Privacy info panel component for detailed privacy information display
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct PrivacyInfoPanel {
    /// Whether the panel is currently expanded
    pub expanded: bool,
    /// Animation progress for smooth expand/collapse (0.0 to 1.0)
    pub animation_progress: f32,
    /// Target animation state
    pub target_expanded: bool,
    /// Panel content height for smooth animations
    pub content_height: f32,
}

impl PrivacyInfoPanel {
    // new() function removed - not used in current implementation
    // If needed in future, will be re-added with actual usage

    /// Start expand animation
    #[inline]
    pub fn start_expand(&mut self) {
        self.target_expanded = true;
    }

    /// Start collapse animation  
    #[inline]
    pub fn start_collapse(&mut self) {
        self.target_expanded = false;
    }

    /// Update animation progress with smooth interpolation
    #[inline]
    pub fn update_animation(&mut self, delta_time: f32) -> bool {
        const ANIMATION_SPEED: f32 = 4.0; // Animations per second

        let target_progress = if self.target_expanded { 1.0 } else { 0.0 };
        let progress_diff = target_progress - self.animation_progress;

        if progress_diff.abs() > 0.001 {
            self.animation_progress += progress_diff * ANIMATION_SPEED * delta_time;
            self.animation_progress = self.animation_progress.clamp(0.0, 1.0);

            self.expanded = self.animation_progress > 0.001;
            true
        } else {
            self.animation_progress = target_progress;
            self.expanded = target_progress > 0.5;
            false
        }
    }
}

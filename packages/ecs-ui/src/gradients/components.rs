//! Gradient component for theme-based gradient management with animation support

use bevy::prelude::*;
use crate::theme::colors::BackgroundGradient;
use super::states::{GradientComponentType, GradientInteractionState};
use super::theme::GradientTheme;

/// GradientComponent for managing gradient state and animations
///
/// Integrates with GradientTheme resource and supports state transitions with custom gradients.
/// 
/// # Example
/// ```rust
/// let gradient_comp = GradientComponent::list_item()
///     .with_transition_speed(0.3);
/// 
/// commands.spawn((
///     NodeBundle::default(),
///     gradient_comp,
///     Interaction::default(),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub struct GradientComponent {
    /// Component type for theme selection
    pub component_type: GradientComponentType,
    /// Current interaction state for gradient selection
    pub interaction_state: GradientInteractionState,
    /// Custom gradient override (if any)
    pub custom_gradient: Option<BackgroundGradient>,
    /// Animation speed for state transitions (seconds)
    pub transition_speed: f32,
    /// Accumulated time since transition started (in seconds)
    pub elapsed_transition_time: f32,
    /// Previous interaction state for detecting state changes
    pub previous_state: Option<GradientInteractionState>,
}

impl Default for GradientComponent {
    fn default() -> Self {
        Self {
            component_type: GradientComponentType::PrimaryContainer,
            interaction_state: GradientInteractionState::Default,
            custom_gradient: None,
            transition_speed: 0.2, // 200ms transition
            elapsed_transition_time: 0.0,
            previous_state: None,
        }
    }
}

impl GradientComponent {
    /// Create gradient component for primary container
    #[inline]
    pub fn primary_container() -> Self {
        Self {
            component_type: GradientComponentType::PrimaryContainer,
            elapsed_transition_time: 0.0,
            previous_state: None,
            ..Default::default()
        }
    }

    /// Create gradient component for secondary container
    #[inline]
    pub fn secondary_container() -> Self {
        Self {
            component_type: GradientComponentType::SecondaryContainer,
            elapsed_transition_time: 0.0,
            previous_state: None,
            ..Default::default()
        }
    }

    /// Create gradient component for list item (renamed from result_item)
    /// 
    /// Generic constructor for any interactive list item.
    #[inline]
    pub fn list_item() -> Self {
        Self {
            component_type: GradientComponentType::ListItem,
            elapsed_transition_time: 0.0,
            previous_state: None,
            ..Default::default()
        }
    }

    /// Set custom gradient override
    /// 
    /// When set, this gradient will be used instead of theme gradients.
    #[inline]
    pub fn with_custom_gradient(mut self, gradient: BackgroundGradient) -> Self {
        self.custom_gradient = Some(gradient);
        self
    }

    /// Set transition speed for animations
    /// 
    /// Clamped to 0.05-2.0 seconds for reasonable animation speeds.
    #[inline]
    pub fn with_transition_speed(mut self, speed: f32) -> Self {
        self.transition_speed = speed.clamp(0.05, 2.0);
        self
    }

    /// Get current gradient based on component type and state
    ///
    /// Resolves the appropriate gradient by:
    /// 1. Using custom gradient if set
    /// 2. Checking interaction state for ListItem types
    /// 3. Falling back to component type default
    #[inline]
    pub fn get_current_gradient<'a>(&'a self, theme: &'a GradientTheme) -> &'a BackgroundGradient {
        // Use custom gradient if available
        if let Some(ref custom) = self.custom_gradient {
            return custom;
        }

        // Select gradient based on interaction state and component type
        match (self.component_type, self.interaction_state) {
            (GradientComponentType::ListItem, GradientInteractionState::Hover) => {
                &theme.list_item_hover
            },
            (GradientComponentType::ListItem, GradientInteractionState::Selected) => {
                &theme.list_item_selected
            },
            (component_type, _) => theme.get_component_gradient(component_type),
        }
    }
}

use bevy::prelude::*;

/// Common trait for UI components that can be styled with gradients
pub trait GradientStyled {
    /// Apply gradient styling to the component
    fn apply_gradient(&mut self, gradient: &super::layout::BackgroundGradient);

    /// Get current gradient state
    fn current_gradient(&self) -> Option<&super::layout::BackgroundGradient>;
}

/// Common trait for interactive UI components
pub trait Interactive {
    /// Handle interaction state changes
    fn set_interaction_state(&mut self, state: InteractionState);

    /// Get current interaction state
    fn interaction_state(&self) -> InteractionState;

    /// Check if component is currently interactive
    fn is_interactive(&self) -> bool {
        matches!(
            self.interaction_state(),
            InteractionState::Default | InteractionState::Hover
        )
    }
}

/// Interaction states for UI components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionState {
    Default,
    Hover,
    Selected,
    Pressed,
    Disabled,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self::Default
    }
}

/// Common trait for components that can display content
pub trait ContentDisplay {
    type Content;

    /// Set the content to display
    fn set_content(&mut self, content: Self::Content);

    /// Get current content
    fn content(&self) -> &Self::Content;

    /// Clear the content
    fn clear_content(&mut self);
}

/// Common trait for components with validation states
pub trait Validatable {
    /// Validate the component's current state
    fn is_valid(&self) -> bool;

    /// Get validation error message if invalid
    fn validation_error(&self) -> Option<String>;
}

//! Interactive gradient component for simple color-based state management

use bevy::prelude::*;
use crate::theme::ColorPalette;

/// InteractiveGradient component for gradient state management
/// 
/// Provides color-based gradient transitions for interactive UI elements.
#[derive(Component, Debug, Clone)]
pub struct InteractiveGradient {
    /// Default color when no interaction
    pub default_color: Color,
    /// Color when hovering
    pub hover_color: Color,
    /// Optional color when selected (None = no selection state)
    pub selected_color: Option<Color>,
    /// Animation transition speed (0.1-1.0 seconds)
    pub transition_speed: f32,
}

impl InteractiveGradient {
    /// Create interactive gradient for list items
    /// 
    /// Generic constructor for any interactive list item (search results, menus, etc.)
    /// 
    /// # Example
    /// ```rust
    /// let gradient = InteractiveGradient::list_item(&theme);
    /// commands.spawn((
    ///     NodeBundle::default(),
    ///     gradient,
    ///     Interaction::default(),
    /// ));
    /// ```
    #[inline]
    pub fn list_item(theme: &ColorPalette) -> Self {
        Self {
            default_color: theme.list_item_background(),
            hover_color: theme.list_item_hover_background(),
            selected_color: Some(theme.list_item_selected_background()),
            transition_speed: 0.2, // Fast but smooth transitions
        }
    }

    /// Create interactive gradient for search input
    /// 
    /// Specialized constructor for search input fields.
    /// No selection state since inputs don't have selected variants.
    #[inline]
    pub fn search_input(theme: &ColorPalette) -> Self {
        Self {
            default_color: theme.search_input_background(),
            hover_color: theme.search_input_background(), // Same as default
            selected_color: None,
            transition_speed: 0.1,
        }
    }
}

//! Gradient theme resource with presets for dark and high-contrast modes

use bevy::prelude::*;
use crate::theme::colors::{BackgroundGradient, GradientFactory};
use super::states::GradientComponentType;

/// Professional gradient theme system for modern UI aesthetics
/// 
/// Resource containing all gradient definitions for consistent theming.
/// Use with GradientComponent for automatic theme-based gradient application.
/// 
/// # Example
/// ```rust
/// fn setup(mut commands: Commands) {
///     commands.insert_resource(GradientTheme::professional_dark());
/// }
/// ```
#[derive(Resource, Debug, Clone)]
pub struct GradientTheme {
    /// Primary container gradient (main launcher background)
    pub primary_container: BackgroundGradient,
    /// Secondary container gradient (elevated surfaces like search input)
    pub secondary_container: BackgroundGradient,
    /// List item gradient (individual list items) - renamed from result_item
    pub list_item: BackgroundGradient,
    /// List item hover gradient (interactive state) - renamed from result_item_hover
    pub list_item_hover: BackgroundGradient,
    /// List item selected gradient (active selection) - renamed from result_item_selected
    pub list_item_selected: BackgroundGradient,
    /// Text overlay gradient (for enhanced readability)
    pub text_overlay: BackgroundGradient,
    /// Border accent gradient (subtle borders and dividers)
    pub border_accent: BackgroundGradient,
    /// Success state gradient (confirmations, positive actions)
    pub success_state: BackgroundGradient,
    /// Warning state gradient (cautions, attention needed)
    pub warning_state: BackgroundGradient,
    /// Error state gradient (failures, critical issues)
    pub error_state: BackgroundGradient,
}

impl GradientTheme {
    /// Get gradient for specific component type
    /// 
    /// Maps component types to their corresponding theme gradients.
    pub fn get_component_gradient(
        &self,
        component_type: GradientComponentType,
    ) -> &BackgroundGradient {
        match component_type {
            GradientComponentType::PrimaryContainer => &self.primary_container,
            GradientComponentType::SecondaryContainer => &self.secondary_container,
            GradientComponentType::ListItem => &self.list_item,
            GradientComponentType::ListItemSelected => &self.list_item_selected,
            GradientComponentType::TextOverlay => &self.text_overlay,
            GradientComponentType::BorderAccent => &self.border_accent,
            GradientComponentType::SuccessState => &self.success_state,
            GradientComponentType::WarningState => &self.warning_state,
            GradientComponentType::ErrorState => &self.error_state,
        }
    }

    /// Create professional dark theme
    /// 
    /// Modern dark theme with subtle gradients and blue accents.
    /// Optimized for Raycast-like launcher aesthetics.
    pub fn professional_dark() -> Self {
        Self {
            primary_container: GradientFactory::dark_professional(0.08, 0.04),
            secondary_container: GradientFactory::dark_professional(0.12, 0.08),
            list_item: GradientFactory::dark_professional(0.10, 0.06),
            list_item_hover: GradientFactory::subtle_accent(
                Color::srgba(0.15, 0.15, 0.18, 0.95),
                0.3,
            ),
            list_item_selected: GradientFactory::subtle_accent(
                Color::srgba(0.2, 0.4, 0.8, 0.9),
                0.5,
            ),
            text_overlay: BackgroundGradient::new(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            border_accent: BackgroundGradient::new(Color::srgba(0.3, 0.3, 0.35, 0.8)),
            success_state: GradientFactory::subtle_accent(Color::srgba(0.2, 0.8, 0.4, 0.9), 0.4),
            warning_state: GradientFactory::subtle_accent(Color::srgba(0.9, 0.7, 0.2, 0.9), 0.4),
            error_state: GradientFactory::subtle_accent(Color::srgba(0.9, 0.3, 0.3, 0.9), 0.4),
        }
    }

    /// Create high contrast theme
    /// 
    /// High contrast theme for accessibility.
    /// Uses solid colors instead of gradients for maximum clarity.
    pub fn high_contrast() -> Self {
        Self {
            primary_container: BackgroundGradient::new(Color::srgba(0.0, 0.0, 0.0, 1.0)),
            secondary_container: BackgroundGradient::new(Color::srgba(0.1, 0.1, 0.1, 1.0)),
            list_item: BackgroundGradient::new(Color::srgba(0.05, 0.05, 0.05, 1.0)),
            list_item_hover: BackgroundGradient::new(Color::srgba(0.2, 0.2, 0.2, 1.0)),
            list_item_selected: BackgroundGradient::new(Color::srgba(0.0, 0.5, 1.0, 1.0)),
            text_overlay: BackgroundGradient::new(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            border_accent: BackgroundGradient::new(Color::srgba(0.5, 0.5, 0.5, 1.0)),
            success_state: BackgroundGradient::new(Color::srgba(0.0, 1.0, 0.0, 1.0)),
            warning_state: BackgroundGradient::new(Color::srgba(1.0, 1.0, 0.0, 1.0)),
            error_state: BackgroundGradient::new(Color::srgba(1.0, 0.0, 0.0, 1.0)),
        }
    }
}

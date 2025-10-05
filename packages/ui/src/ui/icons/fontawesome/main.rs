//! Main FontAwesome struct and public API

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::ui::TextShadow;

use super::detection::IconDetection;
use super::mappings::{ColorMappings, IconMappings, SizeConfigs};
use action_items_ecs_ui::theme::Theme;
use crate::ui::typography::TypographyScale;
use crate::ui::icons::{IconSize, IconType};

/// FontAwesome icon system with Unicode character mappings for zero-allocation icon rendering
#[derive(Resource, Debug, Clone)]
pub struct FontAwesome {
    /// Icon mappings from type to Unicode character
    pub icon_mappings: IconMappings,
    /// Icon colors based on result type
    pub color_mappings: ColorMappings,
    /// Icon size configurations
    pub size_configs: SizeConfigs,
}

impl Default for FontAwesome {
    fn default() -> Self {
        Self::new()
    }
}

impl FontAwesome {
    /// Create new FontAwesome system with comprehensive icon mappings
    pub fn new() -> Self {
        Self {
            icon_mappings: IconMappings::new(),
            color_mappings: ColorMappings,
            size_configs: SizeConfigs::new(),
        }
    }

    /// Get Unicode character for icon type with fallback handling
    #[inline]
    pub fn get_icon_char(&self, icon_type: IconType) -> char {
        self.icon_mappings.get_char(icon_type)
    }

    /// Get color for icon type with theme integration
    #[inline]
    pub fn get_icon_color(&self, icon_type: IconType, theme: &Theme) -> Color {
        self.color_mappings.get_color(icon_type, theme)
    }

    /// Get font size for icon size category
    #[inline]
    pub fn get_icon_size(&self, size: IconSize) -> f32 {
        self.size_configs.get_size(size)
    }

    /// Detect icon type from file extension with comprehensive mapping
    pub fn detect_icon_type_from_extension(&self, extension: &str) -> IconType {
        IconDetection::detect_from_extension(extension)
    }

    /// Detect icon type from file path with intelligent analysis
    pub fn detect_icon_type_from_path(&self, path: &str) -> IconType {
        IconDetection::detect_from_path(path)
    }

    /// Create TextBundle with FontAwesome icon
    pub fn create_icon_text(
        &self,
        icon_type: IconType,
        size: IconSize,
        theme: &Theme,
        typography: &TypographyScale,
    ) -> (Text, TextFont, TextColor, TextShadow) {
        let character = self.get_icon_char(icon_type);
        let color = self.get_icon_color(icon_type, theme);
        let font_size = self.get_icon_size(size);

        (
            Text::new(character.to_string()),
            TextFont {
                font: typography.font_handles.fontawesome_solid.clone(),
                font_size,
                ..default()
            },
            TextColor(color),
            TextShadow {
                color: theme.colors.text_secondary.with_alpha(0.5),
                offset: Vec2::new(0.0, 1.0),
            },
        )
    }
}

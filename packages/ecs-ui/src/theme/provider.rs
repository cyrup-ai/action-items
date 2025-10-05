use bevy::prelude::*;

use super::colors::ColorPalette;
use super::shadows::{ShadowConfig, ShadowElevation, ShadowTheme};
use super::spacing::{SpacingScale, SpacingTheme};
use super::typography::{FontScale, TypographyTheme};

/// Theme provider trait for consistent access patterns
pub trait ThemeProvider {
    #[allow(dead_code)] // Infrastructure for future theme system implementation
    fn get_color(&self, name: &str) -> Option<Color>;
    fn get_font_size(&self, scale: FontScale) -> f32;
    fn get_spacing(&self, scale: SpacingScale) -> f32;
    fn get_shadow(&self, elevation: ShadowElevation) -> &ShadowConfig;
}

/// Professional color palette optimized for modern launcher UI
#[derive(Resource, Debug, Clone, Default)]
pub struct Theme {
    pub colors: ColorPalette,
    pub typography: TypographyTheme,
    pub spacing: SpacingTheme,
    pub shadows: ShadowTheme,
}

impl ThemeProvider for Theme {
    fn get_color(&self, name: &str) -> Option<Color> {
        match name {
            "background.primary" => Some(self.colors.background_primary),
            "background.secondary" => Some(self.colors.background_secondary),
            "background.tertiary" => Some(self.colors.background_tertiary),
            "background.elevated" => Some(self.colors.background_elevated),
            "surface.default" => Some(self.colors.surface_default),
            "surface.hover" => Some(self.colors.surface_hover),
            "surface.active" => Some(self.colors.surface_active),
            "surface.selected" => Some(self.colors.surface_selected),
            "accent.blue" => Some(self.colors.accent_blue),
            "accent.purple" => Some(self.colors.accent_purple),
            "accent.green" => Some(self.colors.accent_green),
            "accent.orange" => Some(self.colors.accent_orange),
            "accent.red" => Some(self.colors.accent_red),
            "accent.yellow" => Some(self.colors.accent_yellow),
            "text.primary" => Some(self.colors.text_primary),
            "text.secondary" => Some(self.colors.text_secondary),
            "text.tertiary" => Some(self.colors.text_tertiary),
            "text.inverse" => Some(self.colors.text_inverse),
            "border.subtle" => Some(self.colors.border_subtle),
            "border.default" => Some(self.colors.border_default),
            "border.strong" => Some(self.colors.border_strong),
            "border.accent" => Some(self.colors.border_accent),
            "success" => Some(self.colors.success),
            "warning" => Some(self.colors.warning),
            "error" => Some(self.colors.error),
            "info" => Some(self.colors.info),
            _ => None,
        }
    }

    fn get_font_size(&self, scale: FontScale) -> f32 {
        match scale {
            FontScale::XS => self.typography.font_size_xs,
            FontScale::SM => self.typography.font_size_sm,
            FontScale::Base => self.typography.font_size_base,
            FontScale::LG => self.typography.font_size_lg,
            FontScale::XL => self.typography.font_size_xl,
            FontScale::Xxl => self.typography.font_size_2xl,
        }
    }

    fn get_spacing(&self, scale: SpacingScale) -> f32 {
        match scale {
            SpacingScale::XS => self.spacing.space_xs,
            SpacingScale::SM => self.spacing.space_sm,
            SpacingScale::MD => self.spacing.space_md,
            SpacingScale::LG => self.spacing.space_lg,
            SpacingScale::XL => self.spacing.space_xl,
            SpacingScale::Xxl => self.spacing.space_2xl,
            SpacingScale::Xxxl => self.spacing.space_3xl,
            SpacingScale::Xxxxl => self.spacing.space_4xl,
        }
    }

    fn get_shadow(&self, elevation: ShadowElevation) -> &ShadowConfig {
        match elevation {
            ShadowElevation::SM => &self.shadows.shadow_sm,
            ShadowElevation::MD => &self.shadows.shadow_md,
            ShadowElevation::LG => &self.shadows.shadow_lg,
            ShadowElevation::XL => &self.shadows.shadow_xl,
        }
    }
}

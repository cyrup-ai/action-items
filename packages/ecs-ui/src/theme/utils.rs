use bevy::prelude::*;

use super::provider::{Theme, ThemeProvider};
use super::shadows::ShadowElevation;
use super::spacing::SpacingScale;

/// Helper functions for creating themed UI components
impl Theme {
    /// Create a BoxShadow from a ShadowConfig
    pub fn create_box_shadow(&self, elevation: ShadowElevation) -> BoxShadow {
        let config = self.get_shadow(elevation);
        let mut shadows = vec![ShadowStyle {
            color: config.primary.color,
            x_offset: Val::VMin(config.primary.x_offset * 0.1),
            y_offset: Val::VMin(config.primary.y_offset * 0.1),
            spread_radius: Val::VMin(config.primary.spread_radius * 0.1),
            blur_radius: Val::VMin(config.primary.blur_radius * 0.1),
        }];

        if let Some(secondary) = &config.secondary {
            shadows.push(ShadowStyle {
                color: secondary.color,
                x_offset: Val::VMin(secondary.x_offset * 0.1),
                y_offset: Val::VMin(secondary.y_offset * 0.1),
                spread_radius: Val::VMin(secondary.spread_radius * 0.1),
                blur_radius: Val::VMin(secondary.blur_radius * 0.1),
            });
        }

        if let Some(tertiary) = &config.tertiary {
            shadows.push(ShadowStyle {
                color: tertiary.color,
                x_offset: Val::VMin(tertiary.x_offset * 0.1),
                y_offset: Val::VMin(tertiary.y_offset * 0.1),
                spread_radius: Val::VMin(tertiary.spread_radius * 0.1),
                blur_radius: Val::VMin(tertiary.blur_radius * 0.1),
            });
        }

        BoxShadow(shadows)
    }

    // Gradient functions temporarily disabled until Bevy 0.17+ adds gradient support

    /// Get UI spacing as Val::VMin (viewport-relative)
    pub fn spacing_px(&self, scale: SpacingScale) -> Val {
        Val::VMin(self.get_spacing(scale) * 0.1)
    }

    /// Get UI spacing as UiRect with all sides equal
    pub fn spacing_rect(&self, scale: SpacingScale) -> UiRect {
        UiRect::all(self.spacing_px(scale))
    }

    /// Get UI spacing as UiRect with different horizontal and vertical values
    pub fn spacing_rect_hv(&self, horizontal: SpacingScale, vertical: SpacingScale) -> UiRect {
        UiRect::axes(self.spacing_px(horizontal), self.spacing_px(vertical))
    }
}

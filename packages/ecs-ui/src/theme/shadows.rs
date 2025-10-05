use bevy::prelude::*;

/// Professional shadow system for depth and elevation
#[derive(Debug, Clone)]
pub struct ShadowTheme {
    pub shadow_sm: ShadowConfig,
    pub shadow_md: ShadowConfig,
    pub shadow_lg: ShadowConfig,
    pub shadow_xl: ShadowConfig,
}

#[derive(Debug, Clone)]
pub struct ShadowConfig {
    pub primary: ShadowLayer,
    pub secondary: Option<ShadowLayer>,
    pub tertiary: Option<ShadowLayer>,
}

#[derive(Debug, Clone)]
pub struct ShadowLayer {
    pub color: Color,
    pub x_offset: f32,
    pub y_offset: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
}

impl Default for ShadowTheme {
    fn default() -> Self {
        Self {
            shadow_sm: ShadowConfig {
                primary: ShadowLayer {
                    color: Color::BLACK.with_alpha(0.10),
                    x_offset: 0.0,
                    y_offset: 1.0,
                    blur_radius: 3.0,
                    spread_radius: 0.0,
                },
                secondary: Some(ShadowLayer {
                    color: Color::BLACK.with_alpha(0.06),
                    x_offset: 0.0,
                    y_offset: 1.0,
                    blur_radius: 2.0,
                    spread_radius: 0.0,
                }),
                tertiary: None,
            },
            shadow_md: ShadowConfig {
                primary: ShadowLayer {
                    color: Color::BLACK.with_alpha(0.15),
                    x_offset: 0.0,
                    y_offset: 4.0,
                    blur_radius: 12.0,
                    spread_radius: 0.0,
                },
                secondary: Some(ShadowLayer {
                    color: Color::BLACK.with_alpha(0.08),
                    x_offset: 0.0,
                    y_offset: 2.0,
                    blur_radius: 4.0,
                    spread_radius: 0.0,
                }),
                tertiary: None,
            },
            shadow_lg: ShadowConfig {
                primary: ShadowLayer {
                    color: Color::BLACK.with_alpha(0.25),
                    x_offset: 0.0,
                    y_offset: 8.0,
                    blur_radius: 24.0,
                    spread_radius: 0.0,
                },
                secondary: Some(ShadowLayer {
                    color: Color::BLACK.with_alpha(0.12),
                    x_offset: 0.0,
                    y_offset: 4.0,
                    blur_radius: 8.0,
                    spread_radius: 0.0,
                }),
                tertiary: Some(ShadowLayer {
                    color: Color::BLACK.with_alpha(0.08),
                    x_offset: 0.0,
                    y_offset: 1.0,
                    blur_radius: 3.0,
                    spread_radius: 0.0,
                }),
            },
            shadow_xl: ShadowConfig {
                primary: ShadowLayer {
                    color: Color::BLACK.with_alpha(0.35),
                    x_offset: 0.0,
                    y_offset: 16.0,
                    blur_radius: 48.0,
                    spread_radius: 0.0,
                },
                secondary: Some(ShadowLayer {
                    color: Color::BLACK.with_alpha(0.20),
                    x_offset: 0.0,
                    y_offset: 8.0,
                    blur_radius: 16.0,
                    spread_radius: 0.0,
                }),
                tertiary: Some(ShadowLayer {
                    color: Color::BLACK.with_alpha(0.12),
                    x_offset: 0.0,
                    y_offset: 2.0,
                    blur_radius: 6.0,
                    spread_radius: 0.0,
                }),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ShadowElevation {
    SM,
    MD,
    LG,
    XL,
}

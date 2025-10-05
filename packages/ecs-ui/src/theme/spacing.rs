// Note: Bevy prelude will be used when theme spacing system is implemented

/// Consistent spacing system using 4px grid
#[derive(Debug, Clone)]
pub struct SpacingTheme {
    pub space_xs: f32,  // 4px
    pub space_sm: f32,  // 8px
    pub space_md: f32,  // 12px
    pub space_lg: f32,  // 16px
    pub space_xl: f32,  // 24px
    pub space_2xl: f32, // 32px
    pub space_3xl: f32, // 48px
    pub space_4xl: f32, // 64px
}

impl Default for SpacingTheme {
    fn default() -> Self {
        Self {
            space_xs: 4.0,
            space_sm: 8.0,
            space_md: 12.0,
            space_lg: 16.0,
            space_xl: 24.0,
            space_2xl: 32.0,
            space_3xl: 48.0,
            space_4xl: 64.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpacingScale {
    XS,
    SM,
    MD,
    LG,
    XL,
    Xxl,
    Xxxl,
    Xxxxl,
}

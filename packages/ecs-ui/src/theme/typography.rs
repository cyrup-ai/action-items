/// Typography system with scale and font management
#[derive(Debug, Clone)]
pub struct TypographyTheme {
    // Font sizes using modular scale (1.25 ratio)
    pub font_size_xs: f32,   // 12px - Small captions
    pub font_size_sm: f32,   // 14px - Body text, descriptions
    pub font_size_base: f32, // 16px - Default body
    pub font_size_lg: f32,   // 18px - Result titles
    pub font_size_xl: f32,   // 22px - Search input
    pub font_size_2xl: f32,  // 28px - Headers

    // Line heights for optimal readability
    pub line_height_tight: f32,   // 1.25
    pub line_height_normal: f32,  // 1.5
    pub line_height_relaxed: f32, // 1.75

    // Font weights
    pub font_weight_normal: &'static str,
    pub font_weight_medium: &'static str,
    pub font_weight_semibold: &'static str,
}

impl Default for TypographyTheme {
    fn default() -> Self {
        Self {
            font_size_xs: 12.0,
            font_size_sm: 14.0,
            font_size_base: 16.0,
            font_size_lg: 18.0,
            font_size_xl: 22.0,
            font_size_2xl: 28.0,

            line_height_tight: 1.25,
            line_height_normal: 1.5,
            line_height_relaxed: 1.75,

            font_weight_normal: "normal",
            font_weight_medium: "medium",
            font_weight_semibold: "semibold",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FontScale {
    XS,
    SM,
    Base,
    LG,
    XL,
    Xxl,
}

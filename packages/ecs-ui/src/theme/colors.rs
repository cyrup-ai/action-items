use bevy::prelude::*;

// Gradient types for ColorPalette gradient methods
#[derive(Clone, Debug)]
pub struct BackgroundGradient {
    pub color: Color,
    pub color_stops: Vec<ColorStop>,
    pub opacity: f32,
}

#[derive(Clone, Debug)]
pub struct LinearGradient {
    pub angle: f32,
    pub stops: Vec<ColorStop>,
}

#[derive(Clone, Debug)]
pub struct ColorStop {
    pub position: f32,
    pub color: Color,
}

impl ColorStop {
    pub fn new(position: f32, color: Color) -> Self {
        Self { position, color }
    }
}

impl BackgroundGradient {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            color_stops: Vec::new(),
            opacity: 1.0,
        }
    }

    /// Convert to Bevy BackgroundColor for UI components
    /// 
    /// Helper method that extracts the primary color from a gradient
    /// and wraps it in Bevy's BackgroundColor type for use in UI nodes.
    pub fn to_bevy_background(&self) -> BackgroundColor {
        BackgroundColor(self.color)
    }
}

impl From<LinearGradient> for BackgroundGradient {
    fn from(linear: LinearGradient) -> Self {
        let color = linear
            .stops
            .first()
            .map(|s| s.color)
            .unwrap_or(Color::WHITE);
        Self {
            color,
            color_stops: linear.stops,
            opacity: 1.0,
        }
    }
}

pub struct GradientFactory;

impl GradientFactory {
    /// Create new linear gradient using native Bevy BackgroundGradient
    /// 
    /// # Parameters
    /// - `angle`: Gradient angle in degrees (0° = right, 90° = down, 180° = left, 270° = up)
    /// - `stops`: Vector of color stops defining the gradient progression
    /// 
    /// # Returns
    /// BackgroundGradient configured with the specified angle and color stops
    #[inline]
    pub fn linear(angle: f32, stops: Vec<ColorStop>) -> BackgroundGradient {
        BackgroundGradient::from(LinearGradient {
            angle: angle.to_radians(),
            stops,
        })
    }

    /// Create ColorStop from color and percentage position
    /// 
    /// # Parameters
    /// - `color`: Any Bevy Color for this stop
    /// - `position_percent`: Position along gradient (0.0-100.0), where 0.0 is start and 100.0 is end
    /// 
    /// # Returns
    /// ColorStop positioned at the specified location with the given color
    #[inline]
    pub fn color_stop(color: Color, position_percent: f32) -> ColorStop {
        ColorStop::new(position_percent, color)
    }

    /// Create dark professional gradient for modern UI aesthetics
    /// 
    /// Generates a vertical gradient (top to bottom) with subtle color variation,
    /// ideal for dark-themed container backgrounds. Adds slight blue tint for depth.
    /// 
    /// # Parameters
    /// - `start_lightness`: Top color lightness (0.0-1.0), typically 0.08-0.15 for dark themes
    /// - `end_lightness`: Bottom color lightness (0.0-1.0), typically slightly darker than start
    /// 
    /// # Returns
    /// Vertical gradient from lighter top to darker bottom with 98% opacity
    #[inline]
    pub fn dark_professional(start_lightness: f32, end_lightness: f32) -> BackgroundGradient {
        GradientFactory::linear(
            180.0, // Top to bottom (degrees)
            vec![
                GradientFactory::color_stop(
                    Color::srgba(
                        start_lightness,
                        start_lightness,
                        start_lightness + 0.02, // Slight blue tint for depth
                        0.98,
                    ),
                    0.0,
                ),
                GradientFactory::color_stop(
                    Color::srgba(
                        end_lightness,
                        end_lightness,
                        end_lightness + 0.02, // Slight blue tint for depth
                        0.98
                    ),
                    100.0,
                ),
            ],
        )
    }

    /// Create subtle accent gradient for interactive elements
    /// 
    /// Generates a diagonal gradient (top-left to bottom-right) with highlight and shadow
    /// based on the provided base color. The gradient creates depth by lightening the color
    /// at the top-left and darkening it at the bottom-right.
    /// 
    /// # Parameters
    /// - `base_color`: Any Color to use as the gradient base (e.g., accent color, brand color)
    /// - `intensity`: Gradient intensity factor (0.0-1.0), affects highlight/shadow strength
    ///   - 0.0 = minimal gradient effect (nearly flat)
    ///   - 0.5 = moderate gradient effect (subtle depth)
    ///   - 1.0 = strong gradient effect (pronounced depth)
    /// 
    /// # Algorithm
    /// - Highlight: base color * (1.0 + intensity * 0.15) - up to 15% lighter
    /// - Shadow: base color * (1.0 - intensity * 0.10) - up to 10% darker
    /// 
    /// # Returns
    /// Diagonal gradient creating subtle 3D depth effect for interactive UI elements
    /// 
    /// # Example
    /// ```rust
    /// let accent = Color::srgba(0.0, 0.48, 1.0, 1.0); // Blue
    /// let gradient = GradientFactory::subtle_accent(accent, 0.5); // Moderate intensity
    /// ```
    #[inline]
    pub fn subtle_accent(base_color: Color, intensity: f32) -> BackgroundGradient {
        let (r, g, b, a) = match base_color {
            Color::Srgba(srgba) => (srgba.red, srgba.green, srgba.blue, srgba.alpha),
            _ => (0.5, 0.5, 0.5, 1.0), // Fallback to neutral gray
        };

        let highlight_factor = 1.0 + (intensity * 0.15); // Up to 15% lighter
        let shadow_factor = 1.0 - (intensity * 0.1);     // Up to 10% darker

        GradientFactory::linear(
            135.0, // Diagonal gradient (top-left to bottom-right)
            vec![
                GradientFactory::color_stop(
                    Color::srgba(
                        (r * highlight_factor).min(1.0), // Clamp to valid range
                        (g * highlight_factor).min(1.0),
                        (b * highlight_factor).min(1.0),
                        a,
                    ),
                    0.0, // Top-left (highlight)
                ),
                GradientFactory::color_stop(
                    Color::srgba(
                        (r * shadow_factor).max(0.0), // Clamp to valid range
                        (g * shadow_factor).max(0.0),
                        (b * shadow_factor).max(0.0),
                        a,
                    ),
                    100.0, // Bottom-right (shadow)
                ),
            ],
        )
    }
}

/// Comprehensive color system with sRGBA values optimized for translucent backgrounds
#[derive(Debug, Clone)]
pub struct ColorPalette {
    // Primary backgrounds
    pub background_primary: Color,
    pub background_secondary: Color,
    pub background_tertiary: Color,
    pub background_elevated: Color,

    // Interactive elements
    pub surface_default: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub surface_selected: Color,

    // Accent colors
    pub accent_blue: Color,
    pub accent_purple: Color,
    pub accent_green: Color,
    pub accent_orange: Color,
    pub accent_red: Color,
    pub accent_yellow: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_inverse: Color,

    // Border colors
    pub border_subtle: Color,
    pub border_default: Color,
    pub border_strong: Color,
    pub border_accent: Color,

    // Semantic colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            // Primary backgrounds - optimized for translucency and depth
            background_primary: Color::srgba(0.08, 0.08, 0.09, 0.95),
            background_secondary: Color::srgba(0.12, 0.12, 0.14, 0.98),
            background_tertiary: Color::srgba(0.16, 0.16, 0.18, 0.99),
            background_elevated: Color::srgba(0.20, 0.20, 0.22, 0.99),

            // Interactive surfaces
            surface_default: Color::srgba(0.15, 0.15, 0.17, 0.80),
            surface_hover: Color::srgba(0.22, 0.22, 0.24, 0.85),
            surface_active: Color::srgba(0.28, 0.28, 0.30, 0.90),
            surface_selected: Color::srgba(0.0, 0.48, 1.0, 0.15),

            // Modern accent palette
            accent_blue: Color::srgba(0.0, 0.48, 1.0, 1.0), // #007AFF
            accent_purple: Color::srgba(0.55, 0.27, 0.88, 1.0), // #8B46E0
            accent_green: Color::srgba(0.20, 0.78, 0.35, 1.0), // #34C759
            accent_orange: Color::srgba(1.0, 0.58, 0.0, 1.0), // #FF9500
            accent_red: Color::srgba(1.0, 0.23, 0.19, 1.0), // #FF3B30
            accent_yellow: Color::srgba(1.0, 0.80, 0.0, 1.0), // #FFCC00

            // Text hierarchy with proper contrast
            text_primary: Color::srgba(0.95, 0.95, 0.97, 1.0), // High contrast
            text_secondary: Color::srgba(0.70, 0.70, 0.75, 1.0), // Medium contrast
            text_tertiary: Color::srgba(0.50, 0.50, 0.55, 1.0), // Low contrast
            text_inverse: Color::srgba(0.05, 0.05, 0.05, 1.0), // Dark text on light

            // Border system
            border_subtle: Color::srgba(0.20, 0.20, 0.24, 0.60),
            border_default: Color::srgba(0.30, 0.30, 0.34, 0.80),
            border_strong: Color::srgba(0.40, 0.40, 0.44, 1.0),
            border_accent: Color::srgba(0.0, 0.48, 1.0, 0.60),

            // Semantic colors with appropriate alpha
            success: Color::srgba(0.20, 0.78, 0.35, 0.90),
            warning: Color::srgba(1.0, 0.80, 0.0, 0.90),
            error: Color::srgba(1.0, 0.23, 0.19, 0.90),
            info: Color::srgba(0.0, 0.48, 1.0, 0.90),
        }
    }
}

impl ColorPalette {
    /// Professional color definitions for Raycast-like UI (gradients disabled until Bevy 0.17+)
    /// Container background color (dark theme)
    #[inline]
    pub fn container_background(&self) -> Color {
        // Use solid color instead of gradient until Bevy gradient support is added
        Color::srgba(0.10, 0.10, 0.12, 0.98)
    }

    /// Search input background (elevated surface feeling)
    #[inline]
    pub fn search_input_background(&self) -> Color {
        // Solid color replacement for gradient until Bevy supports gradients
        Color::srgba(0.16, 0.16, 0.18, 0.87)
    }

    /// List item background (default state) - generic for any list UI
    #[inline]
    pub fn list_item_background(&self) -> Color {
        Color::srgba(0.12, 0.12, 0.14, 0.82)
    }

    /// List item hover background
    #[inline]
    pub fn list_item_hover_background(&self) -> Color {
        Color::srgba(0.0, 0.48, 1.0, 0.25)
    }

    /// List item selected background
    #[inline]
    pub fn list_item_selected_background(&self) -> Color {
        Color::srgba(0.0, 0.48, 1.0, 0.60)
    }

    /// Search focus border color
    #[inline]
    pub fn search_focus_border(&self) -> Color {
        Color::srgba(0.0, 0.48, 1.0, 0.8)
    }

    /// Container background gradient (dark to slightly lighter)
    #[inline]
    pub fn container_gradient(&self) -> BackgroundGradient {
        GradientFactory::dark_professional(0.08, 0.12)
    }

    /// Search input gradient (elevated surface feeling)
    #[inline]
    pub fn search_input_gradient(&self) -> BackgroundGradient {
        GradientFactory::linear(
            180.0,
            vec![
                GradientFactory::color_stop(Color::srgba(0.18, 0.18, 0.20, 0.85), 0.0),
                GradientFactory::color_stop(Color::srgba(0.15, 0.15, 0.17, 0.90), 100.0),
            ],
        )
    }

    /// List item default gradient - generic for any list UI
    #[inline]
    pub fn list_item_gradient(&self) -> BackgroundGradient {
        GradientFactory::linear(
            180.0,
            vec![
                GradientFactory::color_stop(Color::srgba(0.13, 0.13, 0.15, 0.80), 0.0),
                GradientFactory::color_stop(Color::srgba(0.11, 0.11, 0.13, 0.85), 100.0),
            ],
        )
    }

    /// List item hover gradient (blue accent)
    #[inline]
    pub fn list_item_hover_gradient(&self) -> BackgroundGradient {
        GradientFactory::linear(
            180.0,
            vec![
                GradientFactory::color_stop(Color::srgba(0.0, 0.48, 1.0, 0.25), 0.0),
                GradientFactory::color_stop(Color::srgba(0.18, 0.18, 0.22, 0.90), 100.0),
            ],
        )
    }

    /// List item selected gradient (strong blue)
    #[inline]
    pub fn list_item_selected_gradient(&self) -> BackgroundGradient {
        GradientFactory::linear(
            180.0,
            vec![
                GradientFactory::color_stop(Color::srgba(0.0, 0.48, 1.0, 0.60), 0.0),
                GradientFactory::color_stop(Color::srgba(0.0, 0.38, 0.80, 0.80), 100.0),
            ],
        )
    }
}

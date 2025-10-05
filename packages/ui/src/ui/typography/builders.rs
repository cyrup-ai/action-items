//! Builder patterns for text styles and shadows

use bevy::prelude::*;

use super::types::TextStyleConfig;

/// Builder pattern for creating custom text styles
#[derive(Debug, Clone)]
#[allow(dead_code)] // Infrastructure for future typography system implementation
pub struct TextStyleBuilder {
    font: Handle<Font>,
    font_size: f32,
    color: Color,
    line_height: f32,
    letter_spacing: f32,
}

impl TextStyleBuilder {
    /// Create a new text style builder with default values
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn new(font: Handle<Font>) -> Self {
        Self {
            font,
            font_size: 16.0,
            color: Color::WHITE,
            line_height: 1.5,
            letter_spacing: 0.0,
        }
    }

    /// Set the font size
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set the text color
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the line height multiplier
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn line_height(mut self, height: f32) -> Self {
        self.line_height = height;
        self
    }

    /// Set the letter spacing in em units
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn letter_spacing(mut self, spacing: f32) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// Build the text style components
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn build(self) -> (TextFont, TextColor) {
        (
            TextFont {
                font: self.font,
                font_size: self.font_size,
                ..default()
            },
            TextColor(self.color),
        )
    }

    /// Build just the TextFont component
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn build_font(self) -> TextFont {
        TextFont {
            font: self.font,
            font_size: self.font_size,
            ..default()
        }
    }

    /// Build the complete text style config
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn build_config(self) -> TextStyleConfig {
        TextStyleConfig {
            font: self.font,
            font_size: self.font_size,
            color: self.color,
            line_height: self.line_height,
            letter_spacing: self.letter_spacing,
        }
    }
}

/// Text shadow builder for enhanced readability
#[derive(Debug, Clone)]
pub struct TextShadowBuilder {
    offset: Vec2,
    blur_radius: f32,
    color: Color,
}

impl TextShadowBuilder {
    /// Create a new text shadow builder
    pub fn new() -> Self {
        Self {
            offset: Vec2::new(0.0, 1.0),
            blur_radius: 2.0,
            color: Color::BLACK.with_alpha(0.25),
        }
    }

    /// Set the shadow offset
    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Vec2::new(x, y);
        self
    }

    /// Set the blur radius
    pub fn blur_radius(mut self, radius: f32) -> Self {
        self.blur_radius = radius;
        self
    }

    /// Set the shadow color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Build the text shadow
    pub fn build(self) -> TextShadow {
        TextShadow {
            offset: self.offset,
            color: self.color,
        }
    }
}

impl Default for TextShadowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

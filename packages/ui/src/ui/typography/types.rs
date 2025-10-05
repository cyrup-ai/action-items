//! Core typography types and configurations

use bevy::prelude::*;

/// Font handles for different font families
#[derive(Debug, Clone)]
pub struct FontHandles {
    pub ubuntu_regular: Handle<Font>,
    pub ubuntu_medium: Handle<Font>,
    pub ubuntu_bold: Handle<Font>,
    pub fira_code_regular: Handle<Font>,
    pub fontawesome_solid: Handle<Font>,
}

/// Pre-configured text styles for common UI elements
#[derive(Debug, Clone)]
pub struct TextStyles {
    pub search_input: TextStyleConfig,
    pub result_title: TextStyleConfig,
    pub result_description: TextStyleConfig,
    pub header: TextStyleConfig,
    pub body: TextStyleConfig,
    pub caption: TextStyleConfig,
    pub monospace: TextStyleConfig,
}

/// Configuration for a complete text style
#[derive(Debug, Clone)]
pub struct TextStyleConfig {
    pub font: Handle<Font>,
    pub font_size: f32,
    pub color: Color,
    pub line_height: f32,
    pub letter_spacing: f32,
}

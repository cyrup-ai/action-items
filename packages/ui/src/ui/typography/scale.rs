//! Typography scale resource for consistent text styling

use bevy::prelude::*;

use super::types::{FontHandles, TextStyleConfig, TextStyles};
use action_items_ecs_ui::theme::{FontScale, Theme, ThemeProvider};

/// Typography scale resource for consistent text styling across the application
#[derive(Resource, Debug, Clone)]
pub struct TypographyScale {
    pub font_handles: FontHandles,
    pub text_styles: TextStyles,
}

impl TypographyScale {
    /// Create a new typography scale with loaded fonts
    pub fn new(
        ubuntu_regular: Handle<Font>,
        ubuntu_medium: Handle<Font>,
        ubuntu_bold: Handle<Font>,
        fira_code_regular: Handle<Font>,
        fontawesome_solid: Handle<Font>,
        theme: &Theme,
    ) -> Self {
        let font_handles = FontHandles {
            ubuntu_regular: ubuntu_regular.clone(),
            ubuntu_medium: ubuntu_medium.clone(),
            ubuntu_bold: ubuntu_bold.clone(),
            fira_code_regular: fira_code_regular.clone(),
            fontawesome_solid: fontawesome_solid.clone(),
        };

        let text_styles = TextStyles {
            search_input: TextStyleConfig {
                font: ubuntu_regular.clone(),
                font_size: theme.get_font_size(FontScale::XL),
                color: theme.colors.text_primary,
                line_height: theme.typography.line_height_normal,
                letter_spacing: 0.0,
            },
            result_title: TextStyleConfig {
                font: ubuntu_medium.clone(),
                font_size: theme.get_font_size(FontScale::LG),
                color: theme.colors.text_primary,
                line_height: theme.typography.line_height_tight,
                letter_spacing: -0.01,
            },
            result_description: TextStyleConfig {
                font: ubuntu_regular.clone(),
                font_size: theme.get_font_size(FontScale::SM),
                color: theme.colors.text_secondary,
                line_height: theme.typography.line_height_normal,
                letter_spacing: 0.0,
            },
            header: TextStyleConfig {
                font: ubuntu_bold.clone(),
                font_size: theme.get_font_size(FontScale::Xxl),
                color: theme.colors.text_primary,
                line_height: theme.typography.line_height_tight,
                letter_spacing: -0.02,
            },
            body: TextStyleConfig {
                font: ubuntu_regular.clone(),
                font_size: theme.get_font_size(FontScale::Base),
                color: theme.colors.text_primary,
                line_height: theme.typography.line_height_normal,
                letter_spacing: 0.0,
            },
            caption: TextStyleConfig {
                font: ubuntu_regular.clone(),
                font_size: theme.get_font_size(FontScale::XS),
                color: theme.colors.text_tertiary,
                line_height: theme.typography.line_height_tight,
                letter_spacing: 0.01,
            },
            monospace: TextStyleConfig {
                font: fira_code_regular.clone(),
                font_size: theme.get_font_size(FontScale::Base),
                color: theme.colors.text_primary,
                line_height: theme.typography.line_height_relaxed,
                letter_spacing: 0.0,
            },
        };

        Self {
            font_handles,
            text_styles,
        }
    }
}

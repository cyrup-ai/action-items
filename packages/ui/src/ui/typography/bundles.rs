//! Helper functions for creating complete text bundles

use bevy::prelude::*;

use super::builders::TextShadowBuilder;
use super::scale::TypographyScale;

/// Helper functions for creating complete text bundles
pub struct TextBundleBuilder;

impl TextBundleBuilder {
    /// Create a search input text bundle
    pub fn search_input(
        text: &str,
        typography: &TypographyScale,
    ) -> (Text, TextFont, TextColor, TextShadow) {
        let style = &typography.text_styles.search_input;
        (
            Text::new(text),
            TextFont {
                font: style.font.clone(),
                font_size: style.font_size,
                ..default()
            },
            TextColor(style.color),
            TextShadowBuilder::new()
                .offset(0.0, 1.0)
                .blur_radius(2.0)
                .color(Color::BLACK.with_alpha(0.15))
                .build(),
        )
    }

    /// Create a search icon text bundle
    pub fn search_icon(typography: &TypographyScale) -> (Text, TextFont, TextColor) {
        let style = &typography.text_styles.search_input;
        (
            Text::new("\u{f002}"), // FontAwesome search icon
            TextFont {
                font: style.font.clone(),
                font_size: style.font_size,
                ..default()
            },
            TextColor(style.color),
        )
    }

    /// Create a result title text bundle
    pub fn result_title(text: &str, typography: &TypographyScale) -> (Text, TextFont, TextColor) {
        let style = &typography.text_styles.result_title;
        (
            Text::new(text),
            TextFont {
                font: style.font.clone(),
                font_size: style.font_size,
                ..default()
            },
            TextColor(style.color),
        )
    }

    /// Create a result description text bundle
    pub fn result_description(
        text: &str,
        typography: &TypographyScale,
    ) -> (Text, TextFont, TextColor) {
        let style = &typography.text_styles.result_description;
        (
            Text::new(text),
            TextFont {
                font: style.font.clone(),
                font_size: style.font_size,
                ..default()
            },
            TextColor(style.color),
        )
    }

    /// Create a header text bundle
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn header(
        text: &str,
        typography: &TypographyScale,
    ) -> (Text, TextFont, TextColor, TextShadow) {
        let style = &typography.text_styles.header;
        (
            Text::new(text),
            TextFont {
                font: style.font.clone(),
                font_size: style.font_size,
                ..default()
            },
            TextColor(style.color),
            TextShadowBuilder::new()
                .offset(0.0, 2.0)
                .blur_radius(4.0)
                .color(Color::BLACK.with_alpha(0.20))
                .build(),
        )
    }

    /// Create a body text bundle
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn body(text: &str, typography: &TypographyScale) -> (Text, TextFont, TextColor) {
        let style = &typography.text_styles.body;
        (
            Text::new(text),
            TextFont {
                font: style.font.clone(),
                font_size: style.font_size,
                ..default()
            },
            TextColor(style.color),
        )
    }

    /// Create a caption text bundle
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn caption(text: &str, typography: &TypographyScale) -> (Text, TextFont, TextColor) {
        let style = &typography.text_styles.caption;
        (
            Text::new(text),
            TextFont {
                font: style.font.clone(),
                font_size: style.font_size,
                ..default()
            },
            TextColor(style.color),
        )
    }

    /// Create a monospace text bundle
    #[allow(dead_code)] // Infrastructure for future typography system implementation
    pub fn monospace(text: &str, typography: &TypographyScale) -> (Text, TextFont, TextColor) {
        let style = &typography.text_styles.monospace;
        (
            Text::new(text),
            TextFont {
                font: style.font.clone(),
                font_size: style.font_size,
                ..default()
            },
            TextColor(style.color),
        )
    }
}

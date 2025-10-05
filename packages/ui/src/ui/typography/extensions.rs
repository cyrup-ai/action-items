//! Extension traits for convenient text styling

use bevy::prelude::*;

use super::scale::TypographyScale;

/// Extension trait for convenient text styling
#[allow(dead_code)] // Infrastructure for future typography system implementation
pub trait TextStyleExt {
    /// Apply a search input style
    fn search_input_style(self, typography: &TypographyScale) -> Self;

    /// Apply a result title style  
    fn result_title_style(self, typography: &TypographyScale) -> Self;

    /// Apply a result description style
    fn result_description_style(self, typography: &TypographyScale) -> Self;

    /// Apply a header style
    fn header_style(self, typography: &TypographyScale) -> Self;

    /// Apply a body text style
    fn body_style(self, typography: &TypographyScale) -> Self;

    /// Apply a caption style
    fn caption_style(self, typography: &TypographyScale) -> Self;

    /// Apply a monospace style
    fn monospace_style(self, typography: &TypographyScale) -> Self;
}

impl TextStyleExt for TextFont {
    fn search_input_style(mut self, typography: &TypographyScale) -> Self {
        let style = &typography.text_styles.search_input;
        self.font = style.font.clone();
        self.font_size = style.font_size;
        self
    }

    fn result_title_style(mut self, typography: &TypographyScale) -> Self {
        let style = &typography.text_styles.result_title;
        self.font = style.font.clone();
        self.font_size = style.font_size;
        self
    }

    fn result_description_style(mut self, typography: &TypographyScale) -> Self {
        let style = &typography.text_styles.result_description;
        self.font = style.font.clone();
        self.font_size = style.font_size;
        self
    }

    fn header_style(mut self, typography: &TypographyScale) -> Self {
        let style = &typography.text_styles.header;
        self.font = style.font.clone();
        self.font_size = style.font_size;
        self
    }

    fn body_style(mut self, typography: &TypographyScale) -> Self {
        let style = &typography.text_styles.body;
        self.font = style.font.clone();
        self.font_size = style.font_size;
        self
    }

    fn caption_style(mut self, typography: &TypographyScale) -> Self {
        let style = &typography.text_styles.caption;
        self.font = style.font.clone();
        self.font_size = style.font_size;
        self
    }

    fn monospace_style(mut self, typography: &TypographyScale) -> Self {
        let style = &typography.text_styles.monospace;
        self.font = style.font.clone();
        self.font_size = style.font_size;
        self
    }
}

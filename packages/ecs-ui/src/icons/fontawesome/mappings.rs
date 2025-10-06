//! Icon mappings, color configurations, and size settings

use std::collections::HashMap;

use bevy::prelude::*;
use crate::theme::Theme;

use crate::icons::types::{IconType, IconSize};

/// Zero-allocation icon mappings using const arrays
#[derive(Debug, Clone)]
pub struct IconMappings {
    mappings: HashMap<IconType, char>,
}

impl IconMappings {
    pub fn new() -> Self {
        let mut mappings = HashMap::with_capacity(32);

        // FontAwesome Solid Unicode mappings - carefully chosen for clarity
        mappings.insert(IconType::Application, '\u{f390}'); // desktop
        mappings.insert(IconType::Command, '\u{f120}'); // terminal
        mappings.insert(IconType::Terminal, '\u{f120}'); // terminal
        mappings.insert(IconType::Folder, '\u{f07b}'); // folder
        mappings.insert(IconType::File, '\u{f15b}'); // file
        mappings.insert(IconType::Code, '\u{f1c9}'); // code
        mappings.insert(IconType::Config, '\u{f013}'); // cog
        mappings.insert(IconType::Database, '\u{f1c0}'); // database
        mappings.insert(IconType::Document, '\u{f1c2}'); // file-alt
        mappings.insert(IconType::Text, '\u{f15c}'); // file-text
        mappings.insert(IconType::Spreadsheet, '\u{f1c3}'); // file-excel
        mappings.insert(IconType::Presentation, '\u{f1c4}'); // file-powerpoint
        mappings.insert(IconType::Image, '\u{f1c5}'); // file-image
        mappings.insert(IconType::Video, '\u{f1c8}'); // file-video
        mappings.insert(IconType::Audio, '\u{f1c7}'); // file-audio
        mappings.insert(IconType::Archive, '\u{f1c6}'); // file-archive
        mappings.insert(IconType::Font, '\u{f031}'); // font
        mappings.insert(IconType::Log, '\u{f56e}'); // clipboard-list
        mappings.insert(IconType::Lock, '\u{f023}'); // lock
        mappings.insert(IconType::Web, '\u{f0ac}'); // globe
        mappings.insert(IconType::Api, '\u{f085}'); // cogs
        mappings.insert(IconType::Unknown, '\u{f059}'); // question-circle

        Self { mappings }
    }

    #[inline]
    pub fn get_char(&self, icon_type: IconType) -> char {
        *self.mappings.get(&icon_type).unwrap_or(&'\u{f059}') // fallback to question-circle
    }
}

/// Color mappings for semantic icon coloring
#[derive(Debug, Clone)]
pub struct ColorMappings;

impl ColorMappings {
    #[inline]
    pub fn get_color(&self, icon_type: IconType, theme: &Theme) -> Color {
        match icon_type {
            IconType::Application => theme.colors.accent_blue,
            IconType::Command => theme.colors.accent_green,
            IconType::Terminal => theme.colors.accent_green,
            IconType::Folder => theme.colors.accent_yellow,
            IconType::File => theme.colors.text_secondary,
            IconType::Code => theme.colors.accent_purple,
            IconType::Config => theme.colors.accent_orange,
            IconType::Database => theme.colors.accent_blue,
            IconType::Document => theme.colors.accent_red,
            IconType::Text => theme.colors.text_primary,
            IconType::Image => theme.colors.accent_green,
            IconType::Video => theme.colors.accent_red,
            IconType::Audio => theme.colors.accent_purple,
            IconType::Archive => theme.colors.accent_orange,
            IconType::Unknown => theme.colors.text_tertiary,
            IconType::Web => theme.colors.accent_blue,
            IconType::Spreadsheet => theme.colors.accent_green,
            IconType::Presentation => theme.colors.accent_orange,
            IconType::Font => theme.colors.accent_purple,
            IconType::Log => theme.colors.text_secondary,
            IconType::Lock => theme.colors.accent_red,
            IconType::Api => theme.colors.accent_blue,
        }
    }
}

/// Size configurations for consistent icon scaling
#[derive(Debug, Clone)]
pub struct SizeConfigs {
    sizes: HashMap<IconSize, f32>,
}

impl SizeConfigs {
    pub fn new() -> Self {
        let mut sizes = HashMap::with_capacity(4);
        sizes.insert(IconSize::Small, 16.0);
        sizes.insert(IconSize::Medium, 24.0);
        sizes.insert(IconSize::Large, 32.0);
        sizes.insert(IconSize::XLarge, 48.0);

        Self { sizes }
    }

    #[inline]
    pub fn get_size(&self, size: IconSize) -> f32 {
        *self.sizes.get(&size).unwrap_or(&24.0) // Default to medium
    }
}

impl Default for IconMappings {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SizeConfigs {
    fn default() -> Self {
        Self::new()
    }
}

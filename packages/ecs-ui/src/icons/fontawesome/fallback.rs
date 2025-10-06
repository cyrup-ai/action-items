//! Fallback icon system for graceful degradation

use std::collections::HashMap;

use bevy::prelude::*;

use crate::icons::types::IconType;

/// Fallback icon system for graceful degradation
#[derive(Resource, Debug, Clone)]
pub struct IconFallback {
    pub primary_fallback: char,
    pub type_fallbacks: HashMap<IconType, char>,
}

impl Default for IconFallback {
    fn default() -> Self {
        let mut type_fallbacks = HashMap::new();

        // Simple ASCII fallbacks for extreme cases
        type_fallbacks.insert(IconType::Application, '▢');
        type_fallbacks.insert(IconType::Folder, '▣');
        type_fallbacks.insert(IconType::File, '▤');
        type_fallbacks.insert(IconType::Code, '▥');
        type_fallbacks.insert(IconType::Unknown, '?');

        Self {
            primary_fallback: '●',
            type_fallbacks,
        }
    }
}

impl IconFallback {
    /// Get fallback character for icon type
    #[inline]
    pub fn get_fallback(&self, icon_type: IconType) -> char {
        *self
            .type_fallbacks
            .get(&icon_type)
            .unwrap_or(&self.primary_fallback)
    }
}

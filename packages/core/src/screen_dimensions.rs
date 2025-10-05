//! Screen dimensions resource for UI calculations
//!
//! Simple screen dimension tracking for viewport-responsive UI

use bevy::prelude::*;

/// Screen dimensions resource for viewport-responsive UI calculations  
#[derive(Resource, Debug, Clone, Copy)]
pub struct ScreenDimensions {
    /// Physical width of active screen in pixels
    pub width: u32,
    /// Physical height of active screen in pixels  
    pub height: u32,
    /// Scale factor for high-DPI displays
    pub scale_factor: f64,
    /// Logical width accounting for DPI scaling
    pub logical_width: f32,
    /// Logical height accounting for DPI scaling
    pub logical_height: f32,
}

impl Default for ScreenDimensions {
    #[inline]
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
            logical_width: 1920.0,
            logical_height: 1080.0,
        }
    }
}

impl ScreenDimensions {
    /// Calculate viewport percentage as pixel value
    #[inline]
    pub fn vw_to_pixels(&self, vw_percent: f32) -> f32 {
        (self.logical_width * vw_percent) / 100.0
    }

    /// Calculate viewport height percentage as pixel value  
    #[inline]
    pub fn vh_to_pixels(&self, vh_percent: f32) -> f32 {
        (self.logical_height * vh_percent) / 100.0
    }

    /// Get screen aspect ratio
    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        self.logical_width / self.logical_height
    }

    /// Check if screen is in landscape orientation
    #[inline]
    pub fn is_landscape(&self) -> bool {
        self.logical_width > self.logical_height
    }

    /// Check if screen is in portrait orientation
    #[inline]
    pub fn is_portrait(&self) -> bool {
        self.logical_height > self.logical_width
    }
}

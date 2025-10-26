//! Window state management
//!
//! Zero-allocation window state tracking with blazing-fast focus and animation management.

use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};

use crate::window::errors::WindowModeError;
use crate::window::positioning::ScreenDimensions;

/// Window animation states
/// Zero-allocation component for blazing-fast opacity animations
#[derive(Component)]
pub struct WindowAnimation {
    pub target_opacity: f32,
    pub current_opacity: f32,
    pub animation_speed: f32,
}

/// Viewport conversion state for screen-to-Bevy-Val translations
/// Zero-allocation viewport calculations with pre-computed ratios for blazing-fast conversions
#[derive(Resource, Debug, Clone, Copy)]
pub struct ViewportState {
    /// Pre-calculated width conversion ratio (1.0 / logical_width * 100.0)
    pub width_ratio: f32,
    /// Pre-calculated height conversion ratio (1.0 / logical_height * 100.0)  
    pub height_ratio: f32,
    /// Scale factor for DPI-aware calculations
    pub scale_factor: f64,
    /// Cached logical dimensions for validation
    pub logical_width: f32,
    pub logical_height: f32,
    /// Last update timestamp for change detection
    pub last_update: std::time::Instant,
}

impl Default for ViewportState {
    #[inline]
    fn default() -> Self {
        // Safe fallback values for 1920x1080 at 1.0 scale
        Self {
            width_ratio: 100.0 / 1920.0,
            height_ratio: 100.0 / 1080.0,
            scale_factor: 1.0,
            logical_width: 1920.0,
            logical_height: 1080.0,
            last_update: std::time::Instant::now(),
        }
    }
}

impl ViewportState {
    /// Update viewport state from screen dimensions
    /// Zero-allocation update with optimized ratio pre-calculation
    #[inline]
    pub fn update_from_screen_dimensions(&mut self, dimensions: &ScreenDimensions) {
        // Only update if dimensions actually changed to avoid unnecessary work
        if (self.logical_width - dimensions.logical_width).abs() > f32::EPSILON
            || (self.logical_height - dimensions.logical_height).abs() > f32::EPSILON
            || (self.scale_factor - dimensions.scale_factor).abs() > f64::EPSILON
        {
            // Pre-calculate conversion ratios for zero-allocation conversions
            self.width_ratio = 100.0 / dimensions.logical_width.max(1.0);
            self.height_ratio = 100.0 / dimensions.logical_height.max(1.0);
            self.scale_factor = dimensions.scale_factor;
            self.logical_width = dimensions.logical_width;
            self.logical_height = dimensions.logical_height;
            self.last_update = std::time::Instant::now();
        }
    }
}

/// Resource to track window visibility and sizing
/// Zero-allocation window state management with blazing-fast updates
#[derive(Resource)]
pub struct LauncherState {
    pub visible: bool,
    pub window_entity: Option<Entity>,
    pub current_height: f32,
    pub target_height: f32,
    pub has_gained_focus: bool, // Track if window has gained focus since being shown
    pub show_timestamp: Option<std::time::Instant>, // Track when window was shown
}

/// Resource for dynamic window mode management with macOS fullscreen workarounds
/// Zero-allocation window mode optimization with blazing-fast mode switching
#[derive(Resource, Debug, Clone)]
pub struct WindowModeManager {
    pub current_mode: WindowMode,
    pub target_mode: WindowMode,
    pub fullscreen_capable: bool,
    pub bevy_fullscreen_works: bool, // Track if Bevy's BorderlessFullscreen works
    pub fallback_mode: WindowMode,
    pub last_mode_change: Option<std::time::Instant>,
    pub mode_change_cooldown: std::time::Duration,
}

impl Default for WindowModeManager {
    #[inline]
    fn default() -> Self {
        Self {
            current_mode: WindowMode::Windowed,
            target_mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary), /* Try fullscreen by default */
            fullscreen_capable: true, // Assume capable until proven otherwise
            bevy_fullscreen_works: false, // Conservative assumption for macOS
            fallback_mode: WindowMode::Windowed,
            last_mode_change: None,
            mode_change_cooldown: std::time::Duration::from_millis(500),
        }
    }
}

impl WindowModeManager {
    /// Check if enough time has passed since last mode change to allow another change
    /// Uses last_mode_change and mode_change_cooldown fields
    #[inline]
    pub fn can_change_mode(&self) -> bool {
        match self.last_mode_change {
            Some(last_change) => last_change.elapsed() >= self.mode_change_cooldown,
            None => true, // No previous change, allow first change
        }
    }

    /// Attempt to change to target mode, using fallback if target fails
    /// Uses current_mode, target_mode, fullscreen_capable, bevy_fullscreen_works, and fallback_mode
    /// fields
    #[inline]
    pub fn attempt_mode_change(&mut self) -> Result<WindowMode, WindowModeError> {
        if !self.can_change_mode() {
            return Err(WindowModeError::CooldownActive);
        }

        // Capture the previous mode before changing it for accurate logging
        let previous_mode = self.current_mode;

        // Check if target mode is feasible based on capability flags
        let effective_target = match &self.target_mode {
            WindowMode::BorderlessFullscreen(..) | WindowMode::Fullscreen(..) => {
                if !self.fullscreen_capable {
                    tracing::warn!(
                        "Fullscreen not capable, using fallback mode: {:?}",
                        self.fallback_mode
                    );
                    self.fallback_mode
                } else if !self.bevy_fullscreen_works
                    && matches!(self.target_mode, WindowMode::BorderlessFullscreen(..))
                {
                    tracing::warn!(
                        "Bevy BorderlessFullscreen doesn't work, using fallback mode: {:?}",
                        self.fallback_mode
                    );
                    self.fallback_mode
                } else {
                    self.target_mode
                }
            },
            _ => self.target_mode,
        };

        // Update current mode and timestamp
        self.current_mode = effective_target;
        self.last_mode_change = Some(std::time::Instant::now());

        tracing::info!(
            "Window mode changed from {:?} to {:?} (target: {:?}, fullscreen_capable: {}, \
             bevy_works: {})",
            previous_mode,
            effective_target,
            self.target_mode,
            self.fullscreen_capable,
            self.bevy_fullscreen_works
        );

        Ok(effective_target)
    }

    /// Update capability flags based on runtime testing
    /// Uses fullscreen_capable and bevy_fullscreen_works fields
    #[inline]
    pub fn update_capabilities(&mut self, fullscreen_works: bool, bevy_fullscreen_works: bool) {
        self.fullscreen_capable = fullscreen_works;
        self.bevy_fullscreen_works = bevy_fullscreen_works;

        tracing::debug!(
            "Updated window mode capabilities: fullscreen_capable={}, bevy_fullscreen_works={}",
            self.fullscreen_capable,
            self.bevy_fullscreen_works
        );
    }

    /// Set new target mode if different from current
    /// Uses target_mode field
    #[inline]
    pub fn set_target_mode(&mut self, new_target: WindowMode) {
        if !std::mem::discriminant(&self.target_mode).eq(&std::mem::discriminant(&new_target)) {
            tracing::debug!(
                "Target window mode changed from {:?} to {:?}",
                self.target_mode,
                new_target
            );
            self.target_mode = new_target;
        }
    }

    /// Get time since last mode change for diagnostic purposes
    /// Uses last_mode_change field
    #[inline]
    pub fn time_since_last_change(&self) -> Option<std::time::Duration> {
        self.last_mode_change.map(|instant| instant.elapsed())
    }
}

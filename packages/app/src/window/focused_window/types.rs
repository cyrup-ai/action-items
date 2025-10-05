//! Data types and structures for focused window detection

use bevy::window::Monitor;

/// Rectangle representing window or screen bounds with integer coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl WindowBounds {
    /// Create new window bounds
    #[inline]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Convert Bevy Monitor to WindowBounds for overlap calculation
impl From<&Monitor> for WindowBounds {
    #[inline]
    fn from(monitor: &Monitor) -> Self {
        WindowBounds::new(
            monitor.physical_position.x,
            monitor.physical_position.y,
            monitor.physical_width as i32,
            monitor.physical_height as i32,
        )
    }
}

/// Error types for focused window detection
#[derive(Debug, thiserror::Error)]
pub enum FocusedWindowError {
    #[error("No focused window found")]
    NoFocusedWindow,
    #[error("System API error: {0}")]
    SystemError(String),
    #[error("Display server not found - neither X11 nor Wayland available")]
    #[allow(dead_code)]
    DisplayServerNotFound,
    #[error("Wayland compositor not supported: {0}")]
    #[allow(dead_code)]
    CompositorNotSupported(String),
    // UnsupportedPlatform removed - platform routing ensures only appropriate functions are
    // called
}

/// Result type for focused window operations
pub type FocusedWindowResult<T> = Result<T, FocusedWindowError>;

//! Window management error types
//!
//! Production-quality error types for window operations with detailed context

/// Errors that can occur during window mode management operations
#[derive(Debug, thiserror::Error)]
pub enum WindowModeError {
    #[error("Mode change cooldown still active")]
    CooldownActive,
    #[error("Fullscreen capability not available on this system")]
    #[allow(dead_code)]
    FullscreenNotCapable,
    #[error("Bevy BorderlessFullscreen implementation not working")]
    #[allow(dead_code)]
    BevyFullscreenFailed,
}

/// Errors that can occur during screen dimension calculations
#[derive(Debug, thiserror::Error)]
pub enum ScreenDimensionError {
    #[error("No monitors found in system")]
    #[allow(dead_code)]
    NoMonitorsFound,
    #[error("Active monitor not found")]
    ActiveMonitorNotFound,
    #[error("Invalid monitor entity")]
    InvalidMonitorEntity,
    #[error("Monitor dimensions invalid: width={width}, height={height}")]
    InvalidDimensions { width: u32, height: u32 },
}

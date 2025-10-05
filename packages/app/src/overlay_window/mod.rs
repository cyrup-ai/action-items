//! Cross-platform overlay window configuration module
//!
//! This module provides cross-platform overlay window functionality with
//! platform-specific implementations for macOS, Windows, and Unix systems.
//!
//! The module has been decomposed into logical submodules:
//! - `plugin` - Bevy plugin and main system
//! - `platform` - Platform-specific configuration dispatcher
//! - `macos` - macOS NSPanel configuration
//! - `windows` - Windows non-activating overlay
//! - `unix` - Unix/Linux platform configurations (Wayland, X11, XCB)
//! - `types` - Error types and window attributes

pub mod macos;
pub mod platform;
pub mod plugin;
pub mod types;
pub mod unix;
pub mod windows;

// Re-export main types and functions
pub use plugin::OverlayWindowPlugin;
use tracing::{debug, error, info, warn};
pub use types::{OverlayConfigurationError, OverlayConfigurationResult, OverlayError};

/// Comprehensive overlay error diagnostic system
/// Uses all OverlayError checking methods for detailed error classification and recovery guidance
#[allow(dead_code)] // Used for error diagnostics - may not be called in all configurations
pub fn handle_overlay_error_diagnostics(overlay_error: &OverlayError) -> String {
    let mut diagnostics = Vec::new();
    let mut recovery_suggestions = Vec::new();

    // Use all 6 OverlayError checking methods for comprehensive error analysis
    if overlay_error.is_platform_error() {
        diagnostics.push("Platform compatibility issue detected");
        recovery_suggestions.push("Verify window handle matches expected platform");
        error!(
            "Platform mismatch error in overlay configuration: {}",
            overlay_error
        );
    }

    if overlay_error.is_handle_error() {
        diagnostics.push("Window handle access failure detected");
        recovery_suggestions
            .push("Ensure window is properly initialized before overlay configuration");
        error!(
            "Handle access error in overlay configuration: {}",
            overlay_error
        );
    }

    if overlay_error.is_wayland_error() {
        diagnostics.push("Wayland-specific error detected");
        recovery_suggestions.push("Check Wayland compositor support and layer shell availability");
        warn!("Wayland overlay error: {}", overlay_error);
    }

    if overlay_error.is_windows_error() {
        diagnostics.push("Windows-specific error detected");
        recovery_suggestions
            .push("Verify Windows API access and extended window style permissions");
        warn!("Windows overlay error: {}", overlay_error);
    }

    if overlay_error.is_x11_error() {
        diagnostics.push("X11/XCB error detected");
        recovery_suggestions.push("Check X11 connection and override_redirect permissions");
        warn!("X11/XCB overlay error: {}", overlay_error);
    }

    if overlay_error.is_macos_error() {
        diagnostics.push("macOS-specific error detected");
        recovery_suggestions
            .push("Verify NSWindow setLevel permissions and macOS security settings");
        warn!("macOS overlay error: {}", overlay_error);
    }

    // If none of the specific error types matched, it's an unrecognized error pattern
    if diagnostics.is_empty() {
        diagnostics.push("Unrecognized overlay error pattern");
        recovery_suggestions.push("Review error details and platform-specific requirements");
        debug!("Unclassified overlay error: {}", overlay_error);
    }

    let diagnostic_summary = format!(
        "Overlay Error Diagnostics: {} | Recovery: {}",
        diagnostics.join(", "),
        recovery_suggestions.join(" & ")
    );

    info!("Overlay error diagnostic complete: {}", diagnostic_summary);
    diagnostic_summary
}

/// Cross-platform overlay configuration system
/// Uses both platform-specific configuration functions based on the current platform
pub fn configure_overlay_window_cross_platform(
    winit_window: &winit::window::Window,
) -> Result<OverlayConfigurationResult, OverlayConfigurationError> {
    let mut results: Vec<OverlayConfigurationResult> = Vec::new();

    // Configure overlay based on platform using the appropriate function
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        match unix::configure_unix_overlay(winit_window) {
            Ok(()) => {
                let result = OverlayConfigurationResult::success(
                    "Unix",
                    "overlay configuration completed successfully",
                );
                info!("✅ Unix overlay configuration completed successfully");
                return Ok(result);
            },
            Err(overlay_error) => {
                let diagnostic = handle_overlay_error_diagnostics(&overlay_error);
                let result = OverlayConfigurationResult::failure(
                    "Unix",
                    format!("overlay failed: {}", diagnostic),
                );
                results.push(result);
                error!("Unix overlay configuration failed: {}", overlay_error);
            },
        }
    }

    #[cfg(target_os = "windows")]
    {
        match windows::configure_windows_noactivate(winit_window) {
            Ok(()) => {
                let result = OverlayConfigurationResult::success(
                    "Windows",
                    "overlay configuration completed successfully",
                );
                info!("✅ Windows overlay configuration completed successfully");
                return Ok(result);
            },
            Err(overlay_error) => {
                let diagnostic = handle_overlay_error_diagnostics(&overlay_error);
                let result = OverlayConfigurationResult::failure(
                    "Windows",
                    format!("overlay failed: {}", diagnostic),
                );
                results.push(result);
                error!("Windows overlay configuration failed: {}", overlay_error);
            },
        }
    }

    #[cfg(target_os = "macos")]
    {
        match macos::configure_macos_panel(winit_window) {
            Ok(()) => {
                let result = OverlayConfigurationResult::success(
                    "macOS",
                    "overlay configuration completed successfully",
                );
                info!("✅ macOS overlay configuration completed successfully");
                return Ok(result);
            },
            Err(overlay_error) => {
                let diagnostic = handle_overlay_error_diagnostics(&overlay_error);
                let result = OverlayConfigurationResult::failure(
                    "macOS",
                    format!("overlay failed: {}", diagnostic),
                );
                results.push(result);
                error!("macOS overlay configuration failed: {}", overlay_error);
            },
        }
    }

    // If we reach here, all platform configurations failed
    warn!("All platform overlay configurations failed");
    Err(OverlayConfigurationError::AllPlatformsFailed)
}

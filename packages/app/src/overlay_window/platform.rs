//! Platform-specific overlay window configuration dispatcher

use super::types::OverlayError;

/// Platform-specific overlay window configuration
pub fn configure_platform_overlay(
    winit_window: &winit::window::Window,
) -> Result<(), OverlayError> {
    #[cfg(target_os = "macos")]
    return super::macos::configure_macos_panel(winit_window);

    #[cfg(target_os = "windows")]
    return super::windows::configure_windows_noactivate(winit_window);

    #[cfg(all(unix, not(target_os = "macos")))]
    return super::unix::configure_unix_overlay(winit_window);

    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    {
        tracing::info!(
            "Platform-specific overlay configuration not available, using default behavior"
        );
        Ok(())
    }
}

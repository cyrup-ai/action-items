//! Windows-specific window activation code
//!
//! This module contains all Windows-specific window activation logic using
//! Win32 APIs.

#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use tracing::{debug, error, info};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    ASFW_ANY, AllowSetForegroundWindow, BringWindowToTop, SW_RESTORE, SetForegroundWindow,
    ShowWindow,
};

#[cfg(target_os = "windows")]
use super::super::types::*;

/// Activate window on Windows using Win32 APIs
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn activate_window_windows(winit_window: &winit::window::Window) -> ActivationResult<()> {
    use raw_window_handle::Win32WindowHandle;

    let window_handle = winit_window
        .window_handle()
        .map_err(|_| ActivationError::WindowHandle(ERROR_WINDOW_HANDLE))?;

    match window_handle.as_raw() {
        RawWindowHandle::Win32(Win32WindowHandle { hwnd, .. }) => {
            // SAFETY: Win32 API calls require unsafe for FFI
            // Approved by David Maple 08/17/2025
            unsafe {
                let hwnd = hwnd.get() as isize;

                if hwnd == 0 {
                    return Err(ActivationError::UnsupportedPlatform(
                        ERROR_WIN32_INVALID_HWND,
                    ));
                }

                // Allow this process to set foreground window
                let allow_result = AllowSetForegroundWindow(ASFW_ANY);
                if allow_result == 0 {
                    return Err(ActivationError::UnsupportedPlatform(
                        ERROR_WIN32_ALLOW_FOREGROUND,
                    ));
                }

                // Bring window to top first
                let top_result = BringWindowToTop(hwnd);
                if top_result == 0 {
                    return Err(ActivationError::UnsupportedPlatform(
                        ERROR_WIN32_BRING_TO_TOP,
                    ));
                }

                // Restore if minimized
                let show_result = ShowWindow(hwnd, SW_RESTORE);
                if show_result == 0 {
                    // ShowWindow can return 0 if window was already restored, not an error
                    debug!("ShowWindow returned 0 - window may already be visible");
                }

                // Bring to foreground
                let foreground_result = SetForegroundWindow(hwnd);
                if foreground_result == 0 {
                    return Err(ActivationError::UnsupportedPlatform(
                        ERROR_WIN32_SET_FOREGROUND,
                    ));
                }
            }

            info!("Windows window activation completed successfully");
            Ok(())
        },
        _ => Err(ActivationError::UnsupportedPlatform(
            ERROR_UNSUPPORTED_PLATFORM_WINDOWS,
        )),
    }
}

// Platform-specific implementation unavailable for this target
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn activate_window_windows(
    _winit_window: &winit::window::Window,
) -> super::super::types::ActivationResult<()> {
    Err(super::super::types::ActivationError::UnsupportedPlatform(
        "Windows activation called on non-Windows platform",
    ))
}

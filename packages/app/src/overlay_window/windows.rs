//! Windows non-activating overlay configuration

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, SetWindowLongPtrW, WS_EX_NOACTIVATE, WS_EX_TOPMOST,
};

use super::types::OverlayError;

#[cfg(target_os = "windows")]
#[allow(dead_code)] // Platform-specific function - only used on Windows systems
pub fn configure_windows_noactivate(
    winit_window: &winit::window::Window,
) -> Result<(), OverlayError> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let window_handle = winit_window
        .window_handle()
        .map_err(|_| OverlayError::HandleAccess)?;

    if let RawWindowHandle::Win32(win32_handle) = window_handle.as_raw() {
        unsafe {
            let hwnd = win32_handle.hwnd.get() as isize;

            // Get current extended window style
            let current_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            if current_style == 0 {
                return Err(OverlayError::WindowsGetStyleFailed);
            }

            // Add WS_EX_NOACTIVATE and WS_EX_TOPMOST flags
            let new_style = current_style | (WS_EX_NOACTIVATE as isize) | (WS_EX_TOPMOST as isize);

            // Apply the new style
            let result = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
            if result == 0 && new_style != current_style {
                return Err(OverlayError::WindowsSetStyleFailed);
            }

            // Validate that the style was actually applied
            let verified_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            if verified_style & (WS_EX_NOACTIVATE as isize) == 0
                || verified_style & (WS_EX_TOPMOST as isize) == 0
            {
                return Err(OverlayError::WindowsSetStyleFailed);
            }

            tracing::info!("✅ Configured Windows non-activating overlay");
        }
        Ok(())
    } else {
        Err(OverlayError::PlatformMismatch)
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)] // Platform-specific function - only used on Windows systems
pub fn configure_windows_noactivate(
    _winit_window: &winit::window::Window,
) -> Result<(), OverlayError> {
    tracing::warn!("Windows overlay configuration not available on this platform");
    Err(OverlayError::PlatformMismatch)
}

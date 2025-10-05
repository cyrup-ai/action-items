//! Windows-specific focused window detection using Win32 APIs

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{HWND, RECT};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

#[cfg(target_os = "windows")]
use super::types::{FocusedWindowError, FocusedWindowResult, WindowBounds};

/// Windows-specific focused window detection using Win32 APIs
/// Uses GetForegroundWindow() and GetWindowRect() for accurate bounds detection
#[cfg(target_os = "windows")]
#[inline]
pub fn get_focused_window_bounds_windows() -> FocusedWindowResult<WindowBounds> {
    unsafe {
        // Get the handle of the currently focused (foreground) window
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0 == 0 {
            return Err(FocusedWindowError::NoFocusedWindow);
        }

        // Get the window rectangle (bounds) in screen coordinates
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        let result = GetWindowRect(hwnd, &mut rect);
        if result == 0 {
            // GetWindowRect failed - get last error would be ideal but keeping it simple
            return Err(FocusedWindowError::SystemError(
                "GetWindowRect failed to retrieve window bounds".to_string(),
            ));
        }

        // Calculate width and height from the rectangle
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        // Validate that we got reasonable dimensions
        if width <= 0 || height <= 0 {
            return Err(FocusedWindowError::SystemError(
                "Invalid window dimensions detected".to_string(),
            ));
        }

        tracing::debug!(
            "Windows focused window detected: {}x{} at ({}, {})",
            width,
            height,
            rect.left,
            rect.top
        );

        Ok(WindowBounds::new(rect.left, rect.top, width, height))
    }
}

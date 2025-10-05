//! Cross-platform entry point for focused window detection

use super::types::{FocusedWindowResult, WindowBounds};

/// Get the bounds of the currently focused application window
/// Cross-platform implementation supporting macOS, Windows, and Linux
#[inline]
pub fn get_focused_window_bounds() -> FocusedWindowResult<WindowBounds> {
    #[cfg(target_os = "macos")]
    {
        super::macos::get_focused_window_bounds_macos()
    }

    #[cfg(target_os = "windows")]
    {
        super::windows::get_focused_window_bounds_windows()
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        super::linux::get_focused_window_bounds_linux()
    }
}

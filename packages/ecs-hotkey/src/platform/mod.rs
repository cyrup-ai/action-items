//! Platform-specific hotkey handling
//!
//! Cross-platform abstractions for hotkey management with platform-specific optimizations.

pub mod linux;
pub mod macos;
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux_wayland;
#[cfg(target_os = "linux")]
pub mod linux_wayland_kde;
#[cfg(target_os = "linux")]
pub mod linux_wayland_portal;

pub use linux::*;
pub use macos::*;
pub use windows::*;

#[cfg(target_os = "linux")]
pub use linux_wayland::*;

/// Check platform-specific permissions and requirements
pub fn check_platform_permissions() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos::check_macos_permissions().map_err(|e| e.to_string())
    }

    #[cfg(target_os = "windows")]
    {
        windows::check_windows_permissions()
    }

    #[cfg(target_os = "linux")]
    {
        linux::check_linux_permissions()
    }
}

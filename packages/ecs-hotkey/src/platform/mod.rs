//! Platform-specific hotkey handling
//!
//! Cross-platform abstractions for hotkey management with platform-specific optimizations.

pub mod linux;
pub mod macos;
pub mod windows;

pub use linux::*;
pub use macos::*;
pub use windows::*;

/// Platform-specific hotkey initialization
pub fn init_platform_hotkeys() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos::init_macos_hotkey_system().map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        windows::init_windows_hotkeys().map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        linux::init_linux_hotkeys().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

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

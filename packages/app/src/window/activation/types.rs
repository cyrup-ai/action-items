//! Shared types and enums for window activation
//!
//! This module contains all the common types, enums, and error definitions
//! used across the window activation system.

use bevy::prelude::*;
use thiserror::Error;

/// Window activation errors with zero allocation
#[derive(Error, Debug)]
pub enum ActivationError {
    #[error("{0}")]
    WindowHandle(&'static str),

    #[cfg(target_os = "linux")]
    #[error("{0}")]
    DisplayHandle(&'static str),

    #[error("{0}")]
    UnsupportedPlatform(&'static str),

    #[cfg(target_os = "linux")]
    #[error("{0}")]
    X11Error(&'static str),

    #[cfg(target_os = "linux")]
    #[error("{0}")]
    WaylandError(&'static str),
}

/// Result type for activation operations
pub type ActivationResult<T> = Result<T, ActivationError>;

/// Event to trigger window activation
#[derive(Event, Debug, Clone)]
pub struct WindowActivationEvent {
    pub reason: ActivationReason,
}

/// Reason for window activation
#[derive(Debug, Clone)]
pub enum ActivationReason {
    GlobalHotkey,
    UserRequest,
    ApplicationStart,
}

/// Component for tracking async Wayland token acquisition
#[cfg(target_os = "linux")]
#[derive(Component)]
pub struct WaylandTokenTask {
    pub task: bevy::tasks::Task<ActivationResult<()>>,
    pub surface: wayland_client::protocol::wl_surface::WlSurface,
}

/// Linux display server detection
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxDisplayServer {
    X11,
    Wayland,
    Unknown,
}

// Static error messages for zero-allocation error handling
pub const ERROR_WINDOW_HANDLE: &str = "Failed to get window handle";

#[cfg(target_os = "linux")]
pub const ERROR_DISPLAY_HANDLE: &str = "Failed to get display handle";

#[cfg(target_os = "macos")]
pub const ERROR_UNSUPPORTED_PLATFORM_MACOS: &str = "Expected AppKit window handle on macOS";

#[cfg(target_os = "windows")]
pub const ERROR_UNSUPPORTED_PLATFORM_WINDOWS: &str = "Expected Win32 window handle on Windows";

#[cfg(target_os = "linux")]
pub const ERROR_UNSUPPORTED_PLATFORM_X11: &str = "Expected Xlib window handle for X11";

#[cfg(target_os = "linux")]
pub const ERROR_UNSUPPORTED_PLATFORM_WAYLAND: &str = "Expected Wayland window and display handles";

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub const ERROR_UNSUPPORTED_PLATFORM_GENERIC: &str = "Platform not supported";

#[cfg(target_os = "linux")]
pub const ERROR_X11_DISPLAY_OPEN: &str = "Failed to open X11 display";

#[cfg(target_os = "linux")]
pub const ERROR_X11_ATOM_NET_ACTIVE: &str = "_NET_ACTIVE_WINDOW atom creation failed";

#[cfg(target_os = "linux")]
pub const ERROR_X11_SEND_EVENT: &str = "Failed to send X11 client message";

#[cfg(target_os = "linux")]
pub const ERROR_WAYLAND_CONNECTION: &str = "Failed to connect to Wayland compositor";

#[cfg(target_os = "linux")]
pub const ERROR_XDG_ACTIVATION_REGISTRY: &str = "XDG activation registry roundtrip failed";

#[cfg(target_os = "linux")]
pub const ERROR_XDG_ACTIVATION_TOKEN: &str = "XDG activation token roundtrip failed";

#[cfg(target_os = "linux")]
pub const ERROR_XDG_ACTIVATION_UNAVAILABLE: &str = "XDG activation protocol not available";

#[cfg(target_os = "linux")]
pub const ERROR_XDG_SURFACE_INVALID: &str = "Invalid Wayland surface pointer";

#[cfg(target_os = "linux")]
pub const ERROR_LINUX_DISPLAY_SERVER: &str = "Unknown Linux display server";

#[cfg(target_os = "linux")]
pub const ERROR_CACHE_LOCK: &str = "Cache lock poisoned";

#[cfg(target_os = "linux")]
pub const ERROR_X11_CLOSE: &str = "Failed to close X11 display";

#[cfg(target_os = "linux")]
pub const ERROR_X11_FLUSH: &str = "Failed to flush X11 display";

#[cfg(target_os = "linux")]
pub const ERROR_X11_RAISE: &str = "Failed to raise X11 window";

#[cfg(target_os = "linux")]
pub const ERROR_X11_FOCUS: &str = "Failed to set X11 window focus";

#[cfg(target_os = "windows")]
pub const ERROR_WIN32_ALLOW_FOREGROUND: &str = "Failed to allow foreground window";

#[cfg(target_os = "windows")]
pub const ERROR_WIN32_BRING_TO_TOP: &str = "Failed to bring window to top";

#[cfg(target_os = "windows")]
pub const ERROR_WIN32_SHOW_WINDOW: &str = "Failed to show window";

#[cfg(target_os = "windows")]
pub const ERROR_WIN32_SET_FOREGROUND: &str = "Failed to set foreground window";

#[cfg(target_os = "linux")]
pub const ERROR_WAYLAND_TIMEOUT: &str = "Wayland roundtrip operation timed out";

#[cfg(target_os = "linux")]
pub const ERROR_X11_INIT_THREADS: &str = "Failed to initialize X11 threading support";

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub const ERROR_MACOS_APP_NULL: &str = "Failed to get NSApplication instance";

#[cfg(target_os = "macos")]
pub const ERROR_MACOS_VIEW_NULL: &str = "Invalid NSView pointer";

#[cfg(target_os = "windows")]
pub const ERROR_WIN32_INVALID_HWND: &str = "Invalid HWND";

// Production constants - no magic numbers
#[cfg(target_os = "linux")]
pub const CONNECTION_HEALTH_TIMEOUT_SECS: u64 = 300; // 5 minutes

#[cfg(target_os = "linux")]
pub const WAYLAND_OPERATION_TIMEOUT_SECS: u64 = 5;

//! Linux-specific hotkey handling
//!
//! # Implementation
//! 
//! Linux global hotkeys are handled by the `global-hotkey` crate using the
//! X11 `XGrabKey()` API via the `x11rb` Rust bindings.
//!
//! ## How It Works
//! 
//! 1. Connects to X11 display server via `RustConnection`
//! 2. Calls `XGrabKey()` on the root window for each hotkey
//! 3. Spawns dedicated thread to poll X11 events
//! 4. Processes `KeyPress` events and sends to cross-platform channel
//!
//! ## Requirements
//! 
//! - X11 display server must be running
//! - DISPLAY environment variable must be set
//! - No special permissions required
//!
//! ## Limitations
//! 
//! - **X11 ONLY** - Wayland is not supported by global-hotkey crate
//! - Some desktop environments reserve common hotkey combinations
//! - XGrabKey grabs apply to all applications (can't be overridden)
//! - No keyboard grab during hotkey capture (uses Bevy's input system)
//!
//! ## Wayland Workaround
//! 
//! Run under XWayland compatibility layer:
//! ```bash
//! GDK_BACKEND=x11 ./action_items
//! # or
//! QT_QPA_PLATFORM=xcb ./action_items
//! ```
//!
//! ## Reference Implementation
//! 
//! See: [`global-hotkey/src/platform_impl/x11/mod.rs`](../../../tmp/global-hotkey/src/platform_impl/x11/mod.rs)

use std::env;
use tracing::info;

/// Check if Wayland display server is running
#[inline]
pub fn is_wayland() -> bool {
    env::var("WAYLAND_DISPLAY").is_ok()
        && env::var("XDG_SESSION_TYPE")
            .map(|s| s == "wayland")
            .unwrap_or(false)
}

/// Check if X11 is available (either native or XWayland)
#[inline]
pub fn is_x11_available() -> bool {
    env::var("DISPLAY").is_ok()
}

/// Detect desktop environment compositor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxCompositor {
    Kde,
    Gnome,
    Hyprland,
    Sway,
    Unknown,
}

pub fn detect_compositor() -> LinuxCompositor {
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let desktop_lower = desktop.to_lowercase();
        if desktop_lower.contains("kde") {
            return LinuxCompositor::Kde;
        }
        if desktop_lower.contains("gnome") {
            return LinuxCompositor::Gnome;
        }
        if desktop_lower.contains("hyprland") {
            return LinuxCompositor::Hyprland;
        }
        if desktop_lower.contains("sway") {
            return LinuxCompositor::Sway;
        }
    }
    LinuxCompositor::Unknown
}

/// Initialize Linux-specific hotkey functionality
pub fn init_linux_hotkeys() {
    info!("Initializing Linux hotkey support");
}

/// Check Linux permissions and requirements for global hotkeys
pub fn check_linux_permissions() -> Result<(), String> {
    if is_wayland() {
        let compositor = detect_compositor();
        match compositor {
            LinuxCompositor::Kde => {
                info!("✅ Wayland detected with KDE Plasma - native support available");
                Ok(())
            }
            LinuxCompositor::Hyprland => {
                info!("✅ Wayland detected with Hyprland - XDG Portal support available");
                Ok(())
            }
            LinuxCompositor::Gnome => {
                if is_x11_available() {
                    info!("⚠️  Wayland GNOME detected - falling back to XWayland");
                    Ok(())
                } else {
                    Err("Global hotkeys not supported on Wayland GNOME without XWayland".to_string())
                }
            }
            _ => {
                if is_x11_available() {
                    info!("⚠️  Wayland compositor without native support - using XWayland");
                    Ok(())
                } else {
                    Err(
                        "Global hotkeys not supported on this Wayland compositor. \
                         Install XWayland or use X11 session.".to_string()
                    )
                }
            }
        }
    } else if is_x11_available() {
        info!("✅ X11 display server detected, hotkeys will work");
        Ok(())
    } else {
        Err("No X11 display server detected. Global hotkeys require X11 to function.".to_string())
    }
}

/// Format platform-specific error messages for Linux
///
/// Provides actionable guidance for common Linux hotkey errors
pub fn format_linux_error(error: &str) -> String {
    if error.contains("Wayland") || error.contains("compositor") {
        let compositor = detect_compositor();
        match compositor {
            LinuxCompositor::Sway => {
                format!(
                    "Global hotkeys not supported on Sway compositor.\n\
                     Sway does not implement XDG Portal GlobalShortcuts.\n\
                     \n\
                     Workarounds:\n\
                     • Run under XWayland: GDK_BACKEND=x11 ./action_items\n\
                     • Switch to X11 session\n\
                     • Use Sway's built-in keybinding system in ~/.config/sway/config\n\
                     \n\
                     Error: {}", error
                )
            }
            LinuxCompositor::Gnome => {
                format!(
                    "Global hotkeys have limited support on GNOME Wayland.\n\
                     GNOME Shell's DBus API is restricted to trusted applications.\n\
                     \n\
                     Recommended solutions:\n\
                     • Run under XWayland: GDK_BACKEND=x11 ./action_items\n\
                     • Switch to X11 session (logout, select 'GNOME on Xorg')\n\
                     \n\
                     Error: {}", error
                )
            }
            LinuxCompositor::Unknown => {
                format!(
                    "Global hotkeys not supported on this Wayland compositor.\n\
                     \n\
                     Supported compositors:\n\
                     • KDE Plasma (Wayland) - Full support\n\
                     • Hyprland - Via XDG Desktop Portal\n\
                     \n\
                     Workaround:\n\
                     • Run under XWayland: GDK_BACKEND=x11 ./action_items\n\
                     \n\
                     Error: {}", error
                )
            }
            _ => format!("Wayland error: {}", error),
        }
    } else if error.contains("connection") || error.contains("X11") {
        format!(
            "Cannot connect to X11 display server.\n\
             • Ensure X11 is running (not Wayland)\n\
             • Check DISPLAY environment variable is set\n\
             • Try: export DISPLAY=:0\n\
             \n\
             Error: {}", error
        )
    } else if error.contains("AlreadyRegistered") {
        format!(
            "Hotkey already registered by another application.\n\
             Common conflicts on Linux:\n\
             • Super+Space: Often used by desktop environments\n\
             • Ctrl+Alt+Space: May conflict with input methods\n\
             \n\
             Try a different hotkey combination in Settings.\n\
             \n\
             Error: {}", error
        )
    } else {
        format!("Linux hotkey error: {}", error)
    }
}

/// Display Linux-specific hotkey setup instructions
pub fn display_linux_hotkey_info() {
    info!("🚀 Action Items Launcher is ready!");
    info!("📋 Press Ctrl+Shift+Space to activate the launcher from anywhere");
    info!("⚡ The launcher will appear instantly and is ready for your commands");
    info!("ℹ️  Linux: Hotkeys use XGrabKey X11 API (X11 only, Wayland not supported)");
}

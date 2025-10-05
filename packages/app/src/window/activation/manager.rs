//! Core window activation management
//!
//! This module provides the main window activation coordination and management
//! functionality, integrating with Bevy systems and coordinating platform-specific
//! activation methods.

use bevy::prelude::*;
use bevy::winit::WinitWindows;
use tracing::{debug, error, info, warn};

#[cfg(target_os = "linux")]
use super::platform::linux::activate_window_linux;
#[cfg(target_os = "macos")]
use super::platform::macos::activate_window_macos;
#[cfg(target_os = "windows")]
use super::platform::windows::activate_window_windows;
use super::types::*;

/// Cross-platform window activation system for Bevy
///
/// This system activates windows when needed using platform-specific methods.
/// Uses Single<&mut Window> pattern for primary window operations per Bevy best practices.
pub fn window_activation_system(
    #[cfg(target_os = "linux")] mut commands: Commands,
    mut window: Single<&mut Window>,
    winit_windows: NonSend<WinitWindows>,
    mut activation_events: EventReader<WindowActivationEvent>,
) {
    for event in activation_events.read() {
        info!("Window activation requested due to: {:?}", event.reason);

        // First, ensure window is visible
        window.visible = true;
        window.set_minimized(false);

        // Get the primary winit window - use first available window
        if let Some((_, winit_window)) = winit_windows.windows.iter().next() {
            let result = activate_window_with_handle(
                #[cfg(target_os = "linux")]
                commands.reborrow(),
                winit_window,
            );

            match result {
                Ok(()) => {
                    info!(
                        "Primary window activation completed successfully for reason: {:?}",
                        event.reason
                    );
                },
                Err(e) => {
                    error!(
                        "Primary window activation failed for reason {:?}: {}",
                        event.reason, e
                    );
                },
            }
        } else {
            error!("Could not find primary winit window - this should never happen");
        }
    }
}

/// Activate window using the actual winit window handle
fn activate_window_with_handle(
    #[cfg(target_os = "linux")] mut commands: Commands,
    winit_window: &winit::window::Window,
) -> ActivationResult<()> {
    #[cfg(target_os = "macos")]
    return activate_window_macos(winit_window);

    #[cfg(target_os = "windows")]
    return activate_window_windows(winit_window);

    #[cfg(target_os = "linux")]
    return activate_window_linux(commands, winit_window);

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    Err(ActivationError::UnsupportedPlatform(
        ERROR_UNSUPPORTED_PLATFORM_GENERIC,
    ))
}

/// Legacy function for backwards compatibility
pub fn activate_window(window: &mut Window) {
    warn!(
        "Using legacy activate_window function - consider migrating to the \
         window_activation_system"
    );

    window.visible = true;
    window.set_minimized(false);

    debug!("Legacy window activation completed - functionality limited without window handle");
}

/// Initialize platform-specific window activation requirements
pub fn init_window_activation() {
    #[cfg(target_os = "macos")]
    {
        use super::policies::check_accessibility_permissions_macos;
        if !check_accessibility_permissions_macos() {
            error!("Missing accessibility permissions - global hotkeys may not work!");
        }
    }

    info!("Window activation system initialized");
}

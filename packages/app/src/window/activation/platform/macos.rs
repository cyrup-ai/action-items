//! macOS-specific window activation code
//!
//! This module contains all macOS-specific window activation logic using
//! AppKit and Objective-C APIs.

#[cfg(target_os = "macos")]
use objc2_app_kit::NSApplication;
#[cfg(target_os = "macos")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[cfg(target_os = "macos")]
use tracing::{debug, info};

#[cfg(target_os = "macos")]
use super::super::types::*;

/// Activate window on macOS using AppKit
#[cfg(target_os = "macos")]
pub fn activate_window_macos(winit_window: &winit::window::Window) -> ActivationResult<()> {
    use raw_window_handle::AppKitWindowHandle;

    let window_handle = winit_window
        .window_handle()
        .map_err(|_| ActivationError::WindowHandle(ERROR_WINDOW_HANDLE))?;

    match window_handle.as_raw() {
        RawWindowHandle::AppKit(AppKitWindowHandle { ns_view, .. }) => {
            // SAFETY: AppKit API calls require unsafe for FFI
            // Approved by David Maple 08/17/2025
            let ns_view_ptr = ns_view.as_ptr();
            if ns_view_ptr.is_null() {
                return Err(ActivationError::UnsupportedPlatform(ERROR_MACOS_VIEW_NULL));
            }

            // Get main thread marker for NSApplication
            let mtm =
                objc2::MainThreadMarker::new().ok_or(ActivationError::UnsupportedPlatform(
                    "NSApplication activation must be called from the main thread",
                ))?;
            let app = NSApplication::sharedApplication(mtm);

            // Activate the application and bring to front
            app.activate();

            debug!("NSApplication activation completed");

            info!("macOS window activation completed successfully");
            Ok(())
        },
        _ => Err(ActivationError::UnsupportedPlatform(
            ERROR_UNSUPPORTED_PLATFORM_MACOS,
        )),
    }
}

// Platform-specific implementation unavailable for this target
#[cfg(not(target_os = "macos"))]
pub fn activate_window_macos(
    _winit_window: &winit::window::Window,
) -> super::super::types::ActivationResult<()> {
    Err(super::super::types::ActivationError::UnsupportedPlatform(
        "macOS activation called on non-macOS platform",
    ))
}

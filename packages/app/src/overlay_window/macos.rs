//! macOS NSPanel configuration for non-activating overlay behavior

#[cfg(target_os = "macos")]
use objc2::msg_send;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tracing::info;

use super::types::OverlayError;

/// NSFloatingWindowLevel constant for windows that float above most other windows
/// This level appears above normal windows and most applications, including fullscreen apps
#[cfg(target_os = "macos")]
const NS_FLOATING_WINDOW_LEVEL: i64 = 3;

#[cfg(target_os = "macos")]
pub fn configure_macos_panel(winit_window: &winit::window::Window) -> Result<(), OverlayError> {
    use objc2::msg_send;
    use objc2_app_kit::NSWindow;

    let window_handle = winit_window
        .window_handle()
        .map_err(|_| OverlayError::HandleAccess)?;

    if let RawWindowHandle::AppKit(appkit_handle) = window_handle.as_raw() {
        // Get NSView from handle, then get its window
        let ns_view = appkit_handle.ns_view.as_ptr();
        let ns_window = unsafe {
            let view: *const objc2_foundation::NSObject = ns_view as *const _;
            let window_ptr: *const NSWindow = msg_send![view, window];
            &*window_ptr
        };

        // Set window level to appear over fullscreen applications
        unsafe {
            // Set the window level to floating window level
            let _: () = msg_send![ns_window, setLevel: NS_FLOATING_WINDOW_LEVEL];

            // Validate that the level was actually set by reading it back
            let current_level: i64 = msg_send![ns_window, level];
            if current_level != NS_FLOATING_WINDOW_LEVEL {
                return Err(OverlayError::MacOSSetLevelFailed);
            }
        }

        info!("✅ Configured macOS overlay window with FloatingWindowLevel");
        Ok(())
    } else {
        Err(OverlayError::PlatformMismatch)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_macos_panel(_winit_window: &winit::window::Window) -> Result<(), OverlayError> {
    Err(OverlayError::PlatformMismatch)
}

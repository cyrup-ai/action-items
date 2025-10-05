//! macOS-specific focused window detection using NSWorkspace + Core Graphics

#[cfg(target_os = "macos")]
use objc2_app_kit::NSWorkspace;

use super::types::{FocusedWindowError, FocusedWindowResult, WindowBounds};

/// macOS-specific focused window detection using NSWorkspace + Core Graphics
/// Professional implementation that correctly identifies the focused application
#[cfg(target_os = "macos")]
#[inline]
pub fn get_focused_window_bounds_macos() -> FocusedWindowResult<WindowBounds> {
    use core_foundation::array::{CFArrayGetCount, CFArrayGetValueAtIndex};
    use core_foundation::base::{CFTypeRef, TCFType};
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::number::CFNumber;
    use core_foundation::string::CFString;
    use core_graphics::display::CGDisplay;

    // Use constant value directly: K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY = 1 << 0 = 1
    const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1;

    // Step 1: Get the PID of the frontmost (focused) application using NSWorkspace
    let frontmost_app_pid = match get_frontmost_application_pid() {
        Some(pid) => pid,
        None => return Err(FocusedWindowError::NoFocusedWindow),
    };

    // Step 2: Get list of all on-screen windows
    let window_list =
        match CGDisplay::window_list_info(K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY, None) {
            Some(list) => list,
            None => {
                return Err(FocusedWindowError::SystemError(
                    "Failed to get window list".to_string(),
                ));
            },
        };

    let window_count = unsafe { CFArrayGetCount(window_list.as_concrete_TypeRef()) };
    if window_count == 0 {
        return Err(FocusedWindowError::NoFocusedWindow);
    }

    // Step 3: Find the main window of the frontmost application
    let mut best_window_bounds: Option<WindowBounds> = None;
    let mut best_window_area = 0i64;

    for i in 0..window_count {
        let window_info_ref =
            unsafe { CFArrayGetValueAtIndex(window_list.as_concrete_TypeRef(), i) };

        if window_info_ref.is_null() {
            continue;
        }

        // Cast to CFDictionary - window info is a dictionary
        let window_dict = unsafe {
            CFDictionary::<CFString, CFTypeRef>::wrap_under_get_rule(window_info_ref as _)
        };

        // Check if this window belongs to the frontmost application
        let owner_pid_key = CFString::from_static_string("kCGWindowOwnerPID");
        if let Some(pid_ref) = window_dict.find(owner_pid_key) {
            let pid_number = unsafe { CFNumber::wrap_under_get_rule(*pid_ref as _) };
            if let Some(window_pid) = pid_number.to_i32() {
                if window_pid != frontmost_app_pid {
                    continue; // Skip windows not belonging to frontmost app
                }
            } else {
                continue;
            }
        } else {
            continue;
        }

        // Check window layer - we want layer 0 (normal application windows)
        let layer_key = CFString::from_static_string("kCGWindowLayer");
        if let Some(layer_ref) = window_dict.find(layer_key) {
            let layer_number = unsafe { CFNumber::wrap_under_get_rule(*layer_ref as _) };
            if let Some(layer_value) = layer_number.to_i32() {
                // Skip if not a normal window (layer 0)
                if layer_value != 0 {
                    continue;
                }
            } else {
                continue;
            }
        } else {
            continue;
        }

        // Get window bounds dictionary
        let bounds_key = CFString::from_static_string("kCGWindowBounds");
        if let Some(bounds_ref) = window_dict.find(bounds_key) {
            // Parse CGRect from CFDictionary
            let bounds_dict = unsafe {
                CFDictionary::<CFString, CFNumber>::wrap_under_get_rule(*bounds_ref as _)
            };

            let x_key = CFString::from_static_string("X");
            let y_key = CFString::from_static_string("Y");
            let width_key = CFString::from_static_string("Width");
            let height_key = CFString::from_static_string("Height");

            let x = bounds_dict
                .find(x_key)
                .and_then(|num| num.to_f64())
                .unwrap_or(0.0) as i32;

            let y = bounds_dict
                .find(y_key)
                .and_then(|num| num.to_f64())
                .unwrap_or(0.0) as i32;

            let width = bounds_dict
                .find(width_key)
                .and_then(|num| num.to_f64())
                .unwrap_or(0.0) as i32;

            let height = bounds_dict
                .find(height_key)
                .and_then(|num| num.to_f64())
                .unwrap_or(0.0) as i32;

            // Validate bounds and find the largest window (likely the main window)
            if width > 0 && height > 0 {
                let bounds = WindowBounds::new(x, y, width, height);
                let area = bounds.area();

                if area > best_window_area {
                    best_window_area = area;
                    best_window_bounds = Some(bounds);
                }
            }
        }
    }

    best_window_bounds.ok_or(FocusedWindowError::NoFocusedWindow)
}

/// Get the PID of the frontmost (focused) application using NSWorkspace via objc2
#[cfg(target_os = "macos")]
fn get_frontmost_application_pid() -> Option<i32> {
    let workspace = NSWorkspace::sharedWorkspace();
    if let Some(frontmost_app) = workspace.frontmostApplication() {
        let pid = frontmost_app.processIdentifier();
        if pid > 0 { Some(pid) } else { None }
    } else {
        None
    }
}

//! Bevy window integration for menu initialization
//!
//! Note: Menu initialization happens in the plugin's build() method for macOS.
//! For Windows and Linux, manual initialization with window handles is required.

/// Initialize menu for Windows with HWND
#[cfg(target_os = "windows")]
pub fn initialize_for_windows(hwnd: isize) -> Result<(), String> {
    if let Some(menu) = crate::resources::get_global_menu() {
        unsafe {
            menu.init_for_hwnd(hwnd)
                .map_err(|e| format!("Failed to init menu for Windows: {}", e))
        }
    } else {
        Err("No global menu found. Ensure NativeMenuPlugin is added first.".to_string())
    }
}

/// Initialize menu for Linux with GTK window
#[cfg(target_os = "linux")]
pub fn initialize_for_linux(gtk_window: &gtk::Window, container: Option<&gtk::Box>) -> Result<(), String> {
    if let Some(menu) = crate::resources::get_global_menu() {
        menu.init_for_gtk_window(gtk_window, container)
            .map_err(|e| format!("Failed to init menu for Linux: {}", e))
    } else {
        Err("No global menu found. Ensure NativeMenuPlugin is added first.".to_string())
    }
}

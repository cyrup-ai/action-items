#[cfg(target_os = "macos")]
use bevy::prelude::*;
#[cfg(target_os = "macos")]
use bevy::winit::WinitWindows;
#[cfg(target_os = "macos")]
use tracing::info;

#[cfg(target_os = "macos")]
use crate::window::WindowModeManager;

#[cfg(target_os = "macos")]
pub fn configure_non_activating_panel(
    mut windows: Query<Entity, (With<Window>, Added<Window>)>,
    _winit_windows: NonSend<WinitWindows>,
    _window_mode_manager: ResMut<WindowModeManager>,
) {
    // Imports removed - overlay functionality handled by OverlayWindowPlugin

    // Note: Overlay window configuration is handled by OverlayWindowPlugin
    // This function is kept for compatibility but no longer needs to configure anything
    for _window_entity in windows.iter_mut() {
        // No-op - overlay functionality is handled by the dedicated OverlayWindowPlugin
        info!("✅ configure_non_activating_panel called (overlay handled by OverlayWindowPlugin)");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_non_activating_panel() {
    // No-op for non-macOS platforms
}

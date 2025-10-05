//! Bevy plugin and main system for overlay window configuration

use bevy::prelude::*;
use bevy::winit::WinitWindows;
use tracing::{error, info};

use super::platform::configure_platform_overlay;

/// Plugin for configuring cross-platform overlay windows
pub struct OverlayWindowPlugin;

impl Plugin for OverlayWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, configure_overlay_window);
    }
}

/// Configure the window as a non-activating overlay for each platform
fn configure_overlay_window(
    windows: Query<Entity, (With<Window>, Added<Window>)>,
    winit_windows: NonSend<WinitWindows>,
) {
    for window_entity in windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(window_entity) {
            match configure_platform_overlay(winit_window) {
                Ok(_) => info!("✅ Configured overlay window for platform"),
                Err(e) => error!("❌ Failed to configure overlay window: {}", e),
            }
        }
    }
}

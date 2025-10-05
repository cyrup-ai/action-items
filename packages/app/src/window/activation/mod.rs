//! Public API and module coordination for window activation
//!
//! This module provides the main public interface for window activation
//! functionality and coordinates all the submodules.

use bevy::prelude::*;
pub use manager::*;
// Re-export all public types and functions
pub use types::*;

// Platform-specific modules
pub mod platform {
    pub mod linux;
    pub mod macos;
    pub mod windows;
}

// Core modules
pub mod events;
pub mod manager;
pub mod policies;
pub mod types;

/// Plugin for window activation functionality
pub struct WindowActivationPlugin;

impl Plugin for WindowActivationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WindowActivationEvent>()
            .add_systems(Update, window_activation_system);

        #[cfg(target_os = "linux")]
        app.add_systems(Update, platform::linux::poll_wayland_tasks);

        app.add_systems(Startup, init_window_activation);
    }
}

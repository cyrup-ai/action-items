//! Action Items Permissions
//!
//! Cross-platform system permissions management with Bevy integration.
//! Provides unified access to camera, microphone, location, and other system permissions.
//! Includes a comprehensive wizard system for guided permission setup.
#![recursion_limit = "256"]

use bevy::prelude::*;

pub mod events;
pub mod manager;
pub mod plugin;
pub mod traits;
pub mod types;
pub mod wizard;

#[cfg(target_os = "macos")]
pub mod platforms;

#[cfg(target_os = "windows")]
pub mod platforms;

#[cfg(target_os = "linux")]
pub mod platforms;

// Re-export core types and traits
pub use events::*;
pub use manager::PermissionManager;
pub use plugin::{
    PermissionChanged, PermissionPlugin, PermissionRequest, PermissionRequestError,
    PermissionResource,
};
pub use traits::PermissionHandler;
pub use types::{PermissionError, PermissionStatus, PermissionType};
pub use wizard::PermissionWizardPlugin;

/// Convenience function to add the permission system to a Bevy app
pub fn add_permissions(app: &mut App) {
    app.add_plugins(PermissionPlugin);
}

/// Convenience function to add the permission system with wizard to a Bevy app
pub fn add_permissions_with_wizard(app: &mut App) {
    app.add_plugins((PermissionPlugin, PermissionWizardPlugin::default()));
}

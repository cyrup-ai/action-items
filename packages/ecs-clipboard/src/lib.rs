//! Action Items ECS Clipboard
//!
//! Bevy ECS plugin for cross-platform clipboard operations.
//! Provides unified access to text, HTML, images, and file clipboard data.
//! Built on top of the battle-tested arboard library for robust clipboard support.
#![recursion_limit = "256"]

use bevy::prelude::*;

pub mod manager;
pub mod plugin;
pub mod types;

// Re-export core types
// Compatibility exports for existing code
#[cfg(feature = "image-data")]
pub use arboard::ImageData;
pub use arboard::{Clipboard, Error};
pub use manager::ArboardManager;
pub use plugin::{
    ClipboardChanged, ClipboardPlugin, ClipboardRequest, ClipboardResource, ClipboardResponse,
};
pub use types::{ClipboardData, ClipboardError, ClipboardFormat, ClipboardChangeEvent};

/// Convenience function to add the clipboard system to a Bevy app
pub fn add_clipboard(app: &mut App) {
    app.add_plugins(ClipboardPlugin);
}

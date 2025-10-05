//! ECS Compression Service
//!
//! Production-quality compression service following established ECS patterns.
//! Zero allocation, blazing-fast, no locking, elegant ergonomic code.

pub mod manager;
pub mod plugin;
pub mod types;

use bevy::prelude::*;
pub use manager::CompressionManager;
pub use plugin::CompressionPlugin;
pub use types::{CompressionRequest, CompressionResponse, *};

/// Convenience function to add the compression system to a Bevy app
#[inline]
pub fn add_compression(app: &mut App) {
    app.add_plugins(CompressionPlugin::default());
}

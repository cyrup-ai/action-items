//! Native plugin wrapper module
//!
//! This module provides a Bevy Plugin wrapper around NativePlugin implementations,
//! allowing existing NativePlugin implementations to be registered as proper Bevy plugins
//! and participate in the Bevy ECS lifecycle without changing existing plugin trait interfaces.
//!
//! The module has been decomposed into logical submodules:
//! - `types` - Core types and metadata structures
//! - `wrapper` - Main wrapper implementation and constructor
//! - `plugin_impl` - Bevy Plugin trait implementation
//! - `systems` - ECS systems and components

pub mod core;
pub mod plugin_impl;
pub mod systems;
pub mod types;

// Re-export main types and functions
pub use systems::execute_native_plugin_system;
pub use types::{NativePluginWrapper, PluginComponent, PluginMetadata};

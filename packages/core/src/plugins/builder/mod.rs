//! Plugin builder module
//!
//! This module provides a fluent builder API for creating launcher plugins with
//! comprehensive configuration options and multiple interface implementations.
//!
//! The module has been decomposed into logical submodules:
//! - `types` - Type definitions and aliases for handler signatures
//! - `traits` - LauncherPlugin trait definition
//! - `plugin_builder` - Core builder structure and construction
//! - `configuration` - Fluent API methods for setting properties
//! - `built_plugin` - Built plugin implementation with NativePlugin trait
//! - `ffi_implementation` - FFI-compatible plugin interface implementation

pub mod built_plugin;
pub mod configuration;
pub mod ffi_implementation;
pub mod plugin_builder;
pub mod traits;
pub mod types;

// Re-export all public types and functions
pub use plugin_builder::PluginBuilder;
pub use traits::LauncherPlugin;
pub use types::{ActionHandler, RefreshHandler, SearchHandler};

//! Extism plugin adapter modules
//!
//! Modular organization of Extism plugin adapter functionality.

pub mod action_execution;
pub mod background_operations;
pub mod command_execution;
pub mod core;
pub mod function_calls;
pub mod initialization;
pub mod native_plugin_impl;
pub mod search_operations;

// Re-export the main adapter struct
pub use core::ExtismPluginAdapter;

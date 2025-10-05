//! Configuration manager modules
//!
//! Modular organization of configuration management functionality.

pub mod configuration_operations;
pub mod context_integration;
pub mod core;
pub mod events;
pub mod persistence;
pub mod plugin_registration;
pub mod validation;

// Re-export main types and functions
pub use core::ConfigManager;

pub use events::ConfigEvent;

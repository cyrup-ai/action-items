//! Plugin context module providing service access and event handling using modern event-driven
//! architecture

pub mod core;
pub mod events;
pub mod services;

// Re-export main types for convenience
pub use core::*;

pub use events::*;
pub use services::*;

// Re-export from plugin_services for convenience
// pub use crate::plugin_services::{PluginCache, StorageDirectory};

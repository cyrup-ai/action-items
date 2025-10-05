//! Plugin context module providing service access and event handling

pub mod core;
pub mod events;
pub mod services;

// Re-export main types for convenience
pub use core::*;

pub use events::*;
pub use services::*;

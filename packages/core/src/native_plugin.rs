//! Backward compatibility wrapper for native_plugin
//!
//! This module re-exports the new modular native plugin functionality
//! for backward compatibility.

// Re-export all native plugin functionality from the new location
pub use crate::plugins::native::*;

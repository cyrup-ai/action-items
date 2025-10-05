//! Backward compatibility wrapper for native_plugin_wrapper
//!
//! This module re-exports the new modular native plugin wrapper functionality
//! for backward compatibility.

// Re-export all native plugin wrapper functionality from the new location
pub use crate::plugins::native::wrapper::*;

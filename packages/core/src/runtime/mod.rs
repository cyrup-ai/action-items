//! Deno runtime integration and JavaScript execution layer
//!
//! This module provides the core runtime functionality for executing JavaScript/TypeScript
//! plugins using the Deno runtime. It includes isolated operations, plugin wrappers,
//! and the complete Deno integration layer.

pub use isolated_ops::*;
pub use plugin_wrapper::*;

pub mod deno;
pub mod isolated_ops;
pub mod plugin_wrapper;

// Re-export key Deno runtime functionality
pub use deno::*;

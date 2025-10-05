//! Raycast compatibility layer
//!
//! Zero-allocation Raycast extension adapter with blazing-fast modular organization.
//! This module provides complete compatibility with Raycast extensions while maintaining
//! high performance and clean separation of concerns.

// Re-export adapter functionality (maintains backward compatibility)
pub use adapter::*;
// Re-export all Raycast-specific functionality
pub use discovery::*;
pub use loader::*;
pub use plugin::*;
pub use wrapper::*;

// Module declarations
pub mod adapter;
pub mod discovery;
pub mod loader;
pub mod plugin;
pub mod wrapper;

//! Plugin Management Systems
//!
//! Comprehensive plugin lifecycle management, authentication, and capability handling.
//! All systems are designed for zero-allocation performance with blazing-fast operations.

pub mod authentication;
pub mod capability_index;
pub mod lifecycle;
pub mod permissions;
pub mod registration;

// Re-export key types
pub use capability_index::PluginCapabilityIndex;

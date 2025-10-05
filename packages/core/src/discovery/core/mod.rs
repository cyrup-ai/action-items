//! Core plugin discovery functionality
//!
//! This module provides the main discovery orchestration, scanning, and wrapper creation
//! capabilities for finding and loading plugins from various sources including WASM files,
//! native dynamic libraries, and Rust plugin projects.

pub mod build_management;
pub mod detection;
pub mod logging;
pub mod orchestration;
pub mod scanning;
pub mod types;
pub mod wrapper_creation;

// Re-export main types and functions for backward compatibility
pub use logging::log_loaded_plugins;
pub use orchestration::discover_plugin_wrappers;
pub use types::{DiscoveredPlugin, DiscoveryConfig};

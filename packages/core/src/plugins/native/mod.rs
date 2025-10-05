//! Native plugin support
//!
//! Zero-allocation native plugin system with blazing-fast execution and direct system integration.

pub mod bridge_integration;
pub mod wrapper;

pub use bridge_integration::create_native_plugin_context_with_bridge;
pub use wrapper::*;

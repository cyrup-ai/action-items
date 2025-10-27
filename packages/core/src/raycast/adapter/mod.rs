//! Raycast Adapter - Modular Architecture
//!
//! Zero-allocation Raycast extension adapter with blazing-fast modular organization.
//! Converts Raycast extensions to our plugin interface format with full compatibility.

// Public API re-exports
pub use api_adapter::*;
pub use configuration::*;
pub use conversion::*;
pub use host_functions::{
    HostFunction, HostFunctionRegistry, get_host_function_registry as create_host_functions,
};
pub use implementation::RaycastAdapter;

// Import the modular implementation
mod api_adapter;
mod configuration;
mod conversion;
mod host_functions;
mod implementation;

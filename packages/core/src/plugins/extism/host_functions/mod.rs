//! Extism host function modules
//!
//! Modular organization of host functions for Extism WASM plugins.

pub mod cache;
pub mod clipboard;
pub mod core;
pub mod http;
pub mod notification;
pub mod storage;

// Re-export core types and factory function
pub use core::{ExtismHostUserData, create_host_functions};

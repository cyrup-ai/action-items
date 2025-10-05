//! Plugin discovery and dynamic loading system
//!
//! This module provides comprehensive plugin discovery capabilities, including
//! filesystem-based discovery, dynamic loading, and adapter patterns for
//! different plugin architectures.

pub use core::*;

pub use adapter::*;
pub use loader::*;

pub mod adapter;
pub mod core;
pub mod loader;

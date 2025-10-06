//! FontAwesome icon system module
//!
//! Provides a comprehensive FontAwesome icon system with Unicode character mappings
//! for zero-allocation icon rendering, intelligent type detection, and semantic coloring.
//!
//! The module has been decomposed into logical submodules:
//! - `mappings` - Icon mappings, color configurations, and size settings
//! - `detection` - Icon type detection utilities for intelligent inference
//! - `fallback` - Fallback icon system for graceful degradation
//! - `main` - Main FontAwesome struct and public API

pub mod detection;
pub mod fallback;
pub mod main;
pub mod mappings;

// Re-export all public types and functions
pub use detection::IconDetection;
pub use fallback::IconFallback;
pub use main::FontAwesome;
pub use mappings::{ColorMappings, IconMappings, SizeConfigs};

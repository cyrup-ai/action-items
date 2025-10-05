//! Typography module
//!
//! This module provides a comprehensive typography system for consistent text styling
//! across the application. It includes font management, text style configurations,
//! builder patterns, and helper functions for creating text bundles.
//!
//! The module has been decomposed into logical submodules:
//! - `types` - Core data structures and text style configurations
//! - `scale` - Typography scale resource and font management
//! - `builders` - Builder patterns for text styles and shadows
//! - `extensions` - Extension traits for convenient styling
//! - `bundles` - Helper functions for creating complete text bundles

pub mod builders;
pub mod bundles;
pub mod extensions;
pub mod scale;
pub mod types;

// Re-export all public types and functions
// Note: Typography builders will be used when text styling system is implemented
pub use bundles::TextBundleBuilder;
// Note: TextStyleExt will be used when typography extensions are implemented
pub use scale::TypographyScale;
// Note: Typography types will be used when text styling system is implemented

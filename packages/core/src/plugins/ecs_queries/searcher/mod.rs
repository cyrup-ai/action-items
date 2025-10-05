//! ECS-based plugin search functionality module

pub mod core;
pub mod search_strategies;
pub mod search_types;

// Re-export main types for convenience
pub use core::*;

pub use search_strategies::*;
pub use search_types::*;

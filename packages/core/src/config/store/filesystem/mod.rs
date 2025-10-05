//! File system-based configuration store module

pub mod backup_restore;
pub mod change_tracking;
pub mod core;
pub mod file_operations;
pub mod serialization;

// Re-export main types for convenience
pub use core::*;

pub use backup_restore::*;
pub use change_tracking::*;
pub use file_operations::*;
pub use serialization::*;

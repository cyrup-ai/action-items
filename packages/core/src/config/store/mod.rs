//! Configuration Store Trait and Implementations
//!
//! Provides pluggable configuration persistence backends for the plugin system.
//! Supports JSON and TOML storage with validation and change tracking.
//!
//! The module has been decomposed into logical submodules:
//! - `types` - Core data structures and enums for configuration storage
//! - `trait_definition` - ConfigStore trait definition with async methods
//! - `filesystem` - File system-based configuration store implementation
//! - `memory` - In-memory configuration store for testing

pub mod filesystem;
pub mod memory;
pub mod trait_definition;
pub mod types;

// Re-export all public types and functions
pub use filesystem::FileSystemConfigStore;
pub use memory::MemoryConfigStore;
pub use trait_definition::ConfigStore;
pub use types::{ChangeType, ConfigChange, ConfigValue, PluginConfig, StorageFormat};

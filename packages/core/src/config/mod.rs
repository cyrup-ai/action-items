//! Plugin Configuration Management
//!
//! Provides comprehensive configuration persistence, validation, and management
//! for all plugin types (Native, WASM, Raycast).

pub mod manager;
pub mod store;
pub mod validation;

pub use action_items_common::AppDirectories;
pub use manager::{ConfigEvent, ConfigManager};
pub use store::{
    ChangeType, ConfigChange, ConfigStore, ConfigValue, FileSystemConfigStore, MemoryConfigStore,
    PluginConfig, StorageFormat,
};
pub use validation::{ValidationEngine, ValidationError, ValidationResult};

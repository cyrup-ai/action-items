//! Public API and coordination for ECS query executor

pub mod context;
pub mod monitoring;
pub mod native;
pub mod scheduler;
pub mod types;
pub mod wasm;

// Re-export main components for public API
pub use scheduler::PluginExecutor;
pub use types::{
    ExecutionRequest, ExecutionResponse, ExecutionResult, PERFECT_HASH_COMMANDS,
    PERFECT_HASH_TABLE_SIZE, PluginCapability, is_known_command, perfect_hash_command,
};

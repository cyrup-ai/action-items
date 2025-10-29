//! ECS Cache Service - Bevy ECS wrapper around goldylox high-performance cache
//!
//! Provides multi-tier caching with TTL, LRU eviction, and cache warming
//! specifically designed for the Action Items launcher architecture.
//!
//! ## Architecture
//!
//! This service uses the shared Tokio runtime provided by bevy-tokio-tasks plugin
//! for cache initialization, as goldylox requires a Tokio runtime context during
//! initialization. Once initialized, cache operations work fine in Bevy's AsyncComputeTaskPool.
//!
//! **IMPORTANT**: TokioTasksPlugin MUST be added to the Bevy app before EcsCachePlugin.

pub mod components;
pub mod events;
pub mod plugin;
pub mod resources;
pub mod systems;

// Re-export the main plugin
pub use components::*;
// Re-export key types for external use
pub use events::*;
pub use plugin::EcsCachePlugin;
pub use resources::*;

//! ECS Cache Service - Bevy ECS wrapper around goldylox high-performance cache
//!
//! Provides multi-tier caching with TTL, LRU eviction, and cache warming
//! specifically designed for the Action Items launcher architecture.

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

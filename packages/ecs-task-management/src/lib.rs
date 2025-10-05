//! Enterprise-grade async task management with Bevy ECS integration
//!
//! Follows official Bevy async_compute.rs patterns for task management with
//! CommandQueue-based world modifications and proper ECS integration.

pub mod components;
pub mod events;
pub mod plugin;
pub mod systems;
pub mod traits;

// Re-export all public items
pub use components::*;
pub use events::*;
pub use plugin::*;
pub use systems::*;
pub use traits::*;

//! Generic performance monitoring system for Bevy applications
//!
//! Zero-allocation performance tracking with virtualization support.
//! This system is framework-agnostic and works with any UI architecture.
//!
//! ## Architecture
//!
//! - **Resources**: [`PerformanceMetrics`], [`PerformanceManager`]
//! - **Components**: [`VirtualizedEntity`]
//! - **Events**: [`PerformanceWarning`]
//! - **Systems**: Frame time tracking, entity virtualization, memory optimization
//! - **Plugin**: [`PerformancePlugin`] registers complete service
//!
//! ## Usage
//!
//! ```rust
//! use bevy::prelude::*;
//! use action_items_ecs_ui::performance::*;
//!
//! // Add plugin to app
//! fn setup_app(app: &mut App) {
//!     app.add_plugins(PerformancePlugin);
//! }
//!
//! // Access metrics
//! fn check_performance(metrics: Res<PerformanceMetrics>) {
//!     println!("FPS: {:.1}", metrics.fps());
//! }
//!
//! // Mark entities for virtualization
//! fn spawn_optimized(mut commands: Commands) {
//!     commands.spawn((
//!         SpriteBundle::default(),
//!         VirtualizedEntity::default(),
//!     ));
//! }
//! ```

pub mod components;
pub mod resources;
pub mod events;
pub mod systems;
pub mod plugin;

// Re-export main types
pub use components::VirtualizedEntity;
pub use resources::{PerformanceMetrics, PerformanceManager};
pub use events::{PerformanceWarning, WarningLevel};
pub use systems::*;
pub use plugin::PerformancePlugin;

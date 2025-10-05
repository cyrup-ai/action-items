//! Responsive layout system for viewport-based UI sizing
//!
//! This module provides components, systems, and utilities for creating
//! responsive user interfaces that adapt to different screen sizes.
//!
//! # Components
//! - [`ViewportResponsiveContainer`] - Viewport-responsive container sizing (Vw/Vh units)
//! - [`ContentConstraints`] - Content overflow management for scrollable areas
//! - [`TextTruncation`] - Text truncation to prevent horizontal expansion
//!
//! # Plugin
//! - [`ResponsivePlugin`] - Bevy plugin for responsive layout system initialization
//!
//! # Systems
//! - [`update_viewport_responsive_container_system`] - Syncs ViewportResponsiveContainer to Node
//! - [`text_truncation_system`] - Applies text truncation based on TextTruncation component
//!
//! # Example
//! ```rust
//! use bevy::prelude::*;
//! use action_items_ecs_ui::responsive::{ResponsivePlugin, ViewportResponsiveContainer};
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn((
//!         Node {
//!             ..ViewportResponsiveContainer::default().to_node_style()
//!         },
//!         ViewportResponsiveContainer::default(),
//!     ));
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(ResponsivePlugin)
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//! ```

pub mod components;
pub mod plugin;
pub mod systems;

// Re-export main types for convenience
pub use components::{ContentConstraints, TextTruncation, TruncationState, ViewportResponsiveContainer};
pub use plugin::ResponsivePlugin;
pub use systems::*;

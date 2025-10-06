//! Gradient system for theme-based UI gradients with animation support
//!
//! This module provides components, resources, and utilities for managing
//! gradients in interactive UI elements.
//!
//! # Components
//! - [`GradientComponent`] - Full-featured gradient with theme integration
//! - [`InteractiveGradient`] - Simple color-based gradient for basic interactions
//!
//! # Resources
//! - [`GradientTheme`] - Theme resource with gradient presets
//!
//! # Plugin
//! - [`GradientPlugin`] - Bevy plugin for gradient system initialization
//!
//! # Example
//! ```rust
//! use bevy::prelude::*;
//! use action_items_ecs_ui::gradients::{GradientPlugin, GradientComponent};
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn((
//!         NodeBundle::default(),
//!         GradientComponent::list_item(),
//!     ));
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(GradientPlugin)
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//! ```

pub mod components;
pub mod interactive;
pub mod plugin;
pub mod states;
pub mod systems;
pub mod theme;

// Re-export main types for convenience
pub use components::GradientComponent;
pub use interactive::{InteractiveGradient, InteractiveGradientTransition};
pub use plugin::GradientPlugin;
pub use states::{GradientComponentType, GradientInteractionState};
pub use systems::*;
pub use theme::GradientTheme;

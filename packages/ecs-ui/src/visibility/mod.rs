//! Generic visibility animation system for UI components
//!
//! Complete Bevy ECS system providing visibility animations with zero-allocation performance.
//! This system is framework-agnostic and can be used with any UI architecture.
//!
//! ## Architecture
//!
//! - **Components**: [`WindowAnimation`], [`AnimationState`], [`EasingFunction`], [`UiComponentTarget`]
//! - **Events**: [`UiVisibilityEvent`], [`UiAnimationCompleteEvent`]
//! - **Systems**: Generic animation logic (queries `UiComponentTarget` component)
//! - **Plugin**: [`VisibilityPlugin`] registers complete service
//!
//! ## Usage
//!
//! ```rust
//! use bevy::prelude::*;
//! use action_items_ecs_ui::visibility::*;
//!
//! // Add plugin to app
//! fn setup_app(app: &mut App) {
//!     app.add_plugins(VisibilityPlugin);
//! }
//!
//! // Spawn entity with both app-specific and generic markers
//! fn spawn_dialog(mut commands: Commands) {
//!     commands.spawn((
//!         MyDialogContainer,  // App-specific marker
//!         UiComponentTarget::Dialog,  // Generic marker
//!         NodeBundle::default(),
//!     ));
//! }
//!
//! // Send visibility event
//! fn toggle_dialog(mut events: EventWriter<UiVisibilityEvent>) {
//!     events.send(UiVisibilityEvent::animated(
//!         true,
//!         Duration::from_millis(200),
//!         UiComponentTarget::Dialog,
//!     ));
//! }
//! ```

pub mod components;
pub mod events;
pub mod systems;
pub mod targets;
pub mod plugin;

// Re-export all public types
pub use components::{
    AnimationLoop, AnimationPlayState, AnimationSequence, AnimationState, BezierEasing,
    EasingFunction, WindowAnimation,
};
pub use events::{UiAnimationCompleteEvent, UiVisibilityAnimation, UiVisibilityEvent};
pub use plugin::VisibilityPlugin;
pub use systems::{animate_window_visibility_system, handle_ui_visibility_events};
pub use targets::{UiComponentTarget, UiVisibilityAnimationType};

//! Animation system for smooth UI transitions
//!
//! Provides easing functions, animation state management, and UI-specific
//! animation components for window and list animations.

pub mod easing;
pub mod state;
pub mod window;
pub mod plugin;

// Re-export all public types
pub use easing::{EasingFunction, BezierEasing};
pub use state::{AnimationState, AnimationSequence, AnimationPlayState, AnimationLoop};
pub use window::{WindowAnimation, ListAnimation, ItemAnimation, lerp};
pub use plugin::AnimationPlugin;

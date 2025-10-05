//! Visibility events for UI component show/hide animations

use std::time::Duration;
use bevy::prelude::*;

use super::targets::{UiComponentTarget, UiVisibilityAnimationType};

/// Event sent when visibility animation completes for specific component types
#[derive(Event, Debug, Clone)]
pub struct UiAnimationCompleteEvent {
    /// Component that completed animation
    pub target: UiComponentTarget,
    /// Whether the animation was a show (true) or hide (false) operation
    pub was_show: bool,
}

/// Event for controlling UI component visibility with optional animation
///
/// ## Usage
///
/// ```rust
/// // Immediate visibility change
/// events.send(UiVisibilityEvent::immediate(true, UiComponentTarget::Dialog));
///
/// // Animated transition
/// events.send(UiVisibilityEvent::animated(
///     false,
///     Duration::from_millis(300),
///     UiComponentTarget::Panel,
/// ));
/// ```
#[derive(Event, Debug, Clone)]
pub struct UiVisibilityEvent {
    /// Target visibility state
    pub visible: bool,
    /// Animation configuration (None for immediate change)
    pub animation: Option<UiVisibilityAnimation>,
    /// Target component type(s) for this animation
    pub target: UiComponentTarget,
}

/// Animation configuration for UI visibility transitions
#[derive(Debug, Clone)]
pub struct UiVisibilityAnimation {
    /// Animation duration
    pub duration: Duration,
    /// Animation type
    pub animation_type: UiVisibilityAnimationType,
}

impl UiVisibilityEvent {
    /// Create immediate visibility change event for specific component
    #[inline]
    pub const fn immediate(visible: bool, target: UiComponentTarget) -> Self {
        Self {
            visible,
            animation: None,
            target,
        }
    }

    /// Create animated visibility change event with fade and scale for specific component
    #[inline]
    pub fn animated(visible: bool, duration: Duration, target: UiComponentTarget) -> Self {
        Self {
            visible,
            animation: Some(UiVisibilityAnimation {
                duration,
                animation_type: UiVisibilityAnimationType::FadeScale,
            }),
            target,
        }
    }

    /// Create animated visibility change event with custom animation type for specific component
    #[inline]
    pub fn with_animation(
        visible: bool,
        duration: Duration,
        animation_type: UiVisibilityAnimationType,
        target: UiComponentTarget,
    ) -> Self {
        Self {
            visible,
            animation: Some(UiVisibilityAnimation {
                duration,
                animation_type,
            }),
            target,
        }
    }
}

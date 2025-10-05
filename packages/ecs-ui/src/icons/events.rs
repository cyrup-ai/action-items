//! Icon event system for extraction, state changes, and animations

use bevy::prelude::*;
use std::path::PathBuf;

use super::types::IconSize;
use super::components::IconInteractionState;

/// Request icon extraction from file path
///
/// Triggers async extraction task that reads platform-specific icon data:
/// - macOS: .icns, .app bundles
/// - Windows: .ico, .exe embedded icons
/// - Linux: .desktop files, theme icons
///
/// ## Async Workflow
/// 1. Event sent with path and size
/// 2. Background task spawned
/// 3. Platform-specific extraction performed
/// 4. Result returned via IconExtractionResult
///
/// ## Example
/// ```rust
/// events.send(IconExtractionRequest::new(
///     "app::Calculator".to_string(),
///     PathBuf::from("/Applications/Calculator.app"),
///     IconSize::Medium,
/// ));
/// ```
#[derive(Event, Clone, Debug)]
pub struct IconExtractionRequest {
    /// Unique identifier for this icon (used for caching)
    pub id: String,
    /// File path to extract icon from
    pub path: PathBuf,
    /// Requested icon size
    pub size: IconSize,
}

impl IconExtractionRequest {
    /// Create new extraction request
    pub fn new(id: String, path: PathBuf, size: IconSize) -> Self {
        Self { id, path, size }
    }
}

/// Result of icon extraction operation
///
/// Sent when async extraction completes successfully.
/// Contains raw RGBA image data ready for GPU upload.
#[derive(Event, Debug)]
pub struct IconExtractionResult {
    /// Icon identifier matching the request
    pub id: String,
    /// Raw RGBA image data (4 bytes per pixel)
    pub icon_data: Vec<u8>,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
}

impl IconExtractionResult {
    /// Create new extraction result
    pub fn new(id: String, icon_data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            id,
            icon_data,
            width,
            height,
        }
    }
}

/// Icon color change event
///
/// Requests color change for an icon entity, optionally animated.
/// Used for theming, hover effects, and state changes.
///
/// ## Example
/// ```rust
/// // Immediate color change
/// events.send(IconColorChangeEvent::immediate(
///     entity,
///     Color::srgb(1.0, 0.5, 0.0),
/// ));
///
/// // Animated color transition
/// events.send(IconColorChangeEvent::animated(
///     entity,
///     Color::srgb(0.0, 0.8, 1.0),
/// ));
/// ```
#[derive(Event, Clone, Debug)]
pub struct IconColorChangeEvent {
    /// Target entity to change color
    pub entity: Entity,
    /// New color to apply
    pub new_color: Color,
    /// Whether to animate the transition
    pub animated: bool,
}

impl IconColorChangeEvent {
    /// Create immediate color change
    pub fn immediate(entity: Entity, new_color: Color) -> Self {
        Self {
            entity,
            new_color,
            animated: false,
        }
    }

    /// Create animated color change
    pub fn animated(entity: Entity, new_color: Color) -> Self {
        Self {
            entity,
            new_color,
            animated: true,
        }
    }
}

/// Icon size change event
///
/// Requests size change for an icon entity, optionally animated.
/// Useful for zoom effects, responsive layouts, and state feedback.
#[derive(Event, Clone, Debug)]
pub struct IconSizeChangeEvent {
    /// Target entity to change size
    pub entity: Entity,
    /// New size to apply
    pub new_size: IconSize,
    /// Whether to animate the transition
    pub animated: bool,
}

impl IconSizeChangeEvent {
    /// Create immediate size change
    pub fn immediate(entity: Entity, new_size: IconSize) -> Self {
        Self {
            entity,
            new_size,
            animated: false,
        }
    }

    /// Create animated size change
    pub fn animated(entity: Entity, new_size: IconSize) -> Self {
        Self {
            entity,
            new_size,
            animated: true,
        }
    }
}

/// Icon state change event
///
/// Notifies systems of icon interaction state changes.
/// Enables reactive styling based on hover/selected/pressed states.
///
/// ## Example
/// ```rust
/// events.send(IconStateChangeEvent::new(
///     entity,
///     IconInteractionState::Hover,
/// ));
/// ```
#[derive(Event, Clone, Debug)]
pub struct IconStateChangeEvent {
    /// Entity that changed state
    pub entity: Entity,
    /// New interaction state
    pub new_state: IconInteractionState,
}

impl IconStateChangeEvent {
    /// Create new state change event
    pub fn new(entity: Entity, new_state: IconInteractionState) -> Self {
        Self { entity, new_state }
    }
}

/// Icon animation type identifier
///
/// Distinguishes between different animation types for targeted handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconAnimationType {
    /// Color transition animation
    ColorTransition,
    /// Size transition animation
    SizeTransition,
    /// State transition animation
    StateTransition,
}

/// Icon animation complete event
///
/// Sent when icon animation finishes, enabling:
/// - Chained animations (sequential effects)
/// - Lifecycle management (cleanup after animation)
/// - State synchronization (update UI after transition)
///
/// ## Example
/// ```rust
/// // Listen for completion
/// fn handle_completion(mut events: EventReader<IconAnimationCompleteEvent>) {
///     for event in events.read() {
///         match event.animation_type {
///             IconAnimationType::ColorTransition => {
///                 // Color animation done, start size animation
///             },
///             _ => {},
///         }
///     }
/// }
/// ```
#[derive(Event, Clone, Debug)]
pub struct IconAnimationCompleteEvent {
    /// Entity that completed animation
    pub entity: Entity,
    /// Type of animation that completed
    pub animation_type: IconAnimationType,
}

impl IconAnimationCompleteEvent {
    /// Create new animation complete event
    pub fn new(entity: Entity, animation_type: IconAnimationType) -> Self {
        Self {
            entity,
            animation_type,
        }
    }

    /// Create color transition complete event
    pub fn color_transition(entity: Entity) -> Self {
        Self::new(entity, IconAnimationType::ColorTransition)
    }

    /// Create size transition complete event
    pub fn size_transition(entity: Entity) -> Self {
        Self::new(entity, IconAnimationType::SizeTransition)
    }

    /// Create state transition complete event
    pub fn state_transition(entity: Entity) -> Self {
        Self::new(entity, IconAnimationType::StateTransition)
    }
}

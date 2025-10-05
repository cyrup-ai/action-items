//! Generic component targeting for visibility animations

use bevy::prelude::Component;

/// UI component types that can be targeted for visibility animations
///
/// Generic names support any UI architecture (launcher, settings, etc.)
///
/// This serves dual purpose:
/// 1. As a field in UiVisibilityEvent to specify target
/// 2. As a Component attached to entities for generic system queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum UiComponentTarget {
    /// Primary container (e.g., main window, launcher)
    PrimaryContainer,
    /// Dialog surfaces (e.g., modals, preferences)
    Dialog,
    /// Panel components (e.g., settings, configuration)
    Panel,
    /// Secondary panels (e.g., status indicators, info displays)
    SecondaryPanel,
    /// All component types (use sparingly for global animations)
    All,
}

/// Types of visibility animations supported
#[derive(Debug, Clone, Copy)]
pub enum UiVisibilityAnimationType {
    /// Fade in/out with scaling
    FadeScale,
    /// Fade in/out only
    Fade,
    /// Scale in/out only
    Scale,
}

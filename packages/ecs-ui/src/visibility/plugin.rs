//! Visibility plugin for complete generic visibility animation system

use bevy::prelude::*;

use super::events::{UiAnimationCompleteEvent, UiVisibilityEvent};
use super::systems::{animate_window_visibility_system, handle_ui_visibility_events};

/// Plugin that provides complete generic visibility animation system
///
/// Registers events and systems for visibility animations.
/// Provides generic animation logic with no app-specific dependencies.
///
/// Add this plugin to enable visibility event system:
/// ```rust
/// app.add_plugins(VisibilityPlugin);
/// ```
#[derive(Debug, Default, Clone)]
pub struct VisibilityPlugin;

impl Plugin for VisibilityPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register events
            .add_event::<UiVisibilityEvent>()
            .add_event::<UiAnimationCompleteEvent>()
            // Register generic systems
            .add_systems(
                Update,
                (
                    handle_ui_visibility_events,
                    animate_window_visibility_system,
                ),
            );
    }
}

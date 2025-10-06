//! Icon system plugin for Bevy integration

use bevy::prelude::*;

use super::{
    // Resources (from cache.rs, theme.rs, fontawesome/mod.rs)
    IconCache,
    IconTheme,
    FontAwesome,
    // Events (from events.rs)
    IconExtractionRequest,
    IconExtractionResult,
    IconColorChangeEvent,
    IconSizeChangeEvent,
    IconStateChangeEvent,
    IconAnimationCompleteEvent,
    // Systems from extraction module (re-exported by extraction/mod.rs)
    process_icon_extraction_requests,
    poll_icon_extraction_tasks,
    process_icon_extraction_results,
    // Systems from systems.rs
    apply_icon_system,
    interactive_icon_system,
    animate_icon_transitions_system,
    handle_icon_color_change_events,
    handle_icon_size_change_events,
    optimize_icon_performance_system,
    validate_icon_cache_system,
};

/// Complete interactive icon system plugin
///
/// Provides full icon functionality:
/// - Icon types and theming (IconSize, IconType, IconTheme)
/// - FontAwesome icon rendering system
/// - Interactive icon components with state tracking
/// - Smooth animations and transitions
/// - Async icon extraction from files (macOS/Windows/Linux)
/// - Event-driven color/size changes
/// - Performance optimization and caching
///
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use action_items_ecs_ui::icons::IconPlugin;
///
/// App::new()
///     .add_plugins(IconPlugin)
///     .run();
/// ```
#[derive(Debug, Default, Clone)]
pub struct IconPlugin;

impl Plugin for IconPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources - icon cache, theme, and FontAwesome
            .init_resource::<IconCache>()
            .init_resource::<IconTheme>()
            .init_resource::<FontAwesome>()

            // Events - extraction, interactive changes, animations
            .add_event::<IconExtractionRequest>()
            .add_event::<IconExtractionResult>()
            .add_event::<IconColorChangeEvent>()
            .add_event::<IconSizeChangeEvent>()
            .add_event::<IconStateChangeEvent>()
            .add_event::<IconAnimationCompleteEvent>()

            // Core icon systems (run every frame)
            .add_systems(
                Update,
                (
                    apply_icon_system,
                    interactive_icon_system,
                    animate_icon_transitions_system,
                ),
            )

            // Icon extraction systems (async)
            .add_systems(
                Update,
                (
                    process_icon_extraction_requests,
                    poll_icon_extraction_tasks,
                    process_icon_extraction_results,
                ),
            )

            // Event handler systems
            .add_systems(
                Update,
                (
                    handle_icon_color_change_events,
                    handle_icon_size_change_events,
                ),
            )

            // Performance systems (run when cache changes)
            .add_systems(
                Update,
                (
                    optimize_icon_performance_system,
                    validate_icon_cache_system,
                )
                    .run_if(resource_changed::<IconCache>),
            );
    }
}

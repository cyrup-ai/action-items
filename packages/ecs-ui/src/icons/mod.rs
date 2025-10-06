//! Core icon type system for Bevy applications

pub mod types;
pub mod theme;
pub mod cache;
pub mod components;
pub mod events;
pub mod fontawesome;
pub mod extraction;
pub mod systems;
pub mod plugin;

// Re-export public types
pub use types::{IconSize, IconType};
pub use theme::{ThemeColors, IconTheme};
pub use cache::IconCache;
pub use components::{IconInteractionState, IconComponent, IconAnimation};
pub use events::{
    IconExtractionRequest,
    IconExtractionResult,
    IconColorChangeEvent,
    IconSizeChangeEvent,
    IconStateChangeEvent,
    IconAnimationCompleteEvent,
    IconAnimationType,
};
pub use fontawesome::{FontAwesome, IconDetection, IconFallback};
pub use extraction::{
    IconExtractionInProgress,
    process_icon_extraction_requests,
    poll_icon_extraction_tasks,
    process_icon_extraction_results,
};
pub use systems::{
    apply_icon_system,
    interactive_icon_system,
    animate_icon_transitions_system,
    handle_icon_color_change_events,
    handle_icon_size_change_events,
    optimize_icon_performance_system,
    validate_icon_cache_system,
};
pub use plugin::IconPlugin;

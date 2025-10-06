//! Launcher icon system - app-specific helpers + ecs-ui infrastructure

// App-specific modules
pub mod extraction;  // Uses LauncherIconCache, launcher path formats
pub mod privacy_icons;  // Privacy indicator UI (launcher-specific)
pub mod types;  // LauncherIconCache wrapper
pub mod utils;  // ActionItem/SearchResult → IconType mapping

// Re-export ecs-ui infrastructure
pub use action_items_ecs_ui::icons::{
    // Core types
    IconType, IconSize, IconTheme, ThemeColors,
    // FontAwesome system
    FontAwesome, IconDetection, IconFallback,
    // Events
    IconExtractionRequest, IconExtractionResult,
    IconColorChangeEvent, IconSizeChangeEvent,
    // Components (if needed for custom icon UI)
    IconComponent, IconInteractionState, IconAnimation,
};

// Re-export app-specific types
pub use types::LauncherIconCache;

// Re-export app-specific helpers
pub use utils::{get_icon_for_result, get_icon_for_search_result};
pub use privacy_icons::{
    PrivacyContainerStyle, PrivacyIconTheme, PrivacyIcons,
    spawn_privacy_indicators_ui,
};

// Re-export extraction utilities
pub use extraction::{
    request_icon_extraction,
    process_icon_extraction_requests,
    poll_icon_extraction_tasks,
    process_icon_extraction_results,
};

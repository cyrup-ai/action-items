//! Launcher icon system - app-specific helpers + ecs-ui infrastructure

// App-specific modules
pub mod privacy_icons;  // Privacy indicator UI (launcher-specific)
pub mod types;          // LauncherIconCache wrapper
pub mod utils;          // ActionItem/SearchResult → IconType mapping

// Re-export ecs-ui infrastructure (IconPlugin provides these systems)
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
pub use types::GenericIconFallbacks;

// Re-export app-specific helpers
pub use utils::{get_icon_for_result, get_icon_for_search_result};
pub use privacy_icons::{
    PrivacyContainerStyle, PrivacyIconTheme, PrivacyIcons,
    spawn_privacy_indicators_ui,
};

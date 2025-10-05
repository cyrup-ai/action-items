pub mod extraction;
pub mod fontawesome;
pub mod generation;
pub mod privacy_icons; // Re-enabled to fix warnings
pub mod types;
pub mod utils;

// Re-export public types and functions
pub use extraction::{
    process_icon_extraction_requests, process_icon_extraction_results, request_icon_extraction,
};
pub use generation::create_generic_icon;
pub use privacy_icons::{
    PrivacyContainerStyle, PrivacyIconTheme, PrivacyIcons, spawn_privacy_indicators_ui,
};
pub use types::{
    LauncherIconCache, IconExtractionQueue, IconExtractionRequest, IconExtractionResult,
};
// Re-export ecs-ui icon types
pub use action_items_ecs_ui::icons::{IconSize, IconType, ThemeColors, IconTheme};
pub use utils::{get_icon_for_result, get_icon_for_search_result};

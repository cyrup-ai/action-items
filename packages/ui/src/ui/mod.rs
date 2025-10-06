pub mod accessibility;
pub mod ai_menu;
pub mod components;
pub mod icons;
pub mod performance;
pub mod systems;
pub mod typography;

// Re-export public types and functions
pub use ai_menu::{PrivacyConfiguration, PrivacyIndicatorPlugin, PrivacyIndicators};
pub use components::{UiFonts, UiState, set_ui_visibility};
pub use icons::{LauncherIconCache, FontAwesome, IconExtractionRequest, IconExtractionResult};
// Re-export ecs-ui icon types
pub use action_items_ecs_ui::icons::{IconSize, IconType, IconTheme, ThemeColors};
// Note: Visibility types are re-exported from ecs-ui in lib.rs
// Note: Systems are available but not re-exported

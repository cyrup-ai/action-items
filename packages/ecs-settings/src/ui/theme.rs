use bevy::prelude::Color;

// Window colors
pub const SETTINGS_WINDOW_BG: Color = Color::srgba(0.10, 0.10, 0.10, 0.95);
pub const SETTINGS_SIDEBAR_BG: Color = Color::srgba(0.08, 0.08, 0.08, 1.0);
pub const SETTINGS_CONTENT_BG: Color = Color::srgba(0.12, 0.12, 0.12, 1.0);

// Tab colors
pub const TAB_INACTIVE: Color = Color::srgba(0.15, 0.15, 0.15, 1.0);
pub const TAB_ACTIVE: Color = Color::srgba(0.25, 0.50, 0.80, 1.0);
pub const TAB_HOVER: Color = Color::srgba(0.20, 0.20, 0.20, 1.0);

// Text colors
pub const TEXT_PRIMARY: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);
pub const TEXT_SECONDARY: Color = Color::srgba(0.7, 0.7, 0.7, 1.0);
pub const TEXT_DISABLED: Color = Color::srgba(0.4, 0.4, 0.4, 1.0);

// Control colors
pub const CHECKBOX_BG: Color = Color::srgba(0.20, 0.20, 0.20, 1.0);
pub const CHECKBOX_CHECKED: Color = Color::srgba(0.30, 0.60, 0.90, 1.0);
pub const INPUT_BG: Color = Color::srgba(0.15, 0.15, 0.15, 1.0);
pub const INPUT_FOCUS: Color = Color::srgba(0.25, 0.50, 0.80, 1.0);
pub const BUTTON_PRIMARY: Color = Color::srgba(0.30, 0.60, 0.90, 1.0);
pub const BUTTON_SECONDARY: Color = Color::srgba(0.25, 0.25, 0.25, 1.0);
pub const BUTTON_BG: Color = Color::srgba(0.25, 0.50, 0.80, 1.0);

// Extension card colors
pub const CARD_BG: Color = Color::srgba(0.14, 0.14, 0.16, 1.0);
pub const TOGGLE_ON: Color = Color::srgba(0.30, 0.70, 0.50, 1.0);
pub const TOGGLE_OFF: Color = Color::srgba(0.25, 0.25, 0.25, 1.0);

// Sizing
pub const SIDEBAR_WIDTH: f32 = 200.0;
pub const TAB_HEIGHT: f32 = 40.0;
pub const SECTION_SPACING: f32 = 20.0;
pub const LABEL_WIDTH_PCT: f32 = 45.0;
pub const CONTROL_WIDTH_PCT: f32 = 50.0;
pub const CONTROL_OFFSET_PCT: f32 = 48.0;

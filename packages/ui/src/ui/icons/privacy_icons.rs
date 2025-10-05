//! Privacy indicator icon definitions
//!
//! Zero-allocation icon definitions for blazing-fast privacy indicator rendering.

use bevy::prelude::*;

use super::super::ai_menu::privacy_events::IndicatorType;
use super::super::ai_menu::{PrivacyIconButton, PrivacyIndicators};
use crate::ui::components::PrivacyIndicatorPanel;

/// Privacy indicator icon definitions using Unicode symbols for cross-platform compatibility
pub struct PrivacyIcons;

impl PrivacyIcons {
    /// Full control indicator - horizontal line symbol
    pub const FULL_CONTROL: &'static str = "─";

    /// No collection indicator - lock symbol  
    pub const NO_COLLECTION: &'static str = "🔒";

    /// Encrypted indicator - shield symbol
    pub const ENCRYPTED: &'static str = "🛡️";

    /// Info details button - info symbol
    pub const INFO_DETAILS: &'static str = "ℹ️";

    /// Get icon text for the specified privacy indicator type
    #[inline]
    pub fn get_icon_text(indicator_type: &IndicatorType) -> &'static str {
        match indicator_type {
            IndicatorType::FullControl => Self::FULL_CONTROL,
            IndicatorType::NoCollection => Self::NO_COLLECTION,
            IndicatorType::Encrypted => Self::ENCRYPTED,
            IndicatorType::InfoDetails => Self::INFO_DETAILS,
        }
    }
}

/// Privacy icon theme colors for consistent visual design
pub struct PrivacyIconTheme;

impl PrivacyIconTheme {
    /// Active state color (white)
    pub const ACTIVE: Color = Color::srgb(1.0, 1.0, 1.0);

    /// Inactive state color (gray #888888)
    pub const INACTIVE: Color = Color::srgb(0.53, 0.53, 0.53);

    /// Hover state color (light gray)
    pub const HOVER: Color = Color::srgb(0.9, 0.9, 0.9);

    /// Pressed state color (darker gray)
    pub const PRESSED: Color = Color::srgb(0.8, 0.8, 0.8);

    /// Container background (dark theme)
    pub const CONTAINER_BG: Color = Color::srgb(0.16, 0.16, 0.16);

    /// Container border (subtle white)
    pub const CONTAINER_BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.1);

    /// Info button background (slightly lighter than container)
    pub const INFO_BG: Color = Color::srgb(0.20, 0.20, 0.20);
}

/// Privacy indicator container styling constants
pub struct PrivacyContainerStyle;

impl PrivacyContainerStyle {
    /// Horizontal padding in pixels
    pub const HORIZONTAL_PADDING: f32 = 8.0;

    /// Vertical padding in pixels  
    pub const VERTICAL_PADDING: f32 = 6.0;

    /// Corner radius for subtle rounded corners
    pub const CORNER_RADIUS: f32 = 4.0;

    /// Icon size in pixels
    pub const ICON_SIZE: f32 = 16.0;

    /// Spacing between icons
    pub const ICON_SPACING: f32 = 12.0;

    /// Container height (fixed to prevent layout shift)
    pub const CONTAINER_HEIGHT: f32 = 28.0;

    /// Info button border radius
    pub const INFO_BORDER_RADIUS: f32 = 2.0;
}

/// Component for privacy indicator icon rendering with efficient text caching
#[derive(Component, Debug)]
pub struct PrivacyIcon {
    /// Cached icon text for this indicator
    pub icon_text: &'static str,
    /// Icon size for consistent rendering
    pub size: f32,
    /// Current visual state for color determination
    pub is_active: bool,
}

impl PrivacyIcon {
    /// Create new privacy icon for the specified indicator type
    #[inline]
    pub fn new(indicator_type: &IndicatorType) -> Self {
        Self {
            icon_text: PrivacyIcons::get_icon_text(indicator_type),
            size: PrivacyContainerStyle::ICON_SIZE,
            is_active: false,
        }
    }

    /// Update active state and return true if changed
    #[inline]
    pub fn set_active(&mut self, active: bool) -> bool {
        if self.is_active != active {
            self.is_active = active;
            true
        } else {
            false
        }
    }
}

/// System to create privacy indicator UI layout with professional styling
pub fn spawn_privacy_indicators_ui(commands: &mut Commands, font_handle: Handle<Font>) -> Entity {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(PrivacyContainerStyle::CONTAINER_HEIGHT),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(PrivacyContainerStyle::HORIZONTAL_PADDING)),
                margin: UiRect::bottom(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(PrivacyIconTheme::CONTAINER_BG),
            BorderColor(PrivacyIconTheme::CONTAINER_BORDER),
            PrivacyIndicatorPanel, // Add component marker for animation targeting
        ))
        .with_children(|parent| {
            // Full Control indicator
            parent
                .spawn((
                    Text::new(PrivacyIcons::FULL_CONTROL),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: PrivacyContainerStyle::ICON_SIZE,
                        ..default()
                    },
                    TextColor(PrivacyIconTheme::INACTIVE),
                ))
                .insert(PrivacyIcon::new(&IndicatorType::FullControl));

            // No Collection indicator
            parent
                .spawn((
                    Text::new(PrivacyIcons::NO_COLLECTION),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: PrivacyContainerStyle::ICON_SIZE,
                        ..default()
                    },
                    TextColor(PrivacyIconTheme::INACTIVE),
                ))
                .insert(PrivacyIcon::new(&IndicatorType::NoCollection));

            // Encrypted indicator
            parent
                .spawn((
                    Text::new(PrivacyIcons::ENCRYPTED),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: PrivacyContainerStyle::ICON_SIZE,
                        ..default()
                    },
                    TextColor(PrivacyIconTheme::INACTIVE),
                ))
                .insert(PrivacyIcon::new(&IndicatorType::Encrypted));

            // Info Details button (interactive)
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(PrivacyContainerStyle::ICON_SIZE + 4.0),
                        height: Val::Px(PrivacyContainerStyle::ICON_SIZE + 4.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::left(Val::Px(PrivacyContainerStyle::ICON_SPACING)),
                        ..default()
                    },
                    BackgroundColor(PrivacyIconTheme::INFO_BG),
                    BorderRadius::all(Val::Px(PrivacyContainerStyle::INFO_BORDER_RADIUS)),
                    Interaction::None,
                ))
                .with_children(|button_parent| {
                    button_parent.spawn((
                        Text::new(PrivacyIcons::INFO_DETAILS),
                        TextFont {
                            font: font_handle,
                            font_size: PrivacyContainerStyle::ICON_SIZE,
                            ..default()
                        },
                        TextColor(PrivacyIconTheme::ACTIVE),
                    ));
                })
                .insert(PrivacyIconButton::new(IndicatorType::InfoDetails));
        })
        .insert(PrivacyIndicators::default())
        .id()
}

/// Spawn individual privacy indicator icon with proper styling
/// Used for modular creation of privacy indicators per spec requirements in
/// ./spec/tasks/AI_Menu/2_privacy_indicators.md Returns the Entity ID of the spawned icon for
/// further configuration
#[inline]
pub fn spawn_privacy_icon(
    commands: &mut Commands,
    parent_entity: Entity,
    indicator_type: &IndicatorType,
    font_handle: Handle<Font>,
) -> Entity {
    commands
        .spawn((
            Text::new(PrivacyIcons::get_icon_text(indicator_type)),
            TextFont {
                font: font_handle,
                font_size: PrivacyContainerStyle::ICON_SIZE,
                ..default()
            },
            TextColor(PrivacyIconTheme::INACTIVE),
            PrivacyIcon::new(indicator_type),
            ChildOf(parent_entity),
        ))
        .id()
}

/// Spawn interactive privacy info button with hover and click handling
/// Implements expandable info details per spec requirements in
/// ./spec/tasks/AI_Menu/2_privacy_indicators.md Returns the Entity ID of the spawned button for
/// further configuration
#[inline]
pub fn spawn_privacy_info_button(
    commands: &mut Commands,
    parent_entity: Entity,
    font_handle: Handle<Font>,
) -> Entity {
    let button_entity = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(PrivacyContainerStyle::ICON_SIZE + 4.0),
                height: Val::Px(PrivacyContainerStyle::ICON_SIZE + 4.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor(Color::NONE),
            PrivacyIconButton::new(IndicatorType::InfoDetails),
            ChildOf(parent_entity),
        ))
        .id();

    // Spawn the button text as a child
    commands.spawn((
        Text::new(PrivacyIcons::INFO_DETAILS),
        TextFont {
            font: font_handle,
            font_size: PrivacyContainerStyle::ICON_SIZE,
            ..default()
        },
        TextColor(PrivacyIconTheme::ACTIVE),
        ChildOf(button_entity),
    ));

    button_entity
}

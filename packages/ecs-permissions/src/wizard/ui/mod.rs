//! Wizard UI Components
//!
//! Defines UI components for the permission setup wizard using ecs-ui integration.
//! This module provides component definitions and bundles for the wizard interface
//! with responsive layouts, animations, and theming support.

#![allow(dead_code)]

pub mod commands;
pub mod theme;
pub mod observers;
pub mod permission_screens;

pub use commands::*;
pub use theme::*;
pub use permission_screens::*;

use bevy::prelude::*;
use action_items_ecs_ui::prelude::*;

use crate::wizard::{WizardAction};

/// Animation state component for wizard UI elements
#[derive(Component, Debug)]
pub struct WizardAnimationState {
    /// Whether the element is currently animated in
    pub is_animated_in: bool,
    /// Animation progress (0.0 to 1.0)
    pub animation_progress: f32,
    /// Entrance animation speed
    pub entrance_speed: f32,
    /// Exit animation speed
    pub exit_speed: f32,
}

impl Default for WizardAnimationState {
    fn default() -> Self {
        Self {
            is_animated_in: false,
            animation_progress: 0.0,
            entrance_speed: 8.0,
            exit_speed: 6.0,
        }
    }
}

/// Main wizard modal window component with ecs-ui integration
#[derive(Component, Debug)]
pub struct WizardModalWindow {
    /// Main modal layout
    pub layout: UiLayout,
    /// Backdrop layout (full viewport)
    pub backdrop_layout: UiLayout,
    /// Animation state
    pub animation_state: WizardAnimationState,
}

impl Default for WizardModalWindow {
    fn default() -> Self {
        Self {
            layout: UiLayout::window()
                .size((Vw(80.0), Vh(70.0)))
                .pos((Vw(50.0), Vh(50.0)))
                .anchor(Anchor::Center)
                .pack(),
            backdrop_layout: UiLayout::window()
                .size((Vw(100.0), Vh(100.0)))
                .pos((Vw(0.0), Vh(0.0)))
                .pack(),
            animation_state: WizardAnimationState::default(),
        }
    }
}

/// Wizard button type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum WizardButtonType {
    Primary,
    Secondary,
    Cancel,
    Skip,
}

/// Wizard button component
#[derive(Component, Debug)]
pub struct WizardButton {
    /// The action this button performs
    pub action: WizardAction,
    /// The visual style of this button
    pub button_type: WizardButtonType,
}

/// Bundle for creating a complete wizard modal with ecs-ui integration
#[derive(Bundle)]
pub struct WizardModalBundle {
    pub modal: WizardModalWindow,
    pub layout: UiLayout,
    pub hover_state: UiHover,
    pub animation_state: WizardAnimationState,
    pub visibility: Visibility,
    pub name: Name,
}

impl Default for WizardModalBundle {
    fn default() -> Self {
        Self {
            modal: WizardModalWindow::default(),
            layout: UiLayout::window()
                .size((Vw(80.0), Vh(70.0)))
                .pos((Vw(50.0), Vh(50.0)))
                .anchor(Anchor::Center)
                .pack(),
            hover_state: UiHover::new().forward_speed(8.0).backward_speed(4.0),
            animation_state: WizardAnimationState::default(),
            visibility: Visibility::Hidden,
            name: Name::new("WizardModal"),
        }
    }
}

/// Bundle for wizard button entities with ecs-ui integration
#[derive(Bundle)]
pub struct WizardButtonBundle {
    pub button: WizardButton,
    pub layout: UiLayout,
    pub color: UiColor,
    pub hover_state: UiHover,
    pub click_state: UiClicked,
    pub text: Text,
    pub text_size: UiTextSize,
    pub pickable: Pickable,
    pub visibility: Visibility,
    pub name: Name,
}

impl WizardButtonBundle {
    /// Create a new wizard button bundle
    pub fn new(text: &str, action: WizardAction, button_type: WizardButtonType) -> Self {
        let (color, _hover_color) = match button_type {
            WizardButtonType::Primary => (
                UiColor::from(Color::srgba(0.2, 0.6, 0.9, 1.0)),
                UiColor::from(Color::srgba(0.3, 0.7, 1.0, 1.0))
            ),
            WizardButtonType::Secondary => (
                UiColor::from(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                UiColor::from(Color::srgba(0.5, 0.5, 0.5, 1.0))
            ),
            WizardButtonType::Cancel => (
                UiColor::from(Color::srgba(0.8, 0.2, 0.2, 1.0)),
                UiColor::from(Color::srgba(0.9, 0.3, 0.3, 1.0))
            ),
            WizardButtonType::Skip => (
                UiColor::from(Color::srgba(0.6, 0.6, 0.6, 1.0)),
                UiColor::from(Color::srgba(0.7, 0.7, 0.7, 1.0))
            ),
        };
        
        Self {
            button: WizardButton { action, button_type: button_type.clone() },
            layout: UiLayout::window()
                .size((Rl(18.0), Rl(60.0)))
                .pack(),
            color,
            hover_state: UiHover::new().forward_speed(10.0).backward_speed(5.0),
            click_state: UiClicked::new().forward_speed(15.0).backward_speed(8.0),
            text: Text::new(text),
            text_size: UiTextSize::from(Em(1.0)),
            pickable: Pickable::default(),
            visibility: Visibility::Hidden, // Hidden initially
            name: Name::new(format!("WizardButton_{:?}", action)),
        }
    }
}

/// Get Unicode icon for a permission type
///
/// Returns appropriate Unicode symbols for visual identification of permission types.
pub fn get_permission_icon(permission_type: crate::types::PermissionType) -> &'static str {
    use crate::types::PermissionType;
    
    match permission_type {
        PermissionType::Camera => "📷",
        PermissionType::Microphone => "🎤",
        PermissionType::Accessibility => "♿",
        PermissionType::AccessibilityMouse => "🖱️",
        PermissionType::ScreenCapture => "🖥️",
        PermissionType::InputMonitoring => "⌨️",
        PermissionType::FullDiskAccess => "💾",
        PermissionType::WiFi => "📡",
        PermissionType::Bluetooth => "📲",
        PermissionType::Contacts => "👥",
        PermissionType::AddressBook => "📇",
        PermissionType::Calendar => "📅",
        PermissionType::Reminders => "📝",
        PermissionType::Photos => "🖼️",
        PermissionType::PhotosAdd => "📸",
        PermissionType::Location => "📍",
        PermissionType::FileProviderDomain => "📂",
        PermissionType::FileProviderPresence => "📁",
        PermissionType::DesktopFolder => "🗂️",
        PermissionType::DocumentsFolder => "📄",
        PermissionType::DownloadsFolder => "⬇️",
        PermissionType::NetworkVolumes => "🌐",
        PermissionType::RemovableVolumes => "💿",
        PermissionType::UbiquitousFileProvider => "☁️",
        PermissionType::SpeechRecognition => "🗣️",
        PermissionType::MediaLibrary => "🎵",
        PermissionType::AppleEvents => "🍎",
        PermissionType::Siri => "💬",
        PermissionType::Motion => "🏃",
        PermissionType::FaceID => "👤",
        PermissionType::Calls => "📞",
        PermissionType::FocusStatus => "🎯",
        PermissionType::NearbyInteraction => "📡",
        PermissionType::PostEvent => "📮",
        PermissionType::RemoteDesktop => "🖥️",
        PermissionType::DeveloperTools => "🛠️",
        PermissionType::AdminFiles => "🔑",
        PermissionType::WillfulWrite => "✍️",
        PermissionType::All => "🔐",
    }
}

/// Get colored status indicator for permission status
///
/// Returns Unicode symbols with appropriate colors for permission states:
/// - ✅ granted (green)
/// - ❌ denied (red)
/// - ⏳ pending (orange)
pub fn get_permission_status_indicator(status: crate::types::PermissionStatus) -> &'static str {
    use crate::types::PermissionStatus;
    
    match status {
        PermissionStatus::Authorized => "✅",
        PermissionStatus::Denied => "❌",
        PermissionStatus::Restricted => "❌",
        PermissionStatus::NotDetermined => "⏳",
        PermissionStatus::Unknown => "⏳",
    }
}

/// Get color for permission status
///
/// Returns appropriate Color for visual feedback on permission states.
pub fn get_permission_status_color(status: crate::types::PermissionStatus) -> Color {
    use crate::types::PermissionStatus;
    
    match status {
        PermissionStatus::Authorized => Color::srgb(0.0, 0.8, 0.0), // Green
        PermissionStatus::Denied => Color::srgb(0.9, 0.0, 0.0), // Red
        PermissionStatus::Restricted => Color::srgb(0.9, 0.0, 0.0), // Red
        PermissionStatus::NotDetermined => Color::srgb(0.9, 0.6, 0.0), // Orange
        PermissionStatus::Unknown => Color::srgb(0.9, 0.6, 0.0), // Orange
    }
}
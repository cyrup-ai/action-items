//! Wizard Theme System
//!
//! Provides comprehensive theming support for wizard UI with dynamic color schemes,
//! animation speeds, and responsive design parameters.

use bevy::prelude::*;
use action_items_ecs_ui::prelude::*;
use crate::types::PermissionStatus;

/// Main wizard theme resource
#[derive(Resource, Debug, Clone)]
pub struct WizardTheme {
    /// Primary color scheme
    pub primary_color: UiColor,
    /// Secondary color scheme
    pub secondary_color: UiColor,
    /// Success/granted color
    pub success_color: UiColor,
    /// Error/denied color
    pub error_color: UiColor,
    /// Warning/pending color
    pub warning_color: UiColor,
    /// Pending/requesting color
    pub pending_color: UiColor,
    /// Card background color
    pub card_background: UiColor,
    /// Card hover background color
    pub card_hover_background: UiColor,
    /// Modal background color
    pub modal_background: UiColor,
    /// Backdrop overlay color
    pub backdrop_color: UiColor,
    /// Animation speeds
    pub animation_speeds: AnimationSpeeds,
    /// Current theme preset
    pub current_preset: ThemePreset,
}

/// Animation speed configuration
#[derive(Debug, Clone)]
pub struct AnimationSpeeds {
    /// UI element entrance speed
    pub entrance_speed: f32,
    /// UI element exit speed
    pub exit_speed: f32,
    /// Hover animation speed
    pub hover_speed: f32,
    /// Click animation speed
    pub click_speed: f32,
    /// Status change animation speed
    pub status_change_speed: f32,
}

impl Default for AnimationSpeeds {
    fn default() -> Self {
        Self {
            entrance_speed: 8.0,
            exit_speed: 6.0,
            hover_speed: 10.0,
            click_speed: 15.0,
            status_change_speed: 5.0,
        }
    }
}

/// Available theme presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreset {
    /// Dark theme (default)
    Dark,
    /// Light theme
    Light,
    /// High contrast theme
    HighContrast,
    /// System theme (follows OS preference)
    System,
}

impl Default for ThemePreset {
    fn default() -> Self {
        Self::Dark
    }
}

impl Default for WizardTheme {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl WizardTheme {
    /// Create a dark theme
    pub fn dark_theme() -> Self {
        Self {
            primary_color: UiColor::from(Color::srgba(0.2, 0.6, 0.9, 1.0)),
            secondary_color: UiColor::from(Color::srgba(0.4, 0.4, 0.4, 1.0)),
            success_color: UiColor::from(Color::srgba(0.2, 0.8, 0.2, 1.0)),
            error_color: UiColor::from(Color::srgba(0.8, 0.2, 0.2, 1.0)),
            warning_color: UiColor::from(Color::srgba(0.8, 0.6, 0.2, 1.0)),
            pending_color: UiColor::from(Color::srgba(0.2, 0.4, 0.8, 1.0)),
            card_background: UiColor::from(Color::srgba(0.25, 0.25, 0.3, 1.0)),
            card_hover_background: UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
            modal_background: UiColor::from(Color::srgba(0.1, 0.1, 0.15, 0.95)),
            backdrop_color: UiColor::from(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            animation_speeds: AnimationSpeeds::default(),
            current_preset: ThemePreset::Dark,
        }
    }
    
    /// Create a light theme
    pub fn light_theme() -> Self {
        Self {
            primary_color: UiColor::from(Color::srgba(0.1, 0.4, 0.7, 1.0)),
            secondary_color: UiColor::from(Color::srgba(0.6, 0.6, 0.6, 1.0)),
            success_color: UiColor::from(Color::srgba(0.1, 0.6, 0.1, 1.0)),
            error_color: UiColor::from(Color::srgba(0.7, 0.1, 0.1, 1.0)),
            warning_color: UiColor::from(Color::srgba(0.7, 0.5, 0.1, 1.0)),
            pending_color: UiColor::from(Color::srgba(0.1, 0.3, 0.6, 1.0)),
            card_background: UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
            card_hover_background: UiColor::from(Color::srgba(0.85, 0.85, 0.9, 1.0)),
            modal_background: UiColor::from(Color::srgba(0.95, 0.95, 0.98, 0.95)),
            backdrop_color: UiColor::from(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            animation_speeds: AnimationSpeeds::default(),
            current_preset: ThemePreset::Light,
        }
    }
    
    /// Create a high contrast theme
    pub fn high_contrast_theme() -> Self {
        Self {
            primary_color: UiColor::from(Color::srgba(0.0, 0.5, 1.0, 1.0)),
            secondary_color: UiColor::from(Color::srgba(0.5, 0.5, 0.5, 1.0)),
            success_color: UiColor::from(Color::srgba(0.0, 1.0, 0.0, 1.0)),
            error_color: UiColor::from(Color::srgba(1.0, 0.0, 0.0, 1.0)),
            warning_color: UiColor::from(Color::srgba(1.0, 0.8, 0.0, 1.0)),
            pending_color: UiColor::from(Color::srgba(0.0, 0.3, 1.0, 1.0)),
            card_background: UiColor::from(Color::srgba(0.0, 0.0, 0.0, 1.0)),
            card_hover_background: UiColor::from(Color::srgba(0.2, 0.2, 0.2, 1.0)),
            modal_background: UiColor::from(Color::srgba(0.0, 0.0, 0.0, 1.0)),
            backdrop_color: UiColor::from(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            animation_speeds: AnimationSpeeds {
                entrance_speed: 12.0,
                exit_speed: 10.0,
                hover_speed: 15.0,
                click_speed: 20.0,
                status_change_speed: 8.0,
            },
            current_preset: ThemePreset::HighContrast,
        }
    }
    
    /// Switch to a different theme preset
    pub fn switch_preset(&mut self, preset: ThemePreset) {
        *self = match preset {
            ThemePreset::Dark => Self::dark_theme(),
            ThemePreset::Light => Self::light_theme(),
            ThemePreset::HighContrast => Self::high_contrast_theme(),
            ThemePreset::System => {
                // For now, default to dark theme
                // In a real implementation, this would detect system preference
                Self::dark_theme()
            },
        };
        self.current_preset = preset;
    }
}

/// Theme animation state resource
#[derive(Resource, Debug, Default)]
pub struct ThemeAnimationState {
    /// Whether theme is currently transitioning
    pub is_transitioning: bool,
    /// Transition progress (0.0 to 1.0)
    pub transition_progress: f32,
    /// Transition speed
    pub transition_speed: f32,
}

impl ThemeAnimationState {
    /// Start a theme transition
    pub fn start_transition(&mut self) {
        self.is_transitioning = true;
        self.transition_progress = 0.0;
        self.transition_speed = 5.0;
    }
    
    /// Update transition progress
    pub fn update_transition(&mut self, delta_time: f32) {
        if self.is_transitioning {
            self.transition_progress += delta_time * self.transition_speed;
            if self.transition_progress >= 1.0 {
                self.transition_progress = 1.0;
                self.is_transitioning = false;
            }
        }
    }
}

/// Event for theme preset changes
#[derive(Event, Debug, Clone, Copy)]
pub struct ThemePresetChangeEvent {
    /// The new theme preset to switch to
    pub new_preset: ThemePreset,
}

impl ThemePresetChangeEvent {
    /// Create a new theme change event
    pub fn new(preset: ThemePreset) -> Self {
        Self { new_preset: preset }
    }
}

/// System to apply wizard theme to UI elements
pub fn apply_wizard_theme(
    theme: Res<WizardTheme>,
    mut modal_query: Query<&mut UiColor, (With<crate::wizard::WizardRoot>, Without<crate::wizard::PermissionCard>)>,
) {
    for mut color in modal_query.iter_mut() {
        *color = theme.modal_background.clone();
    }
}

/// System to update permission card colors based on theme
pub fn update_permission_card_colors(
    theme: Res<WizardTheme>,
    mut card_query: Query<(&crate::wizard::PermissionCard, &mut UiColor)>,
) {
    if !theme.is_changed() {
        return;
    }
    
    for (card, mut color) in card_query.iter_mut() {
        *color = match card.status {
            PermissionStatus::Authorized => theme.success_color.clone(),
            PermissionStatus::Denied => theme.error_color.clone(),
            PermissionStatus::NotDetermined => theme.warning_color.clone(),
            PermissionStatus::Unknown => theme.card_background.clone(),
            PermissionStatus::Restricted => theme.error_color.clone(),
        };
    }
}

/// System to update animation speeds based on theme
pub fn update_animation_speeds(
    theme: Res<WizardTheme>,
    mut hover_query: Query<&mut UiHover>,
    mut click_query: Query<&mut UiClicked>,
) {
    if !theme.is_changed() {
        return;
    }
    
    for mut hover in hover_query.iter_mut() {
        *hover = hover.clone().forward_speed(theme.animation_speeds.hover_speed)
                     .backward_speed(theme.animation_speeds.hover_speed * 0.5);
    }
    
    for mut click in click_query.iter_mut() {
        *click = click.clone().forward_speed(theme.animation_speeds.click_speed)
                     .backward_speed(theme.animation_speeds.click_speed * 0.5);
    }
}

/// System to handle theme preset changes
pub fn switch_theme_preset(
    mut _theme_events: EventReader<ThemePresetChangeEvent>,
    mut theme: ResMut<WizardTheme>,
    mut animation_state: ResMut<ThemeAnimationState>,
) {
    for event in _theme_events.read() {
        if event.new_preset != theme.current_preset {
            theme.switch_preset(event.new_preset);
            animation_state.start_transition();
            info!("Switched wizard theme to: {:?}", event.new_preset);
        }
    }
}

/// System to detect system theme changes (macOS specific)
#[cfg(target_os = "macos")]
pub fn detect_system_theme_changes(
    _theme_events: EventWriter<ThemePresetChangeEvent>,
    theme: Res<WizardTheme>,
    mut last_check: Local<Option<std::time::Instant>>,
) {
    const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);
    
    if theme.current_preset != ThemePreset::System {
        return;
    }
    
    if last_check.map(|t| t.elapsed() < CHECK_INTERVAL).unwrap_or(false) {
        return;
    }
    
    // In a real implementation, this would check macOS system preferences
    // For now, we'll just update the last check time
    *last_check = Some(std::time::Instant::now());
}
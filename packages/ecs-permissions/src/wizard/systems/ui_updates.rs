//! Wizard UI Update Systems
//!
//! High-performance systems for updating wizard UI elements including
//! permission cards, navigation buttons, progress indicators, and animations.
//! Integrates with ecs-ui for responsive design and smooth transitions.

use bevy::prelude::*;
use action_items_ecs_ui::prelude::*;
use tracing::{debug, warn};

use crate::types::{PermissionType, PermissionStatus};
use crate::wizard::events::PermissionStatusExt;
use crate::wizard::{
    WizardState, WizardRoot, PermissionCard, WizardPanel,
    WizardNavigationButton, WizardProgressIndicator,
    NavigationAction, PermissionCardTitle, PermissionCardDescription,
    PermissionCardButton, PermissionCardStatus, PermissionCardRequirement,
};
use crate::wizard::ui::theme::WizardTheme;
use crate::wizard::ui::{get_permission_icon, get_permission_status_indicator, get_permission_status_color};
use crate::wizard::systems::permissions::WizardPermissionManager;

/// System to update permission cards with smooth animations
/// 
/// Updates permission card states, colors, and animations based on current
/// permission statuses. Provides visual feedback for permission changes.
pub fn update_permission_cards_with_animations(
    mut card_query: Query<(&mut PermissionCard, &mut UiColor), Changed<PermissionCard>>,
    time: Res<Time>,
) {
    for (mut card, mut color) in card_query.iter_mut() {
        // Update animation progress
        if card.status_animation < 1.0 {
            card.status_animation += time.delta_secs() * 3.0; // 3x speed for snappy animations
            card.status_animation = card.status_animation.min(1.0);
        }
        
        // Update color based on status with animation
        let target_color = card.status.color();
        let animation_factor = card.status_animation;
        
        // Smooth color interpolation
        let current_color = color.colors.get(&UiBase::id())
            .copied()
            .unwrap_or(Color::srgb(0.5, 0.5, 0.5));
        
        let current_srgba: bevy::color::Srgba = current_color.into();
        let target_srgba: bevy::color::Srgba = target_color.into();
        
        let interpolated = Color::srgba(
            current_srgba.red + (target_srgba.red - current_srgba.red) * animation_factor,
            current_srgba.green + (target_srgba.green - current_srgba.green) * animation_factor,
            current_srgba.blue + (target_srgba.blue - current_srgba.blue) * animation_factor,
            current_srgba.alpha + (target_srgba.alpha - current_srgba.alpha) * animation_factor,
        );
        
        *color = UiColor::from(interpolated);
    }
}

/// System to update navigation buttons based on current wizard state
/// 
/// Dynamically enables/disables and shows/hides navigation buttons
/// based on the current wizard state and validation rules.
pub fn update_navigation_buttons_with_states(
    mut button_query: Query<(&mut WizardNavigationButton, &mut Visibility)>,
    wizard_state: Res<State<WizardState>>,
    permission_manager: Option<Res<WizardPermissionManager>>,
) {
    if !wizard_state.is_changed() {
        return;
    }
    
    let current_state = *wizard_state.get();
    
    for (mut button, mut visibility) in button_query.iter_mut() {
        button.update_for_state(current_state);
        
        // Additional logic for specific button states
        match button.action {
            NavigationAction::Next => {
                // Disable next button if current step cannot be validated
                if current_state == WizardState::RequestingPermissions {
                    if let Some(manager) = &permission_manager {
                        button.is_enabled = manager.all_required_permissions_granted();
                    }
                }
            },
            NavigationAction::Back => {
                // Always allow going back except from welcome
                button.is_enabled = current_state != WizardState::Welcome;
            },
            _ => {}, // Other buttons use default logic
        }
        
        // Update visibility
        *visibility = if button.is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// System to update wizard modal theming
/// 
/// Applies consistent theming across all wizard UI elements.
/// Responds to theme changes and system preferences.
pub fn update_wizard_modal_theming(
    mut root_query: Query<&mut UiColor, (With<WizardRoot>, Without<PermissionCard>)>,
    wizard_state: Res<State<WizardState>>,
) {
    // Only update when state changes to avoid unnecessary work
    if !wizard_state.is_changed() {
        return;
    }
    
    for mut color in root_query.iter_mut() {
        // Apply state-specific theming
        let theme_color = match wizard_state.get() {
            WizardState::Welcome => Color::srgba(0.1, 0.1, 0.15, 0.95),
            WizardState::CheckingPermissions => Color::srgba(0.1, 0.12, 0.18, 0.95),
            WizardState::RequestingPermissions => Color::srgba(0.15, 0.1, 0.1, 0.95),
            WizardState::SettingUpHotkeys => Color::srgba(0.1, 0.15, 0.1, 0.95),
            WizardState::Complete => Color::srgba(0.1, 0.15, 0.1, 0.95),
            WizardState::NotStarted => Color::srgba(0.1, 0.1, 0.1, 0.0),
        };
        
        *color = UiColor::from(theme_color);
    }
}

/// System to update wizard progress with responsive layout
/// 
/// Updates progress indicators and adjusts layout based on screen size
/// and current wizard state. Provides smooth progress animations.
pub fn update_wizard_progress_with_layout(
    mut progress_query: Query<&mut WizardProgressIndicator>,
    wizard_state: Res<State<WizardState>>,
    windows: Query<&Window>,
) {
    if !wizard_state.is_changed() {
        return;
    }
    
    let current_state = *wizard_state.get();
    
    for mut indicator in progress_query.iter_mut() {
        indicator.update_for_state(current_state);
        
        // Adjust display based on screen size
        if let Ok(window) = windows.single() {
            indicator.show_details = window.width() > 600.0;
        }
    }
}

/// System to update wizard status text with real-time feedback
/// 
/// Provides contextual status messages based on current wizard state
/// and permission checking progress. Updates text dynamically.
pub fn update_wizard_status_text(
    wizard_state: Res<State<WizardState>>,
    permission_manager: Option<Res<WizardPermissionManager>>,
    mut text_query: Query<(Entity, &mut Text), (With<Name>, Without<PermissionCard>)>,
    names: Query<&Name>,
) {
    if !wizard_state.is_changed() {
        return;
    }
    
    let current_state = *wizard_state.get();
    
    for (entity, mut text) in text_query.iter_mut() {
        if let Ok(name) = names.get(entity) {
            if name.as_str().contains("StatusDisplay") {
                let status_message = match current_state {
                    WizardState::NotStarted => "Ready to begin setup",
                    WizardState::Welcome => "Welcome to Action Items permission setup",
                    WizardState::CheckingPermissions => {
                        if let Some(manager) = &permission_manager {
                            let active_count = manager.active_request_count();
                            if active_count > 0 {
                                "Checking system permissions..."
                            } else {
                                "Permission check complete"
                            }
                        } else {
                            "Checking system permissions..."
                        }
                    },
                    WizardState::RequestingPermissions => "Please grant required permissions",
                    WizardState::SettingUpHotkeys => "Setting up global hotkeys",
                    WizardState::Complete => "Setup complete! Welcome to Action Items",
                };
                
                // In Bevy 0.16, Text is a wrapper around a single string
                text.0 = status_message.to_string();
                
                debug!("Updated status text: {}", status_message);
            }
        }
    }
}

/// System to handle permission card hover effects with ecs-ui
/// 
/// Manages hover states and color transitions for permission cards.
/// Integrates with theme system for consistent visual feedback.
pub fn handle_permission_card_hover_effects(
    mut card_query: Query<(&PermissionCard, &mut UiColor, &Interaction), Changed<Interaction>>,
) {
    for (card, mut color, interaction) in card_query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                // Apply hover effect
                let hover_color = Color::srgba(0.8, 0.8, 0.9, 1.0);
                *color = UiColor::from(hover_color);
            },
            Interaction::None => {
                // Restore status color
                let status_color = card.status.color();
                *color = UiColor::from(status_color);
            },
            Interaction::Pressed => {
                // Apply pressed effect
                let pressed_color = Color::srgba(0.6, 0.6, 0.7, 1.0);
                *color = UiColor::from(pressed_color);
            },
        }
    }
}

/// System to update wizard button text based on state
/// 
/// Dynamically updates button labels based on current wizard context.
/// Provides contextual action labels for better user experience.
pub fn update_wizard_button_text(
    wizard_state: Res<State<WizardState>>,
    mut button_query: Query<(&WizardNavigationButton, &mut Text)>,
) {
    if !wizard_state.is_changed() {
        return;
    }
    
    let current_state = *wizard_state.get();
    
    for (button, mut text) in button_query.iter_mut() {
        let button_text = match (button.action, current_state) {
            (NavigationAction::Next, WizardState::Welcome) => "Get Started",
            (NavigationAction::Next, WizardState::CheckingPermissions) => "Continue",
            (NavigationAction::Next, WizardState::RequestingPermissions) => "Continue",
            (NavigationAction::Next, WizardState::SettingUpHotkeys) => "Finish",
            (NavigationAction::Next, _) => "Next",
            (NavigationAction::Back, _) => "Back",
            (NavigationAction::Skip, _) => "Skip",
            (NavigationAction::Cancel, _) => "Cancel",
            (NavigationAction::Finish, _) => "Finish",
        };
        
        // In Bevy 0.16, Text is a wrapper around a single string
        text.0 = button_text.to_string();
    }
}

/// System to handle UI element fade-in animations
/// 
/// Provides smooth entrance animations for UI elements using ecs-ui's animation system.
/// Creates polished transitions when elements become visible.
pub fn animate_ui_element_fade_in(
    mut ui_query: Query<(Entity, &mut UiColor, &Visibility), Changed<Visibility>>,
    time: Res<Time>,
    mut animation_states: Local<std::collections::HashMap<Entity, f32>>,
) {
    for (entity, mut color, visibility) in ui_query.iter_mut() {
        let animation_progress = animation_states.entry(entity).or_insert(0.0);
        
        match visibility {
            Visibility::Visible => {
                if *animation_progress < 1.0 {
                    *animation_progress += time.delta_secs() * 3.0; // 3x speed for snappy animations
                    *animation_progress = animation_progress.min(1.0);
                    
                    // Apply fade-in effect
                    let alpha = *animation_progress;
                    let base_color = color.colors.get(&UiBase::id())
                        .copied()
                        .unwrap_or(Color::srgb(0.5, 0.5, 0.5));
                    let base_srgba: bevy::color::Srgba = base_color.into();
                    *color = UiColor::from(Color::srgba(base_srgba.red, base_srgba.green, base_srgba.blue, alpha));
                }
            },
            Visibility::Hidden => {
                if *animation_progress > 0.0 {
                    *animation_progress -= time.delta_secs() * 3.0;
                    *animation_progress = animation_progress.max(0.0);
                    
                    // Apply fade-out effect
                    let alpha = *animation_progress;
                    let base_color = color.colors.get(&UiBase::id())
                        .copied()
                        .unwrap_or(Color::srgb(0.5, 0.5, 0.5));
                    let base_srgba: bevy::color::Srgba = base_color.into();
                    *color = UiColor::from(Color::srgba(base_srgba.red, base_srgba.green, base_srgba.blue, alpha));
                }
            },
            _ => {},
        }
    }
}

/// System to update permission card grid positions responsively
/// 
/// Dynamically adjusts card positions based on viewport size and content.
/// Provides adaptive layouts for different screen sizes.
pub fn update_permission_card_grid_positions(
    mut card_query: Query<(&PermissionCard, &mut UiLayout)>,
    windows: Query<&Window>,
    wizard_state: Res<State<WizardState>>,
) {
    // Only update when state changes or during permission-related states
    if !matches!(*wizard_state.get(), WizardState::CheckingPermissions | WizardState::RequestingPermissions | WizardState::Complete) {
        return;
    }
    
    if let Ok(window) = windows.single() {
        let (columns, card_width, card_height, margin) = match window.width() {
            w if w > 1200.0 => (3, Rl(28.0), Rl(20.0), 6.0),   // Large screens: 3 columns
            w if w > 800.0 => (2, Rl(42.0), Rl(22.0), 8.0),    // Medium screens: 2 columns
            _ => (1, Rl(85.0), Rl(18.0), 7.5),                 // Small screens: 1 column
        };
        
        for (card, mut layout) in card_query.iter_mut() {
            let card_index = get_permission_index(card.permission_type);
            let col = card_index % columns;
            let row = card_index / columns;
            
            let x_pos = Rl(margin + (col as f32 * (100.0 - 2.0 * margin) / columns as f32));
            let y_pos = Rl(20.0 + (row as f32 * 25.0));
            
            *layout = UiLayout::window()
                .size((card_width, card_height))
                .pos((x_pos, y_pos))
                .pack();
        }
    }
}

/// Helper function to get consistent permission index for grid positioning
/// 
/// Note: This function explicitly handles only the permissions used in the wizard.
/// If new permission types are added, they must be added here to prevent grid overlaps.
fn get_permission_index(permission: PermissionType) -> usize {
    match permission {
        PermissionType::Accessibility => 0,
        PermissionType::ScreenCapture => 1, 
        PermissionType::InputMonitoring => 2,
        PermissionType::Camera => 3,
        PermissionType::Microphone => 4,
        PermissionType::FullDiskAccess => 5,
        PermissionType::WiFi => 6,
        // Explicitly handle other known permissions if they exist
        // Remove catch-all to force explicit handling of new permission types
        other => {
            warn!("Unknown permission type in wizard grid: {:?}", other);
            7 // Safe fallback position, but logs the issue
        }
    }
}

/// Run condition to check if ecs-ui wizard updates should be active
pub fn wizard_ui_active(wizard_state: Res<State<WizardState>>) -> bool {
    // UI updates should be active when wizard is active OR just completed
    // (to handle completion animations and cleanup)
    matches!(
        wizard_state.get(),
        WizardState::Welcome | WizardState::CheckingPermissions |
        WizardState::RequestingPermissions | WizardState::SettingUpHotkeys |
        WizardState::Complete
    )
}

/// System to setup hotkey configuration interface for SettingUpHotkeys panel
///
/// Creates hotkey input fields, default suggestions, conflict detection,
/// validation feedback, and skip option using ecs-ui components.
pub fn setup_hotkey_configuration_interface_system(
    mut commands: Commands,
    wizard_state: Res<State<WizardState>>,
    panel_query: Query<Entity, (With<WizardPanel>, Without<Name>)>,
    existing_interface: Query<&Name>,
    mut last_state: Local<Option<WizardState>>,
) {
    let current_state = *wizard_state.get();

    // Only setup interface when entering SettingUpHotkeys state
    if last_state.map(|s| s != current_state).unwrap_or(true) {
        if current_state == WizardState::SettingUpHotkeys {
            // Find the SettingUpHotkeys panel
            for panel_entity in panel_query.iter() {
                // Check if interface already exists to avoid duplicates
                let interface_exists = existing_interface
                    .iter()
                    .any(|name| name.as_str().contains("HotkeyConfigurationInterface"));

                if !interface_exists {
                    commands.entity(panel_entity).with_children(|parent| {
                        // Main hotkey configuration interface container
                        parent.spawn((
                            UiLayout::window()
                                .size((Rl(80.0), Rl(60.0)))
                                .pos((Rl(10.0), Rl(25.0)))
                                .pack(),
                            UiColor::from(Color::srgba(0.1, 0.1, 0.12, 0.9)),
                            Name::new("HotkeyConfigurationInterface"),
                        )).with_children(|interface| {

                            // Platform-specific default suggestions
                            let platform = detect_current_platform();
                            let (primary_suggestion, secondary_suggestion, modifier_name) = match platform {
                                Platform::MacOS => ("⌘ + Space", "⌘ + Shift + A", "Cmd"),
                                Platform::Windows => ("Ctrl + Space", "Ctrl + Shift + A", "Ctrl"),
                                Platform::Linux => ("Super + Space", "Super + Shift + A", "Super"),
                            };

                            // Default suggestion buttons
                            interface.spawn((
                                UiLayout::window()
                                    .size((Rl(35.0), Rl(8.0)))
                                    .pos((Rl(10.0), Rl(15.0)))
                                    .pack(),
                                UiColor::from(Color::srgba(0.2, 0.6, 0.9, 1.0)),
                                UiHover::new().forward_speed(8.0).backward_speed(4.0),
                                UiClicked::new().forward_speed(12.0).backward_speed(6.0),
                                Text::new(primary_suggestion.to_string()),
                                Interaction::None,
                                Name::new("HotkeyPrimarySuggestionButton"),
                            ));

                            interface.spawn((
                                UiLayout::window()
                                    .size((Rl(35.0), Rl(8.0)))
                                    .pos((Rl(55.0), Rl(15.0)))
                                    .pack(),
                                UiColor::from(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                                UiHover::new().forward_speed(8.0).backward_speed(4.0),
                                UiClicked::new().forward_speed(12.0).backward_speed(6.0),
                                Text::new(secondary_suggestion.to_string()),
                                Interaction::None,
                                Name::new("HotkeySecondarySuggestionButton"),
                            ));

                            // Custom hotkey input field
                            interface.spawn((
                                UiLayout::window()
                                    .size((Rl(80.0), Rl(10.0)))
                                    .pos((Rl(10.0), Rl(30.0)))
                                    .pack(),
                                UiColor::from(Color::srgba(0.15, 0.15, 0.15, 1.0)),
                                UiHover::new().forward_speed(8.0).backward_speed(4.0),
                                Text::new("Press your preferred key combination".to_string()),
                                Interaction::None,
                                Name::new("HotkeyCustomInputField"),
                            ));

                            // Validation feedback area
                            interface.spawn((
                                UiLayout::window()
                                    .size((Rl(80.0), Rl(8.0)))
                                    .pos((Rl(10.0), Rl(45.0)))
                                    .pack(),
                                UiColor::from(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                                Text::new("".to_string()),
                                Name::new("HotkeyValidationFeedback"),
                            ));

                            // Conflict detection status
                            interface.spawn((
                                UiLayout::window()
                                    .size((Rl(80.0), Rl(6.0)))
                                    .pos((Rl(10.0), Rl(55.0)))
                                    .pack(),
                                UiColor::from(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                                Text::new("Checking for conflicts...".to_string()),
                                Name::new("HotkeyConflictStatus"),
                            ));

                            // Skip button
                            interface.spawn((
                                UiLayout::window()
                                    .size((Rl(25.0), Rl(8.0)))
                                    .pos((Rl(10.0), Rl(75.0)))
                                    .pack(),
                                UiColor::from(Color::srgba(0.6, 0.6, 0.6, 1.0)),
                                UiHover::new().forward_speed(8.0).backward_speed(4.0),
                                UiClicked::new().forward_speed(12.0).backward_speed(6.0),
                                Text::new("Skip Hotkey Setup".to_string()),
                                Interaction::None,
                                Name::new("HotkeySkipButton"),
                            ));

                            // Confirm button (initially disabled)
                            interface.spawn((
                                UiLayout::window()
                                    .size((Rl(25.0), Rl(8.0)))
                                    .pos((Rl(65.0), Rl(75.0)))
                                    .pack(),
                                UiColor::from(Color::srgba(0.3, 0.3, 0.3, 0.5)),
                                Text::new("Confirm Hotkey".to_string()),
                                Name::new("HotkeyConfirmButton"),
                            ));

                            // Instructions text
                            interface.spawn((
                                Text::new(format!("Use {} key combinations for global access", modifier_name)),
                                Name::new("HotkeyInstructions"),
                            ));
                        });
                    });

                    debug!("Created hotkey configuration interface for SettingUpHotkeys panel");
                }
            }
        }
        *last_state = Some(current_state);
    }
}

/// Platform detection for hotkey suggestions
fn detect_current_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOS
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "linux") {
        Platform::Linux
    } else {
        Platform::Linux // Default fallback
    }
}

/// Platform enumeration for hotkey suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    MacOS,
    Windows,
    Linux,
}/// System to handle permission card click interactions
/// 
/// Detects clicks on permission cards and triggers permission requests.
/// Provides visual feedback for user interactions.
pub fn handle_permission_card_interactions_system(
    mut card_query: Query<(&PermissionCard, &Interaction, &mut UiColor), Changed<Interaction>>,
    mut permission_events: EventWriter<crate::wizard::events::WizardPermissionRequest>,
) {
    for (card, interaction, mut color) in card_query.iter_mut() {
        if !card.is_interactive {
            continue;
        }
        
        match interaction {
            Interaction::Pressed => {
                // Send permission request event when card is clicked
                permission_events.write(crate::wizard::events::WizardPermissionRequest::new(card.permission_type));
                
                // Apply pressed visual feedback
                let pressed_color = if card.is_required {
                    Color::srgba(0.5, 0.5, 0.7, 1.0) // Blue-ish for required
                } else {
                    Color::srgba(0.5, 0.7, 0.5, 1.0) // Green-ish for optional
                };
                *color = UiColor::from(pressed_color);
            },
            Interaction::Hovered => {
                // Apply hover visual feedback
                let hover_color = if card.is_required {
                    Color::srgba(0.7, 0.7, 0.9, 1.0) // Light blue for required
                } else {
                    Color::srgba(0.7, 0.9, 0.7, 1.0) // Light green for optional
                };
                *color = UiColor::from(hover_color);
            },
            Interaction::None => {
                // Restore status-based color with required/optional distinction
                let base_color = card.status.color();
                let base_srgba: bevy::color::Srgba = base_color.into();
                
                let final_color = if card.is_required {
                    // Slightly more intense colors for required permissions
                    Color::srgba(
                        (base_srgba.red * 1.1).min(1.0),
                        (base_srgba.green * 1.1).min(1.0),
                        (base_srgba.blue * 1.1).min(1.0),
                        base_srgba.alpha,
                    )
                } else {
                    // Slightly muted colors for optional permissions
                    Color::srgba(
                        base_srgba.red * 0.8,
                        base_srgba.green * 0.8,
                        base_srgba.blue * 0.8,
                        base_srgba.alpha * 0.9,
                    )
                };
                *color = UiColor::from(final_color);
            },
        }
    }
}

/// System to populate permission cards with content
/// 
/// Creates child entities for card titles, descriptions, and buttons
/// when new permission cards are added to the scene.
pub fn populate_permission_card_content_system(
    mut commands: Commands,
    card_query: Query<(Entity, &PermissionCard), Added<PermissionCard>>,
    theme: Res<WizardTheme>,
) {
    for (card_entity, card) in card_query.iter() {
        // Create structured content for the permission card
        commands.entity(card_entity).with_children(|parent| {
            // Use theme colors and styling - extract Color from UiColor
            let title_color = if card.is_required {
                theme.primary_color.colors.get(&UiBase::id()).copied().unwrap_or(Color::WHITE)
            } else {
                theme.secondary_color.colors.get(&UiBase::id()).copied().unwrap_or(Color::srgb(0.7, 0.7, 0.7))
            };
            
            // Card title
            parent.spawn((
                Text::new(format!("{}", card.permission_type)),
                TextFont {
                    font: default(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(title_color),
                PermissionCardTitle,
            ));
            
            // Card description - use secondary color for description text
            let description_color = theme.secondary_color.colors.get(&UiBase::id()).copied().unwrap_or(Color::srgb(0.7, 0.7, 0.7));
            parent.spawn((
                Text::new(card.description().to_string()),
                TextFont {
                    font: default(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(description_color),
                PermissionCardDescription,
            ));
            
            // Status indicator
            let status_color = match card.status {
                PermissionStatus::Authorized => theme.success_color.colors.get(&UiBase::id()).copied().unwrap_or(Color::srgb(0.0, 0.8, 0.0)),
                PermissionStatus::Denied => theme.error_color.colors.get(&UiBase::id()).copied().unwrap_or(Color::srgb(0.8, 0.0, 0.0)),
                PermissionStatus::NotDetermined => theme.warning_color.colors.get(&UiBase::id()).copied().unwrap_or(Color::srgb(0.8, 0.8, 0.0)),
                PermissionStatus::Unknown => theme.card_background.colors.get(&UiBase::id()).copied().unwrap_or(Color::srgb(0.5, 0.5, 0.5)),
                PermissionStatus::Restricted => theme.error_color.colors.get(&UiBase::id()).copied().unwrap_or(Color::srgb(0.8, 0.0, 0.0)),
            };
            
            parent.spawn((
                Text::new(card.status.description().to_string()),
                TextFont {
                    font: default(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(status_color),
                PermissionCardStatus,
            ));
            
            // Grant button (only if not already granted)
            if !card.status.is_granted() {
                let button_text = if card.is_required {
                    "Grant Required Permission"
                } else {
                    "Grant Optional Permission"
                };
                
                // Use primary color for button text
                let button_text_color = theme.primary_color.colors.get(&UiBase::id()).copied().unwrap_or(Color::WHITE);
                parent.spawn((
                    Text::new(button_text.to_string()),
                    TextFont {
                        font: default(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(button_text_color),
                    PermissionCardButton,
                ));
            }
            
            // Required/Optional indicator
            let requirement_text = if card.is_required {
                "Required"
            } else {
                "Optional"
            };
            
            let requirement_color = if card.is_required {
                theme.error_color.colors.get(&UiBase::id()).copied().unwrap_or(Color::srgb(0.8, 0.0, 0.0))
            } else {
                theme.secondary_color.colors.get(&UiBase::id()).copied().unwrap_or(Color::srgb(0.7, 0.7, 0.7))
            };
            
            parent.spawn((
                Text::new(requirement_text.to_string()),
                TextFont {
                    font: default(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(requirement_color),
                PermissionCardRequirement,
            ));
        });
    }
}
/// System to populate wizard panels with content
/// 
/// Creates child entities for panel titles, descriptions, and interactive elements
/// when new wizard panels are added to the scene. Transforms empty colored
/// rectangles into informative user interfaces.
pub fn populate_wizard_panel_content_system(
    mut commands: Commands,
    panel_query: Query<(Entity, &WizardPanel), Added<WizardPanel>>,
) {
    for (panel_entity, panel) in panel_query.iter() {
        // Create structured content based on panel state
        commands.entity(panel_entity).with_children(|parent| {
            match panel.state {
                WizardState::Welcome => {
                    // Welcome Panel Content
                    parent.spawn((
                        Text::new("Welcome to Action Items"),
                        Name::new("WizardPanel_Welcome_Title"),
                    ));
                    
                    parent.spawn((
                        Text::new("We need to set up a few permissions to get started"),
                        Name::new("WizardPanel_Welcome_Description"),
                    ));
                    
                    parent.spawn((
                        Text::new("This will only take a minute"),
                        Name::new("WizardPanel_Welcome_Subtitle"),
                    ));
                    
                    // Feature highlights
                    parent.spawn((
                        Text::new("• Quick and easy setup"),
                        Name::new("WizardPanel_Welcome_Feature1"),
                    ));
                    
                    parent.spawn((
                        Text::new("• Secure permission handling"),
                        Name::new("WizardPanel_Welcome_Feature2"),
                    ));
                    
                    parent.spawn((
                        Text::new("• Customizable hotkeys"),
                        Name::new("WizardPanel_Welcome_Feature3"),
                    ));
                },
                
                WizardState::CheckingPermissions => {
                    // CheckingPermissions Panel Content
                    parent.spawn((
                        Text::new("Checking system permissions..."),
                        Name::new("WizardPanel_CheckingPermissions_LoadingText"),
                    ));
                    
                    parent.spawn((
                        Text::new("◯ Accessibility permissions"),
                        Name::new("WizardPanel_CheckingPermissions_AccessibilityProgress"),
                    ));
                    
                    parent.spawn((
                        Text::new("◯ Screen capture permissions"),
                        Name::new("WizardPanel_CheckingPermissions_ScreenCaptureProgress"),
                    ));
                    
                    parent.spawn((
                        Text::new("◯ Input monitoring permissions"),
                        Name::new("WizardPanel_CheckingPermissions_InputMonitoringProgress"),
                    ));
                    
                    parent.spawn((
                        Text::new("◯ Camera permissions"),
                        Name::new("WizardPanel_CheckingPermissions_CameraProgress"),
                    ));
                    
                    parent.spawn((
                        Text::new("◯ Microphone permissions"),
                        Name::new("WizardPanel_CheckingPermissions_MicrophoneProgress"),
                    ));
                    
                    parent.spawn((
                        Text::new("◯ Full disk access permissions"),
                        Name::new("WizardPanel_CheckingPermissions_FullDiskProgress"),
                    ));
                },
                
                WizardState::RequestingPermissions => {
                    // RequestingPermissions Panel Content
                    parent.spawn((
                        Text::new("Please grant the following permissions"),
                        Name::new("WizardPanel_RequestingPermissions_Instructions"),
                    ));
                    
                    parent.spawn((
                        Text::new("These permissions enable Action Items to provide you with the best experience"),
                        Name::new("WizardPanel_RequestingPermissions_Explanation"),
                    ));
                    
                    parent.spawn((
                        Text::new("Click on each permission card below to grant access"),
                        Name::new("WizardPanel_RequestingPermissions_Guidance"),
                    ));
                    
                    parent.spawn((
                        Text::new("Required permissions are marked and must be granted to continue"),
                        Name::new("WizardPanel_RequestingPermissions_RequiredNote"),
                    ));
                    
                    parent.spawn((
                        Text::new("Optional permissions can be granted later in Settings"),
                        Name::new("WizardPanel_RequestingPermissions_OptionalNote"),
                    ));
                },
                
                WizardState::SettingUpHotkeys => {
                    // SettingUpHotkeys Panel Content
                    parent.spawn((
                        Text::new("Set up global hotkeys"),
                        Name::new("WizardPanel_SettingUpHotkeys_Title"),
                    ));
                    
                    parent.spawn((
                        Text::new("Configure keyboard shortcuts to quickly access Action Items"),
                        Name::new("WizardPanel_SettingUpHotkeys_Description"),
                    ));
                    
                    // Default hotkey suggestions
                    parent.spawn((
                        Text::new("Suggested hotkey: ⌘ + Space"),
                        Name::new("WizardPanel_SettingUpHotkeys_DefaultSuggestion"),
                    ));
                    
                    parent.spawn((
                        Text::new("Alternative: ⌘ + ⌥ + A"),
                        Name::new("WizardPanel_SettingUpHotkeys_AlternativeSuggestion"),
                    ));
                    
                    // Configuration interface placeholder
                    parent.spawn((
                        Text::new("Press your preferred key combination"),
                        Name::new("WizardPanel_SettingUpHotkeys_ConfigInterface"),
                    ));
                    
                    parent.spawn((
                        Text::new("You can change this later in Preferences"),
                        Name::new("WizardPanel_SettingUpHotkeys_Instructions"),
                    ));
                },
                
                WizardState::Complete => {
                    // Complete Panel Content
                    parent.spawn((
                        Text::new("Setup complete! Welcome to Action Items"),
                        Name::new("WizardPanel_Complete_SuccessMessage"),
                    ));
                    
                    parent.spawn((
                        Text::new("✓ Permissions configured"),
                        Name::new("WizardPanel_Complete_PermissionsSummary"),
                    ));
                    
                    parent.spawn((
                        Text::new("✓ Hotkeys set up"),
                        Name::new("WizardPanel_Complete_HotkeysSummary"),
                    ));
                    
                    parent.spawn((
                        Text::new("✓ Ready to use"),
                        Name::new("WizardPanel_Complete_ReadySummary"),
                    ));
                    
                    // What's next guidance
                    parent.spawn((
                        Text::new("Press your hotkey to start using Action Items"),
                        Name::new("WizardPanel_Complete_NextSteps1"),
                    ));
                    
                    parent.spawn((
                        Text::new("Visit Preferences to customize further"),
                        Name::new("WizardPanel_Complete_NextSteps2"),
                    ));
                    
                    parent.spawn((
                        Text::new("Check the menu bar for quick access"),
                        Name::new("WizardPanel_Complete_NextSteps3"),
                    ));
                },
                
                WizardState::NotStarted => {
                    // NotStarted state typically wouldn't have a visible panel
                    // but including for completeness
                    parent.spawn((
                        Text::new("Action Items Setup"),
                        Name::new("WizardPanel_NotStarted_Title"),
                    ));
                    
                    parent.spawn((
                        Text::new("Click 'Get Started' to begin"),
                        Name::new("WizardPanel_NotStarted_Instructions"),
                    ));
                },
            }
        });
    }
}

/// System to integrate icons into permission card titles
///
/// Adds visual icons and status indicators to permission cards for better UX.
/// Updates text dynamically based on permission type and status.
pub fn integrate_icons_into_permission_cards(
    card_query: Query<(&PermissionCard, &Children), Changed<PermissionCard>>,
    mut title_query: Query<&mut Text, With<PermissionCardTitle>>,
    mut status_query: Query<(&mut Text, &mut TextColor), (With<PermissionCardStatus>, Without<PermissionCardTitle>)>,
) {
    for (card, children) in card_query.iter() {
        // Update title with icon
        for child_entity in children {
            if let Ok(mut text) = title_query.get_mut(*child_entity) {
                let icon = get_permission_icon(card.permission_type);
                let permission_name = format!("{:?}", card.permission_type);
                text.0 = format!("{} {}", icon, permission_name);
            }
            
            // Update status indicator with colored symbol and text
            if let Ok((mut text, mut color)) = status_query.get_mut(*child_entity) {
                let status_icon = get_permission_status_indicator(card.status);
                let status_color = get_permission_status_color(card.status);
                let status_text = match card.status {
                    PermissionStatus::Authorized => "Granted",
                    PermissionStatus::Denied => "Denied",
                    PermissionStatus::Restricted => "Restricted",
                    PermissionStatus::NotDetermined => "Pending",
                    PermissionStatus::Unknown => "Checking...",
                };
                text.0 = format!("{} {}", status_icon, status_text);
                color.0 = status_color;
            }
        }
    }
}

/// System to add hover effects to permission card buttons
///
/// Enhances permission card buttons with interactive hover and click animations
/// for improved visual feedback.
pub fn add_hover_effects_to_permission_card_buttons(
    mut commands: Commands,
    button_query: Query<Entity, (With<PermissionCardButton>, Without<UiHover>)>,
) {
    for button_entity in button_query.iter() {
        commands.entity(button_entity).insert((
            UiHover::new().forward_speed(8.0).backward_speed(4.0),
            UiClicked::new().forward_speed(12.0).backward_speed(6.0),
            Interaction::None,
        ));
    }
}
//! Privacy indicator systems for real-time updates and interaction handling
//!
//! Zero-allocation systems for blazing-fast privacy status updates with efficient change detection.
//! Integrated with professional gradient theme system for Raycast-like aesthetics.

use bevy::prelude::*;
use tracing::debug;

use super::privacy_events::*;
use super::privacy_indicators::*;
use action_items_ecs_ui::{
    GradientComponent, GradientComponentType, GradientInteractionState, GradientTheme,
    UiVisibilityEvent,
};

/// Complex query type for privacy icon button components
type PrivacyIconQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static PrivacyIconButton,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
    (Changed<PrivacyIconButton>, Without<GradientComponent>),
>;

/// System to update privacy indicators based on configuration changes
/// Uses efficient change detection to minimize unnecessary updates
#[inline]
pub fn update_privacy_indicators_system(
    privacy_config: Res<PrivacyConfiguration>,
    mut privacy_indicators_query: Query<&mut PrivacyIndicators>,
    mut privacy_events: EventWriter<PrivacyStatusChanged>,
) {
    if privacy_config.is_changed() {
        let (full_control, no_collection, encrypted) = privacy_config.calculate_indicators();

        for mut indicators in privacy_indicators_query.iter_mut() {
            if indicators.update_states(full_control, no_collection, encrypted) {
                privacy_events.write(PrivacyStatusChanged::new(
                    full_control,
                    no_collection,
                    encrypted,
                ));
            }
        }
    }
}

/// System to handle privacy icon button interactions with blazing-fast responsiveness
#[inline]
pub fn handle_privacy_button_interactions_system(
    mut button_query: Query<(Entity, &mut PrivacyIconButton, &Interaction), Changed<Interaction>>,
    mut privacy_indicators_query: Query<&mut PrivacyIndicators>,
    mut toggle_events: EventWriter<TogglePrivacyInfo>,
    mut hover_events: EventWriter<PrivacyIndicatorHover>,
    mut ui_events: EventWriter<UiVisibilityEvent>,
) {
    for (entity, mut button, interaction) in button_query.iter_mut() {
        let _old_hover_state = button.hover_state;

        // Update hover state based on interaction
        let new_hover_state = match *interaction {
            Interaction::Hovered => HoverState::Hovered,
            Interaction::Pressed => HoverState::Pressed,
            Interaction::None => HoverState::Normal,
        };

        // Send hover event if state changed
        if button.set_hover_state(new_hover_state) {
            hover_events.write(PrivacyIndicatorHover {
                indicator_type: button.indicator_type,
                entity,
                hovering: new_hover_state != HoverState::Normal,
            });
        }

        // Handle click events for info button
        if matches!(*interaction, Interaction::Pressed)
            && matches!(button.indicator_type, IndicatorType::InfoDetails)
        {
            button.interaction_state = InteractionState::Clicked;

            // Find privacy indicators to toggle info panel
            for mut indicators in privacy_indicators_query.iter_mut() {
                indicators.toggle_info(&mut ui_events);

                toggle_events.write(TogglePrivacyInfo {
                    container_entity: entity,
                    expand: indicators.info_expanded,
                });
            }
        } else if matches!(*interaction, Interaction::None)
            && matches!(button.interaction_state, InteractionState::Clicked)
        {
            button.interaction_state = InteractionState::Released;
        }
    }
}

/// System to animate privacy info panel expansion/collapse with smooth transitions
#[inline]
pub fn animate_privacy_info_panel_system(
    time: Res<Time>,
    mut info_panel_query: Query<(&mut PrivacyInfoPanel, &mut Node)>,
) {
    let delta_time = time.delta_secs();

    for (mut panel, mut style) in info_panel_query.iter_mut() {
        if panel.update_animation(delta_time) {
            // Update panel height based on animation progress
            let target_height = if panel.target_expanded {
                panel.content_height
            } else {
                0.0
            };
            let current_height = panel.animation_progress * target_height;

            style.height = Val::Px(current_height);

            // Update visibility for performance
            style.display = if panel.expanded {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

/// System to update privacy icon visual states with professional gradients
/// Zero-allocation system for applying gradient themes to privacy indicators based on status and
/// interaction
#[inline]
pub fn update_privacy_icon_gradients_system(
    privacy_config: Res<PrivacyConfiguration>,
    _gradient_theme: Res<GradientTheme>,
    mut icon_query: Query<
        (
            &PrivacyIconButton,
            &mut GradientComponent,
            Option<&mut BorderColor>,
        ),
        Changed<PrivacyIconButton>,
    >,
) {
    let (full_control, no_collection, encrypted) = privacy_config.calculate_indicators();

    for (button, mut gradient_component, border_color) in icon_query.iter_mut() {
        // Determine if this indicator should be active
        let is_active = match button.indicator_type {
            IndicatorType::FullControl => full_control,
            IndicatorType::NoCollection => no_collection,
            IndicatorType::Encrypted => encrypted,
            IndicatorType::InfoDetails => true, // Always active
        };

        // Update gradient component based on indicator state and hover status
        let new_gradient_state =
            calculate_gradient_interaction_state(is_active, button.hover_state);

        // Update component type based on indicator status
        gradient_component.component_type = if is_active {
            match button.indicator_type {
                IndicatorType::FullControl
                | IndicatorType::NoCollection
                | IndicatorType::Encrypted => {
                    GradientComponentType::SuccessState // Active privacy indicators use success gradient
                },
                IndicatorType::InfoDetails => GradientComponentType::SecondaryContainer, /* Info button uses secondary gradient */
            }
        } else {
            GradientComponentType::ListItem // Inactive indicators use neutral gradient
        };

        // Update interaction state for gradient animations
        if gradient_component.interaction_state != new_gradient_state {
            gradient_component.interaction_state = new_gradient_state;
        }

        // Maintain border color for additional visual definition
        if let Some(mut border) = border_color {
            let border_color = calculate_privacy_border_color(is_active, button.hover_state);
            *border = BorderColor(border_color);
        }
    }
}

/// Legacy system for backward compatibility with non-gradient privacy icons
/// Zero-allocation fallback system for privacy icons that haven't been upgraded to gradient system
#[inline]
pub fn update_privacy_icon_visuals_system(
    privacy_config: Res<PrivacyConfiguration>,
    mut icon_query: PrivacyIconQuery,
) {
    let (full_control, no_collection, encrypted) = privacy_config.calculate_indicators();

    for (button, mut bg_color, mut border_color) in icon_query.iter_mut() {
        // Determine if this indicator should be active
        let is_active = match button.indicator_type {
            IndicatorType::FullControl => full_control,
            IndicatorType::NoCollection => no_collection,
            IndicatorType::Encrypted => encrypted,
            IndicatorType::InfoDetails => true, // Always active
        };

        // Calculate colors based on active state and hover
        let (background, border) = calculate_indicator_colors(is_active, button.hover_state);

        *bg_color = BackgroundColor(background);
        *border_color = BorderColor(border);
    }
}

/// System to handle privacy info panel toggle events
#[inline]
pub fn handle_privacy_info_toggle_system(
    mut toggle_events: EventReader<TogglePrivacyInfo>,
    mut info_panel_query: Query<&mut PrivacyInfoPanel>,
) {
    for event in toggle_events.read() {
        // Log container entity for debugging
        debug!(
            "Toggling privacy info panel for container: {:?}",
            event.container_entity
        );

        for mut panel in info_panel_query.iter_mut() {
            if event.expand {
                panel.start_expand();
            } else {
                panel.start_collapse();
            }
        }
    }
}

/// System to handle privacy status change events for logging and analytics
#[inline]
pub fn handle_privacy_status_events_system(mut status_events: EventReader<PrivacyStatusChanged>) {
    for event in status_events.read() {
        debug!(
            "Privacy status changed at {:?}: full_control={}, no_collection={}, encrypted={}",
            event.timestamp, event.full_control, event.no_collection, event.encrypted
        );

        // Future: Could trigger analytics, logging, or UI notifications
    }
}

/// System to handle privacy indicator hover events for enhanced UX
#[inline]
pub fn handle_privacy_hover_events_system(mut hover_events: EventReader<PrivacyIndicatorHover>) {
    for event in hover_events.read() {
        debug!(
            "Privacy indicator hover: type={:?}, entity={:?}, hovering={}",
            event.indicator_type, event.entity, event.hovering
        );

        // Future: Could trigger tooltips, sound effects, or visual feedback
    }
}

/// Calculate gradient interaction state based on privacy indicator status and hover state
/// Returns appropriate GradientInteractionState for professional visual feedback
#[inline]
fn calculate_gradient_interaction_state(
    is_active: bool,
    hover_state: HoverState,
) -> GradientInteractionState {
    if !is_active {
        // Inactive indicators always use disabled state regardless of hover
        return GradientInteractionState::Disabled;
    }

    // Active indicators respond to hover states
    match hover_state {
        HoverState::Normal => GradientInteractionState::Default,
        HoverState::Hovered => GradientInteractionState::Hover,
        HoverState::Pressed => GradientInteractionState::Pressed,
    }
}

/// Calculate privacy indicator border color for enhanced visual definition
/// Returns appropriate border color based on indicator status and interaction state
#[inline]
fn calculate_privacy_border_color(is_active: bool, hover_state: HoverState) -> Color {
    // Professional border colors for privacy indicators
    const ACTIVE_BORDER: Color = Color::srgba(0.4, 0.8, 0.4, 0.6); // Green tint for active indicators
    const INACTIVE_BORDER: Color = Color::srgba(0.5, 0.5, 0.5, 0.3); // Gray for inactive
    const HOVER_BORDER: Color = Color::srgba(0.6, 0.9, 0.6, 0.8); // Brighter green on hover
    const PRESSED_BORDER: Color = Color::srgba(0.3, 0.7, 0.3, 0.9); // Darker green when pressed

    if !is_active {
        return INACTIVE_BORDER;
    }

    match hover_state {
        HoverState::Normal => ACTIVE_BORDER,
        HoverState::Hovered => HOVER_BORDER,
        HoverState::Pressed => PRESSED_BORDER,
    }
}

/// Calculate indicator colors based on active state and hover state
/// Returns (background_color, border_color) for efficient visual updates
/// Maintained for backward compatibility with legacy privacy icons
#[inline]
fn calculate_indicator_colors(is_active: bool, hover_state: HoverState) -> (Color, Color) {
    // Theme colors for privacy indicators
    const ACTIVE_COLOR: Color = Color::srgb(1.0, 1.0, 1.0); // White
    const INACTIVE_COLOR: Color = Color::srgb(0.53, 0.53, 0.53); // #888888
    const HOVER_COLOR: Color = Color::srgb(0.9, 0.9, 0.9); // Light gray
    const PRESSED_COLOR: Color = Color::srgb(0.8, 0.8, 0.8); // Darker gray
    const BORDER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.1); // Subtle border

    let base_color = if is_active {
        ACTIVE_COLOR
    } else {
        INACTIVE_COLOR
    };

    let final_color = match hover_state {
        HoverState::Normal => base_color,
        HoverState::Hovered => {
            if is_active {
                HOVER_COLOR
            } else {
                INACTIVE_COLOR
            }
        },
        HoverState::Pressed => {
            if is_active {
                PRESSED_COLOR
            } else {
                INACTIVE_COLOR
            }
        },
    };

    (final_color, BORDER_COLOR)
}

/// System to initialize privacy configuration with secure defaults
#[inline]
pub fn initialize_privacy_configuration_system(
    mut commands: Commands,
    privacy_config: Option<Res<PrivacyConfiguration>>,
) {
    if privacy_config.is_none() {
        commands.insert_resource(PrivacyConfiguration::secure_default());
    }
}

/// Enhanced system for initializing privacy indicators with gradient integration
/// Zero-allocation system for setting up privacy components with professional gradient theming
#[inline]
pub fn initialize_privacy_gradients_system(
    mut commands: Commands,
    privacy_buttons: Query<Entity, (With<PrivacyIconButton>, Without<GradientComponent>)>,
) {
    // Add gradient components to existing privacy buttons that don't have them
    for entity in privacy_buttons.iter() {
        commands
            .entity(entity)
            .insert(GradientComponent::secondary_container().with_transition_speed(0.15)); // Fast transitions for responsive privacy feedback
    }
}

/// Plugin for privacy indicator systems with professional gradient integration
pub struct PrivacyIndicatorPlugin;

impl Plugin for PrivacyIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register privacy events
            .add_event::<PrivacyStatusChanged>()
            .add_event::<TogglePrivacyInfo>()
            .add_event::<PrivacyIndicatorHover>()
            // Initialize privacy configuration and gradients
            .add_systems(
                Startup,
                (
                    initialize_privacy_configuration_system,
                    initialize_privacy_gradients_system,
                )
                    .chain(),
            )
            // Update systems with proper ordering
            .add_systems(
                Update,
                (
                    update_privacy_indicators_system,
                    handle_privacy_button_interactions_system,
                    animate_privacy_info_panel_system,
                    // Gradient-based visual updates (preferred)
                    update_privacy_icon_gradients_system,
                    // Legacy color-based updates (fallback)
                    update_privacy_icon_visuals_system,
                    handle_privacy_info_toggle_system,
                    handle_privacy_status_events_system,
                    handle_privacy_hover_events_system,
                )
                    .chain(),
            );
    }
}

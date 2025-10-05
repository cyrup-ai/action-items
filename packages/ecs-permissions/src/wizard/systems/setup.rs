//! Wizard UI Setup and Entity Pre-allocation Systems
//!
//! Provides systems for efficient wizard UI management through entity pre-allocation
//! and visibility toggling. Optimizes performance by avoiding spawn/despawn cycles.

#![allow(dead_code)]

use std::collections::HashMap;
use bevy::prelude::*;
use action_items_ecs_ui::prelude::*;
use action_items_ecs_ui::Ab;

use crate::types::PermissionType;
use crate::wizard::{
    WizardState, WizardRoot, WizardPanel, WizardNavigationButton,
    WizardProgressIndicator, NavigationAction,
    PermissionCardBundle,
};

/// Resource containing pre-allocated UI entities for efficient management
/// 
/// All wizard UI entities are created once during setup and managed through
/// visibility toggling rather than spawn/despawn cycles for optimal performance.
#[derive(Resource)]
pub struct WizardUIEntities {
    /// Root modal entity containing all wizard UI
    pub modal_root: Entity,
    /// Main modal window entity
    pub modal_window: Entity,
    /// Backdrop overlay entity
    pub backdrop: Entity,
    /// Pre-allocated permission cards mapped by permission type
    pub permission_cards: HashMap<PermissionType, Entity>,
    /// Pre-allocated navigation buttons mapped by action
    pub navigation_buttons: HashMap<NavigationAction, Entity>,
    /// Progress indicator entity
    pub progress_indicator: Entity,
    /// Status display entity
    pub status_display: Entity,
    /// Panel entities for each wizard state
    pub panels: HashMap<WizardState, Entity>,
}

/// System to pre-allocate all wizard UI entities during initialization
/// 
/// Creates the complete wizard UI structure with all entities hidden initially.
/// This avoids expensive spawn/despawn operations during wizard usage.
/// 
/// # Performance Benefits
/// - All entities created once during setup
/// - No allocation overhead during wizard operation
/// - Visibility toggling is much faster than spawn/despawn
/// - Maintains component state across wizard sessions
pub fn setup_wizard_ui(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
) {
    // Create root modal entity with wizard root component
    let modal_root = commands.spawn((
        WizardRoot::new(),
        UiLayout::window()
            .size((Vw(80.0), Vh(70.0)))
            .pos((Vw(50.0), Vh(50.0)))
            .anchor(Anchor::Center)
            .pack(),
        UiColor::from(Color::srgba(0.1, 0.1, 0.15, 0.95)),
        Visibility::Hidden,
        Name::new("WizardModalRoot"),
    )).id();
    
    // Create backdrop overlay
    let backdrop = commands.spawn((
        UiLayout::window()
            .size((Vw(100.0), Vh(100.0)))
            .pos((Vw(0.0), Vh(0.0)))
            .pack(),
        UiColor::from(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        Visibility::Hidden,
        Name::new("WizardBackdrop"),
    )).id();
    
    // Create main modal window
    let modal_window = commands.spawn((
        UiLayout::window()
            .size((Rl(95.0), Rl(90.0)))
            .pos((Rl(50.0), Rl(50.0)))
            .anchor(Anchor::Center)
            .pack(),
        UiColor::from(Color::srgba(0.15, 0.15, 0.2, 1.0)),
        Visibility::Visible,
        Name::new("WizardModalWindow"),
    )).id();
    
    // Create panels for each wizard state
    let mut panels = HashMap::new();
    let wizard_states = [
        WizardState::Welcome,
        WizardState::CheckingPermissions,
        WizardState::RequestingPermissions,
        WizardState::SettingUpHotkeys,
        WizardState::Complete,
    ];
    
    for &state in &wizard_states {
        let panel = commands.spawn((
            WizardPanel::new(state),
            UiLayout::window()
                .size((Rl(90.0), Rl(80.0)))
                .pos((Rl(50.0), Rl(50.0)))
                .anchor(Anchor::Center)
                .pack(),
            UiColor::from(Color::srgba(0.2, 0.2, 0.25, 1.0)),
            Visibility::Hidden,
            Name::new(format!("WizardPanel_{:?}", state)),
        )).id();
        panels.insert(state, panel);
    }
    
    // Create permission cards for all supported permission types
    let mut permission_cards = HashMap::new();
    let permission_types = [
        (PermissionType::Accessibility, true),
        (PermissionType::ScreenCapture, true),
        (PermissionType::InputMonitoring, true),
        (PermissionType::Camera, false),
        (PermissionType::Microphone, false),
        (PermissionType::FullDiskAccess, false),
        (PermissionType::WiFi, false),
    ];
    
    for (permission_type, is_required) in permission_types {
        let card_entity = commands.spawn((
            PermissionCardBundle::new(permission_type, is_required),
            UiLayout::window()
                .size((Rl(28.0), Rl(20.0)))
                .pos((Rl(10.0), Rl(30.0)))
                .pack(),
            UiColor::from(Color::srgba(0.25, 0.25, 0.3, 1.0)),
            Interaction::None,
            Visibility::Hidden,
        )).id();
        permission_cards.insert(permission_type, card_entity);
    }
    
    // Create navigation buttons
    let mut navigation_buttons = HashMap::new();
    let button_actions = [
        NavigationAction::Back,
        NavigationAction::Next,
        NavigationAction::Skip,
        NavigationAction::Cancel,
        NavigationAction::Finish,
    ];
    
    for action in button_actions {
        let button_entity = commands.spawn((
            WizardNavigationButton::new(action),
            UiLayout::window()
                .size((Ab(120.0), Ab(40.0)))
                .pos((Rl(50.0), Rl(90.0)))
                .anchor(Anchor::BottomCenter)
                .pack(),
            UiColor::from(Color::srgba(0.3, 0.5, 0.8, 1.0)),
            Text::new(action.description()),
            Interaction::None,
            Visibility::Hidden,
            Name::new(format!("WizardButton_{:?}", action)),
        )).id();
        navigation_buttons.insert(action, button_entity);
    }
    
    // Create progress indicator
    let progress_indicator = commands.spawn((
        WizardProgressIndicator::default(),
        UiLayout::window()
            .size((Rl(60.0), Ab(30.0)))
            .pos((Rl(50.0), Rl(10.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        UiColor::from(Color::srgba(0.2, 0.4, 0.6, 1.0)),
        Text::new("Step 0 of 4"),
        Visibility::Hidden,
        Name::new("WizardProgressIndicator"),
    )).id();
    
    // Create status display
    let status_display = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(40.0)))
            .pos((Rl(50.0), Rl(25.0)))
            .anchor(Anchor::Center)
            .pack(),
        UiColor::from(Color::srgba(0.15, 0.15, 0.2, 0.8)),
        Text::new("Ready to begin setup"),
        Visibility::Hidden,
        Name::new("WizardStatusDisplay"),
    )).id();
    
    // Set up entity hierarchy
    commands.entity(modal_root).add_child(modal_window);
    commands.entity(modal_window).add_child(progress_indicator);
    commands.entity(modal_window).add_child(status_display);
    
    // Add panels to modal window
    for &panel in panels.values() {
        commands.entity(modal_window).add_child(panel);
    }
    
    // Add permission cards to the requesting permissions panel
    if let Some(&permissions_panel) = panels.get(&WizardState::RequestingPermissions) {
        for &card in permission_cards.values() {
            commands.entity(permissions_panel).add_child(card);
        }
    }
    
    // Add navigation buttons to modal window
    for &button in navigation_buttons.values() {
        commands.entity(modal_window).add_child(button);
    }
    
    // Store entities in resource for later access
    commands.insert_resource(WizardUIEntities {
        modal_root,
        modal_window,
        backdrop,
        permission_cards,
        navigation_buttons,
        progress_indicator,
        status_display,
        panels,
    });
    
    info!("Pre-allocated wizard UI entities for optimal performance");
}

/// System to show wizard UI when entering active states
pub fn show_wizard_ui(
    ui_entities: ResMut<WizardUIEntities>,
    mut visibility_query: Query<&mut Visibility>,
    wizard_state: Res<State<WizardState>>,
    mut last_state: Local<Option<WizardState>>,
) {
    let current_state = *wizard_state.get();
    
    // Only update when state changes
    if last_state.map(|s| s != current_state).unwrap_or(true) {
        // Show/hide backdrop and modal root based on wizard state
        let should_show = current_state.is_active();
        
        if let Ok(mut backdrop_vis) = visibility_query.get_mut(ui_entities.backdrop) {
            *backdrop_vis = if should_show { Visibility::Visible } else { Visibility::Hidden };
        }
        
        if let Ok(mut modal_vis) = visibility_query.get_mut(ui_entities.modal_root) {
            *modal_vis = if should_show { Visibility::Visible } else { Visibility::Hidden };
        }
        
        // Show/hide panels based on current state
        for (&state, &panel_entity) in &ui_entities.panels {
            if let Ok(mut panel_vis) = visibility_query.get_mut(panel_entity) {
                *panel_vis = if state == current_state { 
                    Visibility::Visible 
                } else { 
                    Visibility::Hidden 
                };
            }
        }
        
        // Show/hide permission cards for permission-related states
        let show_cards = matches!(current_state, 
            WizardState::CheckingPermissions | WizardState::RequestingPermissions);
        
        for &card_entity in ui_entities.permission_cards.values() {
            if let Ok(mut card_vis) = visibility_query.get_mut(card_entity) {
                *card_vis = if show_cards { Visibility::Visible } else { Visibility::Hidden };
            }
        }
        
        // Show/hide navigation buttons based on state
        for (&action, &button_entity) in &ui_entities.navigation_buttons {
            if let Ok(mut button_vis) = visibility_query.get_mut(button_entity) {
                let should_show_button = match (action, current_state) {
                    (NavigationAction::Back, WizardState::Welcome) => false,
                    (NavigationAction::Next, WizardState::Complete) => false,
                    (NavigationAction::Finish, WizardState::Complete) => true,
                    (NavigationAction::Finish, _) => false,
                    (_, state) if state.is_active() => true,
                    _ => false,
                };
                
                *button_vis = if should_show_button { 
                    Visibility::Visible 
                } else { 
                    Visibility::Hidden 
                };
            }
        }
        
        // Show progress indicator and status display for active states
        let show_ui_elements = current_state.is_active();
        
        if let Ok(mut progress_vis) = visibility_query.get_mut(ui_entities.progress_indicator) {
            *progress_vis = if show_ui_elements { Visibility::Visible } else { Visibility::Hidden };
        }
        
        if let Ok(mut status_vis) = visibility_query.get_mut(ui_entities.status_display) {
            *status_vis = if show_ui_elements { Visibility::Visible } else { Visibility::Hidden };
        }
        
        *last_state = Some(current_state);
        
        if should_show {
            info!("Showed wizard UI for state: {:?}", current_state);
        } else {
            info!("Hidden wizard UI");
        }
    }
}

/// System to cleanup wizard UI entities when exiting
pub fn cleanup_wizard_ui(
    mut commands: Commands,
    ui_entities: Option<Res<WizardUIEntities>>,
) {
    if let Some(entities) = ui_entities {
        // Despawn all wizard entities
        commands.entity(entities.modal_root).despawn();
        commands.entity(entities.backdrop).despawn();
        
        // Remove the resource
        commands.remove_resource::<WizardUIEntities>();
        
        info!("Cleaned up wizard UI entities");
    }
}
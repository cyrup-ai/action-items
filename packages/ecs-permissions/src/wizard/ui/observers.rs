//! Wizard UI Observers
//!
//! High-performance observer system for wizard UI events with zero allocation
//! and efficient entity lifecycle management.

use bevy::prelude::*;
use crate::wizard::events::PermissionStatusExt;

/// System to register wizard observers for UI events
pub fn register_wizard_observers(
    mut commands: Commands,
) {
    // Register observers for wizard UI events
    commands.spawn((
        Observer::new(handle_wizard_button_click),
        Observer::new(handle_permission_card_click),
        Observer::new(handle_wizard_modal_close),
        Name::new("WizardUIObservers"),
    ));
    
    info!("Registered wizard UI observers");
}

/// Observer function for wizard button clicks
fn handle_wizard_button_click(
    trigger: Trigger<Pointer<Click>>,
    button_query: Query<&crate::wizard::ui::WizardButton>,
    mut navigation_events: EventWriter<crate::wizard::WizardNavigationRequest>,
    mut cancel_events: EventWriter<crate::wizard::WizardCancelRequest>,
) {
    let entity = trigger.target();
    
    if let Ok(button) = button_query.get(entity) {
        match button.action {
            crate::wizard::WizardAction::Next => {
                navigation_events.write(crate::wizard::WizardNavigationRequest::next());
            },
            crate::wizard::WizardAction::Back => {
                navigation_events.write(crate::wizard::WizardNavigationRequest::back());
            },
            crate::wizard::WizardAction::Skip => {
                cancel_events.write(crate::wizard::WizardCancelRequest::user_skipped());
            },
            crate::wizard::WizardAction::Cancel => {
                cancel_events.write(crate::wizard::WizardCancelRequest::user_canceled());
            },
        }
        
        debug!("Handled wizard button click: {:?}", button.action);
    }
}

/// Observer function for permission card clicks
fn handle_permission_card_click(
    trigger: Trigger<Pointer<Click>>,
    card_query: Query<&crate::wizard::PermissionCard>,
    mut permission_events: EventWriter<crate::wizard::WizardPermissionRequest>,
) {
    let entity = trigger.target();
    
    if let Ok(card) = card_query.get(entity) {
        // Only allow interaction if card is interactive and permission is not already granted
        if card.is_interactive && !card.status.is_granted() {
            permission_events.write(crate::wizard::WizardPermissionRequest::new(card.permission_type));
            debug!("Handled permission card click: {:?}", card.permission_type);
        }
    }
}

/// Observer function for wizard modal close events
fn handle_wizard_modal_close(
    trigger: Trigger<Pointer<Click>>,
    modal_query: Query<&crate::wizard::ui::WizardModalWindow>,
    mut cancel_events: EventWriter<crate::wizard::WizardCancelRequest>,
) {
    let entity = trigger.target();
    
    // Check if this is a backdrop click (close modal)
    if modal_query.get(entity).is_ok() {
        cancel_events.write(crate::wizard::WizardCancelRequest::user_canceled());
        debug!("Handled wizard modal backdrop click - canceling wizard");
    }
}

/// System to handle automatic entity removal for temporary UI elements
pub fn handle_auto_remove_entities(
    mut commands: Commands,
    auto_remove_query: Query<(Entity, &crate::wizard::ui::WizardAnimationState), With<AutoRemove>>,
) {
    for (entity, animation_state) in auto_remove_query.iter() {
        // Remove entities that have completed their exit animation
        if !animation_state.is_animated_in && animation_state.animation_progress <= 0.0 {
            commands.entity(entity).despawn();
            debug!("Auto-removed wizard UI entity: {:?}", entity);
        }
    }
}

/// Marker component for entities that should be automatically removed
#[derive(Component)]
pub struct AutoRemove;
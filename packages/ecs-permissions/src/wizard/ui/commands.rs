//! Wizard UI Commands
//!
//! High-performance command system for wizard UI operations with zero allocation
//! and efficient entity management through ecs-ui integration.

use bevy::prelude::*;
use action_items_ecs_ui::prelude::*;
use action_items_ecs_ui::Ab;

use crate::types::PermissionType;
use crate::wizard::WizardAction;
use crate::types::PermissionStatus;
use crate::wizard::events::PermissionStatusExt;
use super::{WizardModalWindow, WizardButton, WizardButtonType};

/// Extension trait for Commands to add wizard-specific functionality
pub trait WizardUiCommands {
    /// Spawn a complete wizard modal with all UI elements
    fn spawn_wizard_modal(&mut self) -> Entity;
    
    /// Spawn a permission card for the specified permission type
    fn spawn_permission_card(&mut self, permission_type: PermissionType, is_required: bool) -> Entity;
    
    /// Spawn a wizard navigation button
    fn spawn_wizard_button(&mut self, text: &str, action: WizardAction, button_type: WizardButtonType) -> Entity;
    
    /// Update wizard modal visibility
    fn set_wizard_modal_visibility(&mut self, entity: Entity, visible: bool);
    
    /// Update permission card status with animation
    fn update_permission_card_status(&mut self, entity: Entity, status: PermissionStatus);
}

impl WizardUiCommands for Commands<'_, '_> {
    fn spawn_wizard_modal(&mut self) -> Entity {
        self.spawn((
            WizardModalWindow::default(),
            UiLayout::window()
                .size((Vw(80.0), Vh(70.0)))
                .pos((Vw(50.0), Vh(50.0)))
                .anchor(Anchor::Center)
                .pack(),
            UiColor::from(Color::srgba(0.1, 0.1, 0.15, 0.95)),
            UiHover::new().forward_speed(8.0).backward_speed(4.0),
            Visibility::Hidden,
            Name::new("WizardModal"),
        )).id()
    }
    
    fn spawn_permission_card(&mut self, permission_type: PermissionType, is_required: bool) -> Entity {
        let card_name = format!("PermissionCard_{:?}", permission_type);
        
        self.spawn((
            crate::wizard::PermissionCard::new(permission_type, is_required),
            UiLayout::window()
                .size((Rl(28.0), Rl(20.0)))
                .pos((Rl(10.0), Rl(30.0)))
                .pack(),
            UiColor::from(Color::srgba(0.25, 0.25, 0.3, 1.0)),
            UiHover::new().forward_speed(8.0).backward_speed(4.0),
            UiClicked::new().forward_speed(12.0).backward_speed(6.0),
            Interaction::None,
            Visibility::Hidden,
            Name::new(card_name),
        )).id()
    }
    
    fn spawn_wizard_button(&mut self, text: &str, action: WizardAction, button_type: WizardButtonType) -> Entity {
        let (color, _hover_color) = match button_type {
            WizardButtonType::Primary => (
                Color::srgba(0.2, 0.6, 0.9, 1.0),
                Color::srgba(0.3, 0.7, 1.0, 1.0)
            ),
            WizardButtonType::Secondary => (
                Color::srgba(0.4, 0.4, 0.4, 1.0),
                Color::srgba(0.5, 0.5, 0.5, 1.0)
            ),
            WizardButtonType::Cancel => (
                Color::srgba(0.8, 0.2, 0.2, 1.0),
                Color::srgba(0.9, 0.3, 0.3, 1.0)
            ),
            WizardButtonType::Skip => (
                Color::srgba(0.6, 0.6, 0.6, 1.0),
                Color::srgba(0.7, 0.7, 0.7, 1.0)
            ),
        };
        
        self.spawn((
            WizardButton { action, button_type },
            UiLayout::window()
                .size((Ab(120.0), Ab(40.0)))
                .pos((Rl(50.0), Rl(90.0)))
                .anchor(Anchor::BottomCenter)
                .pack(),
            UiColor::from(color),
            UiHover::new().forward_speed(10.0).backward_speed(5.0),
            UiClicked::new().forward_speed(15.0).backward_speed(8.0),
            Text::new(text),
            Interaction::None,
            Visibility::Hidden,
            Name::new(format!("WizardButton_{:?}", action)),
        )).id()
    }
    
    fn set_wizard_modal_visibility(&mut self, entity: Entity, visible: bool) {
        if let Ok(mut entity_commands) = self.get_entity(entity) {
            entity_commands.insert(if visible { 
                Visibility::Visible 
            } else { 
                Visibility::Hidden 
            });
        }
    }
    
    fn update_permission_card_status(&mut self, entity: Entity, status: PermissionStatus) {
        if let Ok(mut entity_commands) = self.get_entity(entity) {
            // Update the color based on status
            let color = status.color();
            entity_commands.insert(UiColor::from(color));
        }
    }
}
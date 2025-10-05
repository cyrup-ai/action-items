//! Component lifecycle hooks for accessibility

use bevy::prelude::*;
use bevy::ecs::component::HookContext;
use super::components::FocusableElement;
use super::manager::AccessibilityManager;

/// Register component hooks for automatic accessibility behavior
pub fn register_accessibility_hooks(world: &mut World) {
    world
        .register_component_hooks::<FocusableElement>()
        .on_add(|mut world, HookContext { entity, .. }| {
            // Auto-focus first focusable element if none currently focused
            let should_focus = {
                let manager = world.resource::<AccessibilityManager>();
                manager.focused_element.is_none()
            };
            
            if should_focus {
                if let Some(mut focusable) = world.get_mut::<FocusableElement>(entity) {
                    focusable.focused = true;
                }
                world.resource_mut::<AccessibilityManager>().focused_element = Some(entity);
            }
        });
}

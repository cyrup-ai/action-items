//! ECS systems for menu management

use bevy::prelude::*;
use muda::MenuEvent;
use crate::{events::*, resources::*};

/// Poll muda menu events and convert to Bevy events
/// This system runs every frame and checks for menu clicks
pub fn poll_menu_events<T: Event + Clone + Send + Sync + 'static>(
    resource: Res<MenuResource<T>>,
    mut clicked_events: EventWriter<MenuItemClicked<T>>,
) {
    // Poll muda's event receiver
    // Note: muda uses crossbeam_channel, so try_recv() is non-blocking
    while let Ok(muda_event) = MenuEvent::receiver().try_recv() {
        tracing::debug!("Menu event received: {:?}", muda_event.id);
        
        // Look up the app event for this MenuId
        if let Some(app_event) = resource.event_map.get(&muda_event.id) {
            // Emit Bevy event
            // Note: muda::MenuEvent does not provide modifier key information
            // ModifierKeys remain default until muda library adds this feature
            clicked_events.write(MenuItemClicked {
                item: app_event.clone(),
                modifier_keys: ModifierKeys::default(),
            });
            tracing::info!("Menu item clicked, emitted app event");
        } else {
            tracing::warn!("Received menu event for unknown MenuId: {:?}", muda_event.id);
        }
    }
}

/// Update menu item enabled/disabled state
pub fn update_menu_item_enabled<T: Clone + Send + Sync + 'static>(
    mut events: EventReader<MenuItemSetEnabled>,
    _resource: Res<MenuResource<T>>,
) {
    for event in events.read() {
        crate::resources::MENU_ITEMS.with(|items| {
            if let Some(item) = items.borrow().get(&event.item_id) {
                item.set_enabled(event.enabled);
                tracing::debug!("Set menu item '{}' enabled={}", event.item_id, event.enabled);
            } else {
                tracing::warn!("Menu item '{}' not found", event.item_id);
            }
        });
    }
}

/// Update menu item checked state
pub fn update_menu_item_checked<T: Clone + Send + Sync + 'static>(
    mut events: EventReader<MenuItemSetChecked>,
    _resource: Res<MenuResource<T>>,
) {
    for event in events.read() {
        crate::resources::CHECK_MENU_ITEMS.with(|items| {
            if let Some(item) = items.borrow().get(&event.item_id) {
                item.set_checked(event.checked);
                tracing::debug!("Set menu item '{}' checked={}", event.item_id, event.checked);
            } else {
                tracing::warn!("Check menu item '{}' not found", event.item_id);
            }
        });
    }
}

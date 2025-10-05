//! Menu event types for ECS integration

use bevy::prelude::*;

/// Menu item was clicked
/// Generic over app-specific menu event type
#[derive(Event, Debug, Clone)]
pub struct MenuItemClicked<T: Clone + Send + Sync + 'static> {
    pub item: T,
    pub modifier_keys: ModifierKeys,
}

/// Request to update menu item enabled state
#[derive(Event, Debug, Clone)]
pub struct MenuItemSetEnabled {
    /// String identifier for the menu item (e.g., "save", "new_doc")
    pub item_id: String,
    pub enabled: bool,
}

/// Request to update menu item checked state (for CheckMenuItem)
#[derive(Event, Debug, Clone)]
pub struct MenuItemSetChecked {
    pub item_id: String,
    pub checked: bool,
}

/// Modifier keys pressed during menu interaction
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierKeys {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

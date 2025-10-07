use bevy::prelude::*;
use crate::navigation::{SettingsTab, ExtensionFilter};

/// Settings state management resource
#[derive(Resource, Default)]
pub struct SettingsResource {
    /// Currently active tab
    pub current_tab: SettingsTab,
    
    /// Settings visibility state
    pub is_visible: bool,
    
    /// Active search filter
    pub search_query: String,
    
    /// Current extension filter
    pub extensions_filter: ExtensionFilter,
    
    /// Selected extension ID
    pub selected_extension: Option<String>,
    
    /// Dirty state - unsaved changes
    pub has_unsaved_changes: bool,
}

impl SettingsResource {
    /// Check if a specific tab is currently active
    pub fn is_tab_active(&self, tab: SettingsTab) -> bool {
        self.current_tab == tab
    }
    
    /// Set active tab
    pub fn set_tab(&mut self, tab: SettingsTab) {
        if self.current_tab != tab {
            self.current_tab = tab;
            self.has_unsaved_changes = true;
        }
    }
    
    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }
    
    /// Toggle visibility
    pub fn toggle_visible(&mut self) {
        self.is_visible = !self.is_visible;
    }
}


use std::collections::HashMap;

/// Pre-allocated UI entity infrastructure for zero-allocation tab switching
#[derive(Resource)]
pub struct SettingsUIEntities {
    pub backdrop: Entity,
    pub modal_root: Entity,
    pub title_bar: Entity,
    pub close_button: Entity,
    pub sidebar: Entity,
    pub content_area: Entity,
    pub tab_buttons: HashMap<SettingsTab, Entity>,
    pub tab_panels: HashMap<SettingsTab, Entity>,
}

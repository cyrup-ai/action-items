// UI Components Module - Public API and Exports

pub mod action_buttons;
pub mod layout;
pub mod modal;
pub mod result_list;
pub mod search_bar;
pub mod search_ui_state;
pub mod status_bar;
pub mod traits;

// Re-export all public components and types
pub use action_buttons::*;
// Re-export shared resources and state
use bevy::prelude::*;
pub use layout::*;
pub use modal::*;
pub use result_list::*;
pub use search_bar::*;
pub use search_ui_state::*;
pub use status_bar::*;
pub use traits::*;

/// UI state management resource
#[derive(Resource, Default)]
pub struct UiState {
    pub query: String,
    pub results: Vec<action_items_core::ActionItem>,
    pub selected_index: usize,
    pub visible: bool,
    // UI entity references
    pub container_entity: Option<Entity>,
    pub input_entity: Option<Entity>,
    pub results_container: Option<Entity>,
}

/// UI fonts resource
#[derive(Resource)]
pub struct UiFonts {
    pub regular: Handle<Font>,
    pub medium: Handle<Font>,
    pub bold: Handle<Font>,
    pub monospace: Handle<Font>,
    pub mono: Handle<Font>,
    pub icons: Handle<Font>,
    pub ubuntu_medium: Handle<Font>,
    pub ubuntu_bold: Handle<Font>,
}

// Public function to update UI visibility from main app
/// Component marker for settings containers
#[derive(Component)]
pub struct SettingsContainer;

/// Component marker for privacy indicator panels
#[derive(Component)]
pub struct PrivacyIndicatorPanel;

pub fn set_ui_visibility(mut ui_state: ResMut<UiState>, visible: bool) {
    ui_state.visible = visible;
    if !visible {
        ui_state.query.clear();
        ui_state.results.clear();
        ui_state.selected_index = 0;
    }
}

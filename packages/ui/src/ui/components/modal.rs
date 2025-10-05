use std::time::Duration;

use bevy::prelude::*;

use super::traits::*;
use action_items_ecs_ui::{
    UiAnimationCompleteEvent, UiComponentTarget, UiVisibilityEvent,
};

#[derive(Component)]
pub struct PreferencesContainer;

#[derive(Component)]
pub struct AlternativeHotkeysContainer;

/// Modal state management
#[derive(Resource, Default)]
pub struct ModalState {
    pub is_open: bool,
    pub modal_type: ModalType,
    pub backdrop_visible: bool,
}

/// Types of modal dialogs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalType {
    None,
    Preferences,
    HotkeySettings,
    About,
    Error,
}

impl Default for ModalType {
    fn default() -> Self {
        Self::None
    }
}

impl ModalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_modal(
        &mut self,
        modal_type: ModalType,
        ui_events: &mut EventWriter<UiVisibilityEvent>,
    ) {
        // Update state immediately for show animations (needed for animation to work)
        self.is_open = true;
        self.modal_type = modal_type;
        self.backdrop_visible = true;
        // Send animated show event for modal component
        ui_events.write(UiVisibilityEvent::animated(
            true,
            Duration::from_millis(200),
            UiComponentTarget::Dialog,
        ));
    }

    pub fn close_modal(&mut self, ui_events: &mut EventWriter<UiVisibilityEvent>) {
        // Send animated hide event first for modal component
        ui_events.write(UiVisibilityEvent::animated(
            false,
            Duration::from_millis(150),
            UiComponentTarget::Dialog,
        ));
        // DEFER state updates until animation completes - do not update state immediately
        // Animation completion will be handled by a callback system or completion event
    }

    pub fn is_preferences_open(&self) -> bool {
        self.is_open && matches!(self.modal_type, ModalType::Preferences)
    }

    pub fn is_hotkey_settings_open(&self) -> bool {
        self.is_open && matches!(self.modal_type, ModalType::HotkeySettings)
    }
}

impl ContentDisplay for ModalState {
    type Content = ModalType;

    fn set_content(&mut self, content: Self::Content) {
        self.modal_type = content;
        self.is_open = content != ModalType::None;
        self.backdrop_visible = self.is_open;
    }

    fn content(&self) -> &Self::Content {
        &self.modal_type
    }

    fn clear_content(&mut self) {
        // Note: This trait method cannot access EventWriter, so it uses immediate close
        // For animated closing, call close_modal directly with EventWriter
        self.is_open = false;
        self.modal_type = ModalType::None;
        self.backdrop_visible = false;
    }
}

/// Handle modal animation completion events to update state after animations finish
pub fn handle_modal_animation_completion(
    mut completion_events: EventReader<UiAnimationCompleteEvent>,
    mut modal_state: ResMut<ModalState>,
) {
    for event in completion_events.read() {
        // Only handle modal completion events
        if event.target == UiComponentTarget::Dialog && !event.was_show {
            // Animation was a hide operation that just completed - now update state
            modal_state.is_open = false;
            modal_state.modal_type = ModalType::None;
            modal_state.backdrop_visible = false;
        }
    }
}

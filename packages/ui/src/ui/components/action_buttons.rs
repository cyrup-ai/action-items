use bevy::prelude::*;

use super::traits::*;

#[derive(Component)]
pub struct TestHotkeyButton;

#[derive(Component)]
pub struct ApplyHotkeyButton;

#[derive(Component)]
pub struct PreferencesCloseButton;

/// Button state for interaction tracking
#[derive(Component, Debug, Clone, Default)]
pub struct ButtonState {
    pub is_pressed: bool,
    pub is_hovered: bool,
    pub is_disabled: bool,
}

impl ButtonState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.is_disabled = disabled;
    }

    pub fn is_interactive(&self) -> bool {
        !self.is_disabled
    }
}

impl Interactive for ButtonState {
    fn set_interaction_state(&mut self, state: InteractionState) {
        match state {
            InteractionState::Default => {
                self.is_pressed = false;
                self.is_hovered = false;
                self.is_disabled = false;
            },
            InteractionState::Hover => {
                self.is_hovered = true;
                self.is_pressed = false;
            },
            InteractionState::Pressed => {
                self.is_pressed = true;
                self.is_hovered = false;
            },
            InteractionState::Disabled => {
                self.is_disabled = true;
                self.is_pressed = false;
                self.is_hovered = false;
            },
            InteractionState::Selected => {
                // For buttons, selected is similar to pressed
                self.is_pressed = true;
                self.is_hovered = false;
            },
        }
    }

    fn interaction_state(&self) -> InteractionState {
        if self.is_disabled {
            InteractionState::Disabled
        } else if self.is_pressed {
            InteractionState::Pressed
        } else if self.is_hovered {
            InteractionState::Hover
        } else {
            InteractionState::Default
        }
    }
}

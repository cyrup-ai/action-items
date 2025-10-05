use std::sync::Arc;

use bevy::prelude::*;
use lasso::{Spur, ThreadedRodeo};

/// Resource for managing input focus in the launcher with zero-allocation string handling
#[derive(Resource, Debug, Clone)]
pub struct InputFocus {
    /// Currently focused entity
    pub focused_entity: Option<Entity>,
    /// Whether focus indicator should be visible (keyboard vs mouse interaction)
    pub focus_visible: bool,
    /// String interner for zero-allocation text handling
    pub string_interner: Arc<ThreadedRodeo>,
    /// Pre-interned common strings for maximum performance
    pub placeholder_key: Spur,
    pub empty_string_key: Spur,
}

impl Default for InputFocus {
    fn default() -> Self {
        let interner = Arc::new(ThreadedRodeo::default());

        // Pre-intern critical strings for zero allocation during runtime
        let placeholder_key = interner.get_or_intern("Type to search...");
        let empty_string_key = interner.get_or_intern("");

        Self {
            focused_entity: None,
            focus_visible: false,
            string_interner: interner,
            placeholder_key,
            empty_string_key,
        }
    }
}

impl InputFocus {
    /// Set focus to a specific entity with validation
    #[inline]
    pub fn set_focus(&mut self, entity: Entity) {
        if self.focused_entity != Some(entity) {
            self.focused_entity = Some(entity);
        }
    }

    /// Clear current focus
    #[inline]
    pub fn clear_focus(&mut self) {
        self.focused_entity = None;
        self.focus_visible = false;
    }

    /// Check if a specific entity has focus
    #[inline]
    pub fn is_focused(&self, entity: Entity) -> bool {
        self.focused_entity == Some(entity)
    }

    /// Show focus indicator (typically for keyboard navigation)
    #[inline]
    pub fn show_focus_indicator(&mut self) {
        self.focus_visible = true;
    }

    /// Hide focus indicator (typically for mouse interaction)
    #[inline]
    pub fn hide_focus_indicator(&mut self) {
        self.focus_visible = false;
    }

    /// Get or intern a string with zero allocation for duplicates
    #[inline]
    pub fn intern_string(&self, text: &str) -> Spur {
        self.string_interner.get_or_intern(text)
    }

    /// Resolve an interned string key back to text
    #[inline]
    pub fn resolve_string(&self, key: &Spur) -> &str {
        self.string_interner.resolve(key)
    }

    /// Check if the interner contains a specific string
    #[inline]
    #[allow(dead_code)]
    pub fn contains_string(&self, text: &str) -> Option<Spur> {
        self.string_interner.get(text)
    }

    /// Get the placeholder text key for efficient reuse
    #[inline]
    pub fn placeholder_text_key(&self) -> Spur {
        self.placeholder_key
    }

    /// Get the empty string key for efficient reuse
    #[inline]
    pub fn empty_text_key(&self) -> Spur {
        self.empty_string_key
    }
}

/// Component marker for entities that can receive input focus
#[derive(Component, Debug, Default)]
pub struct Focusable {
    /// Tab order for keyboard navigation (lower values focused first)
    pub tab_index: u32,
    /// Whether this element can receive focus via keyboard navigation
    pub keyboard_focusable: bool,
    /// Whether this element can receive focus via mouse interaction
    pub mouse_focusable: bool,
}

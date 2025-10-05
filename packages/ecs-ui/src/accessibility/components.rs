//! ARIA-compliant accessibility components
//!
//! Based on WCAG 2.1 and ARIA 1.2 specifications

use accesskit::Role;
use bevy::prelude::*;

/// Component for managing accessibility state per ARIA specification
/// 
/// See: https://www.w3.org/TR/wai-aria-1.2/
#[derive(Component, Debug)]
pub struct AccessibleElement {
    /// ARIA role for the element
    pub role: Role,
    /// Accessible name/label
    pub name: String,
    /// Accessible description
    pub description: Option<String>,
    /// Whether the element is focusable
    pub focusable: bool,
    /// Current focus state
    pub focused: bool,
    /// Tab index for keyboard navigation
    pub tab_index: Option<i32>,
    /// ARIA live region type for dynamic content
    pub live_region: Option<LiveRegion>,
}

impl Default for AccessibleElement {
    fn default() -> Self {
        Self {
            role: Role::GenericContainer,
            name: String::new(),
            description: None,
            focusable: false,
            focused: false,
            tab_index: None,
            live_region: None,
        }
    }
}

/// ARIA live region types for dynamic content updates
/// 
/// See: https://www.w3.org/TR/wai-aria-1.2/#aria-live
#[derive(Debug, Clone, Copy)]
pub enum LiveRegion {
    /// Polite updates (announced when user is idle)
    Polite,
    /// Assertive updates (announced immediately)
    Assertive,
    /// Off (no announcements)
    Off,
}

/// Component for keyboard focus management
#[derive(Component, Debug)]
pub struct FocusableElement {
    /// Focus order in tab sequence
    pub tab_order: u32,
    /// Whether currently focused
    pub focused: bool,
    /// Focus visual style
    pub focus_style: FocusStyle,
}

impl Default for FocusableElement {
    fn default() -> Self {
        Self {
            tab_order: 0,
            focused: false,
            focus_style: FocusStyle::Outline,
        }
    }
}

/// Visual focus indication styles per WCAG 2.4.7 Focus Visible
/// 
/// See: https://www.w3.org/WAI/WCAG21/Understanding/focus-visible.html
#[derive(Debug, Clone, Copy)]
pub enum FocusStyle {
    /// Outline border (default, meets WCAG 2.4.7)
    Outline,
    /// Background color change
    Background,
    /// Scale animation
    Scale,
    /// Combined effects (highest visibility)
    Combined,
}

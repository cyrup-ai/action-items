//! Accessibility plugin for Bevy applications
//!
//! Provides WCAG 2.1 Level AA compliant accessibility features:
//! - Keyboard navigation (Tab/Shift+Tab)
//! - Screen reader support
//! - Focus management
//! - High contrast mode
//! - Reduced motion support
//! - Platform-specific accessibility detection

use bevy::prelude::*;
use super::manager::AccessibilityManager;
use super::events::{FocusChanged, ScreenReaderAnnouncement};
use super::navigation::handle_accessibility_navigation;
use super::visuals::{update_focus_visuals, apply_high_contrast_styles};
use super::detection::{detect_accessibility_preferences, AccessibilityDetector};
use super::hooks::register_accessibility_hooks;

pub struct AccessibilityPlugin;

impl Plugin for AccessibilityPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<AccessibilityManager>();
        
        // Try to initialize platform detector (may fail on some platforms)
        if let Ok(detector) = AccessibilityDetector::new() {
            app.insert_resource(detector);
        }
        
        // Register events
        app.add_event::<FocusChanged>()
            .add_event::<ScreenReaderAnnouncement>();
        
        // Add systems
        app.add_systems(Update, (
            handle_accessibility_navigation,
            update_focus_visuals,
            apply_high_contrast_styles,
            detect_accessibility_preferences,
        ));
        
        // Setup hooks
        app.add_systems(Startup, |world: &mut World| {
            register_accessibility_hooks(world);
        });
    }
}

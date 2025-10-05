//! Generic accessibility system for Bevy applications
//!
//! Provides WCAG 2.1 compliant accessibility patterns including:
//! - ARIA-compliant components
//! - Keyboard navigation
//! - Screen reader support  
//! - Platform accessibility detection
//! - Focus management and visual indicators

pub mod components;
pub mod manager;
pub mod navigation;
pub mod visuals;
pub mod detection;
pub mod events;
pub mod hooks;
pub mod plugin;

// Re-export main types
pub use components::{AccessibleElement, FocusableElement, LiveRegion, FocusStyle};
pub use manager::AccessibilityManager;
pub use events::{FocusChanged, ScreenReaderAnnouncement, AnnouncementPriority};
pub use detection::{AccessibilityDetector, AccessibilityState};
pub use plugin::AccessibilityPlugin;

// Re-export systems for advanced usage
pub use navigation::{handle_accessibility_navigation, navigate_focus, clear_focus};
pub use visuals::{update_focus_visuals, apply_high_contrast_styles};
pub use detection::detect_accessibility_preferences;

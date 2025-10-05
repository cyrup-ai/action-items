//! Accessibility events for reactive programming

use bevy::prelude::*;

/// Event fired when keyboard focus changes between elements
#[derive(Event, Debug, Clone)]
pub struct FocusChanged {
    /// Previously focused entity (if any)
    pub old_focus: Option<Entity>,
    /// Newly focused entity (if any)
    pub new_focus: Option<Entity>,
}

/// Event for queuing screen reader announcements
/// 
/// Follows ARIA live region semantics
#[derive(Event, Debug, Clone)]
pub struct ScreenReaderAnnouncement {
    /// Message to announce
    pub message: String,
    /// Announcement priority/interruption level
    pub priority: AnnouncementPriority,
}

/// Screen reader announcement priority levels
#[derive(Debug, Clone, Copy)]
pub enum AnnouncementPriority {
    /// Polite: announce when user is idle (aria-live="polite")
    Polite,
    /// Assertive: announce immediately (aria-live="assertive")
    Assertive,
}

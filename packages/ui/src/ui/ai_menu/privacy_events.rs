//! Privacy indicator events for AI menu system
//!
//! Zero-allocation event types for privacy status changes with blazing-fast propagation.

use bevy::prelude::*;

/// Event triggered when privacy configuration changes
/// Enables real-time updates to privacy indicators
#[derive(Event, Debug, Clone)]
pub struct PrivacyStatusChanged {
    /// Full user control over AI interactions - accessed in event handlers
    pub full_control: bool,
    /// No data collection by AI providers - accessed in event handlers
    pub no_collection: bool,
    /// End-to-end encryption active - accessed in event handlers
    pub encrypted: bool,
    /// Timestamp of the change for audit logging - accessed in event handlers
    pub timestamp: std::time::Instant,
}

impl PrivacyStatusChanged {
    /// Create new privacy status change event with current timestamp
    #[inline]
    pub fn new(full_control: bool, no_collection: bool, encrypted: bool) -> Self {
        Self {
            full_control,
            no_collection,
            encrypted,
            timestamp: std::time::Instant::now(),
        }
    }
}

/// Event for toggling info panel expansion  
#[derive(Event, Debug)]
pub struct TogglePrivacyInfo {
    /// Entity of the privacy indicators container - used for UI targeting in systems
    pub container_entity: Entity,
    /// Whether to expand (true) or collapse (false)
    pub expand: bool,
}

/// Event for privacy indicator hover state changes
#[derive(Event, Debug)]
pub struct PrivacyIndicatorHover {
    /// Type of indicator being hovered - processed in hover systems
    pub indicator_type: IndicatorType,
    /// Entity of the hovered indicator - used for UI targeting
    pub entity: Entity,
    /// Whether entering (true) or leaving (false) hover - drives state changes
    pub hovering: bool,
}

/// Types of privacy indicators for efficient event routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum IndicatorType {
    /// Full control indicator (minus icon)
    FullControl,
    /// No collection indicator (lock icon)  
    NoCollection,
    /// Encrypted indicator (shield icon)
    Encrypted,
    /// Info details expansion button
    InfoDetails,
}

//! Performance optimization components

use bevy::prelude::*;

/// Component for entities that support virtualization
///
/// Entities with this component can be hidden when not visible
/// or when their priority is below threshold, improving performance.
#[derive(Component, Debug)]
pub struct VirtualizedEntity {
    /// Whether this entity should be visible
    pub visible: bool,
    /// Priority level (0-255). Entities with priority < 5 may be culled.
    pub priority: u8,
}

impl Default for VirtualizedEntity {
    fn default() -> Self {
        Self {
            visible: true,
            priority: 0,
        }
    }
}

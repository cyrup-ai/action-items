use bevy::prelude::*;
use bevy::tasks::Task;

use super::metadata::ActionItem;

/// Component to track an in-progress search task for a specific plugin.
#[derive(Component)]
pub struct PendingActionResult {
    pub plugin_id: String,
    pub task: Task<crate::error::Result<Vec<ActionItem>>>,
}

use std::time::Instant;

use bevy::prelude::*;
use bevy::tasks::Task;

use crate::types::{SearchError, SearchId, SearchResult};

/// Component for tracking plugin search tasks - following async_compute.rs pattern
#[derive(Component)]
pub struct PluginSearchTask {
    pub search_id: SearchId,
    pub plugin_id: String,
    pub task: Task<Result<Vec<SearchResult>, SearchError>>,
    pub started_at: Instant,
}

impl PluginSearchTask {
    pub fn new(
        search_id: SearchId,
        plugin_id: String,
        task: Task<Result<Vec<SearchResult>, SearchError>>,
    ) -> Self {
        Self {
            search_id,
            plugin_id,
            task,
            started_at: Instant::now(),
        }
    }

    pub fn execution_time_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }
}

/// Component for search timeout tracking
#[derive(Component)]
pub struct SearchTimeout {
    pub search_id: SearchId,
    pub deadline: Instant,
}

impl SearchTimeout {
    pub fn new(search_id: SearchId, timeout_duration: std::time::Duration) -> Self {
        Self {
            search_id,
            deadline: Instant::now() + timeout_duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.deadline
    }
}

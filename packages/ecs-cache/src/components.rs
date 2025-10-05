use std::time::Instant;

use bevy::prelude::*;
use bevy::tasks::Task;
use goldylox::prelude::CacheOperationError;

use crate::events::{CacheKey, CacheOperationId, CachePartition, CacheValue};

/// Component for tracking ongoing cache operations
#[derive(Component)]
pub struct CacheOperation {
    pub operation_id: CacheOperationId,
    pub partition: CachePartition,
    pub key: CacheKey,
    pub operation_type: CacheOperationType,
    pub started_at: Instant,
    pub requester: String,
}

impl CacheOperation {
    pub fn new(
        operation_id: CacheOperationId,
        partition: impl Into<String>,
        key: impl Into<String>,
        operation_type: CacheOperationType,
        requester: impl Into<String>,
    ) -> Self {
        Self {
            operation_id,
            partition: partition.into(),
            key: key.into(),
            operation_type,
            started_at: Instant::now(),
            requester: requester.into(),
        }
    }

    pub fn execution_time_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }
}

/// Types of cache operations
#[derive(Debug, Clone)]
pub enum CacheOperationType {
    Read,
    Write {
        value: CacheValue,
        ttl_seconds: Option<u64>,
    },
    Invalidate,
    Warmup {
        keys: Vec<CacheKey>,
    },
}

/// Component for background cache warming tasks
#[derive(Component)]
pub struct CacheWarmupTask {
    pub operation_id: CacheOperationId,
    pub partition: CachePartition,
    pub task: Task<Result<Vec<(CacheKey, CacheValue)>, CacheOperationError>>,
    pub started_at: Instant,
    pub requester: String,
}

impl CacheWarmupTask {
    pub fn new(
        operation_id: CacheOperationId,
        partition: impl Into<String>,
        task: Task<Result<Vec<(CacheKey, CacheValue)>, CacheOperationError>>,
        requester: impl Into<String>,
    ) -> Self {
        Self {
            operation_id,
            partition: partition.into(),
            task,
            started_at: Instant::now(),
            requester: requester.into(),
        }
    }

    pub fn execution_time_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }
}

/// Component for cache eviction monitoring
#[derive(Component)]
pub struct CacheEvictionMonitor {
    pub partition: CachePartition,
    pub last_check: Instant,
    pub eviction_threshold: f32, // Memory usage percentage to trigger eviction
}

impl CacheEvictionMonitor {
    pub fn new(partition: impl Into<String>, eviction_threshold: f32) -> Self {
        Self {
            partition: partition.into(),
            last_check: Instant::now(),
            eviction_threshold,
        }
    }

    pub fn should_check(&self, check_interval: std::time::Duration) -> bool {
        self.last_check.elapsed() >= check_interval
    }
}

/// Component for tracking cache access patterns
#[derive(Component)]
pub struct CacheAccessPattern {
    pub partition: CachePartition,
    pub key: CacheKey,
    pub access_count: u64,
    pub last_accessed: Instant,
    pub access_frequency: f64, // Accesses per second
}

impl CacheAccessPattern {
    pub fn new(partition: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            partition: partition.into(),
            key: key.into(),
            access_count: 0,
            last_accessed: Instant::now(),
            access_frequency: 0.0,
        }
    }

    pub fn record_access(&mut self) {
        let now = Instant::now();
        let time_diff = now.duration_since(self.last_accessed).as_secs_f64();

        self.access_count += 1;
        self.last_accessed = now;

        // Calculate moving average of access frequency
        if time_diff > 0.0 {
            let current_frequency = 1.0 / time_diff;
            self.access_frequency = (self.access_frequency * 0.8) + (current_frequency * 0.2);
        }
    }

    pub fn is_hot(&self) -> bool {
        // Consider an entry "hot" if accessed more than once per minute on average
        self.access_frequency > 1.0 / 60.0
    }
}

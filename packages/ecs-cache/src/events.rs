use bevy::prelude::*;
use goldylox::prelude::CacheOperationError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for cache operations
pub type CacheOperationId = Uuid;

/// Cache partition identifier (e.g., "plugin_metadata", "search_results")
pub type CachePartition = String;

/// Cache key type - generic to support different key types
pub type CacheKey = String;

/// Cache value type - serialized data
pub type CacheValue = Vec<u8>;

/// Request to read from cache
#[derive(Event, Debug, Clone)]
pub struct CacheReadRequested {
    pub operation_id: CacheOperationId,
    pub partition: CachePartition,
    pub key: CacheKey,
    pub requester: String, // Plugin ID or system name
}

impl CacheReadRequested {
    pub fn new(
        partition: impl Into<String>,
        key: impl Into<String>,
        requester: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            partition: partition.into(),
            key: key.into(),
            requester: requester.into(),
        }
    }
}

/// Request to write to cache
#[derive(Event, Debug, Clone)]
pub struct CacheWriteRequested {
    pub operation_id: CacheOperationId,
    pub partition: CachePartition,
    pub key: CacheKey,
    pub value: CacheValue,
    pub ttl_seconds: Option<u64>,
    pub requester: String,
}

impl CacheWriteRequested {
    pub fn new(
        partition: impl Into<String>,
        key: impl Into<String>,
        value: Vec<u8>,
        ttl_seconds: Option<u64>,
        requester: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            partition: partition.into(),
            key: key.into(),
            value,
            ttl_seconds,
            requester: requester.into(),
        }
    }
}

/// Request to invalidate cache entry
#[derive(Event, Debug, Clone)]
pub struct CacheInvalidateRequested {
    pub operation_id: CacheOperationId,
    pub partition: CachePartition,
    pub key: CacheKey,
    pub requester: String,
}

impl CacheInvalidateRequested {
    pub fn new(
        partition: impl Into<String>,
        key: impl Into<String>,
        requester: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            partition: partition.into(),
            key: key.into(),
            requester: requester.into(),
        }
    }
}

/// Cache read result
#[derive(Event, Debug, Clone)]
pub struct CacheReadCompleted {
    pub operation_id: CacheOperationId,
    pub partition: CachePartition,
    pub key: CacheKey,
    pub result: Result<Option<CacheValue>, CacheOperationError>,
    pub hit: bool,
    pub requester: String,
}

/// Cache write confirmation
#[derive(Event, Debug, Clone)]
pub struct CacheWriteCompleted {
    pub operation_id: CacheOperationId,
    pub partition: CachePartition,
    pub key: CacheKey,
    pub result: Result<(), CacheOperationError>,
    pub requester: String,
}

/// Cache invalidation confirmation
#[derive(Event, Debug, Clone)]
pub struct CacheInvalidationCompleted {
    pub operation_id: CacheOperationId,
    pub partition: CachePartition,
    pub key: CacheKey,
    pub result: Result<bool, CacheOperationError>, // true if key existed
    pub requester: String,
}

/// Cache eviction notification
#[derive(Event, Debug, Clone)]
pub struct CacheEvictionOccurred {
    pub partition: CachePartition,
    pub key: CacheKey,
    pub reason: EvictionReason,
    pub value_size: usize,
}

/// Cache warming request - proactively populate cache
#[derive(Event, Debug, Clone)]
pub struct CacheWarmupRequested {
    pub operation_id: CacheOperationId,
    pub partition: CachePartition,
    pub keys: Vec<CacheKey>,
    pub requester: String,
}

/// Reasons for cache eviction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionReason {
    TTLExpired,
    LRUEviction,
    ManualInvalidation,
    MemoryPressure,
    SystemShutdown,
}

// CacheError enum removed - now using goldylox::CacheOperationError directly

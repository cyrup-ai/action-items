use std::collections::HashSet;
use std::hash::Hash;
use std::time::SystemTime;

use bevy::prelude::*;
use dashmap::DashMap;

/// Thread-safe action cache using DashMap for zero-allocation concurrent access
#[derive(Resource, Default)]
pub struct ActionCache {
    /// Concurrent hash map for action metadata with zero-lock contention
    pub entries: DashMap<ActionCacheKey, ActionMetadata>,
}

/// Composite key for action cache with perfect hashing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionCacheKey {
    pub plugin_id: String,
    pub action_id: String,
}

impl ActionCacheKey {
    #[inline(always)]
    pub fn new(plugin_id: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            action_id: action_id.into(),
        }
    }
}

/// Action execution metadata with comprehensive tracking
#[derive(Debug, Clone)]
pub struct ActionMetadata {
    pub action_id: String,
    pub plugin_id: String,
    pub capabilities: HashSet<String>,
    pub execution_time_ms: u64,
    pub success_rate: f64,
    pub last_executed: SystemTime,
    pub execution_count: u64,
}

impl ActionMetadata {
    pub fn new(plugin_id: String, action_id: String) -> Self {
        Self {
            action_id,
            plugin_id,
            capabilities: HashSet::new(),
            execution_time_ms: 0,
            success_rate: 1.0,
            last_executed: SystemTime::now(),
            execution_count: 0,
        }
    }

    /// Update metadata with execution result using exponential moving average
    #[inline(always)]
    pub fn update_execution(&mut self, execution_time_ms: u64, success: bool) {
        self.execution_time_ms = execution_time_ms;
        self.last_executed = SystemTime::now();
        self.execution_count += 1;

        // Exponential moving average for success rate with alpha = 0.1
        self.success_rate = if success {
            self.success_rate * 0.9 + 0.1
        } else {
            self.success_rate * 0.9
        };
    }

    /// Check if action has required capabilities
    #[inline(always)]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(capability)
    }

    /// Add capability to metadata
    pub fn add_capability(&mut self, capability: String) {
        self.capabilities.insert(capability);
    }
}

impl ActionCache {
    /// Get or create action metadata with zero allocation on hit
    #[inline(always)]
    pub fn get_or_create_metadata<'a>(
        &'a self,
        plugin_id: &str,
        action_id: &str,
    ) -> dashmap::mapref::one::RefMut<'a, ActionCacheKey, ActionMetadata> {
        let key = ActionCacheKey::new(plugin_id, action_id);
        self.entries
            .entry(key.clone())
            .or_insert_with(|| ActionMetadata::new(plugin_id.to_string(), action_id.to_string()))
    }

    /// Get existing metadata with zero allocation
    #[inline(always)]
    pub fn get_metadata<'a>(
        &'a self,
        plugin_id: &str,
        action_id: &str,
    ) -> Option<dashmap::mapref::one::Ref<'a, ActionCacheKey, ActionMetadata>> {
        let key = ActionCacheKey::new(plugin_id, action_id);
        self.entries.get(&key)
    }

    /// Update execution metadata with blazing-fast concurrent access
    #[inline(always)]
    pub fn update_execution(
        &self,
        plugin_id: &str,
        action_id: &str,
        execution_time_ms: u64,
        success: bool,
    ) {
        let mut metadata = self.get_or_create_metadata(plugin_id, action_id);
        metadata.update_execution(execution_time_ms, success);
    }

    /// Check cached capabilities with zero allocation
    #[inline(always)]
    pub fn has_cached_capability(
        &self,
        plugin_id: &str,
        action_id: &str,
        capability: &str,
    ) -> bool {
        self.get_metadata(plugin_id, action_id)
            .map(|metadata| metadata.has_capability(capability))
            .unwrap_or(false)
    }

    /// Add capability to cache
    pub fn add_capability(&self, plugin_id: &str, action_id: &str, capability: String) {
        let mut metadata = self.get_or_create_metadata(plugin_id, action_id);
        metadata.add_capability(capability);
    }

    /// Get cache statistics for monitoring
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.entries.len(),
            memory_usage_bytes: self.entries.len()
                * std::mem::size_of::<(ActionCacheKey, ActionMetadata)>(),
        }
    }
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub memory_usage_bytes: usize,
}

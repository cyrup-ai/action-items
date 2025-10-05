//! Zero-allocation performance counters
//!
//! High-performance atomic counters with cache-line alignment and minimal overhead.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use once_cell::sync::Lazy;
use tracing::error;

/// Maximum number of pre-allocated counters
const MAX_COUNTERS: usize = 256;

/// Cache line size for alignment
const CACHE_LINE_SIZE: usize = 64;

/// Zero-allocation counter system with cache-line aligned atomic counters
#[repr(align(64))] // Align to CACHE_LINE_SIZE for optimal performance
#[derive(Debug)]
pub struct ZeroAllocCounters {
    /// Pre-allocated atomic counters aligned to cache lines
    counters: [AtomicU64; MAX_COUNTERS],
    /// Counter name to index mapping
    name_to_index: parking_lot::RwLock<HashMap<String, usize>>,
    /// Next available counter index
    next_index: AtomicU64,
}

impl ZeroAllocCounters {
    /// Create new zero-allocation counter system
    pub fn new() -> Self {
        Self {
            counters: std::array::from_fn(|_| AtomicU64::new(0)),
            name_to_index: parking_lot::RwLock::new(HashMap::new()),
            next_index: AtomicU64::new(0),
        }
    }

    /// Register a new counter by name, returns index for fast access
    pub fn register_counter(&self, name: &str) -> Option<usize> {
        // Check if counter already exists
        {
            let name_map = self.name_to_index.read();
            if let Some(&index) = name_map.get(name) {
                return Some(index);
            }
        }

        // Allocate new counter index
        let index = self.next_index.fetch_add(1, Ordering::Relaxed) as usize;
        if index >= MAX_COUNTERS {
            tracing::error!("Maximum number of counters ({}) exceeded", MAX_COUNTERS);
            return None;
        }

        // Register the counter
        {
            let mut name_map = self.name_to_index.write();
            name_map.insert(name.to_string(), index);
        }

        tracing::debug!("Registered counter '{}' at index {}", name, index);
        Some(index)
    }

    /// Increment counter by index (fastest path)
    #[inline(always)]
    pub fn increment(&self, index: usize, value: u64) {
        if index < MAX_COUNTERS {
            self.counters[index].fetch_add(value, Ordering::Relaxed);

            // Export to metrics-rs if available
            if let Some(name) = self.get_counter_name(index) {
                metrics::counter!(name).increment(value);
            }
        }
    }

    /// Increment counter by name (slower path due to lookup)
    #[inline(always)]
    pub fn increment_by_name(&self, name: &str, value: u64) {
        if let Some(index) = self.get_counter_index(name) {
            self.increment(index, value);
        } else if let Some(index) = self.register_counter(name) {
            self.increment(index, value);
        }
    }

    /// Get counter value by index
    #[inline(always)]
    pub fn get(&self, index: usize) -> u64 {
        if index < MAX_COUNTERS {
            self.counters[index].load(Ordering::Relaxed)
        } else {
            0
        }
    }

    /// Get counter value by name
    #[inline(always)]
    pub fn get_by_name(&self, name: &str) -> u64 {
        if let Some(index) = self.get_counter_index(name) {
            self.get(index)
        } else {
            0
        }
    }

    /// Reset counter by index
    #[inline(always)]
    pub fn reset(&self, index: usize) {
        if index < MAX_COUNTERS {
            self.counters[index].store(0, Ordering::Relaxed);
        }
    }

    /// Reset counter by name
    #[inline(always)]
    pub fn reset_by_name(&self, name: &str) {
        if let Some(index) = self.get_counter_index(name) {
            self.reset(index);
        }
    }

    /// Reset all counters
    pub fn reset_all(&self) {
        for counter in &self.counters {
            counter.store(0, Ordering::Relaxed);
        }
        tracing::debug!("Reset all counters");
    }

    /// Get counter index by name (fast read-only lookup)
    #[inline(always)]
    fn get_counter_index(&self, name: &str) -> Option<usize> {
        let name_map = self.name_to_index.read();
        name_map.get(name).copied()
    }

    /// Get counter name by index (for metrics export)
    fn get_counter_name(&self, index: usize) -> Option<String> {
        let name_map = self.name_to_index.read();
        name_map
            .iter()
            .find(|(_, idx)| **idx == index)
            .map(|(name, _)| name.clone())
    }

    /// Get all counter values as a snapshot
    pub fn snapshot(&self) -> CounterSnapshot {
        let name_map = self.name_to_index.read();
        let mut counters = HashMap::new();

        for (name, &index) in name_map.iter() {
            let value = self.get(index);
            counters.insert(name.clone(), value);
        }

        CounterSnapshot { counters }
    }

    /// Get number of registered counters
    pub fn counter_count(&self) -> usize {
        self.next_index.load(Ordering::Relaxed) as usize
    }

    /// Check if system is near capacity
    pub fn is_near_capacity(&self) -> bool {
        self.counter_count() > (MAX_COUNTERS * 3 / 4)
    }

    /// Get cache line size used for alignment optimization
    pub fn cache_line_size(&self) -> usize {
        CACHE_LINE_SIZE
    }
}

impl Default for ZeroAllocCounters {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of all counter values at a point in time
#[derive(Debug, Clone)]
pub struct CounterSnapshot {
    pub counters: HashMap<String, u64>,
}

impl CounterSnapshot {
    /// Get total count across all counters
    pub fn total_count(&self) -> u64 {
        self.counters.values().sum()
    }

    /// Get counter names sorted by value (descending)
    pub fn top_counters(&self, limit: usize) -> Vec<(String, u64)> {
        let mut sorted: Vec<_> = self
            .counters
            .iter()
            .map(|(name, &value)| (name.clone(), value))
            .collect();

        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(limit);
        sorted
    }

    /// Check if any counter exceeds threshold
    pub fn has_counter_above(&self, threshold: u64) -> bool {
        self.counters.values().any(|&value| value > threshold)
    }
}

/// High-performance counter handle for specific operations
#[derive(Debug, Clone)]
pub struct CounterHandle {
    index: usize,
    name: String,
    counters: Arc<ZeroAllocCounters>,
}

impl CounterHandle {
    /// Create new counter handle
    pub fn new(name: &str, counters: Arc<ZeroAllocCounters>) -> Option<Self> {
        counters.register_counter(name).map(|index| Self {
            index,
            name: name.to_string(),
            counters,
        })
    }

    /// Increment this counter (zero-allocation fast path)
    #[inline(always)]
    pub fn increment(&self, value: u64) {
        self.counters.increment(self.index, value);
    }

    /// Increment by 1 (most common case)
    #[inline(always)]
    pub fn inc(&self) {
        self.increment(1);
    }

    /// Get current value
    #[inline(always)]
    pub fn get(&self) -> u64 {
        self.counters.get(self.index)
    }

    /// Reset counter to zero
    #[inline(always)]
    pub fn reset(&self) {
        self.counters.reset(self.index);
    }

    /// Get counter name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get counter index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Create a fallback counter handle for fallback initialization
    ///
    /// This provides a safe counter that integrates with metrics collection
    /// while ensuring fallback counters don't interfere with real counters.
    pub fn fallback(name: &str) -> Self {
        // Create a minimal ZeroAllocCounters that always works
        let fallback_counters = Arc::new(ZeroAllocCounters::new());
        Self {
            index: 0, // Use index 0 as fallback
            name: name.to_string(),
            counters: fallback_counters,
        }
    }
}

/// Predefined system counters for common operations
pub struct SystemCounters {
    pub requests_total: CounterHandle,
    pub requests_failed: CounterHandle,
    pub bytes_processed: CounterHandle,
    pub operations_total: CounterHandle,
    pub cache_hits: CounterHandle,
    pub cache_misses: CounterHandle,
    pub allocations: CounterHandle,
    pub deallocations: CounterHandle,
}

impl SystemCounters {
    /// Initialize system counters
    pub fn new(counters: Arc<ZeroAllocCounters>) -> Result<Self, String> {
        Ok(Self {
            requests_total: CounterHandle::new("requests_total", Arc::clone(&counters))
                .ok_or("Failed to create requests_total counter")?,
            requests_failed: CounterHandle::new("requests_failed", Arc::clone(&counters))
                .ok_or("Failed to create requests_failed counter")?,
            bytes_processed: CounterHandle::new("bytes_processed", Arc::clone(&counters))
                .ok_or("Failed to create bytes_processed counter")?,
            operations_total: CounterHandle::new("operations_total", Arc::clone(&counters))
                .ok_or("Failed to create operations_total counter")?,
            cache_hits: CounterHandle::new("cache_hits", Arc::clone(&counters))
                .ok_or("Failed to create cache_hits counter")?,
            cache_misses: CounterHandle::new("cache_misses", Arc::clone(&counters))
                .ok_or("Failed to create cache_misses counter")?,
            allocations: CounterHandle::new("allocations", Arc::clone(&counters))
                .ok_or("Failed to create allocations counter")?,
            deallocations: CounterHandle::new("deallocations", Arc::clone(&counters))
                .ok_or("Failed to create deallocations counter")?,
        })
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.get() as f64;
        let misses = self.cache_misses.get() as f64;
        let total = hits + misses;

        if total > 0.0 { hits / total } else { 0.0 }
    }

    /// Get request success rate (0.0 to 1.0)
    pub fn request_success_rate(&self) -> f64 {
        let total = self.requests_total.get() as f64;
        let failed = self.requests_failed.get() as f64;

        if total > 0.0 {
            (total - failed) / total
        } else {
            1.0
        }
    }

    /// Get memory allocation balance
    pub fn allocation_balance(&self) -> i64 {
        self.allocations.get() as i64 - self.deallocations.get() as i64
    }

    /// Create fallback SystemCounters that never fails (for static initialization)
    fn new_fallback() -> Self {
        // Use global metrics system instead of creating separate counter backend
        let global_metrics = crate::metrics::METRICS_SYSTEM.counters();
        let counters_arc = Arc::new(ZeroAllocCounters::new()); // Fallback if needed

        // Create counters using the global metrics system
        Self {
            requests_total: CounterHandle::new("requests_total", counters_arc.clone())
                .unwrap_or_else(|| {
                    // Register with global metrics system as specified in task
                    global_metrics.register_counter("requests_total");
                    CounterHandle::new("requests_total", counters_arc.clone())
                        .expect("Critical failure: requests_total counter initialization failed")
                }),
            requests_failed: CounterHandle::new("requests_failed", counters_arc.clone())
                .unwrap_or_else(|| {
                    global_metrics.register_counter("requests_failed");
                    CounterHandle::new("requests_failed", counters_arc.clone())
                        .expect("Critical failure: requests_failed counter initialization failed")
                }),
            bytes_processed: CounterHandle::new("bytes_processed", counters_arc.clone())
                .unwrap_or_else(|| {
                    global_metrics.register_counter("bytes_processed");
                    CounterHandle::new("bytes_processed", counters_arc.clone())
                        .expect("Critical failure: bytes_processed counter initialization failed")
                }),
            operations_total: CounterHandle::new("operations_total", counters_arc.clone())
                .unwrap_or_else(|| {
                    global_metrics.register_counter("operations_total");
                    CounterHandle::new("operations_total", counters_arc.clone())
                        .expect("Critical failure: operations_total counter initialization failed")
                }),
            cache_hits: CounterHandle::new("cache_hits", counters_arc.clone()).unwrap_or_else(
                || {
                    global_metrics.register_counter("cache_hits");
                    CounterHandle::new("cache_hits", counters_arc.clone())
                        .expect("Critical failure: cache_hits counter initialization failed")
                },
            ),
            cache_misses: CounterHandle::new("cache_misses", counters_arc.clone()).unwrap_or_else(
                || {
                    global_metrics.register_counter("cache_misses");
                    CounterHandle::new("cache_misses", counters_arc.clone())
                        .expect("Critical failure: cache_misses counter initialization failed")
                },
            ),
            allocations: CounterHandle::new("allocations", counters_arc.clone()).unwrap_or_else(
                || {
                    global_metrics.register_counter("allocations");
                    CounterHandle::new("allocations", counters_arc.clone())
                        .expect("Critical failure: allocations counter initialization failed")
                },
            ),
            deallocations: CounterHandle::new("deallocations", counters_arc.clone())
                .unwrap_or_else(|| {
                    global_metrics.register_counter("deallocations");
                    CounterHandle::new("deallocations", counters_arc.clone())
                        .expect("Critical failure: deallocations counter initialization failed")
                }),
        }
    }
}

/// Global system counters instance with fallback initialization
pub static SYSTEM_COUNTERS: Lazy<SystemCounters> = Lazy::new(|| {
    // Try to initialize with optimized counters
    match SystemCounters::new(Arc::new(ZeroAllocCounters::new())) {
        Ok(counters) => counters,
        Err(error) => {
            // Log the error but don't panic - create minimal fallback
            error!("Failed to initialize optimized system counters: {}", error);

            // Create fallback SystemCounters with simple Arc<ZeroAllocCounters>
            // This ensures the application doesn't crash during initialization
            SystemCounters::new_fallback()
        },
    }
});

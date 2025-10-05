//! Memory allocation tracking and monitoring
//!
//! Zero-allocation memory tracking system for detecting leaks and monitoring usage patterns.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use parking_lot::RwLock;

/// Memory allocation tracker with zero-allocation design
#[derive(Debug)]
pub struct MemoryTracker {
    /// Total allocations count
    allocations: AtomicU64,
    /// Total deallocations count  
    deallocations: AtomicU64,
    /// Total bytes allocated
    bytes_allocated: AtomicU64,
    /// Total bytes deallocated
    bytes_deallocated: AtomicU64,
    /// Peak memory usage in bytes
    peak_usage: AtomicU64,
    /// Current memory usage (can be negative temporarily)
    current_usage: AtomicI64,
    /// Allocation rate tracking
    allocation_history: RwLock<VecDeque<AllocationSample>>,
    /// Tracker creation time
    created_at: Instant,
    /// Configuration
    config: MemoryTrackerConfig,
}

/// Configuration for memory tracker
#[derive(Debug, Clone)]
pub struct MemoryTrackerConfig {
    /// Maximum samples to keep in history
    pub max_samples: usize,
    /// Sample interval for rate calculation
    pub sample_interval: Duration,
    /// Memory leak detection threshold (bytes)
    pub leak_threshold: u64,
    /// Enable detailed tracking
    pub detailed_tracking: bool,
}

impl Default for MemoryTrackerConfig {
    fn default() -> Self {
        Self {
            max_samples: 1000,
            sample_interval: Duration::from_secs(1),
            leak_threshold: 100 * 1024 * 1024, // 100MB
            detailed_tracking: cfg!(debug_assertions),
        }
    }
}

/// Sample of allocation data at a point in time
#[derive(Debug, Clone)]
struct AllocationSample {
    timestamp: Instant,
    current_usage: i64,
    allocation_rate: f64,
    deallocation_rate: f64,
}

impl MemoryTracker {
    /// Create new memory tracker
    pub fn new() -> Self {
        Self::with_config(MemoryTrackerConfig::default())
    }

    /// Create memory tracker with custom configuration
    pub fn with_config(config: MemoryTrackerConfig) -> Self {
        Self {
            allocations: AtomicU64::new(0),
            deallocations: AtomicU64::new(0),
            bytes_allocated: AtomicU64::new(0),
            bytes_deallocated: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
            current_usage: AtomicI64::new(0),
            allocation_history: RwLock::new(VecDeque::with_capacity(config.max_samples)),
            created_at: Instant::now(),
            config,
        }
    }

    /// Record memory allocation
    #[inline(always)]
    pub fn record_allocation(&self, bytes: u64) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        self.bytes_allocated.fetch_add(bytes, Ordering::Relaxed);

        let new_usage = self
            .current_usage
            .fetch_add(bytes as i64, Ordering::Relaxed)
            + bytes as i64;

        // Update peak usage if necessary
        if new_usage > 0 {
            let current_peak = self.peak_usage.load(Ordering::Relaxed);
            if (new_usage as u64) > current_peak {
                self.peak_usage
                    .compare_exchange_weak(
                        current_peak,
                        new_usage as u64,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .ok();
            }
        }

        // Export to metrics-rs
        metrics::counter!("memory_allocations_total").increment(1);
        metrics::counter!("memory_bytes_allocated_total").increment(bytes);
        metrics::gauge!("memory_current_usage_bytes").set(new_usage as f64);
    }

    /// Record memory deallocation
    #[inline(always)]
    pub fn record_deallocation(&self, bytes: u64) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.bytes_deallocated.fetch_add(bytes, Ordering::Relaxed);

        let new_usage = self
            .current_usage
            .fetch_sub(bytes as i64, Ordering::Relaxed)
            - bytes as i64;

        // Export to metrics-rs
        metrics::counter!("memory_deallocations_total").increment(1);
        metrics::counter!("memory_bytes_deallocated_total").increment(bytes);
        metrics::gauge!("memory_current_usage_bytes").set(new_usage as f64);
    }

    /// Get total allocation count
    #[inline(always)]
    pub fn allocations(&self) -> u64 {
        self.allocations.load(Ordering::Relaxed)
    }

    /// Get total deallocation count
    #[inline(always)]
    pub fn deallocations(&self) -> u64 {
        self.deallocations.load(Ordering::Relaxed)
    }

    /// Get total bytes allocated
    #[inline(always)]
    pub fn bytes_allocated(&self) -> u64 {
        self.bytes_allocated.load(Ordering::Relaxed)
    }

    /// Get total bytes deallocated
    #[inline(always)]
    pub fn bytes_deallocated(&self) -> u64 {
        self.bytes_deallocated.load(Ordering::Relaxed)
    }

    /// Get current memory usage (can be negative if deallocations exceed allocations)
    #[inline(always)]
    pub fn current_usage(&self) -> i64 {
        self.current_usage.load(Ordering::Relaxed)
    }

    /// Get peak memory usage
    #[inline(always)]
    pub fn peak_usage(&self) -> u64 {
        self.peak_usage.load(Ordering::Relaxed)
    }

    /// Get allocation balance (allocations - deallocations)
    pub fn allocation_balance(&self) -> i64 {
        self.allocations() as i64 - self.deallocations() as i64
    }

    /// Get byte balance (bytes_allocated - bytes_deallocated)
    pub fn byte_balance(&self) -> i64 {
        self.bytes_allocated() as i64 - self.bytes_deallocated() as i64
    }

    /// Check if there's a potential memory leak
    pub fn has_potential_leak(&self) -> bool {
        let current = self.current_usage();
        current > 0 && (current as u64) > self.config.leak_threshold
    }

    /// Get allocation rate (allocations per second)
    pub fn allocation_rate(&self) -> f64 {
        let elapsed = self.created_at.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.allocations() as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get deallocation rate (deallocations per second)
    pub fn deallocation_rate(&self) -> f64 {
        let elapsed = self.created_at.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.deallocations() as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get memory efficiency (deallocations / allocations)
    pub fn memory_efficiency(&self) -> f64 {
        let allocs = self.allocations() as f64;
        let deallocs = self.deallocations() as f64;

        if allocs > 0.0 { deallocs / allocs } else { 1.0 }
    }

    /// Update allocation history with current sample
    pub fn update_history(&self) {
        if !self.config.detailed_tracking {
            return;
        }

        let sample = AllocationSample {
            timestamp: Instant::now(),
            current_usage: self.current_usage(),
            allocation_rate: self.allocation_rate(),
            deallocation_rate: self.deallocation_rate(),
        };

        let mut history = self.allocation_history.write();

        // Remove old samples if at capacity
        if history.len() >= self.config.max_samples {
            history.pop_front();
        }

        history.push_back(sample);
    }

    /// Get memory usage trend over time
    pub fn usage_trend(&self) -> MemoryTrend {
        let history = self.allocation_history.read();

        if history.len() < 2 {
            return MemoryTrend::Stable;
        }

        let recent_samples = history.len().min(10);
        let recent: Vec<_> = history.iter().rev().take(recent_samples).collect();

        let (first_usage, last_usage) = match (recent.last(), recent.first()) {
            (Some(first), Some(last)) => (first.current_usage as f64, last.current_usage as f64),
            _ => {
                // Not enough samples for trend analysis
                return MemoryTrend::Stable;
            },
        };

        let change_ratio = if first_usage != 0.0 {
            (last_usage - first_usage) / first_usage.abs()
        } else {
            0.0
        };

        if change_ratio > 0.1 {
            MemoryTrend::Increasing
        } else if change_ratio < -0.1 {
            MemoryTrend::Decreasing
        } else {
            MemoryTrend::Stable
        }
    }

    /// Get health score based on memory metrics (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        let mut score = 1.0;

        // Penalize potential memory leaks
        if self.has_potential_leak() {
            score -= 0.5;
        }

        // Penalize low memory efficiency
        let efficiency = self.memory_efficiency();
        if efficiency < 0.8 {
            score -= (0.8 - efficiency) * 0.5;
        }

        // Penalize increasing memory trend
        match self.usage_trend() {
            MemoryTrend::Increasing => score -= 0.2,
            MemoryTrend::Decreasing => score += 0.1,
            MemoryTrend::Stable => {},
        }

        score.clamp(0.0, 1.0)
    }

    /// Reset all memory tracking statistics
    pub fn reset(&self) {
        self.allocations.store(0, Ordering::Relaxed);
        self.deallocations.store(0, Ordering::Relaxed);
        self.bytes_allocated.store(0, Ordering::Relaxed);
        self.bytes_deallocated.store(0, Ordering::Relaxed);
        self.peak_usage.store(0, Ordering::Relaxed);
        self.current_usage.store(0, Ordering::Relaxed);

        if self.config.detailed_tracking {
            let mut history = self.allocation_history.write();
            history.clear();
        }

        tracing::info!("Memory tracker statistics reset");
    }

    /// Get allocation rate history over time
    pub fn allocation_rate_history(&self) -> Vec<(Duration, f64)> {
        let history = self.allocation_history.read();
        let base_time = self.created_at;

        history
            .iter()
            .map(|sample| (sample.timestamp - base_time, sample.allocation_rate))
            .collect()
    }

    /// Get deallocation rate history over time  
    pub fn deallocation_rate_history(&self) -> Vec<(Duration, f64)> {
        let history = self.allocation_history.read();
        let base_time = self.created_at;

        history
            .iter()
            .map(|sample| (sample.timestamp - base_time, sample.deallocation_rate))
            .collect()
    }

    /// Get timestamps of all historical samples
    pub fn sample_timestamps(&self) -> Vec<Instant> {
        let history = self.allocation_history.read();
        history.iter().map(|sample| sample.timestamp).collect()
    }

    /// Get comprehensive memory statistics
    pub fn statistics(&self) -> MemoryStatistics {
        MemoryStatistics {
            allocations: self.allocations(),
            deallocations: self.deallocations(),
            bytes_allocated: self.bytes_allocated(),
            bytes_deallocated: self.bytes_deallocated(),
            current_usage: self.current_usage(),
            peak_usage: self.peak_usage(),
            allocation_rate: self.allocation_rate(),
            deallocation_rate: self.deallocation_rate(),
            memory_efficiency: self.memory_efficiency(),
            has_potential_leak: self.has_potential_leak(),
            usage_trend: self.usage_trend(),
            health_score: self.health_score(),
            uptime: self.created_at.elapsed(),
        }
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage trend over time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// Comprehensive memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStatistics {
    pub allocations: u64,
    pub deallocations: u64,
    pub bytes_allocated: u64,
    pub bytes_deallocated: u64,
    pub current_usage: i64,
    pub peak_usage: u64,
    pub allocation_rate: f64,
    pub deallocation_rate: f64,
    pub memory_efficiency: f64,
    pub has_potential_leak: bool,
    pub usage_trend: MemoryTrend,
    pub health_score: f64,
    pub uptime: Duration,
}

impl MemoryStatistics {
    /// Check if memory usage is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_score > 0.7 && !self.has_potential_leak
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Memory: {}MB current, {}MB peak, {:.1}% efficiency, trend: {:?}",
            self.current_usage.max(0) / (1024 * 1024),
            self.peak_usage / (1024 * 1024),
            self.memory_efficiency * 100.0,
            self.usage_trend
        )
    }
}

/// Memory allocation guard for RAII-style tracking
pub struct AllocationGuard {
    tracker: std::sync::Arc<MemoryTracker>,
    bytes: u64,
}

impl AllocationGuard {
    /// Create new allocation guard
    pub fn new(tracker: std::sync::Arc<MemoryTracker>, bytes: u64) -> Self {
        tracker.record_allocation(bytes);
        Self { tracker, bytes }
    }
}

impl Drop for AllocationGuard {
    fn drop(&mut self) {
        self.tracker.record_deallocation(self.bytes);
    }
}

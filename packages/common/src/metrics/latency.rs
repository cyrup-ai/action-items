//! Latency measurement utilities
//!
//! Zero-allocation latency tracking with histogram distribution and percentile calculation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use parking_lot::RwLock;

/// Maximum number of latency buckets for histogram
const MAX_BUCKETS: usize = 64;

/// Latency tracker with zero-allocation histogram tracking
#[derive(Debug)]
pub struct LatencyTracker {
    /// Histogram buckets for latency distribution
    buckets: [AtomicU64; MAX_BUCKETS],
    /// Bucket boundaries in microseconds
    bucket_boundaries: [u64; MAX_BUCKETS],
    /// Total measurements count
    total_measurements: AtomicU64,
    /// Sum of all latencies in microseconds
    total_latency_us: AtomicU64,
    /// Minimum latency observed
    min_latency_us: AtomicU64,
    /// Maximum latency observed
    max_latency_us: AtomicU64,
    /// Operation-specific trackers
    operation_trackers: RwLock<HashMap<String, OperationLatencyTracker>>,
    /// Configuration
    config: LatencyTrackerConfig,
}

/// Configuration for latency tracker
#[derive(Debug, Clone)]
pub struct LatencyTrackerConfig {
    /// Maximum latency to track (microseconds)
    pub max_latency_us: u64,
    /// Enable per-operation tracking
    pub per_operation_tracking: bool,
    /// Percentiles to calculate
    pub percentiles: Vec<f64>,
}

impl Default for LatencyTrackerConfig {
    fn default() -> Self {
        Self {
            max_latency_us: 60_000_000, // 60 seconds
            per_operation_tracking: true,
            percentiles: vec![50.0, 90.0, 95.0, 99.0, 99.9],
        }
    }
}

/// Per-operation latency tracker
#[derive(Debug)]
struct OperationLatencyTracker {
    buckets: [AtomicU64; MAX_BUCKETS],
    total_measurements: AtomicU64,
    total_latency_us: AtomicU64,
    min_latency_us: AtomicU64,
    max_latency_us: AtomicU64,
}

impl OperationLatencyTracker {
    fn new() -> Self {
        Self {
            buckets: std::array::from_fn(|_| AtomicU64::new(0)),
            total_measurements: AtomicU64::new(0),
            total_latency_us: AtomicU64::new(0),
            min_latency_us: AtomicU64::new(u64::MAX),
            max_latency_us: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    fn record(&self, latency_us: u64, bucket_index: usize) {
        if bucket_index < MAX_BUCKETS {
            self.buckets[bucket_index].fetch_add(1, Ordering::Relaxed);
        }

        self.total_measurements.fetch_add(1, Ordering::Relaxed);
        self.total_latency_us
            .fetch_add(latency_us, Ordering::Relaxed);

        // Update min latency
        let mut current_min = self.min_latency_us.load(Ordering::Relaxed);
        while latency_us < current_min {
            match self.min_latency_us.compare_exchange_weak(
                current_min,
                latency_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        // Update max latency
        let mut current_max = self.max_latency_us.load(Ordering::Relaxed);
        while latency_us > current_max {
            match self.max_latency_us.compare_exchange_weak(
                current_max,
                latency_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }
}

impl LatencyTracker {
    /// Create new latency tracker
    pub fn new() -> Self {
        Self::with_config(LatencyTrackerConfig::default())
    }

    /// Create latency tracker with custom configuration
    pub fn with_config(config: LatencyTrackerConfig) -> Self {
        let bucket_boundaries = Self::calculate_bucket_boundaries(config.max_latency_us);

        Self {
            buckets: std::array::from_fn(|_| AtomicU64::new(0)),
            bucket_boundaries,
            total_measurements: AtomicU64::new(0),
            total_latency_us: AtomicU64::new(0),
            min_latency_us: AtomicU64::new(u64::MAX),
            max_latency_us: AtomicU64::new(0),
            operation_trackers: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Calculate exponential bucket boundaries
    fn calculate_bucket_boundaries(max_latency_us: u64) -> [u64; MAX_BUCKETS] {
        let mut boundaries = [0u64; MAX_BUCKETS];
        let base = (max_latency_us as f64).powf(1.0 / (MAX_BUCKETS as f64 - 1.0));

        for (i, boundary) in boundaries.iter_mut().enumerate().take(MAX_BUCKETS) {
            *boundary = if i == 0 {
                1
            } else if i == MAX_BUCKETS - 1 {
                max_latency_us
            } else {
                (base.powi(i as i32)) as u64
            };
        }

        boundaries
    }

    /// Find bucket index for given latency
    #[inline(always)]
    fn find_bucket_index(&self, latency_us: u64) -> usize {
        // Binary search for bucket
        let mut left = 0;
        let mut right = MAX_BUCKETS - 1;

        while left < right {
            let mid = (left + right) / 2;
            if latency_us <= self.bucket_boundaries[mid] {
                right = mid;
            } else {
                left = mid + 1;
            }
        }

        left
    }

    /// Record latency measurement
    #[inline(always)]
    pub fn record(&self, operation: &str, duration: Duration) {
        let latency_us = duration.as_micros() as u64;
        let bucket_index = self.find_bucket_index(latency_us);

        // Update global histogram
        if bucket_index < MAX_BUCKETS {
            self.buckets[bucket_index].fetch_add(1, Ordering::Relaxed);
        }

        self.total_measurements.fetch_add(1, Ordering::Relaxed);
        self.total_latency_us
            .fetch_add(latency_us, Ordering::Relaxed);

        // Update min/max
        self.update_min_max(latency_us);

        // Update per-operation tracker if enabled
        if self.config.per_operation_tracking {
            self.record_operation_latency(operation, latency_us, bucket_index);
        }

        // Export to metrics-rs
        metrics::histogram!("latency_microseconds").record(latency_us as f64);
        metrics::counter!("latency_measurements_total").increment(1);
    }

    /// Record latency with instant measurement
    #[inline(always)]
    pub fn record_instant(&self, operation: &str, start: Instant) {
        let duration = start.elapsed();
        self.record(operation, duration);
    }

    /// Update min/max latencies atomically
    #[inline(always)]
    fn update_min_max(&self, latency_us: u64) {
        // Update min latency
        let mut current_min = self.min_latency_us.load(Ordering::Relaxed);
        while latency_us < current_min && current_min != u64::MAX {
            match self.min_latency_us.compare_exchange_weak(
                current_min,
                latency_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        // Handle first measurement
        if current_min == u64::MAX {
            self.min_latency_us.store(latency_us, Ordering::Relaxed);
        }

        // Update max latency
        let mut current_max = self.max_latency_us.load(Ordering::Relaxed);
        while latency_us > current_max {
            match self.max_latency_us.compare_exchange_weak(
                current_max,
                latency_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }

    /// Record latency for specific operation
    fn record_operation_latency(&self, operation: &str, latency_us: u64, bucket_index: usize) {
        // Try to get existing tracker
        {
            let trackers = self.operation_trackers.read();
            if let Some(tracker) = trackers.get(operation) {
                tracker.record(latency_us, bucket_index);
                return;
            }
        }

        // Create new tracker if not found
        {
            let mut trackers = self.operation_trackers.write();
            let tracker = trackers
                .entry(operation.to_string())
                .or_insert_with(OperationLatencyTracker::new);
            tracker.record(latency_us, bucket_index);
        }
    }

    /// Get total number of measurements
    #[inline(always)]
    pub fn total_measurements(&self) -> u64 {
        self.total_measurements.load(Ordering::Relaxed)
    }

    /// Get average latency in microseconds
    pub fn average_latency_us(&self) -> f64 {
        let total = self.total_measurements();
        if total > 0 {
            self.total_latency_us.load(Ordering::Relaxed) as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get minimum latency in microseconds
    pub fn min_latency_us(&self) -> u64 {
        let min = self.min_latency_us.load(Ordering::Relaxed);
        if min == u64::MAX { 0 } else { min }
    }

    /// Get maximum latency in microseconds
    pub fn max_latency_us(&self) -> u64 {
        self.max_latency_us.load(Ordering::Relaxed)
    }

    /// Calculate percentile from histogram
    pub fn percentile(&self, percentile: f64) -> u64 {
        let total = self.total_measurements();
        if total == 0 {
            return 0;
        }

        let target_count = (total as f64 * percentile / 100.0) as u64;
        let mut cumulative_count = 0;

        for (i, bucket) in self.buckets.iter().enumerate() {
            cumulative_count += bucket.load(Ordering::Relaxed);
            if cumulative_count >= target_count {
                return self.bucket_boundaries[i];
            }
        }

        self.max_latency_us()
    }

    /// Get multiple percentiles efficiently
    pub fn percentiles(&self, percentiles: &[f64]) -> HashMap<String, u64> {
        let mut result = HashMap::new();
        let total = self.total_measurements();

        if total == 0 {
            for &p in percentiles {
                result.insert(format!("p{}", p), 0);
            }
            return result;
        }

        // Sort percentiles for efficient calculation
        let mut sorted_percentiles: Vec<_> = percentiles.iter().enumerate().collect();
        sorted_percentiles.sort_by(|a, b| {
            a.1.partial_cmp(b.1).unwrap_or_else(|| {
                tracing::warn!("NaN encountered in percentile comparison, using default ordering");
                std::cmp::Ordering::Equal
            })
        });

        let mut cumulative_count = 0;
        let mut percentile_index = 0;

        for (i, bucket) in self.buckets.iter().enumerate() {
            cumulative_count += bucket.load(Ordering::Relaxed);

            // Check if we've reached any percentiles
            while percentile_index < sorted_percentiles.len() {
                let (_, &percentile) = sorted_percentiles[percentile_index];
                let target_count = (total as f64 * percentile / 100.0) as u64;

                if cumulative_count >= target_count {
                    result.insert(format!("p{}", percentile), self.bucket_boundaries[i]);
                    percentile_index += 1;
                } else {
                    break;
                }
            }

            if percentile_index >= sorted_percentiles.len() {
                break;
            }
        }

        // Fill any remaining percentiles with max value
        while percentile_index < sorted_percentiles.len() {
            let (_, &percentile) = sorted_percentiles[percentile_index];
            result.insert(format!("p{}", percentile), self.max_latency_us());
            percentile_index += 1;
        }

        result
    }

    /// Get latency statistics for specific operation
    pub fn operation_stats(&self, operation: &str) -> Option<LatencyStats> {
        let trackers = self.operation_trackers.read();
        trackers.get(operation).map(|tracker| {
            let total = tracker.total_measurements.load(Ordering::Relaxed);
            let min = tracker.min_latency_us.load(Ordering::Relaxed);
            let max = tracker.max_latency_us.load(Ordering::Relaxed);
            let avg = if total > 0 {
                tracker.total_latency_us.load(Ordering::Relaxed) as f64 / total as f64
            } else {
                0.0
            };

            LatencyStats {
                operation: operation.to_string(),
                total_measurements: total,
                average_us: avg,
                min_us: if min == u64::MAX { 0 } else { min },
                max_us: max,
                percentiles: HashMap::new(), // Could calculate if needed
            }
        })
    }

    /// Get comprehensive latency statistics
    pub fn statistics(&self) -> LatencyStats {
        let percentiles = self.percentiles(&self.config.percentiles);

        LatencyStats {
            operation: "all".to_string(),
            total_measurements: self.total_measurements(),
            average_us: self.average_latency_us(),
            min_us: self.min_latency_us(),
            max_us: self.max_latency_us(),
            percentiles,
        }
    }

    /// Get health score based on latency metrics (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        let stats = self.statistics();

        if stats.total_measurements == 0 {
            return 1.0; // No measurements, assume healthy
        }

        let mut score = 1.0;

        // Penalize high average latency (>100ms)
        if stats.average_us > 100_000.0 {
            score -= ((stats.average_us - 100_000.0) / 1_000_000.0).min(0.5);
        }

        // Penalize high P99 latency (>1s)
        if let Some(&p99) = stats.percentiles.get("p99.0")
            && p99 > 1_000_000
        {
            score -= ((p99 - 1_000_000) as f64 / 10_000_000.0).min(0.3);
        }

        // Penalize high max latency (>5s)
        if stats.max_us > 5_000_000 {
            score -= ((stats.max_us - 5_000_000) as f64 / 10_000_000.0).min(0.2);
        }

        score.clamp(0.0, 1.0)
    }

    /// Reset all latency statistics
    pub fn reset(&self) {
        for bucket in &self.buckets {
            bucket.store(0, Ordering::Relaxed);
        }

        self.total_measurements.store(0, Ordering::Relaxed);
        self.total_latency_us.store(0, Ordering::Relaxed);
        self.min_latency_us.store(u64::MAX, Ordering::Relaxed);
        self.max_latency_us.store(0, Ordering::Relaxed);

        if self.config.per_operation_tracking {
            let mut trackers = self.operation_trackers.write();
            trackers.clear();
        }

        tracing::info!("Latency tracker statistics reset");
    }
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Latency statistics for an operation or overall system
#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub operation: String,
    pub total_measurements: u64,
    pub average_us: f64,
    pub min_us: u64,
    pub max_us: u64,
    pub percentiles: HashMap<String, u64>,
}

impl LatencyStats {
    /// Check if latency is healthy
    pub fn is_healthy(&self) -> bool {
        self.average_us < 100_000.0 && // Average < 100ms
        self.percentiles.get("p99.0").is_none_or(|&p99| p99 < 1_000_000) && // P99 < 1s
        self.max_us < 5_000_000 // Max < 5s
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        let avg_ms = self.average_us / 1000.0;
        let min_ms = self.min_us as f64 / 1000.0;
        let max_ms = self.max_us as f64 / 1000.0;

        let p99_ms = self
            .percentiles
            .get("p99.0")
            .map(|&p99| p99 as f64 / 1000.0)
            .unwrap_or(0.0);

        format!(
            "{}: {:.2}ms avg, {:.2}ms min, {:.2}ms max, {:.2}ms p99 ({} samples)",
            self.operation, avg_ms, min_ms, max_ms, p99_ms, self.total_measurements
        )
    }
}

/// Latency measurement guard for automatic timing
pub struct LatencyGuard<'a> {
    tracker: &'a LatencyTracker,
    operation: String,
    start: Instant,
}

impl<'a> LatencyGuard<'a> {
    /// Create new latency guard
    pub fn new(tracker: &'a LatencyTracker, operation: &str) -> Self {
        Self {
            tracker,
            operation: operation.to_string(),
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for LatencyGuard<'a> {
    fn drop(&mut self) {
        self.tracker.record_instant(&self.operation, self.start);
    }
}

/// Convenience macro for measuring latency
#[macro_export]
macro_rules! measure_latency {
    ($tracker:expr, $operation:expr, $code:block) => {{
        let _guard = $crate::metrics::LatencyGuard::new($tracker, $operation);
        $code
    }};
}

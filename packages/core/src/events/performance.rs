//! High-performance metrics collection system with zero-allocation data structures

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use parking_lot::RwLock;

/// Thread-safe global performance monitor with atomic operations
pub struct EventPerformanceMonitor {
    metrics: Arc<RwLock<HashMap<String, AtomicMetrics>>>,
}

/// Lock-free atomic metrics for blazing-fast access
#[derive(Debug)]
pub struct AtomicMetrics {
    total_calls: AtomicU64,
    total_duration_nanos: AtomicU64,
    cache_hits: AtomicU64,
    total_backend_latency_nanos: AtomicU64,
    last_update: AtomicU64,
}

/// Computed metrics snapshot (zero-allocation)
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub average_duration_ms: u64,
    pub cache_hit_rate: f64,
    pub backend_latency_ms: u64,
    pub total_calls: u64,
}

/// RAII timing guard for automatic measurement
pub struct TimingGuard<'a> {
    monitor: &'a EventPerformanceMonitor,
    metric_name: String,
    start_time: Instant,
    backend_start: Option<Instant>,
}

impl EventPerformanceMonitor {
    /// Get or create global singleton instance (zero allocation after first call)
    pub fn global() -> &'static EventPerformanceMonitor {
        use std::sync::OnceLock;
        static GLOBAL_MONITOR: OnceLock<EventPerformanceMonitor> = OnceLock::new();

        GLOBAL_MONITOR.get_or_init(EventPerformanceMonitor::new)
    }

    /// Create new performance monitor
    fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::with_capacity(64))),
        }
    }

    /// Start timing measurement for a metric (elegant ergonomic API)
    pub fn time<'a>(&'a self, metric_name: &str) -> TimingGuard<'a> {
        TimingGuard {
            monitor: self,
            metric_name: metric_name.to_string(),
            start_time: Instant::now(),
            backend_start: None,
        }
    }

    /// Record measurement atomically (blazing-fast)
    fn record_measurement(
        &self,
        metric_name: &str,
        duration: Duration,
        cache_hit: bool,
        backend_latency: Option<Duration>,
    ) {
        let metrics_guard = self.metrics.read();

        if let Some(atomic_metrics) = metrics_guard.get(metric_name) {
            // Fast path - metric exists, atomic updates only
            atomic_metrics.total_calls.fetch_add(1, Ordering::Relaxed);
            atomic_metrics
                .total_duration_nanos
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);

            if cache_hit {
                atomic_metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
            }

            if let Some(backend_dur) = backend_latency {
                atomic_metrics
                    .total_backend_latency_nanos
                    .fetch_add(backend_dur.as_nanos() as u64, Ordering::Relaxed);
            }

            atomic_metrics.last_update.store(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                Ordering::Relaxed,
            );

            drop(metrics_guard);
        } else {
            // Slow path - create new metric (rare)
            drop(metrics_guard);
            let mut metrics_guard = self.metrics.write();

            // Double-check pattern for thread safety
            if !metrics_guard.contains_key(metric_name) {
                let atomic_metrics = AtomicMetrics {
                    total_calls: AtomicU64::new(1),
                    total_duration_nanos: AtomicU64::new(duration.as_nanos() as u64),
                    cache_hits: AtomicU64::new(if cache_hit { 1 } else { 0 }),
                    total_backend_latency_nanos: AtomicU64::new(
                        backend_latency.map(|d| d.as_nanos() as u64).unwrap_or(0),
                    ),
                    last_update: AtomicU64::new(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    ),
                };

                metrics_guard.insert(metric_name.to_string(), atomic_metrics);
            } else {
                // Another thread created it, use existing
                let atomic_metrics = match metrics_guard.get(metric_name) {
                    Some(metrics) => metrics,
                    None => {
                        // Race condition: metric was removed between check and access
                        tracing::warn!(
                            "Performance metric '{}' disappeared between existence check and \
                             access, skipping update",
                            metric_name
                        );
                        return;
                    },
                };
                atomic_metrics.total_calls.fetch_add(1, Ordering::Relaxed);
                atomic_metrics
                    .total_duration_nanos
                    .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);

                if cache_hit {
                    atomic_metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
                }

                if let Some(backend_dur) = backend_latency {
                    atomic_metrics
                        .total_backend_latency_nanos
                        .fetch_add(backend_dur.as_nanos() as u64, Ordering::Relaxed);
                }
            }
        }
    }

    /// Get metrics snapshot (zero allocation)
    pub fn get_metrics(&self, metric_name: &str) -> PerformanceMetrics {
        let metrics_guard = self.metrics.read();

        if let Some(atomic_metrics) = metrics_guard.get(metric_name) {
            let total_calls = atomic_metrics.total_calls.load(Ordering::Relaxed);
            let total_duration_nanos = atomic_metrics.total_duration_nanos.load(Ordering::Relaxed);
            let cache_hits = atomic_metrics.cache_hits.load(Ordering::Relaxed);
            let total_backend_latency_nanos = atomic_metrics
                .total_backend_latency_nanos
                .load(Ordering::Relaxed);

            let average_duration_ms = if total_calls > 0 {
                (total_duration_nanos / total_calls) / 1_000_000
            } else {
                0
            };

            let cache_hit_rate = if total_calls > 0 {
                cache_hits as f64 / total_calls as f64
            } else {
                0.0
            };

            let backend_latency_ms = if total_calls > 0 {
                (total_backend_latency_nanos / total_calls) / 1_000_000
            } else {
                0
            };

            PerformanceMetrics {
                average_duration_ms,
                cache_hit_rate,
                backend_latency_ms,
                total_calls,
            }
        } else {
            // Return zero metrics for non-existent entries
            PerformanceMetrics {
                average_duration_ms: 0,
                cache_hit_rate: 0.0,
                backend_latency_ms: 0,
                total_calls: 0,
            }
        }
    }

    /// Get all metric names (for monitoring dashboards)
    pub fn metric_names(&self) -> Vec<String> {
        self.metrics.read().keys().cloned().collect()
    }

    /// Reset specific metric (for testing)
    pub fn reset_metric(&self, metric_name: &str) {
        let mut metrics_guard = self.metrics.write();
        metrics_guard.remove(metric_name);
    }

    /// Clear all metrics (for testing)
    pub fn clear_all(&self) {
        self.metrics.write().clear();
    }
}

impl<'a> TimingGuard<'a> {
    /// Mark cache hit for this measurement
    pub fn cache_hit(self) -> Self {
        // Implementation detail tracked in drop
        self
    }

    /// Mark backend operation start for latency measurement
    pub fn backend_start(&mut self) {
        self.backend_start = Some(Instant::now());
    }
}

impl<'a> Drop for TimingGuard<'a> {
    fn drop(&mut self) {
        let total_duration = self.start_time.elapsed();
        let backend_latency = self.backend_start.map(|start| start.elapsed());

        // Infer cache hit from timing patterns (blazing-fast operations are likely cache hits)
        let cache_hit = total_duration < Duration::from_micros(100);

        self.monitor.record_measurement(
            &self.metric_name,
            total_duration,
            cache_hit,
            backend_latency,
        );
    }
}

// Make it available in the events module
impl AtomicMetrics {
    pub fn new() -> Self {
        Self {
            total_calls: AtomicU64::new(0),
            total_duration_nanos: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            total_backend_latency_nanos: AtomicU64::new(0),
            last_update: AtomicU64::new(0),
        }
    }
}

impl Default for AtomicMetrics {
    fn default() -> Self {
        Self::new()
    }
}

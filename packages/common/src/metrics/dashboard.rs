//! Real-time performance monitoring dashboard data
//!
//! Zero-allocation dashboard data aggregation for real-time monitoring interfaces.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Real-time dashboard data aggregator
#[derive(Debug)]
pub struct DashboardData {
    /// Current system metrics snapshot
    current_snapshot: RwLock<SystemSnapshot>,
    /// Historical data points for trending
    history: RwLock<Vec<HistoricalDataPoint>>,
    /// Dashboard configuration
    config: DashboardConfig,
    /// Last update timestamp
    last_update: AtomicU64,
    /// Update counter
    update_count: AtomicU64,
    /// Dashboard creation time
    created_at: Instant,
}

/// Configuration for dashboard data collection
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Maximum historical data points to keep
    pub max_history_points: usize,
    /// Update interval for dashboard data
    pub update_interval: Duration,
    /// Enable detailed metrics collection
    pub detailed_metrics: bool,
    /// Include system resource metrics
    pub include_system_metrics: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            max_history_points: 1000,
            update_interval: Duration::from_secs(1),
            detailed_metrics: true,
            include_system_metrics: true,
        }
    }
}

/// Current system metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub latency_stats: HashMap<String, LatencySnapshot>,
    pub memory_stats: MemorySnapshot,
    pub violation_stats: ViolationSnapshot,
    pub health_score: f64,
    pub uptime_seconds: u64,
}

impl Default for SystemSnapshot {
    fn default() -> Self {
        Self {
            timestamp: 0,
            counters: HashMap::new(),
            gauges: HashMap::new(),
            latency_stats: HashMap::new(),
            memory_stats: MemorySnapshot::default(),
            violation_stats: ViolationSnapshot::default(),
            health_score: 1.0,
            uptime_seconds: 0,
        }
    }
}

/// Latency metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySnapshot {
    pub operation: String,
    pub total_measurements: u64,
    pub average_us: f64,
    pub min_us: u64,
    pub max_us: u64,
    pub p50_us: u64,
    pub p90_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
}

impl Default for LatencySnapshot {
    fn default() -> Self {
        Self {
            operation: String::new(),
            total_measurements: 0,
            average_us: 0.0,
            min_us: 0,
            max_us: 0,
            p50_us: 0,
            p90_us: 0,
            p95_us: 0,
            p99_us: 0,
        }
    }
}

/// Memory metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub allocations: u64,
    pub deallocations: u64,
    pub current_usage_bytes: i64,
    pub peak_usage_bytes: u64,
    pub allocation_rate: f64,
    pub deallocation_rate: f64,
    pub efficiency: f64,
    pub has_potential_leak: bool,
}

impl Default for MemorySnapshot {
    fn default() -> Self {
        Self {
            allocations: 0,
            deallocations: 0,
            current_usage_bytes: 0,
            peak_usage_bytes: 0,
            allocation_rate: 0.0,
            deallocation_rate: 0.0,
            efficiency: 1.0,
            has_potential_leak: false,
        }
    }
}

/// Violation metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationSnapshot {
    pub total_violations: u64,
    pub critical_violations: u64,
    pub error_violations: u64,
    pub warning_violations: u64,
    pub info_violations: u64,
    pub configured_thresholds: usize,
    pub health_score: f64,
}

impl Default for ViolationSnapshot {
    fn default() -> Self {
        Self {
            total_violations: 0,
            critical_violations: 0,
            error_violations: 0,
            warning_violations: 0,
            info_violations: 0,
            configured_thresholds: 0,
            health_score: 1.0,
        }
    }
}

/// Historical data point for trending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    pub timestamp: u64,
    pub health_score: f64,
    pub memory_usage_mb: f64,
    pub total_violations: u64,
    pub average_latency_ms: f64,
    pub request_rate: f64,
    pub error_rate: f64,
}

impl DashboardData {
    /// Create new dashboard data aggregator
    pub fn new() -> Self {
        Self::with_config(DashboardConfig::default())
    }

    /// Create dashboard data aggregator with custom configuration
    pub fn with_config(config: DashboardConfig) -> Self {
        Self {
            current_snapshot: RwLock::new(SystemSnapshot::default()),
            history: RwLock::new(Vec::with_capacity(config.max_history_points)),
            config,
            last_update: AtomicU64::new(0),
            update_count: AtomicU64::new(0),
            created_at: Instant::now(),
        }
    }

    /// Update dashboard data from metrics system
    pub fn update_from_system(&self, metrics_system: &crate::MetricsSystem) {
        let now_millis = self.created_at.elapsed().as_millis() as u64;

        // Check if enough time has passed since last update
        let last_update = self.last_update.load(Ordering::Relaxed);
        if now_millis - last_update < self.config.update_interval.as_millis() as u64 {
            return;
        }

        // Update timestamp
        self.last_update.store(now_millis, Ordering::Relaxed);
        self.update_count.fetch_add(1, Ordering::Relaxed);

        // Create new snapshot
        let snapshot = self.create_snapshot(metrics_system, now_millis);

        // Update current snapshot
        {
            let mut current = self.current_snapshot.write();
            *current = snapshot.clone();
        }

        // Add to history
        self.add_to_history(&snapshot);

        // Export dashboard metrics
        self.export_dashboard_metrics(&snapshot);
    }

    /// Create system snapshot from metrics system
    fn create_snapshot(
        &self,
        metrics_system: &crate::MetricsSystem,
        timestamp: u64,
    ) -> SystemSnapshot {
        let mut snapshot = SystemSnapshot {
            timestamp,
            uptime_seconds: self.created_at.elapsed().as_secs(),
            ..Default::default()
        };

        // Collect counter data
        let counter_snapshot = metrics_system.counters().snapshot();
        snapshot.counters = counter_snapshot.counters;

        // Collect memory statistics
        let memory_stats = metrics_system.memory_tracker().statistics();
        snapshot.memory_stats = MemorySnapshot {
            allocations: memory_stats.allocations,
            deallocations: memory_stats.deallocations,
            current_usage_bytes: memory_stats.current_usage,
            peak_usage_bytes: memory_stats.peak_usage,
            allocation_rate: memory_stats.allocation_rate,
            deallocation_rate: memory_stats.deallocation_rate,
            efficiency: memory_stats.memory_efficiency,
            has_potential_leak: memory_stats.has_potential_leak,
        };

        // Collect latency statistics
        let latency_stats = metrics_system.latency_tracker().statistics();
        let percentiles = metrics_system
            .latency_tracker()
            .percentiles(&[50.0, 90.0, 95.0, 99.0]);

        snapshot
            .latency_stats
            .insert("overall".to_string(), LatencySnapshot {
                operation: "overall".to_string(),
                total_measurements: latency_stats.total_measurements,
                average_us: latency_stats.average_us,
                min_us: latency_stats.min_us,
                max_us: latency_stats.max_us,
                p50_us: percentiles.get("p50.0").copied().unwrap_or(0),
                p90_us: percentiles.get("p90.0").copied().unwrap_or(0),
                p95_us: percentiles.get("p95.0").copied().unwrap_or(0),
                p99_us: percentiles.get("p99.0").copied().unwrap_or(0),
            });

        // Collect violation statistics
        let violation_stats = metrics_system.violation_detector().statistics();
        snapshot.violation_stats = ViolationSnapshot {
            total_violations: violation_stats.total_violations,
            critical_violations: violation_stats
                .violations_by_severity
                .get(&crate::ViolationSeverity::Critical)
                .copied()
                .unwrap_or(0),
            error_violations: violation_stats
                .violations_by_severity
                .get(&crate::ViolationSeverity::Error)
                .copied()
                .unwrap_or(0),
            warning_violations: violation_stats
                .violations_by_severity
                .get(&crate::ViolationSeverity::Warning)
                .copied()
                .unwrap_or(0),
            info_violations: violation_stats
                .violations_by_severity
                .get(&crate::ViolationSeverity::Info)
                .copied()
                .unwrap_or(0),
            configured_thresholds: violation_stats.configured_thresholds,
            health_score: metrics_system.violation_detector().health_score(),
        };

        // Calculate overall health score
        snapshot.health_score = metrics_system.health_score();

        // Add system resource gauges if enabled
        if self.config.include_system_metrics {
            snapshot.gauges.insert(
                "memory_usage_mb".to_string(),
                (memory_stats.current_usage.max(0) as f64) / (1024.0 * 1024.0),
            );
            snapshot.gauges.insert(
                "memory_efficiency".to_string(),
                memory_stats.memory_efficiency,
            );
            snapshot.gauges.insert(
                "average_latency_ms".to_string(),
                latency_stats.average_us / 1000.0,
            );
            snapshot
                .gauges
                .insert("health_score".to_string(), snapshot.health_score);
        }

        snapshot
    }

    /// Add snapshot to historical data
    fn add_to_history(&self, snapshot: &SystemSnapshot) {
        let mut history = self.history.write();

        // Remove old data points if at capacity
        if history.len() >= self.config.max_history_points {
            history.remove(0);
        }

        // Calculate derived metrics for historical tracking
        let memory_usage_mb =
            (snapshot.memory_stats.current_usage_bytes.max(0) as f64) / (1024.0 * 1024.0);
        let average_latency_ms = snapshot
            .latency_stats
            .get("overall")
            .map(|l| l.average_us / 1000.0)
            .unwrap_or(0.0);

        let request_rate = snapshot
            .counters
            .get("requests_total")
            .map(|&total| {
                if snapshot.uptime_seconds > 0 {
                    total as f64 / snapshot.uptime_seconds as f64
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        let error_rate = {
            let total_requests = snapshot
                .counters
                .get("requests_total")
                .copied()
                .unwrap_or(0);
            let failed_requests = snapshot
                .counters
                .get("requests_failed")
                .copied()
                .unwrap_or(0);

            if total_requests > 0 {
                (failed_requests as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            }
        };

        let data_point = HistoricalDataPoint {
            timestamp: snapshot.timestamp,
            health_score: snapshot.health_score,
            memory_usage_mb,
            total_violations: snapshot.violation_stats.total_violations,
            average_latency_ms,
            request_rate,
            error_rate,
        };

        history.push(data_point);
    }

    /// Export dashboard metrics to metrics-rs
    fn export_dashboard_metrics(&self, snapshot: &SystemSnapshot) {
        // Export key dashboard metrics
        metrics::gauge!("dashboard_health_score").set(snapshot.health_score);
        metrics::gauge!("dashboard_uptime_seconds").set(snapshot.uptime_seconds as f64);
        metrics::counter!("dashboard_updates_total").increment(1);

        // Export memory metrics
        metrics::gauge!("dashboard_memory_usage_bytes")
            .set(snapshot.memory_stats.current_usage_bytes.max(0) as f64);
        metrics::gauge!("dashboard_memory_efficiency").set(snapshot.memory_stats.efficiency);

        // Export latency metrics
        if let Some(latency) = snapshot.latency_stats.get("overall") {
            metrics::gauge!("dashboard_average_latency_ms").set(latency.average_us / 1000.0);
            metrics::gauge!("dashboard_p99_latency_ms").set(latency.p99_us as f64 / 1000.0);
        }

        // Export violation metrics
        metrics::gauge!("dashboard_total_violations")
            .set(snapshot.violation_stats.total_violations as f64);
        metrics::gauge!("dashboard_critical_violations")
            .set(snapshot.violation_stats.critical_violations as f64);
    }

    /// Get current system snapshot
    pub fn current_snapshot(&self) -> SystemSnapshot {
        let snapshot = self.current_snapshot.read();
        snapshot.clone()
    }

    /// Get historical data points
    pub fn historical_data(&self, limit: Option<usize>) -> Vec<HistoricalDataPoint> {
        let history = self.history.read();

        match limit {
            Some(n) => {
                let start = history.len().saturating_sub(n);
                history[start..].to_vec()
            },
            None => history.clone(),
        }
    }

    /// Get dashboard summary statistics
    pub fn summary_stats(&self) -> DashboardSummary {
        let snapshot = self.current_snapshot();
        let history = self.history.read();

        let trend_direction = if history.len() >= 2 {
            let recent = &history[history.len() - 1];
            let previous = &history[history.len() - 2];

            if recent.health_score > previous.health_score {
                TrendDirection::Improving
            } else if recent.health_score < previous.health_score {
                TrendDirection::Degrading
            } else {
                TrendDirection::Stable
            }
        } else {
            TrendDirection::Stable
        };

        DashboardSummary {
            current_health_score: snapshot.health_score,
            total_violations: snapshot.violation_stats.total_violations,
            memory_usage_mb: (snapshot.memory_stats.current_usage_bytes.max(0) as f64)
                / (1024.0 * 1024.0),
            average_latency_ms: snapshot
                .latency_stats
                .get("overall")
                .map(|l| l.average_us / 1000.0)
                .unwrap_or(0.0),
            uptime_seconds: snapshot.uptime_seconds,
            trend_direction,
            data_points_collected: history.len(),
            last_update_ago: Duration::from_millis(
                self.created_at.elapsed().as_millis() as u64 - snapshot.timestamp,
            ),
        }
    }

    /// Get dashboard data as JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let snapshot = self.current_snapshot();
        serde_json::to_string_pretty(&snapshot)
    }

    /// Get historical data as JSON string
    pub fn history_to_json(&self, limit: Option<usize>) -> Result<String, serde_json::Error> {
        let history = self.historical_data(limit);
        serde_json::to_string_pretty(&history)
    }

    /// Reset dashboard data
    pub fn reset(&self) {
        {
            let mut current = self.current_snapshot.write();
            *current = SystemSnapshot::default();
        }

        {
            let mut history = self.history.write();
            history.clear();
        }

        self.last_update.store(0, Ordering::Relaxed);
        self.update_count.store(0, Ordering::Relaxed);

        tracing::info!("Dashboard data reset");
    }

    /// Get update statistics
    pub fn update_stats(&self) -> UpdateStats {
        UpdateStats {
            total_updates: self.update_count.load(Ordering::Relaxed),
            last_update: Duration::from_millis(self.last_update.load(Ordering::Relaxed)),
            update_rate: {
                let uptime = self.created_at.elapsed().as_secs_f64();
                if uptime > 0.0 {
                    self.update_count.load(Ordering::Relaxed) as f64 / uptime
                } else {
                    0.0
                }
            },
            uptime: self.created_at.elapsed(),
        }
    }
}

impl Default for DashboardData {
    fn default() -> Self {
        Self::new()
    }
}

/// Dashboard summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub current_health_score: f64,
    pub total_violations: u64,
    pub memory_usage_mb: f64,
    pub average_latency_ms: f64,
    pub uptime_seconds: u64,
    pub trend_direction: TrendDirection,
    pub data_points_collected: usize,
    pub last_update_ago: Duration,
}

/// Trend direction for metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

/// Update statistics for dashboard
#[derive(Debug, Clone)]
pub struct UpdateStats {
    pub total_updates: u64,
    pub last_update: Duration,
    pub update_rate: f64,
    pub uptime: Duration,
}

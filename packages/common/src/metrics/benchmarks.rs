use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::metrics::collectors::MetricCollector;
use crate::metrics::{MetricsConfig, MetricsSystem};

/// Configuration for benchmark metrics collector
#[derive(Debug, Clone, Default)]
pub struct BenchmarkCollectorConfig {
    /// Metrics system configuration
    pub metrics_config: MetricsConfig,
}

/// Configuration for benchmark dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkDashboardConfig {
    /// Maximum historical snapshots to keep
    pub max_snapshots: usize,
    /// Maximum regression alerts to keep
    pub max_alerts: usize,
    /// Plugin execution latency threshold (ms)
    pub plugin_execution_threshold_ms: f64,
    /// Search query latency threshold (ms)
    pub search_query_threshold_ms: f64,
    /// Memory usage threshold (MB)
    pub memory_threshold_mb: f64,
}

impl Default for BenchmarkDashboardConfig {
    fn default() -> Self {
        Self {
            max_snapshots: 1000,
            max_alerts: 100,
            plugin_execution_threshold_ms: 10.0,
            search_query_threshold_ms: 5.0,
            memory_threshold_mb: 100.0,
        }
    }
}

/// Benchmark metrics collector that integrates with the existing metrics system
pub struct BenchmarkMetricsCollector {
    metrics_system: MetricsSystem,
    collector: MetricCollector,
    benchmark_results: HashMap<String, BenchmarkResult>,
    config: BenchmarkCollectorConfig,
}

/// Individual benchmark result data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration_ns: u64,
    pub memory_usage: usize,
    pub throughput: f64,
    pub timestamp: std::time::SystemTime,
}

/// Benchmark snapshot for historical tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSnapshot {
    pub timestamp: std::time::SystemTime,
    pub total_operations: u64,
    pub avg_latency_ms: f64,
    pub memory_usage_mb: f64,
    pub violation_count: u64,
}

/// Benchmark dashboard for performance monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkDashboard {
    historical_data: Vec<BenchmarkSnapshot>,
    regression_alerts: Vec<RegressionAlert>,
    flamegraph_paths: HashMap<String, std::path::PathBuf>,
    config: BenchmarkDashboardConfig,
}

/// Regression alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAlert {
    pub benchmark_name: String,
    pub regression_type: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub degradation_percent: f64,
    pub timestamp: std::time::SystemTime,
}

impl BenchmarkMetricsCollector {
    /// Create a new benchmark metrics collector
    pub fn new() -> Self {
        Self::with_config(BenchmarkCollectorConfig::default())
    }

    /// Create benchmark metrics collector with custom configuration
    pub fn with_config(config: BenchmarkCollectorConfig) -> Self {
        Self {
            metrics_system: MetricsSystem::new(config.metrics_config.clone()),
            collector: MetricCollector::new(),
            benchmark_results: HashMap::new(),
            config,
        }
    }

    /// Record a benchmark result and integrate with metrics system
    pub async fn record_benchmark_result(
        &mut self,
        name: &str,
        duration: Duration,
        throughput: f64,
    ) {
        // Measure actual collection time and coordinate with collector
        let collection_start = std::time::Instant::now();

        // Record metrics through global metrics system
        metrics::histogram!("benchmark_duration_ns").record(duration.as_nanos() as f64);
        metrics::counter!("benchmark_runs_total").increment(1);
        metrics::gauge!("benchmark_throughput").set(throughput);

        // Record actual collection timing if coordination is due
        if self.collector.should_collect("benchmark_duration") {
            let collection_time = collection_start.elapsed();
            let _collection_result =
                self.collector
                    .record_collection("benchmark_duration", 1, collection_time);
        }

        // Update the metrics system
        self.metrics_system.update().await;

        // Store benchmark result
        let result = BenchmarkResult {
            name: name.to_string(),
            duration_ns: duration.as_nanos() as u64,
            memory_usage: self.get_current_memory_usage(),
            throughput,
            timestamp: std::time::SystemTime::now(),
        };

        self.benchmark_results.insert(name.to_string(), result);
    }

    /// Record memory allocation metrics for a benchmark
    pub async fn record_memory_metrics(
        &mut self,
        name: &str,
        allocated: usize,
        deallocated: usize,
    ) {
        // Measure actual collection time and coordinate with collector
        let collection_start = std::time::Instant::now();

        // Record metrics through global metrics system
        metrics::gauge!("benchmark_memory_allocated", "benchmark" => name.to_string())
            .set(allocated as f64);
        metrics::gauge!("benchmark_memory_deallocated", "benchmark" => name.to_string())
            .set(deallocated as f64);

        let net_allocation = allocated.saturating_sub(deallocated);
        metrics::gauge!("benchmark_net_allocation", "benchmark" => name.to_string())
            .set(net_allocation as f64);

        // Record actual collection timing if coordination is due
        if self.collector.should_collect("memory_metrics") {
            let collection_time = collection_start.elapsed();
            let _collection_result = self.collector.record_collection(
                "memory_metrics",
                3, // Three metrics being collected
                collection_time,
            );
        }

        // Update the metrics system
        self.metrics_system.update().await;
    }

    /// Record performance violation for regression detection
    pub async fn record_performance_violation(
        &mut self,
        name: &str,
        expected_ms: f64,
        actual_ms: f64,
    ) {
        if actual_ms > expected_ms {
            let violation_percent = ((actual_ms - expected_ms) / expected_ms) * 100.0;

            // Measure actual collection time and coordinate with collector
            let collection_start = std::time::Instant::now();

            // Record metrics through global metrics system
            metrics::gauge!("benchmark_violation_percent", "benchmark" => name.to_string())
                .set(violation_percent);
            metrics::counter!("performance_violations_total").increment(1);

            // Record actual collection timing if coordination is due
            if self.collector.should_collect("performance_violations") {
                let collection_time = collection_start.elapsed();
                let _collection_result =
                    self.collector
                        .record_collection("performance_violations", 1, collection_time);
            }

            // Update the metrics system
            self.metrics_system.update().await;
        }
    }

    /// Get benchmark results for a specific benchmark
    pub fn get_benchmark_result(&self, name: &str) -> Option<&BenchmarkResult> {
        self.benchmark_results.get(name)
    }

    /// Get all benchmark results
    pub fn get_all_results(&self) -> &HashMap<String, BenchmarkResult> {
        &self.benchmark_results
    }

    /// Export benchmark data for external analysis
    pub fn export_benchmark_data(&self) -> serde_json::Value {
        serde_json::to_value(&self.benchmark_results).unwrap_or_default()
    }

    /// Get reference to the metrics system
    pub fn metrics_system(&self) -> &MetricsSystem {
        &self.metrics_system
    }

    /// Get reference to the collector
    pub fn collector(&self) -> &MetricCollector {
        &self.collector
    }

    /// Get reference to the configuration
    pub fn config(&self) -> &BenchmarkCollectorConfig {
        &self.config
    }

    /// Get current memory usage from the memory tracker
    fn get_current_memory_usage(&self) -> usize {
        let current_usage = self.metrics_system.memory_tracker().current_usage();
        // Convert i64 to usize, handling potential negative values
        current_usage.max(0) as usize
    }
}

impl BenchmarkDashboard {
    /// Create a new benchmark dashboard
    pub fn new() -> Self {
        Self::with_config(BenchmarkDashboardConfig::default())
    }

    /// Create benchmark dashboard with custom configuration
    pub fn with_config(config: BenchmarkDashboardConfig) -> Self {
        Self {
            historical_data: Vec::new(),
            regression_alerts: Vec::new(),
            flamegraph_paths: HashMap::new(),
            config,
        }
    }

    /// Add a benchmark snapshot to historical data
    pub fn add_snapshot(&mut self, snapshot: BenchmarkSnapshot) {
        self.historical_data.push(snapshot);

        // Keep only last N snapshots to prevent unbounded growth
        if self.historical_data.len() > self.config.max_snapshots {
            self.historical_data.remove(0);
        }
    }

    /// Add a regression alert
    pub fn add_regression_alert(&mut self, alert: RegressionAlert) {
        self.regression_alerts.push(alert);

        // Keep only last N alerts
        if self.regression_alerts.len() > self.config.max_alerts {
            self.regression_alerts.remove(0);
        }
    }

    /// Register a flamegraph path for a benchmark
    pub fn register_flamegraph(&mut self, benchmark_name: String, path: std::path::PathBuf) {
        self.flamegraph_paths.insert(benchmark_name, path);
    }

    /// Generate comprehensive performance report
    pub fn generate_performance_report(&self) -> serde_json::Value {
        let mut report = serde_json::Map::new();

        // Historical trend analysis
        if !self.historical_data.is_empty() {
            let latest = &self.historical_data[self.historical_data.len() - 1];
            report.insert(
                "latest_snapshot".to_string(),
                serde_json::to_value(latest).unwrap_or_default(),
            );

            if self.historical_data.len() > 1 {
                let previous = &self.historical_data[self.historical_data.len() - 2];
                let trend = self.calculate_trend(previous, latest);
                report.insert(
                    "performance_trend".to_string(),
                    serde_json::to_value(trend).unwrap_or_default(),
                );
            }
        }

        // Regression detection results
        report.insert(
            "regression_alerts".to_string(),
            serde_json::to_value(&self.regression_alerts).unwrap_or_default(),
        );

        // Flamegraph links
        let flamegraph_links: HashMap<String, String> = self
            .flamegraph_paths
            .iter()
            .map(|(name, path)| (name.clone(), path.to_string_lossy().to_string()))
            .collect();
        report.insert(
            "flamegraph_paths".to_string(),
            serde_json::to_value(flamegraph_links).unwrap_or_default(),
        );

        // Performance target compliance
        let compliance = self.check_performance_compliance();
        report.insert(
            "target_compliance".to_string(),
            serde_json::to_value(compliance).unwrap_or_default(),
        );

        serde_json::Value::Object(report)
    }

    /// Calculate performance trend between two snapshots
    fn calculate_trend(
        &self,
        previous: &BenchmarkSnapshot,
        current: &BenchmarkSnapshot,
    ) -> HashMap<String, f64> {
        let mut trends = HashMap::new();

        // Calculate latency trend
        if previous.avg_latency_ms > 0.0 {
            let latency_change = ((current.avg_latency_ms - previous.avg_latency_ms)
                / previous.avg_latency_ms)
                * 100.0;
            trends.insert("latency_change_percent".to_string(), latency_change);
        }

        // Calculate memory trend
        if previous.memory_usage_mb > 0.0 {
            let memory_change = ((current.memory_usage_mb - previous.memory_usage_mb)
                / previous.memory_usage_mb)
                * 100.0;
            trends.insert("memory_change_percent".to_string(), memory_change);
        }

        trends
    }

    /// Check performance target compliance
    fn check_performance_compliance(&self) -> HashMap<String, bool> {
        let mut compliance = HashMap::new();

        if let Some(latest) = self.historical_data.last() {
            // Check against configured performance targets
            compliance.insert(
                "plugin_execution_under_threshold".to_string(),
                latest.avg_latency_ms < self.config.plugin_execution_threshold_ms,
            );
            compliance.insert(
                "search_query_under_threshold".to_string(),
                latest.avg_latency_ms < self.config.search_query_threshold_ms,
            );
            compliance.insert(
                "memory_under_threshold".to_string(),
                latest.memory_usage_mb < self.config.memory_threshold_mb,
            );
        }

        compliance
    }
}

impl Default for BenchmarkMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BenchmarkDashboard {
    fn default() -> Self {
        Self::new()
    }
}

//! Metric collection systems and utilities
//!
//! Zero-allocation metric collection infrastructure for gathering performance data.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use parking_lot::RwLock;

/// Metric collection coordinator
#[derive(Debug)]
pub struct MetricCollector {
    /// Collection intervals by metric type
    intervals: RwLock<HashMap<String, Duration>>,
    /// Last collection timestamps
    last_collections: RwLock<HashMap<String, AtomicU64>>,
    /// Collection counters
    collection_counts: RwLock<HashMap<String, AtomicU64>>,
    /// Collector configuration
    config: CollectorConfig,
    /// Collector creation time
    created_at: Instant,
    /// Total collections performed
    total_collections: AtomicU64,
}

/// Configuration for metric collection
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Default collection interval
    pub default_interval: Duration,
    /// Maximum number of metric types to track
    pub max_metric_types: usize,
    /// Enable collection timing metrics
    pub enable_timing: bool,
    /// Collection batch size
    pub batch_size: usize,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            default_interval: Duration::from_secs(1),
            max_metric_types: 100,
            enable_timing: true,
            batch_size: 50,
        }
    }
}

/// Collection result with timing information
#[derive(Debug, Clone)]
pub struct CollectionResult {
    pub metric_type: String,
    pub collected_count: u64,
    pub collection_time_us: u64,
    pub timestamp: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Batch collection results
#[derive(Debug, Clone)]
pub struct BatchCollectionResult {
    pub results: Vec<CollectionResult>,
    pub total_time_us: u64,
    pub successful_collections: usize,
    pub failed_collections: usize,
    pub batch_timestamp: u64,
}

impl MetricCollector {
    /// Create new metric collector
    pub fn new() -> Self {
        Self::with_config(CollectorConfig::default())
    }

    /// Create metric collector with custom configuration
    pub fn with_config(config: CollectorConfig) -> Self {
        Self {
            intervals: RwLock::new(HashMap::new()),
            last_collections: RwLock::new(HashMap::new()),
            collection_counts: RwLock::new(HashMap::new()),
            config,
            created_at: Instant::now(),
            total_collections: AtomicU64::new(0),
        }
    }

    /// Record a histogram value
    pub fn record_histogram(&self, name: &str, value: f64) {
        metrics::histogram!(name.to_string()).record(value);
    }

    /// Increment a counter
    pub fn increment_counter(&self, name: &str, value: u64) {
        metrics::counter!(name.to_string()).increment(value);
    }

    /// Record a gauge value
    pub fn record_gauge(&self, name: &str, value: f64) {
        metrics::gauge!(name.to_string()).set(value);
    }

    /// Set collection interval for a metric type
    pub fn set_interval(&self, metric_type: &str, interval: Duration) {
        let mut intervals = self.intervals.write();

        if intervals.len() >= self.config.max_metric_types {
            tracing::warn!(
                "Maximum metric types ({}) reached, ignoring interval for '{}'",
                self.config.max_metric_types,
                metric_type
            );
            return;
        }

        intervals.insert(metric_type.to_string(), interval);

        // Initialize tracking for this metric type
        {
            let mut last_collections = self.last_collections.write();
            last_collections
                .entry(metric_type.to_string())
                .or_insert_with(|| AtomicU64::new(0));
        }
        {
            let mut collection_counts = self.collection_counts.write();
            collection_counts
                .entry(metric_type.to_string())
                .or_insert_with(|| AtomicU64::new(0));
        }

        tracing::debug!(
            "Set collection interval for '{}': {:?}",
            metric_type,
            interval
        );
    }

    /// Check if it's time to collect a metric type
    #[inline(always)]
    pub fn should_collect(&self, metric_type: &str) -> bool {
        let intervals = self.intervals.read();
        let interval = intervals
            .get(metric_type)
            .copied()
            .unwrap_or(self.config.default_interval);

        let last_collections = self.last_collections.read();
        if let Some(last_collection) = last_collections.get(metric_type) {
            let last_collection_millis = last_collection.load(Ordering::Relaxed);
            let now_millis = self.created_at.elapsed().as_millis() as u64;

            (now_millis - last_collection_millis) >= interval.as_millis() as u64
        } else {
            true // Never collected before
        }
    }

    /// Record a collection event
    pub fn record_collection(
        &self,
        metric_type: &str,
        collected_count: u64,
        collection_time: Duration,
    ) -> CollectionResult {
        let now_millis = self.created_at.elapsed().as_millis() as u64;

        // Update last collection timestamp
        if let Some(last_collections) = self.last_collections.read().get(metric_type) {
            last_collections.store(now_millis, Ordering::Relaxed);
        }

        // Increment collection count
        if let Some(collection_counts) = self.collection_counts.read().get(metric_type) {
            collection_counts.fetch_add(1, Ordering::Relaxed);
        }

        // Increment total collections
        self.total_collections.fetch_add(1, Ordering::Relaxed);

        let result = CollectionResult {
            metric_type: metric_type.to_string(),
            collected_count,
            collection_time_us: collection_time.as_micros() as u64,
            timestamp: now_millis,
            success: true,
            error_message: None,
        };

        // Export collection metrics
        if self.config.enable_timing {
            metrics::counter!("metric_collections_total").increment(1);
            metrics::histogram!("metric_collection_duration_us")
                .record(collection_time.as_micros() as f64);
            metrics::counter!("metric_data_points_collected").increment(collected_count);
        }

        tracing::debug!(
            "Recorded collection for '{}': {} points in {:?}",
            metric_type,
            collected_count,
            collection_time
        );

        result
    }

    /// Record a failed collection
    pub fn record_collection_error(&self, metric_type: &str, error: &str) -> CollectionResult {
        let now_millis = self.created_at.elapsed().as_millis() as u64;

        let result = CollectionResult {
            metric_type: metric_type.to_string(),
            collected_count: 0,
            collection_time_us: 0,
            timestamp: now_millis,
            success: false,
            error_message: Some(error.to_string()),
        };

        // Export error metrics
        metrics::counter!("metric_collection_errors_total").increment(1);

        tracing::error!("Collection failed for '{}': {}", metric_type, error);

        result
    }

    /// Collect metrics from all registered types that are due
    pub fn collect_due_metrics(
        &self,
        metrics_system: &crate::MetricsSystem,
    ) -> BatchCollectionResult {
        let batch_start = Instant::now();
        let batch_timestamp = self.created_at.elapsed().as_millis() as u64;
        let mut results = Vec::new();

        let intervals = self.intervals.read();
        let metric_types: Vec<String> = intervals.keys().cloned().collect();
        drop(intervals);

        for metric_type in metric_types {
            if self.should_collect(&metric_type) {
                let collection_start = Instant::now();

                let result = match metric_type.as_str() {
                    "counters" => self.collect_counters(metrics_system),
                    "memory" => self.collect_memory(metrics_system),
                    "latency" => self.collect_latency(metrics_system),
                    "violations" => self.collect_violations(metrics_system),
                    "system" => self.collect_system_metrics(),
                    _ => self.record_collection_error(&metric_type, "Unknown metric type"),
                };

                let collection_time = collection_start.elapsed();
                let final_result = if result.success {
                    self.record_collection(&metric_type, result.collected_count, collection_time)
                } else {
                    result
                };

                results.push(final_result);
            }
        }

        let total_time = batch_start.elapsed();
        let successful_collections = results.iter().filter(|r| r.success).count();
        let failed_collections = results.len() - successful_collections;

        // Export batch metrics
        metrics::histogram!("metric_batch_collection_duration_us")
            .record(total_time.as_micros() as f64);
        metrics::counter!("metric_batch_collections_total").increment(1);

        if failed_collections > 0 {
            metrics::counter!("metric_batch_collection_failures_total")
                .increment(failed_collections as u64);
        }

        BatchCollectionResult {
            results,
            total_time_us: total_time.as_micros() as u64,
            successful_collections,
            failed_collections,
            batch_timestamp,
        }
    }

    /// Collect counter metrics
    fn collect_counters(&self, metrics_system: &crate::MetricsSystem) -> CollectionResult {
        let snapshot = metrics_system.counters().snapshot();
        let collected_count = snapshot.counters.len() as u64;

        // Export counter values
        for (_name, value) in snapshot.counters {
            metrics::counter!("collected_counter").increment(value);
        }

        CollectionResult {
            metric_type: "counters".to_string(),
            collected_count,
            collection_time_us: 0, // Will be set by caller
            timestamp: 0,          // Will be set by caller
            success: true,
            error_message: None,
        }
    }

    /// Collect memory metrics
    fn collect_memory(&self, metrics_system: &crate::MetricsSystem) -> CollectionResult {
        let stats = metrics_system.memory_tracker().statistics();

        // Export memory metrics
        metrics::gauge!("collected_memory_current_usage").set(stats.current_usage.max(0) as f64);
        metrics::gauge!("collected_memory_peak_usage").set(stats.peak_usage as f64);
        metrics::gauge!("collected_memory_efficiency").set(stats.memory_efficiency);
        metrics::counter!("collected_memory_allocations").increment(stats.allocations);
        metrics::counter!("collected_memory_deallocations").increment(stats.deallocations);

        CollectionResult {
            metric_type: "memory".to_string(),
            collected_count: 5, // Number of metrics collected
            collection_time_us: 0,
            timestamp: 0,
            success: true,
            error_message: None,
        }
    }

    /// Collect latency metrics
    fn collect_latency(&self, metrics_system: &crate::MetricsSystem) -> CollectionResult {
        let stats = metrics_system.latency_tracker().statistics();
        let percentiles = metrics_system
            .latency_tracker()
            .percentiles(&[50.0, 90.0, 95.0, 99.0, 99.9]);

        // Export latency metrics
        metrics::gauge!("collected_latency_average_us").set(stats.average_us);
        metrics::gauge!("collected_latency_min_us").set(stats.min_us as f64);
        metrics::gauge!("collected_latency_max_us").set(stats.max_us as f64);
        metrics::counter!("collected_latency_measurements").increment(stats.total_measurements);

        let _percentile_count = percentiles.len();
        for (percentile, value) in &percentiles {
            let percentile_name = percentile.replace('.', "_");
            let percentile_key = format!("collected_latency_p{}_us", percentile_name);
            metrics::gauge!(percentile_key).set(*value as f64);
        }

        CollectionResult {
            metric_type: "latency".to_string(),
            collected_count: 4 + percentiles.len() as u64,
            collection_time_us: 0,
            timestamp: 0,
            success: true,
            error_message: None,
        }
    }

    /// Collect violation metrics
    fn collect_violations(&self, metrics_system: &crate::MetricsSystem) -> CollectionResult {
        let stats = metrics_system.violation_detector().statistics();

        // Export violation metrics
        metrics::counter!("collected_violations_total").increment(stats.total_violations);
        metrics::gauge!("collected_violation_health_score")
            .set(metrics_system.violation_detector().health_score());
        metrics::gauge!("collected_violation_thresholds").set(stats.configured_thresholds as f64);

        let _violations_count = stats.violations_by_severity.len();
        for (severity, count) in &stats.violations_by_severity {
            let severity_name = format!("{:?}", severity).to_lowercase();
            metrics::counter!(format!("collected_violations_{}", severity_name)).increment(*count);
        }

        CollectionResult {
            metric_type: "violations".to_string(),
            collected_count: 3 + stats.violations_by_severity.len() as u64,
            collection_time_us: 0,
            timestamp: 0,
            success: true,
            error_message: None,
        }
    }

    /// Collect system resource metrics with real system monitoring
    fn collect_system_metrics(&self) -> CollectionResult {
        // Real system resource collection using sysinfo
        let start = std::time::Instant::now();
        let mut error_message = None;

        let collected_count = match self.collect_system_resources() {
            Ok(count) => {
                // Also collect basic runtime metrics
                let uptime = self.created_at.elapsed();
                metrics::gauge!("collected_system_uptime_seconds").set(uptime.as_secs() as f64);
                metrics::counter!("collected_system_collections")
                    .increment(self.total_collections.load(Ordering::Relaxed));
                count + 2
            },
            Err(e) => {
                error_message = Some(format!("System resource collection failed: {}", e));
                tracing::error!("Failed to collect system resources: {}", e);

                // Fall back to basic metrics only
                let uptime = self.created_at.elapsed();
                metrics::gauge!("collected_system_uptime_seconds").set(uptime.as_secs() as f64);
                metrics::counter!("collected_system_collections")
                    .increment(self.total_collections.load(Ordering::Relaxed));
                2
            },
        };

        CollectionResult {
            metric_type: "system".to_string(),
            collected_count,
            collection_time_us: start.elapsed().as_micros() as u64,
            timestamp: self.created_at.elapsed().as_millis() as u64,
            success: error_message.is_none(),
            error_message,
        }
    }

    /// Collect detailed system resource metrics using sysinfo
    fn collect_system_resources(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        use sysinfo::System;

        let mut collected_count = 0u64;

        // Initialize system info
        let mut system = System::new_all();
        system.refresh_all();

        // CPU metrics
        let cpu_usage = system.global_cpu_usage();
        metrics::gauge!("collected_system_cpu_usage_percent").set(cpu_usage as f64);
        collected_count += 1;

        let cpu_count = system.cpus().len();
        metrics::gauge!("collected_system_cpu_count").set(cpu_count as f64);
        collected_count += 1;

        // Per-CPU metrics (limited to avoid metric explosion)
        for (i, cpu) in system.cpus().iter().enumerate().take(8) {
            // Limit to first 8 CPUs
            let cpu_name = format!("collected_system_cpu_{}_usage_percent", i);
            metrics::gauge!(cpu_name).set(cpu.cpu_usage() as f64);
            collected_count += 1;
        }

        // Memory metrics
        let total_memory = system.total_memory();
        let available_memory = system.available_memory();
        let used_memory = total_memory - available_memory;
        let memory_usage_percent = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        metrics::gauge!("collected_system_memory_total_bytes").set(total_memory as f64);
        metrics::gauge!("collected_system_memory_available_bytes").set(available_memory as f64);
        metrics::gauge!("collected_system_memory_used_bytes").set(used_memory as f64);
        metrics::gauge!("collected_system_memory_usage_percent").set(memory_usage_percent);
        collected_count += 4;

        // Swap metrics
        let total_swap = system.total_swap();
        let used_swap = system.used_swap();
        let swap_usage_percent = if total_swap > 0 {
            (used_swap as f64 / total_swap as f64) * 100.0
        } else {
            0.0
        };

        metrics::gauge!("collected_system_swap_total_bytes").set(total_swap as f64);
        metrics::gauge!("collected_system_swap_used_bytes").set(used_swap as f64);
        metrics::gauge!("collected_system_swap_usage_percent").set(swap_usage_percent);
        collected_count += 3;

        // Load averages (where available)
        let load_avg = System::load_average();
        metrics::gauge!("collected_system_load_1m").set(load_avg.one);
        metrics::gauge!("collected_system_load_5m").set(load_avg.five);
        metrics::gauge!("collected_system_load_15m").set(load_avg.fifteen);
        collected_count += 3;

        // Process count
        let process_count = system.processes().len();
        metrics::gauge!("collected_system_process_count").set(process_count as f64);
        collected_count += 1;

        // Disk metrics (limited to avoid metric explosion)
        let mut total_disk_space = 0u64;
        let mut available_disk_space = 0u64;
        let mut disk_count = 0;

        for disk in sysinfo::Disks::new_with_refreshed_list().iter().take(10) {
            // Limit to first 10 disks
            let mount_point = disk.mount_point().to_string_lossy();
            let total_space = disk.total_space();
            let available_space = disk.available_space();

            total_disk_space += total_space;
            available_disk_space += available_space;
            disk_count += 1;

            // Individual disk metrics (sanitize mount point for metric name)
            let safe_mount = mount_point.replace(['/', '\\', ':', ' '], "_");
            let disk_usage_percent = if total_space > 0 {
                ((total_space - available_space) as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };

            metrics::gauge!(format!("collected_system_disk_{}_total_bytes", safe_mount))
                .set(total_space as f64);
            metrics::gauge!(format!(
                "collected_system_disk_{}_available_bytes",
                safe_mount
            ))
            .set(available_space as f64);
            metrics::gauge!(format!(
                "collected_system_disk_{}_usage_percent",
                safe_mount
            ))
            .set(disk_usage_percent);
            collected_count += 3;
        }

        // Total disk metrics
        if disk_count > 0 {
            let total_disk_usage_percent = if total_disk_space > 0 {
                ((total_disk_space - available_disk_space) as f64 / total_disk_space as f64) * 100.0
            } else {
                0.0
            };

            metrics::gauge!("collected_system_disk_total_bytes").set(total_disk_space as f64);
            metrics::gauge!("collected_system_disk_available_bytes")
                .set(available_disk_space as f64);
            metrics::gauge!("collected_system_disk_usage_percent").set(total_disk_usage_percent);
            metrics::gauge!("collected_system_disk_count").set(disk_count as f64);
            collected_count += 4;
        }

        // Network metrics (limited to avoid metric explosion)
        let mut total_bytes_received = 0u64;
        let mut total_bytes_transmitted = 0u64;
        let mut network_interface_count = 0;

        for (interface_name, network) in
            sysinfo::Networks::new_with_refreshed_list().iter().take(10)
        {
            // Limit to first 10 interfaces
            let bytes_received = network.total_received();
            let bytes_transmitted = network.total_transmitted();
            let packets_received = network.total_packets_received();
            let packets_transmitted = network.total_packets_transmitted();

            total_bytes_received += bytes_received;
            total_bytes_transmitted += bytes_transmitted;
            network_interface_count += 1;

            // Individual interface metrics (sanitize interface name)
            let safe_interface = interface_name.replace([' ', '-', '.', ':'], "_");

            metrics::gauge!(format!(
                "collected_system_network_{}_bytes_received",
                safe_interface
            ))
            .set(bytes_received as f64);
            metrics::gauge!(format!(
                "collected_system_network_{}_bytes_transmitted",
                safe_interface
            ))
            .set(bytes_transmitted as f64);
            metrics::gauge!(format!(
                "collected_system_network_{}_packets_received",
                safe_interface
            ))
            .set(packets_received as f64);
            metrics::gauge!(format!(
                "collected_system_network_{}_packets_transmitted",
                safe_interface
            ))
            .set(packets_transmitted as f64);
            collected_count += 4;
        }

        // Total network metrics
        if network_interface_count > 0 {
            metrics::gauge!("collected_system_network_total_bytes_received")
                .set(total_bytes_received as f64);
            metrics::gauge!("collected_system_network_total_bytes_transmitted")
                .set(total_bytes_transmitted as f64);
            metrics::gauge!("collected_system_network_interface_count")
                .set(network_interface_count as f64);
            collected_count += 3;
        }

        // System uptime from sysinfo
        let boot_time = System::boot_time();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let system_uptime = current_time - boot_time;

        metrics::gauge!("collected_system_uptime_seconds").set(system_uptime as f64);
        metrics::gauge!("collected_system_boot_time").set(boot_time as f64);
        collected_count += 2;

        tracing::debug!(
            "Collected {} system metrics: CPU, memory, swap, load, processes, {} disks, {} \
             network interfaces",
            collected_count,
            disk_count,
            network_interface_count
        );

        Ok(collected_count)
    }

    /// Get collection statistics for a metric type
    pub fn collection_stats(&self, metric_type: &str) -> Option<CollectionStats> {
        let collection_counts = self.collection_counts.read();
        let last_collections = self.last_collections.read();
        let intervals = self.intervals.read();

        let collection_count = collection_counts.get(metric_type)?.load(Ordering::Relaxed);
        let last_collection = last_collections.get(metric_type)?.load(Ordering::Relaxed);
        let interval = intervals
            .get(metric_type)
            .copied()
            .unwrap_or(self.config.default_interval);

        let uptime = self.created_at.elapsed();
        let collection_rate = if uptime.as_secs() > 0 {
            collection_count as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        let time_since_last = if last_collection > 0 {
            let now_millis = self.created_at.elapsed().as_millis() as u64;
            Duration::from_millis(now_millis - last_collection)
        } else {
            Duration::ZERO
        };

        Some(CollectionStats {
            metric_type: metric_type.to_string(),
            total_collections: collection_count,
            collection_rate,
            configured_interval: interval,
            time_since_last_collection: time_since_last,
            is_overdue: time_since_last > interval,
        })
    }

    /// Get overall collection statistics
    pub fn overall_stats(&self) -> OverallCollectionStats {
        let intervals = self.intervals.read();
        let _collection_counts = self.collection_counts.read();

        let configured_types = intervals.len();
        let total_collections = self.total_collections.load(Ordering::Relaxed);

        let uptime = self.created_at.elapsed();
        let overall_rate = if uptime.as_secs() > 0 {
            total_collections as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        let mut active_types = 0;
        let mut overdue_types = 0;

        for metric_type in intervals.keys() {
            if let Some(stats) = self.collection_stats(metric_type) {
                if stats.total_collections > 0 {
                    active_types += 1;
                }
                if stats.is_overdue {
                    overdue_types += 1;
                }
            }
        }

        OverallCollectionStats {
            configured_types,
            active_types,
            overdue_types,
            total_collections,
            overall_collection_rate: overall_rate,
            uptime,
        }
    }

    /// Reset collection statistics
    pub fn reset(&self) {
        let collection_counts = self.collection_counts.read();
        for (_, count) in collection_counts.iter() {
            count.store(0, Ordering::Relaxed);
        }

        let last_collections = self.last_collections.read();
        for (_, last_collection) in last_collections.iter() {
            last_collection.store(0, Ordering::Relaxed);
        }

        self.total_collections.store(0, Ordering::Relaxed);

        tracing::info!("Metric collector statistics reset");
    }

    /// Get list of configured metric types
    pub fn configured_types(&self) -> Vec<String> {
        let intervals = self.intervals.read();
        intervals.keys().cloned().collect()
    }

    /// Remove a metric type from collection
    pub fn remove_type(&self, metric_type: &str) {
        let mut intervals = self.intervals.write();
        intervals.remove(metric_type);

        let mut collection_counts = self.collection_counts.write();
        collection_counts.remove(metric_type);

        let mut last_collections = self.last_collections.write();
        last_collections.remove(metric_type);

        tracing::debug!("Removed metric type '{}' from collection", metric_type);
    }
}

impl Default for MetricCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Collection statistics for a specific metric type
#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub metric_type: String,
    pub total_collections: u64,
    pub collection_rate: f64,
    pub configured_interval: Duration,
    pub time_since_last_collection: Duration,
    pub is_overdue: bool,
}

/// Overall collection statistics
#[derive(Debug, Clone)]
pub struct OverallCollectionStats {
    pub configured_types: usize,
    pub active_types: usize,
    pub overdue_types: usize,
    pub total_collections: u64,
    pub overall_collection_rate: f64,
    pub uptime: Duration,
}

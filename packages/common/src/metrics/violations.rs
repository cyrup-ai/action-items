//! Performance violation detection and alerting
//!
//! Zero-allocation threshold-based monitoring for detecting performance violations.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Performance violation detector with configurable thresholds
#[derive(Debug)]
pub struct ViolationDetector {
    /// Threshold configurations by metric name
    thresholds: RwLock<HashMap<String, ViolationThreshold>>,
    /// Violation counts by metric name
    violation_counts: RwLock<HashMap<String, AtomicU64>>,
    /// Last violation timestamps
    last_violations: RwLock<HashMap<String, AtomicU64>>,
    /// Total violations across all metrics
    total_violations: AtomicU64,
    /// Detector creation time
    created_at: Instant,
    /// Configuration
    config: ViolationDetectorConfig,
}

/// Configuration for violation detector
#[derive(Debug, Clone)]
pub struct ViolationDetectorConfig {
    /// Default violation threshold
    pub default_threshold: f64,
    /// Cooldown period between violations (seconds)
    pub cooldown_seconds: u64,
    /// Maximum number of metrics to track
    pub max_metrics: usize,
    /// Enable violation logging
    pub enable_logging: bool,
    /// Enable persistent storage of violations
    pub enable_persistent_storage: bool,
    /// Storage backend configuration
    pub storage_backend: ViolationStorageBackend,
}

/// Storage backend options for violation persistence
#[derive(Debug, Clone)]
pub enum ViolationStorageBackend {
    /// Memory-only storage (no persistence)
    Memory,
    /// JSON lines file storage
    JsonFile(std::path::PathBuf),
}

/// Stored violation record for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationRecord {
    /// The violation that occurred
    pub violation: Violation,
    /// When it was stored
    pub stored_at: SystemTime,
}

/// Violation storage errors
#[derive(Debug, thiserror::Error)]
pub enum ViolationStorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Storage backend error: {0}")]
    Backend(String),
}

impl Default for ViolationDetectorConfig {
    fn default() -> Self {
        Self {
            default_threshold: 1000.0, // Default 1000ms for latency
            cooldown_seconds: 60,      // 1 minute cooldown
            max_metrics: 1000,
            enable_logging: true,
            enable_persistent_storage: false,
            storage_backend: ViolationStorageBackend::Memory,
        }
    }
}

/// Violation threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationThreshold {
    /// Threshold value
    pub threshold: f64,
    /// Threshold type
    pub threshold_type: ThresholdType,
    /// Violation severity
    pub severity: ViolationSeverity,
    /// Description of what this threshold monitors
    pub description: String,
}

/// Type of threshold comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdType {
    /// Value must be less than threshold
    LessThan,
    /// Value must be greater than threshold
    GreaterThan,
    /// Value must be less than or equal to threshold
    LessThanOrEqual,
    /// Value must be greater than or equal to threshold
    GreaterThanOrEqual,
}

/// Severity of violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Represents a specific violation that occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Name of the metric that violated the threshold
    pub metric_name: String,
    /// The threshold configuration that was violated
    pub threshold: ViolationThreshold,
    /// The actual value that caused the violation
    pub actual_value: f64,
    /// When the violation occurred (seconds since detector creation)
    pub timestamp: u64,
    /// Severity of the violation
    pub severity: ViolationSeverity,
    /// Human-readable description of the violation
    pub description: String,
}

impl ViolationDetector {
    /// Create new violation detector
    pub fn new() -> Self {
        Self::with_config(ViolationDetectorConfig::default())
    }

    /// Create violation detector with custom configuration
    pub fn with_config(config: ViolationDetectorConfig) -> Self {
        let detector = Self {
            thresholds: RwLock::new(HashMap::new()),
            violation_counts: RwLock::new(HashMap::new()),
            last_violations: RwLock::new(HashMap::new()),
            total_violations: AtomicU64::new(0),
            created_at: Instant::now(),
            config,
        };

        // Setup default thresholds
        detector.setup_default_thresholds();
        detector
    }

    /// Setup default performance thresholds
    fn setup_default_thresholds(&self) {
        self.set_threshold("latency_ms", ViolationThreshold {
            threshold: 1000.0,
            threshold_type: ThresholdType::LessThan,
            severity: ViolationSeverity::Warning,
            description: "Request latency should be under 1 second".to_string(),
        });

        self.set_threshold("memory_usage_mb", ViolationThreshold {
            threshold: 1024.0,
            threshold_type: ThresholdType::LessThan,
            severity: ViolationSeverity::Error,
            description: "Memory usage should be under 1GB".to_string(),
        });

        self.set_threshold("cpu_usage_percent", ViolationThreshold {
            threshold: 80.0,
            threshold_type: ThresholdType::LessThan,
            severity: ViolationSeverity::Warning,
            description: "CPU usage should be under 80%".to_string(),
        });

        self.set_threshold("error_rate_percent", ViolationThreshold {
            threshold: 5.0,
            threshold_type: ThresholdType::LessThan,
            severity: ViolationSeverity::Error,
            description: "Error rate should be under 5%".to_string(),
        });

        self.set_threshold("queue_depth", ViolationThreshold {
            threshold: 1000.0,
            threshold_type: ThresholdType::LessThan,
            severity: ViolationSeverity::Warning,
            description: "Queue depth should be under 1000 items".to_string(),
        });
    }

    /// Set threshold for a metric
    pub fn set_threshold(&self, metric: &str, threshold: ViolationThreshold) {
        let mut thresholds = self.thresholds.write();

        if thresholds.len() >= self.config.max_metrics {
            tracing::warn!(
                "Maximum number of metrics ({}) reached, ignoring threshold for '{}'",
                self.config.max_metrics,
                metric
            );
            return;
        }

        thresholds.insert(metric.to_string(), threshold);

        // Initialize violation tracking for this metric
        {
            let mut counts = self.violation_counts.write();
            counts
                .entry(metric.to_string())
                .or_insert_with(|| AtomicU64::new(0));
        }
        {
            let mut last_violations = self.last_violations.write();
            last_violations
                .entry(metric.to_string())
                .or_insert_with(|| AtomicU64::new(0));
        }

        tracing::debug!(
            "Set threshold for metric '{}': {:?}",
            metric,
            thresholds.get(metric)
        );
    }

    /// Remove threshold for a metric
    pub fn remove_threshold(&self, metric: &str) {
        let mut thresholds = self.thresholds.write();
        thresholds.remove(metric);

        let mut counts = self.violation_counts.write();
        counts.remove(metric);

        let mut last_violations = self.last_violations.write();
        last_violations.remove(metric);

        tracing::debug!("Removed threshold for metric '{}'", metric);
    }

    /// Check if value violates threshold
    #[inline(always)]
    pub fn check(&self, metric: &str, value: f64) -> bool {
        let thresholds = self.thresholds.read();

        if let Some(threshold_config) = thresholds.get(metric) {
            let violates = self.check_threshold_violation(value, threshold_config);

            if violates {
                self.record_violation(metric, value, threshold_config);
            }

            violates
        } else {
            // Use default threshold if no specific threshold is set
            let violates = value > self.config.default_threshold;
            if violates {
                self.record_default_violation(metric, value);
            }
            violates
        }
    }

    /// Check threshold violation based on type
    #[inline(always)]
    fn check_threshold_violation(&self, value: f64, threshold: &ViolationThreshold) -> bool {
        match threshold.threshold_type {
            ThresholdType::LessThan => value >= threshold.threshold,
            ThresholdType::GreaterThan => value <= threshold.threshold,
            ThresholdType::LessThanOrEqual => value > threshold.threshold,
            ThresholdType::GreaterThanOrEqual => value < threshold.threshold,
        }
    }

    /// Record a violation
    fn record_violation(&self, metric: &str, value: f64, threshold: &ViolationThreshold) {
        // Check cooldown period
        if !self.is_cooldown_expired(metric) {
            return;
        }

        // Increment violation count
        if let Some(counts) = self.violation_counts.read().get(metric) {
            counts.fetch_add(1, Ordering::Relaxed);
        }

        // Update last violation timestamp
        let now_millis = self.created_at.elapsed().as_millis() as u64;
        if let Some(last_violations) = self.last_violations.read().get(metric) {
            last_violations.store(now_millis, Ordering::Relaxed);
        }

        // Increment total violations
        self.total_violations.fetch_add(1, Ordering::Relaxed);

        // Log violation if enabled
        if self.config.enable_logging {
            match threshold.severity {
                ViolationSeverity::Info => {
                    tracing::info!(
                        "Performance violation: {} = {:.2} (threshold: {:.2}, {})",
                        metric,
                        value,
                        threshold.threshold,
                        threshold.description
                    );
                },
                ViolationSeverity::Warning => {
                    tracing::warn!(
                        "Performance violation: {} = {:.2} (threshold: {:.2}, {})",
                        metric,
                        value,
                        threshold.threshold,
                        threshold.description
                    );
                },
                ViolationSeverity::Error => {
                    tracing::error!(
                        "Performance violation: {} = {:.2} (threshold: {:.2}, {})",
                        metric,
                        value,
                        threshold.threshold,
                        threshold.description
                    );
                },
                ViolationSeverity::Critical => {
                    tracing::error!(
                        "CRITICAL performance violation: {} = {:.2} (threshold: {:.2}, {})",
                        metric,
                        value,
                        threshold.threshold,
                        threshold.description
                    );
                },
            }
        }

        // Export to metrics-rs
        metrics::counter!("performance_violations_total").increment(1);
    }

    /// Record violation with default threshold
    fn record_default_violation(&self, metric: &str, value: f64) {
        let default_threshold = ViolationThreshold {
            threshold: self.config.default_threshold,
            threshold_type: ThresholdType::LessThan,
            severity: ViolationSeverity::Warning,
            description: "Default threshold violation".to_string(),
        };

        self.record_violation(metric, value, &default_threshold);
    }

    /// Check if cooldown period has expired for a metric
    fn is_cooldown_expired(&self, metric: &str) -> bool {
        let last_violations = self.last_violations.read();

        if let Some(last_violation) = last_violations.get(metric) {
            let last_violation_millis = last_violation.load(Ordering::Relaxed);
            if last_violation_millis == 0 {
                return true; // Never violated before
            }

            let now_millis = self.created_at.elapsed().as_millis() as u64;
            let elapsed_seconds = (now_millis - last_violation_millis) / 1000;

            elapsed_seconds >= self.config.cooldown_seconds
        } else {
            true // No previous violation recorded
        }
    }

    /// Get violation count for a metric
    pub fn violation_count(&self, metric: &str) -> u64 {
        let counts = self.violation_counts.read();
        counts
            .get(metric)
            .map(|count| count.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get total violation count across all metrics
    pub fn total_violations(&self) -> u64 {
        self.total_violations.load(Ordering::Relaxed)
    }

    /// Get time since last violation for a metric
    pub fn time_since_last_violation(&self, metric: &str) -> Option<Duration> {
        let last_violations = self.last_violations.read();

        last_violations.get(metric).and_then(|last_violation| {
            let last_violation_millis = last_violation.load(Ordering::Relaxed);
            if last_violation_millis == 0 {
                None
            } else {
                let now_millis = self.created_at.elapsed().as_millis() as u64;
                Some(Duration::from_millis(now_millis - last_violation_millis))
            }
        })
    }

    /// Check all configured thresholds against current system state
    pub fn check_all_thresholds(&self, metrics_system: &crate::MetricsSystem) {
        let thresholds = self.thresholds.read();
        let threshold_count = thresholds.len();

        if threshold_count > 0 {
            metrics::gauge!("violation_thresholds_configured").set(threshold_count as f64);

            let now = self.created_at.elapsed().as_secs();

            for (metric_name, threshold) in thresholds.iter() {
                let current_value = self.get_current_metric_value(metric_name, metrics_system);

                if let Some(value) = current_value {
                    let violated = match threshold.threshold_type {
                        ThresholdType::LessThan => value >= threshold.threshold,
                        ThresholdType::GreaterThan => value <= threshold.threshold,
                        ThresholdType::LessThanOrEqual => value > threshold.threshold,
                        ThresholdType::GreaterThanOrEqual => value < threshold.threshold,
                    };

                    if violated {
                        // Check cooldown period
                        let last_violations = self.last_violations.read();
                        let should_record =
                            if let Some(last_violation) = last_violations.get(metric_name) {
                                let last_violation_time = last_violation.load(Ordering::Relaxed);
                                (now - last_violation_time) >= self.config.cooldown_seconds
                            } else {
                                true
                            };

                        if should_record {
                            // Record violation
                            drop(last_violations);
                            let mut last_violations = self.last_violations.write();
                            last_violations
                                .entry(metric_name.clone())
                                .or_insert_with(|| AtomicU64::new(0))
                                .store(now, Ordering::Relaxed);
                            drop(last_violations);

                            // Update violation counts
                            let violation_counts = self.violation_counts.read();
                            if let Some(count) = violation_counts.get(metric_name) {
                                count.fetch_add(1, Ordering::Relaxed);
                            } else {
                                drop(violation_counts);
                                let mut violation_counts = self.violation_counts.write();
                                violation_counts
                                    .entry(metric_name.clone())
                                    .or_insert_with(|| AtomicU64::new(0))
                                    .fetch_add(1, Ordering::Relaxed);
                            }

                            // Update total violations
                            self.total_violations.fetch_add(1, Ordering::Relaxed);

                            // Export violation metrics
                            metrics::counter!("threshold_violations_total").increment(1);
                            metrics::gauge!("threshold_violation_value").set(value);
                            metrics::gauge!("threshold_violation_limit").set(threshold.threshold);

                            // Log violation if enabled
                            if self.config.enable_logging {
                                tracing::warn!(
                                    "Threshold violation detected: {} = {} (threshold: {} {:?})",
                                    metric_name,
                                    value,
                                    threshold.threshold,
                                    threshold.threshold_type
                                );
                            }

                            // Create violation record
                            let violation = Violation {
                                metric_name: metric_name.clone(),
                                threshold: threshold.clone(),
                                actual_value: value,
                                timestamp: now,
                                severity: threshold.severity,
                                description: format!(
                                    "{} violated {} threshold of {}",
                                    metric_name,
                                    match threshold.threshold_type {
                                        ThresholdType::LessThan => "less-than",
                                        ThresholdType::GreaterThan => "greater-than",
                                        ThresholdType::LessThanOrEqual => "less-than-or-equal",
                                        ThresholdType::GreaterThanOrEqual =>
                                            "greater-than-or-equal",
                                    },
                                    threshold.threshold
                                ),
                            };

                            // Store violation for history using configurable persistent storage
                            if self.config.enable_persistent_storage {
                                let violation_data = ViolationRecord {
                                    violation: violation.clone(),
                                    stored_at: SystemTime::now(),
                                };

                                match &self.config.storage_backend {
                                    ViolationStorageBackend::JsonFile(path) => {
                                        if let Ok(json) = serde_json::to_string(&violation_data) {
                                            // Append to JSON lines file for efficient storage
                                            let _ = std::fs::OpenOptions::new()
                                                .create(true)
                                                .append(true)
                                                .open(path)
                                                .and_then(|mut file| {
                                                    use std::io::Write;
                                                    writeln!(file, "{}", json)
                                                });
                                        }
                                    },
                                    ViolationStorageBackend::Memory => {
                                        // Store in memory-based history (existing behavior)
                                        tracing::info!(
                                            violation.metric_name,
                                            violation.actual_value,
                                            violation.threshold.threshold,
                                            severity = ?violation.severity,
                                            "violation_recorded"
                                        );
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get current value for a specific metric from the metrics system
    fn get_current_metric_value(
        &self,
        metric_name: &str,
        metrics_system: &crate::MetricsSystem,
    ) -> Option<f64> {
        match metric_name {
            // Memory metrics
            "memory_current_usage" => {
                let stats = metrics_system.memory_tracker().statistics();
                Some(stats.current_usage.max(0) as f64)
            },
            "memory_peak_usage" => {
                let stats = metrics_system.memory_tracker().statistics();
                Some(stats.peak_usage as f64)
            },
            "memory_efficiency" => {
                let stats = metrics_system.memory_tracker().statistics();
                Some(stats.memory_efficiency)
            },
            "memory_allocations" => {
                let stats = metrics_system.memory_tracker().statistics();
                Some(stats.allocations as f64)
            },

            // Latency metrics
            "latency_average_us" => {
                let stats = metrics_system.latency_tracker().statistics();
                Some(stats.average_us)
            },
            "latency_max_us" => {
                let stats = metrics_system.latency_tracker().statistics();
                Some(stats.max_us as f64)
            },
            "latency_min_us" => {
                let stats = metrics_system.latency_tracker().statistics();
                Some(stats.min_us as f64)
            },

            // Counter metrics
            metric if metric.starts_with("counter_") => {
                let counter_name = &metric[8..]; // Remove "counter_" prefix
                let snapshot = metrics_system.counters().snapshot();
                snapshot
                    .counters
                    .get(counter_name)
                    .copied()
                    .map(|v| v as f64)
            },

            // Percentile metrics
            metric if metric.starts_with("latency_p") && metric.ends_with("_us") => {
                // Extract percentile value (e.g., "latency_p95_us" -> 95.0)
                let percentile_str = &metric[9..metric.len() - 3]; // Remove "latency_p" and "_us"
                let percentile_str = percentile_str.replace('_', "."); // Convert "95_5" to "95.5"

                if let Ok(percentile) = percentile_str.parse::<f64>() {
                    let percentiles = metrics_system.latency_tracker().percentiles(&[percentile]);
                    percentiles.get(&percentile_str).copied().map(|v| v as f64)
                } else {
                    None
                }
            },

            // Default case - try to find in dashboard if available
            _ => {
                // In a more advanced implementation, this could query additional metric sources
                // or plugin-specific metrics. For now, we return None for unknown metrics.
                None
            },
        }
    }

    /// Get violation statistics
    pub fn statistics(&self) -> ViolationStatistics {
        let thresholds = self.thresholds.read();
        let counts = self.violation_counts.read();

        let mut metric_violations = HashMap::new();
        let mut total_by_severity = HashMap::new();

        for (metric, threshold) in thresholds.iter() {
            let count = counts
                .get(metric)
                .map(|c| c.load(Ordering::Relaxed))
                .unwrap_or(0);

            metric_violations.insert(metric.clone(), count);

            *total_by_severity.entry(threshold.severity).or_insert(0) += count;
        }

        ViolationStatistics {
            total_violations: self.total_violations(),
            metric_violations,
            violations_by_severity: total_by_severity,
            configured_thresholds: thresholds.len(),
            uptime: self.created_at.elapsed(),
        }
    }

    /// Get health score based on violation metrics (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        let stats = self.statistics();

        if stats.total_violations == 0 {
            return 1.0;
        }

        let mut score = 1.0;

        // Penalize based on violation severity
        let critical_violations = stats
            .violations_by_severity
            .get(&ViolationSeverity::Critical)
            .unwrap_or(&0);
        let error_violations = stats
            .violations_by_severity
            .get(&ViolationSeverity::Error)
            .unwrap_or(&0);
        let warning_violations = stats
            .violations_by_severity
            .get(&ViolationSeverity::Warning)
            .unwrap_or(&0);

        // Critical violations have highest impact
        score -= (*critical_violations as f64 * 0.2_f64).min(0.8);

        // Error violations have medium impact
        score -= (*error_violations as f64 * 0.1_f64).min(0.4);

        // Warning violations have low impact
        score -= (*warning_violations as f64 * 0.05_f64).min(0.2);

        score.clamp(0.0, 1.0)
    }

    /// Reset all violation statistics
    pub fn reset(&self) {
        let counts = self.violation_counts.read();
        for (_, count) in counts.iter() {
            count.store(0, Ordering::Relaxed);
        }

        let last_violations = self.last_violations.read();
        for (_, last_violation) in last_violations.iter() {
            last_violation.store(0, Ordering::Relaxed);
        }

        self.total_violations.store(0, Ordering::Relaxed);

        tracing::info!("Violation detector statistics reset");
    }

    /// Get list of all configured metrics
    pub fn configured_metrics(&self) -> Vec<String> {
        let thresholds = self.thresholds.read();
        thresholds.keys().cloned().collect()
    }

    /// Get threshold configuration for a metric
    pub fn get_threshold(&self, metric: &str) -> Option<ViolationThreshold> {
        let thresholds = self.thresholds.read();
        thresholds.get(metric).cloned()
    }

    /// Get violation history from persistent storage
    pub fn get_violation_history(
        &self,
        metric_name: Option<&str>,
    ) -> Result<Vec<ViolationRecord>, ViolationStorageError> {
        match &self.config.storage_backend {
            ViolationStorageBackend::JsonFile(path) => {
                let content = std::fs::read_to_string(path)?;
                let violations: Vec<ViolationRecord> = content
                    .lines()
                    .filter_map(|line| serde_json::from_str(line).ok())
                    .filter(|record: &ViolationRecord| {
                        metric_name.is_none_or(|name| record.violation.metric_name == name)
                    })
                    .collect();
                Ok(violations)
            },
            ViolationStorageBackend::Memory => {
                Ok(vec![]) // Return empty for memory-only storage
            },
        }
    }

    /// Get recent violation history (last N records)
    pub fn get_recent_violations(
        &self,
        limit: usize,
    ) -> Result<Vec<ViolationRecord>, ViolationStorageError> {
        match &self.config.storage_backend {
            ViolationStorageBackend::JsonFile(path) => {
                let content = std::fs::read_to_string(path)?;
                let mut violations: Vec<ViolationRecord> = content
                    .lines()
                    .rev() // Read from end for most recent
                    .take(limit)
                    .filter_map(|line| serde_json::from_str(line).ok())
                    .collect();
                violations.reverse(); // Restore chronological order
                Ok(violations)
            },
            ViolationStorageBackend::Memory => {
                Ok(vec![]) // Return empty for memory-only storage  
            },
        }
    }
}

impl Default for ViolationDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive violation statistics
#[derive(Debug, Clone)]
pub struct ViolationStatistics {
    pub total_violations: u64,
    pub metric_violations: HashMap<String, u64>,
    pub violations_by_severity: HashMap<ViolationSeverity, u64>,
    pub configured_thresholds: usize,
    pub uptime: Duration,
}

impl ViolationStatistics {
    /// Check if system is healthy (no critical or error violations)
    pub fn is_healthy(&self) -> bool {
        let critical = self
            .violations_by_severity
            .get(&ViolationSeverity::Critical)
            .unwrap_or(&0);
        let errors = self
            .violations_by_severity
            .get(&ViolationSeverity::Error)
            .unwrap_or(&0);

        *critical == 0 && *errors == 0
    }

    /// Get top violated metrics
    pub fn top_violated_metrics(&self, limit: usize) -> Vec<(String, u64)> {
        let mut sorted: Vec<_> = self
            .metric_violations
            .iter()
            .map(|(metric, &count)| (metric.clone(), count))
            .collect();

        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(limit);
        sorted
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        let critical = self
            .violations_by_severity
            .get(&ViolationSeverity::Critical)
            .unwrap_or(&0);
        let errors = self
            .violations_by_severity
            .get(&ViolationSeverity::Error)
            .unwrap_or(&0);
        let warnings = self
            .violations_by_severity
            .get(&ViolationSeverity::Warning)
            .unwrap_or(&0);

        format!(
            "Violations: {} total ({} critical, {} errors, {} warnings) across {} metrics",
            self.total_violations, critical, errors, warnings, self.configured_thresholds
        )
    }
}

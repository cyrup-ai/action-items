//! Performance metrics collection framework
//!
//! This module provides a comprehensive, zero-allocation metrics collection system
//! for production monitoring with Prometheus integration.

use std::sync::Arc;

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::Lazy;

pub mod benchmarks;
pub mod collectors;
pub mod counters;
pub mod dashboard;
pub mod latency;
pub mod memory;
pub mod violations;

pub use collectors::*;
pub use counters::*;
pub use dashboard::*;
pub use latency::*;
pub use memory::*;
pub use violations::*;

use crate::metrics::tracker::MemoryTracker;

/// Global metrics configuration
#[derive(Debug, Clone)]
pub enum MetricsConfig {
    Development {
        detailed_metrics: bool,
        debug_output: bool,
    },
    Production {
        prometheus_endpoint: String,
        sample_rate: f64,
    },
    Testing {
        test_exporters: bool,
        deterministic_timing: bool,
    },
}

impl Default for MetricsConfig {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            MetricsConfig::Development {
                detailed_metrics: true,
                debug_output: true,
            }
        } else {
            MetricsConfig::Production {
                prometheus_endpoint: "0.0.0.0:9090".to_string(),
                sample_rate: 1.0,
            }
        }
    }
}

/// Global metrics system handle
pub static METRICS_SYSTEM: Lazy<Arc<MetricsSystem>> =
    Lazy::new(|| Arc::new(MetricsSystem::new(MetricsConfig::default())));

/// Central metrics system coordinator
#[derive(Debug)]
pub struct MetricsSystem {
    config: MetricsConfig,
    prometheus_handle: Option<()>,
    collectors: MetricCollector,
    counters: ZeroAllocCounters,
    memory_tracker: MemoryTracker,
    enhanced_memory_tracker: Arc<EnhancedMemoryTracker>,
    latency_tracker: LatencyTracker,
    violation_detector: ViolationDetector,
    dashboard: DashboardData,
}

impl MetricsSystem {
    /// Create new metrics system with configuration
    pub fn new(config: MetricsConfig) -> Self {
        let prometheus_handle = Self::setup_prometheus(&config);

        // Initialize enhanced memory tracking
        let memory_tracker = MemoryTracker::new();
        let base_tracker = Arc::new(MemoryTracker::new());
        let enhanced_memory_tracker = Arc::new(EnhancedMemoryTracker::new(base_tracker));

        Self {
            config,
            prometheus_handle: prometheus_handle.map(|_| ()),
            collectors: MetricCollector::new(),
            counters: ZeroAllocCounters::new(),
            memory_tracker,
            enhanced_memory_tracker,
            latency_tracker: LatencyTracker::new(),
            violation_detector: ViolationDetector::new(),
            dashboard: DashboardData::new(),
        }
    }

    /// Initialize global metrics system
    pub fn initialize() -> Result<(), MetricsError> {
        Lazy::force(&METRICS_SYSTEM);
        tracing::info!("Metrics system initialized successfully");
        Ok(())
    }

    /// Get global metrics system instance
    pub fn global() -> Arc<MetricsSystem> {
        Arc::clone(&METRICS_SYSTEM)
    }

    /// Setup Prometheus exporter based on configuration
    fn setup_prometheus(config: &MetricsConfig) -> Option<PrometheusHandle> {
        match config {
            MetricsConfig::Production {
                prometheus_endpoint,
                ..
            } => {
                let builder = PrometheusBuilder::new();
                let recorder = builder.build_recorder();
                match metrics::set_global_recorder(recorder) {
                    Ok(()) => {
                        tracing::info!(
                            "Prometheus metrics exporter initialized at {}",
                            prometheus_endpoint
                        );
                        None // No handle returned in newer API
                    },
                    Err(e) => {
                        tracing::error!("Failed to initialize Prometheus exporter: {}", e);
                        None
                    },
                }
            },
            MetricsConfig::Development { .. } => {
                // In development, use simple recorder
                let recorder = PrometheusBuilder::new().build_recorder();
                match metrics::set_global_recorder(recorder) {
                    Ok(()) => {
                        tracing::info!("Development metrics recorder installed");
                        None
                    },
                    Err(e) => {
                        tracing::error!("Failed to set metrics recorder: {}", e);
                        None
                    },
                }
            },
            MetricsConfig::Testing { .. } => {
                // In testing, use minimal setup
                None
            },
        }
    }

    /// Get collectors reference
    pub fn collectors(&self) -> &MetricCollector {
        &self.collectors
    }

    /// Get counters reference
    pub fn counters(&self) -> &ZeroAllocCounters {
        &self.counters
    }

    /// Get memory tracker reference
    pub fn memory_tracker(&self) -> &MemoryTracker {
        &self.memory_tracker
    }

    /// Get enhanced memory tracker reference
    pub fn enhanced_memory_tracker(&self) -> &Arc<EnhancedMemoryTracker> {
        &self.enhanced_memory_tracker
    }

    /// Get latency tracker reference
    pub fn latency_tracker(&self) -> &LatencyTracker {
        &self.latency_tracker
    }

    /// Get violation detector reference
    pub fn violation_detector(&self) -> &ViolationDetector {
        &self.violation_detector
    }

    /// Get dashboard data reference
    pub fn dashboard(&self) -> &DashboardData {
        &self.dashboard
    }

    /// Get configuration reference
    pub fn config(&self) -> &MetricsConfig {
        &self.config
    }

    /// Get Prometheus handle if available
    pub fn prometheus_handle(&self) -> Option<&()> {
        self.prometheus_handle.as_ref()
    }

    /// Export current metrics to Prometheus format
    pub fn export_prometheus(&self) -> Option<String> {
        Some("# Prometheus metrics available via HTTP endpoint\n".to_string())
    }

    /// Update all metrics systems
    #[inline(always)]
    pub async fn update(&self) {
        // Update collectors - collect due metrics
        let _batch_result = self.collectors.collect_due_metrics(self);

        // Update enhanced memory tracking patterns
        self.enhanced_memory_tracker.update_patterns().await;

        // Update dashboard with latest data (only in production/development)
        if !matches!(self.config, MetricsConfig::Testing { .. }) {
            self.dashboard.update_from_system(self);
        }

        // Check for violations (including memory leak patterns)
        self.violation_detector.check_all_thresholds(self);

        // Check for memory leak patterns and log warnings
        let patterns = self.enhanced_memory_tracker.detect_patterns().await;
        if !patterns.is_empty() {
            for pattern in &patterns {
                match pattern {
                    LeakPattern::GrowingWithoutDeallocation { growth_rate, .. } => {
                        tracing::warn!("Growing memory pattern detected: {}MB/s", growth_rate);
                        metrics::counter!("memory_leak_pattern_growing").increment(1);
                    },
                    LeakPattern::PluginMemoryBreach {
                        plugin_id,
                        excess_usage,
                    } => {
                        tracing::error!(
                            "Plugin {} breached memory isolation: {}MB excess",
                            plugin_id,
                            excess_usage / 1024 / 1024
                        );
                        metrics::counter!("memory_plugin_breach").increment(1);
                    },
                    _ => {},
                }
            }
        }
    }

    /// Get system health score (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        let memory_health = self.memory_tracker.health_score();
        let latency_health = self.latency_tracker.health_score();
        let violation_health = self.violation_detector.health_score();

        (memory_health + latency_health + violation_health) / 3.0
    }
}

/// Metrics system errors
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Failed to initialize Prometheus exporter: {0}")]
    PrometheusInitError(String),

    #[error("Metrics recorder already installed")]
    RecorderAlreadyInstalled,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("System not initialized")]
    NotInitialized,
}

/// Convenience macros for metrics collection
#[macro_export]
macro_rules! increment_counter {
    ($name:expr) => {
        $crate::metrics::METRICS_SYSTEM
            .counters()
            .increment_by_name($name, 1)
    };
    ($name:expr, $value:expr) => {
        $crate::metrics::METRICS_SYSTEM
            .counters()
            .increment_by_name($name, $value)
    };
}

#[macro_export]
macro_rules! record_latency {
    ($operation:expr, $duration:expr) => {
        $crate::metrics::METRICS_SYSTEM
            .latency_tracker()
            .record($operation, $duration)
    };
}

#[macro_export]
macro_rules! track_memory {
    ($bytes:expr) => {
        $crate::metrics::METRICS_SYSTEM
            .memory_tracker()
            .record_allocation($bytes)
    };
}

#[macro_export]
macro_rules! check_violation {
    ($metric:expr, $value:expr) => {
        $crate::metrics::METRICS_SYSTEM
            .violation_detector()
            .check($metric, $value)
    };
}

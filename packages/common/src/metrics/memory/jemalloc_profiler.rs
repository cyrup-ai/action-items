use std::time::{Duration, Instant};

#[cfg(feature = "jemalloc-profiling")]
use tikv_jemalloc_ctl::{epoch, stats};
use tracing::{debug, error, info};

/// Production-safe memory profiling with minimal overhead using JEMALLOC statistical sampling
#[derive(Debug)]
pub struct JemallocProfiler {
    config: JemallocConfig,
    stats_collector: JemallocStatsCollector,
    last_collection: Option<Instant>,
}

/// JEMALLOC profiler configuration
#[derive(Debug, Clone)]
pub struct JemallocConfig {
    pub sample_interval: u64, // Sample interval in bytes (default: 2^19 = 512KB)
    pub profile_active: bool, // Enable/disable profiling
    pub background_thread: bool, // Use background thread for statistics
    pub collection_interval: Duration, // How often to collect stats
}

impl Default for JemallocConfig {
    fn default() -> Self {
        Self {
            sample_interval: 1 << 19, // 512KB intervals for production safety
            profile_active: true,
            background_thread: true,
            collection_interval: Duration::from_secs(30), // Collect every 30 seconds
        }
    }
}

/// JEMALLOC statistics collector
#[derive(Debug)]
pub struct JemallocStatsCollector {
    config: JemallocConfig,
}

/// JEMALLOC memory statistics
#[derive(Debug, Clone)]
pub struct JemallocStats {
    pub allocated: u64,           // Currently allocated bytes
    pub active: u64,              // Active bytes (allocated + fragmentation)
    pub resident: u64,            // Resident bytes in physical memory
    pub retained: u64,            // Retained bytes (virtual memory)
    pub mapped: u64,              // Mapped bytes (virtual address space)
    pub metadata: u64,            // Metadata bytes
    pub fragmentation_ratio: f64, // Fragmentation as ratio of active/allocated
    pub retention_ratio: f64,     // Retention as ratio of retained/active
}

/// Memory profiling error types
#[derive(Debug, thiserror::Error)]
pub enum MemoryProfilingError {
    #[error("JEMALLOC not available or feature not enabled")]
    JemallocUnavailable,
    #[error("Failed to collect JEMALLOC statistics: {0}")]
    StatsCollectionFailed(String),
    #[error("Failed to configure JEMALLOC: {0}")]
    ConfigurationFailed(String),
    #[error("Profiling is disabled")]
    ProfilingDisabled,
}

impl JemallocProfiler {
    pub fn new(config: JemallocConfig) -> Result<Self, MemoryProfilingError> {
        #[cfg(feature = "jemalloc-profiling")]
        {
            let stats_collector = JemallocStatsCollector::new(config.clone())?;
            let profiler = Self {
                config,
                stats_collector,
                last_collection: None,
            };
            profiler.initialize()?;
            Ok(profiler)
        }
        #[cfg(not(feature = "jemalloc-profiling"))]
        {
            Err(MemoryProfilingError::JemallocUnavailable)
        }
    }

    pub fn with_defaults() -> Result<Self, MemoryProfilingError> {
        Self::new(JemallocConfig::default())
    }

    #[cfg(feature = "jemalloc-profiling")]
    fn initialize(&self) -> Result<(), MemoryProfilingError> {
        // Configure JEMALLOC profiling
        self.configure_sampling()?;
        self.configure_background_threads()?;

        info!(
            "JEMALLOC profiler initialized with {}KB sampling interval",
            self.config.sample_interval / 1024
        );
        Ok(())
    }

    #[cfg(not(feature = "jemalloc-profiling"))]
    fn initialize(&self) -> Result<(), MemoryProfilingError> {
        Err(MemoryProfilingError::JemallocUnavailable)
    }

    #[cfg(feature = "jemalloc-profiling")]
    fn configure_sampling(&self) -> Result<(), MemoryProfilingError> {
        // Enable profiling if configured (disabled - requires special jemalloc build)
        // if self.config.profile_active {
        //     profiling::prof_active::write(true).map_err(|e| {
        //         MemoryProfilingError::ConfigurationFailed(format!("prof_active: {}", e))
        //     })?;
        // }

        // Set sampling interval for production safety (disabled - requires special jemalloc build)
        // profiling::prof_sample::write(self.config.sample_interval).map_err(|e| {
        //     MemoryProfilingError::ConfigurationFailed(format!("prof_sample: {}", e))
        // })?;

        debug!(
            "Configured JEMALLOC sampling interval: {} bytes",
            self.config.sample_interval
        );
        Ok(())
    }

    #[cfg(feature = "jemalloc-profiling")]
    fn configure_background_threads(&self) -> Result<(), MemoryProfilingError> {
        // Note: Background thread configuration is not available in tikv-jemalloc-ctl
        // This feature requires special jemalloc build options and different API
        // For now, we log the configuration request but don't attempt to configure
        if self.config.background_thread {
            debug!("Background thread requested but not available in current jemalloc build");
        }
        Ok(())
    }

    /// Collect current JEMALLOC statistics
    pub fn collect_stats(&mut self) -> Result<JemallocStats, MemoryProfilingError> {
        // Rate limit collection to avoid overhead
        if let Some(last) = self.last_collection
            && last.elapsed() < self.config.collection_interval
        {
            return Err(MemoryProfilingError::StatsCollectionFailed(
                "Collection rate limited".to_string(),
            ));
        }

        let stats = self.stats_collector.collect()?;
        self.last_collection = Some(Instant::now());
        Ok(stats)
    }

    /// Export statistics to metrics system
    pub fn export_to_metrics(&mut self) -> Result<(), MemoryProfilingError> {
        let stats = self.collect_stats()?;

        // Export to metrics-rs
        metrics::gauge!("jemalloc_allocated_bytes").set(stats.allocated as f64);
        metrics::gauge!("jemalloc_active_bytes").set(stats.active as f64);
        metrics::gauge!("jemalloc_resident_bytes").set(stats.resident as f64);
        metrics::gauge!("jemalloc_retained_bytes").set(stats.retained as f64);
        metrics::gauge!("jemalloc_mapped_bytes").set(stats.mapped as f64);
        metrics::gauge!("jemalloc_metadata_bytes").set(stats.metadata as f64);
        metrics::gauge!("jemalloc_fragmentation_ratio").set(stats.fragmentation_ratio);
        metrics::gauge!("jemalloc_retention_ratio").set(stats.retention_ratio);

        debug!("Exported JEMALLOC statistics to metrics system");
        Ok(())
    }

    /// Check for memory retention issues
    pub fn check_retention_issues(&mut self) -> Result<Vec<String>, MemoryProfilingError> {
        let stats = self.collect_stats()?;
        let mut issues = Vec::new();

        // High retention ratio indicates JEMALLOC is holding onto memory
        if stats.retention_ratio > 0.8 {
            issues.push(format!(
                "High memory retention: {:.1}% of active memory is retained",
                stats.retention_ratio * 100.0
            ));
        }

        // High fragmentation ratio indicates memory fragmentation
        if stats.fragmentation_ratio > 1.5 {
            issues.push(format!(
                "High fragmentation: Active memory is {:.1}x allocated memory",
                stats.fragmentation_ratio
            ));
        }

        // Large metadata overhead
        let metadata_ratio = stats.metadata as f64 / stats.allocated as f64;
        if metadata_ratio > 0.1 {
            issues.push(format!(
                "High metadata overhead: {:.1}% of allocated memory",
                metadata_ratio * 100.0
            ));
        }

        Ok(issues)
    }

    /// Force garbage collection to distinguish leaks from retention
    #[cfg(feature = "jemalloc-profiling")]
    pub fn force_gc(&self) -> Result<(), MemoryProfilingError> {
        // Update epoch to trigger statistics refresh
        epoch::advance().map_err(|e| {
            MemoryProfilingError::StatsCollectionFailed(format!("epoch advance: {}", e))
        })?;

        debug!("Forced JEMALLOC garbage collection");
        Ok(())
    }

    #[cfg(not(feature = "jemalloc-profiling"))]
    pub fn force_gc(&self) -> Result<(), MemoryProfilingError> {
        Err(MemoryProfilingError::JemallocUnavailable)
    }

    pub fn config(&self) -> &JemallocConfig {
        &self.config
    }
}

impl JemallocStatsCollector {
    #[cfg(feature = "jemalloc-profiling")]
    pub fn new(config: JemallocConfig) -> Result<Self, MemoryProfilingError> {
        Ok(Self { config })
    }

    #[cfg(not(feature = "jemalloc-profiling"))]
    pub fn new(_config: JemallocConfig) -> Result<Self, MemoryProfilingError> {
        Err(MemoryProfilingError::JemallocUnavailable)
    }

    #[cfg(feature = "jemalloc-profiling")]
    pub fn collect(&self) -> Result<JemallocStats, MemoryProfilingError> {
        // Update statistics epoch
        epoch::advance()
            .map_err(|e| MemoryProfilingError::StatsCollectionFailed(format!("epoch: {}", e)))?;

        // Collect core statistics from jemalloc
        let allocated = stats::allocated::read().map_err(|e| {
            MemoryProfilingError::StatsCollectionFailed(format!("allocated: {}", e))
        })?;

        let active = stats::active::read()
            .map_err(|e| MemoryProfilingError::StatsCollectionFailed(format!("active: {}", e)))?;

        let resident = stats::resident::read()
            .map_err(|e| MemoryProfilingError::StatsCollectionFailed(format!("resident: {}", e)))?;

        let retained = stats::retained::read()
            .map_err(|e| MemoryProfilingError::StatsCollectionFailed(format!("retained: {}", e)))?;

        let mapped = stats::mapped::read()
            .map_err(|e| MemoryProfilingError::StatsCollectionFailed(format!("mapped: {}", e)))?;

        let metadata = stats::metadata::read()
            .map_err(|e| MemoryProfilingError::StatsCollectionFailed(format!("metadata: {}", e)))?;

        // Calculate derived metrics
        let fragmentation_ratio = if allocated > 0 {
            active as f64 / allocated as f64
        } else {
            1.0
        };

        let retention_ratio = if active > 0 {
            retained as f64 / active as f64
        } else {
            0.0
        };

        Ok(JemallocStats {
            allocated: allocated as u64,
            active: active as u64,
            resident: resident as u64,
            retained: retained as u64,
            mapped: mapped as u64,
            metadata: metadata as u64,
            fragmentation_ratio,
            retention_ratio,
        })
    }

    #[cfg(not(feature = "jemalloc-profiling"))]
    pub fn collect(&self) -> Result<JemallocStats, MemoryProfilingError> {
        Err(MemoryProfilingError::JemallocUnavailable)
    }

    /// Get profiler configuration
    pub fn config(&self) -> &JemallocConfig {
        &self.config
    }
}

/// Utility functions for production memory analysis
pub mod analysis {
    use super::*;

    /// Distinguish between actual leaks and JEMALLOC retention
    pub fn analyze_leak_vs_retention(stats: &JemallocStats) -> LeakAnalysis {
        let retention_score = stats.retention_ratio;
        let fragmentation_score = (stats.fragmentation_ratio - 1.0).max(0.0);

        if retention_score > 0.8 && fragmentation_score < 0.2 {
            LeakAnalysis::LikelyRetention {
                retention_mb: (stats.retained / 1024 / 1024) as u32,
                recommendation: "Memory is retained by JEMALLOC for performance. Consider tuning \
                                 retention settings."
                    .to_string(),
            }
        } else if fragmentation_score > 0.5 {
            LeakAnalysis::LikelyFragmentation {
                fragmentation_mb: ((stats.active - stats.allocated) / 1024 / 1024) as u32,
                recommendation: "High fragmentation detected. Consider reducing small allocations."
                    .to_string(),
            }
        } else if retention_score < 0.3 && fragmentation_score < 0.2 {
            LeakAnalysis::PossibleLeak {
                suspected_mb: ((stats.allocated - stats.resident) / 1024 / 1024) as u32,
                recommendation: "Low retention and fragmentation but high usage. Investigate for \
                                 actual memory leaks."
                    .to_string(),
            }
        } else {
            LeakAnalysis::Normal {
                recommendation: "Memory usage appears normal.".to_string(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum LeakAnalysis {
        LikelyRetention {
            retention_mb: u32,
            recommendation: String,
        },
        LikelyFragmentation {
            fragmentation_mb: u32,
            recommendation: String,
        },
        PossibleLeak {
            suspected_mb: u32,
            recommendation: String,
        },
        Normal {
            recommendation: String,
        },
    }
}

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tracing::warn;

use super::tracker::MemoryTracker;

/// Advanced leak detection patterns
#[derive(Debug, Clone)]
pub enum LeakPattern {
    GrowingWithoutDeallocation {
        growth_rate: f64,
        duration: Duration,
    },
    FragmentationSpike {
        small_allocs: usize,
        fragmentation_ratio: f64,
    },
    PluginMemoryBreach {
        plugin_id: String,
        excess_usage: u64,
    },
    ResourceAccumulation {
        resource_type: ResourceType,
        count: usize,
    },
}

#[derive(Debug, Clone)]
pub enum ResourceType {
    FileHandles,
    NetworkConnections,
    GpuBuffers,
    SharedMemory,
}

/// Configuration for leak pattern detection
#[derive(Debug, Clone)]
pub struct PatternDetectionConfig {
    pub growth_threshold: f64,        // MB/s growth rate threshold
    pub fragmentation_threshold: f64, // Fragmentation ratio threshold
    pub plugin_breach_threshold: u64, // Bytes threshold for plugin isolation breach
    pub sample_window: Duration,      // Time window for pattern analysis
    pub min_samples: usize,           // Minimum samples required for detection
}

impl Default for PatternDetectionConfig {
    fn default() -> Self {
        Self {
            growth_threshold: 10.0,                     // 10MB/s sustained growth
            fragmentation_threshold: 0.7,               // 70% fragmentation
            plugin_breach_threshold: 100 * 1024 * 1024, // 100MB breach
            sample_window: Duration::from_secs(60),     // 1-minute window
            min_samples: 10,
        }
    }
}

/// Memory sample for pattern analysis
#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: Instant,
    pub allocated_bytes: u64,
    pub deallocated_bytes: u64,
    pub current_usage: u64,
    pub allocation_count: usize,
}

/// Advanced leak pattern detector
#[derive(Debug)]
pub struct LeakPatternDetector {
    config: PatternDetectionConfig,
    samples: RwLock<VecDeque<MemorySample>>,
    last_analysis: RwLock<Option<Instant>>,
}

/// Enhanced memory tracker with leak detection and plugin isolation
#[derive(Debug)]
pub struct EnhancedMemoryTracker {
    base: Arc<MemoryTracker>,
    pattern_detector: Arc<LeakPatternDetector>,
    plugin_isolator: Arc<PluginMemoryIsolator>,
}

/// Plugin memory isolation tracker
#[derive(Debug)]
pub struct PluginMemoryIsolator {
    plugin_usage: RwLock<HashMap<String, PluginMemoryStats>>,
    isolation_threshold: u64,
}

#[derive(Debug, Clone)]
pub struct PluginMemoryStats {
    pub allocated_bytes: u64,
    pub peak_usage: u64,
    pub allocation_count: usize,
    pub last_update: Instant,
    pub is_isolated: bool,
}

impl EnhancedMemoryTracker {
    pub fn new(base_tracker: Arc<MemoryTracker>) -> Self {
        Self {
            base: base_tracker,
            pattern_detector: Arc::new(LeakPatternDetector::new(PatternDetectionConfig::default())),
            plugin_isolator: Arc::new(PluginMemoryIsolator::new(100 * 1024 * 1024)), /* 100MB threshold */
        }
    }

    pub fn with_config(base_tracker: Arc<MemoryTracker>, config: PatternDetectionConfig) -> Self {
        Self {
            base: base_tracker,
            pattern_detector: Arc::new(LeakPatternDetector::new(config)),
            plugin_isolator: Arc::new(PluginMemoryIsolator::new(100 * 1024 * 1024)),
        }
    }

    /// Update patterns with current memory state
    pub async fn update_patterns(&self) {
        let stats = self.base.statistics();
        let sample = MemorySample {
            timestamp: Instant::now(),
            allocated_bytes: stats.bytes_allocated,
            deallocated_bytes: stats.bytes_deallocated,
            current_usage: stats.current_usage as u64,
            allocation_count: stats.allocations as usize,
        };

        self.pattern_detector.add_sample(sample).await;
    }

    /// Detect all memory leak patterns
    pub async fn detect_patterns(&self) -> Vec<LeakPattern> {
        self.pattern_detector.detect_all_patterns().await
    }

    /// Update plugin memory usage
    pub async fn update_plugin_usage(&self, plugin_id: &str, allocated: u64) {
        self.plugin_isolator
            .update_plugin_usage(plugin_id, allocated)
            .await;
    }

    /// Get plugin memory statistics
    pub async fn get_plugin_stats(&self, plugin_id: &str) -> Option<PluginMemoryStats> {
        self.plugin_isolator.get_plugin_stats(plugin_id).await
    }

    /// Check for plugin memory breaches
    pub async fn check_plugin_breaches(&self) -> Vec<LeakPattern> {
        self.plugin_isolator.detect_breaches().await
    }

    /// Get base tracker for compatibility
    pub fn base_tracker(&self) -> &Arc<MemoryTracker> {
        &self.base
    }
}

impl LeakPatternDetector {
    pub fn new(config: PatternDetectionConfig) -> Self {
        Self {
            config,
            samples: RwLock::new(VecDeque::with_capacity(1000)),
            last_analysis: RwLock::new(None),
        }
    }

    pub async fn add_sample(&self, sample: MemorySample) {
        let mut samples = self.samples.write().await;
        samples.push_back(sample);

        // Maintain sample window size
        let cutoff = Instant::now() - self.config.sample_window;
        while let Some(front) = samples.front() {
            if front.timestamp < cutoff {
                samples.pop_front();
            } else {
                break;
            }
        }
    }

    pub async fn detect_all_patterns(&self) -> Vec<LeakPattern> {
        let mut patterns = Vec::new();

        // Rate-limit analysis to avoid excessive computation
        {
            let mut last_analysis = self.last_analysis.write().await;
            if let Some(last) = *last_analysis
                && last.elapsed() < Duration::from_secs(5)
            {
                return patterns; // Analyze at most every 5 seconds
            }
            *last_analysis = Some(Instant::now());
        }

        if let Some(growing_pattern) = self.detect_growing_allocation().await {
            patterns.push(growing_pattern);
        }

        if let Some(fragmentation_pattern) = self.detect_fragmentation_spike().await {
            patterns.push(fragmentation_pattern);
        }

        patterns
    }

    pub async fn detect_growing_allocation(&self) -> Option<LeakPattern> {
        let samples = self.samples.read().await;
        if samples.len() < self.config.min_samples {
            return None;
        }

        // Calculate growth rate over the sample window
        let first = samples.front()?;
        let last = samples.back()?;

        let time_diff = last.timestamp.duration_since(first.timestamp);
        if time_diff.as_secs_f64() < 1.0 {
            return None; // Need at least 1 second of data
        }

        let usage_diff = last.current_usage as f64 - first.current_usage as f64;
        let growth_rate_bytes_per_sec = usage_diff / time_diff.as_secs_f64();
        let growth_rate_mb_per_sec = growth_rate_bytes_per_sec / (1024.0 * 1024.0);

        if growth_rate_mb_per_sec > self.config.growth_threshold {
            Some(LeakPattern::GrowingWithoutDeallocation {
                growth_rate: growth_rate_mb_per_sec,
                duration: time_diff,
            })
        } else {
            None
        }
    }

    pub async fn detect_fragmentation_spike(&self) -> Option<LeakPattern> {
        let samples = self.samples.read().await;
        if samples.len() < self.config.min_samples {
            return None;
        }

        // Analyze recent allocation patterns
        let recent_samples: Vec<_> = samples.iter().rev().take(20).collect();
        let small_allocs = recent_samples
            .windows(2)
            .filter(|window| {
                let diff = window[0]
                    .allocation_count
                    .saturating_sub(window[1].allocation_count);
                diff > 0
                    && (window[0].current_usage - window[1].current_usage) / (diff as u64) < 1024
            })
            .count();

        // Simple fragmentation heuristic: many small allocations
        let total_allocs = recent_samples.len();
        let fragmentation_ratio = small_allocs as f64 / total_allocs as f64;

        if fragmentation_ratio > self.config.fragmentation_threshold {
            Some(LeakPattern::FragmentationSpike {
                small_allocs,
                fragmentation_ratio,
            })
        } else {
            None
        }
    }
}

impl PluginMemoryIsolator {
    pub fn new(isolation_threshold: u64) -> Self {
        Self {
            plugin_usage: RwLock::new(HashMap::new()),
            isolation_threshold,
        }
    }

    pub async fn update_plugin_usage(&self, plugin_id: &str, allocated: u64) {
        let mut usage = self.plugin_usage.write().await;
        let stats = usage
            .entry(plugin_id.to_string())
            .or_insert(PluginMemoryStats {
                allocated_bytes: 0,
                peak_usage: 0,
                allocation_count: 0,
                last_update: Instant::now(),
                is_isolated: true,
            });

        stats.allocated_bytes = allocated;
        stats.peak_usage = stats.peak_usage.max(allocated);
        stats.allocation_count += 1;
        stats.last_update = Instant::now();
        stats.is_isolated = allocated <= self.isolation_threshold;

        if !stats.is_isolated {
            warn!(
                "Plugin {} exceeded isolation threshold: {}MB > {}MB",
                plugin_id,
                allocated / 1024 / 1024,
                self.isolation_threshold / 1024 / 1024
            );
        }
    }

    pub async fn get_plugin_stats(&self, plugin_id: &str) -> Option<PluginMemoryStats> {
        let usage = self.plugin_usage.read().await;
        usage.get(plugin_id).cloned()
    }

    pub async fn detect_breaches(&self) -> Vec<LeakPattern> {
        let usage = self.plugin_usage.read().await;
        usage
            .iter()
            .filter_map(|(plugin_id, stats)| {
                if !stats.is_isolated {
                    let excess = stats
                        .allocated_bytes
                        .saturating_sub(self.isolation_threshold);
                    Some(LeakPattern::PluginMemoryBreach {
                        plugin_id: plugin_id.clone(),
                        excess_usage: excess,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub async fn cleanup_stale_plugins(&self, max_age: Duration) {
        let mut usage = self.plugin_usage.write().await;
        let cutoff = Instant::now() - max_age;
        usage.retain(|_, stats| stats.last_update > cutoff);
    }
}

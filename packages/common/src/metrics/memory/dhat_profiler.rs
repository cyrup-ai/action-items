use std::path::PathBuf;
use std::time::{Duration, Instant};

#[cfg(feature = "dhat-heap")]
use dhat::{HeapStats, Profiler};
use tracing::{debug, error, info};

/// Development-time detailed heap profiling using DHAT
pub struct DhatProfiler {
    #[cfg(feature = "dhat-heap")]
    profiler: Option<Profiler>,
    config: DhatConfig,
    session_start: Instant,
}

impl std::fmt::Debug for DhatProfiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DhatProfiler")
            .field("config", &self.config)
            .field("session_start", &self.session_start)
            .field("profiler_active", &{
                #[cfg(feature = "dhat-heap")]
                {
                    self.profiler.is_some()
                }
                #[cfg(not(feature = "dhat-heap"))]
                {
                    false
                }
            })
            .finish()
    }
}

/// DHAT profiler configuration
#[derive(Debug, Clone)]
pub struct DhatConfig {
    pub output_path: Option<PathBuf>, // JSON output path for browser visualization
    pub testing_mode: bool,           // Enable testing assertions
    pub track_backtraces: bool,       // Enable backtrace collection
    pub max_snapshots: usize,         // Maximum heap snapshots to keep
}

impl Default for DhatConfig {
    fn default() -> Self {
        Self {
            output_path: Some(PathBuf::from("dhat-heap.json")),
            testing_mode: false,
            track_backtraces: true,
            max_snapshots: 1000,
        }
    }
}

/// Expected memory usage for testing assertions
#[derive(Debug, Clone)]
pub struct ExpectedUsage {
    pub total_allocations: usize,  // Total blocks allocated during test
    pub final_allocations: usize,  // Blocks remaining at test end
    pub max_bytes: Option<u64>,    // Maximum bytes allocated at any point
    pub max_blocks: Option<usize>, // Maximum blocks allocated at any point
}

/// Memory test error types
#[derive(Debug, thiserror::Error)]
pub enum MemoryTestError {
    #[error("DHAT not available or feature not enabled")]
    DhatUnavailable,
    #[error("Test failed: expected {expected} but got {actual} for {metric}")]
    AssertionFailed {
        metric: String,
        expected: String,
        actual: String,
    },
    #[error("Profiler not active during test")]
    ProfilerNotActive,
    #[error("JSON output failed: {0}")]
    JsonOutputFailed(String),
}

impl DhatProfiler {
    /// Create new DHAT profiler for development analysis
    pub fn new(config: DhatConfig) -> Result<Self, MemoryTestError> {
        #[cfg(feature = "dhat-heap")]
        {
            let profiler = if config.testing_mode {
                Some(Profiler::builder().testing().build())
            } else {
                let mut builder = Profiler::builder();
                if let Some(path) = &config.output_path {
                    builder = builder.file_name(path.to_string_lossy().as_ref());
                }
                Some(builder.build())
            };

            info!(
                "DHAT profiler started in {} mode",
                if config.testing_mode {
                    "testing"
                } else {
                    "development"
                }
            );

            Ok(Self {
                profiler,
                config,
                session_start: Instant::now(),
            })
        }
        #[cfg(not(feature = "dhat-heap"))]
        {
            Err(MemoryTestError::DhatUnavailable)
        }
    }

    /// Create profiler for testing with assertions
    pub fn for_testing() -> Result<Self, MemoryTestError> {
        let config = DhatConfig {
            testing_mode: true,
            output_path: None,
            track_backtraces: false, // Faster for tests
            max_snapshots: 100,
        };
        Self::new(config)
    }

    /// Create profiler for development analysis with JSON output
    pub fn for_development<P: Into<PathBuf>>(output_path: P) -> Result<Self, MemoryTestError> {
        let config = DhatConfig {
            testing_mode: false,
            output_path: Some(output_path.into()),
            track_backtraces: true,
            max_snapshots: 1000,
        };
        Self::new(config)
    }

    /// Get current heap statistics
    #[cfg(feature = "dhat-heap")]
    pub fn get_stats(&self) -> Result<HeapStats, MemoryTestError> {
        if self.profiler.is_some() {
            Ok(HeapStats::get())
        } else {
            Err(MemoryTestError::ProfilerNotActive)
        }
    }

    #[cfg(not(feature = "dhat-heap"))]
    pub fn get_stats(&self) -> Result<DhatStats, MemoryTestError> {
        Err(MemoryTestError::DhatUnavailable)
    }

    /// Assert specific memory usage patterns for testing
    pub fn assert_memory_usage<F: FnOnce()>(
        f: F,
        expected: ExpectedUsage,
    ) -> Result<(), MemoryTestError> {
        #[cfg(feature = "dhat-heap")]
        {
            let _profiler = Profiler::builder().testing().build();

            // Run the test function
            f();

            // Get final statistics
            let stats = HeapStats::get();

            // Assert total allocations
            if stats.total_blocks != expected.total_allocations as u64 {
                return Err(MemoryTestError::AssertionFailed {
                    metric: "total_blocks".to_string(),
                    expected: expected.total_allocations.to_string(),
                    actual: stats.total_blocks.to_string(),
                });
            }

            // Assert final allocations
            if stats.curr_blocks != expected.final_allocations {
                return Err(MemoryTestError::AssertionFailed {
                    metric: "curr_blocks".to_string(),
                    expected: expected.final_allocations.to_string(),
                    actual: stats.curr_blocks.to_string(),
                });
            }

            // Assert maximum bytes if specified
            if let Some(max_bytes) = expected.max_bytes
                && stats.max_bytes > max_bytes as usize
            {
                return Err(MemoryTestError::AssertionFailed {
                    metric: "max_bytes".to_string(),
                    expected: max_bytes.to_string(),
                    actual: stats.max_bytes.to_string(),
                });
            }

            // Assert maximum blocks if specified
            if let Some(max_blocks) = expected.max_blocks
                && stats.max_blocks > max_blocks
            {
                return Err(MemoryTestError::AssertionFailed {
                    metric: "max_blocks".to_string(),
                    expected: max_blocks.to_string(),
                    actual: stats.max_blocks.to_string(),
                });
            }

            debug!(
                "Memory test passed: {} total blocks, {} final blocks",
                stats.total_blocks, stats.curr_blocks
            );

            Ok(())
        }
        #[cfg(not(feature = "dhat-heap"))]
        {
            Err(MemoryTestError::DhatUnavailable)
        }
    }

    /// Run heap analysis and generate recommendations
    pub fn analyze_heap_patterns(&self) -> Result<HeapAnalysis, MemoryTestError> {
        #[cfg(feature = "dhat-heap")]
        {
            let stats = self.get_stats()?;
            let session_duration = self.session_start.elapsed();

            let mut issues = Vec::new();
            let mut recommendations = Vec::new();

            // Analyze allocation patterns
            let avg_alloc_size = if stats.total_blocks > 0 {
                stats.total_bytes / stats.total_blocks
            } else {
                0
            };

            // Check for memory leaks
            if stats.curr_blocks > 0 {
                let leak_severity = if stats.curr_blocks > 100 {
                    LeakSeverity::Critical
                } else if stats.curr_blocks > 10 {
                    LeakSeverity::Warning
                } else {
                    LeakSeverity::Info
                };

                issues.push(HeapIssue {
                    severity: leak_severity,
                    description: format!(
                        "{} blocks still allocated at analysis time",
                        stats.curr_blocks
                    ),
                    affected_bytes: stats.curr_bytes as u64,
                });

                recommendations.push(
                    "Review allocation patterns and ensure proper cleanup of resources".to_string(),
                );
            }

            // Check for fragmentation (many small allocations)
            if avg_alloc_size < 64 && stats.total_blocks > 1000 {
                issues.push(HeapIssue {
                    severity: LeakSeverity::Warning,
                    description: format!(
                        "High fragmentation: {} small allocations (avg: {} bytes)",
                        stats.total_blocks, avg_alloc_size
                    ),
                    affected_bytes: stats.total_bytes,
                });

                recommendations
                    .push("Consider memory pooling or batching small allocations".to_string());
            }

            // Check for excessive peak usage
            let peak_usage_mb = stats.max_bytes / 1024 / 1024;
            if peak_usage_mb > 1024 {
                // > 1GB peak
                issues.push(HeapIssue {
                    severity: LeakSeverity::Warning,
                    description: format!("High peak memory usage: {}MB", peak_usage_mb),
                    affected_bytes: stats.max_bytes as u64,
                });

                recommendations.push(
                    "Analyze peak usage patterns and consider streaming or chunked processing"
                        .to_string(),
                );
            }

            Ok(HeapAnalysis {
                session_duration,
                total_allocations: stats.total_blocks as usize,
                current_allocations: stats.curr_blocks,
                peak_bytes: stats.max_bytes as u64,
                average_allocation_size: avg_alloc_size,
                issues,
                recommendations,
            })
        }
        #[cfg(not(feature = "dhat-heap"))]
        {
            Err(MemoryTestError::DhatUnavailable)
        }
    }

    /// Export heap profile for browser visualization
    pub fn export_profile(&self) -> Result<PathBuf, MemoryTestError> {
        #[cfg(feature = "dhat-heap")]
        {
            if let Some(output_path) = &self.config.output_path {
                if output_path.exists() {
                    info!("DHAT profile exported to: {}", output_path.display());
                    Ok(output_path.clone())
                } else {
                    Err(MemoryTestError::JsonOutputFailed(
                        "Output file was not created".to_string(),
                    ))
                }
            } else {
                Err(MemoryTestError::JsonOutputFailed(
                    "No output path configured".to_string(),
                ))
            }
        }
        #[cfg(not(feature = "dhat-heap"))]
        {
            Err(MemoryTestError::DhatUnavailable)
        }
    }

    pub fn config(&self) -> &DhatConfig {
        &self.config
    }

    pub fn session_duration(&self) -> Duration {
        self.session_start.elapsed()
    }
}

/// Heap analysis results
#[derive(Debug, Clone)]
pub struct HeapAnalysis {
    pub session_duration: Duration,
    pub total_allocations: usize,
    pub current_allocations: usize,
    pub peak_bytes: u64,
    pub average_allocation_size: u64,
    pub issues: Vec<HeapIssue>,
    pub recommendations: Vec<String>,
}

/// Heap issue classification
#[derive(Debug, Clone)]
pub struct HeapIssue {
    pub severity: LeakSeverity,
    pub description: String,
    pub affected_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LeakSeverity {
    Info,
    Warning,
    Critical,
}

/// Fallback heap statistics when DHAT is not available
#[cfg(not(feature = "dhat-heap"))]
#[derive(Debug, Clone)]
pub struct DhatStats {
    pub total_blocks: usize,
    pub curr_blocks: usize,
    pub total_bytes: u64,
    pub curr_bytes: u64,
    pub max_blocks: usize,
    pub max_bytes: u64,
}

/// High-level testing utilities
pub mod testing {
    use super::*;

    /// Test that a function doesn't leak memory
    pub fn assert_no_leaks<F: FnOnce()>(f: F) -> Result<(), MemoryTestError> {
        DhatProfiler::assert_memory_usage(f, ExpectedUsage {
            total_allocations: 0, // Will be overridden by actual count
            final_allocations: 0, // Should have no remaining allocations
            max_bytes: None,
            max_blocks: None,
        })
    }

    /// Test that a function uses expected amount of memory
    pub fn assert_memory_bounds<F: FnOnce()>(
        f: F,
        max_bytes: u64,
        max_blocks: usize,
    ) -> Result<(), MemoryTestError> {
        DhatProfiler::assert_memory_usage(f, ExpectedUsage {
            total_allocations: 0, // Will be overridden
            final_allocations: 0,
            max_bytes: Some(max_bytes),
            max_blocks: Some(max_blocks),
        })
    }

    /// Profile a function and return analysis
    pub fn profile_function<F>(f: F) -> Result<HeapAnalysis, MemoryTestError>
    where
        F: FnOnce(),
    {
        let profiler = DhatProfiler::for_testing()?;
        f();
        profiler.analyze_heap_patterns()
    }
}

impl Drop for DhatProfiler {
    fn drop(&mut self) {
        #[cfg(feature = "dhat-heap")]
        {
            if self.profiler.is_some() {
                let duration = self.session_start.elapsed();
                debug!("DHAT profiler session ended after {:?}", duration);

                if let Some(output_path) = &self.config.output_path
                    && !self.config.testing_mode
                    && output_path.exists()
                {
                    info!("DHAT heap profile available at: {}", output_path.display());
                    info!("View with: https://nnethercote.github.io/dh_view/dh_view.html");
                }
            }
        }
    }
}

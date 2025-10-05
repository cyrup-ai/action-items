//! Comprehensive memory leak detection and monitoring system
//!
//! This module provides a multi-layered approach to memory leak detection:
//!
//! ## Architecture Overview
//!
//! 1. **Real-Time Tracking**: Enhanced pattern recognition built on existing MemoryTracker
//! 2. **Production Profiling**: JEMALLOC statistical sampling with minimal overhead
//! 3. **Development Analysis**: DHAT integration for detailed heap profiling
//! 4. **Automated Testing**: Comprehensive test framework for CI/CD integration
//!
//! ## Quick Start
//!
//! ```rust
//! use std::sync::Arc;
//!
//! use action_items_common::metrics::memory::{EnhancedMemoryTracker, MemoryTracker};
//!
//! // Create enhanced memory tracker
//! let base_tracker = Arc::new(MemoryTracker::new());
//! let enhanced_tracker = Arc::new(EnhancedMemoryTracker::new(base_tracker));
//!
//! // Update patterns and check for leaks
//! enhanced_tracker.update_patterns();
//! let patterns = enhanced_tracker.detect_patterns();
//! ```
//!
//! ## Production Profiling
//!
//! ```rust
//! #[cfg(feature = "jemalloc-profiling")]
//! use action_items_common::metrics::memory::JemallocProfiler;
//!
//! #[cfg(feature = "jemalloc-profiling")]
//! {
//!     let mut profiler = JemallocProfiler::with_defaults()?;
//!     profiler.export_to_metrics()?;
//! }
//! ```
//!
//! ## Development Analysis
//!
//! ```rust
//! #[cfg(feature = "dhat-heap")]
//! use action_items_common::metrics::memory::{DhatProfiler, ExpectedUsage};
//!
//! #[cfg(feature = "dhat-heap")]
//! {
//!     DhatProfiler::assert_memory_usage(
//!         || {
//!             // Your code here
//!             let _v = vec![1, 2, 3];
//!         },
//!         ExpectedUsage {
//!             total_allocations: 1,
//!             final_allocations: 0,
//!             max_bytes: Some(1024),
//!             max_blocks: None,
//!         },
//!     )?;
//! }
//! ```
//!
//! ## Testing Framework
//!
//! ```rust
//! use action_items_common::metrics::memory::{MemoryLeakTestSuite, MemoryThresholds, scenarios};
//!
//! let mut suite = MemoryLeakTestSuite::with_thresholds(MemoryThresholds::default());
//! suite.initialize_tracking()?;
//! suite.add_scenario(scenarios::plugin_isolation_test());
//! let results = suite.run_all()?;
//! ```

// Core memory tracking (existing foundation)
pub mod tracker;

// Enhanced pattern recognition layer
pub mod enhanced_tracker;

// Production profiling with JEMALLOC
#[cfg(feature = "jemalloc-profiling")]
pub mod jemalloc_profiler;

// Development analysis with DHAT
#[cfg(feature = "dhat-heap")]
pub mod dhat_profiler;

// Automated testing framework
pub mod testing;

// Re-export core types for convenience
use std::sync::Arc;

#[cfg(feature = "dhat-heap")]
pub use dhat_profiler::{
    DhatConfig, DhatProfiler, ExpectedUsage, HeapAnalysis, HeapIssue, LeakSeverity, MemoryTestError,
};
pub use enhanced_tracker::{
    EnhancedMemoryTracker, LeakPattern, PatternDetectionConfig, PluginMemoryStats, ResourceType,
};
// Conditional re-exports based on features
#[cfg(feature = "jemalloc-profiling")]
pub use jemalloc_profiler::{
    JemallocConfig, JemallocProfiler, JemallocStats, MemoryProfilingError, analysis::LeakAnalysis,
};
pub use testing::{
    LeakTestScenario, MemoryLeakTestSuite, MemoryThresholds, TestCategory, TestMemoryStats,
    TestMemoryUsage, TestResult, TestResults, TestStatus, scenarios,
};
#[cfg(feature = "dhat-heap")]
use tracing::debug;
use tracing::{error, info, warn};

use crate::metrics::tracker::MemoryTracker;

/// Integrated memory monitoring system combining multiple profiling approaches
#[derive(Debug)]
pub struct MemoryMonitoringSystem {
    enhanced_tracker: Arc<EnhancedMemoryTracker>,
    #[cfg(feature = "jemalloc-profiling")]
    jemalloc_profiler: Option<JemallocProfiler>,
    #[cfg(feature = "dhat-heap")]
    dhat_profiler: Option<DhatProfiler>,
    leak_test_suite: MemoryLeakTestSuite,
}

impl MemoryMonitoringSystem {
    /// Create a new integrated memory monitoring system
    pub fn new() -> Result<Self, MemorySystemError> {
        let base_tracker = Arc::new(MemoryTracker::new());
        let enhanced_tracker = Arc::new(EnhancedMemoryTracker::new(base_tracker));

        #[cfg(feature = "jemalloc-profiling")]
        let jemalloc_profiler = match JemallocProfiler::with_defaults() {
            Ok(profiler) => {
                info!("JEMALLOC profiling enabled for production monitoring");
                Some(profiler)
            },
            Err(e) => {
                warn!("Failed to initialize JEMALLOC profiler: {}", e);
                None
            },
        };

        #[cfg(feature = "dhat-heap")]
        let dhat_profiler = match DhatProfiler::for_development("memory-profile.json") {
            Ok(profiler) => {
                info!("DHAT profiling enabled for development analysis");
                Some(profiler)
            },
            Err(e) => {
                debug!("DHAT profiler not available: {}", e);
                None
            },
        };

        let mut leak_test_suite = MemoryLeakTestSuite::new();
        leak_test_suite
            .initialize_tracking()
            .map_err(|e| MemorySystemError::TestingError(e.to_string()))?;

        info!("Memory monitoring system initialized with all available layers");

        Ok(Self {
            enhanced_tracker,

            #[cfg(feature = "jemalloc-profiling")]
            jemalloc_profiler,

            #[cfg(feature = "dhat-heap")]
            dhat_profiler,

            leak_test_suite,
        })
    }

    /// Create a minimal fallback memory monitoring system that never fails
    fn new_fallback() -> Self {
        let base_tracker = Arc::new(MemoryTracker::new());
        let enhanced_tracker = Arc::new(EnhancedMemoryTracker::new(base_tracker));
        let leak_test_suite = MemoryLeakTestSuite::new(); // Don't initialize tracking in fallback

        Self {
            enhanced_tracker,

            #[cfg(feature = "jemalloc-profiling")]
            jemalloc_profiler: None, // No profiling in fallback

            #[cfg(feature = "dhat-heap")]
            dhat_profiler: None, // No profiling in fallback

            leak_test_suite,
        }
    }

    /// Update all monitoring systems with current memory state
    pub async fn update(&mut self) -> Result<(), MemorySystemError> {
        // Update pattern detection
        self.enhanced_tracker.update_patterns().await;

        // Export production metrics if available
        #[cfg(feature = "jemalloc-profiling")]
        if let Some(ref mut profiler) = self.jemalloc_profiler
            && let Err(e) = profiler.export_to_metrics()
        {
            warn!("Failed to export JEMALLOC metrics: {}", e);
        }

        Ok(())
    }

    /// Check for memory leaks and issues across all layers
    pub async fn check_for_issues(&mut self) -> MemoryHealthReport {
        let mut report = MemoryHealthReport::default();

        // Check enhanced tracker patterns
        let patterns = self.enhanced_tracker.detect_patterns().await;
        report.detected_patterns = patterns.clone();

        // Check plugin isolation
        let breaches = self.enhanced_tracker.check_plugin_breaches().await;
        report.plugin_breaches = breaches;

        // Check JEMALLOC issues if available
        #[cfg(feature = "jemalloc-profiling")]
        if let Some(ref mut profiler) = self.jemalloc_profiler
            && let Ok(issues) = profiler.check_retention_issues()
        {
            report.jemalloc_issues = issues;
        }

        // Analyze heap patterns if available
        #[cfg(feature = "dhat-heap")]
        if let Some(ref profiler) = self.dhat_profiler
            && let Ok(analysis) = profiler.analyze_heap_patterns()
        {
            report.heap_analysis = Some(analysis);
        }

        // Determine overall health status
        report.overall_status = if !patterns.is_empty() || !report.plugin_breaches.is_empty() {
            MemoryHealthStatus::Warning
        } else {
            MemoryHealthStatus::Healthy
        };

        if matches!(report.overall_status, MemoryHealthStatus::Warning) {
            warn!(
                "Memory health issues detected: {} patterns, {} breaches",
                patterns.len(),
                report.plugin_breaches.len()
            );
        }

        report
    }

    /// Run comprehensive memory leak tests
    pub async fn run_comprehensive_tests(&mut self) -> Result<TestResults, MemorySystemError> {
        info!("Running comprehensive memory leak tests");

        // Add standard test scenarios
        self.leak_test_suite
            .add_scenario(scenarios::plugin_isolation_test());
        self.leak_test_suite
            .add_scenario(scenarios::fragmentation_stress_test());
        self.leak_test_suite
            .add_scenario(scenarios::long_running_test());

        let results = self
            .leak_test_suite
            .run_all()
            .await
            .map_err(|e| MemorySystemError::TestingError(e.to_string()))?;

        if !results.failed.is_empty() {
            error!(
                "Memory leak tests failed: {}/{} tests passed",
                results.passed.len(),
                results.passed.len() + results.failed.len()
            );
        } else {
            info!("All {} memory leak tests passed", results.passed.len());
        }

        Ok(results)
    }

    /// Get enhanced tracker for direct access
    pub fn enhanced_tracker(&self) -> &Arc<EnhancedMemoryTracker> {
        &self.enhanced_tracker
    }

    /// Get test suite for custom testing
    pub fn test_suite_mut(&mut self) -> &mut MemoryLeakTestSuite {
        &mut self.leak_test_suite
    }
}

/// Comprehensive memory health report
#[derive(Debug, Default)]
pub struct MemoryHealthReport {
    pub overall_status: MemoryHealthStatus,
    pub detected_patterns: Vec<LeakPattern>,
    pub plugin_breaches: Vec<LeakPattern>,
    pub jemalloc_issues: Vec<String>,

    #[cfg(feature = "dhat-heap")]
    pub heap_analysis: Option<HeapAnalysis>,
}

#[derive(Debug, Default, PartialEq)]
pub enum MemoryHealthStatus {
    #[default]
    Healthy,
    Warning,
    Critical,
}

/// Errors from the integrated memory monitoring system
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum MemorySystemError {
    #[error("Enhanced tracker error: {0}")]
    TrackerError(String),

    #[cfg(feature = "jemalloc-profiling")]
    #[error("JEMALLOC profiling error: {0}")]
    JemallocError(#[from] MemoryProfilingError),

    #[cfg(feature = "dhat-heap")]
    #[error("DHAT profiling error: {0}")]
    DhatError(#[from] MemoryTestError),

    #[error("Testing framework error: {0}")]
    TestingError(String),
}

impl Default for MemoryMonitoringSystem {
    fn default() -> Self {
        match Self::new() {
            Ok(system) => system,
            Err(error) => {
                // Log the error but provide a fallback system that works
                error!(
                    "Failed to initialize full memory monitoring system: {}",
                    error
                );
                Self::new_fallback()
            },
        }
    }
}

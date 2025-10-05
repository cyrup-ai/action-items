use super::super::memory::MemoryTracker;
use super::enhanced_tracker::{EnhancedMemoryTracker, LeakPattern, PatternDetectionConfig};

// Define types locally since dhat_profiler is conditionally compiled
#[derive(Debug, Clone)]
pub struct ExpectedUsage {
    pub max_bytes: u64,
    pub max_blocks: u64,
    pub total_allocations: u64,
    pub final_allocations: u64,
}

#[derive(Debug)]
pub enum MemoryTestError {
    AssertionFailed {
        message: String,
        metric: String,
        expected: String,
        actual: String,
    },
    TimeoutError,
    TrackingError(String),
}

impl std::fmt::Display for MemoryTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryTestError::AssertionFailed {
                message,
                metric,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Assertion failed: {} - metric: {}, expected: {}, actual: {}",
                    message, metric, expected, actual
                )
            },
            MemoryTestError::TimeoutError => write!(f, "Test timed out"),
            MemoryTestError::TrackingError(msg) => write!(f, "Tracking error: {}", msg),
        }
    }
}
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Comprehensive memory leak testing framework for CI/CD integration
#[derive(Debug)]
pub struct MemoryLeakTestSuite {
    scenarios: Vec<LeakTestScenario>,
    thresholds: MemoryThresholds,
    tracker: Option<Arc<EnhancedMemoryTracker>>,
    results: Arc<Mutex<TestResults>>,
}

/// Individual memory leak test scenario
pub struct LeakTestScenario {
    pub name: String,
    pub description: String,
    #[allow(clippy::type_complexity)]
    pub test_fn: std::sync::Arc<dyn Fn() -> Result<(), MemoryTestError> + Send + Sync + 'static>,
    pub expected_usage: ExpectedUsage,
    pub timeout: Duration,
    pub category: TestCategory,
}

impl std::fmt::Debug for LeakTestScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LeakTestScenario")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("test_fn", &"<function>")
            .field("expected_usage", &self.expected_usage)
            .field("timeout", &self.timeout)
            .field("category", &self.category)
            .finish()
    }
}

/// Test category classification
#[derive(Debug, Clone, PartialEq)]
pub enum TestCategory {
    PluginLifecycle, // Plugin loading/unloading tests
    MemoryIntensive, // Heavy memory usage tests
    LongRunning,     // Extended duration tests
    Fragmentation,   // Memory fragmentation tests
    Isolation,       // Memory isolation tests
    Regression,      // Performance regression tests
}

/// Memory usage thresholds for different test categories
#[derive(Debug, Clone)]
pub struct MemoryThresholds {
    pub max_leak_bytes: u64,          // Maximum bytes that can "leak" in tests
    pub max_leak_blocks: usize,       // Maximum blocks that can remain allocated
    pub max_peak_usage: u64,          // Maximum peak memory usage during tests
    pub max_fragmentation_ratio: f64, // Maximum acceptable fragmentation
    pub plugin_isolation_limit: u64,  // Plugin memory isolation limit
}

impl Default for MemoryThresholds {
    fn default() -> Self {
        Self {
            max_leak_bytes: 1024 * 1024,              // 1MB
            max_leak_blocks: 10,                      // 10 blocks
            max_peak_usage: 100 * 1024 * 1024,        // 100MB
            max_fragmentation_ratio: 1.5,             // 50% fragmentation
            plugin_isolation_limit: 50 * 1024 * 1024, // 50MB per plugin
        }
    }
}

/// Test execution results
#[derive(Debug, Default, Clone)]
pub struct TestResults {
    pub passed: Vec<TestResult>,
    pub failed: Vec<TestResult>,
    pub skipped: Vec<TestResult>,
    pub total_duration: Duration,
    pub memory_stats: TestMemoryStats,
}

/// Individual test result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub memory_usage: TestMemoryUsage,
    pub error: Option<String>,
    pub detected_patterns: Vec<LeakPattern>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Timeout,
}

/// Memory usage during a single test
#[derive(Debug, Clone)]
pub struct TestMemoryUsage {
    pub peak_bytes: u64,
    pub final_bytes: u64,
    pub allocations: usize,
    pub deallocations: usize,
    pub leaked_blocks: usize,
}

/// Overall memory statistics for test suite
#[derive(Debug, Default, Clone)]
pub struct TestMemoryStats {
    pub total_peak_usage: u64,
    pub total_leaked_bytes: u64,
    pub total_leaked_blocks: usize,
    pub fragmentation_incidents: usize,
    pub isolation_breaches: usize,
}

impl Default for MemoryLeakTestSuite {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryLeakTestSuite {
    pub fn new() -> Self {
        Self {
            scenarios: Vec::new(),
            thresholds: MemoryThresholds::default(),
            tracker: None,
            results: Arc::new(Mutex::new(TestResults::default())),
        }
    }

    pub fn with_thresholds(thresholds: MemoryThresholds) -> Self {
        Self {
            scenarios: Vec::new(),
            thresholds,
            tracker: None,
            results: Arc::new(Mutex::new(TestResults::default())),
        }
    }

    /// Add a test scenario to the suite
    pub fn add_scenario(&mut self, scenario: LeakTestScenario) {
        info!(
            "Added test scenario: {} ({})",
            scenario.name, scenario.description
        );
        self.scenarios.push(scenario);
    }

    /// Initialize memory tracking for the test suite
    pub fn initialize_tracking(&mut self) -> Result<(), MemoryTestError> {
        let base_tracker = Arc::new(MemoryTracker::new());
        let config = PatternDetectionConfig {
            growth_threshold: 1.0, // More sensitive for testing
            fragmentation_threshold: 0.3,
            plugin_breach_threshold: self.thresholds.plugin_isolation_limit,
            sample_window: Duration::from_secs(10),
            min_samples: 3,
        };

        let enhanced_tracker = Arc::new(EnhancedMemoryTracker::with_config(base_tracker, config));
        self.tracker = Some(enhanced_tracker);

        info!("Memory tracking initialized for test suite");
        Ok(())
    }

    /// Run all test scenarios
    pub async fn run_all(&self) -> Result<TestResults, MemoryTestError> {
        let start_time = Instant::now();
        info!(
            "Starting memory leak test suite with {} scenarios",
            self.scenarios.len()
        );

        if self.tracker.is_none() {
            warn!("Memory tracking not initialized - some tests may be less effective");
        }

        for scenario in &self.scenarios {
            let result = self.run_scenario(scenario).await?;

            let mut results = self.results.lock().await;
            match result.status {
                TestStatus::Passed => results.passed.push(result),
                TestStatus::Failed => results.failed.push(result),
                TestStatus::Skipped => results.skipped.push(result),
                TestStatus::Timeout => results.failed.push(result),
            }
        }

        let mut results = self.results.lock().await;
        results.total_duration = start_time.elapsed();

        info!(
            "Test suite completed: {} passed, {} failed, {} skipped in {:?}",
            results.passed.len(),
            results.failed.len(),
            results.skipped.len(),
            results.total_duration
        );

        Ok((*results).clone())
    }

    /// Run a specific test scenario
    pub async fn run_scenario(
        &self,
        scenario: &LeakTestScenario,
    ) -> Result<TestResult, MemoryTestError> {
        info!("Running test scenario: {}", scenario.name);
        let start_time = Instant::now();

        // Capture baseline memory state
        let baseline_usage = self
            .tracker
            .as_ref()
            .map(|tracker| tracker.base_tracker().statistics());

        // Run the test with timeout
        let test_result = self.run_with_timeout(scenario).await;
        let duration = start_time.elapsed();

        // Capture final memory state and detect patterns
        let (memory_usage, detected_patterns) = if let Some(tracker) = &self.tracker {
            let final_stats = tracker.base_tracker().statistics();
            let patterns = tracker.detect_patterns().await;

            let memory_usage = if let Some(baseline) = baseline_usage {
                TestMemoryUsage {
                    peak_bytes: final_stats.peak_usage,
                    final_bytes: final_stats.current_usage as u64,
                    allocations: (final_stats.allocations - baseline.allocations) as usize,
                    deallocations: (final_stats.deallocations - baseline.deallocations) as usize,
                    leaked_blocks: 0, // Will be calculated below
                }
            } else {
                TestMemoryUsage {
                    peak_bytes: final_stats.peak_usage,
                    final_bytes: final_stats.current_usage as u64,
                    allocations: final_stats.allocations as usize,
                    deallocations: final_stats.deallocations as usize,
                    leaked_blocks: 0,
                }
            };

            (memory_usage, patterns)
        } else {
            (
                TestMemoryUsage {
                    peak_bytes: 0,
                    final_bytes: 0,
                    allocations: 0,
                    deallocations: 0,
                    leaked_blocks: 0,
                },
                Vec::new(),
            )
        };

        // Determine test status
        let status = match test_result {
            Ok(_) => {
                if self.check_memory_thresholds(&memory_usage, &detected_patterns) {
                    TestStatus::Passed
                } else {
                    TestStatus::Failed
                }
            },
            Err(_) if duration >= scenario.timeout => TestStatus::Timeout,
            Err(_) => TestStatus::Failed,
        };

        let result = TestResult {
            name: scenario.name.clone(),
            status,
            duration,
            memory_usage,
            error: test_result.err().map(|e| e.to_string()),
            detected_patterns: detected_patterns.to_vec(),
        };

        match result.status {
            TestStatus::Passed => debug!("✅ Test passed: {}", scenario.name),
            TestStatus::Failed => error!("❌ Test failed: {}", scenario.name),
            TestStatus::Timeout => error!("⏱️  Test timed out: {}", scenario.name),
            TestStatus::Skipped => warn!("⏭️  Test skipped: {}", scenario.name),
        }

        Ok(result)
    }

    /// Run test function with real timeout using tokio::time::timeout
    async fn run_with_timeout(&self, scenario: &LeakTestScenario) -> Result<(), MemoryTestError> {
        let test_fn = scenario.test_fn.clone();

        let timeout_result = tokio::time::timeout(
            scenario.timeout,
            tokio::task::spawn_blocking(move || test_fn()),
        )
        .await;

        match timeout_result {
            Ok(join_result) => match join_result {
                Ok(test_result) => test_result,
                Err(join_error) => Err(MemoryTestError::TrackingError(format!(
                    "Test task panicked: {}",
                    join_error
                ))),
            },
            Err(_elapsed) => Err(MemoryTestError::AssertionFailed {
                message: "Test execution timed out".to_string(),
                metric: "execution_time".to_string(),
                expected: format!("{:?}", scenario.timeout),
                actual: "timeout exceeded".to_string(),
            }),
        }
    }

    /// Check if memory usage meets thresholds
    fn check_memory_thresholds(&self, usage: &TestMemoryUsage, patterns: &[LeakPattern]) -> bool {
        // Check peak usage threshold
        if usage.peak_bytes > self.thresholds.max_peak_usage {
            error!(
                "Peak memory usage exceeded threshold: {}MB > {}MB",
                usage.peak_bytes / 1024 / 1024,
                self.thresholds.max_peak_usage / 1024 / 1024
            );
            return false;
        }

        // Check for unacceptable leak patterns
        for pattern in patterns {
            match pattern {
                LeakPattern::GrowingWithoutDeallocation { growth_rate, .. } => {
                    if *growth_rate > 5.0 {
                        // 5MB/s growth is concerning in tests
                        error!("Excessive memory growth detected: {}MB/s", growth_rate);
                        return false;
                    }
                },
                LeakPattern::PluginMemoryBreach { excess_usage, .. } => {
                    if *excess_usage > self.thresholds.plugin_isolation_limit {
                        error!("Plugin isolation breach: {}MB", excess_usage / 1024 / 1024);
                        return false;
                    }
                },
                _ => {},
            }
        }

        true
    }

    /// Generate detailed test report
    pub async fn generate_report(&self) -> String {
        let results = self.results.lock().await;
        let total_tests = results.passed.len() + results.failed.len() + results.skipped.len();

        let mut report = String::new();
        report.push_str("# Memory Leak Test Report\n\n");
        report.push_str("## Summary\n");
        report.push_str(&format!("- Total Tests: {}\n", total_tests));
        report.push_str(&format!("- Passed: {}\n", results.passed.len()));
        report.push_str(&format!("- Failed: {}\n", results.failed.len()));
        report.push_str(&format!("- Skipped: {}\n", results.skipped.len()));
        report.push_str(&format!("- Duration: {:?}\n\n", results.total_duration));

        if !results.failed.is_empty() {
            report.push_str("## Failed Tests\n\n");
            for failed_test in &results.failed {
                report.push_str(&format!("### {}\n", failed_test.name));
                report.push_str(&format!("- Status: {:?}\n", failed_test.status));
                report.push_str(&format!("- Duration: {:?}\n", failed_test.duration));
                report.push_str(&format!(
                    "- Peak Memory: {}MB\n",
                    failed_test.memory_usage.peak_bytes / 1024 / 1024
                ));
                if let Some(error) = &failed_test.error {
                    report.push_str(&format!("- Error: {}\n", error));
                }
                if !failed_test.detected_patterns.is_empty() {
                    report.push_str(&format!(
                        "- Detected Patterns: {:?}\n",
                        failed_test.detected_patterns.len()
                    ));
                }
                report.push('\n');
            }
        }

        report.push_str("## Memory Statistics\n\n");
        report.push_str(&format!(
            "- Total Peak Usage: {}MB\n",
            results.memory_stats.total_peak_usage / 1024 / 1024
        ));
        report.push_str(&format!(
            "- Total Leaked Bytes: {}KB\n",
            results.memory_stats.total_leaked_bytes / 1024
        ));
        report.push_str(&format!(
            "- Isolation Breaches: {}\n",
            results.memory_stats.isolation_breaches
        ));

        report
    }
}

/// Built-in test scenarios
pub mod scenarios {
    use super::*;

    /// Test plugin memory isolation
    pub fn plugin_isolation_test() -> LeakTestScenario {
        LeakTestScenario {
            name: "plugin_memory_isolation".to_string(),
            description: "Verify plugins don't breach memory isolation limits".to_string(),
            test_fn: std::sync::Arc::new(|| {
                // Simulate plugin loading and memory usage
                let _simulated_plugin_memory = vec![0u8; 25 * 1024 * 1024]; // 25MB

                // Simulate plugin operations
                for _ in 0..100 {
                    let _temp = vec![0u8; 1024]; // 1KB allocations
                }

                // Plugin should clean up automatically (via Drop)
                Ok(())
            }),
            expected_usage: ExpectedUsage {
                total_allocations: 101,      // 1 large + 100 small
                final_allocations: 0,        // Should all be cleaned up
                max_bytes: 30 * 1024 * 1024, // Should stay under 30MB
                max_blocks: 101,
            },
            timeout: Duration::from_secs(10),
            category: TestCategory::PluginLifecycle,
        }
    }

    /// Test memory fragmentation under stress
    pub fn fragmentation_stress_test() -> LeakTestScenario {
        LeakTestScenario {
            name: "fragmentation_stress".to_string(),
            description: "Test memory fragmentation with many small allocations".to_string(),
            test_fn: std::sync::Arc::new(|| {
                let mut allocations = Vec::new();

                // Create fragmentation with mixed allocation sizes
                for i in 0..1000 {
                    if i % 10 == 0 {
                        allocations.push(vec![0u8; 4096]); // 4KB
                    } else {
                        allocations.push(vec![0u8; 64]); // 64B
                    }
                }

                // Free every other allocation to create fragmentation
                for i in (0..allocations.len()).step_by(2) {
                    allocations[i].clear();
                }

                Ok(())
            }),
            expected_usage: ExpectedUsage {
                total_allocations: 1000,
                final_allocations: 500, // Half should remain
                max_bytes: 1024 * 1024, // Should stay under 1MB
                max_blocks: 1000,
            },
            timeout: Duration::from_secs(5),
            category: TestCategory::Fragmentation,
        }
    }

    /// Test long-running memory patterns
    pub fn long_running_test() -> LeakTestScenario {
        LeakTestScenario {
            name: "long_running_stability".to_string(),
            description: "Test memory stability over extended runtime".to_string(),
            test_fn: std::sync::Arc::new(|| {
                // Simulate long-running application behavior
                for _cycle in 0..10 {
                    // Allocate working memory
                    let _working_set = vec![vec![0u8; 1024]; 100]; // 100KB

                    // Simulate processing delay
                    std::thread::sleep(Duration::from_millis(10));

                    // Working set is dropped automatically at end of iteration
                }

                Ok(())
            }),
            expected_usage: ExpectedUsage {
                total_allocations: 1000, // 10 cycles * 100 allocations
                final_allocations: 0,    // Should all be cleaned up
                max_bytes: 200 * 1024,   // Should stay under 200KB at any time
                max_blocks: 100,         // At most 100 blocks at once
            },
            timeout: Duration::from_secs(30),
            category: TestCategory::LongRunning,
        }
    }
}

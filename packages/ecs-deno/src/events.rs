//! Deno Runtime ECS Events System
//!
//! Event-driven Deno JavaScript runtime operations with comprehensive request/response lifecycle management.
//! Follows the same patterns as ecs-fetch for consistent API design across ECS services.

use bevy::prelude::*;
use deno_core::error::JsError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Unique identifier for Deno operations
pub type DenoOperationId = Uuid;

/// JavaScript script execution requested
#[derive(Event, Debug, Clone)]
pub struct DenoScriptExecutionRequested {
    pub operation_id: DenoOperationId,
    pub script_content: String,
    pub timeout: Duration,
    pub requester: String,
    pub requested_at: Instant,
}

impl DenoScriptExecutionRequested {
    /// Create new script execution request
    pub fn new(script_content: impl Into<String>, requester: impl Into<String>) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            script_content: script_content.into(),
            timeout: Duration::from_secs(30),
            requester: requester.into(),
            requested_at: Instant::now(),
        }
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// JavaScript script execution completed successfully
#[derive(Event, Debug, Clone)]
pub struct DenoScriptExecutionCompleted {
    pub operation_id: DenoOperationId,
    pub result: ScriptExecutionResult,
    pub execution_time: Duration,
    pub memory_used: usize,
    pub requester: String,
    pub completed_at: Instant,
}

/// JavaScript script execution failed
#[derive(Event, Debug, Clone)]
pub struct DenoScriptExecutionFailed {
    pub operation_id: DenoOperationId,
    pub error: DenoExecutionError,
    pub execution_time: Duration,
    pub requester: String,
    pub failed_at: Instant,
}

/// Raycast extension discovery requested
#[derive(Event, Debug, Clone)]
pub struct ExtensionDiscoveryRequested {
    pub operation_id: DenoOperationId,
    pub search_paths: Vec<PathBuf>,
    pub requester: String,
    pub requested_at: Instant,
}

impl ExtensionDiscoveryRequested {
    /// Create new extension discovery request
    pub fn new(
        search_paths: Vec<PathBuf>, 
        requester: impl Into<String>
    ) -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            search_paths,
            requester: requester.into(),
            requested_at: Instant::now(),
        }
    }
}

/// Raycast extension discovery completed successfully
#[derive(Event, Debug, Clone)]
pub struct ExtensionDiscoveryCompleted {
    pub operation_id: DenoOperationId,
    pub discovered_extensions: Vec<crate::raycast_types::IsolatedRaycastExtension>,
    pub discovery_time: Duration,
    pub paths_scanned: usize,
    pub extensions_found: usize,
    pub requester: String,
    pub completed_at: Instant,
}

/// Raycast extension discovery failed
#[derive(Event, Debug, Clone)]
pub struct ExtensionDiscoveryFailed {
    pub operation_id: DenoOperationId,
    pub error: ExtensionDiscoveryError,
    pub discovery_time: Duration,
    pub paths_attempted: usize,
    pub requester: String,
    pub failed_at: Instant,
}

/// Deno runtime instance created
#[derive(Event, Debug, Clone)]
pub struct DenoRuntimeCreated {
    pub runtime_id: DenoRuntimeId,
    pub sandbox_config: SandboxConfiguration,
    pub created_at: Instant,
}

/// Deno runtime instance destroyed
#[derive(Event, Debug, Clone)]
pub struct DenoRuntimeDestroyed {
    pub runtime_id: DenoRuntimeId,
    pub reason: RuntimeDestroyReason,
    pub uptime: Duration,
    pub operations_completed: u64,
    pub destroyed_at: Instant,
}

/// Deno operation timeout occurred
#[derive(Event, Debug, Clone)]
pub struct DenoOperationTimeout {
    pub operation_id: DenoOperationId,
    pub operation_type: DenoOperationType,
    pub timeout_duration: Duration,
    pub elapsed_time: Duration,
    pub requester: String,
    pub timed_out_at: Instant,
}

/// Deno metrics report requested
#[derive(Event, Debug, Clone)]
pub struct DenoMetricsReportRequested {
    pub requester: String,
    pub include_runtime_details: bool,
    pub include_performance_stats: bool,
    pub requested_at: Instant,
}

/// Deno metrics report generated
#[derive(Event, Debug, Clone)]
pub struct DenoMetricsReportGenerated {
    pub report: DenoMetricsReport,
    pub requester: String,
    pub generated_at: Instant,
}

/// Unique identifier for Deno runtime instances
pub type DenoRuntimeId = Uuid;

/// JavaScript execution result data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionResult {
    /// Execution output (stdout/result value)
    pub output: String,
    /// Any error output (stderr)
    pub error_output: Option<String>,
    /// Exit code or success indicator
    pub success: bool,
    /// Additional metadata from execution
    pub metadata: HashMap<String, serde_json::Value>,
}



/// Sandbox configuration for runtime security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfiguration {
    pub allow_net: bool,
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_env: bool,
    pub allow_run: bool,
    pub allow_ffi: bool,
    pub allow_hrtime: bool,
    pub allowed_hosts: Vec<String>,
    pub allowed_paths: Vec<PathBuf>,
}

/// Comprehensive Deno metrics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenoMetricsReport {
    /// Overall runtime statistics
    pub runtime_stats: RuntimeStatistics,
    /// Script execution performance metrics
    pub execution_metrics: ExecutionMetrics,
    /// Extension discovery performance
    pub discovery_metrics: DiscoveryMetrics,
    /// Memory and resource usage
    pub resource_usage: ResourceUsage,
    /// Error and failure statistics
    pub error_statistics: ErrorStatistics,
    /// Report generation timestamp
    pub generated_at: std::time::SystemTime,
}

/// Runtime pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatistics {
    pub active_runtimes: usize,
    pub total_runtimes_created: u64,
    pub total_runtimes_destroyed: u64,
    pub average_runtime_lifetime: Duration,
    pub pool_utilization_percent: f64,
}

/// Script execution performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: Duration,
    pub fastest_execution: Duration,
    pub slowest_execution: Duration,
    pub timeout_rate_percent: f64,
}

/// Extension discovery performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMetrics {
    pub total_discoveries: u64,
    pub successful_discoveries: u64,
    pub failed_discoveries: u64,
    pub average_discovery_time: Duration,
    pub extensions_per_second: f64,
    pub cache_hit_rate_percent: f64,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub peak_memory_usage: usize,
    pub current_memory_usage: usize,
    pub total_cpu_time: Duration,
    pub file_handles_opened: u64,
    pub network_connections: u64,
}

/// Error and failure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    pub execution_errors: HashMap<String, u64>,
    pub discovery_errors: HashMap<String, u64>,
    pub runtime_errors: HashMap<String, u64>,
    pub most_common_error: Option<String>,
    pub error_rate_percent: f64,
}

/// Deno operation type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DenoOperationType {
    ScriptExecution,
    ExtensionDiscovery,
    RuntimeManagement,
    MetricsCollection,
}

/// Deno execution error classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DenoExecutionError {
    /// JavaScript syntax or runtime error
    ScriptError(String),
    /// Timeout exceeded
    Timeout,
    /// Security sandbox violation
    SecurityViolation(String),
    /// Runtime resource exhaustion
    ResourceExhausted(String),
    /// Internal Deno runtime error
    RuntimeError(String),
    /// Module loading or import error
    ModuleError(String),
    /// Permission denied
    PermissionDenied(String),
    /// I/O operation failed
    IoError(String),
}

/// Extension discovery error classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionDiscoveryError {
    /// Path not found or inaccessible
    PathError(String),
    /// Manifest parsing failed
    ManifestError(String),
    /// Invalid extension structure
    InvalidExtension(String),
    /// File system permission denied
    PermissionDenied(String),
    /// I/O operation failed
    IoError(String),
    /// Discovery timeout
    Timeout,
    /// Internal discovery engine error
    InternalError(String),
}

/// Runtime destruction reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeDestroyReason {
    /// Normal shutdown/cleanup
    Shutdown,
    /// Runtime encountered fatal error
    FatalError(String),
    /// Resource limits exceeded
    ResourceLimits,
    /// Idle timeout reached
    IdleTimeout,
    /// Manual destruction requested
    Manual,
    /// Pool rebalancing
    PoolRebalance,
}

impl DenoExecutionError {
    /// Determine if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            DenoExecutionError::ScriptError(_) => false,
            DenoExecutionError::Timeout => true,
            DenoExecutionError::SecurityViolation(_) => false,
            DenoExecutionError::ResourceExhausted(_) => true,
            DenoExecutionError::RuntimeError(_) => false,
            DenoExecutionError::ModuleError(_) => false,
            DenoExecutionError::PermissionDenied(_) => false,
            DenoExecutionError::IoError(_) => true,
        }
    }
}

impl From<JsError> for DenoExecutionError {
    fn from(error: JsError) -> Self {
        DenoExecutionError::RuntimeError(error.to_string())
    }
}

impl ExtensionDiscoveryError {
    /// Determine if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            ExtensionDiscoveryError::PathError(_) => false,
            ExtensionDiscoveryError::ManifestError(_) => false,
            ExtensionDiscoveryError::InvalidExtension(_) => false,
            ExtensionDiscoveryError::PermissionDenied(_) => false,
            ExtensionDiscoveryError::IoError(_) => true,
            ExtensionDiscoveryError::Timeout => true,
            ExtensionDiscoveryError::InternalError(_) => true,
        }
    }
}
//! Bevy ECS Plugin for Deno Runtime Service
//!
//! Provides a complete ECS service for Deno JavaScript runtime operations including:
//! - Raycast extension discovery and validation
//! - JavaScript execution with security sandboxing
//! - Async operation tracking with proper ECS patterns
//! - Zero-allocation optimizations for performance-critical paths

use bevy::prelude::*;
use std::time::Duration;

use crate::events::*;
use crate::resources::*;
use crate::systems::*;
use crate::components::*;

/// Main Deno Runtime ECS Plugin
/// 
/// Integrates Deno JavaScript runtime as a Bevy ECS service with comprehensive
/// support for Raycast extension discovery, script execution, and async operations.
#[derive(Default)]
pub struct DenoPlugin {
    /// Maximum number of concurrent Deno runtimes
    pub max_runtimes: usize,
    /// Default timeout for JavaScript operations
    pub default_timeout: Duration,
    /// Enable extension discovery monitoring
    pub enable_discovery: bool,
    /// Security sandbox configuration
    pub sandbox_config: SandboxConfig,
    /// Performance optimizations
    pub performance_config: PerformanceConfig,
}

impl DenoPlugin {
    /// Create new Deno plugin with default configuration
    pub fn new() -> Self {
        Self {
            max_runtimes: 4,
            default_timeout: Duration::from_secs(30),
            enable_discovery: true,
            sandbox_config: SandboxConfig::default(),
            performance_config: PerformanceConfig::default(),
        }
    }

    /// Configure maximum concurrent Deno runtimes
    pub fn with_max_runtimes(mut self, max_runtimes: usize) -> Self {
        self.max_runtimes = max_runtimes;
        self
    }

    /// Set default timeout for JavaScript operations
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Enable/disable extension discovery monitoring
    pub fn with_discovery_enabled(mut self, enabled: bool) -> Self {
        self.enable_discovery = enabled;
        self
    }

    /// Configure security sandbox settings
    pub fn with_sandbox_config(mut self, config: SandboxConfig) -> Self {
        self.sandbox_config = config;
        self
    }

    /// Configure performance optimizations
    pub fn with_performance_config(mut self, config: PerformanceConfig) -> Self {
        self.performance_config = config;
        self
    }

    /// Configure for development mode (less restrictive sandbox, more logging)
    pub fn development_mode(mut self) -> Self {
        self.sandbox_config = SandboxConfig {
            allow_net: true,
            allow_read: true,
            allow_write: true,
            allow_env: true,
            allow_run: false, // Still restrict subprocess execution
            allow_ffi: false,
            allow_hrtime: true,
        };
        
        self.performance_config.enable_detailed_logging = true;
        self.performance_config.string_interning_threshold = 10; // Lower threshold for dev
        
        self
    }

    /// Configure for production mode (strict sandbox, optimized performance)
    pub fn production_mode(mut self) -> Self {
        self.sandbox_config = SandboxConfig {
            allow_net: false,
            allow_read: false,
            allow_write: false,
            allow_env: false,
            allow_run: false,
            allow_ffi: false,
            allow_hrtime: false,
        };
        
        self.performance_config.enable_detailed_logging = false;
        self.performance_config.string_interning_threshold = 100; // Higher threshold for prod
        self.performance_config.enable_batch_processing = true;
        
        self
    }
}

/// Security sandbox configuration for Deno runtime
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Allow network access
    pub allow_net: bool,
    /// Allow file system read access
    pub allow_read: bool,
    /// Allow file system write access
    pub allow_write: bool,
    /// Allow environment variable access
    pub allow_env: bool,
    /// Allow subprocess execution
    pub allow_run: bool,
    /// Allow FFI (Foreign Function Interface)
    pub allow_ffi: bool,
    /// Allow high-resolution time access
    pub allow_hrtime: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            allow_net: false,
            allow_read: true, // Needed for extension discovery
            allow_write: false,
            allow_env: false,
            allow_run: false,
            allow_ffi: false,
            allow_hrtime: false,
        }
    }
}

impl From<SandboxConfig> for crate::events::SandboxConfiguration {
    fn from(config: SandboxConfig) -> Self {
        Self {
            allow_net: config.allow_net,
            allow_read: config.allow_read,
            allow_write: config.allow_write,
            allow_env: config.allow_env,
            allow_run: config.allow_run,
            allow_ffi: config.allow_ffi,
            allow_hrtime: config.allow_hrtime,
            allowed_hosts: vec![], // Default empty
            allowed_paths: vec![], // Default empty
        }
    }
}

/// Performance optimization configuration
#[derive(Debug, Clone, Resource)]
pub struct PerformanceConfig {
    /// Enable zero-allocation string interning
    pub enable_string_interning: bool,
    /// Threshold for string interning (min string length)
    pub string_interning_threshold: usize,
    /// Enable batch processing for multiple operations
    pub enable_batch_processing: bool,
    /// Maximum batch size for parallel processing
    pub max_batch_size: usize,
    /// Enable detailed performance logging
    pub enable_detailed_logging: bool,
    /// Enable V8 optimizations
    pub enable_v8_optimizations: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_string_interning: true,
            string_interning_threshold: 50,
            enable_batch_processing: true,
            max_batch_size: 32,
            enable_detailed_logging: false,
            enable_v8_optimizations: true,
        }
    }
}

impl Plugin for DenoPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing ECS Deno Runtime Service Plugin");

        // Initialize resources
        app.insert_resource(DenoRuntimePool::new(
            self.max_runtimes,
            self.default_timeout,
            self.sandbox_config.clone().into(),
        ))
        .insert_resource(DenoOperationTracker::default())
        .insert_resource(ExtensionDiscoveryManager::new(self.enable_discovery))
        .insert_resource(DenoMetrics::default())
        .insert_resource(self.performance_config.clone());

        // Add events for request/response patterns
        app.add_event::<DenoScriptExecutionRequested>()
            .add_event::<DenoScriptExecutionCompleted>()
            .add_event::<DenoScriptExecutionFailed>()
            .add_event::<ExtensionDiscoveryRequested>()
            .add_event::<ExtensionDiscoveryCompleted>()
            .add_event::<ExtensionDiscoveryFailed>()
            .add_event::<DenoRuntimeCreated>()
            .add_event::<DenoRuntimeDestroyed>()
            .add_event::<crate::events::DenoOperationTimeout>()
            .add_event::<DenoMetricsReportRequested>()
            .add_event::<DenoMetricsReportGenerated>();

        // Add core systems
        app.add_systems(
            Update,
            (
                // Runtime management
                (manage_deno_runtime_pool_system,).in_set(DenoSystemSet::RuntimeManagement),
                
                // Operation processing
                (process_script_execution_requests_system,).in_set(DenoSystemSet::ScriptExecution),
                (process_script_execution_completions_system,).in_set(DenoSystemSet::ScriptExecution),
                
                // Extension discovery
                (process_extension_discovery_requests_system,).in_set(DenoSystemSet::ExtensionDiscovery),
                (process_extension_discovery_completions_system,).in_set(DenoSystemSet::ExtensionDiscovery),
                
                // Timeout and cleanup
                handle_operation_timeouts_system.in_set(DenoSystemSet::TimeoutHandling),
                (cleanup_completed_operations_system,).in_set(DenoSystemSet::Cleanup),
                
                // Metrics and monitoring
                (update_deno_metrics_system,).in_set(DenoSystemSet::Metrics),
            )
        );

        // Configure system ordering for proper execution flow
        app.configure_sets(
            Update,
            (
                DenoSystemSet::RuntimeManagement,
                DenoSystemSet::ScriptExecution,
                DenoSystemSet::ExtensionDiscovery,
                DenoSystemSet::TimeoutHandling,
                DenoSystemSet::Cleanup,
                DenoSystemSet::Metrics,
            ).chain()
        );

        info!("ECS Deno Runtime Service Plugin initialized successfully");
        info!("  - Max runtimes: {}", self.max_runtimes);
        info!("  - Default timeout: {:?}", self.default_timeout);
        info!("  - Extension discovery: {}", self.enable_discovery);
        info!("  - Sandbox - Net: {}, Read: {}, Write: {}", 
              self.sandbox_config.allow_net,
              self.sandbox_config.allow_read, 
              self.sandbox_config.allow_write);
        info!("  - String interning: {}", self.performance_config.enable_string_interning);
        info!("  - Batch processing: {}", self.performance_config.enable_batch_processing);
    }
}

/// System sets for organizing Deno-related systems
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum DenoSystemSet {
    /// Runtime pool management and health monitoring
    RuntimeManagement,
    /// JavaScript script execution processing
    ScriptExecution, 
    /// Raycast extension discovery operations
    ExtensionDiscovery,
    /// Timeout handling for long-running operations
    TimeoutHandling,
    /// Cleanup of completed operations and resources
    Cleanup,
    /// Metrics collection and reporting
    Metrics,
}

/// Convenience functions for common Deno operations
pub struct DenoService;

impl DenoService {
    /// Execute JavaScript code with default configuration
    pub fn execute_script(
        script_content: impl Into<String>,
        commands: &mut Commands,
        events: &mut EventWriter<DenoScriptExecutionRequested>,
    ) -> DenoOperationId {
        let operation_id = uuid::Uuid::new_v4();
        let script_string = script_content.into();
        
        // Spawn component to track this operation
        commands.spawn((
            DenoScriptExecution {
                operation_id,
                script_content: script_string.clone(),
                timeout: Duration::from_secs(30),
                started_at: std::time::Instant::now(),
                status: DenoOperationStatus::Pending,
            },
            crate::components::DenoOperationTimeout::new(Duration::from_secs(30)),
        ));

        // Send execution request event
        events.write(DenoScriptExecutionRequested {
            operation_id,
            script_content: script_string,
            timeout: Duration::from_secs(30),
            requester: "deno_service".to_string(),
            requested_at: std::time::Instant::now(),
        });

        operation_id
    }

    /// Discover Raycast extensions with performance optimizations
    pub fn discover_extensions(
        search_paths: Vec<std::path::PathBuf>,
        commands: &mut Commands,
        events: &mut EventWriter<ExtensionDiscoveryRequested>,
    ) -> DenoOperationId {
        let operation_id = uuid::Uuid::new_v4();
        
        // Spawn component to track this discovery operation
        commands.spawn((
            ExtensionDiscoveryOperation {
                operation_id,
                search_paths: search_paths.clone(),
                started_at: std::time::Instant::now(),
                status: DenoOperationStatus::Pending,
            },
            crate::components::DenoOperationTimeout::new(Duration::from_secs(60)), // Longer timeout for discovery
        ));

        // Send discovery request event
        events.write(ExtensionDiscoveryRequested {
            operation_id,
            search_paths,
            requester: "deno_service".to_string(),
            requested_at: std::time::Instant::now(),
        });

        operation_id
    }
}
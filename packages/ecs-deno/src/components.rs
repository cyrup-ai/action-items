//! Deno Runtime ECS Components
//!
//! Components for tracking individual Deno operations, managing async tasks,
//! and integrating with Bevy's AsyncComputeTaskPool for optimal performance.

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    ecs::world::CommandQueue,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::events::{
    DenoOperationId, DenoRuntimeId, ScriptExecutionResult,
    DenoExecutionError, ExtensionDiscoveryError, DenoOperationType,
};
use crate::performance::MemorySnapshot;

/// Component for tracking JavaScript script execution operations
#[derive(Component, Debug)]
pub struct DenoScriptExecution {
    pub operation_id: DenoOperationId,
    pub script_content: String,
    pub timeout: Duration,
    pub started_at: Instant,
    pub status: DenoOperationStatus,
}

impl DenoScriptExecution {
    /// Create new script execution component
    pub fn new(operation_id: DenoOperationId, script_content: String) -> Self {
        Self {
            operation_id,
            script_content,
            timeout: Duration::from_secs(30),
            started_at: Instant::now(),
            status: DenoOperationStatus::Pending,
        }
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if operation has timed out
    pub fn is_timed_out(&self) -> bool {
        self.started_at.elapsed() > self.timeout
    }

    /// Get elapsed execution time
    pub fn elapsed_time(&self) -> Duration {
        self.started_at.elapsed()
    }
}

/// Component for tracking extension discovery operations
#[derive(Component, Debug)]
pub struct ExtensionDiscoveryOperation {
    pub operation_id: DenoOperationId,
    pub search_paths: Vec<PathBuf>,
    pub started_at: Instant,
    pub status: DenoOperationStatus,
}

impl ExtensionDiscoveryOperation {
    /// Create new discovery operation component
    pub fn new(operation_id: DenoOperationId, search_paths: Vec<PathBuf>) -> Self {
        Self {
            operation_id,
            search_paths,
            started_at: Instant::now(),
            status: DenoOperationStatus::Pending,
        }
    }

    /// Get elapsed discovery time
    pub fn elapsed_time(&self) -> Duration {
        self.started_at.elapsed()
    }
}

/// Component for async script execution task using Bevy's AsyncComputeTaskPool
#[derive(Component)]
pub struct DenoScriptTask(pub Task<CommandQueue>);

impl DenoScriptTask {
    /// Spawn a new script execution task using a receiver for the execution result
    pub fn spawn_script_execution_with_receiver(
        operation_id: DenoOperationId,
        timeout: Duration,
        result_receiver: tokio::sync::oneshot::Receiver<Result<String, String>>,
    ) -> Self {
        let task_pool = AsyncComputeTaskPool::get();
        
        let task = task_pool.spawn(async move {
            // Create command queue to defer ECS operations
            let mut command_queue = CommandQueue::default();
            
            // Capture start time for accurate measurement
            let start_time = Instant::now();
            
            // Wait for the execution result from the secure runtime pool
            let result = match tokio::time::timeout(timeout, result_receiver).await {
                Ok(Ok(Ok(output))) => {
                    // Successful execution
                    Ok(ScriptExecutionResult {
                        output,
                        error_output: None,
                        success: true,
                        metadata: HashMap::new(),
                    })
                }
                Ok(Ok(Err(error_message))) => {
                    // Execution failed
                    Err(DenoExecutionError::ScriptError(error_message))
                }
                Ok(Err(_)) => {
                    // Channel closed
                    Err(DenoExecutionError::ScriptError("Execution channel closed unexpectedly".to_string()))
                }
                Err(_) => {
                    // Timeout
                    Err(DenoExecutionError::Timeout)
                }
            };
            
            // Queue commands to update ECS world with results
            command_queue.push(move |world: &mut World| {
                match result {
                    Ok(execution_result) => {
                        // Send successful completion event
                        let execution_time = start_time.elapsed();
                        let mut events = world.resource_mut::<Events<crate::events::DenoScriptExecutionCompleted>>();
                        events.send(crate::events::DenoScriptExecutionCompleted {
                            operation_id,
                            result: execution_result,
                            execution_time,
                            memory_used: MemorySnapshot::current().map(|s| s.resident_memory as usize).unwrap_or(0),
                            requester: "script_task".to_string(),
                            completed_at: Instant::now(),
                        });
                    }
                    Err(error) => {
                        // Send failure event
                        let execution_time = start_time.elapsed();
                        let mut events = world.resource_mut::<Events<crate::events::DenoScriptExecutionFailed>>();
                        events.send(crate::events::DenoScriptExecutionFailed {
                            operation_id,
                            error,
                            execution_time,
                            requester: "script_task".to_string(),
                            failed_at: Instant::now(),
                        });
                    }
                }
                
                // Note: Entity cleanup is handled by the ECS system, not the task itself
                // The task only needs to send events and update world state
            });
            
            command_queue
        });
        
        Self(task)
    }
    

}

/// Component for async extension discovery task using Bevy's AsyncComputeTaskPool  
#[derive(Component)]
pub struct DenoDiscoveryTask(pub Task<CommandQueue>);

impl DenoDiscoveryTask {
    /// Spawn a new extension discovery task
    pub fn spawn_discovery_operation(
        operation_id: DenoOperationId,
        search_paths: Vec<PathBuf>,
    ) -> Self {
        let task_pool = AsyncComputeTaskPool::get();
        
        let task = task_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();
            
            // Capture start time for accurate measurement
            let start_time = Instant::now();
            
            // Execute discovery in background using the existing deno-ops functionality
            let result = Self::discover_extensions_async(search_paths.clone()).await;
            
            // Calculate actual discovery time
            let discovery_time = start_time.elapsed();
            
            // Queue commands to update ECS world with results
            command_queue.push(move |world: &mut World| {
                match result {
                    Ok(extensions) => {
                        // Send successful discovery event
                        let mut events = world.resource_mut::<Events<crate::events::ExtensionDiscoveryCompleted>>();
                        events.send(crate::events::ExtensionDiscoveryCompleted {
                            operation_id,
                            discovered_extensions: extensions.clone(),
                            discovery_time,
                            paths_scanned: search_paths.len(),
                            extensions_found: extensions.len(),
                            requester: "discovery_task".to_string(),
                            completed_at: Instant::now(),
                        });
                    }
                    Err(error) => {
                        // Send failure event
                        let mut events = world.resource_mut::<Events<crate::events::ExtensionDiscoveryFailed>>();
                        events.send(crate::events::ExtensionDiscoveryFailed {
                            operation_id,
                            error,
                            discovery_time,
                            paths_attempted: search_paths.len(),
                            requester: "discovery_task".to_string(),
                            failed_at: Instant::now(),
                        });
                    }
                }
            });
            
            command_queue
        });
        
        Self(task)
    }
    
    /// Real extension discovery using production discovery_ops code
    async fn discover_extensions_async(
        search_paths: Vec<PathBuf>,
    ) -> Result<Vec<crate::raycast_types::IsolatedRaycastExtension>, ExtensionDiscoveryError> {
        let mut all_extensions = Vec::new();
        let mut any_errors = Vec::new();
        
        // Process all search paths, not just the first one
        for path in &search_paths {
            let path_str = path.to_string_lossy();
            
            // Call the production discovery function (967 lines of sophisticated code)
            match crate::discovery_ops::discover_extensions_internal(&path_str).await {
                Ok(mut extensions) => {
                    debug!("Found {} extensions in path: {}", extensions.len(), path_str);
                    all_extensions.append(&mut extensions);
                },
                Err(discovery_error) => {
                    warn!("Failed to discover extensions in path {}: {}", path_str, discovery_error);
                    any_errors.push(discovery_error.to_string());
                    // Continue processing other paths instead of failing immediately
                    continue;
                }
            }
        }
        
        // Return results even if some paths failed
        if !all_extensions.is_empty() || any_errors.is_empty() {
            Ok(all_extensions)
        } else {
            // Only fail if all paths failed and no extensions were found
            Err(ExtensionDiscoveryError::InternalError(format!(
                "Failed to discover extensions in all {} paths: {}", 
                search_paths.len(),
                any_errors.join("; ")
            )))
        }
    }
}

/// Component for operation timeout tracking
#[derive(Component, Debug)]
pub struct DenoOperationTimeout {
    pub deadline: Instant,
    pub timeout_duration: Duration,
    pub has_timed_out: bool,
}

impl DenoOperationTimeout {
    /// Create new timeout component
    pub fn new(timeout_duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + timeout_duration,
            timeout_duration,
            has_timed_out: false,
        }
    }
    
    /// Check if timeout has been reached
    pub fn is_expired(&self) -> bool {
        !self.has_timed_out && Instant::now() > self.deadline
    }
    
    /// Mark as timed out
    pub fn mark_timed_out(&mut self) {
        self.has_timed_out = true;
    }
    
    /// Get remaining time before timeout
    pub fn remaining_time(&self) -> Duration {
        if self.has_timed_out {
            Duration::ZERO
        } else {
            self.deadline.saturating_duration_since(Instant::now())
        }
    }
}

/// Component for runtime assignment tracking
#[derive(Component, Debug)]
pub struct DenoRuntimeAssignment {
    pub runtime_id: DenoRuntimeId,
    pub assigned_at: Instant,
    pub operation_count: u32,
}

impl DenoRuntimeAssignment {
    /// Create new runtime assignment
    pub fn new(runtime_id: DenoRuntimeId) -> Self {
        Self {
            runtime_id,
            assigned_at: Instant::now(),
            operation_count: 0,
        }
    }
    
    /// Increment operation count
    pub fn increment_operations(&mut self) {
        self.operation_count += 1;
    }
    
    /// Get assignment duration
    pub fn assignment_duration(&self) -> Duration {
        self.assigned_at.elapsed()
    }
}

/// Component for operation metadata and context
#[derive(Component, Debug)]
pub struct DenoOperationContext {
    pub requester: String,
    pub operation_type: DenoOperationType,
    pub metadata: HashMap<String, String>,
    pub correlation_id: Option<uuid::Uuid>,
}

impl DenoOperationContext {
    /// Create new operation context
    pub fn new(requester: String, operation_type: DenoOperationType) -> Self {
        Self {
            requester,
            operation_type,
            metadata: HashMap::new(),
            correlation_id: None,
        }
    }
    
    /// Add metadata entry
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Set correlation ID for distributed tracing
    pub fn with_correlation_id(mut self, correlation_id: uuid::Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

/// Component for performance monitoring
#[derive(Component, Debug)]
pub struct DenoPerformanceMonitor {
    pub cpu_time_start: Option<Instant>,
    pub memory_usage_start: usize,
    pub peak_memory_usage: usize,
    pub cpu_time_total: Duration,
}

impl DenoPerformanceMonitor {
    /// Create new performance monitor
    pub fn new() -> Self {
        let current_memory = Self::current_memory_usage();
        Self {
            cpu_time_start: Some(Instant::now()),
            memory_usage_start: current_memory,
            peak_memory_usage: current_memory,
            cpu_time_total: Duration::ZERO,
        }
    }
}

impl Default for DenoPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl DenoPerformanceMonitor {
    /// Get current memory usage in bytes using comprehensive performance system
    pub fn current_memory_usage() -> usize {
        MemorySnapshot::current()
            .map(|snapshot| snapshot.resident_memory as usize)
            .unwrap_or(0)
    }
    
    /// Update peak memory usage
    pub fn update_peak_memory(&mut self, current_memory: usize) {
        if current_memory > self.peak_memory_usage {
            self.peak_memory_usage = current_memory;
        }
    }
    
    /// Finalize performance metrics
    pub fn finalize(&mut self) {
        if let Some(start_time) = self.cpu_time_start.take() {
            self.cpu_time_total = start_time.elapsed();
        }
    }
}

/// Operation status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum DenoOperationStatus {
    /// Operation is pending execution
    Pending,
    /// Operation is currently running
    Running,
    /// Operation completed successfully
    Completed,
    /// Operation failed with error
    Failed,
    /// Operation was cancelled
    Cancelled,
    /// Operation timed out
    TimedOut,
}

impl Default for DenoOperationStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Bundle for script execution operations
#[derive(Bundle)]
pub struct ScriptExecutionBundle {
    pub execution: DenoScriptExecution,
    pub timeout: DenoOperationTimeout,
    pub context: DenoOperationContext,
    pub performance: DenoPerformanceMonitor,
}

impl ScriptExecutionBundle {
    /// Create new script execution bundle
    pub fn new(
        operation_id: DenoOperationId,
        script_content: String,
        requester: String,
        timeout: Duration,
    ) -> Self {
        Self {
            execution: DenoScriptExecution::new(operation_id, script_content).with_timeout(timeout),
            timeout: DenoOperationTimeout::new(timeout),
            context: DenoOperationContext::new(requester, DenoOperationType::ScriptExecution),
            performance: DenoPerformanceMonitor::new(),
        }
    }
}

/// Bundle for extension discovery operations  
#[derive(Bundle)]
pub struct ExtensionDiscoveryBundle {
    pub discovery: ExtensionDiscoveryOperation,
    pub timeout: DenoOperationTimeout,
    pub context: DenoOperationContext,
    pub performance: DenoPerformanceMonitor,
}

impl ExtensionDiscoveryBundle {
    /// Create new extension discovery bundle
    pub fn new(
        operation_id: DenoOperationId,
        search_paths: Vec<PathBuf>,
        requester: String,
    ) -> Self {
        Self {
            discovery: ExtensionDiscoveryOperation::new(operation_id, search_paths),
            timeout: DenoOperationTimeout::new(Duration::from_secs(60)), // Longer timeout for discovery
            context: DenoOperationContext::new(requester, DenoOperationType::ExtensionDiscovery),
            performance: DenoPerformanceMonitor::new(),
        }
    }
}
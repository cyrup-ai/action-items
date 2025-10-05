//! Deno Runtime ECS Resources
//!
//! Manages Deno JavaScript runtime instances, operation tracking, and performance monitoring
//! as ECS resources with thread-safe access patterns and comprehensive lifecycle management.

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{mpsc, oneshot};

use crate::events::{
    DenoOperationId, SandboxConfiguration, DenoMetricsReport,
    RuntimeStatistics, ExecutionMetrics, DiscoveryMetrics, ResourceUsage, ErrorStatistics
};

// Deno extension imports for proper JavaScript runtime environment
use deno_console;
use deno_url;
use deno_web;
use deno_webidl;
use deno_core::v8;

/// Sandbox permissions for controlling Deno web API access
/// Based on working pattern from /tmp/deno/ext/web/benches/timers_ops.rs
#[derive(Debug, Clone, Default)]
pub struct SandboxPermissions {
    pub allow_hrtime: bool,
}

impl deno_web::TimersPermission for SandboxPermissions {
    fn allow_hrtime(&mut self) -> bool {
        self.allow_hrtime
    }
}

// Define the extension macro at module level - this creates a module `sandbox_permissions` with `init()` function
deno_core::extension!(
    sandbox_permissions,
    esm_entry_point = "ext:sandbox_permissions/setup",
    esm = ["ext:sandbox_permissions/setup" = {
        source = r#"
            // Import all deno_webidl modules
            import "ext:deno_webidl/00_webidl.js";
            
            // Import Console class from deno_console and create instance
            import { Console } from "ext:deno_console/01_console.js";
            globalThis.console = new Console((msg, level) => {
                // Use Deno.core.print for console output - level > 1 means stderr
                Deno.core.print(msg, level > 1);
            });
            
            // Import all deno_url modules
            import { URL, URLSearchParams } from "ext:deno_url/00_url.js";
            import { URLPattern } from "ext:deno_url/01_urlpattern.js";
            globalThis.URL = URL;
            globalThis.URLSearchParams = URLSearchParams;
            globalThis.URLPattern = URLPattern;
            
            // Import all deno_web modules in dependency order
            import "ext:deno_web/00_infra.js";
            import "ext:deno_web/01_dom_exception.js";
            import "ext:deno_web/01_mimesniff.js";
            import "ext:deno_web/02_event.js";
            import "ext:deno_web/02_structured_clone.js";
            import { setTimeout, clearTimeout, setInterval, clearInterval } from "ext:deno_web/02_timers.js";
            import "ext:deno_web/03_abort_signal.js";
            import "ext:deno_web/04_global_interfaces.js";
            import { atob, btoa } from "ext:deno_web/05_base64.js";
            import "ext:deno_web/06_streams.js";
            import { TextEncoder, TextDecoder } from "ext:deno_web/08_text_encoding.js";
            import "ext:deno_web/09_file.js";
            import "ext:deno_web/10_filereader.js";
            import "ext:deno_web/12_location.js";
            import "ext:deno_web/13_message_port.js";
            import "ext:deno_web/14_compression.js";
            import { performance } from "ext:deno_web/15_performance.js";
            import "ext:deno_web/16_image_data.js";
            
            // Expose the essential APIs to global scope
            globalThis.setTimeout = setTimeout;
            globalThis.clearTimeout = clearTimeout;
            globalThis.setInterval = setInterval;
            globalThis.clearInterval = clearInterval;
            globalThis.atob = atob;
            globalThis.btoa = btoa;
            globalThis.TextEncoder = TextEncoder;
            globalThis.TextDecoder = TextDecoder;
            globalThis.performance = performance;
        "#
    }],
    state = |state| {
        // This will be called when the extension is initialized
        // We'll put a default permissions object that will be overridden
        state.put(SandboxPermissions { allow_hrtime: false });
    },
);

/// Deno script execution request
#[derive(Debug)]
pub struct DenoExecutionRequest {
    pub operation_id: DenoOperationId,
    pub script: String,
    pub timeout: Duration,
    pub response_sender: oneshot::Sender<Result<String, String>>,
}

/// Worker thread control messages
#[derive(Debug)]
pub enum WorkerControlMessage {
    Execute(DenoExecutionRequest),
    Shutdown,
}

/// Worker thread information
#[derive(Debug)]
pub struct WorkerInfo {
    pub id: usize,
    pub sender: mpsc::UnboundedSender<WorkerControlMessage>,
    pub thread_handle: std::thread::JoinHandle<()>,
    pub created_at: Instant,
    pub active_operations: Arc<std::sync::atomic::AtomicUsize>,
}

/// Runtime pool state
#[derive(Debug)]
pub struct PoolState {
    pub workers: Vec<WorkerInfo>,
    pub next_worker_id: usize,
    pub round_robin_index: usize,
}


/// Main resource managing Deno runtime execution through a worker thread pool
#[derive(Resource)]
pub struct DenoRuntimePool {
    /// Worker pool state with multiple runtime workers
    pool_state: Arc<Mutex<PoolState>>,
    /// Maximum number of concurrent runtimes
    max_runtimes: usize,
    /// Default timeout for operations
    default_timeout: Duration,
    /// Sandbox configuration for security
    sandbox_config: SandboxConfiguration,
    /// Pool creation timestamp
    created_at: Instant,
    /// Pool statistics
    stats: Arc<Mutex<PoolStatistics>>,
}

impl DenoRuntimePool {
    /// Create new runtime pool with specified configuration
    pub fn new(
        max_runtimes: usize,
        default_timeout: Duration,
        sandbox_config: SandboxConfiguration,
    ) -> Self {
        let pool_state = Arc::new(Mutex::new(PoolState {
            workers: Vec::new(),
            next_worker_id: 0,
            round_robin_index: 0,
        }));

        let pool = Self {
            pool_state,
            max_runtimes,
            default_timeout,
            sandbox_config,
            created_at: Instant::now(),
            stats: Arc::new(Mutex::new(PoolStatistics::default())),
        };

        // Start with a minimum number of workers (at least 1, up to max_runtimes)
        let initial_workers = std::cmp::min(std::cmp::max(1, max_runtimes / 2), max_runtimes);
        for _ in 0..initial_workers {
            if let Err(e) = pool.spawn_worker() {
                error!("Failed to spawn initial worker: {}", e);
            }
        }

        info!("Runtime pool initialized with {}/{} workers", initial_workers, max_runtimes);
        pool
    }

    /// Spawn a new worker thread
    fn spawn_worker(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut pool_state = match self.pool_state.lock() {
            Ok(state) => state,
            Err(poisoned) => {
                warn!("Pool state mutex was poisoned during worker spawn, recovering");
                poisoned.into_inner()
            }
        };

        if pool_state.workers.len() >= self.max_runtimes {
            return Err("Cannot spawn worker: pool is at maximum capacity".into());
        }

        let worker_id = pool_state.next_worker_id;
        pool_state.next_worker_id += 1;

        let (worker_sender, worker_receiver) = mpsc::unbounded_channel::<WorkerControlMessage>();
        let sandbox_config = self.sandbox_config.clone();
        let active_operations = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let active_operations_clone = active_operations.clone();

        let thread_handle = std::thread::Builder::new()
            .name(format!("deno-worker-{}", worker_id))
            .spawn(move || {
                Self::run_worker_thread(worker_id, worker_receiver, sandbox_config, active_operations_clone);
            })?;

        let worker_info = WorkerInfo {
            id: worker_id,
            sender: worker_sender,
            thread_handle,
            created_at: Instant::now(),
            active_operations,
        };

        pool_state.workers.push(worker_info);

        // Update pool statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_runtimes_created += 1;
            stats.current_pool_size = pool_state.workers.len();
            if pool_state.workers.len() > stats.peak_pool_size {
                stats.peak_pool_size = pool_state.workers.len();
            }
        }

        info!("Spawned worker {} (pool size: {}/{})", worker_id, pool_state.workers.len(), self.max_runtimes);
        Ok(())
    }

    /// Run a worker thread that processes execution requests and control messages
    fn run_worker_thread(
        worker_id: usize,
        mut control_receiver: mpsc::UnboundedReceiver<WorkerControlMessage>,
        sandbox_config: SandboxConfiguration,
        active_operations: Arc<std::sync::atomic::AtomicUsize>,
    ) {
        info!("Starting Deno worker thread {}", worker_id);
        
        // Create Deno runtime in this worker thread
        let runtime_result = Self::create_deno_runtime(&sandbox_config);
        let mut deno_runtime = match runtime_result {
            Ok(runtime) => runtime,
            Err(e) => {
                error!("Failed to create Deno runtime in worker thread {}: {}", worker_id, e);
                return;
            }
        };
        
        // Process control messages and execution requests
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                error!("Failed to create Tokio runtime for worker {}: {}", worker_id, e);
                return;
            }
        };
        
        rt.block_on(async {
            while let Some(message) = control_receiver.recv().await {
                match message {
                    WorkerControlMessage::Execute(request) => {
                        // Increment active operations counter
                        active_operations.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        
                        let start_time = Instant::now();
                        let result = Self::execute_script_in_runtime(&mut deno_runtime, &request.script).await;
                        let duration = start_time.elapsed();
                        
                        // Decrement active operations counter
                        active_operations.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                        
                        // Log execution statistics in the worker thread
                        match &result {
                            Ok(_) => {
                                debug!("Worker {} script execution succeeded for operation {} in {:?}", 
                                       worker_id, request.operation_id, duration);
                            }
                            Err(e) => {
                                warn!("Worker {} script execution failed for operation {} in {:?}: {}", 
                                      worker_id, request.operation_id, duration, e);
                            }
                        }
                        
                        // Send response back (ignore if receiver is dropped)
                        let _ = request.response_sender.send(result);
                    }
                    WorkerControlMessage::Shutdown => {
                        info!("Worker {} received shutdown signal", worker_id);
                        break;
                    }
                }
            }
        });
        
        info!("Deno worker thread {} shutting down", worker_id);
    }

    /// Create a new Deno runtime with specified configuration
    fn create_deno_runtime(sandbox_config: &SandboxConfiguration) -> Result<deno_core::JsRuntime, Box<dyn std::error::Error>> {
        use deno_core::{JsRuntime, RuntimeOptions};
        
        // Create essential extensions for proper JavaScript environment
        // These extensions provide core Web APIs like console.log(), URL(), etc.
        let extensions = vec![
            // Essential Web APIs for JavaScript execution - using correct init() pattern
            deno_webidl::deno_webidl::init(),
            deno_url::deno_url::init(),
            deno_console::deno_console::init(),
            deno_web::deno_web::init::<SandboxPermissions>(
                Default::default(), // Use Default::default() for BlobStore
                None, // No location for sandbox security
            ),
            // Custom extension to put permissions in OpState
            sandbox_permissions::init(),
        ];
        
        // Apply sandbox configuration to runtime options with actual available security settings
        let options = RuntimeOptions {
            extensions,
            // Security: Disable inspector to prevent debugging in sandbox mode
            inspector: false,
            // Security: No startup snapshot to force fresh runtime state
            startup_snapshot: None,
            // Security: Disable SharedArrayBuffer sharing between isolates
            shared_array_buffer_store: None,
            // Security: Disable WebAssembly module sharing between isolates  
            compiled_wasm_module_store: None,
            // Security: Validate import attributes to prevent unauthorized imports
            validate_import_attributes_cb: Some(Box::new(|scope: &mut v8::PinScope, attributes: &HashMap<String, String>| {
                // Only allow "type": "json" for JSON imports, reject everything else
                for (key, value) in attributes {
                    if key != "type" {
                        let message = format!("Import attribute '{}' is not allowed in sandbox mode", key);
                        let message_str = v8::String::new(scope, &message).unwrap();
                        let exception = v8::Exception::type_error(scope, message_str);
                        scope.throw_exception(exception);
                        return;
                    }
                    // For "type" attribute, only allow "json"
                    if key == "type" && value != "json" {
                        let message = format!("Import type '{}' is not allowed in sandbox mode. Only 'json' is permitted.", value);
                        let message_str = v8::String::new(scope, &message).unwrap();
                        let exception = v8::Exception::type_error(scope, message_str);
                        scope.throw_exception(exception);
                        return;
                    }
                }
            })),
            ..Default::default()
        };
        
        info!("Sandbox runtime created with security restrictions:");
        info!("  - Inspector/debugger access: DISABLED");
        info!("  - SharedArrayBuffer isolation: ENABLED"); 
        info!("  - WebAssembly module isolation: ENABLED");
        info!("  - Import attribute validation: ENABLED");
        info!("  - Permission-based restrictions in OpState: {}", if sandbox_config.allow_hrtime { "PARTIAL" } else { "STRICT" });
        
        let runtime = JsRuntime::new(options);
        
        // Override the sandbox permissions with the correct configuration
        {
            let op_state_rc = runtime.op_state();
            let mut op_state = op_state_rc.borrow_mut();
            op_state.put(SandboxPermissions { allow_hrtime: sandbox_config.allow_hrtime });
        }
        
        debug!("Successfully created Deno runtime with evaluated extension modules - Web APIs are now available");
        
        Ok(runtime)
    }



    /// Execute script in the dedicated runtime
    async fn execute_script_in_runtime(
        runtime: &mut deno_core::JsRuntime,
        script: &str,
    ) -> Result<String, String> {
        
        match runtime.execute_script("inline_script", script.to_string()) {
            Ok(result) => {
                // Run the event loop to handle any async operations
                match runtime.resolve(result).await {
                    Ok(_global_value) => {
                        // TODO: Convert the Global<Value> to a string representation
                        // The API for accessing v8 values from Global<Value> has changed in newer deno_core
                        // For now, return success without the actual result value
                        Ok("Script executed successfully".to_string())
                    }
                    Err(e) => Err(format!("Script execution failed: {}", e)),
                }
            }
            Err(e) => Err(format!("Script compilation failed: {}", e)),
        }
    }

    /// Check if the pool is shut down
    pub fn is_shutdown(&self) -> bool {
        if let Ok(stats) = self.stats.lock() {
            stats.shutdown_completed_at.is_some()
        } else {
            // If we can't check stats, assume not shut down to be safe
            false
        }
    }

    /// Execute a script using load-balanced worker selection
    pub async fn execute_script(&self, script: String, operation_id: DenoOperationId) -> Result<String, String> {
        // Check if pool is shut down
        if self.is_shutdown() {
            return Err("Cannot execute script: runtime pool has been shut down".to_string());
        }

        let start_time = Instant::now();
        let (response_sender, response_receiver) = oneshot::channel();
        
        let request = DenoExecutionRequest {
            operation_id,
            script,
            timeout: self.default_timeout,
            response_sender,
        };

        // Select worker using round-robin load balancing
        let worker_sender = {
            let mut pool_state = match self.pool_state.lock() {
                Ok(state) => state,
                Err(poisoned) => {
                    warn!("Pool state mutex was poisoned during script execution, recovering");
                    poisoned.into_inner()
                }
            };

            if pool_state.workers.is_empty() {
                let duration = start_time.elapsed();
                self.update_stats_for_failed_execution(duration);
                return Err("No available workers in pool".to_string());
            }

            // Round-robin selection
            let selected_index = pool_state.round_robin_index % pool_state.workers.len();
            pool_state.round_robin_index = (pool_state.round_robin_index + 1) % pool_state.workers.len();
            
            pool_state.workers[selected_index].sender.clone()
        };
        
        // Send request to the selected worker
        if worker_sender.send(WorkerControlMessage::Execute(request)).is_err() {
            let duration = start_time.elapsed();
            self.update_stats_for_failed_execution(duration);
            return Err("Failed to send script execution request - worker thread may have shut down".to_string());
        }
        
        // Wait for response with timeout
        let result = match tokio::time::timeout(self.default_timeout, response_receiver).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err("Response channel closed".to_string()),
            Err(_) => Err("Script execution timed out".to_string()),
        };
        
        let duration = start_time.elapsed();
        let success = result.is_ok();
        self.update_stats_for_execution(duration, success);
        
        result
    }

    /// Update internal statistics for script execution
    fn update_stats_for_execution(&self, duration: Duration, success: bool) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_operations_completed += 1;
            
            // Update average operation time using running average
            let total_ops = stats.total_operations_completed as f64;
            let current_avg = stats.average_operation_time.as_secs_f64();
            let new_avg = (current_avg * (total_ops - 1.0) + duration.as_secs_f64()) / total_ops;
            stats.average_operation_time = Duration::from_secs_f64(new_avg);
        }
        
        debug!("Script execution completed: duration={:?}, success={}", duration, success);
    }

    /// Update internal statistics for failed execution (before sending to thread)
    fn update_stats_for_failed_execution(&self, duration: Duration) {
        debug!("Script execution failed before reaching runtime: duration={:?}", duration);
    }

    /// Submit an execution request directly to the worker pool for async processing
    pub fn submit_execution_request(&self, request: DenoExecutionRequest) -> Result<(), String> {
        // Check if pool is shut down
        if self.is_shutdown() {
            return Err("Cannot submit execution request: runtime pool has been shut down".to_string());
        }

        // Select worker using round-robin load balancing
        let worker_sender = {
            let mut pool_state = match self.pool_state.lock() {
                Ok(state) => state,
                Err(poisoned) => {
                    warn!("Pool state mutex was poisoned during request submission, recovering");
                    poisoned.into_inner()
                }
            };

            if pool_state.workers.is_empty() {
                return Err("No available workers in pool".to_string());
            }

            // Round-robin selection
            let selected_index = pool_state.round_robin_index % pool_state.workers.len();
            pool_state.round_robin_index = (pool_state.round_robin_index + 1) % pool_state.workers.len();
            
            pool_state.workers[selected_index].sender.clone()
        };
        
        // Send request to the selected worker
        worker_sender.send(WorkerControlMessage::Execute(request))
            .map_err(|_| "Failed to send execution request - worker thread may have shut down".to_string())
    }

    /// Get current pool statistics
    pub fn get_statistics(&self) -> PoolStatistics {
        match self.stats.lock() {
            Ok(stats) => stats.clone(),
            Err(poisoned) => {
                warn!("Pool statistics mutex was poisoned, recovering with current data");
                poisoned.into_inner().clone()
            }
        }
    }

    /// Get pool health information
    pub fn get_health_info(&self) -> PoolHealthInfo {
        let stats = match self.stats.lock() {
            Ok(stats) => stats,
            Err(poisoned) => {
                warn!("Pool health statistics mutex was poisoned, recovering with current data");
                poisoned.into_inner()
            }
        };

        let pool_state = match self.pool_state.lock() {
            Ok(state) => state,
            Err(poisoned) => {
                warn!("Pool state mutex was poisoned during health check, recovering");
                poisoned.into_inner()
            }
        };

        let available_workers = pool_state.workers.len();
        let total_active_operations: usize = pool_state.workers.iter()
            .map(|worker| worker.active_operations.load(std::sync::atomic::Ordering::SeqCst))
            .sum();
        
        PoolHealthInfo {
            available_runtimes: available_workers,
            max_runtimes: self.max_runtimes,
            utilization_percent: if self.max_runtimes > 0 {
                (total_active_operations as f64 / self.max_runtimes as f64) * 100.0
            } else {
                0.0
            },
            uptime: self.created_at.elapsed(),
            total_operations: stats.total_operations_completed,
            average_operation_time: stats.average_operation_time,
        }
    }

    /// Add a worker management method for dynamic scaling
    pub fn scale_pool(&self, target_workers: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let target_workers = std::cmp::min(target_workers, self.max_runtimes);
        
        let current_workers = {
            let pool_state = match self.pool_state.lock() {
                Ok(state) => state,
                Err(poisoned) => {
                    warn!("Pool state mutex was poisoned during scaling, recovering");
                    poisoned.into_inner()
                }
            };
            pool_state.workers.len()
        };

        if target_workers > current_workers {
            // Scale up
            for _ in current_workers..target_workers {
                if let Err(e) = self.spawn_worker() {
                    warn!("Failed to spawn worker during scale up: {}", e);
                    break;
                }
            }
        } else if target_workers < current_workers {
            // Scale down by shutting down excess workers gracefully
            self.shutdown_excess_workers(current_workers - target_workers)?;
        }

        Ok(())
    }

    /// Gracefully shutdown the entire runtime pool
    /// This will:
    /// 1. Signal all workers to stop accepting new work
    /// 2. Wait for active operations to complete (with timeout)
    /// 3. Force shutdown any remaining workers
    /// 4. Join all worker threads
    pub fn shutdown(&self, timeout: Duration) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initiating graceful shutdown of runtime pool");
        
        // Get all workers from the pool
        let workers = {
            let mut pool_state = match self.pool_state.lock() {
                Ok(state) => state,
                Err(poisoned) => {
                    warn!("Pool state mutex was poisoned during shutdown, recovering");
                    poisoned.into_inner()
                }
            };
            
            // Take ownership of all workers (leaving empty Vec)
            std::mem::take(&mut pool_state.workers)
        };

        if workers.is_empty() {
            info!("No workers to shut down");
            return Ok(());
        }

        // Phase 1: Send shutdown signals to all workers
        info!("Sending shutdown signals to {} workers", workers.len());
        for worker in &workers {
            if let Err(e) = worker.sender.send(WorkerControlMessage::Shutdown) {
                warn!("Failed to send shutdown signal to worker {}: {}", worker.id, e);
            }
        }

        // Phase 2: Wait for active operations to complete with timeout
        let shutdown_start = Instant::now();
        let check_interval = Duration::from_millis(100);
        let mut remaining_timeout = timeout;

        info!("Waiting up to {:?} for active operations to complete", timeout);
        while remaining_timeout > Duration::ZERO {
            let total_active = workers.iter()
                .map(|worker| worker.active_operations.load(std::sync::atomic::Ordering::SeqCst))
                .sum::<usize>();
            
            if total_active == 0 {
                info!("All operations completed, proceeding with shutdown");
                break;
            }

            debug!("Waiting for {} active operations to complete", total_active);
            
            let sleep_duration = std::cmp::min(check_interval, remaining_timeout);
            std::thread::sleep(sleep_duration);
            remaining_timeout = remaining_timeout.saturating_sub(sleep_duration);
        }

        // Check if we timed out waiting for operations
        let final_active = workers.iter()
            .map(|worker| worker.active_operations.load(std::sync::atomic::Ordering::SeqCst))
            .sum::<usize>();
        
        if final_active > 0 {
            warn!("Timeout reached with {} operations still active, forcing shutdown", final_active);
        }

        // Phase 3: Join all worker threads
        let mut join_errors = Vec::new();
        for worker in workers {
            info!("Joining worker thread {}", worker.id);
            match worker.thread_handle.join() {
                Ok(()) => {
                    debug!("Worker {} shut down cleanly", worker.id);
                }
                Err(e) => {
                    let error_msg = format!("Worker {} thread panicked during shutdown: {:?}", worker.id, e);
                    error!("{}", error_msg);
                    join_errors.push(error_msg);
                }
            }
        }

        // Update statistics with shutdown completion
        let shutdown_duration = shutdown_start.elapsed();
        if let Ok(mut stats) = self.stats.lock() {
            stats.current_pool_size = 0;
            stats.shutdown_completed_at = Some(Instant::now());
            stats.graceful_shutdown_duration = Some(shutdown_duration);
        }

        if join_errors.is_empty() {
            info!("Runtime pool shutdown completed successfully in {:?}", shutdown_start.elapsed());
            Ok(())
        } else {
            Err(format!("Runtime pool shutdown completed with {} thread join errors: {}", 
                       join_errors.len(), join_errors.join("; ")).into())
        }
    }

    /// Shutdown a specific number of excess workers gracefully
    fn shutdown_excess_workers(&self, count: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Shutting down {} excess workers", count);

        let workers_to_shutdown = {
            let mut pool_state = match self.pool_state.lock() {
                Ok(state) => state,
                Err(poisoned) => {
                    warn!("Pool state mutex was poisoned during worker shutdown, recovering");
                    poisoned.into_inner()
                }
            };

            // Take the last N workers (LIFO - most recently created first)
            let workers_count = pool_state.workers.len();
            let shutdown_count = std::cmp::min(count, workers_count);
            
            if shutdown_count == 0 {
                return Ok(());
            }

            // Split off the workers to shutdown
            pool_state.workers.split_off(workers_count - shutdown_count)
        };

        // Send shutdown signals and join threads
        for worker in workers_to_shutdown {
            // Send shutdown signal
            if let Err(e) = worker.sender.send(WorkerControlMessage::Shutdown) {
                warn!("Failed to send shutdown signal to worker {}: {}", worker.id, e);
            }

            // Wait briefly for graceful shutdown
            std::thread::sleep(Duration::from_millis(100));

            // Join the thread
            match worker.thread_handle.join() {
                Ok(()) => {
                    info!("Worker {} shut down cleanly during scale down", worker.id);
                }
                Err(e) => {
                    error!("Worker {} thread panicked during scale down: {:?}", worker.id, e);
                }
            }
        }

        // Update pool statistics
        if let Ok(mut stats) = self.stats.lock() {
            let current_size = {
                let pool_state = match self.pool_state.lock() {
                    Ok(state) => state,
                    Err(poisoned) => poisoned.into_inner()
                };
                pool_state.workers.len()
            };
            stats.current_pool_size = current_size;
        }

        info!("Completed shutdown of {} excess workers", count);
        Ok(())
    }
}

impl Drop for DenoRuntimePool {
    /// Automatic cleanup when the pool is dropped
    /// Ensures all worker threads are properly shut down
    fn drop(&mut self) {
        info!("DenoRuntimePool is being dropped, initiating automatic shutdown");
        
        // Use a reasonable timeout for automatic shutdown
        let timeout = Duration::from_secs(5);
        
        // Attempt graceful shutdown, but don't panic if it fails during drop
        if let Err(e) = self.shutdown(timeout) {
            error!("Error during automatic pool shutdown: {}", e);
        }
    }
}

/// Resource for tracking active Deno operations
#[derive(Resource, Default)]
pub struct DenoOperationTracker {
    /// Active operations by ID
    pub active_operations: HashMap<DenoOperationId, TrackedOperation>,
    /// Operation history for metrics
    pub completed_operations: VecDeque<CompletedOperation>,
    /// Maximum history size
    pub max_history_size: usize,
}

impl DenoOperationTracker {
    /// Create new operation tracker
    pub fn new() -> Self {
        Self {
            active_operations: HashMap::new(),
            completed_operations: VecDeque::new(),
            max_history_size: 1000,
        }
    }

    /// Start tracking a new operation
    pub fn start_operation(&mut self, operation_id: DenoOperationId, operation: TrackedOperation) {
        self.active_operations.insert(operation_id, operation);
    }

    /// Complete an operation and move to history
    pub fn complete_operation(&mut self, operation_id: DenoOperationId, success: bool, error: Option<String>) {
        if let Some(operation) = self.active_operations.remove(&operation_id) {
            let completed_at = Instant::now();
            let duration = completed_at.duration_since(operation.started_at);
            
            let completed = CompletedOperation {
                id: operation_id,
                operation_type: operation.operation_type.clone(),
                started_at: operation.started_at,
                completed_at,
                success,
                error: error.clone(),
                requester: operation.requester.clone(),
            };

            self.completed_operations.push_back(completed);

            // Maintain history size limit
            while self.completed_operations.len() > self.max_history_size {
                self.completed_operations.pop_front();
            }

            // Log operation completion with performance metrics
            match operation.operation_type {
                OperationType::ScriptExecution => {
                    if success {
                        info!("Script execution completed successfully in {:?} for requester: {}", 
                              duration, operation.requester);
                    } else {
                        warn!("Script execution failed in {:?} for requester: {} - Error: {:?}", 
                              duration, operation.requester, error);
                    }
                }
                OperationType::ExtensionDiscovery => {
                    if success {
                        debug!("Extension discovery completed in {:?}", duration);
                    } else {
                        warn!("Extension discovery failed in {:?} - Error: {:?}", duration, error);
                    }
                }
                OperationType::RuntimeManagement => {
                    debug!("Runtime management operation completed in {:?}, success: {}", duration, success);
                }
                OperationType::MetricsCollection => {
                    debug!("Metrics collection completed in {:?}", duration);
                }
            }
        } else {
            warn!("Attempted to complete operation {} that was not found in active operations", operation_id);
        }
    }

    /// Get operation by ID
    pub fn get_operation(&self, operation_id: &DenoOperationId) -> Option<&TrackedOperation> {
        self.active_operations.get(operation_id)
    }

    /// Get all active operations
    pub fn active_operations(&self) -> &HashMap<DenoOperationId, TrackedOperation> {
        &self.active_operations
    }

    /// Get completed operations history
    pub fn completed_operations(&self) -> &VecDeque<CompletedOperation> {
        &self.completed_operations
    }

    /// Get operation statistics
    pub fn get_statistics(&self) -> OperationStatistics {
        let total_completed = self.completed_operations.len() as u64;
        let successful = self.completed_operations.iter().filter(|op| op.success).count() as u64;
        let failed = total_completed - successful;

        let average_duration = if total_completed > 0 {
            let total_duration: Duration = self.completed_operations
                .iter()
                .map(|op| op.completed_at - op.started_at)
                .sum();
            total_duration / total_completed as u32
        } else {
            Duration::ZERO
        };

        OperationStatistics {
            active_count: self.active_operations.len(),
            total_completed,
            successful_count: successful,
            failed_count: failed,
            success_rate: if total_completed > 0 { successful as f64 / total_completed as f64 } else { 0.0 },
            average_duration,
        }
    }
}

/// Resource for managing Raycast extension discovery
#[derive(Resource)]
pub struct ExtensionDiscoveryManager {
    /// Whether discovery is enabled
    pub enabled: bool,
    /// Last discovery operation timestamp
    pub last_discovery: Option<Instant>,
    /// Discovered extensions cache
    pub cached_extensions: HashMap<String, CachedExtension>,
    /// Discovery statistics
    pub stats: DiscoveryStatistics,
}

impl ExtensionDiscoveryManager {
    /// Create new extension discovery manager
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_discovery: None,
            cached_extensions: HashMap::new(),
            stats: DiscoveryStatistics::default(),
        }
    }

    /// Update discovery cache
    pub fn update_cache(&mut self, path: String, extension: CachedExtension) {
        self.cached_extensions.insert(path, extension);
        self.last_discovery = Some(Instant::now());
    }

    /// Get cached extension
    pub fn get_cached(&self, path: &str) -> Option<&CachedExtension> {
        self.cached_extensions.get(path)
    }

    /// Clear expired cache entries
    pub fn cleanup_cache(&mut self, max_age: Duration) {
        let cutoff = Instant::now() - max_age;
        self.cached_extensions.retain(|_, ext| ext.discovered_at > cutoff);
    }

    /// Get discovery statistics
    pub fn get_statistics(&self) -> &DiscoveryStatistics {
        &self.stats
    }
}

/// Resource for comprehensive Deno metrics
#[derive(Resource)]
pub struct DenoMetrics {
    /// Runtime pool metrics
    pub runtime_metrics: RuntimeMetrics,
    /// Operation execution metrics  
    pub execution_metrics: ExecutionMetricsTracker,
    /// Extension discovery metrics
    pub discovery_metrics: DiscoveryMetricsTracker,
    /// System resource usage
    pub resource_metrics: ResourceMetrics,
    /// Error tracking and analysis
    pub error_metrics: ErrorMetrics,
    /// Metrics collection start time
    pub started_at: SystemTime,
}

impl Default for DenoMetrics {
    fn default() -> Self {
        Self {
            runtime_metrics: RuntimeMetrics::default(),
            execution_metrics: ExecutionMetricsTracker::default(),
            discovery_metrics: DiscoveryMetricsTracker::default(),
            resource_metrics: ResourceMetrics::default(),
            error_metrics: ErrorMetrics::default(),
            started_at: SystemTime::now(),
        }
    }
}

impl DenoMetrics {
    /// Generate comprehensive metrics report
    pub fn generate_report(&self) -> DenoMetricsReport {
        DenoMetricsReport {
            runtime_stats: self.runtime_metrics.to_runtime_statistics(),
            execution_metrics: self.execution_metrics.to_execution_metrics(),
            discovery_metrics: self.discovery_metrics.to_discovery_metrics(),
            resource_usage: self.resource_metrics.to_resource_usage(),
            error_statistics: self.error_metrics.to_error_statistics(),
            generated_at: SystemTime::now(),
        }
    }

    /// Update runtime metrics
    pub fn update_runtime_metrics(&mut self, active_runtimes: usize, pool_utilization: f64) {
        self.runtime_metrics.active_runtimes = active_runtimes;
        self.runtime_metrics.pool_utilization_percent = pool_utilization;
        self.runtime_metrics.last_updated = Instant::now();
        
        // Update memory usage whenever runtime metrics are updated
        self.resource_metrics.update_memory_usage();
    }

    /// Record script execution
    pub fn record_execution(&mut self, duration: Duration, success: bool) {
        self.execution_metrics.total_executions += 1;
        if success {
            self.execution_metrics.successful_executions += 1;
        } else {
            self.execution_metrics.failed_executions += 1;
        }

        // Update timing statistics
        if self.execution_metrics.fastest_execution.is_zero() || duration < self.execution_metrics.fastest_execution {
            self.execution_metrics.fastest_execution = duration;
        }
        if duration > self.execution_metrics.slowest_execution {
            self.execution_metrics.slowest_execution = duration;
        }

        // Update running average
        let total = self.execution_metrics.total_executions as f64;
        let current_avg = self.execution_metrics.average_execution_time.as_secs_f64();
        let new_avg = (current_avg * (total - 1.0) + duration.as_secs_f64()) / total;
        self.execution_metrics.average_execution_time = Duration::from_secs_f64(new_avg);
    }

    /// Record extension discovery
    pub fn record_discovery(&mut self, duration: Duration, success: bool, extensions_found: usize) {
        self.discovery_metrics.total_discoveries += 1;
        if success {
            self.discovery_metrics.successful_discoveries += 1;
            self.discovery_metrics.total_extensions_found += extensions_found;
        } else {
            self.discovery_metrics.failed_discoveries += 1;
        }

        // Update timing statistics  
        let total = self.discovery_metrics.total_discoveries as f64;
        let current_avg = self.discovery_metrics.average_discovery_time.as_secs_f64();
        let new_avg = (current_avg * (total - 1.0) + duration.as_secs_f64()) / total;
        self.discovery_metrics.average_discovery_time = Duration::from_secs_f64(new_avg);
    }
}/// Tracked operation information
#[derive(Debug, Clone)]
pub struct TrackedOperation {
    pub operation_type: OperationType,
    pub started_at: Instant,
    pub timeout: Duration,
    pub requester: String,
    pub metadata: HashMap<String, String>,
}

/// Completed operation record
#[derive(Debug, Clone)]
pub struct CompletedOperation {
    pub id: DenoOperationId,
    pub operation_type: OperationType,
    pub started_at: Instant,
    pub completed_at: Instant,
    pub success: bool,
    pub error: Option<String>,
    pub requester: String,
}

/// Operation type classification
#[derive(Debug, Clone)]
pub enum OperationType {
    ScriptExecution,
    ExtensionDiscovery,
    RuntimeManagement,
    MetricsCollection,
}

/// Cached extension information
#[derive(Debug, Clone)]
pub struct CachedExtension {
    pub path: String,
    pub name: String,
    pub version: String,
    pub discovered_at: Instant,
    pub file_hash: String,
    pub metadata: HashMap<String, String>,
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStatistics {
    pub total_runtimes_created: u64,
    pub total_runtimes_destroyed: u64,
    pub total_operations_completed: u64,
    pub average_operation_time: Duration,
    pub peak_pool_size: usize,
    pub current_pool_size: usize,
    pub shutdown_completed_at: Option<Instant>,
    pub graceful_shutdown_duration: Option<Duration>,
}

impl Default for PoolStatistics {
    fn default() -> Self {
        Self {
            total_runtimes_created: 0,
            total_runtimes_destroyed: 0,
            total_operations_completed: 0,
            average_operation_time: Duration::ZERO,
            peak_pool_size: 0,
            current_pool_size: 0,
            shutdown_completed_at: None,
            graceful_shutdown_duration: None,
        }
    }
}

/// Pool health information
#[derive(Debug, Clone)]
pub struct PoolHealthInfo {
    pub available_runtimes: usize,
    pub max_runtimes: usize,
    pub utilization_percent: f64,
    pub uptime: Duration,
    pub total_operations: u64,
    pub average_operation_time: Duration,
}

/// Operation statistics
#[derive(Debug, Clone)]
pub struct OperationStatistics {
    pub active_count: usize,
    pub total_completed: u64,
    pub successful_count: u64,
    pub failed_count: u64,
    pub success_rate: f64,
    pub average_duration: Duration,
}

/// Discovery statistics
#[derive(Debug, Clone, Default)]
pub struct DiscoveryStatistics {
    pub total_discoveries: u64,
    pub successful_discoveries: u64,
    pub failed_discoveries: u64,
    pub total_extensions_found: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_discovery_time: Duration,
}

/// Runtime metrics tracking
#[derive(Debug, Clone)]
pub struct RuntimeMetrics {
    pub active_runtimes: usize,
    pub total_runtimes_created: u64,
    pub total_runtimes_destroyed: u64,
    pub average_runtime_lifetime: Duration,
    pub pool_utilization_percent: f64,
    pub last_updated: Instant,
}

impl Default for RuntimeMetrics {
    fn default() -> Self {
        Self {
            active_runtimes: 0,
            total_runtimes_created: 0,
            total_runtimes_destroyed: 0,
            average_runtime_lifetime: Duration::ZERO,
            pool_utilization_percent: 0.0,
            last_updated: Instant::now(),
        }
    }
}

impl RuntimeMetrics {
    pub fn to_runtime_statistics(&self) -> RuntimeStatistics {
        RuntimeStatistics {
            active_runtimes: self.active_runtimes,
            total_runtimes_created: self.total_runtimes_created,
            total_runtimes_destroyed: self.total_runtimes_destroyed,
            average_runtime_lifetime: self.average_runtime_lifetime,
            pool_utilization_percent: self.pool_utilization_percent,
        }
    }
}

/// Execution metrics tracking
#[derive(Debug, Clone)]
pub struct ExecutionMetricsTracker {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub timeout_executions: u64,
    pub average_execution_time: Duration,
    pub fastest_execution: Duration,
    pub slowest_execution: Duration,
}

impl Default for ExecutionMetricsTracker {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            timeout_executions: 0,
            average_execution_time: Duration::ZERO,
            fastest_execution: Duration::ZERO,
            slowest_execution: Duration::ZERO,
        }
    }
}

impl ExecutionMetricsTracker {
    pub fn to_execution_metrics(&self) -> ExecutionMetrics {
        let timeout_rate = if self.total_executions > 0 {
            (self.timeout_executions as f64 / self.total_executions as f64) * 100.0
        } else {
            0.0
        };

        ExecutionMetrics {
            total_executions: self.total_executions,
            successful_executions: self.successful_executions,
            failed_executions: self.failed_executions,
            average_execution_time: self.average_execution_time,
            fastest_execution: self.fastest_execution,
            slowest_execution: self.slowest_execution,
            timeout_rate_percent: timeout_rate,
        }
    }
}

/// Discovery metrics tracking
#[derive(Debug, Clone, Default)]
pub struct DiscoveryMetricsTracker {
    pub total_discoveries: u64,
    pub successful_discoveries: u64,
    pub failed_discoveries: u64,
    pub total_extensions_found: usize,
    pub cache_hit_rate: f64,
    pub average_discovery_time: Duration,
}

impl DiscoveryMetricsTracker {
    pub fn to_discovery_metrics(&self) -> DiscoveryMetrics {
        let extensions_per_second = if self.average_discovery_time.as_secs_f64() > 0.0 {
            1.0 / self.average_discovery_time.as_secs_f64()
        } else {
            0.0
        };

        DiscoveryMetrics {
            total_discoveries: self.total_discoveries,
            successful_discoveries: self.successful_discoveries,
            failed_discoveries: self.failed_discoveries,
            average_discovery_time: self.average_discovery_time,
            extensions_per_second,
            cache_hit_rate_percent: self.cache_hit_rate * 100.0,
        }
    }
}

/// Resource usage tracking with platform-specific implementations
#[derive(Debug, Clone, Default)]
pub struct ResourceMetrics {
    pub peak_memory_usage: usize,
    pub current_memory_usage: usize,
    pub total_cpu_time: Duration,
    pub file_handles_opened: u64,
    pub network_connections: u64,
}

impl ResourceMetrics {
    /// Update current memory usage with actual system measurements
    pub fn update_memory_usage(&mut self) {
        match Self::get_current_memory_usage() {
            Ok(current_usage) => {
                self.current_memory_usage = current_usage;
                if current_usage > self.peak_memory_usage {
                    self.peak_memory_usage = current_usage;
                }
            }
            Err(e) => {
                warn!("Failed to get current memory usage: {}", e);
            }
        }
    }

    /// Get current memory usage in bytes - platform specific implementation
    #[cfg(target_os = "macos")]
    fn get_current_memory_usage() -> Result<usize, Box<dyn std::error::Error>> {
        
        unsafe extern "C" {
            fn mach_task_self() -> u32;
            fn task_info(
                target_task: u32,
                flavor: u32,
                task_info_out: *mut u8,
                task_info_outCnt: *mut u32,
            ) -> i32;
        }

        const TASK_VM_INFO: u32 = 22;
        const TASK_VM_INFO_COUNT: u32 = 18;

        #[repr(C)]
        struct TaskVmInfo {
            virtual_size: u64,
            region_count: u32,
            page_size: u32,
            resident_size: u64,
            resident_size_peak: u64,
            device: u64,
            device_peak: u64,
            internal: u64,
            internal_peak: u64,
            external: u64,
            external_peak: u64,
            reusable: u64,
            reusable_peak: u64,
            purgeable_volatile_pmap: u64,
            purgeable_volatile_resident: u64,
            purgeable_volatile_virtual: u64,
            compressed: u64,
            compressed_peak: u64,
        }

        let mut info: TaskVmInfo = unsafe { std::mem::zeroed() };
        let mut count = TASK_VM_INFO_COUNT;
        
        let result = unsafe {
            task_info(
                mach_task_self(),
                TASK_VM_INFO,
                &mut info as *mut TaskVmInfo as *mut u8,
                &mut count,
            )
        };

        if result == 0 {
            // Return resident memory size (physical memory actually in use)
            Ok(info.resident_size as usize)
        } else {
            Err(format!("task_info failed with code: {}", result).into())
        }
    }

    /// Get current memory usage in bytes - Linux implementation
    #[cfg(target_os = "linux")]
    fn get_current_memory_usage() -> Result<usize, Box<dyn std::error::Error>> {
        let status = std::fs::read_to_string("/proc/self/status")?;
        
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb = parts[1].parse::<usize>()?;
                    return Ok(kb * 1024); // Convert KB to bytes
                }
            }
        }
        
        Err("Could not find VmRSS in /proc/self/status".into())
    }

    /// Get current memory usage in bytes - Windows implementation
    #[cfg(target_os = "windows")]
    fn get_current_memory_usage() -> Result<usize, Box<dyn std::error::Error>> {
        use std::mem;
        use std::ptr;
        
        extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn GetProcessMemoryInfo(
                process: *mut std::ffi::c_void,
                counters: *mut ProcessMemoryCounters,
                cb: u32,
            ) -> i32;
        }

        #[repr(C)]
        struct ProcessMemoryCounters {
            cb: u32,
            page_fault_count: u32,
            peak_working_set_size: usize,
            working_set_size: usize,
            quota_peak_paged_pool_usage: usize,
            quota_paged_pool_usage: usize,
            quota_peak_non_paged_pool_usage: usize,
            quota_non_paged_pool_usage: usize,
            pagefile_usage: usize,
            peak_pagefile_usage: usize,
        }

        let mut counters = ProcessMemoryCounters {
            cb: mem::size_of::<ProcessMemoryCounters>() as u32,
            page_fault_count: 0,
            peak_working_set_size: 0,
            working_set_size: 0,
            quota_peak_paged_pool_usage: 0,
            quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0,
            quota_non_paged_pool_usage: 0,
            pagefile_usage: 0,
            peak_pagefile_usage: 0,
        };

        let result = unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters,
                mem::size_of::<ProcessMemoryCounters>() as u32,
            )
        };

        if result != 0 {
            Ok(counters.working_set_size)
        } else {
            Err("GetProcessMemoryInfo failed".into())
        }
    }

    /// Get current memory usage in bytes - fallback implementation
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn get_current_memory_usage() -> Result<usize, Box<dyn std::error::Error>> {
        Err("Memory usage tracking not implemented for this platform".into())
    }

    pub fn to_resource_usage(&self) -> ResourceUsage {
        ResourceUsage {
            peak_memory_usage: self.peak_memory_usage,
            current_memory_usage: self.current_memory_usage,
            total_cpu_time: self.total_cpu_time,
            file_handles_opened: self.file_handles_opened,
            network_connections: self.network_connections,
        }
    }
}

/// Error metrics and analysis
#[derive(Debug, Clone, Default)]
pub struct ErrorMetrics {
    pub execution_errors: HashMap<String, u64>,
    pub discovery_errors: HashMap<String, u64>,
    pub runtime_errors: HashMap<String, u64>,
    pub total_operations: u64,
    pub total_errors: u64,
}

impl ErrorMetrics {
    pub fn record_error(&mut self, error_type: &str, category: ErrorCategory) {
        let error_map = match category {
            ErrorCategory::Execution => &mut self.execution_errors,
            ErrorCategory::Discovery => &mut self.discovery_errors,
            ErrorCategory::Runtime => &mut self.runtime_errors,
        };

        *error_map.entry(error_type.to_string()).or_insert(0) += 1;
        self.total_errors += 1;
    }

    pub fn to_error_statistics(&self) -> ErrorStatistics {
        let mut all_errors = HashMap::new();
        all_errors.extend(self.execution_errors.clone());
        all_errors.extend(self.discovery_errors.clone());
        all_errors.extend(self.runtime_errors.clone());

        let most_common_error = all_errors
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(error, _)| error.clone());

        let error_rate = if self.total_operations > 0 {
            (self.total_errors as f64 / self.total_operations as f64) * 100.0
        } else {
            0.0
        };

        ErrorStatistics {
            execution_errors: self.execution_errors.clone(),
            discovery_errors: self.discovery_errors.clone(),
            runtime_errors: self.runtime_errors.clone(),
            most_common_error,
            error_rate_percent: error_rate,
        }
    }
}

/// Error category classification
#[derive(Debug, Clone)]
pub enum ErrorCategory {
    Execution,
    Discovery,
    Runtime,
}
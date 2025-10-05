//! Enterprise-grade task management components with comprehensive error handling

use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::tasks::Task;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Comprehensive task type enumeration for all operation categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskType {
    /// Hotkey preference loading/saving operations
    HotkeyPreferences,
    /// File I/O and storage operations  
    StorageOperation,
    /// Clipboard operations (read/write/monitor)
    ClipboardOperation,
    /// Search operations across plugins
    SearchOperation,
    /// Raycast integration operations
    RaycastOperation,
    /// TLS/SSL certificate operations
    TlsOperation,
    /// Cache cleanup and maintenance
    CacheCleanup,
    /// Generic async operations
    Generic,
}

impl TaskType {
    /// Get default timeout for this task type (zero-allocation)
    #[inline]
    pub const fn default_timeout(self) -> Duration {
        match self {
            Self::HotkeyPreferences => Duration::from_secs(5),
            Self::StorageOperation => Duration::from_secs(30),
            Self::ClipboardOperation => Duration::from_secs(2),
            Self::SearchOperation => Duration::from_secs(10),
            Self::RaycastOperation => Duration::from_secs(15),
            Self::TlsOperation => Duration::from_secs(60),
            Self::CacheCleanup => Duration::from_secs(120),
            Self::Generic => Duration::from_secs(15),
        }
    }

    /// Get task type name (zero-allocation)
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::HotkeyPreferences => "HotkeyPreferences",
            Self::StorageOperation => "StorageOperation",
            Self::ClipboardOperation => "ClipboardOperation",
            Self::SearchOperation => "SearchOperation",
            Self::RaycastOperation => "RaycastOperation",
            Self::TlsOperation => "TlsOperation",
            Self::CacheCleanup => "CacheCleanup",
            Self::Generic => "Generic",
        }
    }
}

/// Comprehensive error handling for task operations
#[derive(Debug, Clone)]
pub enum TaskError {
    /// Task exceeded timeout deadline
    Timeout {
        task_id: Uuid,
        task_type: TaskType,
        timeout_ms: u64,
        elapsed_ms: u64,
        message: String,
    },
    /// Task was cancelled before completion
    Cancelled {
        task_id: Uuid,
        task_type: TaskType,
        message: String,
    },
    /// Task execution failed with error
    Execution {
        task_id: Uuid,
        task_type: TaskType,
        message: String,
        source: Option<String>,
    },
    /// System-level error occurred
    System {
        task_id: Uuid,
        task_type: TaskType,
        message: String,
        error_code: Option<i32>,
    },
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::Timeout {
                task_id,
                task_type,
                timeout_ms,
                elapsed_ms,
                message,
            } => {
                write!(
                    f,
                    "Task {} ({}) timed out after {}ms (limit: {}ms): {}",
                    task_id,
                    task_type.name(),
                    elapsed_ms,
                    timeout_ms,
                    message
                )
            },
            TaskError::Cancelled {
                task_id,
                task_type,
                message,
            } => {
                write!(
                    f,
                    "Task {} ({}) was cancelled: {}",
                    task_id,
                    task_type.name(),
                    message
                )
            },
            TaskError::Execution {
                task_id,
                task_type,
                message,
                source,
            } => {
                if let Some(src) = source {
                    write!(
                        f,
                        "Task {} ({}) execution failed: {} (source: {})",
                        task_id,
                        task_type.name(),
                        message,
                        src
                    )
                } else {
                    write!(
                        f,
                        "Task {} ({}) execution failed: {}",
                        task_id,
                        task_type.name(),
                        message
                    )
                }
            },
            TaskError::System {
                task_id,
                task_type,
                message,
                error_code,
            } => {
                if let Some(code) = error_code {
                    write!(
                        f,
                        "Task {} ({}) system error [{}]: {}",
                        task_id,
                        task_type.name(),
                        code,
                        message
                    )
                } else {
                    write!(
                        f,
                        "Task {} ({}) system error: {}",
                        task_id,
                        task_type.name(),
                        message
                    )
                }
            },
        }
    }
}

impl std::error::Error for TaskError {}

/// Standard result type for all task operations
pub type TaskResult<T> = Result<T, TaskError>;

/// Type alias for completion callbacks (zero-allocation function pointers)
pub type TaskCallback<T> = fn(TaskResult<T>);

/// Enhanced task operation component with result handling and callbacks
#[derive(Component)]
pub struct TaskOperation<T: Send + Sync + 'static = ()> {
    pub id: Uuid,
    pub task: Task<TaskResult<T>>,
    pub task_type: TaskType,
    pub created_at: Instant,
    pub timeout_duration: Duration,
    pub completion_callback: Option<TaskCallback<T>>,
}

impl<T: Send + Sync + 'static> TaskOperation<T> {
    /// Create new task operation with type-specific timeout
    #[inline]
    pub fn new(task: Task<TaskResult<T>>, task_type: TaskType) -> Self {
        Self {
            id: Uuid::new_v4(),
            task,
            task_type,
            created_at: Instant::now(),
            timeout_duration: task_type.default_timeout(),
            completion_callback: None,
        }
    }

    /// Set custom timeout duration
    #[inline]
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout_duration = duration;
        self
    }

    /// Set completion callback
    #[inline]
    pub fn with_callback(mut self, callback: TaskCallback<T>) -> Self {
        self.completion_callback = Some(callback);
        self
    }

    /// Check if task has expired (zero-allocation)
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.timeout_duration
    }

    /// Get elapsed time in milliseconds
    #[inline]
    pub fn elapsed_ms(&self) -> u64 {
        self.created_at.elapsed().as_millis() as u64
    }
}

/// Component for storing completed task results with automatic cleanup
#[derive(Component)]
pub struct TaskResultComponent<T: Send + Sync + 'static> {
    pub result: T,
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub completed_at: Instant,
}

impl<T: Send + Sync + 'static> TaskResultComponent<T> {
    #[inline]
    pub fn new(result: T, task_id: Uuid, task_type: TaskType) -> Self {
        Self {
            result,
            task_id,
            task_type,
            completed_at: Instant::now(),
        }
    }
}

/// Component for scheduling task result cleanup after delay
#[derive(Component)]
pub struct TaskResultCleanupTimer<T: Send + Sync + 'static> {
    pub cleanup_deadline: Instant,
    pub cleanup_delay: Duration,
    pub created_at: Instant,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync + 'static> TaskResultCleanupTimer<T> {
    #[inline]
    pub fn new(delay: Duration) -> Self {
        let now = Instant::now();
        Self {
            cleanup_deadline: now + delay,
            cleanup_delay: delay,
            created_at: now,
            _phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn is_ready_for_cleanup(&self, current_time: Instant) -> bool {
        current_time >= self.cleanup_deadline
    }
}

/// Specialized task components for each operation type
macro_rules! define_task_component {
    ($name:ident, $task_type:expr, $result_type:ty) => {
        #[derive(Component)]
        pub struct $name {
            pub base: TaskOperation<$result_type>,
        }

        impl $name {
            #[inline]
            pub fn new(task: Task<TaskResult<$result_type>>) -> Self {
                Self {
                    base: TaskOperation::new(task, $task_type),
                }
            }

            #[inline]
            pub fn with_timeout(mut self, duration: Duration) -> Self {
                self.base = self.base.with_timeout(duration);
                self
            }

            #[inline]
            pub fn with_callback(mut self, callback: TaskCallback<$result_type>) -> Self {
                self.base = self.base.with_callback(callback);
                self
            }
        }
    };
}

// Define specialized task components for each operation type
define_task_component!(
    HotkeyPreferencesLoadTask,
    TaskType::HotkeyPreferences,
    HotkeyPreferencesResult
);
define_task_component!(
    HotkeyPreferencesPersistTask,
    TaskType::HotkeyPreferences,
    std::path::PathBuf
);
define_task_component!(
    StorageOperationTask,
    TaskType::StorageOperation,
    std::path::PathBuf
);
define_task_component!(ClipboardOperationTask, TaskType::ClipboardOperation, String);
define_task_component!(SearchOperationTask, TaskType::SearchOperation, Vec<String>);
define_task_component!(RaycastOperationTask, TaskType::RaycastOperation, String);
define_task_component!(TlsOperationTask, TaskType::TlsOperation, ());
define_task_component!(CacheCleanupTask, TaskType::CacheCleanup, u64);
define_task_component!(GenericTask, TaskType::Generic, String);

/// Data types for hotkey operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyPreferencesResult {
    pub preferred_combinations: Vec<HotkeyDefinition>,
    pub auto_fallback: bool,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyDefinition {
    pub modifiers: u32,
    pub code: u32,
    pub description: String,
    pub enabled: bool,
}

/// Enhanced task statistics resource with per-type tracking
#[derive(Resource)]
pub struct TaskStatistics {
    pub total_spawned: u64,
    pub total_completed: u64,
    pub total_failed: u64,
    pub total_expired: u64,
    pub total_cancelled: u64,
    pub active_tasks_by_type: HashMap<TaskType, u64>,
    pub completion_times: HashMap<TaskType, Vec<u64>>,
    pub last_reset: Instant,
}

impl Default for TaskStatistics {
    fn default() -> Self {
        Self {
            total_spawned: 0,
            total_completed: 0,
            total_failed: 0,
            total_expired: 0,
            total_cancelled: 0,
            active_tasks_by_type: HashMap::new(),
            completion_times: HashMap::new(),
            last_reset: Instant::now(),
        }
    }
}

impl TaskStatistics {
    /// Get active task count for specific type (zero-allocation)
    #[inline]
    pub fn active_task_count(&self, task_type: TaskType) -> u64 {
        self.active_tasks_by_type
            .get(&task_type)
            .copied()
            .unwrap_or(0)
    }

    /// Get total active task count (optimized)
    #[inline]
    pub fn total_active_tasks(&self) -> u64 {
        self.active_tasks_by_type.values().sum()
    }

    /// Increment active task count for type
    #[inline]
    pub fn increment_active(&mut self, task_type: TaskType) {
        *self.active_tasks_by_type.entry(task_type).or_insert(0) += 1;
        self.total_spawned += 1;
    }

    /// Decrement active task count for type  
    #[inline]
    pub fn decrement_active(&mut self, task_type: TaskType) {
        if let Some(count) = self.active_tasks_by_type.get_mut(&task_type) {
            *count = count.saturating_sub(1);
        }
    }

    /// Record task completion time
    #[inline]
    pub fn record_completion(&mut self, task_type: TaskType, duration_ms: u64) {
        self.decrement_active(task_type);
        self.total_completed += 1;
        self.completion_times
            .entry(task_type)
            .or_default()
            .push(duration_ms);
    }

    /// Record task failure
    #[inline]
    pub fn record_failure(&mut self, task_type: TaskType) {
        self.decrement_active(task_type);
        self.total_failed += 1;
    }

    /// Record task expiration
    #[inline]
    pub fn record_expiration(&mut self, task_type: TaskType) {
        self.decrement_active(task_type);
        self.total_expired += 1;
    }
}

/// Enhanced configuration resource with per-type settings
#[derive(Resource)]
pub struct TaskManagementConfig {
    pub default_timeout: Duration,
    pub max_concurrent_tasks: usize,
    pub max_concurrent_per_type: HashMap<TaskType, usize>,
    pub enable_statistics: bool,
    pub enable_result_storage: bool,
    pub result_cleanup_delay: Duration,
    pub statistics_log_interval: Duration,
}

impl Default for TaskManagementConfig {
    fn default() -> Self {
        let mut max_concurrent_per_type = HashMap::new();
        max_concurrent_per_type.insert(TaskType::HotkeyPreferences, 5);
        max_concurrent_per_type.insert(TaskType::StorageOperation, 20);
        max_concurrent_per_type.insert(TaskType::ClipboardOperation, 10);
        max_concurrent_per_type.insert(TaskType::SearchOperation, 15);
        max_concurrent_per_type.insert(TaskType::RaycastOperation, 5);
        max_concurrent_per_type.insert(TaskType::TlsOperation, 3);
        max_concurrent_per_type.insert(TaskType::CacheCleanup, 2);
        max_concurrent_per_type.insert(TaskType::Generic, 50);

        Self {
            default_timeout: Duration::from_secs(30),
            max_concurrent_tasks: 200,
            max_concurrent_per_type,
            enable_statistics: true,
            enable_result_storage: true,
            result_cleanup_delay: Duration::from_secs(300), // 5 minutes
            statistics_log_interval: Duration::from_secs(60), // 1 minute
        }
    }
}

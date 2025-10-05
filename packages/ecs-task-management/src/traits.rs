//! Extension traits for comprehensive task management with zero-allocation optimizations

use std::future::Future;
use std::time::Duration;

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use uuid::Uuid;

use crate::components::*;
use crate::events::*;

/// Comprehensive extension trait for Commands to spawn all task types with zero-allocation
/// optimizations
pub trait TaskSpawner {
    // =============================================================================
    // Hotkey Preferences Operations
    // =============================================================================

    /// Spawn hotkey preferences loading task
    fn spawn_hotkey_preferences_load_task<F>(&mut self, future: F) -> Uuid
    where
        F: Future<Output = TaskResult<HotkeyPreferencesResult>> + Send + 'static;

    /// Spawn hotkey preferences loading task with custom timeout
    fn spawn_hotkey_preferences_load_task_with_timeout<F>(
        &mut self,
        future: F,
        timeout: Duration,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<HotkeyPreferencesResult>> + Send + 'static;

    /// Spawn hotkey preferences loading task with completion callback
    fn spawn_hotkey_preferences_load_task_with_callback<F>(
        &mut self,
        future: F,
        callback: TaskCallback<HotkeyPreferencesResult>,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<HotkeyPreferencesResult>> + Send + 'static;

    /// Spawn hotkey preferences persistence task
    fn spawn_hotkey_preferences_persist_task<F>(&mut self, future: F) -> Uuid
    where
        F: Future<Output = TaskResult<std::path::PathBuf>> + Send + 'static;

    /// Spawn hotkey preferences persistence task with custom timeout
    fn spawn_hotkey_preferences_persist_task_with_timeout<F>(
        &mut self,
        future: F,
        timeout: Duration,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<std::path::PathBuf>> + Send + 'static;

    /// Spawn hotkey preferences persistence task with completion callback
    fn spawn_hotkey_preferences_persist_task_with_callback<F>(
        &mut self,
        future: F,
        callback: TaskCallback<std::path::PathBuf>,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<std::path::PathBuf>> + Send + 'static;

    // =============================================================================
    // Storage Operations
    // =============================================================================

    /// Spawn storage operation task
    fn spawn_storage_operation_task<F>(&mut self, future: F) -> Uuid
    where
        F: Future<Output = TaskResult<std::path::PathBuf>> + Send + 'static;

    /// Spawn storage operation task with custom timeout
    fn spawn_storage_operation_task_with_timeout<F>(
        &mut self,
        future: F,
        timeout: Duration,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<std::path::PathBuf>> + Send + 'static;

    /// Spawn storage operation task with completion callback
    fn spawn_storage_operation_task_with_callback<F>(
        &mut self,
        future: F,
        callback: TaskCallback<std::path::PathBuf>,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<std::path::PathBuf>> + Send + 'static;

    // =============================================================================
    // Clipboard Operations
    // =============================================================================

    /// Spawn clipboard operation task
    fn spawn_clipboard_operation_task<F>(&mut self, future: F) -> Uuid
    where
        F: Future<Output = TaskResult<String>> + Send + 'static;

    /// Spawn clipboard operation task with custom timeout
    fn spawn_clipboard_operation_task_with_timeout<F>(
        &mut self,
        future: F,
        timeout: Duration,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<String>> + Send + 'static;

    /// Spawn clipboard operation task with completion callback
    fn spawn_clipboard_operation_task_with_callback<F>(
        &mut self,
        future: F,
        callback: TaskCallback<String>,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<String>> + Send + 'static;

    // =============================================================================
    // Search Operations
    // =============================================================================

    /// Spawn search operation task
    fn spawn_search_operation_task<F>(&mut self, future: F) -> Uuid
    where
        F: Future<Output = TaskResult<Vec<String>>> + Send + 'static;

    /// Spawn search operation task with custom timeout
    fn spawn_search_operation_task_with_timeout<F>(&mut self, future: F, timeout: Duration) -> Uuid
    where
        F: Future<Output = TaskResult<Vec<String>>> + Send + 'static;

    /// Spawn search operation task with completion callback
    fn spawn_search_operation_task_with_callback<F>(
        &mut self,
        future: F,
        callback: TaskCallback<Vec<String>>,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<Vec<String>>> + Send + 'static;

    // =============================================================================
    // Raycast Operations
    // =============================================================================

    /// Spawn raycast operation task
    fn spawn_raycast_operation_task<F>(&mut self, future: F) -> Uuid
    where
        F: Future<Output = TaskResult<String>> + Send + 'static;

    /// Spawn raycast operation task with custom timeout
    fn spawn_raycast_operation_task_with_timeout<F>(
        &mut self,
        future: F,
        timeout: Duration,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<String>> + Send + 'static;

    /// Spawn raycast operation task with completion callback
    fn spawn_raycast_operation_task_with_callback<F>(
        &mut self,
        future: F,
        callback: TaskCallback<String>,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<String>> + Send + 'static;

    // =============================================================================
    // TLS Operations
    // =============================================================================

    /// Spawn TLS operation task
    fn spawn_tls_operation_task<F>(&mut self, future: F) -> Uuid
    where
        F: Future<Output = TaskResult<()>> + Send + 'static;

    /// Spawn TLS operation task with custom timeout
    fn spawn_tls_operation_task_with_timeout<F>(&mut self, future: F, timeout: Duration) -> Uuid
    where
        F: Future<Output = TaskResult<()>> + Send + 'static;

    /// Spawn TLS operation task with completion callback
    fn spawn_tls_operation_task_with_callback<F>(
        &mut self,
        future: F,
        callback: TaskCallback<()>,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<()>> + Send + 'static;

    // =============================================================================
    // Cache Cleanup Operations
    // =============================================================================

    /// Spawn cache cleanup task
    fn spawn_cache_cleanup_task<F>(&mut self, future: F) -> Uuid
    where
        F: Future<Output = TaskResult<u64>> + Send + 'static;

    /// Spawn cache cleanup task with custom timeout
    fn spawn_cache_cleanup_task_with_timeout<F>(&mut self, future: F, timeout: Duration) -> Uuid
    where
        F: Future<Output = TaskResult<u64>> + Send + 'static;

    /// Spawn cache cleanup task with completion callback
    fn spawn_cache_cleanup_task_with_callback<F>(
        &mut self,
        future: F,
        callback: TaskCallback<u64>,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<u64>> + Send + 'static;

    // =============================================================================
    // Generic Operations
    // =============================================================================

    /// Spawn generic task
    fn spawn_generic_task<F>(&mut self, future: F) -> Uuid
    where
        F: Future<Output = TaskResult<String>> + Send + 'static;

    /// Spawn generic task with custom timeout
    fn spawn_generic_task_with_timeout<F>(&mut self, future: F, timeout: Duration) -> Uuid
    where
        F: Future<Output = TaskResult<String>> + Send + 'static;

    /// Spawn generic task with completion callback
    fn spawn_generic_task_with_callback<F>(
        &mut self,
        future: F,
        callback: TaskCallback<String>,
    ) -> Uuid
    where
        F: Future<Output = TaskResult<String>> + Send + 'static;
}

/// Macro to generate task spawning implementations with zero-allocation optimizations
macro_rules! impl_task_spawner {
    (
        $task_component:ty,
        $result_type:ty,
        $task_type:expr,
        $method_base:ident,
        $method_timeout:ident,
        $method_callback:ident
    ) => {
        #[inline]
        fn $method_base<F>(&mut self, future: F) -> Uuid
        where
            F: Future<Output = TaskResult<$result_type>> + Send + 'static,
        {
            let task_pool = AsyncComputeTaskPool::get();
            let task = task_pool.spawn(future);
            let task_component = <$task_component>::new(task);
            let task_id = task_component.base.id;

            // Trigger spawned event (zero-allocation)
            self.trigger(TaskSpawnedEvent {
                id: task_id,
                operation_type: $task_type.name().to_string(),
            });

            // Spawn task component
            self.spawn((task_component, Name::new(stringify!($task_component))));

            task_id
        }

        #[inline]
        fn $method_timeout<F>(&mut self, future: F, timeout: Duration) -> Uuid
        where
            F: Future<Output = TaskResult<$result_type>> + Send + 'static,
        {
            let task_pool = AsyncComputeTaskPool::get();
            let task = task_pool.spawn(future);
            let task_component = <$task_component>::new(task).with_timeout(timeout);
            let task_id = task_component.base.id;

            // Trigger spawned event (zero-allocation)
            self.trigger(TaskSpawnedEvent {
                id: task_id,
                operation_type: $task_type.name().to_string(),
            });

            // Spawn task component with custom timeout
            self.spawn((
                task_component,
                Name::new(concat!(stringify!($task_component), "WithTimeout")),
            ));

            task_id
        }

        #[inline]
        fn $method_callback<F>(&mut self, future: F, callback: TaskCallback<$result_type>) -> Uuid
        where
            F: Future<Output = TaskResult<$result_type>> + Send + 'static,
        {
            let task_pool = AsyncComputeTaskPool::get();
            let task = task_pool.spawn(future);
            let task_component = <$task_component>::new(task).with_callback(callback);
            let task_id = task_component.base.id;

            // Trigger spawned event (zero-allocation)
            self.trigger(TaskSpawnedEvent {
                id: task_id,
                operation_type: $task_type.name().to_string(),
            });

            // Spawn task component with callback
            self.spawn((
                task_component,
                Name::new(concat!(stringify!($task_component), "WithCallback")),
            ));

            task_id
        }
    };
}

impl<'w, 's> TaskSpawner for Commands<'w, 's> {
    // Generate implementations for all task types using macro
    impl_task_spawner!(
        HotkeyPreferencesLoadTask,
        HotkeyPreferencesResult,
        TaskType::HotkeyPreferences,
        spawn_hotkey_preferences_load_task,
        spawn_hotkey_preferences_load_task_with_timeout,
        spawn_hotkey_preferences_load_task_with_callback
    );

    impl_task_spawner!(
        HotkeyPreferencesPersistTask,
        std::path::PathBuf,
        TaskType::HotkeyPreferences,
        spawn_hotkey_preferences_persist_task,
        spawn_hotkey_preferences_persist_task_with_timeout,
        spawn_hotkey_preferences_persist_task_with_callback
    );

    impl_task_spawner!(
        StorageOperationTask,
        std::path::PathBuf,
        TaskType::StorageOperation,
        spawn_storage_operation_task,
        spawn_storage_operation_task_with_timeout,
        spawn_storage_operation_task_with_callback
    );

    impl_task_spawner!(
        ClipboardOperationTask,
        String,
        TaskType::ClipboardOperation,
        spawn_clipboard_operation_task,
        spawn_clipboard_operation_task_with_timeout,
        spawn_clipboard_operation_task_with_callback
    );

    impl_task_spawner!(
        SearchOperationTask,
        Vec<String>,
        TaskType::SearchOperation,
        spawn_search_operation_task,
        spawn_search_operation_task_with_timeout,
        spawn_search_operation_task_with_callback
    );

    impl_task_spawner!(
        RaycastOperationTask,
        String,
        TaskType::RaycastOperation,
        spawn_raycast_operation_task,
        spawn_raycast_operation_task_with_timeout,
        spawn_raycast_operation_task_with_callback
    );

    impl_task_spawner!(
        TlsOperationTask,
        (),
        TaskType::TlsOperation,
        spawn_tls_operation_task,
        spawn_tls_operation_task_with_timeout,
        spawn_tls_operation_task_with_callback
    );

    impl_task_spawner!(
        CacheCleanupTask,
        u64,
        TaskType::CacheCleanup,
        spawn_cache_cleanup_task,
        spawn_cache_cleanup_task_with_timeout,
        spawn_cache_cleanup_task_with_callback
    );

    impl_task_spawner!(
        GenericTask,
        String,
        TaskType::Generic,
        spawn_generic_task,
        spawn_generic_task_with_timeout,
        spawn_generic_task_with_callback
    );
}

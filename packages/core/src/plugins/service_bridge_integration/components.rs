//! Components for Service Bridge Integration
//!
//! Proper Bevy ECS components following async_compute.rs patterns

use bevy::prelude::*;
use bevy::tasks::Task;
use bevy::time::{Timer, TimerMode};
use ecs_service_bridge::components::{Capability, PluginStatus};
use ecs_service_bridge::events::PluginMessageEvent;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use super::events::{TaskStatus, OperationStatus};

/// Component to track plugin service bridge registration
#[derive(Component)]
pub struct ServiceBridgeRegistration {
    pub plugin_id: String,
    pub channel: String,
    pub capabilities: Vec<Capability>,
    pub status: PluginStatus,
}

/// Component for async plugin message processing tasks following async_compute.rs pattern
#[derive(Component)]
pub struct PluginMessageTask {
    pub task: Task<PluginMessageEvent>,
    pub operation_id: Uuid,
    pub plugin_id: String,
    pub status: TaskStatus,
    pub created_at: SystemTime,
    pub timeout_duration: Duration,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl PluginMessageTask {
    /// Create a new plugin message task with default timeout and retry settings
    pub fn new(
        task: Task<PluginMessageEvent>,
        operation_id: Uuid,
        plugin_id: String,
    ) -> Self {
        Self {
            task,
            operation_id,
            plugin_id,
            status: TaskStatus::Pending,
            created_at: SystemTime::now(),
            timeout_duration: Duration::from_secs(30), // 30 second default timeout
            retry_count: 0,
            max_retries: 3, // Default max 3 retries
        }
    }

    /// Create a new plugin message task with custom timeout
    pub fn new_with_timeout(
        task: Task<PluginMessageEvent>,
        operation_id: Uuid,
        plugin_id: String,
        timeout_duration: Duration,
    ) -> Self {
        Self {
            task,
            operation_id,
            plugin_id,
            status: TaskStatus::Pending,
            created_at: SystemTime::now(),
            timeout_duration,
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// Check if the task has timed out
    pub fn is_timed_out(&self) -> bool {
        self.created_at.elapsed().unwrap_or(Duration::ZERO) > self.timeout_duration
    }

    /// Check if the task can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Mark task as started
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
    }

    /// Mark task as completed
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
    }

    /// Mark task as failed
    pub fn fail(&mut self) {
        self.status = TaskStatus::Failed;
    }

    /// Mark task as timed out
    pub fn timeout(&mut self) {
        self.status = TaskStatus::TimedOut;
    }

    /// Mark task as cancelled
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Component for tracking operation correlation timeouts using Bevy Timer
#[derive(Component)]
pub struct OperationTimeoutTimer {
    pub timer: Timer,
    pub operation_id: Uuid,
    pub plugin_id: String,
    pub status: OperationStatus,
}

impl OperationTimeoutTimer {
    /// Create a new operation timeout timer with 30-second duration
    pub fn new(operation_id: Uuid, plugin_id: String) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(30), TimerMode::Once),
            operation_id,
            plugin_id,
            status: OperationStatus::WaitingForResponse,
        }
    }

    /// Create a new operation timeout timer with custom duration
    pub fn new_with_duration(operation_id: Uuid, plugin_id: String, duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
            operation_id,
            plugin_id,
            status: OperationStatus::WaitingForResponse,
        }
    }

    /// Check if the timer has finished (operation timed out)
    pub fn is_timed_out(&self) -> bool {
        self.timer.finished()
    }

    /// Update operation status
    pub fn set_status(&mut self, status: OperationStatus) {
        self.status = status;
    }

    /// Mark operation as processing response
    pub fn processing_response(&mut self) {
        self.status = OperationStatus::ProcessingResponse;
    }

    /// Mark operation as completed
    pub fn complete(&mut self) {
        self.status = OperationStatus::Completed;
    }

    /// Mark operation as failed
    pub fn fail(&mut self) {
        self.status = OperationStatus::Failed;
    }

    /// Mark operation as timed out
    pub fn timeout(&mut self) {
        self.status = OperationStatus::TimedOut;
    }

    /// Mark operation as orphaned
    pub fn orphan(&mut self) {
        self.status = OperationStatus::Orphaned;
    }
}

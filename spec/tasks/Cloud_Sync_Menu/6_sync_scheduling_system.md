# Task 6: Sync Scheduling System

## Overview
Implement intelligent sync scheduling system with configurable intervals, event-driven triggers, bandwidth optimization, and smart conflict avoidance for seamless background synchronization.

## Architecture Reference
**Bevy Example**: `./docs/bevy/examples/scheduling/time_systems.rs` (lines 89-134) - Time-based system scheduling
**Bevy Example**: `./docs/bevy/examples/events/event_scheduling.rs` (lines 156-203) - Event-driven scheduling patterns

## Implementation

### File: `core/src/cloud/scheduling/mod.rs`
```rust
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::time::{Timer, TimerMode};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Resource, Clone, Debug)]
pub struct SyncScheduler {
    pub schedules: HashMap<String, SyncSchedule>,
    pub active_syncs: HashMap<String, ActiveSyncTask>,
    pub sync_queue: VecDeque<QueuedSyncRequest>,
    pub bandwidth_monitor: BandwidthMonitor,
    pub conflict_predictor: ConflictPredictor,
    pub user_activity_tracker: UserActivityTracker,
}

// Reference: ./docs/bevy/examples/scheduling/time_systems.rs lines 234-267
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct SyncSchedule {
    pub schedule_id: String,
    pub provider_id: String,
    pub schedule_type: SyncScheduleType,
    pub interval: SyncInterval,
    pub conditions: SyncConditions,
    pub priority: SyncPriority,
    pub next_run: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub retry_policy: RetryPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SyncScheduleType {
    Periodic {
        interval: Duration,
        alignment: TimeAlignment,
    },
    EventTriggered {
        triggers: Vec<SyncTrigger>,
        debounce_ms: u64,
    },
    Manual {
        require_confirmation: bool,
    },
    Smart {
        base_interval: Duration,
        adaptive_factors: AdaptiveFactors,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SyncTrigger {
    FileModified { path_patterns: Vec<String> },
    ApplicationStartup,
    UserActivity { threshold_minutes: u32 },
    NetworkConnected,
    BandwidthAvailable { min_mbps: f32 },
    TimeOfDay { hours: Vec<u8> },
    SystemIdle { idle_minutes: u32 },
}

// Reference: ./docs/bevy/examples/systems/resource_management.rs lines 178-212
#[derive(Resource, Clone, Debug)]
pub struct BandwidthMonitor {
    pub current_usage: f32, // Mbps
    pub available_bandwidth: f32, // Mbps
    pub sync_allocation: f32, // Percentage of available
    pub usage_history: VecDeque<BandwidthSample>,
    pub throttling_active: bool,
    pub network_type: NetworkType,
}

impl BandwidthMonitor {
    pub fn can_schedule_sync(&self, estimated_data_mb: f32, duration_estimate: Duration) -> bool {
        let required_bandwidth = estimated_data_mb * 8.0 / duration_estimate.num_seconds() as f32;
        let available_for_sync = self.available_bandwidth * (self.sync_allocation / 100.0);
        
        required_bandwidth <= available_for_sync && !self.is_peak_usage_period()
    }
    
    pub fn optimize_chunk_size(&self, base_chunk_size: usize) -> usize {
        match self.network_type {
            NetworkType::Wifi => base_chunk_size,
            NetworkType::Cellular => base_chunk_size / 4,
            NetworkType::Ethernet => base_chunk_size * 2,
            NetworkType::Limited => base_chunk_size / 8,
        }
    }
    
    fn is_peak_usage_period(&self) -> bool {
        let recent_samples = self.usage_history.iter().rev().take(10);
        let average_usage = recent_samples.map(|s| s.usage_mbps).sum::<f32>() / 10.0;
        average_usage > (self.available_bandwidth * 0.8)
    }
}

#[derive(Resource, Clone, Debug)]
pub struct ConflictPredictor {
    pub file_access_patterns: HashMap<String, FileAccessPattern>,
    pub user_work_schedule: UserWorkSchedule,
    pub application_usage: HashMap<String, AppUsagePattern>,
    pub predicted_conflicts: Vec<PredictedConflict>,
}

impl ConflictPredictor {
    // Reference: ./docs/bevy/examples/ai/prediction_systems.rs lines 123-156
    pub fn predict_sync_conflicts(&mut self, sync_request: &QueuedSyncRequest) -> Vec<PredictedConflict> {
        let mut conflicts = Vec::new();
        
        // Check file access patterns
        for file_path in &sync_request.files_to_sync {
            if let Some(pattern) = self.file_access_patterns.get(file_path) {
                if pattern.is_likely_accessed_at(sync_request.scheduled_time) {
                    conflicts.push(PredictedConflict {
                        conflict_type: ConflictType::FileInUse,
                        probability: pattern.access_probability,
                        affected_files: vec![file_path.clone()],
                        suggested_delay: Duration::minutes(pattern.average_session_duration),
                    });
                }
            }
        }
        
        // Check application conflicts
        for (app_name, usage_pattern) in &self.application_usage {
            if usage_pattern.conflicts_with_sync(sync_request) {
                conflicts.push(PredictedConflict {
                    conflict_type: ConflictType::ApplicationConflict,
                    probability: usage_pattern.conflict_probability,
                    affected_files: sync_request.files_to_sync.clone(),
                    suggested_delay: Duration::minutes(30),
                });
            }
        }
        
        conflicts
    }
    
    pub fn optimal_sync_window(&self, sync_request: &QueuedSyncRequest) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        let mut candidate_times = Vec::new();
        
        // Generate candidate times over next 24 hours
        for hours_ahead in 0..24 {
            let candidate_time = now + Duration::hours(hours_ahead);
            let conflicts = self.predict_sync_conflicts(&QueuedSyncRequest {
                scheduled_time: candidate_time,
                ..sync_request.clone()
            });
            
            let total_conflict_probability: f32 = conflicts.iter()
                .map(|c| c.probability)
                .sum();
            
            if total_conflict_probability < 0.2 {
                candidate_times.push((candidate_time, total_conflict_probability));
            }
        }
        
        // Return time with lowest conflict probability
        candidate_times.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidate_times.first().map(|(time, _)| *time)
    }
}

// Reference: ./docs/bevy/examples/systems/system_scheduling.rs lines 267-301
#[derive(Component, Clone, Debug)]
pub struct ActiveSyncTask {
    pub task_id: String,
    pub provider_id: String,
    pub sync_type: SyncType,
    pub status: SyncTaskStatus,
    pub progress: SyncProgress,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub bandwidth_used: f32,
    pub async_task: Option<Task<Result<SyncResult, SyncError>>>,
}

impl ActiveSyncTask {
    pub fn update_progress(&mut self, bytes_transferred: u64, total_bytes: u64, files_completed: usize, total_files: usize) {
        self.progress = SyncProgress {
            bytes_transferred,
            total_bytes,
            files_completed,
            total_files,
            percentage: ((bytes_transferred as f64 / total_bytes as f64) * 100.0) as f32,
            current_file: None,
            transfer_rate_mbps: self.calculate_transfer_rate(),
        };
        
        // Update estimated completion
        if self.progress.transfer_rate_mbps > 0.0 {
            let remaining_bytes = total_bytes - bytes_transferred;
            let remaining_seconds = (remaining_bytes as f64 / (self.progress.transfer_rate_mbps * 1_048_576.0)) as i64;
            self.estimated_completion = Some(Utc::now() + Duration::seconds(remaining_seconds));
        }
    }
    
    fn calculate_transfer_rate(&self) -> f32 {
        let elapsed = Utc::now().signed_duration_since(self.started_at);
        if elapsed.num_seconds() > 0 {
            (self.progress.bytes_transferred as f64 / elapsed.num_seconds() as f64 / 1_048_576.0) as f32
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedSyncRequest {
    pub request_id: String,
    pub provider_id: String,
    pub sync_type: SyncType,
    pub files_to_sync: Vec<String>,
    pub priority: SyncPriority,
    pub scheduled_time: DateTime<Utc>,
    pub max_duration: Option<Duration>,
    pub bandwidth_limit: Option<f32>,
    pub retry_count: u32,
    pub dependencies: Vec<String>,
}
```

### File: `core/src/cloud/scheduling/systems.rs`
```rust
use super::*;
use bevy::prelude::*;

// Reference: ./docs/bevy/examples/systems/time_systems.rs lines 334-378
pub fn sync_scheduler_system(
    mut scheduler: ResMut<SyncScheduler>,
    mut commands: Commands,
    time: Res<Time>,
    mut sync_events: EventWriter<SyncScheduledEvent>,
) {
    let now = Utc::now();
    
    // Process scheduled syncs
    let mut schedules_to_run = Vec::new();
    
    for (schedule_id, schedule) in &mut scheduler.schedules {
        if !schedule.enabled || schedule.next_run > now {
            continue;
        }
        
        // Check sync conditions
        if !scheduler.evaluate_sync_conditions(schedule) {
            // Reschedule for later
            schedule.next_run = now + Duration::minutes(5);
            continue;
        }
        
        schedules_to_run.push(schedule_id.clone());
    }
    
    // Execute eligible syncs
    for schedule_id in schedules_to_run {
        if let Some(schedule) = scheduler.schedules.get_mut(&schedule_id) {
            let sync_request = QueuedSyncRequest {
                request_id: format!("{}_{}", schedule_id, now.timestamp()),
                provider_id: schedule.provider_id.clone(),
                sync_type: determine_sync_type(schedule),
                files_to_sync: get_files_for_schedule(schedule),
                priority: schedule.priority,
                scheduled_time: now,
                max_duration: schedule.conditions.max_duration,
                bandwidth_limit: schedule.conditions.bandwidth_limit,
                retry_count: 0,
                dependencies: Vec::new(),
            };
            
            // Check for conflicts and bandwidth
            let conflicts = scheduler.conflict_predictor.predict_sync_conflicts(&sync_request);
            let bandwidth_ok = scheduler.bandwidth_monitor.can_schedule_sync(
                estimate_data_size(&sync_request), 
                schedule.conditions.max_duration.unwrap_or(Duration::hours(1))
            );
            
            if conflicts.is_empty() && bandwidth_ok {
                // Execute immediately
                scheduler.execute_sync_request(sync_request.clone());
                schedule.last_run = Some(now);
                schedule.next_run = calculate_next_run(schedule, now);
                
                sync_events.send(SyncScheduledEvent {
                    schedule_id: schedule_id.clone(),
                    request_id: sync_request.request_id,
                    scheduled_for: now,
                });
            } else {
                // Find optimal time or queue for later
                if let Some(optimal_time) = scheduler.conflict_predictor.optimal_sync_window(&sync_request) {
                    let mut delayed_request = sync_request;
                    delayed_request.scheduled_time = optimal_time;
                    scheduler.sync_queue.push_back(delayed_request);
                    
                    schedule.next_run = optimal_time;
                } else {
                    // No good window found, try again in 1 hour
                    schedule.next_run = now + Duration::hours(1);
                }
            }
        }
    }
}

// Reference: ./docs/bevy/examples/systems/async_task_management.rs lines 189-234
pub fn active_sync_monitor_system(
    mut scheduler: ResMut<SyncScheduler>,
    mut sync_completed: EventWriter<SyncCompletedEvent>,
    mut sync_failed: EventWriter<SyncFailedEvent>,
) {
    let mut completed_tasks = Vec::new();
    let mut failed_tasks = Vec::new();
    
    for (task_id, active_task) in &mut scheduler.active_syncs {
        if let Some(ref mut async_task) = active_task.async_task {
            if let Some(result) = block_on(future::poll_once(async_task)) {
                match result {
                    Ok(sync_result) => {
                        active_task.status = SyncTaskStatus::Completed;
                        sync_completed.send(SyncCompletedEvent {
                            task_id: task_id.clone(),
                            provider_id: active_task.provider_id.clone(),
                            result: sync_result,
                            duration: Utc::now().signed_duration_since(active_task.started_at),
                        });
                        completed_tasks.push(task_id.clone());
                    },
                    Err(sync_error) => {
                        active_task.status = SyncTaskStatus::Failed;
                        
                        // Check if should retry
                        let schedule = scheduler.schedules.get(&active_task.provider_id);
                        let should_retry = schedule
                            .map(|s| s.retry_policy.should_retry(&sync_error, active_task.retry_count))
                            .unwrap_or(false);
                            
                        if should_retry {
                            // Schedule retry
                            let retry_delay = calculate_retry_delay(&sync_error, active_task.retry_count);
                            let retry_request = create_retry_request(active_task, sync_error.clone(), retry_delay);
                            scheduler.sync_queue.push_back(retry_request);
                        } else {
                            sync_failed.send(SyncFailedEvent {
                                task_id: task_id.clone(),
                                provider_id: active_task.provider_id.clone(),
                                error: sync_error,
                                retry_count: active_task.retry_count,
                            });
                        }
                        failed_tasks.push(task_id.clone());
                    }
                }
            }
        }
    }
    
    // Clean up completed/failed tasks
    for task_id in completed_tasks.iter().chain(failed_tasks.iter()) {
        scheduler.active_syncs.remove(task_id);
    }
}

pub fn bandwidth_monitoring_system(
    mut bandwidth_monitor: ResMut<BandwidthMonitor>,
    time: Res<Time>,
    network_info: Res<NetworkInfo>,
) {
    // Sample current bandwidth usage
    let current_sample = BandwidthSample {
        timestamp: Utc::now(),
        usage_mbps: network_info.current_usage_mbps,
        available_mbps: network_info.total_bandwidth_mbps,
        network_type: network_info.connection_type,
    };
    
    bandwidth_monitor.usage_history.push_back(current_sample);
    
    // Keep only last 100 samples (roughly 10 minutes at 6s intervals)
    if bandwidth_monitor.usage_history.len() > 100 {
        bandwidth_monitor.usage_history.pop_front();
    }
    
    // Update current readings
    bandwidth_monitor.current_usage = current_sample.usage_mbps;
    bandwidth_monitor.available_bandwidth = current_sample.available_mbps;
    bandwidth_monitor.network_type = current_sample.network_type;
    
    // Adjust throttling if necessary
    let usage_percentage = (bandwidth_monitor.current_usage / bandwidth_monitor.available_bandwidth) * 100.0;
    bandwidth_monitor.throttling_active = usage_percentage > 85.0;
}
```

### File: `ui/src/ui/cloud/sync_scheduling.rs`
```rust
// Reference: ./docs/bevy/examples/ui/dynamic_lists.rs lines 278-323
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SyncSchedulingProps {
    pub schedules: Vec<SyncSchedule>,
    pub active_syncs: Vec<ActiveSyncTask>,
    pub bandwidth_status: BandwidthMonitor,
    pub on_schedule_create: EventHandler<SyncScheduleRequest>,
    pub on_schedule_edit: EventHandler<(String, SyncScheduleRequest)>,
    pub on_schedule_delete: EventHandler<String>,
    pub on_manual_sync: EventHandler<String>,
}

pub fn SyncSchedulingPanel(props: SyncSchedulingProps) -> Element {
    let create_modal_open = use_signal(|| false);
    let selected_schedule = use_signal(|| Option::<String>::None);
    
    rsx! {
        div {
            class: "sync-scheduling-panel",
            style: "
                display: flex;
                flex_direction: column;
                height: 100%;
                padding: 20px;
                gap: 20px;
            ",
            
            // Header with bandwidth status
            div {
                class: "panel-header",
                style: "
                    display: flex;
                    justify_content: space-between;
                    align_items: center;
                    margin-bottom: 20px;
                ",
                
                h2 { 
                    style: "margin: 0; color: #ffffff; font-size: 20px;",
                    "Sync Scheduling"
                }
                
                div {
                    class: "bandwidth-indicator",
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 12px;
                        background: rgba(30, 30, 30, 0.8);
                        padding: 8px 16px;
                        border-radius: 8px;
                        border: 1px solid rgba(70, 70, 70, 0.3);
                    ",
                    
                    div {
                        class: "bandwidth-info",
                        style: "display: flex; flex-direction: column; align-items: flex-end;",
                        
                        div {
                            style: "color: #ffffff; font-size: 12px; font-weight: 500;",
                            "Bandwidth: {format_bandwidth(props.bandwidth_status.current_usage)} / {format_bandwidth(props.bandwidth_status.available_bandwidth)}"
                        }
                        div {
                            style: "color: #888; font-size: 10px;",
                            "Network: {format_network_type(props.bandwidth_status.network_type)}"
                        }
                    }
                    
                    div {
                        class: "bandwidth-meter",
                        style: "
                            width: 60px;
                            height: 6px;
                            background: rgba(60, 60, 60, 0.5);
                            border-radius: 3px;
                            overflow: hidden;
                        ",
                        div {
                            style: "
                                height: 100%;
                                background: {if props.bandwidth_status.throttling_active { 
                                    \"linear-gradient(90deg, #FF9800 0%, #F44336 100%)\" 
                                } else { 
                                    \"linear-gradient(90deg, #4CAF50 0%, #2196F3 100%)\" 
                                }};
                                width: {(props.bandwidth_status.current_usage / props.bandwidth_status.available_bandwidth * 100.0).min(100.0)}%;
                                transition: width 0.3s ease;
                            "
                        }
                    }
                }
                
                button {
                    style: "
                        background: linear-gradient(135deg, #2196F3 0%, #1976D2 100%);
                        color: white;
                        border: none;
                        padding: 8px 16px;
                        border-radius: 6px;
                        font-size: 12px;
                        cursor: pointer;
                        transition: all 0.2s ease;
                    ",
                    onclick: move |_| create_modal_open.set(true),
                    "+ New Schedule"
                }
            }
            
            // Active syncs section
            if !props.active_syncs.is_empty() {
                div {
                    class: "active-syncs",
                    style: "margin-bottom: 20px;",
                    
                    h3 {
                        style: "color: #ffffff; font-size: 16px; margin-bottom: 12px;",
                        "Active Syncs ({props.active_syncs.len()})"
                    }
                    
                    for active_sync in props.active_syncs {
                        div {
                            key: "{active_sync.task_id}",
                            class: "active-sync-item",
                            style: "
                                background: rgba(25, 25, 25, 0.8);
                                border-radius: 8px;
                                padding: 16px;
                                margin-bottom: 12px;
                                border: 1px solid rgba(70, 70, 70, 0.3);
                            ",
                            
                            div {
                                class: "sync-header",
                                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                                
                                div {
                                    style: "display: flex; align-items: center; gap: 12px;",
                                    
                                    div {
                                        class: "provider-badge",
                                        style: "
                                            background: rgba(33, 150, 243, 0.2);
                                            color: #2196F3;
                                            padding: 2px 8px;
                                            border-radius: 4px;
                                            font-size: 10px;
                                            text-transform: uppercase;
                                        ",
                                        "{active_sync.provider_id}"
                                    }
                                    
                                    h4 {
                                        style: "margin: 0; color: #ffffff; font-size: 14px;",
                                        "{format_sync_type(active_sync.sync_type)} - {active_sync.progress.files_completed}/{active_sync.progress.total_files} files"
                                    }
                                }
                                
                                div {
                                    style: "color: #888; font-size: 12px;",
                                    "{active_sync.progress.percentage:.1}% • {format_transfer_rate(active_sync.progress.transfer_rate_mbps)}"
                                }
                            }
                            
                            // Progress bar
                            div {
                                class: "progress-container",
                                style: "
                                    background: rgba(60, 60, 60, 0.3);
                                    border-radius: 4px;
                                    height: 6px;
                                    overflow: hidden;
                                    margin-bottom: 8px;
                                ",
                                div {
                                    class: "progress-fill",
                                    style: "
                                        height: 100%;
                                        background: linear-gradient(90deg, #4CAF50 0%, #2196F3 100%);
                                        width: {active_sync.progress.percentage}%;
                                        transition: width 0.3s ease;
                                    "
                                }
                            }
                            
                            // ETA and stats
                            div {
                                class: "sync-stats",
                                style: "display: flex; justify-content: space-between; color: #888; font-size: 11px;",
                                
                                div {
                                    "Started: {format_time_ago(active_sync.started_at)}"
                                }
                                
                                if let Some(eta) = active_sync.estimated_completion {
                                    div {
                                        "ETA: {format_relative_time(eta)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Scheduled syncs list
            div {
                class: "scheduled-syncs",
                style: "flex: 1; overflow-y: auto;",
                
                h3 {
                    style: "color: #ffffff; font-size: 16px; margin-bottom: 12px;",
                    "Sync Schedules ({props.schedules.len()})"
                }
                
                for schedule in props.schedules {
                    div {
                        key: "{schedule.schedule_id}",
                        class: "schedule-item",
                        style: "
                            background: rgba(30, 30, 30, 0.6);
                            border-radius: 8px;
                            padding: 16px;
                            margin-bottom: 12px;
                            border: 1px solid rgba(70, 70, 70, 0.3);
                            cursor: pointer;
                            transition: all 0.2s ease;
                        ",
                        onclick: move |_| selected_schedule.set(Some(schedule.schedule_id.clone())),
                        
                        div {
                            class: "schedule-header",
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                            
                            div {
                                style: "display: flex; align-items: center; gap: 12px;",
                                
                                div {
                                    class: "status-indicator",
                                    style: "
                                        width: 8px;
                                        height: 8px;
                                        border-radius: 50%;
                                        background: {if schedule.enabled { \"#4CAF50\" } else { \"#666\" }};
                                    "
                                }
                                
                                div {
                                    class: "provider-badge",
                                    style: "
                                        background: rgba(33, 150, 243, 0.2);
                                        color: #2196F3;
                                        padding: 2px 8px;
                                        border-radius: 4px;
                                        font-size: 10px;
                                    ",
                                    "{schedule.provider_id}"
                                }
                                
                                h4 {
                                    style: "margin: 0; color: #ffffff; font-size: 14px;",
                                    "{format_schedule_type(schedule.schedule_type)}"
                                }
                            }
                            
                            div {
                                class: "schedule-actions",
                                style: "display: flex; gap: 8px;",
                                
                                button {
                                    style: "
                                        background: rgba(76, 175, 80, 0.8);
                                        color: white;
                                        border: none;
                                        padding: 4px 8px;
                                        border-radius: 4px;
                                        font-size: 10px;
                                        cursor: pointer;
                                    ",
                                    onclick: move |_| props.on_manual_sync.call(schedule.schedule_id.clone()),
                                    "Sync Now"
                                }
                                
                                button {
                                    style: "
                                        background: rgba(244, 67, 54, 0.8);
                                        color: white;
                                        border: none;
                                        padding: 4px 8px;
                                        border-radius: 4px;
                                        font-size: 10px;
                                        cursor: pointer;
                                    ",
                                    onclick: move |_| props.on_schedule_delete.call(schedule.schedule_id.clone()),
                                    "Delete"
                                }
                            }
                        }
                        
                        div {
                            class: "schedule-details",
                            style: "color: #888; font-size: 12px;",
                            
                            div {
                                style: "margin-bottom: 4px;",
                                "Next run: {format_relative_time(schedule.next_run)}"
                            }
                            
                            if let Some(last_run) = schedule.last_run {
                                div {
                                    "Last run: {format_time_ago(last_run)}"
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Create/edit schedule modal
        if create_modal_open() {
            SyncScheduleModal {
                is_open: create_modal_open(),
                schedule: None,
                on_close: move |_| create_modal_open.set(false),
                on_save: move |schedule_request| {
                    props.on_schedule_create.call(schedule_request);
                    create_modal_open.set(false);
                }
            }
        }
    }
}
```

## Integration Points

### Event System
- **Scheduling events**: `SyncScheduledEvent`, `SyncRescheduledEvent`, `SyncCancelledEvent`
- **Progress events**: `SyncProgressEvent`, `SyncCompletedEvent`, `SyncFailedEvent`
- **Bandwidth events**: `BandwidthThresholdExceeded`, `NetworkTypeChanged`

### Performance Optimization
- Intelligent queuing with priority-based scheduling
- Bandwidth-aware chunk size optimization
- Conflict prediction to avoid sync interruptions
- User activity tracking for optimal timing

### Smart Scheduling Features
- Adaptive interval adjustment based on file change frequency
- Network-aware scheduling (avoid cellular data usage)
- User behavior learning for optimal sync windows
- Dependency-based sync ordering

**Bevy Integration**: Reference `./docs/bevy/examples/scheduling/time_systems.rs` lines 378-423 for advanced time-based scheduling patterns
**Async Architecture**: Reference `./docs/bevy/examples/async_compute.rs` lines 289-334 for background sync task management
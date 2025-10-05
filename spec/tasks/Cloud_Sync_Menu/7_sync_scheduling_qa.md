# Task 7: Sync Scheduling System QA Validation

## Overview
Comprehensive quality assurance validation for sync scheduling system covering timing accuracy, conflict prediction, bandwidth optimization, and user experience under various network and usage scenarios.

## Architecture Reference
**Bevy Example**: `./docs/bevy/examples/testing/time_based_tests.rs` (lines 134-178) - Time-based system testing
**Bevy Example**: `./docs/bevy/examples/testing/async_scheduling_tests.rs` (lines 89-134) - Async scheduling validation

## Validation Categories

### 1. Schedule Timing Validation

#### Periodic Schedule Accuracy
```rust
// Reference: ./docs/bevy/examples/testing/time_based_tests.rs lines 201-245
#[cfg(test)]
mod schedule_timing_tests {
    use super::*;
    use bevy::prelude::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_periodic_schedule_accuracy() {
        let mut app = App::new();
        app.add_plugins(SyncSchedulerPlugin);
        
        let world = app.world_mut();
        let mut scheduler = world.resource_mut::<SyncScheduler>();
        
        // Create 5-minute periodic schedule
        let schedule = SyncSchedule {
            schedule_id: "test_periodic".to_string(),
            provider_id: "test_provider".to_string(),
            schedule_type: SyncScheduleType::Periodic {
                interval: Duration::minutes(5),
                alignment: TimeAlignment::None,
            },
            interval: SyncInterval::Fixed(Duration::minutes(5)),
            conditions: SyncConditions::default(),
            priority: SyncPriority::Normal,
            next_run: Utc::now() + Duration::seconds(10),
            last_run: None,
            enabled: true,
            retry_policy: RetryPolicy::default(),
        };
        
        scheduler.schedules.insert("test_periodic".to_string(), schedule);
        
        // Fast-forward time and validate execution timing
        let mut execution_times = Vec::new();
        let start_time = Utc::now();
        
        for _ in 0..5 {
            // Advance time by 5 minutes
            app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs(300));
            app.update();
            
            // Check if sync was triggered
            let events = app.world().resource::<Events<SyncScheduledEvent>>();
            if !events.is_empty() {
                execution_times.push(Utc::now().signed_duration_since(start_time));
            }
        }
        
        // Validate timing accuracy (should be close to 5, 10, 15, 20, 25 minutes)
        assert_eq!(execution_times.len(), 5);
        for (i, actual_time) in execution_times.iter().enumerate() {
            let expected_minutes = (i + 1) * 5;
            let expected_duration = Duration::minutes(expected_minutes as i64);
            let difference = (actual_time.num_seconds() - expected_duration.num_seconds()).abs();
            
            // Allow 10 second tolerance
            assert!(difference < 10, "Schedule timing off by {} seconds", difference);
        }
    }
    
    #[tokio::test]
    async fn test_event_triggered_schedule_debouncing() {
        let mut scheduler = SyncScheduler::new();
        
        let schedule = SyncSchedule {
            schedule_id: "test_event".to_string(),
            provider_id: "test_provider".to_string(),
            schedule_type: SyncScheduleType::EventTriggered {
                triggers: vec![SyncTrigger::FileModified { 
                    path_patterns: vec!["*.txt".to_string()] 
                }],
                debounce_ms: 5000, // 5 second debounce
            },
            // ... other fields
        };
        
        scheduler.schedules.insert("test_event".to_string(), schedule);
        
        // Simulate rapid file modifications
        let start_time = Utc::now();
        for i in 0..10 {
            scheduler.handle_file_modified_event(&format!("test{}.txt", i));
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        // Should only trigger once after debounce period
        tokio::time::sleep(Duration::from_secs(6)).await;
        
        let triggered_syncs = scheduler.get_triggered_syncs_since(start_time);
        assert_eq!(triggered_syncs.len(), 1, "Debouncing failed - expected 1 sync, got {}", triggered_syncs.len());
    }
}
```

#### Smart Schedule Adaptation
```rust
#[tokio::test]
async fn test_adaptive_interval_adjustment() {
    let mut scheduler = SyncScheduler::new();
    
    let mut schedule = SyncSchedule {
        schedule_id: "adaptive_test".to_string(),
        provider_id: "test_provider".to_string(),
        schedule_type: SyncScheduleType::Smart {
            base_interval: Duration::hours(1),
            adaptive_factors: AdaptiveFactors {
                file_change_frequency_weight: 0.4,
                user_activity_weight: 0.3,
                network_availability_weight: 0.3,
            },
        },
        // ... other fields
    };
    
    // Simulate high file activity
    for _ in 0..50 {
        scheduler.record_file_change("test_file.txt");
    }
    
    // Update schedule based on activity
    scheduler.update_adaptive_intervals();
    
    let updated_schedule = scheduler.schedules.get("adaptive_test").unwrap();
    
    // Interval should have decreased due to high activity
    if let SyncScheduleType::Smart { base_interval, .. } = &updated_schedule.schedule_type {
        let current_interval = scheduler.calculate_adaptive_interval(updated_schedule);
        assert!(current_interval < *base_interval, "Adaptive interval should decrease with high activity");
    }
}
```

### 2. Bandwidth Management Validation

#### Bandwidth-Aware Scheduling
```rust
// Reference: ./docs/bevy/examples/testing/network_tests.rs lines 267-312
#[tokio::test]
async fn test_bandwidth_aware_scheduling() {
    let mut scheduler = SyncScheduler::new();
    
    // Set low bandwidth scenario
    scheduler.bandwidth_monitor.available_bandwidth = 2.0; // 2 Mbps
    scheduler.bandwidth_monitor.sync_allocation = 50.0; // 50% for sync
    scheduler.bandwidth_monitor.current_usage = 1.5; // 1.5 Mbps in use
    
    let large_sync_request = QueuedSyncRequest {
        request_id: "large_sync".to_string(),
        provider_id: "test_provider".to_string(),
        sync_type: SyncType::Full,
        files_to_sync: vec!["large_file.bin".to_string()],
        priority: SyncPriority::Normal,
        scheduled_time: Utc::now(),
        max_duration: Some(Duration::hours(1)),
        bandwidth_limit: None,
        retry_count: 0,
        dependencies: Vec::new(),
    };
    
    // Should not schedule due to insufficient bandwidth
    let can_schedule = scheduler.bandwidth_monitor.can_schedule_sync(
        100.0, // 100MB estimated
        Duration::hours(1)
    );
    
    assert!(!can_schedule, "Should not schedule large sync with limited bandwidth");
    
    // Reduce bandwidth usage
    scheduler.bandwidth_monitor.current_usage = 0.5;
    
    let can_schedule_now = scheduler.bandwidth_monitor.can_schedule_sync(
        100.0,
        Duration::hours(1)
    );
    
    assert!(can_schedule_now, "Should schedule sync when bandwidth is available");
}

#[tokio::test]
async fn test_dynamic_chunk_size_optimization() {
    let mut bandwidth_monitor = BandwidthMonitor::new();
    
    // Test WiFi scenario
    bandwidth_monitor.network_type = NetworkType::Wifi;
    let wifi_chunk_size = bandwidth_monitor.optimize_chunk_size(1024 * 1024); // 1MB
    assert_eq!(wifi_chunk_size, 1024 * 1024, "WiFi should use full chunk size");
    
    // Test cellular scenario
    bandwidth_monitor.network_type = NetworkType::Cellular;
    let cellular_chunk_size = bandwidth_monitor.optimize_chunk_size(1024 * 1024);
    assert_eq!(cellular_chunk_size, 256 * 1024, "Cellular should reduce chunk size by 4x");
    
    // Test limited bandwidth scenario
    bandwidth_monitor.network_type = NetworkType::Limited;
    let limited_chunk_size = bandwidth_monitor.optimize_chunk_size(1024 * 1024);
    assert_eq!(limited_chunk_size, 128 * 1024, "Limited should reduce chunk size by 8x");
}
```

### 3. Conflict Prediction Validation

#### File Access Pattern Prediction
```rust
// Reference: ./docs/bevy/examples/testing/prediction_tests.rs lines 156-201
#[tokio::test]
async fn test_file_access_pattern_prediction() {
    let mut conflict_predictor = ConflictPredictor::new();
    
    // Simulate user working on file during business hours
    let work_pattern = FileAccessPattern {
        file_path: "/work/document.docx".to_string(),
        access_times: generate_business_hours_pattern(),
        average_session_duration: 120, // 2 hours
        access_probability_by_hour: create_probability_curve(),
    };
    
    conflict_predictor.file_access_patterns.insert(
        "/work/document.docx".to_string(), 
        work_pattern
    );
    
    // Test sync during predicted work hours
    let work_time_sync = QueuedSyncRequest {
        scheduled_time: Utc::now().with_hour(10).unwrap(), // 10 AM
        files_to_sync: vec!["/work/document.docx".to_string()],
        // ... other fields
    };
    
    let conflicts = conflict_predictor.predict_sync_conflicts(&work_time_sync);
    assert!(!conflicts.is_empty(), "Should predict conflicts during work hours");
    assert!(conflicts[0].probability > 0.7, "High probability conflict expected");
    
    // Test sync during off hours
    let off_hours_sync = QueuedSyncRequest {
        scheduled_time: Utc::now().with_hour(2).unwrap(), // 2 AM
        files_to_sync: vec!["/work/document.docx".to_string()],
        // ... other fields
    };
    
    let off_conflicts = conflict_predictor.predict_sync_conflicts(&off_hours_sync);
    assert!(off_conflicts.is_empty() || off_conflicts[0].probability < 0.2, 
           "Low conflict probability expected during off hours");
}

#[tokio::test]
async fn test_optimal_sync_window_calculation() {
    let mut conflict_predictor = ConflictPredictor::new();
    
    // Set up realistic user patterns
    conflict_predictor.user_work_schedule = UserWorkSchedule {
        weekday_start: 9, // 9 AM
        weekday_end: 17,  // 5 PM
        timezone: "UTC".to_string(),
        break_times: vec![12, 13], // Lunch break
    };
    
    let sync_request = QueuedSyncRequest {
        scheduled_time: Utc::now().with_hour(14).unwrap(), // 2 PM - during work
        files_to_sync: vec!["/work/active_project.txt".to_string()],
        // ... other fields
    };
    
    let optimal_time = conflict_predictor.optimal_sync_window(&sync_request);
    
    assert!(optimal_time.is_some(), "Should find an optimal sync window");
    
    let optimal_hour = optimal_time.unwrap().hour();
    
    // Should be outside work hours or during lunch
    assert!(optimal_hour < 9 || optimal_hour > 17 || (optimal_hour >= 12 && optimal_hour <= 13),
           "Optimal time should avoid work hours: found hour {}", optimal_hour);
}
```

### 4. Performance Under Load

#### Concurrent Schedule Processing
```rust
// Reference: ./docs/bevy/examples/testing/load_tests.rs lines 234-289
#[tokio::test]
async fn test_high_volume_schedule_processing() {
    let mut scheduler = SyncScheduler::new();
    
    // Create 100 schedules with various types
    for i in 0..100 {
        let schedule = SyncSchedule {
            schedule_id: format!("schedule_{}", i),
            provider_id: format!("provider_{}", i % 5), // 5 different providers
            schedule_type: match i % 3 {
                0 => SyncScheduleType::Periodic { 
                    interval: Duration::minutes(5 + i % 30), 
                    alignment: TimeAlignment::None 
                },
                1 => SyncScheduleType::EventTriggered { 
                    triggers: vec![SyncTrigger::FileModified { 
                        path_patterns: vec![format!("*.{}", i % 10)] 
                    }],
                    debounce_ms: 1000 
                },
                _ => SyncScheduleType::Smart {
                    base_interval: Duration::hours(1),
                    adaptive_factors: AdaptiveFactors::default(),
                },
            },
            next_run: Utc::now() + Duration::seconds(i as i64 % 300), // Stagger runs
            enabled: true,
            // ... other fields
        };
        
        scheduler.schedules.insert(format!("schedule_{}", i), schedule);
    }
    
    let start_time = Instant::now();
    
    // Process all schedules
    scheduler.process_all_schedules().await;
    
    let processing_time = start_time.elapsed();
    
    // Should process 100 schedules in under 1 second
    assert!(processing_time < Duration::from_secs(1), 
           "Processing 100 schedules took too long: {:?}", processing_time);
    
    // Verify no schedule processing failures
    let failed_schedules = scheduler.get_failed_schedules();
    assert!(failed_schedules.is_empty(), 
           "No schedule processing failures expected, found: {:?}", failed_schedules);
}

#[tokio::test]
async fn test_memory_usage_with_active_syncs() {
    let mut scheduler = SyncScheduler::new();
    let initial_memory = get_process_memory_usage();
    
    // Create 50 simultaneous active syncs
    for i in 0..50 {
        let active_sync = ActiveSyncTask {
            task_id: format!("sync_{}", i),
            provider_id: format!("provider_{}", i % 5),
            sync_type: SyncType::Incremental,
            status: SyncTaskStatus::Running,
            progress: SyncProgress::new(),
            started_at: Utc::now(),
            estimated_completion: Some(Utc::now() + Duration::hours(1)),
            bandwidth_used: 1.0,
            async_task: Some(create_mock_sync_task()),
        };
        
        scheduler.active_syncs.insert(format!("sync_{}", i), active_sync);
    }
    
    // Monitor memory usage over time
    let mut max_memory = initial_memory;
    for _ in 0..60 { // 1 minute
        scheduler.update_active_syncs();
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        let current_memory = get_process_memory_usage();
        max_memory = max_memory.max(current_memory);
    }
    
    // Memory usage should not exceed 2x initial memory
    assert!(max_memory < initial_memory * 2, 
           "Memory usage grew too much: {} -> {}", initial_memory, max_memory);
}
```

### 5. Error Recovery Validation

#### Network Interruption Handling
```rust
#[tokio::test]
async fn test_sync_interruption_recovery() {
    let mut scheduler = SyncScheduler::new();
    
    let sync_request = QueuedSyncRequest {
        request_id: "interruption_test".to_string(),
        provider_id: "test_provider".to_string(),
        sync_type: SyncType::Full,
        files_to_sync: vec!["large_file.bin".to_string()],
        priority: SyncPriority::Normal,
        scheduled_time: Utc::now(),
        max_duration: Some(Duration::hours(1)),
        bandwidth_limit: None,
        retry_count: 0,
        dependencies: Vec::new(),
    };
    
    // Start sync
    let task_id = scheduler.execute_sync_request(sync_request).await;
    
    // Simulate network interruption after 30 seconds
    tokio::time::sleep(Duration::from_secs(30)).await;
    scheduler.simulate_network_interruption();
    
    // Wait for recovery attempt
    tokio::time::sleep(Duration::from_secs(10)).await;
    
    // Verify sync resumed or rescheduled
    let active_sync = scheduler.active_syncs.get(&task_id);
    if let Some(sync) = active_sync {
        assert!(matches!(sync.status, SyncTaskStatus::Running | SyncTaskStatus::Retrying),
               "Sync should be running or retrying after interruption");
    } else {
        // Check if rescheduled
        let rescheduled = scheduler.sync_queue.iter()
            .any(|req| req.request_id.contains("interruption_test"));
        assert!(rescheduled, "Sync should be rescheduled if not running");
    }
}
```

## Pass Criteria

### Timing Accuracy Requirements
- [ ] Periodic schedules execute within 10 seconds of scheduled time
- [ ] Event-triggered schedules respect debounce periods correctly
- [ ] Smart schedules adapt intervals based on activity patterns
- [ ] Schedule alignment works for hourly/daily boundaries

### Bandwidth Management Requirements
- [ ] Sync scheduling respects bandwidth allocation limits
- [ ] Chunk sizes optimize based on network type
- [ ] Peak usage detection prevents sync during high usage
- [ ] Throttling activates when usage exceeds 85%

### Conflict Prediction Requirements
- [ ] File access pattern prediction accuracy > 75%
- [ ] Optimal sync window found within 24 hours 90% of time
- [ ] User work schedule patterns correctly identified
- [ ] Application conflict detection prevents interruptions

### Performance Requirements
- [ ] 100 schedules processed in <1 second
- [ ] Memory usage stable with 50 concurrent syncs
- [ ] Schedule evaluation overhead <100ms per schedule
- [ ] Conflict prediction completes in <500ms

### Recovery Requirements
- [ ] Network interruptions don't lose sync progress
- [ ] Failed syncs retry according to policy
- [ ] Schedule re-evaluation after system resume
- [ ] Graceful degradation under high load

### User Experience Requirements
- [ ] Real-time progress updates during active syncs
- [ ] Accurate ETA calculations within 10% margin
- [ ] Bandwidth indicator updates within 5 seconds
- [ ] Schedule status changes reflect immediately

## Integration Testing
```bash
# Run scheduling system tests
cargo test --test sync_scheduling_integration -- --nocapture

# Load testing with concurrent schedules
cargo test --test scheduling_load_tests -- --release --nocapture

# Network resilience testing
cargo test --test network_interruption_tests -- --nocapture

# UI responsiveness during heavy scheduling
cargo test --test ui_scheduling_responsiveness -- --nocapture
```

**Critical Validation**: All timing-dependent tests must pass consistently across multiple runs to ensure reliability in production environments.

**Performance Baseline**: Reference `./docs/bevy/examples/benchmarking/scheduling_performance.rs` lines 289-334 for scheduling system performance benchmarks.
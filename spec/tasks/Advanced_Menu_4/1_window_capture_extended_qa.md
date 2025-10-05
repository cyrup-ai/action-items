# Task 1: QA Validation - Extended Window Capture System with Automation

## Comprehensive Testing Protocol

**File**: `tests/ui/capture_automation_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: WindowCaptureExtended, ClipboardManager, FileManager, ShareSystem  

### Test Categories

#### 1. Post-Capture Automation Testing
**Reference**: `./docs/bevy/examples/window/screenshot.rs:485-518`
```rust
#[test]
fn test_post_capture_automation_workflow() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, window_capture_extended_system)
       .add_event::<CaptureCompletedEvent>()
       .add_event::<ClipboardEvent>()
       .add_event::<FileOperationEvent>()
       .add_event::<ShareEvent>();

    let automation_settings = AutomationSettings {
        copy_to_clipboard: true,
        show_in_finder: true,
        auto_share_enabled: false,
        default_share_destinations: Vec::new(),
        post_capture_actions: vec![
            PostCaptureAction::CopyToClipboard,
            PostCaptureAction::ShowInFinder,
        ],
        notification_settings: NotificationSettings::default(),
    };

    let mut capture_extended = WindowCaptureExtended {
        basic_capture: WindowCaptureManager::default(),
        automation_settings,
        clipboard_integration: ClipboardIntegration::default(),
        file_management: CaptureFileManager::default(),
        batch_operations: BatchCaptureManager::default(),
    };
    
    app.world_mut().insert_resource(capture_extended);

    // Simulate capture completion
    let mut capture_events = app.world_mut().resource_mut::<Events<CaptureCompletedEvent>>();
    capture_events.send(CaptureCompletedEvent {
        capture_id: "test_capture_001".to_string(),
        capture_data: CaptureData {
            id: "test_capture_001".to_string(),
            image_data: vec![0x89, 0x50, 0x4E, 0x47], // PNG header
            format: ImageFormat::PNG,
            file_path: Some(PathBuf::from("/tmp/test_capture.png")),
            dimensions: (800, 600),
            metadata: CaptureMetadata::default(),
        },
        file_path: Some(PathBuf::from("/tmp/test_capture.png")),
        metadata: CaptureMetadata::default(),
    });

    app.update();

    // Verify clipboard event was generated
    let clipboard_events = app.world().resource::<Events<ClipboardEvent>>();
    let mut clipboard_reader = clipboard_events.get_reader();
    let clipboard_events_vec: Vec<_> = clipboard_reader.read(&clipboard_events).collect();
    
    assert!(!clipboard_events_vec.is_empty(), "Clipboard copy should be triggered");
    assert!(matches!(clipboard_events_vec[0], ClipboardEvent::CopyImage { .. }));

    // Verify file operation event was generated
    let file_events = app.world().resource::<Events<FileOperationEvent>>();
    let mut file_reader = file_events.get_reader();
    let file_events_vec: Vec<_> = file_reader.read(&file_events).collect();
    
    assert!(!file_events_vec.is_empty(), "Show in Finder should be triggered");
    assert!(matches!(file_events_vec[0], FileOperationEvent::RevealInFinder { .. }));
}
```

#### 2. Clipboard Integration Testing
```rust
#[test]
fn test_clipboard_integration_formats() {
    let clipboard_formats = vec![
        ClipboardFormat::Image,
        ClipboardFormat::FilePath,
        ClipboardFormat::Both,
    ];

    let test_capture_data = CaptureData {
        id: "clipboard_test".to_string(),
        image_data: create_test_png_data(),
        format: ImageFormat::PNG,
        file_path: Some(PathBuf::from("/tmp/test.png")),
        dimensions: (100, 100),
        metadata: CaptureMetadata::default(),
    };

    for format in clipboard_formats {
        let clipboard_integration = ClipboardIntegration {
            clipboard_format: format,
            quality_settings: ClipboardQuality::High,
            metadata_inclusion: true,
            history_enabled: true,
            clipboard_history: VecDeque::new(),
        };

        let mut clipboard_events = Events::<ClipboardEvent>::default();
        let mut clipboard_writer = clipboard_events.get_writer();

        execute_clipboard_copy(
            &test_capture_data,
            &mut clipboard_writer,
            &clipboard_integration,
        );

        let events: Vec<_> = clipboard_events.get_reader().read(&clipboard_events).collect();

        match format {
            ClipboardFormat::Image => {
                assert_eq!(events.len(), 1);
                assert!(matches!(events[0], ClipboardEvent::CopyImage { .. }));
            }
            ClipboardFormat::FilePath => {
                assert_eq!(events.len(), 1);
                assert!(matches!(events[0], ClipboardEvent::CopyText { .. }));
            }
            ClipboardFormat::Both => {
                assert_eq!(events.len(), 2);
                assert!(events.iter().any(|e| matches!(e, ClipboardEvent::CopyImage { .. })));
                assert!(events.iter().any(|e| matches!(e, ClipboardEvent::CopyText { .. })));
            }
        }
    }
}
```

#### 3. Batch Capture System Testing
**Reference**: `./docs/bevy/examples/async_tasks/async_compute.rs:385-422`
```rust
#[test]
fn test_batch_capture_operations() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, batch_capture_system)
       .add_event::<BatchCaptureEvent>()
       .add_event::<CaptureRequestEvent>()
       .add_event::<FileOperationEvent>();

    let batch_settings = BatchSettings {
        interval_ms: 500,
        max_captures: 5,
        capture_targets: vec![
            CaptureTarget::ApplicationWindow,
            CaptureTarget::ActiveWindow,
        ],
        output_format: BatchOutputFormat::Individual,
        naming_convention: NamingConvention::Timestamp,
    };

    let mut batch_manager = BatchCaptureManager {
        batch_settings: batch_settings.clone(),
        active_batches: HashMap::new(),
        batch_history: VecDeque::new(),
        scheduling_enabled: true,
    };
    
    app.world_mut().insert_resource(batch_manager);

    // Start batch capture
    let mut batch_events = app.world_mut().resource_mut::<Events<BatchCaptureEvent>>();
    batch_events.send(BatchCaptureEvent::StartBatch {
        settings: batch_settings.clone(),
        targets: vec![
            CaptureTarget::ApplicationWindow,
            CaptureTarget::ActiveWindow,
            CaptureTarget::FullScreen,
        ],
    });

    app.update();

    // Verify batch operation was created
    let batch_manager = app.world().resource::<BatchCaptureManager>();
    assert_eq!(batch_manager.active_batches.len(), 1);

    let (batch_id, operation) = batch_manager.active_batches.iter().next().unwrap();
    assert_eq!(operation.targets.len(), 3);
    assert_eq!(operation.status, BatchStatus::Running);

    // Verify capture requests were generated
    let capture_events = app.world().resource::<Events<CaptureRequestEvent>>();
    let mut capture_reader = capture_events.get_reader();
    let capture_events_vec: Vec<_> = capture_reader.read(&capture_events).collect();
    
    assert_eq!(capture_events_vec.len(), 3, "Should generate capture request for each target");
    
    for event in &capture_events_vec {
        if let CaptureRequestEvent::ScheduledCapture { batch_id: Some(bid), .. } = event {
            assert_eq!(bid, batch_id);
        }
    }
}
```

#### 4. Collage Creation Testing
```rust
#[test]
fn test_collage_creation_from_batch() {
    let captures = vec![
        create_test_capture_data(100, 100, "capture1"),
        create_test_capture_data(150, 150, "capture2"),
        create_test_capture_data(120, 80, "capture3"),
    ];

    let collage_settings = CollageSettings {
        layout: CollageLayout::Grid { rows: 2, cols: 2 },
        spacing: 10,
        background_color: Color::WHITE,
        resize_mode: ResizeMode::Fit,
        output_size: Some((800, 600)),
    };

    let result = create_collage_from_captures(&captures, &collage_settings);
    
    assert!(result.is_ok(), "Collage creation should succeed with valid inputs");
    
    let collage_data = result.unwrap();
    assert!(!collage_data.is_empty(), "Collage data should not be empty");
    
    // Verify PNG header
    assert_eq!(&collage_data[0..4], &[0x89, 0x50, 0x4E, 0x47], "Should produce valid PNG");
    
    // Verify collage is larger than individual captures
    assert!(collage_data.len() > captures[0].image_data.len(), 
        "Collage should be larger than individual captures");
}
```

#### 5. Automation Error Handling Testing
**Reference**: `./docs/bevy/examples/error_handling/error_handling.rs:125-158`
```rust
#[test]
fn test_automation_error_handling() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, window_capture_extended_system)
       .add_event::<CaptureCompletedEvent>()
       .add_event::<NotificationEvent>();

    // Setup with invalid automation settings
    let automation_settings = AutomationSettings {
        copy_to_clipboard: true,
        show_in_finder: true,
        auto_share_enabled: true,
        default_share_destinations: vec![
            ShareDestination::InvalidService("nonexistent://service".to_string()),
        ],
        post_capture_actions: vec![
            PostCaptureAction::RunScript("/nonexistent/script.sh".to_string()),
            PostCaptureAction::ShareTo(ShareDestination::InvalidService("bad://url".to_string())),
        ],
        notification_settings: NotificationSettings { success_enabled: true, error_enabled: true },
    };

    let mut capture_extended = WindowCaptureExtended {
        automation_settings,
        ..Default::default()
    };
    
    app.world_mut().insert_resource(capture_extended);

    // Send capture completion with problematic data
    let mut capture_events = app.world_mut().resource_mut::<Events<CaptureCompletedEvent>>();
    capture_events.send(CaptureCompletedEvent {
        capture_id: "error_test".to_string(),
        capture_data: CaptureData {
            id: "error_test".to_string(),
            image_data: vec![], // Empty image data to trigger errors
            format: ImageFormat::PNG,
            file_path: None, // No file path to trigger errors
            dimensions: (0, 0),
            metadata: CaptureMetadata::default(),
        },
        file_path: None,
        metadata: CaptureMetadata::default(),
    });

    app.update();

    // Verify error notifications were generated
    let notification_events = app.world().resource::<Events<NotificationEvent>>();
    let mut notification_reader = notification_events.get_reader();
    let notifications: Vec<_> = notification_reader.read(&notification_events).collect();
    
    assert!(!notifications.is_empty(), "Should generate error notifications");
    
    let has_error_notification = notifications.iter().any(|notif| {
        matches!(notif, NotificationEvent::CaptureAutomationError { .. })
    });
    
    assert!(has_error_notification, "Should include automation error notification");
}
```

#### 6. Clipboard History Management Testing
```rust
#[test]
fn test_clipboard_history_management() {
    let mut clipboard_integration = ClipboardIntegration {
        clipboard_format: ClipboardFormat::Both,
        quality_settings: ClipboardQuality::High,
        metadata_inclusion: true,
        history_enabled: true,
        clipboard_history: VecDeque::new(),
    };

    // Add entries to history
    for i in 0..10 {
        let entry = ClipboardEntry {
            id: format!("entry_{}", i),
            timestamp: chrono::Utc::now() - chrono::Duration::minutes(i),
            capture_id: format!("capture_{}", i),
            format: ClipboardFormat::Image,
            preview_data: vec![i as u8; 100], // Mock preview data
        };
        
        clipboard_integration.clipboard_history.push_back(entry);
    }

    assert_eq!(clipboard_integration.clipboard_history.len(), 10);

    // Test history cleanup
    cleanup_clipboard_history(&mut clipboard_integration);
    
    // Should maintain reasonable history size (implementation dependent)
    assert!(clipboard_integration.clipboard_history.len() <= 50, 
        "History should be limited to reasonable size");

    // Verify most recent entries are preserved
    let newest_entry = clipboard_integration.clipboard_history.back().unwrap();
    assert_eq!(newest_entry.capture_id, "capture_0"); // Most recent
    
    // Test history retrieval
    let recent_entries = get_recent_clipboard_entries(&clipboard_integration, 5);
    assert!(recent_entries.len() <= 5);
    assert!(recent_entries.iter().all(|entry| {
        clipboard_integration.clipboard_history.iter().any(|h| h.id == entry.id)
    }));
}
```

### Edge Case Testing

#### 7. Concurrent Automation Operations Testing
```rust
#[test]
fn test_concurrent_automation_operations() {
    let mut capture_extended = WindowCaptureExtended::default();
    capture_extended.automation_settings.copy_to_clipboard = true;
    capture_extended.automation_settings.show_in_finder = true;

    // Simulate multiple concurrent capture completions
    let concurrent_captures = (0..5).map(|i| {
        CaptureCompletedEvent {
            capture_id: format!("concurrent_capture_{}", i),
            capture_data: create_test_capture_data(100, 100, &format!("test_{}", i)),
            file_path: Some(PathBuf::from(format!("/tmp/capture_{}.png", i))),
            metadata: CaptureMetadata::default(),
        }
    }).collect::<Vec<_>>();

    // Process all captures concurrently (in practice would be handled by system)
    let results: Vec<_> = concurrent_captures.into_iter()
        .map(|capture| {
            process_capture_automation(
                &capture,
                &capture_extended.automation_settings,
                &mut capture_extended.clipboard_integration,
                &capture_extended.file_management,
            )
        })
        .collect();

    // All operations should complete successfully
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Concurrent automation {} should succeed", i);
        
        let actions = result.as_ref().unwrap();
        assert!(actions.contains(&PostCaptureAction::CopyToClipboard));
        assert!(actions.contains(&PostCaptureAction::ShowInFinder));
    }

    // Verify no resource conflicts occurred
    assert_eq!(capture_extended.clipboard_integration.clipboard_history.len(), 5);
}
```

#### 8. Performance Stress Testing
```rust
#[test]
fn test_automation_performance_stress() {
    let automation_settings = AutomationSettings {
        copy_to_clipboard: true,
        show_in_finder: true,
        auto_share_enabled: false,
        default_share_destinations: Vec::new(),
        post_capture_actions: vec![
            PostCaptureAction::CopyToClipboard,
            PostCaptureAction::ShowInFinder,
        ],
        notification_settings: NotificationSettings::default(),
    };

    let start_time = std::time::Instant::now();
    
    // Process many automation requests
    for i in 0..100 {
        let capture_event = CaptureCompletedEvent {
            capture_id: format!("stress_test_{}", i),
            capture_data: create_test_capture_data(50, 50, &format!("stress_{}", i)),
            file_path: Some(PathBuf::from(format!("/tmp/stress_{}.png", i))),
            metadata: CaptureMetadata::default(),
        };
        
        let mut clipboard_integration = ClipboardIntegration::default();
        let file_management = CaptureFileManager::default();
        
        let result = process_capture_automation(
            &capture_event,
            &automation_settings,
            &mut clipboard_integration,
            &file_management,
        );
        
        assert!(result.is_ok(), "Stress test iteration {} should succeed", i);
    }
    
    let duration = start_time.elapsed();
    assert!(duration.as_millis() < 5000, 
        "100 automation operations should complete within 5 seconds, took {}ms", 
        duration.as_millis());
}
```

### Manual Testing Checklist

- [ ] Copy to Clipboard checkbox correctly enables/disables clipboard automation
- [ ] Show in Finder checkbox correctly enables/disables file revelation
- [ ] Record Hotkey button successfully registers capture hotkeys
- [ ] Batch capture creates multiple screenshots with correct timing
- [ ] Collage mode combines multiple captures into single image
- [ ] Clipboard history tracks all automated clipboard operations
- [ ] Error notifications appear for failed automation actions
- [ ] Performance remains responsive during batch operations
- [ ] Post-capture scripts execute with correct file paths
- [ ] Share integration works with configured destinations

**Bevy Examples**: `./docs/bevy/examples/window/screenshot.rs:525-562`, `./docs/bevy/examples/async_tasks/async_compute.rs:425-462`  
**Integration Points**: All extended capture system components  
**Success Criteria**: All tests pass, sub-2s automation completion, reliable clipboard integration
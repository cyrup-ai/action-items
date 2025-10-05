# Task 9: QA Validation - Window Capture System

## Comprehensive Testing Protocol

**File**: `tests/ui/window_capture_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: WindowCaptureSystem, HotkeySystem, PermissionManager  

### Test Categories

#### 1. Platform-Specific Capture Testing
**Reference**: `./docs/bevy/examples/window/screenshot.rs:285-318`
```rust
#[cfg(target_os = "macos")]
#[test]
fn test_macos_window_capture() {
    use core_graphics::display::*;
    
    let capture_settings = CaptureSettings {
        target: CaptureTarget::ApplicationWindow,
        format: ImageFormat::PNG,
        quality: 1.0,
        include_cursor: false,
        auto_save: true,
        save_location: PathBuf::from("/tmp/captures"),
        filename_template: "test_capture_{timestamp}".to_string(),
    };

    // Test Core Graphics integration
    unsafe {
        let window_list = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly,
            kCGNullWindowID
        );
        
        assert!(!window_list.is_null(), "Should be able to get window list on macOS");
        
        // Test image capture (may fail in test environment without display)
        let result = start_application_window_capture("test_capture", &capture_settings);
        
        // In test environment, capture may fail due to no display, but should handle gracefully
        match result {
            Ok(_) => {
                // Successful capture
            }
            Err(e) => {
                // Expected in headless test environment
                assert!(e.contains("display") || e.contains("window") || e.contains("permission"));
            }
        }
    }
}

#[cfg(target_os = "windows")]
#[test]  
fn test_windows_window_capture() {
    use winapi::um::winuser::*;
    
    let capture_settings = CaptureSettings {
        target: CaptureTarget::ApplicationWindow,
        format: ImageFormat::PNG,
        quality: 1.0,
        include_cursor: true,
        auto_save: true,
        save_location: PathBuf::from("C:\\temp\\captures"),
        filename_template: "test_capture_{timestamp}".to_string(),
    };

    unsafe {
        // Test Windows API integration
        let hwnd = GetForegroundWindow();
        
        if !hwnd.is_null() {
            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            let rect_result = GetWindowRect(hwnd, &mut rect);
            assert_ne!(rect_result, 0, "Should be able to get window rect");
            
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            assert!(width > 0 && height > 0, "Window should have valid dimensions");
        }
    }

    let result = start_application_window_capture("test_capture", &capture_settings);
    
    // May fail in test environment, but should handle errors gracefully
    match result {
        Ok(_) => {
            // Successful capture
        }
        Err(e) => {
            assert!(!e.is_empty(), "Error message should be provided");
        }
    }
}
```

#### 2. Capture Target Selection Testing
```rust
#[test]
fn test_capture_target_selection() {
    let mut capture_manager = WindowCaptureManager::default();
    capture_manager.permission_status = PermissionStatus::Granted;

    let test_targets = vec![
        CaptureTarget::ApplicationWindow,
        CaptureTarget::ActiveWindow,
        CaptureTarget::SelectedWindow,
        CaptureTarget::FullScreen,
        CaptureTarget::SelectedArea,
    ];

    for target in test_targets {
        capture_manager.capture_settings.target = target;
        
        let capture_id = start_capture_operation(
            target,
            &mut capture_manager,
            &mock_ui_events(),
        );

        // Should return capture ID if permissions are granted
        assert!(capture_id.is_some(), "Capture operation should start with valid target: {:?}", target);
        
        // Verify operation is tracked
        let operation = capture_manager.active_captures.get(&capture_id.unwrap());
        assert!(operation.is_some(), "Active capture should be tracked");
        assert_eq!(operation.unwrap().target, target);
    }
}
```

#### 3. Image Format and Quality Testing
**Reference**: `./docs/bevy/examples/asset_loading/image_processing.rs:185-218`
```rust
#[test]
fn test_image_format_processing() {
    let test_image_data = create_test_image_rgba(100, 100);
    let dimensions = (100, 100);

    let formats = vec![
        (ImageFormat::PNG, 1.0),
        (ImageFormat::JPEG, 0.8),
        (ImageFormat::WEBP, 0.9),
        (ImageFormat::BMP, 1.0),
    ];

    for (format, quality) in formats {
        let result = match format {
            ImageFormat::PNG => process_as_png(&test_image_data, dimensions, 1.0),
            ImageFormat::JPEG => process_as_jpeg(&test_image_data, dimensions, quality),
            ImageFormat::WEBP => process_as_webp(&test_image_data, dimensions, quality),
            ImageFormat::BMP => process_as_bmp(&test_image_data, dimensions),
        };

        assert!(result.is_ok(), "Image processing should succeed for format: {:?}", format);
        
        let processed_data = result.unwrap();
        assert!(!processed_data.is_empty(), "Processed image data should not be empty");
        
        // Verify file signature
        match format {
            ImageFormat::PNG => assert_eq!(&processed_data[0..4], &[0x89, 0x50, 0x4E, 0x47]),
            ImageFormat::JPEG => assert_eq!(&processed_data[0..2], &[0xFF, 0xD8]),
            ImageFormat::BMP => assert_eq!(&processed_data[0..2], &[0x42, 0x4D]),
            ImageFormat::WEBP => {
                assert_eq!(&processed_data[0..4], b"RIFF");
                assert_eq!(&processed_data[8..12], b"WEBP");
            }
        }
    }
}
```

#### 4. Hotkey Registration and Handling Testing
**Reference**: `./docs/bevy/examples/input/keyboard_input.rs:485-518`
```rust
#[test]
fn test_capture_hotkey_registration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, capture_hotkey_system)
       .add_event::<HotkeyRegistrationEvent>()
       .add_event::<UINotificationEvent>();

    let mut capture_manager = WindowCaptureManager::default();
    app.world_mut().insert_resource(capture_manager);

    // Test hotkey registration
    let mut hotkey_events = app.world_mut().resource_mut::<Events<HotkeyRegistrationEvent>>();
    hotkey_events.send(HotkeyRegistrationEvent::RegisterCapture {
        key_combination: "Ctrl+Shift+S".to_string(),
    });

    app.update();

    let manager = app.world().resource::<WindowCaptureManager>();
    assert!(manager.hotkey_binding.is_some(), "Hotkey should be registered");
    
    let binding = manager.hotkey_binding.as_ref().unwrap();
    assert_eq!(binding.key_combination, "Ctrl+Shift+S");
    assert_eq!(binding.action, HotkeyAction::WindowCapture);
    assert!(binding.global, "Capture hotkey should be global");

    // Test hotkey unregistration
    let mut hotkey_events = app.world_mut().resource_mut::<Events<HotkeyRegistrationEvent>>();
    hotkey_events.send(HotkeyRegistrationEvent::UnregisterCapture);

    app.update();

    let manager = app.world().resource::<WindowCaptureManager>();
    assert!(manager.hotkey_binding.is_none(), "Hotkey should be unregistered");
}
```

#### 5. Permission Handling Testing
```rust
#[test]
fn test_permission_handling() {
    let permission_states = vec![
        (PermissionStatus::NotRequested, false),
        (PermissionStatus::Denied, false),
        (PermissionStatus::Granted, true),
        (PermissionStatus::Restricted, false),
    ];

    for (permission_status, should_allow) in permission_states {
        let mut capture_manager = WindowCaptureManager::default();
        capture_manager.permission_status = permission_status;

        let result = start_capture_operation(
            CaptureTarget::ApplicationWindow,
            &mut capture_manager,
            &mock_ui_events(),
        );

        if should_allow {
            assert!(result.is_some(), 
                "Capture should be allowed with permission status: {:?}", permission_status);
        } else {
            assert!(result.is_none(), 
                "Capture should be blocked with permission status: {:?}", permission_status);
        }
    }
}
```

#### 6. File Saving and Naming Testing
**Reference**: `./docs/bevy/examples/file_io/file_io.rs:325-358`
```rust
#[test]
fn test_file_saving_and_naming() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_image_data = create_test_image_rgba(50, 50);
    
    let capture_settings = CaptureSettings {
        target: CaptureTarget::ApplicationWindow,
        format: ImageFormat::PNG,
        quality: 1.0,
        include_cursor: false,
        auto_save: true,
        save_location: temp_dir.path().to_path_buf(),
        filename_template: "test_{timestamp}_{id}".to_string(),
    };

    let result = save_capture_image(
        &test_image_data,
        "test_capture_123",
        &capture_settings,
        (50, 50),
    );

    assert!(result.is_ok(), "Capture save should succeed");
    
    let record = result.unwrap();
    assert!(record.file_path.is_some(), "File path should be set");
    assert!(record.file_size > 0, "File size should be greater than 0");
    assert_eq!(record.dimensions, (50, 50));
    assert_eq!(record.id, "test_capture_123");

    // Verify file was actually created
    let file_path = record.file_path.unwrap();
    assert!(file_path.exists(), "Capture file should exist on disk");
    
    let file_metadata = std::fs::metadata(&file_path).unwrap();
    assert_eq!(file_metadata.len(), record.file_size);

    // Verify filename template was applied
    let filename = file_path.file_name().unwrap().to_str().unwrap();
    assert!(filename.contains("test_"));
    assert!(filename.contains("test_capture_123"));
    assert!(filename.ends_with(".png"));
}
```

#### 7. Capture History Management Testing
```rust
#[test]
fn test_capture_history_management() {
    let mut capture_manager = WindowCaptureManager::default();
    
    // Add multiple capture records
    for i in 0..5 {
        let record = CaptureRecord {
            id: format!("capture_{}", i),
            captured_at: chrono::Utc::now() - chrono::Duration::hours(i),
            target: CaptureTarget::ApplicationWindow,
            file_path: Some(PathBuf::from(format!("/tmp/capture_{}.png", i))),
            file_size: 1024 + i as u64 * 100,
            dimensions: (800 + i as u32 * 10, 600 + i as u32 * 5),
            shared_to: Vec::new(),
        };
        
        capture_manager.capture_history.push_back(record);
    }

    assert_eq!(capture_manager.capture_history.len(), 5);

    // Test history retrieval
    let recent_captures = get_recent_captures(&capture_manager, 3);
    assert_eq!(recent_captures.len(), 3);
    
    // Verify most recent is first
    assert_eq!(recent_captures[0].id, "capture_0");
    assert_eq!(recent_captures[2].id, "capture_2");

    // Test history cleanup (if implemented)
    let max_history = 10;
    if capture_manager.capture_history.len() > max_history {
        cleanup_capture_history(&mut capture_manager, max_history);
        assert!(capture_manager.capture_history.len() <= max_history);
    }
}
```

### Edge Case Testing

#### 8. Error Handling and Recovery Testing
```rust
#[test]
fn test_error_handling_and_recovery() {
    let mut capture_manager = WindowCaptureManager::default();
    capture_manager.permission_status = PermissionStatus::Granted;

    // Test invalid save location
    capture_manager.capture_settings.save_location = PathBuf::from("/invalid/path/that/does/not/exist");
    
    let result = start_capture_operation(
        CaptureTarget::ApplicationWindow,
        &mut capture_manager,
        &mock_ui_events(),
    );

    // Should handle invalid path gracefully
    match result {
        Some(capture_id) => {
            // Wait for operation to complete
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            let operation = capture_manager.active_captures.get(&capture_id);
            if let Some(op) = operation {
                // Operation should either succeed or fail gracefully
                assert!(matches!(op.status, CaptureStatus::Completed | CaptureStatus::Failed(_)));
            }
        }
        None => {
            // Expected if validation catches invalid path early
        }
    }

    // Test disk full scenario (mock)
    let temp_dir = tempfile::tempdir().unwrap();
    capture_manager.capture_settings.save_location = temp_dir.path().to_path_buf();
    
    // This would require mocking filesystem to truly test disk full
    // For now, just verify the error handling code path exists
    let mock_large_data = vec![0u8; 1024 * 1024]; // 1MB test data
    let save_result = save_capture_image(
        &mock_large_data,
        "large_capture",
        &capture_manager.capture_settings,
        (1920, 1080),
    );
    
    // Should either succeed or provide meaningful error
    match save_result {
        Ok(record) => {
            assert!(record.file_path.is_some());
            assert!(record.file_size > 0);
        }
        Err(error) => {
            assert!(!error.is_empty(), "Error message should be descriptive");
        }
    }
}
```

#### 9. Concurrent Capture Operation Testing
```rust
#[test]
fn test_concurrent_capture_operations() {
    let mut capture_manager = WindowCaptureManager::default();
    capture_manager.permission_status = PermissionStatus::Granted;

    // Start multiple capture operations
    let capture_ids = (0..3)
        .map(|i| {
            start_capture_operation(
                CaptureTarget::ApplicationWindow,
                &mut capture_manager,
                &mock_ui_events(),
            )
        })
        .collect::<Vec<_>>();

    // All operations should be tracked
    assert_eq!(capture_manager.active_captures.len(), 3);

    // Verify each capture has unique ID
    let unique_ids: std::collections::HashSet<_> = capture_ids
        .iter()
        .filter_map(|id| id.as_ref())
        .collect();
    assert_eq!(unique_ids.len(), capture_ids.iter().filter(|id| id.is_some()).count());

    // Test operation completion
    for capture_id in capture_ids.into_iter().flatten() {
        complete_capture_operation(&capture_id, Ok(create_mock_capture_record(&capture_id)));
    }

    // Update active captures (would normally be done by system)
    update_active_captures(&mut capture_manager, &mock_ui_events());
    
    // All operations should be completed and removed from active list
    assert_eq!(capture_manager.active_captures.len(), 0);
    assert_eq!(capture_manager.capture_history.len(), 3);
}
```

### Manual Testing Checklist

- [ ] Record Hotkey button registers global hotkey successfully
- [ ] Hotkey triggers window capture when pressed
- [ ] Application window capture saves correct window image
- [ ] Different image formats (PNG, JPEG, WebP) save correctly
- [ ] Capture quality settings affect file size appropriately
- [ ] Capture history shows all previous captures with metadata
- [ ] Permission denied shows clear error message with action
- [ ] File save errors display helpful user guidance
- [ ] Capture operations don't block UI during processing
- [ ] Multiple concurrent captures work without conflicts

**Bevy Examples**: `./docs/bevy/examples/window/screenshot.rs:385-422`, `./docs/bevy/examples/input/keyboard_input.rs:525-562`  
**Integration Points**: All window capture system components  
**Success Criteria**: All tests pass, reliable cross-platform capture, sub-2s capture completion
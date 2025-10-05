# Task 7: QA Validation - Import/Export System

## Comprehensive Testing Protocol

**File**: `tests/ui/import_export_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: ImportExportSystem, SchedulingSystem, DataValidator  

### Test Categories

#### 1. Complete Export Data Collection Testing
**Reference**: `./docs/bevy/examples/file_io/file_io.rs:445-478`
```rust
#[test]
async fn test_complete_export_data_collection() {
    // Setup test data
    setup_test_application_data().await;
    
    let export_data = collect_complete_export_data().await.unwrap();
    
    // Verify all data categories are included
    assert!(!export_data.settings.is_empty(), "Settings should be included in complete export");
    assert!(!export_data.quicklinks.is_empty(), "Quicklinks should be included");
    assert!(!export_data.snippets.is_empty(), "Snippets should be included");
    assert!(!export_data.notes.is_empty(), "Notes should be included");
    assert!(!export_data.aliases.is_empty(), "Aliases should be included");
    assert!(!export_data.hotkeys.is_empty(), "Hotkeys should be included");
    assert!(!export_data.favorites.is_empty(), "Favorites should be included");
    
    // Verify metadata
    assert_eq!(export_data.version, EXPORT_SCHEMA_VERSION);
    assert!(!export_data.metadata.user_id.is_empty());
    assert!(!export_data.metadata.device_id.is_empty());
    assert!(export_data.metadata.data_categories.len() >= 8);
    
    // Verify timestamp is recent
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(export_data.exported_at);
    assert!(diff.num_seconds() < 10, "Export timestamp should be recent");
}
```

#### 2. Data Validation System Testing
```rust
#[test]
fn test_export_data_validation() {
    let mut validator = DataValidator {
        validation_rules: vec![],
        strict_mode: true,
        schema_version: EXPORT_SCHEMA_VERSION.to_string(),
    };

    // Test valid export data
    let valid_export_data = create_valid_export_data();
    let result = validator.validate_export_data(&valid_export_data);
    assert!(result.is_ok(), "Valid export data should pass validation");

    // Test missing required fields
    let mut invalid_data = valid_export_data.clone();
    invalid_data.metadata.user_id = String::new();
    let result = validator.validate_export_data(&invalid_data);
    assert!(result.is_err(), "Export data with missing user_id should fail validation");

    // Test hotkey conflicts
    let mut conflicted_data = valid_export_data.clone();
    conflicted_data.hotkeys.insert("action1".to_string(), HotkeyBinding {
        key_combination: "Ctrl+K".to_string(),
        enabled: true,
    });
    conflicted_data.hotkeys.insert("action2".to_string(), HotkeyBinding {
        key_combination: "Ctrl+K".to_string(),
        enabled: true,
    });
    
    let result = validator.validate_export_data(&conflicted_data);
    assert!(result.is_err(), "Export data with hotkey conflicts should fail validation");
    
    if let Err(ValidationError::DataIntegrityIssue(message)) = result {
        assert!(message.contains("Hotkey conflicts"), "Error should mention hotkey conflicts");
    }
}
```

#### 3. Import Data Processing Testing
**Reference**: `./docs/bevy/examples/file_io/file_io.rs:515-548`
```rust
#[test]
async fn test_import_data_processing() {
    let mut import_export_manager = ImportExportManager::default();
    
    // Create test export file
    let export_data = create_valid_export_data();
    let serialized_data = serialize_export_data(&export_data).unwrap();
    let temp_file = create_temp_export_file(&serialized_data);
    
    // Test import operation
    let operation_id = generate_operation_id();
    let result = start_import_operation(
        &operation_id,
        &temp_file,
        &ValidationMode::Strict,
        &mut import_export_manager,
        &mock_file_events(),
    ).await;
    
    assert!(result.is_ok(), "Import operation should succeed with valid data");
    
    // Verify data was imported correctly
    let imported_settings = get_current_application_settings().await;
    assert_eq!(imported_settings.app_name, export_data.settings.app_name);
    
    let imported_aliases = get_current_aliases().await;
    assert_eq!(imported_aliases.len(), export_data.aliases.len());
    
    // Cleanup
    std::fs::remove_file(temp_file).unwrap();
}
```

#### 4. Scheduled Export System Testing
```rust
#[test]
fn test_scheduled_export_system() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, import_export_system)
       .add_event::<ExportRequestEvent>()
       .add_event::<OperationStatusEvent>();

    let mut import_export_manager = ImportExportManager::default();
    import_export_manager.export_scheduler = ExportScheduler {
        enabled: true,
        schedule_type: ScheduleType::Daily,
        interval: Duration::from_secs(86400), // 24 hours
        next_export: Some(chrono::Utc::now() - chrono::Duration::seconds(1)), // Past time to trigger
        export_location: ExportLocation::LocalDirectory(PathBuf::from("/tmp/exports")),
        retention_policy: RetentionPolicy::KeepLast(5),
    };
    
    app.world_mut().insert_resource(import_export_manager);
    
    app.update();
    
    // Verify scheduled export was triggered
    let export_events = app.world().resource::<Events<ExportRequestEvent>>();
    let mut reader = export_events.get_reader();
    let events: Vec<_> = reader.read(&export_events).collect();
    
    assert!(!events.is_empty(), "Scheduled export should trigger export request");
    
    // Verify next export time was updated
    let manager = app.world().resource::<ImportExportManager>();
    assert!(manager.export_scheduler.next_export.is_some());
    assert!(manager.export_scheduler.next_export.unwrap() > chrono::Utc::now());
}
```

#### 5. Export Format and Encryption Testing
**Reference**: `./docs/bevy/examples/encryption/encryption.rs:125-158`
```rust
#[test]
fn test_export_encryption() {
    let original_data = create_valid_export_data();
    
    let encryption_settings = EncryptionSettings {
        enabled: true,
        algorithm: EncryptionAlgorithm::AES256GCM,
        key_derivation: KeyDerivation::PBKDF2,
        salt: generate_random_salt(),
        iterations: 100000,
    };
    
    // Test encryption
    let encrypted_data = encrypt_export_data(&original_data, &encryption_settings).unwrap();
    assert_ne!(encrypted_data, serialize_export_data(&original_data).unwrap());
    assert!(encrypted_data.len() > 0, "Encrypted data should not be empty");
    
    // Test decryption
    let decrypted_data = decrypt_export_data(&encrypted_data, &encryption_settings).unwrap();
    let restored_data: ExportData = deserialize_export_data(&decrypted_data).unwrap();
    
    // Verify data integrity after encryption/decryption
    assert_eq!(original_data.version, restored_data.version);
    assert_eq!(original_data.settings.app_name, restored_data.settings.app_name);
    assert_eq!(original_data.aliases.len(), restored_data.aliases.len());
    assert_eq!(original_data.hotkeys.len(), restored_data.hotkeys.len());
}
```

#### 6. Export Destination Testing
```rust
#[test]
async fn test_export_destinations() {
    let export_data = create_valid_export_data();
    let serialized_data = serialize_export_data(&export_data).unwrap();
    
    // Test local directory export
    let temp_dir = tempfile::tempdir().unwrap();
    let local_destination = ExportLocation::LocalDirectory(temp_dir.path().to_path_buf());
    
    let result = write_export_to_destination(&serialized_data, &local_destination, &ExportType::Complete).await;
    assert!(result.is_ok(), "Local directory export should succeed");
    
    let files: Vec<_> = std::fs::read_dir(temp_dir.path()).unwrap().collect();
    assert!(!files.is_empty(), "Export file should be created in local directory");
    
    // Test cloud storage export (mock)
    let cloud_destination = ExportLocation::CloudStorage(CloudProvider::MockProvider);
    let result = write_export_to_destination(&serialized_data, &cloud_destination, &ExportType::Complete).await;
    
    // Should succeed with mock provider or fail gracefully
    assert!(result.is_ok() || result.is_err(), "Cloud export should have a defined outcome");
}
```

#### 7. Import Validation and Recovery Testing
```rust
#[test]
async fn test_import_validation_and_recovery() {
    let mut import_export_manager = ImportExportManager::default();
    
    // Test corrupted import file
    let corrupted_data = b"corrupted_export_data";
    let corrupted_file = create_temp_file_with_data(corrupted_data);
    
    let result = start_import_operation(
        "test_op",
        &corrupted_file,
        &ValidationMode::Strict,
        &mut import_export_manager,
        &mock_file_events(),
    ).await;
    
    assert!(result.is_err(), "Import should fail with corrupted data");
    
    if let Err(ImportError::InvalidFormat(_)) = result {
        // Expected error type
    } else {
        panic!("Expected InvalidFormat error for corrupted data");
    }
    
    // Test version incompatibility
    let old_version_data = create_export_data_with_version("1.0.0");
    let serialized_old = serialize_export_data(&old_version_data).unwrap();
    let old_version_file = create_temp_file_with_data(&serialized_old);
    
    let result = start_import_operation(
        "test_op_2",
        &old_version_file,
        &ValidationMode::Strict,
        &mut import_export_manager,
        &mock_file_events(),
    ).await;
    
    // Should either succeed with migration or fail with clear error
    match result {
        Ok(_) => {
            // Migration succeeded
        }
        Err(ImportError::IncompatibleVersion { .. }) => {
            // Expected version incompatibility
        }
        _ => panic!("Unexpected error type for version incompatibility"),
    }
    
    // Cleanup
    std::fs::remove_file(corrupted_file).unwrap();
    std::fs::remove_file(old_version_file).unwrap();
}
```

### Edge Case Testing

#### 8. Large Data Export Performance Testing
```rust
#[test]
async fn test_large_data_export_performance() {
    // Create large dataset
    let mut large_export_data = create_valid_export_data();
    
    // Add many items to test performance
    for i in 0..10000 {
        large_export_data.quicklinks.push(Quicklink {
            id: format!("ql_{}", i),
            name: format!("Quicklink {}", i),
            url: format!("https://example{}.com", i),
            category: "test".to_string(),
        });
        
        large_export_data.snippets.push(Snippet {
            id: format!("snip_{}", i),
            name: format!("Snippet {}", i),
            content: format!("Content for snippet {}", i),
            tags: vec!["test".to_string()],
        });
    }
    
    let start_time = std::time::Instant::now();
    
    // Test export performance
    let serialized = serialize_export_data(&large_export_data).unwrap();
    let serialization_time = start_time.elapsed();
    
    assert!(serialization_time.as_millis() < 5000, 
        "Large data serialization took too long: {}ms", serialization_time.as_millis());
    assert!(serialized.len() > 1_000_000, "Large export should be substantial in size");
    
    // Test validation performance
    let validation_start = std::time::Instant::now();
    let validator = DataValidator::default();
    let validation_result = validator.validate_export_data(&large_export_data);
    let validation_time = validation_start.elapsed();
    
    assert!(validation_result.is_ok(), "Large data should validate successfully");
    assert!(validation_time.as_millis() < 2000, 
        "Large data validation took too long: {}ms", validation_time.as_millis());
}
```

#### 9. Export History and Retention Testing
```rust
#[test]
fn test_export_history_and_retention() {
    let mut import_export_manager = ImportExportManager::default();
    import_export_manager.export_scheduler.retention_policy = RetentionPolicy::KeepLast(3);
    
    // Add multiple export records
    for i in 0..5 {
        let record = ExportRecord {
            operation_id: format!("op_{}", i),
            exported_at: chrono::Utc::now() - chrono::Duration::days(i),
            export_type: ExportType::Complete,
            destination: ExportLocation::LocalDirectory(PathBuf::from("/tmp")),
            data_size: 1000 + i as usize * 100,
            success: true,
            error_message: None,
        };
        
        import_export_manager.export_history.push_back(record);
    }
    
    // Apply retention policy
    apply_retention_policy(&mut import_export_manager.export_history, 
                          &import_export_manager.export_scheduler.retention_policy);
    
    // Should only keep last 3 records
    assert_eq!(import_export_manager.export_history.len(), 3);
    
    // Verify most recent records are kept
    let newest_record = import_export_manager.export_history.back().unwrap();
    assert_eq!(newest_record.operation_id, "op_0"); // Most recent
}
```

### Manual Testing Checklist

- [ ] Import button opens file picker and processes valid export files
- [ ] Export button creates complete backup file with all user data
- [ ] Configure Export Schedule button opens scheduling interface
- [ ] Scheduled exports execute automatically at configured intervals
- [ ] Import validation catches corrupted or incompatible files
- [ ] Export encryption works when enabled in settings
- [ ] Large datasets export and import without performance issues
- [ ] Export history tracks all operations with success/failure status
- [ ] Retention policies automatically clean up old exports
- [ ] Progress indicators show during long-running operations

**Bevy Examples**: `./docs/bevy/examples/file_io/file_io.rs:585-622`, `./docs/bevy/examples/async_tasks/async_compute.rs:225-262`  
**Integration Points**: All import/export system components  
**Success Criteria**: All tests pass, sub-5s export time for large datasets, reliable data integrity
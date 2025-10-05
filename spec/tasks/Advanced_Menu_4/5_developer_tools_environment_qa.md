# Task 5: QA Validation - Developer Tools Environment Control System

## Comprehensive Testing Protocol

**File**: `tests/ui/developer_environment_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: DeveloperEnvironment, ProcessManager, LoggingSystem, NodeRuntime  

### Test Categories

#### 1. Node.js Environment Configuration Testing
**Reference**: `./docs/bevy/examples/process/process_management.rs:425-458`
```rust
#[test]
fn test_node_environment_configuration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, developer_environment_system)
       .add_event::<DeveloperEnvironmentEvent>()
       .add_event::<ProcessEvent>()
       .add_event::<UINotificationEvent>();

    let mut dev_environment = DeveloperEnvironment::default();
    dev_environment.node_environment.use_production_env = false;
    
    app.world_mut().insert_resource(dev_environment);

    // Test switching to production environment
    let mut env_events = app.world_mut().resource_mut::<Events<DeveloperEnvironmentEvent>>();
    env_events.send(DeveloperEnvironmentEvent::SetNodeEnvironment {
        use_production: true,
    });

    app.update();

    // Verify environment was updated
    let environment = app.world().resource::<DeveloperEnvironment>();
    assert!(environment.node_environment.use_production_env, 
        "Node environment should be set to production");

    // Verify process events were generated for environment variables
    let process_events = app.world().resource::<Events<ProcessEvent>>();
    let mut process_reader = process_events.get_reader();
    let events: Vec<_> = process_reader.read(&process_events).collect();
    
    assert!(!events.is_empty(), "Environment change should generate process events");
    
    // Check for NODE_ENV setting
    let has_node_env = events.iter().any(|event| {
        if let ProcessEvent::SetEnvironmentVariable { key, value, .. } = event {
            key == "NODE_ENV" && value == "production"
        } else {
            false
        }
    });
    
    assert!(has_node_env, "Should set NODE_ENV to production");

    // Verify success notification
    let ui_events = app.world().resource::<Events<UINotificationEvent>>();
    let mut ui_reader = ui_events.get_reader();
    let ui_events_vec: Vec<_> = ui_reader.read(&ui_events).collect();
    
    let has_success = ui_events_vec.iter().any(|event| {
        matches!(event, UINotificationEvent::Success { .. })
    });
    
    assert!(has_success, "Should generate success notification");
}
```

#### 2. Node.js Runtime Detection Testing
```rust
#[test]
fn test_node_runtime_detection() {
    let mut node_env = NodeEnvironment::default();
    
    // Test Node.js detection
    let result = detect_node_runtime_info(&mut node_env);
    
    // May succeed or fail depending on test environment
    match result {
        Ok(_) => {
            assert!(node_env.node_version.is_some(), "Node version should be detected");
            
            // Verify version format
            let version = node_env.node_version.as_ref().unwrap();
            assert!(version.starts_with('v') || version.chars().next().unwrap().is_ascii_digit(),
                "Version should start with 'v' or digit: {}", version);
        }
        Err(EnvironmentError::NodeNotFound) => {
            // Expected in environments without Node.js
            assert!(node_env.node_version.is_none());
        }
        Err(other) => {
            panic!("Unexpected error during Node detection: {:?}", other);
        }
    }

    // Test package manager detection
    let package_manager_result = detect_package_manager();
    assert!(package_manager_result.is_ok(), "Package manager detection should not fail");
    
    // Should default to npm if no lock files present
    let package_manager = package_manager_result.unwrap();
    assert!(matches!(package_manager, 
        PackageManager::Npm | PackageManager::Yarn | PackageManager::Pnpm | PackageManager::Bun
    ));
}
```

#### 3. Environment Variable Management Testing
**Reference**: `./docs/bevy/examples/environment/env_vars.rs:125-158`
```rust
#[test]
fn test_environment_variable_management() {
    let test_cases = vec![
        // (use_production, expected_node_env, expected_debug)
        (true, "production", ""),
        (false, "development", "*"),
    ];

    for (use_production, expected_node_env, expected_debug) in test_cases {
        let mut node_env = NodeEnvironment::default();
        let process_events = Events::<ProcessEvent>::default();
        let mut process_writer = process_events.get_writer();

        let result = configure_node_environment(
            &mut node_env,
            use_production,
            &process_writer,
        );

        assert!(result.is_ok(), "Environment configuration should succeed");
        
        let changes = result.unwrap();
        
        // Verify NODE_ENV setting
        assert_eq!(changes.get("NODE_ENV"), Some(&expected_node_env.to_string()),
            "NODE_ENV should be set to {}", expected_node_env);
        
        // Verify DEBUG setting
        assert_eq!(changes.get("DEBUG"), Some(&expected_debug.to_string()),
            "DEBUG should be set to '{}'", expected_debug);

        // Verify environment variables were updated in node_env
        assert_eq!(node_env.environment_variables.get("NODE_ENV"), 
                   Some(&expected_node_env.to_string()));

        // Verify use_production_env flag
        assert_eq!(node_env.use_production_env, use_production);
    }
}
```

#### 4. Logging Configuration Testing
**Reference**: `./docs/bevy/examples/logging/logging.rs:385-422`
```rust
#[test]
fn test_logging_configuration() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    let mut logging_config = LoggingConfiguration {
        use_file_logging: false,
        use_os_log: true,
        log_level: LogLevel::Debug,
        output_directory: temp_dir.path().to_path_buf(),
        file_rotation: FileRotationConfig {
            max_file_size_mb: 10,
            max_files: 5,
            rotation_frequency: RotationFrequency::Daily,
            compress_rotated: true,
            cleanup_older_than_days: Some(7),
        },
        structured_logging: true,
        performance_logging: false,
    };

    let logging_events = Events::<LoggingEvent>::default();
    let mut logging_writer = logging_events.get_writer();

    // Test switching to file logging
    let result = configure_file_logging(&mut logging_config, true, &logging_writer);
    assert!(result.is_ok(), "File logging configuration should succeed");
    
    assert!(logging_config.use_file_logging, "Should enable file logging");
    assert!(!logging_config.use_os_log, "Should disable OS logging");

    // Verify log directory was created
    assert!(temp_dir.path().exists(), "Log directory should be created");
    assert!(temp_dir.path().is_dir(), "Log path should be a directory");

    // Test switching back to OS logging
    let result = configure_file_logging(&mut logging_config, false, &logging_writer);
    assert!(result.is_ok(), "OS logging configuration should succeed");
    
    assert!(!logging_config.use_file_logging, "Should disable file logging");
    assert!(logging_config.use_os_log, "Should enable OS logging");
}
```

#### 5. File Logging System Testing
```rust
#[test]
fn test_file_logging_system() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    let logging_config = LoggingConfiguration {
        use_file_logging: true,
        use_os_log: false,
        log_level: LogLevel::Info,
        output_directory: temp_dir.path().to_path_buf(),
        file_rotation: FileRotationConfig {
            max_file_size_mb: 1, // Small size for testing
            max_files: 3,
            rotation_frequency: RotationFrequency::Never,
            compress_rotated: false,
            cleanup_older_than_days: None,
        },
        structured_logging: false,
        performance_logging: true,
    };

    // Initialize file logger
    let result = initialize_file_logger(&logging_config);
    
    // May fail in test environment without proper setup
    match result {
        Ok(_) => {
            // Test logging functionality
            log::info!("Test log message for file logging");
            log::warn!("Test warning message");
            log::error!("Test error message");

            // Give logger time to flush
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Check for log files
            let entries: Vec<_> = std::fs::read_dir(&temp_dir).unwrap().collect();
            let log_files: Vec<_> = entries.iter()
                .filter_map(|entry| entry.as_ref().ok())
                .filter(|entry| {
                    entry.path().extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "log")
                        .unwrap_or(false)
                })
                .collect();

            if !log_files.is_empty() {
                // Verify log file content
                let log_file_path = &log_files[0].path();
                let log_content = std::fs::read_to_string(log_file_path).unwrap();
                
                assert!(log_content.contains("Test log message"), 
                    "Log file should contain test messages");
            }
        }
        Err(LoggingError::SubscriberSetupFailed(_)) => {
            // Expected if global subscriber is already set
        }
        Err(other) => {
            panic!("Unexpected logging error: {:?}", other);
        }
    }
}
```

#### 6. Environment Validation Testing
```rust
#[test]
fn test_environment_validation() {
    // Test with valid environment
    let valid_environment = DeveloperEnvironment {
        node_environment: NodeEnvironment {
            use_production_env: true,
            node_version: Some("v18.17.0".to_string()),
            npm_version: Some("9.6.7".to_string()),
            environment_variables: {
                let mut vars = HashMap::new();
                vars.insert("NODE_ENV".to_string(), "production".to_string());
                vars
            },
            node_modules_path: Some(PathBuf::from("/usr/local/lib/node_modules")),
            package_manager: PackageManager::Npm,
            execution_context: ExecutionContext::Development,
        },
        logging_configuration: LoggingConfiguration::default(),
        runtime_settings: RuntimeSettings::default(),
        development_state: DevelopmentState::default(),
        process_monitoring: ProcessMonitoring::default(),
        security_settings: DevelopmentSecurity::default(),
    };

    let validation_result = validate_development_environment(&valid_environment);
    assert!(validation_result.is_ok(), "Valid environment should pass validation");
    
    let report = validation_result.unwrap();
    assert!(report.checks_passed > 0, "Should pass at least some validation checks");

    // Test with invalid environment (old Node version)
    let mut invalid_environment = valid_environment.clone();
    invalid_environment.node_environment.node_version = Some("v14.0.0".to_string());
    
    let validation_result = validate_development_environment(&invalid_environment);
    assert!(validation_result.is_err(), "Invalid environment should fail validation");
    
    if let Err(issues) = validation_result {
        assert!(!issues.is_empty(), "Should report validation issues");
        
        let has_version_issue = issues.iter().any(|issue| {
            issue.contains("version") || issue.contains("old")
        });
        
        assert!(has_version_issue, "Should report Node version issue");
    }
}
```

#### 7. Process Monitoring Testing
**Reference**: `./docs/bevy/examples/system_info/system_monitor.rs:185-218`
```rust
#[test]
fn test_process_monitoring() {
    let mut process_monitoring = ProcessMonitoring {
        active_processes: HashMap::new(),
        resource_usage: ResourceUsage::default(),
        performance_metrics: PerformanceMetrics::default(),
        error_tracking: ErrorTracking::default(),
    };

    // Add test processes
    let test_process = ProcessInfo {
        pid: 12345,
        command: "node".to_string(),
        arguments: vec!["index.js".to_string()],
        working_directory: PathBuf::from("/tmp"),
        environment: HashMap::new(),
        started_at: Instant::now(),
        status: ProcessStatus::Running,
        resource_usage: ProcessResourceUsage {
            cpu_percent: 15.5,
            memory_mb: 128.0,
            handles: 45,
        },
    };

    process_monitoring.active_processes.insert("node_process".to_string(), test_process);

    // Test resource usage update
    update_resource_usage(&mut process_monitoring);
    
    // Verify resource usage was updated
    assert!(process_monitoring.resource_usage.cpu_usage_percent >= 0.0);
    assert!(process_monitoring.resource_usage.memory_usage_percent >= 0.0);

    // Test process health check
    let ui_events = Events::<UINotificationEvent>::default();
    let mut ui_writer = ui_events.get_writer();
    
    check_process_health(&process_monitoring, &ui_writer);
    
    // Should complete without panic
    assert_eq!(process_monitoring.active_processes.len(), 1);
}
```

### Edge Case Testing

#### 8. Node.js Version Parsing Testing
```rust
#[test]
fn test_node_version_parsing() {
    let test_cases = vec![
        // (version_string, expected_result)
        ("v18.17.0", Ok((18, 17, 0))),
        ("v16.20.1", Ok((16, 20, 1))),
        ("v20.5.0", Ok((20, 5, 0))),
        ("18.17.0", Ok((18, 17, 0))), // Without 'v' prefix
        ("invalid", Err(())),
        ("v", Err(())),
        ("", Err(())),
    ];

    for (version_string, expected) in test_cases {
        let result = parse_node_version(version_string);
        
        match (result, expected) {
            (Ok(actual), Ok(expected)) => {
                assert_eq!(actual, expected, 
                    "Version parsing failed for: {}", version_string);
            }
            (Err(_), Err(_)) => {
                // Both are errors - this is expected
            }
            (actual, expected) => {
                panic!("Unexpected result for '{}': got {:?}, expected {:?}", 
                       version_string, actual, expected);
            }
        }
    }
}
```

#### 9. Package Manager Detection Edge Cases
```rust
#[test]
fn test_package_manager_detection_edge_cases() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();
    
    // Change to temp directory
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Test with no lock files (should default to npm)
    let result = detect_package_manager();
    assert_eq!(result.unwrap(), PackageManager::Npm);
    
    // Test with yarn lock file
    std::fs::write("yarn.lock", "").unwrap();
    let result = detect_package_manager();
    assert_eq!(result.unwrap(), PackageManager::Yarn);
    
    // Test with multiple lock files (yarn should take precedence)
    std::fs::write("package-lock.json", "{}").unwrap();
    let result = detect_package_manager();
    assert_eq!(result.unwrap(), PackageManager::Yarn);
    
    // Remove yarn.lock, should detect npm
    std::fs::remove_file("yarn.lock").unwrap();
    let result = detect_package_manager();
    assert_eq!(result.unwrap(), PackageManager::Npm);
    
    // Test with pnpm lock file
    std::fs::remove_file("package-lock.json").unwrap();
    std::fs::write("pnpm-lock.yaml", "").unwrap();
    let result = detect_package_manager();
    assert_eq!(result.unwrap(), PackageManager::Pnpm);
    
    // Test with bun lock file
    std::fs::remove_file("pnpm-lock.yaml").unwrap();
    std::fs::write("bun.lockb", "").unwrap();
    let result = detect_package_manager();
    assert_eq!(result.unwrap(), PackageManager::Bun);
    
    // Restore original working directory
    std::env::set_current_dir(original_cwd).unwrap();
}
```

#### 10. Log Rotation and Cleanup Testing
```rust
#[test]
fn test_log_rotation_and_cleanup() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    let rotation_config = FileRotationConfig {
        max_file_size_mb: 1,
        max_files: 3,
        rotation_frequency: RotationFrequency::Daily,
        compress_rotated: false,
        cleanup_older_than_days: Some(1),
    };

    // Test rotation setup
    let result = setup_log_rotation(&rotation_config);
    assert!(result.is_ok(), "Log rotation setup should succeed");

    // Create test log files with different timestamps
    for i in 0..5 {
        let log_file_name = format!("raycast-dev.{}.log", i);
        let log_file_path = temp_dir.path().join(log_file_name);
        
        std::fs::write(&log_file_path, format!("Log content {}", i)).unwrap();
        
        // Set file modification time to simulate old files
        if i < 2 {
            let old_time = std::time::SystemTime::now() - std::time::Duration::from_secs(86400 * 2); // 2 days ago
            filetime::set_file_mtime(&log_file_path, filetime::FileTime::from_system_time(old_time)).unwrap();
        }
    }

    // Test cleanup
    let result = cleanup_old_log_files(&temp_dir.path(), &rotation_config);
    assert!(result.is_ok(), "Log cleanup should succeed");

    // Count remaining files
    let remaining_files: Vec<_> = std::fs::read_dir(&temp_dir).unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "log")
                .unwrap_or(false)
        })
        .collect();

    // Should have removed old files while keeping max_files limit
    assert!(remaining_files.len() <= rotation_config.max_files,
        "Should not exceed max files limit after cleanup");
}
```

### Manual Testing Checklist

- [ ] Node production environment checkbox correctly switches NODE_ENV
- [ ] File logging checkbox correctly switches between file and OS logging
- [ ] Environment validation button reports Node.js version and package manager
- [ ] Runtime information panel shows correct Node.js and npm versions
- [ ] Log files are created in specified directory when file logging is enabled
- [ ] Log rotation works when files reach size limit
- [ ] Development processes are monitored and resource usage is tracked
- [ ] Error notifications appear when Node.js or npm are not installed
- [ ] Package manager detection works for different project types
- [ ] Log level changes affect what messages are written to files

**Bevy Examples**: `./docs/bevy/examples/process/process_management.rs:525-562`, `./docs/bevy/examples/logging/logging.rs:485-522`  
**Integration Points**: All developer environment system components  
**Success Criteria**: All tests pass, reliable Node.js detection, proper logging configuration
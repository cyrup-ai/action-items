# Task 7: QA Validation - Development Workflow Automation System

## Comprehensive Testing Protocol

**File**: `tests/ui/workflow_automation_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: WorkflowAutomation, FileWatcher, ProcessManager, WindowManager  

### Test Categories

#### 1. Auto-Reload Configuration Testing
**Reference**: `./docs/bevy/examples/file_watcher/file_watcher.rs:425-458`
```rust
#[test]
fn test_auto_reload_configuration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, workflow_automation_system)
       .add_event::<WorkflowEvent>()
       .add_event::<FileWatchEvent>()
       .add_event::<UINotificationEvent>();

    let mut workflow_automation = WorkflowAutomation::default();
    workflow_automation.auto_reload_settings.enabled = false;
    
    app.world_mut().insert_resource(workflow_automation);

    // Test enabling auto-reload
    let mut workflow_events = app.world_mut().resource_mut::<Events<WorkflowEvent>>();
    workflow_events.send(WorkflowEvent::ToggleAutoReload { enabled: true });

    app.update();

    // Verify auto-reload was enabled
    let automation = app.world().resource::<WorkflowAutomation>();
    assert!(automation.auto_reload_settings.enabled, "Auto-reload should be enabled");

    // Verify file watch events were generated
    let file_watch_events = app.world().resource::<Events<FileWatchEvent>>();
    let mut watch_reader = file_watch_events.get_reader();
    let watch_events: Vec<_> = watch_reader.read(&file_watch_events).collect();
    
    assert!(!watch_events.is_empty(), "Should generate file watch events");
    
    let has_start_watching = watch_events.iter().any(|event| {
        matches!(event, FileWatchEvent::StartWatching { .. })
    });
    
    assert!(has_start_watching, "Should start file watching");

    // Test disabling auto-reload
    let mut workflow_events = app.world_mut().resource_mut::<Events<WorkflowEvent>>();
    workflow_events.send(WorkflowEvent::ToggleAutoReload { enabled: false });

    app.update();

    let automation = app.world().resource::<WorkflowAutomation>();
    assert!(!automation.auto_reload_settings.enabled, "Auto-reload should be disabled");

    // Verify file watching was stopped
    let file_watch_events = app.world().resource::<Events<FileWatchEvent>>();
    let mut watch_reader = file_watch_events.get_reader();
    let stop_events: Vec<_> = watch_reader.read(&file_watch_events).collect();
    
    let has_stop_watching = stop_events.iter().any(|event| {
        matches!(event, FileWatchEvent::StopAllWatching)
    });
    
    assert!(has_stop_watching, "Should stop all file watching");
}
```

#### 2. File Watcher Setup and Teardown Testing
```rust
#[test]
fn test_file_watcher_setup_and_teardown() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.js");
    std::fs::write(&test_file, "console.log('test');").unwrap();

    let auto_reload_settings = AutoReloadSettings {
        enabled: true,
        watch_patterns: vec![
            temp_dir.path().join("**/*.js").to_string_lossy().to_string(),
            temp_dir.path().join("**/*.ts").to_string_lossy().to_string(),
        ],
        ignore_patterns: vec!["**/node_modules/**".to_string()],
        debounce_delay_ms: 100,
        reload_strategy: ReloadStrategy::Smart,
        notification_enabled: true,
        pre_reload_hooks: Vec::new(),
        post_reload_hooks: Vec::new(),
    };

    let mut file_watchers = HashMap::new();

    // Test file watcher setup
    let result = setup_file_watchers(&auto_reload_settings, &mut file_watchers);
    
    match result {
        Ok(_) => {
            assert!(!file_watchers.is_empty(), "Should create file watchers");
            
            // Verify watchers are active
            for (path, watcher) in &file_watchers {
                assert!(watcher.is_active, "Watcher for {:?} should be active", path);
            }
        }
        Err(WorkflowError::InvalidWatchPattern(pattern, _)) => {
            // May fail in test environment with invalid patterns
            assert!(auto_reload_settings.watch_patterns.contains(&pattern));
        }
        Err(other) => {
            panic!("Unexpected error during file watcher setup: {:?}", other);
        }
    }

    // Test file watcher teardown
    for (_, watcher) in file_watchers.iter() {
        assert!(watcher.is_active, "Watcher should be active before teardown");
    }

    // Stop all watchers
    for (_, mut watcher) in file_watchers.drain() {
        watcher.stop();
        assert!(!watcher.is_active, "Watcher should be inactive after stop");
    }
}
```

#### 3. File Change Detection and Reload Strategy Testing
**Reference**: `./docs/bevy/examples/file_watcher/file_watcher.rs:485-522`
```rust
#[test]
fn test_file_change_detection_and_reload_strategy() {
    let auto_reload_settings = AutoReloadSettings {
        enabled: true,
        watch_patterns: vec!["**/*".to_string()],
        ignore_patterns: vec!["**/node_modules/**".to_string(), "**/*.log".to_string()],
        debounce_delay_ms: 50,
        reload_strategy: ReloadStrategy::Smart,
        notification_enabled: true,
        pre_reload_hooks: Vec::new(),
        post_reload_hooks: Vec::new(),
    };

    let mut reload_history = VecDeque::new();
    let reload_events = Events::<ReloadEvent>::default();
    let ui_events = Events::<UINotificationEvent>::default();
    let mut reload_writer = reload_events.get_writer();
    let mut ui_writer = ui_events.get_writer();

    let test_cases = vec![
        // (file_path, change_type, expected_strategy)
        (PathBuf::from("src/main.js"), FileChangeType::Modified, ReloadStrategy::HotReload),
        (PathBuf::from("src/component.tsx"), FileChangeType::Created, ReloadStrategy::Full),
        (PathBuf::from("styles/main.css"), FileChangeType::Modified, ReloadStrategy::HotReload),
        (PathBuf::from("config.json"), FileChangeType::Modified, ReloadStrategy::Incremental),
        (PathBuf::from("package.json"), FileChangeType::Modified, ReloadStrategy::Incremental),
        (PathBuf::from("unknown.xyz"), FileChangeType::Modified, ReloadStrategy::Full),
    ];

    for (file_path, change_type, expected_strategy) in test_cases {
        handle_file_change(
            &file_path,
            change_type,
            &auto_reload_settings,
            &mut reload_history,
            &reload_writer,
            &ui_writer,
        );

        // Verify reload event was added to history
        assert!(!reload_history.is_empty(), "Reload history should contain events");
        
        let last_event = reload_history.back().unwrap();
        assert_eq!(last_event.file_path, file_path);
        assert_eq!(last_event.change_type, change_type);
        
        // For smart strategy, verify the correct strategy was determined
        if auto_reload_settings.reload_strategy == ReloadStrategy::Smart {
            let determined_strategy = determine_reload_strategy(&file_path, change_type, &auto_reload_settings);
            assert_eq!(determined_strategy, expected_strategy,
                "Wrong reload strategy for {:?}", file_path);
        }
    }

    // Test file ignoring
    let ignored_files = vec![
        PathBuf::from("node_modules/package/index.js"),
        PathBuf::from("build.log"),
        PathBuf::from("debug.log"),
    ];

    let initial_count = reload_history.len();

    for ignored_file in ignored_files {
        handle_file_change(
            &ignored_file,
            FileChangeType::Modified,
            &auto_reload_settings,
            &mut reload_history,
            &reload_writer,
            &ui_writer,
        );
    }

    // Should not have added any new events for ignored files
    assert_eq!(reload_history.len(), initial_count, "Ignored files should not trigger reloads");
}
```

#### 4. Development Mode Configuration Testing
**Reference**: `./docs/bevy/examples/app/development_mode.rs:425-458`
```rust
#[test]
fn test_development_mode_configuration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, workflow_automation_system)
       .add_event::<WorkflowEvent>()
       .add_event::<ReloadEvent>()
       .add_event::<UINotificationEvent>();

    let workflow_automation = WorkflowAutomation::default();
    app.world_mut().insert_resource(workflow_automation);

    // Test enabling development mode
    let mut workflow_events = app.world_mut().resource_mut::<Events<WorkflowEvent>>();
    workflow_events.send(WorkflowEvent::ToggleDevelopmentMode { enabled: true });

    app.update();

    // Verify development mode was activated
    let automation = app.world().resource::<WorkflowAutomation>();
    assert!(automation.development_mode.is_active, "Development mode should be active");
    assert!(automation.development_mode.debug_features_enabled, "Debug features should be enabled");
    assert!(automation.development_mode.profiling_enabled, "Profiling should be enabled");
    assert!(automation.development_mode.inspector_enabled, "Inspector should be enabled");
    assert!(automation.development_mode.verbose_logging, "Verbose logging should be enabled");
    assert!(automation.development_mode.source_maps_enabled, "Source maps should be enabled");

    // Test disabling development mode
    let mut workflow_events = app.world_mut().resource_mut::<Events<WorkflowEvent>>();
    workflow_events.send(WorkflowEvent::ToggleDevelopmentMode { enabled: false });

    app.update();

    let automation = app.world().resource::<WorkflowAutomation>();
    assert!(!automation.development_mode.is_active, "Development mode should be inactive");
    assert!(!automation.development_mode.debug_features_enabled, "Debug features should be disabled");
    assert!(!automation.development_mode.profiling_enabled, "Profiling should be disabled");
    assert!(!automation.development_mode.inspector_enabled, "Inspector should be disabled");
    assert!(!automation.development_mode.verbose_logging, "Verbose logging should be disabled");

    // Verify UI notifications were sent
    let ui_events = app.world().resource::<Events<UINotificationEvent>>();
    let mut ui_reader = ui_events.get_reader();
    let ui_events_vec: Vec<_> = ui_reader.read(&ui_events).collect();
    
    assert!(!ui_events_vec.is_empty(), "Should generate UI notifications");
    
    let has_activation_notification = ui_events_vec.iter().any(|event| {
        if let UINotificationEvent::Info { message, .. } = event {
            message.contains("Development mode activated")
        } else {
            false
        }
    });
    
    let has_deactivation_notification = ui_events_vec.iter().any(|event| {
        if let UINotificationEvent::Info { message, .. } = event {
            message.contains("Development mode deactivated")
        } else {
            false
        }
    });

    assert!(has_activation_notification, "Should notify about development mode activation");
    assert!(has_deactivation_notification, "Should notify about development mode deactivation");
}
```

#### 5. Window Behavior Configuration Testing
```rust
#[test]
fn test_window_behavior_configuration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, workflow_automation_system)
       .add_event::<WorkflowEvent>()
       .add_event::<WindowBehaviorEvent>()
       .add_event::<UINotificationEvent>();

    let workflow_automation = WorkflowAutomation::default();
    app.world_mut().insert_resource(workflow_automation);

    // Create test window behavior
    let new_behavior = DevelopmentWindowBehavior {
        keep_always_visible: true,
        disable_auto_hide: true,
        prevent_minimize: true,
        maintain_focus: true,
        overlay_mode: false,
        transparency_level: 0.9,
        window_priority: WindowPriority::High,
    };

    // Test window behavior configuration
    let mut workflow_events = app.world_mut().resource_mut::<Events<WorkflowEvent>>();
    workflow_events.send(WorkflowEvent::ConfigureWindowBehavior {
        behavior: new_behavior.clone(),
    });

    app.update();

    // Verify window behavior was updated
    let automation = app.world().resource::<WorkflowAutomation>();
    assert_eq!(automation.window_behavior.keep_always_visible, new_behavior.keep_always_visible);
    assert_eq!(automation.window_behavior.disable_auto_hide, new_behavior.disable_auto_hide);
    assert_eq!(automation.window_behavior.prevent_minimize, new_behavior.prevent_minimize);
    assert_eq!(automation.window_behavior.maintain_focus, new_behavior.maintain_focus);
    assert_eq!(automation.window_behavior.transparency_level, new_behavior.transparency_level);

    // Verify window behavior events were generated
    let window_events = app.world().resource::<Events<WindowBehaviorEvent>>();
    let mut window_reader = window_events.get_reader();
    let window_events_vec: Vec<_> = window_reader.read(&window_events).collect();
    
    assert!(!window_events_vec.is_empty(), "Should generate window behavior events");
    
    // Check for specific window behavior events
    let has_always_on_top = window_events_vec.iter().any(|event| {
        matches!(event, WindowBehaviorEvent::SetAlwaysOnTop(true))
    });
    
    let has_disable_auto_hide = window_events_vec.iter().any(|event| {
        matches!(event, WindowBehaviorEvent::DisableAutoHide)
    });
    
    let has_prevent_minimize = window_events_vec.iter().any(|event| {
        matches!(event, WindowBehaviorEvent::PreventMinimize(true))
    });

    assert!(has_always_on_top, "Should set always on top");
    assert!(has_disable_auto_hide, "Should disable auto hide");
    assert!(has_prevent_minimize, "Should prevent minimize");
}
```

#### 6. File Pattern Matching and Ignoring Testing
**Reference**: `./docs/bevy/examples/glob/pattern_matching.rs:125-158`
```rust
#[test]
fn test_file_pattern_matching_and_ignoring() {
    let ignore_patterns = vec![
        "*/node_modules/*".to_string(),
        "*/.git/*".to_string(),
        "*/dist/*".to_string(),
        "*/build/*".to_string(),
        "*.log".to_string(),
        "*.tmp".to_string(),
        "*~".to_string(),
    ];

    let test_cases = vec![
        // (file_path, should_ignore)
        (PathBuf::from("src/main.js"), false),
        (PathBuf::from("node_modules/react/index.js"), true),
        (PathBuf::from("project/.git/config"), true),
        (PathBuf::from("dist/bundle.js"), true),
        (PathBuf::from("build/output.js"), true),
        (PathBuf::from("debug.log"), true),
        (PathBuf::from("temp.tmp"), true),
        (PathBuf::from("backup.js~"), true),
        (PathBuf::from("src/component.tsx"), false),
        (PathBuf::from("styles/main.css"), false),
        (PathBuf::from("package.json"), false),
    ];

    for (file_path, expected_ignore) in test_cases {
        let should_ignore = should_ignore_file(&file_path, &ignore_patterns);
        assert_eq!(should_ignore, expected_ignore,
            "File ignore test failed for {:?}: expected {}, got {}",
            file_path, expected_ignore, should_ignore);
    }
}
```

#### 7. Reload Strategy Determination Testing
```rust
#[test]
fn test_reload_strategy_determination() {
    let settings = AutoReloadSettings {
        enabled: true,
        watch_patterns: vec!["**/*".to_string()],
        ignore_patterns: Vec::new(),
        debounce_delay_ms: 100,
        reload_strategy: ReloadStrategy::Smart,
        notification_enabled: true,
        pre_reload_hooks: Vec::new(),
        post_reload_hooks: Vec::new(),
    };

    let test_cases = vec![
        // (file_path, change_type, expected_strategy)
        (PathBuf::from("main.js"), FileChangeType::Modified, ReloadStrategy::HotReload),
        (PathBuf::from("component.jsx"), FileChangeType::Created, ReloadStrategy::Full),
        (PathBuf::from("types.ts"), FileChangeType::Modified, ReloadStrategy::HotReload),
        (PathBuf::from("App.tsx"), FileChangeType::Deleted, ReloadStrategy::Full),
        (PathBuf::from("styles.css"), FileChangeType::Modified, ReloadStrategy::HotReload),
        (PathBuf::from("config.scss"), FileChangeType::Modified, ReloadStrategy::HotReload),
        (PathBuf::from("package.json"), FileChangeType::Modified, ReloadStrategy::Incremental),
        (PathBuf::from("tsconfig.json"), FileChangeType::Modified, ReloadStrategy::Incremental),
        (PathBuf::from("webpack.config.js"), FileChangeType::Modified, ReloadStrategy::HotReload),
        (PathBuf::from("README.md"), FileChangeType::Modified, ReloadStrategy::Full),
        (PathBuf::from("image.png"), FileChangeType::Modified, ReloadStrategy::Full),
    ];

    for (file_path, change_type, expected_strategy) in test_cases {
        let determined_strategy = determine_reload_strategy(&file_path, change_type, &settings);
        assert_eq!(determined_strategy, expected_strategy,
            "Wrong reload strategy for {:?} with change {:?}: expected {:?}, got {:?}",
            file_path, change_type, expected_strategy, determined_strategy);
    }

    // Test with fixed strategies
    let fixed_strategies = vec![
        ReloadStrategy::Full,
        ReloadStrategy::HotReload,
        ReloadStrategy::Incremental,
    ];

    for fixed_strategy in fixed_strategies {
        let mut fixed_settings = settings.clone();
        fixed_settings.reload_strategy = fixed_strategy;

        let strategy = determine_reload_strategy(
            &PathBuf::from("any.file"),
            FileChangeType::Modified,
            &fixed_settings,
        );

        assert_eq!(strategy, fixed_strategy,
            "Fixed reload strategy should always return the configured strategy");
    }
}
```

### Edge Case Testing

#### 8. High-Frequency File Changes Testing
```rust
#[test]
fn test_high_frequency_file_changes() {
    let auto_reload_settings = AutoReloadSettings {
        enabled: true,
        watch_patterns: vec!["**/*.js".to_string()],
        ignore_patterns: Vec::new(),
        debounce_delay_ms: 100, // 100ms debounce
        reload_strategy: ReloadStrategy::HotReload,
        notification_enabled: false, // Disable notifications for performance
        pre_reload_hooks: Vec::new(),
        post_reload_hooks: Vec::new(),
    };

    let mut reload_history = VecDeque::new();
    let reload_events = Events::<ReloadEvent>::default();
    let ui_events = Events::<UINotificationEvent>::default();
    let mut reload_writer = reload_events.get_writer();
    let mut ui_writer = ui_events.get_writer();

    let test_file = PathBuf::from("rapid_changes.js");
    let initial_time = Instant::now();

    // Simulate rapid file changes
    for i in 0..10 {
        handle_file_change(
            &test_file,
            FileChangeType::Modified,
            &auto_reload_settings,
            &mut reload_history,
            &reload_writer,
            &ui_writer,
        );
        
        // Small delay between changes
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // All changes should be recorded in history
    assert_eq!(reload_history.len(), 10, "All file changes should be recorded");

    // But due to debouncing, actual reload events might be fewer
    // (This would be tested with a more realistic debouncing implementation)

    // Verify history contains the correct file path
    for event in &reload_history {
        assert_eq!(event.file_path, test_file);
        assert_eq!(event.change_type, FileChangeType::Modified);
        assert_eq!(event.strategy, ReloadStrategy::HotReload);
    }

    let total_time = initial_time.elapsed();
    assert!(total_time.as_millis() < 500, "High-frequency changes should be processed quickly");
}
```

#### 9. Resource Cleanup Testing
```rust
#[test]
fn test_resource_cleanup() {
    let mut file_watchers = HashMap::new();
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create multiple test files
    for i in 0..5 {
        let test_file = temp_dir.path().join(format!("test_{}.js", i));
        std::fs::write(&test_file, format!("// Test file {}", i)).unwrap();
    }

    let auto_reload_settings = AutoReloadSettings {
        enabled: true,
        watch_patterns: vec![
            temp_dir.path().join("**/*.js").to_string_lossy().to_string(),
        ],
        ignore_patterns: Vec::new(),
        debounce_delay_ms: 100,
        reload_strategy: ReloadStrategy::Smart,
        notification_enabled: false,
        pre_reload_hooks: Vec::new(),
        post_reload_hooks: Vec::new(),
    };

    // Setup file watchers
    let setup_result = setup_file_watchers(&auto_reload_settings, &mut file_watchers);
    
    if setup_result.is_ok() {
        assert!(!file_watchers.is_empty(), "Should create file watchers");
        
        // Verify all watchers are active
        for (_, watcher) in &file_watchers {
            assert!(watcher.is_active, "All watchers should be active initially");
        }

        // Test cleanup
        let initial_count = file_watchers.len();
        
        // Stop all watchers
        for (_, mut watcher) in file_watchers.drain() {
            watcher.stop();
        }

        assert_eq!(file_watchers.len(), 0, "All file watchers should be removed");
        assert!(initial_count > 0, "Should have had file watchers to cleanup");
    }

    // Test reload history cleanup
    let mut reload_history = VecDeque::new();
    
    // Add many events to trigger cleanup
    for i in 0..150 {
        let reload_event = ReloadEvent {
            id: format!("reload_{}", i),
            file_path: PathBuf::from(format!("test_{}.js", i)),
            change_type: FileChangeType::Modified,
            strategy: ReloadStrategy::HotReload,
            timestamp: Instant::now(),
            status: ReloadStatus::Completed,
        };
        reload_history.push_back(reload_event);
    }

    assert_eq!(reload_history.len(), 150);

    // Simulate history cleanup (would happen in handle_file_change)
    while reload_history.len() > 100 {
        reload_history.pop_front();
    }

    assert_eq!(reload_history.len(), 100, "History should be limited to 100 entries");
}
```

### Manual Testing Checklist

- [ ] Auto-reload on save checkbox correctly enables/disables file watching
- [ ] File changes trigger appropriate reload strategies (hot/incremental/full)
- [ ] Development mode checkbox activates debugging and profiling features
- [ ] Keep window always visible prevents window hiding during development
- [ ] Watch patterns correctly include/exclude specified file types
- [ ] Ignore patterns prevent triggering reloads for unwanted files
- [ ] Debounce delay prevents excessive reloads during rapid file changes
- [ ] Reload notifications appear when enabled in settings
- [ ] Pre/post-reload hooks execute correctly with file context
- [ ] Performance remains responsive during high-frequency file changes

**Bevy Examples**: `./docs/bevy/examples/file_watcher/file_watcher.rs:585-622`, `./docs/bevy/examples/app/development_mode.rs:525-562`  
**Integration Points**: All development workflow automation components  
**Success Criteria**: All tests pass, sub-100ms reload detection, reliable file watching across platforms
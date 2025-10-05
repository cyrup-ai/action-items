# Task 6: Development Workflow Automation System

## Implementation Details

**File**: `ui/src/ui/workflow_automation.rs`  
**Lines**: 285-395  
**Architecture**: Automated development workflow with file watching and hot-reload capabilities  
**Integration**: FileWatcher, ProcessManager, WindowManager, DevelopmentState  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct WorkflowAutomation {
    pub auto_reload_settings: AutoReloadSettings,
    pub development_mode: DevelopmentMode,
    pub window_behavior: DevelopmentWindowBehavior,
    pub file_watchers: HashMap<PathBuf, FileWatcher>,
    pub reload_history: VecDeque<ReloadEvent>,
    pub automation_metrics: AutomationMetrics,
}

#[derive(Clone, Debug)]
pub struct AutoReloadSettings {
    pub enabled: bool,
    pub watch_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub debounce_delay_ms: u64,
    pub reload_strategy: ReloadStrategy,
    pub notification_enabled: bool,
    pub pre_reload_hooks: Vec<String>,
    pub post_reload_hooks: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReloadStrategy {
    Full,          // Complete application restart
    HotReload,     // In-place module reload
    Incremental,   // Only reload changed modules
    Smart,         // Choose strategy based on change type
}

#[derive(Clone, Debug)]
pub struct DevelopmentMode {
    pub is_active: bool,
    pub debug_features_enabled: bool,
    pub profiling_enabled: bool,
    pub verbose_logging: bool,
    pub development_panels: Vec<DevelopmentPanel>,
    pub inspector_enabled: bool,
    pub source_maps_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct DevelopmentWindowBehavior {
    pub keep_always_visible: bool,
    pub disable_auto_hide: bool,
    pub prevent_minimize: bool,
    pub maintain_focus: bool,
    pub overlay_mode: bool,
    pub transparency_level: f32,
    pub window_priority: WindowPriority,
}

pub fn workflow_automation_system(
    mut workflow_automation: ResMut<WorkflowAutomation>,
    mut workflow_events: EventReader<WorkflowEvent>,
    mut file_events: EventWriter<FileWatchEvent>,
    mut reload_events: EventWriter<ReloadEvent>,
    mut window_events: EventWriter<WindowBehaviorEvent>,
    mut ui_events: EventWriter<UINotificationEvent>,
    time: Res<Time>,
) {
    // Process workflow automation events
    for event in workflow_events.read() {
        match event {
            WorkflowEvent::ToggleAutoReload { enabled } => {
                let result = configure_auto_reload(
                    &mut workflow_automation.auto_reload_settings,
                    *enabled,
                    &mut workflow_automation.file_watchers,
                    &file_events,
                );

                match result {
                    Ok(_) => {
                        ui_events.send(UINotificationEvent::Success {
                            message: format!("Auto-reload {}", 
                                if *enabled { "enabled" } else { "disabled" }
                            ),
                        });

                        // Update metrics
                        workflow_automation.automation_metrics.auto_reload_toggles += 1;
                        
                        if *enabled {
                            start_file_watching(&mut workflow_automation, &file_events);
                        } else {
                            stop_file_watching(&mut workflow_automation);
                        }
                    }
                    Err(error) => {
                        ui_events.send(UINotificationEvent::Error {
                            message: format!("Failed to configure auto-reload: {}", error),
                            action: Some(UIAction::OpenDeveloperSettings),
                        });
                    }
                }
            }

            WorkflowEvent::ToggleDevelopmentMode { enabled } => {
                let result = configure_development_mode(
                    &mut workflow_automation.development_mode,
                    *enabled,
                    &reload_events,
                );

                match result {
                    Ok(changes) => {
                        ui_events.send(UINotificationEvent::Info {
                            message: format!("Development mode {}", 
                                if *enabled { "activated" } else { "deactivated" }
                            ),
                            duration: Some(Duration::from_secs(3)),
                        });

                        // Apply development mode changes
                        for change in changes {
                            apply_development_mode_change(change, &reload_events, &window_events);
                        }
                    }
                    Err(error) => {
                        ui_events.send(UINotificationEvent::Error {
                            message: format!("Failed to configure development mode: {}", error),
                            action: Some(UIAction::RestartApplication),
                        });
                    }
                }
            }

            WorkflowEvent::ConfigureWindowBehavior { behavior } => {
                let result = configure_window_behavior(
                    &mut workflow_automation.window_behavior,
                    behavior,
                    &window_events,
                );

                match result {
                    Ok(_) => {
                        ui_events.send(UINotificationEvent::Success {
                            message: "Window behavior updated".to_string(),
                        });
                    }
                    Err(error) => {
                        ui_events.send(UINotificationEvent::Warning {
                            message: format!("Window behavior update failed: {}", error),
                            details: Some("Some settings may require application restart".to_string()),
                        });
                    }
                }
            }

            WorkflowEvent::FileChanged { file_path, change_type } => {
                handle_file_change(
                    file_path,
                    *change_type,
                    &workflow_automation.auto_reload_settings,
                    &mut workflow_automation.reload_history,
                    &reload_events,
                    &ui_events,
                );
            }
        }
    }

    // Process pending file watch events
    process_file_watch_events(&mut workflow_automation, &reload_events, &ui_events);

    // Update automation metrics
    update_automation_metrics(&mut workflow_automation.automation_metrics, &time);
}

fn configure_auto_reload(
    settings: &mut AutoReloadSettings,
    enabled: bool,
    file_watchers: &mut HashMap<PathBuf, FileWatcher>,
    file_events: &EventWriter<FileWatchEvent>,
) -> Result<(), WorkflowError> {
    settings.enabled = enabled;

    if enabled {
        // Setup file watchers for watched patterns
        setup_file_watchers(settings, file_watchers)?;
        
        // Send file watch start events
        for pattern in &settings.watch_patterns {
            file_events.send(FileWatchEvent::StartWatching {
                pattern: pattern.clone(),
                recursive: true,
                debounce_ms: settings.debounce_delay_ms,
            });
        }
    } else {
        // Stop all file watchers
        for (path, watcher) in file_watchers.drain() {
            watcher.stop();
        }

        file_events.send(FileWatchEvent::StopAllWatching);
    }

    Ok(())
}

fn setup_file_watchers(
    settings: &AutoReloadSettings,
    file_watchers: &mut HashMap<PathBuf, FileWatcher>,
) -> Result<(), WorkflowError> {
    for pattern in &settings.watch_patterns {
        let paths = glob::glob(pattern)
            .map_err(|e| WorkflowError::InvalidWatchPattern(pattern.clone(), e.to_string()))?;

        for path_result in paths {
            let path = path_result
                .map_err(|e| WorkflowError::FileWatchSetup(e.to_string()))?;

            // Skip if already watching
            if file_watchers.contains_key(&path) {
                continue;
            }

            // Create file watcher
            let watcher = create_file_watcher(&path, settings)?;
            file_watchers.insert(path, watcher);
        }
    }

    Ok(())
}
```

### File Watching and Hot Reload

**Reference**: `./docs/bevy/examples/file_watcher/file_watcher.rs:185-228`

```rust
#[derive(Clone, Debug)]
pub struct FileWatcher {
    pub path: PathBuf,
    pub watcher_handle: WatcherHandle,
    pub last_event: Option<Instant>,
    pub event_count: u32,
    pub is_active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
    Metadata,
}

fn create_file_watcher(
    path: &PathBuf,
    settings: &AutoReloadSettings,
) -> Result<FileWatcher, WorkflowError> {
    use notify::{Watcher, RecursiveMode, Event, EventKind};
    
    let (tx, rx) = std::sync::mpsc::channel();
    
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    }).map_err(|e| WorkflowError::FileWatcherCreation(e.to_string()))?;

    // Start watching
    watcher.watch(path, RecursiveMode::Recursive)
        .map_err(|e| WorkflowError::FileWatchStart(e.to_string()))?;

    Ok(FileWatcher {
        path: path.clone(),
        watcher_handle: WatcherHandle::new(watcher, rx),
        last_event: None,
        event_count: 0,
        is_active: true,
    })
}

fn handle_file_change(
    file_path: &PathBuf,
    change_type: FileChangeType,
    settings: &AutoReloadSettings,
    reload_history: &mut VecDeque<ReloadEvent>,
    reload_events: &EventWriter<ReloadEvent>,
    ui_events: &EventWriter<UINotificationEvent>,
) {
    // Check if file should be ignored
    if should_ignore_file(file_path, &settings.ignore_patterns) {
        return;
    }

    // Determine reload strategy
    let strategy = determine_reload_strategy(file_path, change_type, settings);
    
    // Create reload event
    let reload_event = ReloadEvent {
        id: generate_reload_id(),
        file_path: file_path.clone(),
        change_type,
        strategy,
        timestamp: Instant::now(),
        status: ReloadStatus::Pending,
    };

    // Add to history
    reload_history.push_back(reload_event.clone());
    
    // Limit history size
    if reload_history.len() > 100 {
        reload_history.pop_front();
    }

    // Execute pre-reload hooks
    if !settings.pre_reload_hooks.is_empty() {
        execute_reload_hooks(&settings.pre_reload_hooks, &reload_event);
    }

    // Send reload event
    reload_events.send(reload_event.clone());

    // Show notification if enabled
    if settings.notification_enabled {
        ui_events.send(UINotificationEvent::Info {
            message: format!("Reloading due to {} change: {}", 
                change_type.display_name(),
                file_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
            ),
            duration: Some(Duration::from_secs(2)),
        });
    }

    // Execute post-reload hooks
    if !settings.post_reload_hooks.is_empty() {
        execute_reload_hooks(&settings.post_reload_hooks, &reload_event);
    }
}

fn determine_reload_strategy(
    file_path: &PathBuf,
    change_type: FileChangeType,
    settings: &AutoReloadSettings,
) -> ReloadStrategy {
    match settings.reload_strategy {
        ReloadStrategy::Smart => {
            // Determine strategy based on file type and change
            match file_path.extension().and_then(|e| e.to_str()) {
                Some("js") | Some("ts") | Some("jsx") | Some("tsx") => {
                    if matches!(change_type, FileChangeType::Modified) {
                        ReloadStrategy::HotReload
                    } else {
                        ReloadStrategy::Full
                    }
                }
                Some("css") | Some("scss") | Some("sass") => ReloadStrategy::HotReload,
                Some("json") | Some("yaml") | Some("toml") => ReloadStrategy::Incremental,
                _ => ReloadStrategy::Full,
            }
        }
        other => other,
    }
}

fn should_ignore_file(file_path: &PathBuf, ignore_patterns: &[String]) -> bool {
    let path_str = file_path.to_string_lossy();
    
    for pattern in ignore_patterns {
        if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
            if glob_pattern.matches(&path_str) {
                return true;
            }
        }
    }
    
    // Default ignore patterns
    let default_ignores = [
        "*/node_modules/*",
        "*/.git/*",
        "*/dist/*",
        "*/build/*",
        "*.log",
        "*.tmp",
        "*~",
    ];
    
    for pattern in default_ignores {
        if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
            if glob_pattern.matches(&path_str) {
                return true;
            }
        }
    }
    
    false
}
```

### Development Mode Configuration

**Reference**: `./docs/bevy/examples/app/development_mode.rs:225-268`

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum DevelopmentChange {
    DebugPanelToggle(bool),
    ProfilerToggle(bool),
    InspectorToggle(bool),
    VerboseLoggingToggle(bool),
    SourceMapsToggle(bool),
    HotReloadToggle(bool),
}

fn configure_development_mode(
    mode: &mut DevelopmentMode,
    enabled: bool,
    reload_events: &EventWriter<ReloadEvent>,
) -> Result<Vec<DevelopmentChange>, WorkflowError> {
    let mut changes = Vec::new();
    
    mode.is_active = enabled;
    
    if enabled {
        // Enable debug features
        if !mode.debug_features_enabled {
            mode.debug_features_enabled = true;
            changes.push(DevelopmentChange::DebugPanelToggle(true));
        }
        
        // Enable profiling
        if !mode.profiling_enabled {
            mode.profiling_enabled = true;
            changes.push(DevelopmentChange::ProfilerToggle(true));
        }
        
        // Enable inspector
        if !mode.inspector_enabled {
            mode.inspector_enabled = true;
            changes.push(DevelopmentChange::InspectorToggle(true));
        }
        
        // Enable verbose logging
        if !mode.verbose_logging {
            mode.verbose_logging = true;
            changes.push(DevelopmentChange::VerboseLoggingToggle(true));
        }
        
        // Enable source maps
        if !mode.source_maps_enabled {
            mode.source_maps_enabled = true;
            changes.push(DevelopmentChange::SourceMapsToggle(true));
        }
    } else {
        // Disable development features
        mode.debug_features_enabled = false;
        mode.profiling_enabled = false;
        mode.inspector_enabled = false;
        mode.verbose_logging = false;
        
        changes.push(DevelopmentChange::DebugPanelToggle(false));
        changes.push(DevelopmentChange::ProfilerToggle(false));
        changes.push(DevelopmentChange::InspectorToggle(false));
        changes.push(DevelopmentChange::VerboseLoggingToggle(false));
    }
    
    Ok(changes)
}

fn configure_window_behavior(
    behavior: &mut DevelopmentWindowBehavior,
    new_behavior: &DevelopmentWindowBehavior,
    window_events: &EventWriter<WindowBehaviorEvent>,
) -> Result<(), WorkflowError> {
    // Update behavior settings
    *behavior = new_behavior.clone();
    
    // Apply window behavior changes
    if behavior.keep_always_visible {
        window_events.send(WindowBehaviorEvent::SetAlwaysOnTop(true));
    }
    
    if behavior.disable_auto_hide {
        window_events.send(WindowBehaviorEvent::DisableAutoHide);
    }
    
    if behavior.prevent_minimize {
        window_events.send(WindowBehaviorEvent::PreventMinimize(true));
    }
    
    if behavior.maintain_focus {
        window_events.send(WindowBehaviorEvent::MaintainFocus(true));
    }
    
    if behavior.overlay_mode {
        window_events.send(WindowBehaviorEvent::SetOverlayMode(true));
    }
    
    if behavior.transparency_level < 1.0 {
        window_events.send(WindowBehaviorEvent::SetTransparency(behavior.transparency_level));
    }
    
    window_events.send(WindowBehaviorEvent::SetPriority(behavior.window_priority));
    
    Ok(())
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui/ui_checkbox.rs:585-628`

```rust
// Development Workflow section
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(16.0)),
        row_gap: Val::Px(12.0),
        ..default()
    },
    background_color: Color::rgba(0.08, 0.08, 0.08, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(8.0)),
    ..default()
},
children: &[
    // Auto-reload on save
    (SettingsRowBundle {
        label: "Auto-reload on save".to_string(),
        control: ControlType::Checkbox {
            checked: workflow_automation.auto_reload_settings.enabled,
        },
        tooltip: Some("Automatically reload extensions when source files change".to_string()),
        ..default()
    },),
    
    // Disable pop to root search
    (SettingsRowBundle {
        label: "Disable pop to root search".to_string(),
        control: ControlType::Checkbox {
            checked: workflow_automation.development_mode.is_active 
                && !workflow_automation.development_mode.debug_features_enabled,
        },
        tooltip: Some("Prevent automatic return to root search during development".to_string()),
        ..default()
    },),
    
    // Open in development mode
    (SettingsRowBundle {
        label: "Open Raycast in development mode".to_string(),
        control: ControlType::Checkbox {
            checked: workflow_automation.development_mode.is_active,
        },
        tooltip: Some("Launch application with enhanced debugging and development features".to_string()),
        ..default()
    },),
    
    // Keep window always visible
    (SettingsRowBundle {
        label: "Keep window always visible during development".to_string(),
        control: ControlType::Checkbox {
            checked: workflow_automation.window_behavior.keep_always_visible,
        },
        tooltip: Some("Prevent window hiding during development sessions for continuous visibility".to_string()),
        ..default()
    },),
    
    // Advanced auto-reload settings
    (ExpansionPanelBundle {
        header: "Auto-Reload Settings".to_string(),
        expanded: false,
        content: NodeBundle {
            children: &[
                // Watch patterns
                (SettingsRowBundle {
                    label: "Watch Patterns".to_string(),
                    control: ControlType::TextList {
                        items: workflow_automation.auto_reload_settings.watch_patterns.clone(),
                        placeholder: "e.g. **/*.ts, **/*.js".to_string(),
                        editable: true,
                    },
                    ..default()
                },),
                
                // Ignore patterns
                (SettingsRowBundle {
                    label: "Ignore Patterns".to_string(),
                    control: ControlType::TextList {
                        items: workflow_automation.auto_reload_settings.ignore_patterns.clone(),
                        placeholder: "e.g. **/node_modules/**, **/*.log".to_string(),
                        editable: true,
                    },
                    ..default()
                },),
                
                // Debounce delay
                (SettingsRowBundle {
                    label: "Debounce Delay (ms)".to_string(),
                    control: ControlType::Slider {
                        value: workflow_automation.auto_reload_settings.debounce_delay_ms as f32,
                        min: 50.0,
                        max: 5000.0,
                        step: 50.0,
                    },
                    tooltip: Some("Delay before triggering reload after file changes".to_string()),
                    ..default()
                },),
                
                // Reload strategy
                (SettingsRowBundle {
                    label: "Reload Strategy".to_string(),
                    control: ControlType::Dropdown {
                        options: vec![
                            "Smart".to_string(),
                            "Hot Reload".to_string(),
                            "Incremental".to_string(),
                            "Full".to_string(),
                        ],
                        selected: match workflow_automation.auto_reload_settings.reload_strategy {
                            ReloadStrategy::Smart => 0,
                            ReloadStrategy::HotReload => 1,
                            ReloadStrategy::Incremental => 2,
                            ReloadStrategy::Full => 3,
                        },
                    },
                    ..default()
                },),
            ],
            ..default()
        },
        ..default()
    },),
]
```

### Architecture Notes

- Comprehensive file watching system with configurable patterns and ignore rules
- Intelligent reload strategies based on file types and change patterns
- Development mode with enhanced debugging, profiling, and inspection capabilities
- Advanced window behavior control for development workflows
- Real-time metrics collection for automation performance tracking
- Hook system for custom pre/post-reload actions and integrations
- Debouncing to prevent excessive reloads during rapid file changes
- Cross-platform file watching with platform-specific optimizations

**Bevy Examples**: `./docs/bevy/examples/file_watcher/file_watcher.rs:285-322`, `./docs/bevy/examples/app/development_mode.rs:385-422`  
**Integration Points**: FileWatcher, ProcessManager, WindowManager, DevelopmentState  
**Dependencies**: FileSystemWatcher, ReloadEngine, WindowController, DevelopmentTools
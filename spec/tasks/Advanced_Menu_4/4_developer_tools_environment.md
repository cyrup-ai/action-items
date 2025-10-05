# Task 4: Developer Tools Environment Control System

## Implementation Details

**File**: `ui/src/ui/developer_environment.rs`  
**Lines**: 225-345  
**Architecture**: Comprehensive development environment management with Node.js integration  
**Integration**: ProcessManager, EnvironmentManager, LoggingSystem  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct DeveloperEnvironment {
    pub node_environment: NodeEnvironment,
    pub logging_configuration: LoggingConfiguration,
    pub runtime_settings: RuntimeSettings,
    pub development_state: DevelopmentState,
    pub process_monitoring: ProcessMonitoring,
    pub security_settings: DevelopmentSecurity,
}

#[derive(Clone, Debug)]
pub struct NodeEnvironment {
    pub use_production_env: bool,
    pub node_version: Option<String>,
    pub npm_version: Option<String>,
    pub environment_variables: HashMap<String, String>,
    pub node_modules_path: Option<PathBuf>,
    pub package_manager: PackageManager,
    pub execution_context: ExecutionContext,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

#[derive(Clone, Debug)]
pub struct LoggingConfiguration {
    pub use_file_logging: bool,
    pub use_os_log: bool,
    pub log_level: LogLevel,
    pub output_directory: PathBuf,
    pub file_rotation: FileRotationConfig,
    pub structured_logging: bool,
    pub performance_logging: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug)]
pub struct DevelopmentState {
    pub is_development_mode: bool,
    pub debug_features_enabled: bool,
    pub hot_reload_active: bool,
    pub profiling_enabled: bool,
    pub source_maps_enabled: bool,
    pub runtime_statistics: RuntimeStatistics,
}

pub fn developer_environment_system(
    mut dev_environment: ResMut<DeveloperEnvironment>,
    mut env_events: EventReader<DeveloperEnvironmentEvent>,
    mut process_events: EventWriter<ProcessEvent>,
    mut logging_events: EventWriter<LoggingEvent>,
    mut ui_events: EventWriter<UINotificationEvent>,
) {
    // Process development environment changes
    for event in env_events.read() {
        match event {
            DeveloperEnvironmentEvent::SetNodeEnvironment { use_production } => {
                let result = configure_node_environment(
                    &mut dev_environment.node_environment,
                    *use_production,
                    &process_events,
                );

                match result {
                    Ok(changes) => {
                        ui_events.send(UINotificationEvent::Success {
                            message: format!("Node.js environment set to {}", 
                                if *use_production { "production" } else { "development" }
                            ),
                        });

                        // Apply environment variable changes
                        for (key, value) in changes {
                            process_events.send(ProcessEvent::SetEnvironmentVariable {
                                key,
                                value,
                                scope: EnvironmentScope::Development,
                            });
                        }
                    }
                    Err(error) => {
                        ui_events.send(UINotificationEvent::Error {
                            message: format!("Failed to configure Node.js environment: {}", error),
                            action: Some(UIAction::OpenDeveloperDocumentation),
                        });
                    }
                }
            }

            DeveloperEnvironmentEvent::ToggleFileLogging { enabled } => {
                let result = configure_file_logging(
                    &mut dev_environment.logging_configuration,
                    *enabled,
                    &logging_events,
                );

                match result {
                    Ok(_) => {
                        logging_events.send(LoggingEvent::ConfigurationChanged {
                            use_file_logging: *enabled,
                            use_os_log: !enabled,
                        });

                        ui_events.send(UINotificationEvent::Info {
                            message: format!("Switched to {} logging", 
                                if *enabled { "file" } else { "OS" }
                            ),
                            duration: Some(Duration::from_secs(3)),
                        });
                    }
                    Err(error) => {
                        ui_events.send(UINotificationEvent::Error {
                            message: format!("Failed to configure logging: {}", error),
                            action: Some(UIAction::OpenLoggingSettings),
                        });
                    }
                }
            }

            DeveloperEnvironmentEvent::ValidateEnvironment => {
                let validation_result = validate_development_environment(&dev_environment);
                
                match validation_result {
                    Ok(report) => {
                        ui_events.send(UINotificationEvent::Success {
                            message: format!("Environment validation passed ({} checks)", report.checks_passed),
                        });
                    }
                    Err(issues) => {
                        ui_events.send(UINotificationEvent::Warning {
                            message: format!("Environment validation found {} issues", issues.len()),
                            details: Some(format_validation_issues(&issues)),
                        });
                    }
                }
            }
        }
    }

    // Monitor development processes
    monitor_development_processes(&mut dev_environment, &process_events, &ui_events);
}

fn configure_node_environment(
    node_env: &mut NodeEnvironment,
    use_production: bool,
    process_events: &EventWriter<ProcessEvent>,
) -> Result<HashMap<String, String>, EnvironmentError> {
    let mut environment_changes = HashMap::new();

    // Set NODE_ENV
    let node_env_value = if use_production { "production" } else { "development" };
    environment_changes.insert("NODE_ENV".to_string(), node_env_value.to_string());
    node_env.environment_variables.insert("NODE_ENV".to_string(), node_env_value.to_string());

    // Configure additional production environment settings
    if use_production {
        // Disable debug features
        environment_changes.insert("DEBUG".to_string(), "".to_string());
        environment_changes.insert("NODE_OPTIONS".to_string(), "--max-old-space-size=4096".to_string());
        
        // Enable optimizations
        environment_changes.insert("NODE_ENV".to_string(), "production".to_string());
        environment_changes.insert("BABEL_ENV".to_string(), "production".to_string());
        
        // Set production logging
        environment_changes.insert("LOG_LEVEL".to_string(), "warn".to_string());
    } else {
        // Enable development features
        environment_changes.insert("DEBUG".to_string(), "*".to_string());
        environment_changes.insert("NODE_OPTIONS".to_string(), "--inspect --max-old-space-size=8192".to_string());
        
        // Set development-friendly settings
        environment_changes.insert("BABEL_ENV".to_string(), "development".to_string());
        environment_changes.insert("LOG_LEVEL".to_string(), "debug".to_string());
        
        // Enable source maps
        environment_changes.insert("GENERATE_SOURCEMAP".to_string(), "true".to_string());
    }

    // Detect Node.js and npm versions
    detect_node_runtime_info(node_env)?;
    
    // Validate Node.js installation
    validate_node_installation(node_env)?;

    node_env.use_production_env = use_production;
    Ok(environment_changes)
}

fn configure_file_logging(
    logging_config: &mut LoggingConfiguration,
    use_file_logging: bool,
    logging_events: &EventWriter<LoggingEvent>,
) -> Result<(), LoggingError> {
    logging_config.use_file_logging = use_file_logging;
    logging_config.use_os_log = !use_file_logging;

    if use_file_logging {
        // Ensure log directory exists
        std::fs::create_dir_all(&logging_config.output_directory)
            .map_err(|e| LoggingError::DirectoryCreationFailed(e.to_string()))?;

        // Setup file rotation if needed
        setup_log_rotation(&logging_config.file_rotation)?;

        // Initialize file logging
        initialize_file_logger(logging_config)?;
    } else {
        // Switch to OS logging
        #[cfg(target_os = "macos")]
        initialize_os_log_macos()?;
        
        #[cfg(target_os = "windows")]
        initialize_event_log_windows()?;
        
        #[cfg(target_os = "linux")]
        initialize_syslog_linux()?;
    }

    Ok(())
}
```

### Node.js Runtime Integration

**Reference**: `./docs/bevy/examples/process/process_management.rs:285-322`

```rust
#[derive(Clone, Debug)]
pub struct ProcessMonitoring {
    pub active_processes: HashMap<String, ProcessInfo>,
    pub resource_usage: ResourceUsage,
    pub performance_metrics: PerformanceMetrics,
    pub error_tracking: ErrorTracking,
}

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub command: String,
    pub arguments: Vec<String>,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
    pub started_at: Instant,
    pub status: ProcessStatus,
    pub resource_usage: ProcessResourceUsage,
}

fn detect_node_runtime_info(node_env: &mut NodeEnvironment) -> Result<(), EnvironmentError> {
    // Detect Node.js version
    let node_version_output = std::process::Command::new("node")
        .args(&["--version"])
        .output()
        .map_err(|_| EnvironmentError::NodeNotFound)?;

    if node_version_output.status.success() {
        let version_string = String::from_utf8_lossy(&node_version_output.stdout);
        node_env.node_version = Some(version_string.trim().to_string());
    } else {
        return Err(EnvironmentError::NodeVersionDetectionFailed);
    }

    // Detect npm version
    let npm_version_output = std::process::Command::new("npm")
        .args(&["--version"])
        .output()
        .map_err(|_| EnvironmentError::NpmNotFound)?;

    if npm_version_output.status.success() {
        let version_string = String::from_utf8_lossy(&npm_version_output.stdout);
        node_env.npm_version = Some(version_string.trim().to_string());
    }

    // Detect package manager
    node_env.package_manager = detect_package_manager()?;

    // Find node_modules path
    node_env.node_modules_path = find_node_modules_path();

    Ok(())
}

fn detect_package_manager() -> Result<PackageManager, EnvironmentError> {
    // Check for lock files to determine package manager
    if std::path::Path::new("yarn.lock").exists() {
        return Ok(PackageManager::Yarn);
    }
    
    if std::path::Path::new("pnpm-lock.yaml").exists() {
        return Ok(PackageManager::Pnpm);
    }
    
    if std::path::Path::new("bun.lockb").exists() {
        return Ok(PackageManager::Bun);
    }
    
    // Default to npm
    Ok(PackageManager::Npm)
}

fn validate_node_installation(node_env: &NodeEnvironment) -> Result<(), EnvironmentError> {
    // Check minimum Node.js version
    if let Some(ref version) = node_env.node_version {
        let version_num = parse_node_version(version)?;
        if version_num < (16, 0, 0) {
            return Err(EnvironmentError::NodeVersionTooOld {
                current: version.clone(),
                minimum: "16.0.0".to_string(),
            });
        }
    }

    // Validate npm installation
    if node_env.npm_version.is_none() {
        return Err(EnvironmentError::NpmNotInstalled);
    }

    // Check global packages
    validate_global_packages(node_env)?;

    Ok(())
}

fn validate_global_packages(node_env: &NodeEnvironment) -> Result<(), EnvironmentError> {
    let required_packages = vec![
        "typescript",
        "@raycast/api",
        "@raycast/utils",
    ];

    for package in required_packages {
        let check_command = match node_env.package_manager {
            PackageManager::Npm => format!("npm list -g {}", package),
            PackageManager::Yarn => format!("yarn global list --pattern {}", package),
            PackageManager::Pnpm => format!("pnpm list -g {}", package),
            PackageManager::Bun => format!("bun pm ls -g {}", package),
        };

        let output = std::process::Command::new("sh")
            .args(&["-c", &check_command])
            .output()
            .map_err(|_| EnvironmentError::PackageCheckFailed(package.to_string()))?;

        if !output.status.success() {
            log::warn!("Global package {} not found, development features may be limited", package);
        }
    }

    Ok(())
}
```

### Advanced Logging System

**Reference**: `./docs/bevy/examples/logging/logging.rs:185-228`

```rust
#[derive(Clone, Debug)]
pub struct FileRotationConfig {
    pub max_file_size_mb: u64,
    pub max_files: usize,
    pub rotation_frequency: RotationFrequency,
    pub compress_rotated: bool,
    pub cleanup_older_than_days: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RotationFrequency {
    Never,
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

fn initialize_file_logger(config: &LoggingConfiguration) -> Result<(), LoggingError> {
    let log_file_path = config.output_directory.join("raycast-development.log");
    
    // Create file appender with rotation
    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(match config.file_rotation.rotation_frequency {
            RotationFrequency::Never => tracing_appender::rolling::Rotation::NEVER,
            RotationFrequency::Hourly => tracing_appender::rolling::Rotation::HOURLY,
            RotationFrequency::Daily => tracing_appender::rolling::Rotation::DAILY,
            RotationFrequency::Weekly => tracing_appender::rolling::Rotation::DAILY, // Approximate
            RotationFrequency::Monthly => tracing_appender::rolling::Rotation::DAILY, // Approximate
        })
        .filename_prefix("raycast-dev")
        .filename_suffix("log")
        .max_log_files(config.file_rotation.max_files)
        .build(&config.output_directory)
        .map_err(|e| LoggingError::FileAppenderCreationFailed(e.to_string()))?;

    // Configure formatter
    let formatter = if config.structured_logging {
        tracing_subscriber::fmt::format()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
    } else {
        tracing_subscriber::fmt::format()
            .compact()
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
    };

    // Setup subscriber
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(match config.log_level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        })
        .with_writer(file_appender)
        .event_format(formatter)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| LoggingError::SubscriberSetupFailed(e.to_string()))?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn initialize_os_log_macos() -> Result<(), LoggingError> {
    use os_log::{OsLogger, OsLoggerConfig};
    
    let config = OsLoggerConfig::new("com.raycast.development")
        .level_filter(log::LevelFilter::Debug)
        .category("development");
    
    OsLogger::new(config)
        .init()
        .map_err(|e| LoggingError::OsLogInitFailed(e.to_string()))?;
    
    Ok(())
}

fn monitor_development_processes(
    dev_environment: &mut DeveloperEnvironment,
    process_events: &EventWriter<ProcessEvent>,
    ui_events: &EventWriter<UINotificationEvent>,
) {
    // Update resource usage statistics
    update_resource_usage(&mut dev_environment.process_monitoring);
    
    // Check for problematic processes
    check_process_health(&dev_environment.process_monitoring, ui_events);
    
    // Update performance metrics
    update_performance_metrics(&mut dev_environment.development_state);
}

fn update_resource_usage(monitoring: &mut ProcessMonitoring) {
    let current_usage = get_system_resource_usage();
    monitoring.resource_usage = current_usage;
    
    // Log high resource usage
    if monitoring.resource_usage.memory_usage_percent > 80.0 {
        log::warn!("High memory usage detected: {:.1}%", monitoring.resource_usage.memory_usage_percent);
    }
    
    if monitoring.resource_usage.cpu_usage_percent > 90.0 {
        log::warn!("High CPU usage detected: {:.1}%", monitoring.resource_usage.cpu_usage_percent);
    }
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui/ui_checkbox.rs:485-528`

```rust
// Developer Tools section
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
    // Section header
    (TextBundle::from_section(
        "Developer Tools",
        TextStyle {
            font: asset_server.load("fonts/Inter-SemiBold.ttf"),
            font_size: 16.0,
            color: Color::rgb(0.95, 0.95, 0.95),
        },
    ),),
    
    // Node.js environment setting
    (SettingsRowBundle {
        label: "Use Node production environment".to_string(),
        control: ControlType::Checkbox {
            checked: dev_environment.node_environment.use_production_env,
        },
        tooltip: Some("Force production Node.js environment for extension development".to_string()),
        ..default()
    },),
    
    // Logging configuration
    (SettingsRowBundle {
        label: "Use file logging instead of OSLog".to_string(),
        control: ControlType::Checkbox {
            checked: dev_environment.logging_configuration.use_file_logging,
        },
        tooltip: Some("Switch between file-based logging and system logging for development debugging".to_string()),
        ..default()
    },),
    
    // Environment validation
    (ButtonBundle {
        style: Style {
            width: Val::Px(180.0),
            height: Val::Px(32.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        },
        background_color: Color::rgb(0.3, 0.6, 0.3).into(),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    },
    children: &[
        (TextBundle::from_section(
            "Validate Environment",
            TextStyle {
                font: asset_server.load("fonts/Inter-Medium.ttf"),
                font_size: 12.0,
                color: Color::WHITE,
            },
        ),),
    ]),
    
    // Runtime information display
    (ExpansionPanelBundle {
        header: "Runtime Information".to_string(),
        expanded: false,
        content: NodeBundle {
            children: &[
                // Node.js version
                if let Some(ref version) = dev_environment.node_environment.node_version {
                    (InfoRowBundle {
                        label: "Node.js Version".to_string(),
                        value: version.clone(),
                        ..default()
                    },)
                } else { () },
                
                // npm version
                if let Some(ref npm_version) = dev_environment.node_environment.npm_version {
                    (InfoRowBundle {
                        label: "npm Version".to_string(),
                        value: npm_version.clone(),
                        ..default()
                    },)
                } else { () },
                
                // Package manager
                (InfoRowBundle {
                    label: "Package Manager".to_string(),
                    value: format!("{:?}", dev_environment.node_environment.package_manager),
                    ..default()
                },),
                
                // Log directory
                (InfoRowBundle {
                    label: "Log Directory".to_string(),
                    value: dev_environment.logging_configuration.output_directory
                        .to_string_lossy().to_string(),
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

- Comprehensive Node.js environment management with automatic version detection
- Advanced file logging system with rotation, compression, and structured output
- Runtime process monitoring with resource usage tracking and health checks
- Package manager detection and validation for different development workflows
- Environment variable management for development and production modes
- Integration with system logging services (OSLog on macOS, Event Log on Windows)
- Performance metrics collection for development workflow optimization
- Security-aware development environment configuration

**Bevy Examples**: `./docs/bevy/examples/process/process_management.rs:385-422`, `./docs/bevy/examples/logging/logging.rs:285-322`  
**Integration Points**: ProcessManager, LoggingSystem, EnvironmentManager, SecurityManager  
**Dependencies**: NodeRuntime, FileSystem, SystemLogger, ProcessMonitor
# Advanced Menu 4 Specification

## Overview
Advanced Menu 4 represents the most specialized configuration options for developers, system administrators, and power users. This interface provides window capture automation, custom wallpaper integration, comprehensive developer tools, and enterprise-grade proxy/certificate management.

## Layout Architecture
- **Base Layout**: Advanced tab active in primary navigation
- **Specialized Sections**: Developer-focused and enterprise configuration groupings
- **Mixed Control Types**: Checkboxes, file selectors, dropdowns, and action buttons
- **Warning Integration**: Important system restart notifications

## Configuration Sections

### Window Capture System (Extended)

#### Automated Capture Configuration
- **Section Title**: "Window Capture"
- **Description**: "Capture the Raycast window to share it or add a screenshot of your extension to the Store."
- **Primary Action**: "Record Hotkey" button for hotkey assignment

#### Capture Behavior Options
- **Copy to Clipboard**:
  - **Control Type**: Checkbox toggle
  - **Current State**: Enabled (checked)
  - **Purpose**: Automatically copy captured screenshots to system clipboard
  - **Workflow Integration**: Seamless integration with paste workflows

- **Show in Finder**:
  - **Control Type**: Checkbox toggle
  - **Current State**: Disabled (unchecked)
  - **Purpose**: Automatically reveal captured screenshots in Finder
  - **File Management**: Direct access to captured image files

### Visual Customization System

#### Custom Wallpaper Integration
- **Setting**: "Custom Wallpaper"
- **Control Type**: File selection button
- **Current State**: "Select File" (no file currently selected)
- **Purpose**: Custom background wallpaper for launcher interface
- **File Support**: Standard image formats (PNG, JPG, etc.)
- **Info Icon**: Guidelines for optimal wallpaper dimensions and formats

### Developer Tools Configuration

#### Development Environment Settings
- **Section Title**: "Developer Tools"
- **Purpose**: Comprehensive development workflow customization

##### Node.js Environment Control
- **Use Node production environment**:
  - **Control Type**: Checkbox toggle
  - **Current State**: Enabled (checked)
  - **Purpose**: Force production Node.js environment for extension development
  - **Performance**: Optimized performance and error handling

##### Logging Configuration
- **Use file logging instead of OSLog**:
  - **Control Type**: Checkbox toggle
  - **Current State**: Enabled (checked)
  - **Purpose**: Alternative logging system for development debugging
  - **Developer Benefit**: Enhanced debugging capabilities and log persistence

##### Development Workflow Automation
- **Auto-reload on save**:
  - **Control Type**: Checkbox toggle
  - **Current State**: Enabled (checked)
  - **Purpose**: Automatic extension reload when source files change
  - **Developer Experience**: Streamlined development iteration cycle

##### Root Search Behavior Override
- **Disable pop to root search**:
  - **Control Type**: Checkbox toggle
  - **Current State**: Disabled (unchecked)
  - **Purpose**: Prevent automatic return to root search during development
  - **Debug Workflow**: Maintain current state for debugging purposes

##### Development Mode Activation
- **Open Raycast in development mode**:
  - **Control Type**: Checkbox toggle
  - **Current State**: Enabled (checked)
  - **Purpose**: Launch application in development mode with enhanced debugging
  - **Developer Features**: Access to development-only features and diagnostics

##### Window Behavior During Development
- **Keep window always visible during development**:
  - **Control Type**: Checkbox toggle
  - **Current State**: Enabled (checked)
  - **Purpose**: Prevent window hiding during development sessions
  - **Development Efficiency**: Continuous visibility for debugging and testing

### Enterprise Network Configuration

#### Proxy and Certificate Management
- **Section Title**: "Proxy and Certificate Settings"
- **System Integration Notice**: "If you change your system's proxy settings or Keychain certificates later, you need to restart Raycast."
- **Enterprise Focus**: Corporate network integration requirements

##### Web Proxy Configuration
- **Setting**: "Web Proxy"
- **Use System Network Settings**:
  - **Control Type**: Checkbox toggle
  - **Current State**: Enabled (checked)
  - **Purpose**: Inherit proxy settings from system network configuration
  - **Enterprise Integration**: Seamless corporate network integration

##### Certificate Management
- **Setting**: "Certificates"
- **Current Selection**: "Keychain"
- **Control Type**: Dropdown selection
- **Purpose**: Certificate store selection for HTTPS connections
- **Options**:
  - Keychain (macOS system keychain)
  - Custom certificate store
  - Enterprise certificate management
- **Info Icon**: Details about certificate validation and security implications

## Functional Requirements

### Advanced Window Capture System
- **Hotkey Integration**: Global hotkey assignment for automated captures
- **Multi-Format Support**: Support for various image formats and quality settings
- **Workflow Integration**: Seamless integration with clipboard and file management
- **Batch Capture**: Support for automated batch capture scenarios

### Custom Wallpaper Management
- **Dynamic Loading**: Efficient loading and display of custom wallpaper images
- **Format Validation**: Comprehensive validation of supported image formats
- **Performance Optimization**: Optimized rendering without impact on application performance
- **Fallback Handling**: Graceful fallback to default wallpaper for invalid files

### Comprehensive Developer Tools
- **Development Environment Control**: Fine-grained control over development environment settings
- **Hot Reload System**: Intelligent file watching and automatic reload functionality
- **Enhanced Logging**: Advanced logging system with configurable output destinations
- **Debug Mode Integration**: Comprehensive debugging features and developer diagnostics

### Enterprise Network Integration
- **Proxy Support**: Full proxy support including authentication and advanced configurations
- **Certificate Management**: Integration with system and enterprise certificate stores
- **Network Monitoring**: Real-time network connectivity and proxy status monitoring
- **Security Compliance**: Enterprise security and compliance requirements

## Bevy Implementation Examples

### Window Capture Automation
- Reference: `./docs/bevy/examples/window/screenshot.rs` - Automated window capture functionality
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Hotkey assignment and capture triggering

### File Selection Interface
- Reference: `./docs/bevy/examples/ui/button.rs` - File selection button and file picker integration
- Reference: `./docs/bevy/examples/asset_loading/asset_loading.rs` - Dynamic wallpaper loading

### Developer Tools Configuration
- Reference: `./docs/bevy/examples/ui/ui.rs` - Multiple checkbox configuration layout
- Reference: `./docs/bevy/examples/reflection/reflection.rs` - Development settings persistence

### Proxy and Certificate Integration
- Reference: `./docs/bevy/examples/ui/ui.rs` - Dropdown configuration for certificate selection
- Reference: `./docs/bevy/examples/async_tasks/async_compute.rs` - Network configuration validation

### Custom Wallpaper System
- Reference: `./docs/bevy/examples/asset_loading/hot_asset_reloading.rs` - Dynamic wallpaper loading and updates
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Background image rendering and scaling

### Development Mode Controls
- Reference: `./docs/bevy/examples/app/return_after_run.rs` - Development mode state management
- Reference: `./docs/bevy/examples/reflection/reflection.rs` - Developer preferences persistence

### System Integration Monitoring
- Reference: `./docs/bevy/examples/async_tasks/async_compute.rs` - Background system monitoring
- Reference: `./docs/bevy/examples/ui/text.rs` - Dynamic status updates and notifications

## State Management Requirements

### Developer Tool State Management
- **Development Mode State**: Persistent tracking of development mode activation and settings
- **Hot Reload State**: File watching state and automatic reload functionality
- **Logging State**: Dynamic logging configuration and output management
- **Debug State**: Comprehensive debug mode state and feature availability

### Network Configuration State
- **Proxy State Monitoring**: Real-time proxy configuration and connectivity monitoring
- **Certificate State**: Certificate validation and store integration status
- **Network Health**: Continuous network connectivity and performance monitoring
- **Configuration Sync**: Synchronization with system network configuration changes

### Capture System State
- **Hotkey State**: Global hotkey registration and conflict detection
- **Capture Settings**: Persistent capture configuration and behavior preferences
- **File Management**: Capture file location and organization state
- **Clipboard Integration**: Clipboard state management for capture operations

## Security and Enterprise Compliance

### Developer Security Framework
- **Development Mode Security**: Secure development mode with appropriate privilege restrictions
- **Code Execution Security**: Safe execution environment for development and testing
- **Debug Information Security**: Secure handling of debug information and sensitive data
- **Development Data Protection**: Protection of development data and intellectual property

### Enterprise Network Security
- **Proxy Authentication**: Secure proxy authentication and credential management
- **Certificate Validation**: Comprehensive certificate validation and trust management
- **Network Traffic Security**: Secure handling of network traffic and data transmission
- **Compliance Monitoring**: Continuous compliance monitoring and audit trail

### System Integration Security
- **Privilege Management**: Minimal privilege requirements for advanced system integration
- **File System Security**: Secure file system access for wallpaper and capture operations
- **Network Security**: Secure network access and proxy integration
- **Certificate Security**: Secure certificate store access and validation

## Performance Optimization Requirements

### Development Tool Performance
- **Hot Reload Efficiency**: Efficient file watching and reload mechanisms
- **Development Mode Performance**: Optimized performance for development workflows
- **Logging Performance**: High-performance logging system with minimal overhead
- **Debug Performance**: Efficient debugging features without production performance impact

### Network Integration Performance
- **Proxy Performance**: Optimized proxy integration with minimal latency impact
- **Certificate Performance**: Efficient certificate validation and caching
- **Network Monitoring**: Low-overhead network monitoring and status tracking
- **Configuration Performance**: Fast configuration loading and application

### UI Responsiveness
- **Real-time Configuration**: Immediate application of configuration changes
- **Background Processing**: Non-blocking processing of file operations and network tasks
- **Progressive Loading**: Incremental loading of complex configuration options
- **Smooth Transitions**: Fluid transitions for configuration changes and mode switches

## Error Handling and Recovery

### Developer Tool Error Handling
- **Hot Reload Failures**: Graceful handling of file watching and reload failures
- **Development Mode Errors**: Clear error handling and recovery for development mode issues
- **Logging Failures**: Robust error handling for logging system failures
- **Debug Tool Failures**: Recovery mechanisms for debug tool and diagnostic failures

### Network Configuration Error Handling
- **Proxy Configuration Errors**: Clear error messaging and recovery for proxy setup issues
- **Certificate Errors**: Detailed error handling for certificate validation and trust issues
- **Network Connectivity Errors**: Graceful handling of network connectivity failures
- **Configuration Sync Errors**: Recovery mechanisms for system configuration synchronization issues

### System Integration Error Recovery
- **File System Errors**: Recovery from file system access and permission issues
- **System Service Errors**: Handling of system service integration failures
- **Privilege Errors**: Clear guidance for system privilege and permission requirements
- **Configuration Corruption**: Recovery from corrupted configuration data and settings

### User Experience Recovery
- **Configuration Reset**: Safe reset options for problematic advanced configurations
- **Diagnostic Tools**: Built-in diagnostic tools for troubleshooting complex system integration
- **Expert Support**: Clear escalation paths for enterprise and developer support
- **Documentation Integration**: Comprehensive documentation and help integration

## Enterprise Integration Requirements

### Corporate Network Support
- **Proxy Protocol Support**: Support for HTTP, HTTPS, SOCKS, and other proxy protocols
- **Authentication Integration**: Integration with corporate authentication systems
- **Network Policy Compliance**: Compliance with corporate network policies and restrictions
- **Audit and Monitoring**: Comprehensive audit trail for corporate compliance requirements

### Certificate Management Integration
- **Enterprise Certificate Stores**: Integration with enterprise certificate management systems
- **Certificate Lifecycle**: Support for certificate renewal and lifecycle management
- **Trust Chain Validation**: Comprehensive trust chain validation for enterprise certificates
- **Compliance Reporting**: Detailed compliance reporting for certificate usage and validation

### Development Workflow Integration
- **CI/CD Integration**: Integration with corporate CI/CD pipelines and development workflows
- **Source Control Integration**: Seamless integration with enterprise source control systems
- **Development Environment Management**: Centralized management of development environment configurations
- **Team Collaboration**: Enhanced team collaboration features for enterprise development teams

## Bevy Implementation Details

### Component Architecture

#### Developer Tools Configuration Components
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Component, Reflect)]
pub struct WindowCaptureSettings {
    pub hotkey: Option<KeyCombination>,
    pub copy_to_clipboard: bool,
    pub show_in_finder: bool,
    pub auto_save: bool,
    pub save_location: PathBuf,
    pub image_format: CaptureFormat,
    pub quality: f32,
}

#[derive(Component, Reflect)]
pub struct CustomWallpaperConfig {
    pub current_wallpaper: Option<Handle<Image>>,
    pub wallpaper_path: Option<PathBuf>,
    pub scaling_mode: WallpaperScaling,
    pub opacity: f32,
    pub blur_radius: f32,
}

#[derive(Component, Reflect)]
pub struct DeveloperToolsState {
    pub use_node_production: bool,
    pub use_file_logging: bool,
    pub auto_reload_enabled: bool,
    pub disable_pop_to_root: bool,
    pub development_mode: bool,
    pub keep_window_visible: bool,
    pub log_level: LogLevel,
    pub hot_reload_extensions: HashSet<String>,
}

#[derive(Component, Reflect)]
pub struct NetworkProxyConfig {
    pub use_system_proxy: bool,
    pub custom_proxy_url: Option<String>,
    pub proxy_authentication: Option<ProxyAuth>,
    pub certificate_store: CertificateStore,
    pub ssl_verification_enabled: bool,
    pub connection_timeout: u32,
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum CaptureFormat {
    PNG,
    JPEG,
    WEBP,
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum WallpaperScaling {
    Fit,
    Fill,
    Stretch,
    Center,
    Tile,
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Reflect, Clone, PartialEq)]
pub enum CertificateStore {
    System,
    Keychain,
    Custom(String),
}

#[derive(Reflect, Clone)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String, // Should be encrypted in production
    pub auth_type: ProxyAuthType,
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum ProxyAuthType {
    Basic,
    NTLM,
    Kerberos,
}
```

#### File Management Components
```rust
#[derive(Component, Reflect)]
pub struct FileSelector {
    pub label: String,
    pub current_path: Option<PathBuf>,
    pub file_types: Vec<FileType>,
    pub multiple_selection: bool,
    pub directory_mode: bool,
}

#[derive(Reflect, Clone)]
pub struct FileType {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Component, Reflect)]
pub struct FileDropZone {
    pub active: bool,
    pub accepted_types: Vec<String>,
    pub max_file_size: u64,
    pub hover_state: bool,
}
```

### Resource Management for Developer Tools
```rust
#[derive(Resource, Reflect)]
pub struct DeveloperEnvironment {
    pub tools_state: DeveloperToolsState,
    pub capture_settings: WindowCaptureSettings,
    pub wallpaper_config: CustomWallpaperConfig,
    pub proxy_config: NetworkProxyConfig,
    pub file_watchers: HashMap<PathBuf, FileWatcher>,
    pub active_debug_sessions: Vec<DebugSession>,
}

#[derive(Reflect, Clone)]
pub struct FileWatcher {
    pub path: PathBuf,
    pub recursive: bool,
    pub patterns: Vec<String>,
    pub last_modified: std::time::SystemTime,
}

#[derive(Reflect, Clone)]
pub struct DebugSession {
    pub id: String,
    pub name: String,
    pub start_time: std::time::SystemTime,
    pub log_entries: Vec<LogEntry>,
    pub breakpoints: Vec<Breakpoint>,
}

#[derive(Reflect, Clone)]
pub struct LogEntry {
    pub timestamp: std::time::SystemTime,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
}

#[derive(Reflect, Clone)]
pub struct Breakpoint {
    pub file: PathBuf,
    pub line: u32,
    pub condition: Option<String>,
    pub enabled: bool,
}

#[derive(Resource, Default, Reflect)]
pub struct NetworkDiagnostics {
    pub proxy_status: ProxyStatus,
    pub ssl_certificates: Vec<CertificateInfo>,
    pub connection_tests: HashMap<String, ConnectionTest>,
    pub dns_resolution_cache: HashMap<String, Vec<std::net::IpAddr>>,
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum ProxyStatus {
    Connected,
    Disconnected,
    Error,
    Testing,
}

#[derive(Reflect, Clone)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub valid_from: String,
    pub valid_to: String,
    pub fingerprint: String,
    pub trusted: bool,
}
```

### Event System for Developer Tools
```rust
#[derive(Event, Reflect)]
pub enum DeveloperToolsEvent {
    WindowCaptureTriggered,
    WindowCaptureCompleted { path: PathBuf, success: bool },
    WallpaperSelected(PathBuf),
    WallpaperApplied { success: bool, error: Option<String> },
    FileWatcherTriggered { path: PathBuf, change_type: FileChangeType },
    LogLevelChanged(LogLevel),
    DevelopmentModeToggled(bool),
    HotReloadTriggered(String),
    ProxyConfigurationChanged,
    CertificateValidationStarted(String),
    CertificateValidationCompleted { domain: String, valid: bool },
    NetworkDiagnosticsRequested,
    SystemRestartRequired(String),
}

#[derive(Event, Reflect)]
pub enum NetworkEvent {
    ProxyConnectionTest(String),
    ProxyConnectionResult { url: String, success: bool, latency: u64 },
    CertificateVerification { domain: String, certificate: CertificateInfo },
    SSLHandshakeStarted(String),
    SSLHandshakeCompleted { domain: String, success: bool },
    NetworkConfigurationSaved,
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}
```

### Window Capture System Implementation
```rust
fn window_capture_system(
    mut capture_settings: Query<&mut WindowCaptureSettings>,
    mut button_query: Query<(&Interaction, &WindowCaptureButton), Changed<Interaction>>,
    mut checkbox_query: Query<(&Interaction, &mut DeveloperCheckbox), Changed<Interaction>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<DeveloperToolsEvent>,
    mut hotkey_recording: Local<bool>,
) {
    // Handle capture button interactions
    for (interaction, button) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            match button.action {
                CaptureAction::RecordHotkey => {
                    *hotkey_recording = true;
                    info!("Starting hotkey recording for window capture");
                },
                CaptureAction::CaptureNow => {
                    events.send(DeveloperToolsEvent::WindowCaptureTriggered);
                    perform_window_capture();
                },
                CaptureAction::OpenCaptureFolder => {
                    if let Ok(settings) = capture_settings.get_single() {
                        open_folder_in_finder(&settings.save_location);
                    }
                },
            }
        }
    }
    
    // Handle checkbox toggles for capture options
    for (interaction, mut checkbox) in checkbox_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            checkbox.enabled = !checkbox.enabled;
            
            if let Ok(mut settings) = capture_settings.get_single_mut() {
                match checkbox.setting_type {
                    DeveloperCheckboxType::CopyToClipboard => {
                        settings.copy_to_clipboard = checkbox.enabled;
                    },
                    DeveloperCheckboxType::ShowInFinder => {
                        settings.show_in_finder = checkbox.enabled;
                    },
                    _ => {}
                }
            }
        }
    }
    
    // Handle hotkey recording
    if *hotkey_recording {
        if keyboard_input.just_pressed(KeyCode::Escape) {
            *hotkey_recording = false;
            info!("Hotkey recording cancelled");
        } else {
            let combination = capture_key_combination(&keyboard_input);
            if let Some(hotkey) = combination {
                if let Ok(mut settings) = capture_settings.get_single_mut() {
                    settings.hotkey = Some(hotkey);
                    *hotkey_recording = false;
                    info!("Window capture hotkey recorded successfully");
                }
            }
        }
    }
}

#[derive(Component)]
pub struct WindowCaptureButton {
    pub action: CaptureAction,
}

#[derive(Component)]
pub struct DeveloperCheckbox {
    pub setting_type: DeveloperCheckboxType,
    pub enabled: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CaptureAction {
    RecordHotkey,
    CaptureNow,
    OpenCaptureFolder,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DeveloperCheckboxType {
    CopyToClipboard,
    ShowInFinder,
    NodeProduction,
    FileLogging,
    AutoReload,
    DisablePopToRoot,
    DevelopmentMode,
    KeepWindowVisible,
    UseSystemProxy,
}

fn perform_window_capture() {
    // Platform-specific window capture implementation
    info!("Performing window capture");
}

fn open_folder_in_finder(path: &PathBuf) {
    // Platform-specific folder opening
    info!("Opening folder: {:?}", path);
}

fn capture_key_combination(keyboard_input: &Res<ButtonInput<KeyCode>>) -> Option<KeyCombination> {
    let mut modifiers = Vec::new();
    let mut key = None;
    
    // Capture modifier keys
    if keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight) {
        modifiers.push(Modifier::Control);
    }
    if keyboard_input.pressed(KeyCode::SuperLeft) || keyboard_input.pressed(KeyCode::SuperRight) {
        modifiers.push(Modifier::Command);
    }
    if keyboard_input.pressed(KeyCode::AltLeft) || keyboard_input.pressed(KeyCode::AltRight) {
        modifiers.push(Modifier::Alt);
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
        modifiers.push(Modifier::Shift);
    }
    
    // Capture regular key
    for keycode in keyboard_input.get_just_pressed() {
        if !matches!(keycode, 
            KeyCode::ControlLeft | KeyCode::ControlRight |
            KeyCode::SuperLeft | KeyCode::SuperRight |
            KeyCode::AltLeft | KeyCode::AltRight |
            KeyCode::ShiftLeft | KeyCode::ShiftRight |
            KeyCode::Escape
        ) {
            key = Some(*keycode);
            break;
        }
    }
    
    if let Some(key_code) = key {
        Some(KeyCombination { modifiers, key: key_code })
    } else {
        None
    }
}
```

### Custom Wallpaper System
```rust
fn custom_wallpaper_system(
    mut wallpaper_configs: Query<&mut CustomWallpaperConfig>,
    mut file_selectors: Query<(&Interaction, &mut FileSelector), Changed<Interaction>>,
    mut drag_drop_events: EventReader<FileDragAndDrop>,
    asset_server: Res<AssetServer>,
    mut events: EventWriter<DeveloperToolsEvent>,
    mut images: ResMut<Assets<Image>>,
) {
    // Handle file selector interactions
    for (interaction, mut file_selector) in file_selectors.iter_mut() {
        if *interaction == Interaction::Pressed && file_selector.label == "Select File" {
            // Open file picker dialog
            if let Some(path) = open_file_dialog(&file_selector.file_types) {
                file_selector.current_path = Some(path.clone());
                
                // Load and apply wallpaper
                let wallpaper_handle = asset_server.load(path.clone());
                
                if let Ok(mut config) = wallpaper_configs.get_single_mut() {
                    config.current_wallpaper = Some(wallpaper_handle);
                    config.wallpaper_path = Some(path.clone());
                }
                
                events.send(DeveloperToolsEvent::WallpaperSelected(path));
            }
        }
    }
    
    // Handle drag and drop
    for event in drag_drop_events.read() {
        match event {
            FileDragAndDrop::DroppedFile { path_buf, .. } => {
                if is_image_file(path_buf) {
                    let wallpaper_handle = asset_server.load(path_buf.clone());
                    
                    if let Ok(mut config) = wallpaper_configs.get_single_mut() {
                        config.current_wallpaper = Some(wallpaper_handle);
                        config.wallpaper_path = Some(path_buf.clone());
                    }
                    
                    events.send(DeveloperToolsEvent::WallpaperSelected(path_buf.clone()));
                }
            },
            _ => {}
        }
    }
}

fn open_file_dialog(file_types: &[FileType]) -> Option<PathBuf> {
    // Platform-specific file dialog implementation
    // This would use native file dialogs (NSOpenPanel on macOS, etc.)
    None
}

fn is_image_file(path: &PathBuf) -> bool {
    if let Some(extension) = path.extension() {
        matches!(extension.to_str(), Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("gif"))
    } else {
        false
    }
}
```

### Developer Tools Configuration System
```rust
fn developer_tools_system(
    mut tools_state: Query<&mut DeveloperToolsState>,
    mut checkbox_query: Query<(&Interaction, &mut DeveloperCheckbox), Changed<Interaction>>,
    mut events: EventWriter<DeveloperToolsEvent>,
    mut file_watchers: Local<HashMap<PathBuf, FileWatcher>>,
) {
    // Handle developer checkbox toggles
    for (interaction, mut checkbox) in checkbox_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            checkbox.enabled = !checkbox.enabled;
            
            if let Ok(mut state) = tools_state.get_single_mut() {
                match checkbox.setting_type {
                    DeveloperCheckboxType::NodeProduction => {
                        state.use_node_production = checkbox.enabled;
                        if checkbox.enabled {
                            std::env::set_var("NODE_ENV", "production");
                        } else {
                            std::env::set_var("NODE_ENV", "development");
                        }
                    },
                    DeveloperCheckboxType::FileLogging => {
                        state.use_file_logging = checkbox.enabled;
                        configure_logging(checkbox.enabled);
                    },
                    DeveloperCheckboxType::AutoReload => {
                        state.auto_reload_enabled = checkbox.enabled;
                        if checkbox.enabled {
                            setup_file_watchers(&mut file_watchers);
                        } else {
                            clear_file_watchers(&mut file_watchers);
                        }
                    },
                    DeveloperCheckboxType::DisablePopToRoot => {
                        state.disable_pop_to_root = checkbox.enabled;
                    },
                    DeveloperCheckboxType::DevelopmentMode => {
                        state.development_mode = checkbox.enabled;
                        events.send(DeveloperToolsEvent::DevelopmentModeToggled(checkbox.enabled));
                    },
                    DeveloperCheckboxType::KeepWindowVisible => {
                        state.keep_window_visible = checkbox.enabled;
                    },
                    _ => {}
                }
            }
        }
    }
}

fn configure_logging(file_logging_enabled: bool) {
    if file_logging_enabled {
        // Configure file logging
        info!("Switching to file logging");
    } else {
        // Configure OSLog
        info!("Switching to OSLog");
    }
}

fn setup_file_watchers(watchers: &mut HashMap<PathBuf, FileWatcher>) {
    // Setup file system watchers for hot reload
    let watch_paths = vec![
        PathBuf::from("./src"),
        PathBuf::from("./extensions"),
        PathBuf::from("./plugins"),
    ];
    
    for path in watch_paths {
        if path.exists() {
            watchers.insert(path.clone(), FileWatcher {
                path,
                recursive: true,
                patterns: vec!["*.rs".to_string(), "*.js".to_string(), "*.ts".to_string()],
                last_modified: std::time::SystemTime::now(),
            });
        }
    }
    
    info!("File watchers setup for {} paths", watchers.len());
}

fn clear_file_watchers(watchers: &mut HashMap<PathBuf, FileWatcher>) {
    watchers.clear();
    info!("File watchers cleared");
}
```

### Network Configuration System
```rust
fn network_proxy_system(
    mut proxy_configs: Query<&mut NetworkProxyConfig>,
    mut checkbox_query: Query<(&Interaction, &mut DeveloperCheckbox), Changed<Interaction>>,
    mut dropdown_query: Query<(&Interaction, &ProxyConfigDropdown), Changed<Interaction>>,
    mut events: EventWriter<NetworkEvent>,
    mut network_diagnostics: ResMut<NetworkDiagnostics>,
) {
    // Handle proxy configuration checkboxes
    for (interaction, mut checkbox) in checkbox_query.iter_mut() {
        if *interaction == Interaction::Pressed && checkbox.setting_type == DeveloperCheckboxType::UseSystemProxy {
            checkbox.enabled = !checkbox.enabled;
            
            if let Ok(mut config) = proxy_configs.get_single_mut() {
                config.use_system_proxy = checkbox.enabled;
                
                if checkbox.enabled {
                    // Load system proxy settings
                    load_system_proxy_settings(&mut config);
                } else {
                    // Clear proxy settings
                    config.custom_proxy_url = None;
                    config.proxy_authentication = None;
                }
                
                events.send(NetworkEvent::NetworkConfigurationSaved);
                
                // Show restart warning
                events.send(DeveloperToolsEvent::SystemRestartRequired(
                    "Proxy settings changes require a restart to take effect".to_string()
                ));
            }
        }
    }
    
    // Handle certificate store dropdown
    for (interaction, dropdown) in dropdown_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut config) = proxy_configs.get_single_mut() {
                config.certificate_store = dropdown.certificate_store.clone();
                
                // Validate certificate store access
                match &config.certificate_store {
                    CertificateStore::Keychain => {
                        validate_keychain_access(&mut network_diagnostics);
                    },
                    CertificateStore::System => {
                        validate_system_certificates(&mut network_diagnostics);
                    },
                    CertificateStore::Custom(path) => {
                        validate_custom_certificate_store(path, &mut network_diagnostics);
                    },
                }
                
                events.send(NetworkEvent::NetworkConfigurationSaved);
                events.send(DeveloperToolsEvent::SystemRestartRequired(
                    "Certificate store changes require a restart".to_string()
                ));
            }
        }
    }
}

#[derive(Component)]
pub struct ProxyConfigDropdown {
    pub certificate_store: CertificateStore,
}

fn load_system_proxy_settings(config: &mut NetworkProxyConfig) {
    // Platform-specific system proxy detection
    info!("Loading system proxy settings");
    
    // This would use platform APIs to detect proxy settings
    // macOS: SCDynamicStoreCopyProxies
    // Windows: WinINet API
    // Linux: Environment variables and desktop settings
}

fn validate_keychain_access(diagnostics: &mut NetworkDiagnostics) {
    // Test macOS Keychain access
    diagnostics.ssl_certificates.clear();
    
    // This would use Security framework to access Keychain
    info!("Validating Keychain certificate access");
}

fn validate_system_certificates(diagnostics: &mut NetworkDiagnostics) {
    // Test system certificate store access
    info!("Validating system certificate access");
}

fn validate_custom_certificate_store(path: &str, diagnostics: &mut NetworkDiagnostics) {
    // Validate custom certificate store
    info!("Validating custom certificate store at: {}", path);
}
```

### UI Layout System for Developer Tools
```rust
fn spawn_developer_tools_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(24.0)),
            overflow: Overflow::clip_y(), // Enable vertical scrolling
            ..default()
        })
        .with_children(|parent| {
            // Window Capture Section
            spawn_window_capture_section(parent);
            
            // Custom Wallpaper Section
            spawn_wallpaper_section(parent);
            
            // Developer Tools Section
            spawn_developer_tools_section(parent);
            
            // Network Configuration Section
            spawn_network_config_section(parent);
        });
}

fn spawn_window_capture_section(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(32.0)),
        max_width: Val::Px(600.0), // Constrain width
        ..default()
    }).with_children(|parent| {
        // Section title
        parent.spawn((
            Text::new("Window Capture"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));
        
        // Description
        parent.spawn((
            Text::new("Capture the Raycast window to share it or add a screenshot of your extension to the Store."),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                max_width: Val::Px(500.0), // Prevent text overflow
                ..default()
            },
        ));
        
        // Record hotkey button
        spawn_record_hotkey_button(parent);
        
        // Capture options checkboxes
        parent.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            margin: UiRect::top(Val::Px(16.0)),
            row_gap: Val::Px(8.0),
            ..default()
        }).with_children(|parent| {
            spawn_developer_checkbox(parent, "Copy to Clipboard", DeveloperCheckboxType::CopyToClipboard, true);
            spawn_developer_checkbox(parent, "Show in Finder", DeveloperCheckboxType::ShowInFinder, false);
        });
    });
}

fn spawn_wallpaper_section(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(32.0)),
        max_width: Val::Px(600.0),
        ..default()
    }).with_children(|parent| {
        // Section title
        parent.spawn((
            Text::new("Custom Wallpaper"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ));
        
        // File selector button
        parent.spawn((
            Node {
                width: Val::Px(120.0),
                height: Val::Px(36.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0, // Prevent expansion
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            Button,
            Interaction::default(),
            FileSelector {
                label: "Select File".to_string(),
                current_path: None,
                file_types: vec![
                    FileType {
                        name: "Images".to_string(),
                        extensions: vec!["png".to_string(), "jpg".to_string(), "jpeg".to_string()],
                    },
                ],
                multiple_selection: false,
                directory_mode: false,
            },
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Select File"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn spawn_developer_tools_section(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(32.0)),
        max_width: Val::Px(600.0),
        ..default()
    }).with_children(|parent| {
        // Section title
        parent.spawn((
            Text::new("Developer Tools"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ));
        
        // Developer checkboxes
        parent.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            ..default()
        }).with_children(|parent| {
            spawn_developer_checkbox(parent, "Use Node production environment", DeveloperCheckboxType::NodeProduction, true);
            spawn_developer_checkbox(parent, "Use file logging instead of OSLog", DeveloperCheckboxType::FileLogging, true);
            spawn_developer_checkbox(parent, "Auto-reload on save", DeveloperCheckboxType::AutoReload, true);
            spawn_developer_checkbox(parent, "Disable pop to root search", DeveloperCheckboxType::DisablePopToRoot, false);
            spawn_developer_checkbox(parent, "Open Raycast in development mode", DeveloperCheckboxType::DevelopmentMode, true);
            spawn_developer_checkbox(parent, "Keep window always visible during development", DeveloperCheckboxType::KeepWindowVisible, true);
        });
    });
}

fn spawn_network_config_section(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(32.0)),
        max_width: Val::Px(600.0),
        ..default()
    }).with_children(|parent| {
        // Section title
        parent.spawn((
            Text::new("Proxy and Certificate Settings"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));
        
        // Warning message
        parent.spawn((
            Text::new("If you change your system's proxy settings or Keychain certificates later, you need to restart Raycast."),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.8, 0.6, 0.2)), // Warning color
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                max_width: Val::Px(500.0),
                ..default()
            },
        ));
        
        // Proxy settings
        spawn_developer_checkbox(parent, "Use System Network Settings", DeveloperCheckboxType::UseSystemProxy, true);
        
        // Certificate store dropdown
        spawn_certificate_store_dropdown(parent);
    });
}

fn spawn_developer_checkbox(
    parent: &mut ChildBuilder,
    label: &str,
    setting_type: DeveloperCheckboxType,
    default_enabled: bool,
) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(12.0),
        ..default()
    }).with_children(|parent| {
        // Checkbox
        parent.spawn((
            Node {
                width: Val::Px(18.0),
                height: Val::Px(18.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                ..default()
            },
            BackgroundColor(if default_enabled { 
                Color::srgb(0.2, 0.6, 1.0) 
            } else { 
                Color::srgb(0.15, 0.15, 0.15) 
            }),
            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
            Button,
            Interaction::default(),
            DeveloperCheckbox {
                setting_type,
                enabled: default_enabled,
            },
        )).with_children(|parent| {
            if default_enabled {
                parent.spawn((
                    Text::new("✓"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            }
        });
        
        // Label
        parent.spawn((
            Text::new(label),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

fn spawn_record_hotkey_button(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Px(140.0),
            height: Val::Px(36.0),
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0, // Prevent expansion
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        Button,
        Interaction::default(),
        WindowCaptureButton {
            action: CaptureAction::RecordHotkey,
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Record Hotkey"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

fn spawn_certificate_store_dropdown(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        margin: UiRect::top(Val::Px(16.0)),
        ..default()
    }).with_children(|parent| {
        // Label
        parent.spawn((
            Text::new("Certificates"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Dropdown
        parent.spawn((
            Node {
                width: Val::Px(120.0),
                height: Val::Px(36.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0, // Prevent expansion
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            Button,
            Interaction::default(),
            ProxyConfigDropdown {
                certificate_store: CertificateStore::Keychain,
            },
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Keychain"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}
```

### SystemSet Organization for Developer Tools
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum DeveloperToolsSystems {
    Input,
    FileManagement,
    NetworkConfig,
    WindowCapture,
    Development,
    Validation,
    UI,
}

impl Plugin for DeveloperToolsPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<WindowCaptureSettings>()
            .register_type::<CustomWallpaperConfig>()
            .register_type::<DeveloperToolsState>()
            .register_type::<NetworkProxyConfig>()
            .register_type::<FileSelector>()
            
            .init_resource::<DeveloperEnvironment>()
            .init_resource::<NetworkDiagnostics>()
            
            .add_event::<DeveloperToolsEvent>()
            .add_event::<NetworkEvent>()
            
            .configure_sets(Update, (
                DeveloperToolsSystems::Input,
                DeveloperToolsSystems::FileManagement,
                DeveloperToolsSystems::Development,
                DeveloperToolsSystems::WindowCapture,
                DeveloperToolsSystems::NetworkConfig,
                DeveloperToolsSystems::Validation,
                DeveloperToolsSystems::UI,
            ).chain())
            
            .add_systems(Update, (
                // Input handling
                (
                    keyboard_input_system,
                    mouse_interaction_system,
                    file_drag_drop_system,
                ).in_set(DeveloperToolsSystems::Input),
                
                // File management
                (
                    file_selector_system,
                    custom_wallpaper_system,
                    file_watcher_system,
                ).in_set(DeveloperToolsSystems::FileManagement),
                
                // Development tools
                (
                    developer_tools_system,
                    hot_reload_system,
                    logging_configuration_system,
                ).in_set(DeveloperToolsSystems::Development),
                
                // Window capture
                (
                    window_capture_system,
                    capture_hotkey_system,
                ).in_set(DeveloperToolsSystems::WindowCapture),
                
                // Network configuration
                (
                    network_proxy_system,
                    certificate_validation_system,
                    network_diagnostics_system,
                ).in_set(DeveloperToolsSystems::NetworkConfig),
                
                // Validation and testing
                (
                    proxy_connection_test_system,
                    certificate_store_validation_system,
                ).in_set(DeveloperToolsSystems::Validation),
                
                // UI updates with Changed<T> optimization
                (
                    developer_checkbox_ui_system,
                    file_selector_ui_system,
                    network_status_ui_system,
                ).in_set(DeveloperToolsSystems::UI),
            ));
    }
}
```

This comprehensive Bevy implementation provides:

1. **Window capture system** with hotkey recording and configurable options
2. **Custom wallpaper support** with drag-and-drop file selection
3. **Developer tools configuration** with checkbox controls for all development settings
4. **Network proxy and certificate management** with system integration
5. **File management systems** with watchers for hot reload functionality
6. **Flex-based layouts** with proper constraints preventing UI expansion
7. **Component-driven architecture** with full reflection support for debugging
8. **Event-driven patterns** for all configuration changes and developer workflows
9. **Query optimization** using `Changed<T>` filters for efficient performance
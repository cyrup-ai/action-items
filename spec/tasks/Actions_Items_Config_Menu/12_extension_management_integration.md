# Task 12: Extension Management System Integration

## Objective
Implement integration between the configuration interface and the broader extension management system for extension installation, updates, removal, and lifecycle management with the configuration UI.

## Implementation Details

### Target Files
- `core/src/extensions/manager.rs:1-400` - Core extension management system
- `ui/src/ui/components/config/extension_lifecycle.rs:1-250` - UI for extension operations
- `core/src/extensions/installer.rs:1-300` - Extension installation and update system
- `core/src/extensions/metadata_sync.rs:1-180` - Extension metadata synchronization

### Bevy Implementation Patterns

#### Extension Management Resource
**Reference**: `./docs/bevy/examples/ecs/resources.rs:400-450` - Extension management state
**Reference**: `./docs/bevy/examples/async_tasks/async_compute.rs:320-370` - Async extension operations
```rust
// Extension management system resource
#[derive(Resource, Clone, Debug)]
pub struct ExtensionManager {
    pub installed_extensions: HashMap<String, InstalledExtension>,
    pub available_updates: HashMap<String, UpdateInfo>,
    pub installation_queue: Vec<InstallationTask>,
    pub operation_status: HashMap<String, OperationStatus>,
    pub metadata_cache: ExtensionMetadataCache,
}

#[derive(Debug, Clone)]
pub struct InstalledExtension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub icon_path: String,
    pub manifest: ExtensionManifest,
    pub installation_path: PathBuf,
    pub last_updated: DateTime<Utc>,
    pub enabled: bool,
    pub commands: Vec<ExtensionCommand>,
}

#[derive(Debug, Clone)]
pub struct ExtensionCommand {
    pub id: String,
    pub name: String,
    pub alias: Option<String>,
    pub hotkey: Option<String>,
    pub enabled: bool,
    pub action_type: ActionType,
    pub metadata: CommandMetadata,
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub available_version: String,
    pub changelog: String,
    pub update_size: u64,
    pub update_url: String,
    pub is_critical: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Idle,
    Installing { progress: f32 },
    Updating { progress: f32 },
    Removing,
    Failed { error: String },
    Completed,
}

impl ExtensionManager {
    pub fn new() -> Self {
        Self {
            installed_extensions: HashMap::new(),
            available_updates: HashMap::new(),
            installation_queue: Vec::new(),
            operation_status: HashMap::new(),
            metadata_cache: ExtensionMetadataCache::new(),
        }
    }
    
    pub async fn install_extension(&mut self, extension_url: String) -> Result<String, ExtensionError> {
        let installation_id = Uuid::new_v4().to_string();
        
        self.operation_status.insert(
            installation_id.clone(),
            OperationStatus::Installing { progress: 0.0 }
        );
        
        // Queue installation task
        self.installation_queue.push(InstallationTask {
            id: installation_id.clone(),
            operation: ExtensionOperation::Install { url: extension_url },
            priority: TaskPriority::Normal,
        });
        
        Ok(installation_id)
    }
    
    pub async fn update_extension(&mut self, extension_id: String) -> Result<(), ExtensionError> {
        if let Some(update_info) = self.available_updates.get(&extension_id) {
            self.operation_status.insert(
                extension_id.clone(),
                OperationStatus::Updating { progress: 0.0 }
            );
            
            self.installation_queue.push(InstallationTask {
                id: extension_id.clone(),
                operation: ExtensionOperation::Update {
                    extension_id: extension_id.clone(),
                    new_version: update_info.available_version.clone(),
                },
                priority: if update_info.is_critical {
                    TaskPriority::High
                } else {
                    TaskPriority::Normal
                },
            });
            
            Ok(())
        } else {
            Err(ExtensionError::NoUpdateAvailable)
        }
    }
    
    pub fn remove_extension(&mut self, extension_id: String) -> Result<(), ExtensionError> {
        if self.installed_extensions.contains_key(&extension_id) {
            self.operation_status.insert(
                extension_id.clone(),
                OperationStatus::Removing
            );
            
            self.installation_queue.push(InstallationTask {
                id: extension_id.clone(),
                operation: ExtensionOperation::Remove { extension_id },
                priority: TaskPriority::Normal,
            });
            
            Ok(())
        } else {
            Err(ExtensionError::ExtensionNotFound)
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstallationTask {
    pub id: String,
    pub operation: ExtensionOperation,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone)]
pub enum ExtensionOperation {
    Install { url: String },
    Update { extension_id: String, new_version: String },
    Remove { extension_id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}
```

#### Extension Installation System
**Reference**: `./docs/bevy/examples/async_tasks/async_compute.rs:400-450` - Background task processing
```rust
// Extension installation processing system
fn extension_installation_system(
    mut extension_manager: ResMut<ExtensionManager>,
    mut installation_events: EventWriter<ExtensionInstallationEvent>,
    mut commands: Commands,
) {
    // Process installation queue
    if let Some(task) = extension_manager.installation_queue.pop() {
        match task.operation {
            ExtensionOperation::Install { url } => {
                commands.spawn_task(async move {
                    install_extension_async(url, task.id).await
                });
            }
            ExtensionOperation::Update { extension_id, new_version } => {
                commands.spawn_task(async move {
                    update_extension_async(extension_id, new_version, task.id).await
                });
            }
            ExtensionOperation::Remove { extension_id } => {
                commands.spawn_task(async move {
                    remove_extension_async(extension_id, task.id).await
                });
            }
        }
    }
}

// Async extension installation
async fn install_extension_async(url: String, task_id: String) -> Result<InstalledExtension, ExtensionError> {
    // Download extension package
    let package_data = download_extension_package(&url).await?;
    
    // Validate extension manifest
    let manifest = validate_extension_manifest(&package_data)?;
    
    // Extract extension files
    let installation_path = extract_extension_files(&package_data, &manifest).await?;
    
    // Register extension
    let installed_extension = InstalledExtension {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        author: manifest.author.clone(),
        description: manifest.description.clone(),
        icon_path: installation_path.join(&manifest.icon).to_string_lossy().to_string(),
        manifest: manifest.clone(),
        installation_path,
        last_updated: Utc::now(),
        enabled: false, // Start disabled by default
        commands: manifest.commands.into_iter().map(|cmd| ExtensionCommand {
            id: cmd.id,
            name: cmd.name,
            alias: cmd.default_alias,
            hotkey: None,
            enabled: false,
            action_type: cmd.action_type,
            metadata: cmd.metadata,
        }).collect(),
    };
    
    Ok(installed_extension)
}

// Extension package download
async fn download_extension_package(url: &str) -> Result<Vec<u8>, ExtensionError> {
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    } else {
        Err(ExtensionError::DownloadFailed(response.status()))
    }
}

// Extension manifest validation
fn validate_extension_manifest(package_data: &[u8]) -> Result<ExtensionManifest, ExtensionError> {
    // Extract and parse manifest.json
    let archive = zip::ZipArchive::new(std::io::Cursor::new(package_data))?;
    
    // Find and read manifest
    let manifest_data = extract_file_from_archive(&archive, "manifest.json")?;
    let manifest: ExtensionManifest = serde_json::from_slice(&manifest_data)?;
    
    // Validate manifest fields
    if manifest.id.is_empty() || manifest.name.is_empty() || manifest.version.is_empty() {
        return Err(ExtensionError::InvalidManifest("Missing required fields".to_string()));
    }
    
    // Validate version format
    if !is_valid_semver(&manifest.version) {
        return Err(ExtensionError::InvalidManifest("Invalid version format".to_string()));
    }
    
    Ok(manifest)
}
```

### Extension Update System

#### Update Detection and Management
**Reference**: `./docs/bevy/examples/time/timers.rs:80-120` - Periodic update checking
```rust
// Extension update checking system
#[derive(Component)]
pub struct UpdateChecker {
    pub check_timer: Timer,
    pub last_check: DateTime<Utc>,
}

fn extension_update_system(
    mut update_checker: Query<&mut UpdateChecker>,
    mut extension_manager: ResMut<ExtensionManager>,
    mut update_events: EventWriter<UpdateAvailableEvent>,
    time: Res<Time>,
) {
    for mut checker in update_checker.iter_mut() {
        checker.check_timer.tick(time.delta());
        
        if checker.check_timer.just_finished() {
            // Check for updates for all installed extensions
            for (extension_id, extension) in &extension_manager.installed_extensions {
                // Spawn async task to check for updates
                let extension_id = extension_id.clone();
                let current_version = extension.version.clone();
                
                // This would be handled by the async system
                // check_extension_update(extension_id, current_version).await
            }
            
            checker.last_check = Utc::now();
        }
    }
}

// Async update checking
async fn check_extension_update(extension_id: String, current_version: String) -> Option<UpdateInfo> {
    // Check extension repository for updates
    let update_url = format!("https://extensions.repo/api/check/{}", extension_id);
    
    if let Ok(response) = reqwest::get(&update_url).await {
        if let Ok(update_data) = response.json::<UpdateResponse>().await {
            if update_data.version != current_version {
                return Some(UpdateInfo {
                    current_version,
                    available_version: update_data.version,
                    changelog: update_data.changelog,
                    update_size: update_data.size,
                    update_url: update_data.download_url,
                    is_critical: update_data.critical,
                });
            }
        }
    }
    
    None
}

#[derive(Deserialize)]
pub struct UpdateResponse {
    pub version: String,
    pub changelog: String,
    pub size: u64,
    pub download_url: String,
    pub critical: bool,
}
```

### Configuration UI Integration

#### Extension Management UI Components
**Reference**: `./docs/bevy/examples/ui/ui.rs:1600-1650` - Extension management interface
```rust
// Extension management UI integration
#[derive(Component)]
pub struct ExtensionManagementPanel {
    pub show_available_updates: bool,
    pub show_installation_progress: bool,
}

// Extension operation buttons
#[derive(Component)]
pub struct ExtensionActionButton {
    pub extension_id: String,
    pub action: ExtensionAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionAction {
    Update,
    Remove,
    Enable,
    Disable,
    Configure,
    ViewDetails,
}

// Extension management UI system
fn extension_management_ui_system(
    mut interaction_query: Query<(&Interaction, &ExtensionActionButton), Changed<Interaction>>,
    mut extension_manager: ResMut<ExtensionManager>,
    mut action_events: EventWriter<ExtensionActionEvent>,
) {
    for (interaction, action_button) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match action_button.action {
                ExtensionAction::Update => {
                    if let Ok(_) = extension_manager.update_extension(action_button.extension_id.clone()) {
                        action_events.send(ExtensionActionEvent {
                            extension_id: action_button.extension_id.clone(),
                            action: ExtensionAction::Update,
                            result: ActionResult::Started,
                        });
                    }
                }
                ExtensionAction::Remove => {
                    if let Ok(_) = extension_manager.remove_extension(action_button.extension_id.clone()) {
                        action_events.send(ExtensionActionEvent {
                            extension_id: action_button.extension_id.clone(),
                            action: ExtensionAction::Remove,
                            result: ActionResult::Started,
                        });
                    }
                }
                ExtensionAction::Configure => {
                    action_events.send(ExtensionActionEvent {
                        extension_id: action_button.extension_id.clone(),
                        action: ExtensionAction::Configure,
                        result: ActionResult::Started,
                    });
                }
                _ => {
                    // Handle other actions
                }
            }
        }
    }
}
```

### Extension Metadata Synchronization

#### Real-time Extension Data Sync
**Reference**: `./docs/bevy/examples/ecs/change_detection.rs:140-180` - Change detection for extension updates
```rust
// Extension metadata synchronization system
fn extension_metadata_sync_system(
    mut extension_manager: ResMut<ExtensionManager>,
    mut table_state: ResMut<TableState>,
    mut sync_events: EventReader<ExtensionSyncEvent>,
) {
    for sync_event in sync_events.iter() {
        match sync_event {
            ExtensionSyncEvent::ExtensionInstalled { extension } => {
                // Add new extension to table data
                let extension_item = ExtensionItem {
                    id: extension.id.clone(),
                    name: extension.name.clone(),
                    extension_type: ExtensionType::Extension,
                    icon_path: extension.icon_path.clone(),
                    alias: None,
                    hotkey: None,
                    enabled: extension.enabled,
                    parent_id: None,
                    children: extension.commands.iter().map(|cmd| cmd.id.clone()).collect(),
                    metadata: ExtensionMetadata {
                        version: extension.version.clone(),
                        author: extension.author.clone(),
                        description: extension.description.clone(),
                    },
                };
                
                table_state.extensions.push(extension_item);
                
                // Add command items
                for command in &extension.commands {
                    let command_item = ExtensionItem {
                        id: command.id.clone(),
                        name: command.name.clone(),
                        extension_type: ExtensionType::Command,
                        icon_path: extension.icon_path.clone(),
                        alias: command.alias.clone(),
                        hotkey: command.hotkey.clone(),
                        enabled: command.enabled,
                        parent_id: Some(extension.id.clone()),
                        children: vec![],
                        metadata: ExtensionMetadata {
                            version: extension.version.clone(),
                            author: extension.author.clone(),
                            description: command.metadata.description.clone(),
                        },
                    };
                    
                    table_state.extensions.push(command_item);
                }
            }
            ExtensionSyncEvent::ExtensionRemoved { extension_id } => {
                // Remove extension and its commands from table data
                table_state.extensions.retain(|item| {
                    item.id != *extension_id && item.parent_id.as_ref() != Some(extension_id)
                });
            }
            ExtensionSyncEvent::ExtensionUpdated { extension } => {
                // Update extension data in table
                if let Some(item) = table_state.extensions.iter_mut().find(|item| item.id == extension.id) {
                    item.name = extension.name.clone();
                    item.metadata.version = extension.version.clone();
                    item.metadata.description = extension.description.clone();
                }
            }
        }
    }
}

#[derive(Event)]
pub enum ExtensionSyncEvent {
    ExtensionInstalled { extension: InstalledExtension },
    ExtensionRemoved { extension_id: String },
    ExtensionUpdated { extension: InstalledExtension },
}
```

### Error Handling and Recovery

#### Extension Operation Error Management
**Reference**: `./docs/bevy/examples/diagnostics/log_diagnostics.rs:60-100` - Error logging and user feedback
```rust
// Extension error handling system
fn extension_error_handling_system(
    mut error_events: EventReader<ExtensionErrorEvent>,
    mut notification_events: EventWriter<NotificationEvent>,
    mut extension_manager: ResMut<ExtensionManager>,
) {
    for error_event in error_events.iter() {
        match &error_event.error {
            ExtensionError::InstallationFailed { extension_id, reason } => {
                extension_manager.operation_status.insert(
                    extension_id.clone(),
                    OperationStatus::Failed { error: reason.clone() }
                );
                
                notification_events.send(NotificationEvent {
                    title: "Installation Failed".to_string(),
                    message: format!("Failed to install extension: {}", reason),
                    notification_type: NotificationType::Error,
                    duration: Some(Duration::from_secs(10)),
                });
            }
            ExtensionError::UpdateFailed { extension_id, reason } => {
                extension_manager.operation_status.insert(
                    extension_id.clone(),
                    OperationStatus::Failed { error: reason.clone() }
                );
                
                notification_events.send(NotificationEvent {
                    title: "Update Failed".to_string(),
                    message: format!("Failed to update extension: {}", reason),
                    notification_type: NotificationType::Error,
                    duration: Some(Duration::from_secs(10)),
                });
            }
            _ => {
                // Handle other error types
            }
        }
    }
}

#[derive(Event)]
pub struct ExtensionErrorEvent {
    pub extension_id: String,
    pub error: ExtensionError,
}

#[derive(Debug, Clone)]
pub enum ExtensionError {
    InstallationFailed { extension_id: String, reason: String },
    UpdateFailed { extension_id: String, reason: String },
    RemovalFailed { extension_id: String, reason: String },
    InvalidManifest(String),
    DownloadFailed(reqwest::StatusCode),
    NoUpdateAvailable,
    ExtensionNotFound,
}
```

### Event System Integration

#### Extension Management Events
**Reference**: `./docs/bevy/examples/ecs/event.rs:340-370` - Extension operation events
```rust
// Extension management events
#[derive(Event)]
pub struct ExtensionInstallationEvent {
    pub extension_id: String,
    pub installation_status: InstallationStatus,
}

#[derive(Event)]
pub struct ExtensionActionEvent {
    pub extension_id: String,
    pub action: ExtensionAction,
    pub result: ActionResult,
}

#[derive(Event)]
pub struct UpdateAvailableEvent {
    pub extension_id: String,
    pub update_info: UpdateInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallationStatus {
    Started,
    InProgress(f32), // Progress percentage
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionResult {
    Started,
    Completed,
    Failed(String),
}
```

### Architecture Notes

#### Component Structure
- **ExtensionManager**: Core extension lifecycle management
- **UpdateChecker**: Automatic update detection and notification
- **ExtensionMetadataCache**: Cached extension information for performance
- **ExtensionActionButton**: UI components for extension operations

#### Integration Strategy
- **Real-time Sync**: Extension changes immediately reflected in configuration UI
- **Async Operations**: Background installation/update processing
- **Progress Tracking**: Visual feedback for long-running operations
- **Error Recovery**: Comprehensive error handling with user feedback

#### Extension Lifecycle Management
- **Installation**: Download, validate, extract, and register extensions
- **Updates**: Detect, download, and apply extension updates
- **Removal**: Clean uninstall with dependency checking
- **Configuration Sync**: Automatic synchronization with configuration interface

### Quality Standards
- Robust error handling for all extension operations
- Secure extension installation with validation
- Efficient background processing for operations
- Real-time UI updates during extension lifecycle events
- Complete transaction logging for audit and debugging

### Integration Points
- Configuration table integration for extension display
- Detail panel integration for extension configuration
- Enable/disable system integration for extension state
- Search system integration for extension discovery
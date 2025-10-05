# AI Menu 3 - Local AI Models Data Structures

## Implementation Task: Ollama Integration and Local Model Management Data Architecture

### Architecture Overview
Implement comprehensive data structures for local AI model management, Ollama server integration, model installation tracking, and experimental feature management with feature flags.

### Core Components

#### Local Model Management System
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct LocalModelManager {
    pub ollama_config: OllamaConfiguration,
    pub installed_models: HashMap<String, InstalledModel>,
    pub model_registry: ModelRegistry,
    pub installation_queue: VecDeque<ModelInstallation>,
    pub resource_monitor: ModelResourceMonitor,
}

#[derive(Reflect, Clone)]
pub struct OllamaConfiguration {
    pub host: String,                    // "127.0.0.1:11434"
    pub port: u16,                       // 11434
    pub connection_timeout: Duration,
    pub api_version: String,
    pub authentication: Option<OllamaAuth>,
    pub tls_enabled: bool,
}

#[derive(Reflect, Clone)]
pub struct InstalledModel {
    pub model_id: String,
    pub name: String,
    pub version: String,
    pub size: u64,                       // Bytes
    pub install_date: SystemTime,
    pub last_used: SystemTime,
    pub usage_count: u64,
    pub performance_metrics: ModelPerformanceMetrics,
    pub capabilities: ModelCapabilities,
}
```

#### Model Installation System
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ModelInstallationManager {
    pub active_installations: HashMap<String, ModelInstallation>,
    pub installation_history: VecDeque<InstallationRecord>,
    pub download_queue: VecDeque<DownloadRequest>,
    pub registry_cache: ModelRegistryCache,
}

#[derive(Reflect, Clone)]
pub struct ModelInstallation {
    pub model_name: String,
    pub installation_id: String,
    pub status: InstallationStatus,
    pub progress: InstallationProgress,
    pub started_at: SystemTime,
    pub estimated_completion: Option<SystemTime>,
    pub error_message: Option<String>,
}

#[derive(Reflect)]
pub enum InstallationStatus {
    Queued,
    Downloading,
    Extracting,
    Verifying,
    Installing,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Reflect)]
pub struct InstallationProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub download_speed: f64,            // Bytes per second
    pub eta: Option<Duration>,
}
```

### Bevy Implementation References

#### Network Communication for Ollama
- **HTTP Client**: `docs/bevy/examples/async_tasks/async_compute.rs`
  - Asynchronous communication with Ollama REST API
  - Connection pooling and timeout handling
  - Error handling for network failures

#### Resource Management
- **Resource Systems**: `docs/bevy/examples/ecs/startup_system.rs`
  - Global local model manager initialization
  - Resource cleanup and lifecycle management
  - Shared state management across systems

#### Asset Loading for Models
- **Asset Loading**: `docs/bevy/examples/asset/asset_loading.rs`
  - Dynamic model loading and caching
  - Progress tracking for large model downloads
  - Hot-reloading for model updates

#### Event System for Installation
- **Event Handling**: `docs/bevy/examples/ecs/send_and_receive_events.rs`
  - Model installation progress events
  - Installation completion and error notifications
  - Real-time status updates

### Browser Extension Integration

#### Extension Communication System
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct BrowserExtensionManager {
    pub connection_status: ExtensionConnectionStatus,
    pub active_connections: HashMap<String, BrowserConnection>,
    pub context_extraction: ContextExtractionConfig,
    pub privacy_settings: ExtensionPrivacySettings,
    pub communication_protocol: ExtensionProtocol,
}

#[derive(Reflect)]
pub struct ExtensionConnectionStatus {
    pub is_connected: bool,
    pub last_connection: Option<SystemTime>,
    pub connection_count: u64,
    pub supported_browsers: Vec<SupportedBrowser>,
    pub protocol_version: String,
}

#[derive(Reflect)]
pub struct BrowserConnection {
    pub browser_id: String,
    pub browser_type: BrowserType,
    pub tab_count: u32,
    pub active_tab_context: Option<TabContext>,
    pub connection_established: SystemTime,
    pub last_activity: SystemTime,
}

#[derive(Reflect)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Arc,
    Other(String),
}

#[derive(Reflect)]
pub struct TabContext {
    pub url: String,
    pub title: String,
    pub content_type: ContentType,
    pub extracted_text: String,
    pub metadata: HashMap<String, String>,
    pub extraction_timestamp: SystemTime,
}
```

### Experimental Features System

#### Feature Flag Management
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ExperimentalFeaturesManager {
    pub feature_flags: HashMap<String, FeatureFlag>,
    pub user_preferences: ExperimentalPreferences,
    pub rollout_config: RolloutConfiguration,
    pub telemetry_collector: FeatureTelemetry,
}

#[derive(Reflect, Clone)]
pub struct FeatureFlag {
    pub feature_id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub rollout_percentage: f32,        // 0.0 to 100.0
    pub prerequisites: Vec<String>,     // Other feature IDs required
    pub risk_level: FeatureRiskLevel,
    pub experimental_phase: ExperimentalPhase,
}

#[derive(Reflect)]
pub enum ExperimentalPhase {
    Development,        // Early development phase
    Alpha,             // Internal testing
    Beta,              // Limited user testing
    ReleaseCandidate,  // Near production ready
    Deprecated,        // Being phased out
}

// Feature flags from specification:
#[derive(Reflect)]
pub struct SpecificFeatureFlags {
    pub auto_models: bool,              // ON in spec
    pub chat_branching: bool,           // ON in spec
    pub custom_providers: bool,         // OFF in spec
    pub mcp_http_servers: bool,         // ON in spec
    pub ai_extensions_ollama: bool,     // ON in spec
}
```

#### Feature Toggle UI Components
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FeatureToggleGroup {
    pub toggles: Vec<Entity>,
    pub group_title: String,
    pub group_description: String,
    pub layout: ToggleLayout,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FeatureToggle {
    pub feature_id: String,
    pub display_name: String,
    pub current_state: bool,
    pub toggle_style: ToggleStyle,
    pub info_button: Option<Entity>,
    pub interaction_state: InteractionState,
}

#[derive(Reflect)]
pub struct ToggleStyle {
    pub active_color: Color,            // Blue (#007AFF)
    pub inactive_color: Color,          // Gray (#8E8E93)
    pub toggle_size: Vec2,
    pub animation_duration: Duration,
}
```

### Model Performance Monitoring

#### Resource Monitoring System
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ModelResourceMonitor {
    pub memory_usage: HashMap<String, MemoryUsage>,
    pub cpu_usage: HashMap<String, CpuUsage>,
    pub gpu_usage: HashMap<String, GpuUsage>,
    pub disk_usage: DiskUsage,
    pub network_usage: NetworkUsage,
}

#[derive(Reflect)]
pub struct ModelPerformanceMetrics {
    pub inference_time_ms: f64,
    pub tokens_per_second: f64,
    pub memory_peak_mb: f64,
    pub cpu_utilization_percent: f64,
    pub gpu_utilization_percent: Option<f64>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
}

#[derive(Reflect)]
pub struct ModelCapabilities {
    pub max_context_length: usize,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub supports_vision: bool,
    pub supported_languages: Vec<String>,
    pub model_architecture: String,
    pub parameter_count: Option<u64>,
}
```

### Host Configuration and Connection

#### Ollama Host Management
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct OllamaHostManager {
    pub host_config: OllamaConfiguration,
    pub connection_pool: ConnectionPool,
    pub health_monitor: HostHealthMonitor,
    pub api_client: OllamaAPIClient,
}

#[derive(Reflect)]
pub struct HostHealthMonitor {
    pub is_healthy: bool,
    pub last_health_check: SystemTime,
    pub response_time_ms: f64,
    pub uptime: Duration,
    pub error_count: u32,
    pub consecutive_failures: u32,
}

#[derive(Reflect)]
pub struct OllamaAPIClient {
    pub base_url: String,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub authentication: Option<OllamaAuth>,
}
```

### Data Integration and Events

#### Model Management Events
```rust
#[derive(Event)]
pub struct ModelInstallationStarted {
    pub model_name: String,
    pub installation_id: String,
    pub estimated_size: u64,
}

#[derive(Event)]
pub struct ModelInstallationProgress {
    pub installation_id: String,
    pub progress: InstallationProgress,
}

#[derive(Event)]
pub struct ModelInstallationCompleted {
    pub installation_id: String,
    pub model: InstalledModel,
    pub installation_time: Duration,
}

#[derive(Event)]
pub struct OllamaConnectionStatusChanged {
    pub old_status: bool,
    pub new_status: bool,
    pub error_message: Option<String>,
}

#[derive(Event)]
pub struct FeatureFlagToggled {
    pub feature_id: String,
    pub old_state: bool,
    pub new_state: bool,
    pub user_initiated: bool,
}
```

### Privacy and Security Architecture

#### Privacy Controls
```rust
#[derive(Reflect)]
pub struct ExtensionPrivacySettings {
    pub auto_extract_context: bool,
    pub sensitive_content_filtering: bool,
    pub content_type_whitelist: Vec<ContentType>,
    pub domain_whitelist: Vec<String>,
    pub max_context_size: usize,
    pub retention_policy: ContextRetentionPolicy,
}

#[derive(Reflect)]
pub enum ContextRetentionPolicy {
    SessionOnly,           // Clear on session end
    Hours(u32),           // Clear after N hours
    Days(u32),            // Clear after N days
    Manual,               // User must clear manually
}
```

### Performance Optimization

#### Efficient Data Structures
- **HashMap Usage**: Fast lookups for models, features, and connections
- **VecDeque Queues**: Efficient queue operations for downloads and installations
- **Resource Pooling**: Connection pooling for Ollama API calls
- **Memory Management**: Automatic cleanup of completed installations

#### Caching Strategies
- **Model Registry Cache**: Cache model metadata to reduce API calls
- **Connection State Cache**: Cache connection status to avoid repeated checks
- **Feature Flag Cache**: Cache feature states for fast access
- **Context Cache**: Temporary caching of browser context data

### Testing Infrastructure

#### Data Model Testing
- **Serialization Testing**: Verify all data structures serialize correctly
- **Migration Testing**: Test data migration between versions
- **Performance Testing**: Benchmark data operations under load
- **Validation Testing**: Test all data validation rules

#### Integration Testing
- **Ollama Integration**: Test communication with real Ollama instances
- **Browser Extension**: Test browser communication protocol
- **Feature Toggle**: Test feature flag system behavior
- **Error Recovery**: Test recovery from various failure scenarios

### Implementation Files
- `ai_menu_3/local_models.rs` - Core local model data structures
- `ai_menu_3/ollama_types.rs` - Ollama-specific types and configuration
- `ai_menu_3/browser_extension.rs` - Browser extension communication types
- `ai_menu_3/experimental_features.rs` - Feature flag and experimental system
- `ai_menu_3/model_monitoring.rs` - Performance monitoring and metrics
- `ai_menu_3/events.rs` - Event definitions for local AI features

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all model monitoring loops
- **Blazing-fast performance** - efficient HashMap and VecDeque operations
- **Production quality** - complete, secure local AI architecture

## Bevy Implementation Details

### Component Architecture for Local AI Models
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct LocalModelsPanel {
    pub selected_model: Option<String>,
    pub installation_view: InstallationView,
    pub show_experimental_features: bool,
}

#[derive(Component, Reflect)]
pub struct ModelInstallationTask {
    pub model_name: String,
    pub task: Task<Result<InstalledModel, InstallationError>>,
    pub progress_tracker: InstallationProgress,
}
```

### System Architecture with Ordered Operations
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LocalModelsSystemSet {
    HostConnection,
    ModelManagement,
    InstallationTracking,
    PerformanceMonitoring,
    UIUpdate,
}

impl Plugin for LocalModelsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (
            LocalModelsSystemSet::HostConnection,
            LocalModelsSystemSet::ModelManagement,
            LocalModelsSystemSet::InstallationTracking,
            LocalModelsSystemSet::PerformanceMonitoring,
            LocalModelsSystemSet::UIUpdate,
        ).chain())
        .add_systems(Update, (
            monitor_ollama_connection.in_set(LocalModelsSystemSet::HostConnection),
            manage_model_lifecycle.in_set(LocalModelsSystemSet::ModelManagement),
            track_installation_progress.in_set(LocalModelsSystemSet::InstallationTracking),
            monitor_model_performance.in_set(LocalModelsSystemSet::PerformanceMonitoring),
            update_local_models_ui.in_set(LocalModelsSystemSet::UIUpdate),
        ));
    }
}
```

### Async Model Installation with Progress Tracking
```rust
fn spawn_model_installation(
    mut commands: Commands,
    task_pool: Res<AsyncComputeTaskPool>,
    mut model_manager: ResMut<LocalModelManager>,
    installation_requests: Query<&ModelInstallRequest, Added<ModelInstallRequest>>,
) {
    for request in &installation_requests {
        let model_name = request.model_name.clone();
        let task = task_pool.spawn(async move {
            install_model_from_ollama(model_name).await
        });
        
        commands.spawn(ModelInstallationTask {
            model_name: request.model_name.clone(),
            task,
            progress_tracker: InstallationProgress::default(),
        });
    }
}

fn poll_installation_tasks(
    mut commands: Commands,
    mut installation_tasks: Query<(Entity, &mut ModelInstallationTask)>,
    mut model_events: EventWriter<ModelInstallationCompleted>,
) {
    for (entity, mut task) in &mut installation_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.task)) {
            match result {
                Ok(model) => {
                    model_events.write(ModelInstallationCompleted {
                        installation_id: task.model_name.clone(),
                        model,
                        installation_time: Duration::from_secs(60), // placeholder
                    });
                }
                Err(error) => {
                    error!("Model installation failed: {:?}", error);
                }
            }
            commands.entity(entity).despawn();
        }
    }
}
```

### Feature Flag Management with ECS
```rust
fn handle_experimental_feature_toggles(
    mut feature_events: EventReader<FeatureFlagToggled>,
    mut experimental_manager: ResMut<ExperimentalFeaturesManager>,
    mut ui_events: EventWriter<UIUpdateEvent>,
) {
    for event in feature_events.read() {
        if let Some(feature) = experimental_manager.feature_flags.get_mut(&event.feature_id) {
            feature.enabled = event.new_state;
            
            ui_events.write(UIUpdateEvent::ExperimentalFeatureChanged {
                feature_id: event.feature_id.clone(),
                enabled: event.new_state,
            });
            
            info!("Experimental feature '{}' toggled to: {}", event.feature_id, event.new_state);
        }
    }
}
```

### Testing Strategy for Local Models
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_local_model_installation() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, LocalModelsPlugin));
        
        let model_request = ModelInstallRequest {
            model_name: "llama2:7b".to_string(),
            download_config: DownloadConfig::default(),
        };
        
        app.world_mut().spawn(model_request);
        app.update();
        
        // Verify installation task was spawned
        let installation_tasks: Vec<&ModelInstallationTask> = 
            app.world().query::<&ModelInstallationTask>().iter(app.world()).collect();
        assert_eq!(installation_tasks.len(), 1);
    }
}
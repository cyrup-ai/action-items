# AI Menu 3 - Model Installation System

## Implementation Task: Interactive Model Installation with Progress Tracking

### Architecture Overview
Implement comprehensive model installation system including model name input, download management, progress tracking, and installation status monitoring with real-time UI updates.

### Core Components

#### Model Installation Interface
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ModelInstallationInterface {
    pub model_input_field: Entity,       // Text input for model name
    pub download_button: Entity,         // Arrow down icon button
    pub status_display: Entity,          // "X models installed" text
    pub progress_indicator: Option<Entity>, // Progress bar when installing
    pub installation_queue: VecDeque<InstallationRequest>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ModelInputField {
    pub current_text: String,
    pub placeholder: String,             // "Enter a model name"
    pub input_focus: bool,
    pub validation_state: InputValidation,
    pub suggestion_list: Vec<ModelSuggestion>,
    pub cursor_position: usize,
}

#[derive(Reflect)]
pub enum InputValidation {
    Valid(String),                       // Valid model name
    Invalid(String),                     // Error message
    Checking,                           // Validating with registry
    Empty,                              // No input
}
```

#### Model Installation Pipeline
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ModelInstallationManager {
    pub active_installations: HashMap<String, ActiveInstallation>,
    pub installation_history: VecDeque<InstallationRecord>,
    pub registry_client: ModelRegistryClient,
    pub download_manager: DownloadManager,
    pub installation_policies: InstallationPolicies,
}

#[derive(Reflect)]
pub struct ActiveInstallation {
    pub installation_id: String,
    pub model_name: String,
    pub model_tag: String,
    pub status: InstallationStatus,
    pub progress: InstallationProgress,
    pub started_at: SystemTime,
    pub estimated_completion: Option<SystemTime>,
    pub download_info: DownloadInfo,
}

#[derive(Reflect)]
pub enum InstallationStatus {
    Queued,                             // Waiting to start
    Fetching,                          // Downloading model registry info
    Downloading(DownloadPhase),        // Active download with phase
    Verifying,                         // Checksumming downloaded files
    Installing,                        // Installing to Ollama
    Completed,                         // Successfully installed
    Failed(InstallationError),         // Installation failed
    Cancelled,                         // User cancelled
}

#[derive(Reflect)]
pub enum DownloadPhase {
    Manifest,                          // Downloading model manifest
    Layers(u32, u32),                  // Downloading layer X of Y
    Metadata,                          // Downloading model metadata
}
```

### Bevy Implementation References

#### Text Input with Suggestions
- **Text Input**: `docs/bevy/examples/input/text_input.rs`
  - Model name input field with auto-completion
  - Real-time validation and suggestion display
  - Input focus management and cursor positioning

#### Download Progress Tracking
- **Progress Systems**: `docs/bevy/examples/games/loading_screen.rs`
  - Real-time progress bar updates during downloads
  - Multiple progress indicators for different phases
  - Visual feedback for download completion

#### Button with Icon Integration
- **UI Buttons**: `docs/bevy/examples/ui/button.rs`
  - Download arrow button with hover states
  - Button state management (enabled/disabled/loading)
  - Icon integration within button components

#### Async Download Operations
- **Async Tasks**: `docs/bevy/examples/async_tasks/async_compute.rs`
  - Background model downloading without blocking UI
  - Concurrent downloads with proper resource management
  - Progress reporting from async tasks to UI

### Model Registry Integration

#### Registry Client Implementation
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ModelRegistryClient {
    pub registry_url: String,           // Ollama registry or custom
    pub http_client: HttpClient,
    pub cache: RegistryCache,
    pub search_engine: ModelSearchEngine,
}

#[derive(Reflect)]
pub struct ModelSearchEngine {
    pub indexed_models: HashMap<String, ModelInfo>,
    pub search_cache: HashMap<String, Vec<ModelSuggestion>>,
    pub popularity_scores: HashMap<String, f32>,
    pub last_index_update: SystemTime,
}

#[derive(Reflect)]
pub struct ModelSuggestion {
    pub model_name: String,
    pub full_name: String,              // e.g., "llama2:7b"
    pub description: String,
    pub size: u64,
    pub popularity: f32,
    pub download_count: u64,
    pub tags: Vec<String>,
}

#[derive(Reflect)]
pub struct ModelInfo {
    pub name: String,
    pub versions: Vec<ModelVersion>,
    pub description: String,
    pub author: String,
    pub license: String,
    pub architecture: String,
    pub parameters: u64,
    pub quantization: Option<String>,
}
```

#### Auto-completion System
- **Model Name Suggestions**: Real-time suggestions as user types
- **Popular Models**: Prioritize popular and verified models
- **Fuzzy Search**: Intelligent matching for partial model names
- **Category Filtering**: Support for model categories and tags

### Download Management System

#### Download Process Architecture
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DownloadManager {
    pub active_downloads: HashMap<String, Download>,
    pub download_queue: VecDeque<DownloadRequest>,
    pub bandwidth_manager: BandwidthManager,
    pub storage_manager: StorageManager,
    pub concurrent_limit: usize,        // Max simultaneous downloads
}

#[derive(Reflect)]
pub struct Download {
    pub download_id: String,
    pub model_name: String,
    pub url: String,
    pub total_size: u64,
    pub downloaded_bytes: u64,
    pub download_speed: f64,            // Bytes per second
    pub eta: Option<Duration>,
    pub chunks: Vec<DownloadChunk>,
    pub checksum: String,
}

#[derive(Reflect)]
pub struct DownloadProgress {
    pub percentage: f32,                // 0.0 to 100.0
    pub downloaded_mb: f64,
    pub total_mb: f64,
    pub download_speed_mbps: f64,
    pub time_remaining: Option<Duration>,
    pub current_phase: DownloadPhase,
}
```

#### Progressive Download Features
- **Chunked Downloads**: Split large models into manageable chunks
- **Resume Support**: Resume interrupted downloads automatically
- **Bandwidth Management**: Intelligent bandwidth allocation
- **Parallel Downloads**: Multiple concurrent downloads with limits

### Installation Progress Display

#### Visual Progress Components
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct InstallationProgressDisplay {
    pub progress_bar: Entity,
    pub status_text: Entity,
    pub speed_indicator: Entity,
    pub time_remaining: Entity,
    pub cancel_button: Option<Entity>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ProgressBar {
    pub current_progress: f32,          // 0.0 to 1.0
    pub bar_color: Color,
    pub background_color: Color,
    pub animation_speed: f32,
    pub style: ProgressBarStyle,
}

#[derive(Reflect)]
pub enum ProgressBarStyle {
    Determinate,                        // Shows exact progress
    Indeterminate,                      // Animated for unknown progress
    Segmented,                          // Shows phase-based progress
}
```

#### Status Display Updates
- **Real-time Status**: "Installing llama2:7b..." with current operation
- **Progress Percentage**: "Downloading: 45% (2.1GB/4.7GB)"
- **Speed Information**: "12.5 MB/s - 3 minutes remaining"
- **Phase Updates**: "Verifying checksums...", "Installing to Ollama..."

### Model Installation Workflow

#### Installation State Machine
1. **Input Validation**: Validate model name format and availability
2. **Registry Lookup**: Query model registry for metadata and download URLs
3. **Download Initiation**: Start downloading model files with progress tracking
4. **Verification**: Verify downloaded files against checksums
5. **Ollama Installation**: Install model into Ollama using API
6. **Completion**: Update UI and model count display

#### Error Handling and Recovery
```rust
#[derive(Event)]
pub struct ModelInstallationError {
    pub installation_id: String,
    pub model_name: String,
    pub error_type: InstallationErrorType,
    pub error_message: String,
    pub recovery_actions: Vec<RecoveryAction>,
}

#[derive(Reflect)]
pub enum InstallationErrorType {
    ModelNotFound,
    NetworkError,
    InsufficientSpace,
    ChecksumMismatch,
    OllamaUnavailable,
    PermissionDenied,
    CorruptedDownload,
}

#[derive(Reflect)]
pub enum RecoveryAction {
    Retry,
    RetryWithDifferentSource,
    ClearCacheAndRetry,
    FreeSpaceAndRetry,
    ContactSupport,
}
```

### UI Visual Implementation

#### Model Input Field Design
- **Background**: Dark theme input field (#2a2a2a)
- **Placeholder**: "Enter a model name" in gray text
- **Text Color**: White text for user input
- **Focus State**: Subtle border highlight
- **Suggestion Dropdown**: Dark dropdown with model suggestions

#### Download Button Implementation
- **Icon**: Arrow down symbol (↓) 
- **Position**: Right side of input field
- **States**: Normal, hover, disabled, loading
- **Animation**: Brief animation on successful trigger
- **Accessibility**: Proper ARIA labels for screen readers

#### Status Display Format
- **Format**: "X models installed via Ollama"
- **Style**: Medium gray text below input field
- **Dynamic Updates**: Real-time count updates during installations
- **Position**: Left-aligned under input field

### Performance Optimization

#### Download Performance
- **Concurrent Downloads**: Optimal number of simultaneous downloads
- **Connection Pooling**: Reuse HTTP connections for efficiency
- **Compression**: Support for compressed model transfers
- **Caching**: Cache frequently accessed models and metadata

#### UI Responsiveness
- **Background Processing**: All downloads run in background threads
- **Progressive Updates**: Smooth progress bar animations
- **Non-blocking UI**: UI remains responsive during installations
- **Memory Management**: Efficient memory usage for large model downloads

### Storage Management

#### Local Storage Architecture
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct StorageManager {
    pub storage_locations: Vec<StoragePath>,
    pub available_space: u64,
    pub storage_policies: StoragePolicies,
    pub cleanup_scheduler: CleanupScheduler,
}

#[derive(Reflect)]
pub struct StoragePolicies {
    pub max_cache_size: u64,
    pub auto_cleanup_enabled: bool,
    pub cleanup_threshold: f32,         // Percentage of storage used
    pub retain_popular_models: bool,
}
```

#### Disk Space Management
- **Space Validation**: Check available space before downloads
- **Auto-cleanup**: Remove unused models when space is low
- **User Control**: Allow user to manage model storage locations
- **Compression**: Use efficient storage formats for models

### Integration Points

#### Ollama Integration
- **API Coordination**: Coordinate with Ollama host configuration
- **Model Registration**: Register installed models with Ollama
- **Version Management**: Handle model updates and versioning
- **Resource Monitoring**: Monitor Ollama resource usage during installation

#### UI System Integration
- **Progress Updates**: Real-time UI updates during installation process
- **Error Display**: Integrate error messages with notification system
- **Status Synchronization**: Keep status displays synchronized across UI
- **Event Propagation**: Propagate installation events to interested systems

### Testing Requirements

#### Functionality Testing
- **Model Installation**: Test complete installation workflow for various models
- **Error Recovery**: Test recovery from various error conditions
- **Progress Tracking**: Verify accurate progress reporting throughout process
- **Concurrent Installations**: Test multiple simultaneous installations

#### Performance Testing
- **Large Model Downloads**: Test with large models (>10GB)
- **Network Conditions**: Test under various network conditions
- **Resource Usage**: Monitor CPU, memory, and disk usage during installations
- **UI Responsiveness**: Verify UI remains responsive during heavy operations

### Implementation Files
- `ai_menu_3/model_installation.rs` - Core installation management system
- `ai_menu_3/model_registry.rs` - Model registry client and search functionality
- `ai_menu_3/download_manager.rs` - Download management and progress tracking
- `ai_menu_3/installation_ui.rs` - UI components for installation interface
- `ai_menu_3/storage_manager.rs` - Local storage and disk space management
- `ai_menu_3/installation_events.rs` - Installation-related event definitions

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all progress tracking loops
- **Blazing-fast performance** - efficient download and installation pipeline
- **Production quality** - reliable model installation with comprehensive error handling
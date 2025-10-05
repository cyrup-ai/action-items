# AI Menu 3 Specification

## Overview
AI Menu 3 represents the local AI model management and experimental features configuration interface. This view focuses on Ollama integration, local model hosting, browser extension connectivity, and cutting-edge AI features in development.

## Layout Architecture
- **Base Layout**: Consistent with previous AI menu specifications
- **Left Pane**: Standard branding and primary toggle section
- **Right Pane**: Local AI hosting and experimental feature configuration

## Right Pane Configuration Sections

### Ollama Host Configuration
- **Label**: "Ollama Host"
- **Layout**: Two-column layout with label left, controls right
- **Components**:
  - **Host Input Field**: "127.0.0.1:11434" (current value)
    - **Style**: Dark background input field with white text
    - **Width**: Spans majority of right column
  - **Info Icon**: Circular "i" button for Ollama host setup help
  - **Sync Models Button**: "Sync Models"
    - **Style**: Standard button with darker background
    - **Position**: Below host input field
- **Purpose**: Configure connection to local Ollama AI model server
- **Default**: Local loopback address with standard Ollama port (11434)

### Local Model Installation
- **Label**: "Install Ollama Model"
- **Layout**: Two-column layout with label left, controls right
- **Components**:
  - **Model Name Input**: Text input field
    - **Placeholder Text**: "Enter a model name"
    - **Style**: Dark background with placeholder text in gray
    - **Download Icon**: Arrow down icon positioned on right side of input field
  - **Status Display**: "5 models installed via Ollama"
    - **Style**: Medium gray text below input field
    - **Alignment**: Left-aligned under input
- **Functionality**: 
  - Install new models directly from Ollama registry via text input
  - Download icon indicates action trigger
  - Real-time status display of installed model count

### Advanced Integration Features
- **Title**: "Advanced"

#### Browser Extension Integration
- **Label**: "Browser Extension"
- **Layout**: Two-column layout with label left, content right
- **Components**:
  - **Description**: "Bring context from your browser tab into Raycast AI"
    - **Style**: White/light gray text
  - **Connection Status**: "Last successful connection on 8/6/2025, 5:30 PM"
    - **Style**: Medium gray text below description
    - **Format**: Timestamp includes date and time with AM/PM format
- **Functionality**:
  - Real-time browser tab context extraction
  - Connection health monitoring with specific timestamps
  - Status tracking for troubleshooting connectivity issues

### Experimental Features
- **Title**: "Experiments"
- **Description**: "New AI features in development. Your feedback will help us improve these experiments."
- **Layout**: Feature list with consistent two-column format

#### Feature Toggle List
1. **Auto Models**
   - **Current State**: ON (blue toggle, circle positioned right)
   - **Toggle Style**: iOS-style switch with blue active background
   - **Info Icon**: Circular "i" button on far right
   - **Layout**: Feature name left, toggle center-right, info icon far right

2. **Chat Branching**
   - **Current State**: ON (blue toggle, circle positioned right)
   - **Toggle Style**: iOS-style switch with blue active background
   - **Info Icon**: Circular "i" button on far right

3. **Custom Providers**
   - **Current State**: OFF (gray toggle, circle positioned left)
   - **Toggle Style**: iOS-style switch with gray inactive background
   - **Info Icon**: Circular "i" button on far right

4. **MCP HTTP Servers**
   - **Current State**: ON (blue toggle, circle positioned right)
   - **Toggle Style**: iOS-style switch with blue active background
   - **Info Icon**: Circular "i" button on far right

5. **AI Extensions for Ollama Models**
   - **Current State**: ON (blue toggle, circle positioned right)
   - **Toggle Style**: iOS-style switch with blue active background
   - **Info Icon**: Circular "i" button on far right

#### Toggle Interaction Specifications
- **Active State**: Blue background (#007AFF or similar), white circle positioned right
- **Inactive State**: Gray background (#8E8E93 or similar), white circle positioned left
- **Animation**: Smooth slide transition when toggling between states
- **Accessibility**: Each toggle has accessible labels for screen readers

## Visual Design Specifications

### Layout Architecture
- **Left Pane**: Identical to AI_Menu.md and AI_Menu_2.md specifications
- **Right Pane**: Ollama and experimental features configuration
- **Section Grouping**: Clear visual separation between Ollama, Advanced, and Experiments sections
- **Consistent Spacing**: Uniform vertical spacing between sections and elements

### Input Field Specifications
- **Ollama Host Field**: 
  - Dark background input with white text
  - Current value: "127.0.0.1:11434"
  - Full width of right column
- **Model Name Field**:
  - Dark background with gray placeholder text
  - Placeholder: "Enter a model name"
  - Download arrow icon on right side

### Button Specifications
- **Sync Models Button**: Standard button styling with darker background
- **Download Icon**: Arrow down icon within model input field
- **Consistent Styling**: All buttons match design system from other AI menu screens

### Status Text Styling
- **Model Count**: "5 models installed via Ollama"
  - Medium gray text, smaller font size
  - Positioned below model input field
- **Connection Status**: "Last successful connection on 8/6/2025, 5:30 PM"
  - Medium gray text with timestamp formatting
  - Clear date and time format with AM/PM indicator

### Toggle Switch System
- **Active Toggles** (Auto Models, Chat Branching, MCP HTTP Servers, AI Extensions):
  - Blue background (#007AFF or similar)
  - White circular toggle button positioned right
- **Inactive Toggle** (Custom Providers):
  - Gray background (#8E8E93 or similar)
  - White circular toggle button positioned left
- **Consistent Sizing**: All toggles maintain uniform dimensions
- **Smooth Animation**: Slide transition between on/off states

### Icon System
- **Info Icons**: Circular "i" buttons throughout interface
  - Consistent size and positioning
  - Right-aligned next to relevant controls
- **Download Icon**: Arrow down symbol within input field
- **Visual Hierarchy**: Icons support but don't compete with primary controls

### Typography Consistency
- **Section Headers**: Bold, white/light gray for "Experiments", "Advanced"
- **Feature Names**: Regular weight, white/light gray for toggle labels
- **Descriptions**: Medium gray for explanatory text
- **Status Information**: Smaller, medium gray text for timestamps and counts

### Color Scheme Integration
- **Background**: Dark theme consistent with other AI menu screens
- **Text Hierarchy**: White primary, light gray secondary, medium gray tertiary
- **Accent Colors**: Blue for active states, gray for inactive states
- **Border Colors**: Subtle dark borders for input field separation

## Functional Requirements

### Ollama Integration System
- **Host Discovery**: Automatic detection of local Ollama instances
- **Connection Management**: Persistent connection handling with retry logic
- **Model Registry**: Integration with Ollama model repository
- **Performance Monitoring**: Real-time monitoring of local model performance

### Local Model Management
- **Installation Pipeline**: Streamlined model download and installation
- **Version Control**: Support for multiple model versions
- **Resource Management**: Intelligent management of disk space and memory usage
- **Model Switching**: Seamless switching between installed models

### Browser Extension Architecture
- **Context Extraction**: Real-time extraction of browser tab content
- **Privacy Controls**: User control over what context is shared
- **Multi-browser Support**: Compatibility with major browser platforms
- **Secure Communication**: Encrypted communication between browser and application

### Experimental Feature System
- **Feature Flags**: Dynamic enabling/disabling of experimental features
- **Rollback Capability**: Safe rollback mechanism for problematic features
- **Telemetry Collection**: Anonymous usage data for feature improvement
- **Feedback Integration**: User feedback collection for experimental features

## Bevy Implementation Examples

### IP Address Input Field
- Reference: `./docs/bevy/examples/ui/text_input.rs` - IP address input validation
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Real-time input validation

### Model Installation Interface
- Reference: `./docs/bevy/examples/asset_loading/asset_loading.rs` - Progress tracking for model downloads
- Reference: `./docs/bevy/examples/games/loading_screen.rs` - Download progress visualization

### Toggle Switch Groups
- Reference: `./docs/bevy/examples/ui/ui.rs` - Multiple toggle switch layout
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Toggle switch state sprites

### Connection Status Display
- Reference: `./docs/bevy/examples/time/time.rs` - Timestamp formatting and display
- Reference: `./docs/bevy/examples/ui/text.rs` - Status text styling and updates

### Model Synchronization
- Reference: `./docs/bevy/examples/async_tasks/async_compute.rs` - Asynchronous model synchronization
- Reference: `./docs/bevy/examples/animation/animated_fox.rs` - Sync button animations

### Download Progress Indicators
- Reference: `./docs/bevy/examples/ui/ui.rs` - Progress bar implementation
- Reference: `./docs/bevy/examples/animation/custom_skinned_mesh.rs` - Animated progress indicators

### Feature Flag Management
- Reference: `./docs/bevy/examples/reflection/reflection.rs` - Dynamic feature flag system
- Reference: `./docs/bevy/examples/app/return_after_run.rs` - Feature state persistence

## Security and Privacy Requirements

### Local Model Security
- **Sandboxed Execution**: Local models run in isolated environment
- **Resource Limits**: Strict memory and CPU usage limits
- **Network Isolation**: Optional network isolation for sensitive models
- **Audit Logging**: Comprehensive logging of model usage and access

### Browser Extension Security
- **Permission Model**: Explicit user consent for context access
- **Data Minimization**: Only necessary context data transmitted
- **Encryption**: End-to-end encryption for browser-to-application communication
- **Content Filtering**: Automatic filtering of sensitive content

### Experimental Feature Safety
- **Rollback System**: Immediate rollback capability for problematic features
- **Error Isolation**: Feature failures isolated from core functionality
- **Data Protection**: Experimental features cannot access sensitive user data
- **User Consent**: Explicit opt-in required for each experimental feature

## Performance Optimization Requirements

### Local Model Performance
- **Model Caching**: Intelligent caching of frequently used models
- **Memory Management**: Efficient memory allocation and cleanup
- **CPU Optimization**: Multi-core utilization for model inference
- **Storage Optimization**: Compressed model storage with fast decompression

### Network Optimization
- **Connection Pooling**: Efficient reuse of Ollama connections
- **Request Batching**: Batching of multiple model requests
- **Compression**: Compressed communication with Ollama host
- **Offline Support**: Graceful degradation when network unavailable

### UI Responsiveness
- **Async Operations**: All model operations performed asynchronously
- **Progressive Loading**: Incremental loading of model lists and status
- **Responsive Feedback**: Immediate user feedback for all interactions
- **Background Processing**: Heavy operations performed in background threads

## Integration Requirements

### Ollama API Integration
- **Version Compatibility**: Support for multiple Ollama API versions
- **Error Handling**: Robust error handling for Ollama communication failures
- **Authentication**: Support for secured Ollama instances
- **Model Metadata**: Rich metadata support for installed models

### Browser Extension Protocol
- **Cross-Platform Support**: Windows, macOS, and Linux browser support
- **Real-Time Communication**: WebSocket-based real-time communication
- **Content Type Support**: Support for various web content types
- **Selective Context**: User control over what context is extracted

### Experimental Feature Framework
- **A/B Testing**: Built-in A/B testing framework for feature evaluation
- **Gradual Rollout**: Phased rollout capability for new features
- **Usage Analytics**: Anonymous usage tracking for feature improvement
- **Crash Reporting**: Automatic crash reporting for experimental features

## Bevy Implementation Details

### Local AI and Ollama Component Architecture

```rust
use bevy::{prelude::*, utils::HashMap, tasks::Task};

// Ollama host configuration components
#[derive(Component, Reflect)]
pub struct OllamaHostSection {
    pub host_address: String,
    pub port: u16,
    pub connection_status: OllamaConnectionStatus,
    pub last_sync: Option<f64>,
}

#[derive(Component, Reflect)]
pub struct OllamaHostInput {
    pub current_text: String,
    pub is_editing: bool,
    pub is_valid: bool,
}

#[derive(Component, Reflect)]
pub struct SyncModelsButton {
    pub is_syncing: bool,
    pub last_sync_time: Option<f64>,
}

// Local model management components
#[derive(Component, Reflect)]
pub struct LocalModelSection {
    pub installed_models: Vec<OllamaModel>,
    pub installing_models: HashMap<String, DownloadProgress>,
}

#[derive(Component, Reflect)]
pub struct ModelInstallInput {
    pub current_text: String,
    pub placeholder: String,
    pub is_downloading: bool,
}

#[derive(Component, Reflect)]
pub struct ModelDownloadButton;

#[derive(Component, Reflect)]
pub struct ModelStatusDisplay {
    pub installed_count: u32,
    pub total_size_gb: f32,
}

// Browser extension components
#[derive(Component, Reflect)]
pub struct BrowserExtensionSection {
    pub connection_status: ExtensionConnectionStatus,
    pub last_successful_connection: Option<f64>,
    pub supported_browsers: Vec<SupportedBrowser>,
}

// Experimental features components
#[derive(Component, Reflect)]
pub struct ExperimentalFeaturesSection {
    pub available_experiments: Vec<ExperimentalFeature>,
}

#[derive(Component, Reflect)]
pub struct ExperimentToggle {
    pub feature_id: String,
    pub is_enabled: bool,
    pub animation_progress: f32,
}

#[derive(Component, Reflect)]
pub struct FeatureInfoButton {
    pub feature_id: String,
}

// Supporting data structures
#[derive(Clone, Reflect, PartialEq)]
pub enum OllamaConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error(String),
}

#[derive(Clone, Reflect)]
pub struct OllamaModel {
    pub name: String,
    pub size_gb: f32,
    pub tags: Vec<String>,
    pub last_updated: f64,
    pub is_active: bool,
}

#[derive(Clone, Reflect)]
pub struct DownloadProgress {
    pub model_name: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub status: DownloadStatus,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Installing,
    Complete,
    Failed(String),
}

#[derive(Clone, Reflect)]
pub struct ExtensionConnectionStatus {
    pub is_connected: bool,
    pub supported_contexts: Vec<String>,
    pub active_browsers: Vec<String>,
}

#[derive(Clone, Reflect)]
pub struct SupportedBrowser {
    pub name: String,
    pub version: String,
    pub is_installed: bool,
}

#[derive(Clone, Reflect)]
pub struct ExperimentalFeature {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub is_enabled: bool,
    pub stability_level: StabilityLevel,
    pub rollout_percentage: f32,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum StabilityLevel {
    Alpha,
    Beta,
    Stable,
    Deprecated,
}
```

### Advanced Resource Management for Local AI

```rust
// Local AI menu state with Ollama integration
#[derive(Resource, Reflect)]
pub struct LocalAiMenuState {
    pub ollama_config: OllamaConfiguration,
    pub model_management: ModelManagementState,
    pub browser_extension: BrowserExtensionState,
    pub experimental_features: ExperimentalFeatureState,
}

#[derive(Clone, Reflect)]
pub struct OllamaConfiguration {
    pub host: String,
    pub port: u16,
    pub api_version: String,
    pub connection_timeout_secs: u32,
    pub auto_sync_interval_mins: u32,
}

#[derive(Clone, Reflect)]
pub struct ModelManagementState {
    pub installed_models: HashMap<String, OllamaModel>,
    pub pending_downloads: HashMap<String, DownloadProgress>,
    pub storage_path: String,
    pub max_storage_gb: f32,
    pub current_storage_gb: f32,
}

#[derive(Clone, Reflect)]
pub struct BrowserExtensionState {
    pub is_enabled: bool,
    pub last_context_update: Option<f64>,
    pub context_history: Vec<BrowserContext>,
    pub privacy_settings: ContextPrivacySettings,
}

#[derive(Clone, Reflect)]
pub struct BrowserContext {
    pub title: String,
    pub url: String,
    pub content_preview: String,
    pub timestamp: f64,
    pub browser_type: String,
}

#[derive(Clone, Reflect)]
pub struct ContextPrivacySettings {
    pub include_urls: bool,
    pub include_page_content: bool,
    pub max_content_length: u32,
    pub blocked_domains: Vec<String>,
}

#[derive(Clone, Reflect)]
pub struct ExperimentalFeatureState {
    pub enabled_features: HashMap<String, bool>,
    pub feature_usage_stats: HashMap<String, FeatureUsageStats>,
    pub rollout_config: RolloutConfiguration,
}

#[derive(Clone, Reflect)]
pub struct FeatureUsageStats {
    pub activation_count: u32,
    pub total_usage_time_secs: f64,
    pub error_count: u32,
    pub last_used: Option<f64>,
}

#[derive(Clone, Reflect)]
pub struct RolloutConfiguration {
    pub user_segment: String,
    pub feature_availability: HashMap<String, bool>,
    pub a_b_test_assignments: HashMap<String, String>,
}
```

### Event System for Local AI Features

```rust
// Local AI specific events
#[derive(Event, Reflect)]
pub enum LocalAiMenuEvent {
    // Ollama events
    OllamaHostChanged(String, u16),
    OllamaConnectionRequested,
    OllamaModelsSync,
    OllamaModelInstallRequested(String),
    OllamaModelRemoved(String),
    
    // Browser extension events
    BrowserExtensionToggled(bool),
    BrowserContextReceived(BrowserContext),
    PrivacySettingsChanged(ContextPrivacySettings),
    
    // Experimental feature events
    ExperimentToggled(String, bool),
    FeatureInfoRequested(String),
    ExperimentCrashed(String, String),
    FeedbackSubmitted(String, String),
}

#[derive(Event, Reflect)]
pub struct OllamaConnectionEvent {
    pub status: OllamaConnectionStatus,
    pub host: String,
    pub port: u16,
    pub error_message: Option<String>,
}

#[derive(Event, Reflect)]
pub struct ModelDownloadEvent {
    pub model_name: String,
    pub progress: DownloadProgress,
}

#[derive(Event, Reflect)]
pub struct BrowserExtensionStatusEvent {
    pub browser: String,
    pub is_connected: bool,
    pub extension_version: String,
}

#[derive(Event, Reflect)]
pub struct ExperimentalFeatureEvent {
    pub feature_id: String,
    pub event_type: ExperimentEventType,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Reflect)]
pub enum ExperimentEventType {
    Enabled,
    Disabled,
    Used,
    ErrorOccurred,
    FeedbackProvided,
}
```

### System Architecture for Local AI Management

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LocalAiMenuSystems {
    Input,
    NetworkOperations,
    ModelManagement,
    ExperimentManagement,
    StateUpdate,
    Animation,
    Rendering,
}

impl Plugin for LocalAiMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<LocalAiMenuState>()
            .init_resource::<OllamaClient>()
            .init_resource::<BrowserExtensionManager>()
            .init_resource::<ExperimentManager>()
            
            // Events
            .add_event::<LocalAiMenuEvent>()
            .add_event::<OllamaConnectionEvent>()
            .add_event::<ModelDownloadEvent>()
            .add_event::<BrowserExtensionStatusEvent>()
            .add_event::<ExperimentalFeatureEvent>()
            
            // System ordering for async operations
            .configure_sets(Update, (
                LocalAiMenuSystems::Input,
                LocalAiMenuSystems::NetworkOperations,
                LocalAiMenuSystems::ModelManagement,
                LocalAiMenuSystems::ExperimentManagement,
                LocalAiMenuSystems::StateUpdate,
                LocalAiMenuSystems::Animation,
                LocalAiMenuSystems::Rendering,
            ).chain())
            
            // Input handling
            .add_systems(Update, (
                handle_ollama_host_input,
                handle_model_install_input,
                handle_experiment_toggles,
                handle_sync_models_button,
            ).in_set(LocalAiMenuSystems::Input))
            
            // Network and async operations
            .add_systems(Update, (
                manage_ollama_connection,
                process_model_downloads,
                sync_browser_extension_status,
                validate_experiment_availability,
            ).in_set(LocalAiMenuSystems::NetworkOperations))
            
            // Model management
            .add_systems(Update, (
                track_model_installations,
                update_model_storage_usage,
                cleanup_failed_downloads,
            ).in_set(LocalAiMenuSystems::ModelManagement))
            
            // Experiment management
            .add_systems(Update, (
                monitor_experiment_health,
                collect_experiment_usage_stats,
                handle_experiment_rollouts,
            ).in_set(LocalAiMenuSystems::ExperimentManagement))
            
            // State updates
            .add_systems(Update, (
                update_local_ai_menu_state,
                persist_experimental_feature_state,
                sync_browser_context_updates,
            ).in_set(LocalAiMenuSystems::StateUpdate))
            
            // Animations
            .add_systems(Update, (
                animate_experiment_toggles,
                animate_download_progress,
                animate_connection_status,
            ).in_set(LocalAiMenuSystems::Animation))
            
            // Rendering
            .add_systems(Update, (
                update_ollama_status_display,
                update_model_count_display,
                update_experiment_toggle_visuals,
                update_browser_connection_status,
            ).in_set(LocalAiMenuSystems::Rendering));
    }
}
```

### Ollama Integration and Model Management

```rust
fn setup_ollama_section(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    ollama_config: &OllamaConfiguration,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(140.0),
            max_height: Val::Px(200.0), // Constrain section height
            flex_direction: FlexDirection::Column,
            flex_grow: 0.0,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(16.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        OllamaHostSection {
            host_address: ollama_config.host.clone(),
            port: ollama_config.port,
            connection_status: OllamaConnectionStatus::Disconnected,
            last_sync: None,
        },
    )).with_children(|ollama_parent| {
        
        // Ollama Host input row
        spawn_two_column_row(ollama_parent, asset_server, "Ollama Host", |right_parent| {
            right_parent.spawn((
                Node {
                    width: Val::Percent(70.0),
                    max_width: Val::Px(300.0), // Constrain input width
                    height: Val::Px(32.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    flex_grow: 0.0,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
                OllamaHostInput {
                    current_text: format!("{}:{}", ollama_config.host, ollama_config.port),
                    is_editing: false,
                    is_valid: true,
                },
            )).with_children(|input_parent| {
                input_parent.spawn((
                    Text::new(&format!("{}:{}", ollama_config.host, ollama_config.port)),
                    TextFont {
                        font: asset_server.load("fonts/Inter-Regular.ttf"),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });
            
            // Info icon
            spawn_info_icon(right_parent, asset_server);
        });
        
        // Sync Models button
        ollama_parent.spawn((
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::FlexStart,
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            SyncModelsButton {
                is_syncing: false,
                last_sync_time: None,
            },
        )).with_children(|btn_parent| {
            btn_parent.spawn((
                Text::new("Sync Models"),
                TextFont {
                    font: asset_server.load("fonts/Inter-Regular.ttf"),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
    });
}

fn setup_experimental_features_section(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    experiments: &[ExperimentalFeature],
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(200.0),
            max_height: Val::Px(400.0), // Constrain experiments section
            flex_direction: FlexDirection::Column,
            flex_grow: 0.0,
            row_gap: Val::Px(16.0),
            padding: UiRect::all(Val::Px(16.0)),
            overflow: Overflow::clip_y(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        ExperimentalFeaturesSection {
            available_experiments: experiments.to_vec(),
        },
    )).with_children(|exp_parent| {
        
        // Section title and description
        exp_parent.spawn((
            Text::new("Experiments"),
            TextFont {
                font: asset_server.load("fonts/Inter-Bold.ttf"),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
        
        exp_parent.spawn((
            Text::new("New AI features in development. Your feedback will help us improve these experiments."),
            TextFont {
                font: asset_server.load("fonts/Inter-Regular.ttf"),
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
        
        // Experiment toggles container
        exp_parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Auto,
                max_height: Val::Px(300.0), // Constrain toggles container
                flex_direction: FlexDirection::Column,
                flex_grow: 0.0,
                row_gap: Val::Px(12.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
        )).with_children(|toggles_parent| {
            
            for experiment in experiments {
                spawn_experiment_toggle_row(
                    toggles_parent, 
                    asset_server, 
                    experiment
                );
            }
        });
    });
}

fn spawn_experiment_toggle_row(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    experiment: &ExperimentalFeature,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            max_height: Val::Px(40.0), // Constrain row height
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            flex_grow: 0.0,
            ..default()
        },
    )).with_children(|row_parent| {
        
        // Feature name (left)
        row_parent.spawn((
            Text::new(&experiment.display_name),
            TextFont {
                font: asset_server.load("fonts/Inter-Regular.ttf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                width: Val::Percent(60.0),
                max_width: Val::Px(300.0), // Constrain label width
                flex_grow: 0.0,
                ..default()
            },
        ));
        
        // Toggle and info icon container (right)
        row_parent.spawn((
            Node {
                width: Val::Auto,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                flex_grow: 0.0,
                ..default()
            },
        )).with_children(|controls_parent| {
            
            // Toggle switch
            spawn_ios_style_toggle(
                controls_parent,
                asset_server,
                experiment.is_enabled,
                experiment.id.clone(),
            );
            
            // Info icon
            controls_parent.spawn((
                Button,
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                BorderRadius::all(Val::Px(10.0)),
                FeatureInfoButton {
                    feature_id: experiment.id.clone(),
                },
            )).with_children(|icon_parent| {
                icon_parent.spawn((
                    Text::new("i"),
                    TextFont {
                        font: asset_server.load("fonts/Inter-Bold.ttf"),
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });
        });
    });
}
```

### Async Model Download and Management

```rust
// Async model download management using Bevy's task system
fn process_model_downloads(
    mut download_events: EventWriter<ModelDownloadEvent>,
    model_state: Res<LocalAiMenuState>,
    task_pool: Res<AsyncComputeTaskPool>,
    mut commands: Commands,
) {
    for (model_name, progress) in &model_state.model_management.pending_downloads {
        if progress.status == DownloadStatus::Queued {
            let model_name_clone = model_name.clone();
            let ollama_host = model_state.ollama_config.host.clone();
            let ollama_port = model_state.ollama_config.port;
            
            let task = task_pool.spawn(async move {
                // Simulate model download from Ollama
                let mut current_progress = DownloadProgress {
                    model_name: model_name_clone.clone(),
                    bytes_downloaded: 0,
                    total_bytes: 1024 * 1024 * 1024, // 1GB example
                    status: DownloadStatus::Downloading,
                };
                
                // Simulate progressive download
                for chunk in 0..10 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    current_progress.bytes_downloaded += current_progress.total_bytes / 10;
                    
                    // This would send progress updates in a real implementation
                }
                
                current_progress.status = DownloadStatus::Complete;
                
                ModelDownloadEvent {
                    model_name: model_name_clone,
                    progress: current_progress,
                }
            });
            
            commands.spawn(AsyncModelDownloadTask(task));
        }
    }
}

#[derive(Component)]
struct AsyncModelDownloadTask(Task<ModelDownloadEvent>);

fn handle_model_download_results(
    mut commands: Commands,
    mut download_tasks: Query<(Entity, &mut AsyncModelDownloadTask)>,
    mut download_events: EventWriter<ModelDownloadEvent>,
) {
    for (entity, mut task) in download_tasks.iter_mut() {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            download_events.write(result);
            commands.entity(entity).despawn();
        }
    }
}

// Experiment toggle animation system
fn animate_experiment_toggles(
    time: Res<Time>,
    mut toggle_query: Query<(&mut ExperimentToggle, &Children), Changed<ExperimentToggle>>,
    mut transform_query: Query<&mut Transform>,
    mut background_query: Query<&mut BackgroundColor>,
) {
    for (mut toggle, children) in toggle_query.iter_mut() {
        let target_progress = if toggle.is_enabled { 1.0 } else { 0.0 };
        let animation_speed = 6.0;
        
        if (toggle.animation_progress - target_progress).abs() > 0.01 {
            toggle.animation_progress = toggle.animation_progress
                .lerp(target_progress, animation_speed * time.delta_secs());
            
            // Update background color
            let background_color = Color::srgb(
                0.4 * toggle.animation_progress + 0.2, // Gray to blue transition
                0.4 * toggle.animation_progress + 0.2,
                0.8 * toggle.animation_progress + 0.2,
            );
            
            // Find and animate the toggle circle position
            for &child in children.iter() {
                if let Ok(mut transform) = transform_query.get_mut(child) {
                    // Animate circle position within toggle
                    transform.translation.x = -12.0 + (24.0 * toggle.animation_progress);
                }
                
                if let Ok(mut bg_color) = background_query.get_mut(child) {
                    *bg_color = BackgroundColor(background_color);
                }
            }
        }
    }
}
```

### Testing Strategy for Local AI Features

```rust
#[cfg(test)]
mod local_ai_tests {
    use super::*;
    
    #[test]
    fn test_ollama_host_validation() {
        let mut app = setup_test_app();
        
        // Test valid host input
        let ollama_input = OllamaHostInput {
            current_text: "127.0.0.1:11434".to_string(),
            is_editing: true,
            is_valid: true,
        };
        
        app.world_mut().spawn(ollama_input);
        app.update();
        
        let input = app.world().query::<&OllamaHostInput>().single(app.world());
        assert!(input.is_valid);
        assert_eq!(input.current_text, "127.0.0.1:11434");
    }
    
    #[test]
    fn test_experiment_toggle_state() {
        let mut app = setup_test_app();
        
        // Create experiment toggle
        let toggle_entity = app.world_mut().spawn((
            ExperimentToggle {
                feature_id: "auto_models".to_string(),
                is_enabled: false,
                animation_progress: 0.0,
            },
        )).id();
        
        // Send toggle event
        app.world_mut().resource_mut::<Events<LocalAiMenuEvent>>()
            .write(LocalAiMenuEvent::ExperimentToggled("auto_models".to_string(), true));
        
        app.update();
        
        let toggle = app.world().get::<ExperimentToggle>(toggle_entity).unwrap();
        assert!(toggle.is_enabled);
    }
    
    #[test]
    fn test_model_download_progress() {
        let mut app = setup_test_app();
        
        // Create model management state
        let mut model_state = LocalAiMenuState::default();
        model_state.model_management.pending_downloads.insert(
            "llama2".to_string(),
            DownloadProgress {
                model_name: "llama2".to_string(),
                bytes_downloaded: 0,
                total_bytes: 1000,
                status: DownloadStatus::Queued,
            }
        );
        
        app.world_mut().insert_resource(model_state);
        
        // Run several update cycles to simulate download
        for _ in 0..20 {
            app.update();
        }
        
        // Verify download events were generated
        let download_events: Vec<_> = app.world()
            .resource::<Events<ModelDownloadEvent>>()
            .get_reader()
            .read(app.world().resource::<Events<ModelDownloadEvent>>())
            .collect();
        
        assert!(!download_events.is_empty());
    }
}
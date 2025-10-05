# AI Menu 3 - Ollama Host Configuration and Local AI Infrastructure

## Task: Implement Ollama Integration and Local Model Management

### File: `ui/src/ai/ollama/mod.rs` (new file)

Create comprehensive Ollama integration with host configuration, model synchronization, and local AI infrastructure.

### Implementation Requirements

#### Ollama Host Configuration System
- File: `ui/src/ai/ollama/host_config.rs` (new file, line 1-89)
- Host input field management with default "127.0.0.1:11434"
- Connection validation and health monitoring
- Real-time status updates and error handling
- Bevy Example Reference: [`ui/text_input.rs`](../../../docs/bevy/examples/ui/text_input.rs) - Text input patterns for host configuration

#### Model Synchronization System
- File: `ui/src/ai/ollama/sync.rs` (new file, line 1-134)
- "Sync Models" button functionality with async model discovery
- Real-time model count display ("5 models installed via Ollama")
- Model metadata caching and status tracking
- Integration with Ollama API for model registry access

#### Local Model Installation Pipeline
```rust
#[derive(Resource, Debug, Clone)]
pub struct OllamaConfig {
    pub host: String,
    pub port: u16,
    pub connection_status: ConnectionStatus,
    pub installed_models: HashMap<String, ModelInfo>,
    pub sync_in_progress: bool,
}
```

#### Model Installation Interface
- File: `ui/src/ai/ollama/installation.rs` (new file, line 1-123)
- Text input for model name with download icon integration
- Progress tracking for model downloads with real-time feedback
- Model validation and installation status management
- Bevy Example Reference: [`async_tasks/async_compute.rs`](../../../docs/bevy/examples/async_tasks/async_compute.rs) - Asynchronous download operations

#### Connection Health Monitoring
- File: `ui/src/ai/ollama/health.rs` (new file, line 1-67)
- Continuous connection health checks to Ollama instance
- Automatic reconnection logic with exponential backoff
- Error state management and user feedback
- Performance monitoring for local model inference

### Architecture Notes
- Asynchronous Ollama API communication with proper error handling
- Local model caching and efficient metadata storage
- Real-time UI updates for model operations and status
- Integration with existing AI provider system for seamless switching
- Resource management for local model storage and memory usage

### Integration Points
- `ui/src/ai/provider_bridge.rs` - Local model provider integration
- `core/src/runtime/` - Model execution runtime environment
- `app/src/preferences/` - Ollama configuration persistence
- System storage management for model files

### Event System Integration
```rust
#[derive(Event)]
pub enum OllamaEvent {
    HostConfigured(String),
    ModelSyncStarted,
    ModelInstalled(String),
    ConnectionStatusChanged(ConnectionStatus),
    ModelCountUpdated(usize),
}
```

### Bevy Example References
- **Text Input**: [`ui/text_input.rs`](../../../docs/bevy/examples/ui/text_input.rs) - Host configuration input
- **Async Operations**: [`async_tasks/async_compute.rs`](../../../docs/bevy/examples/async_tasks/async_compute.rs) - Model downloads
- **Resource Management**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Ollama config resources
- **UI Updates**: [`ui/ui.rs`](../../../docs/bevy/examples/ui/ui.rs) - Real-time status displays

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.
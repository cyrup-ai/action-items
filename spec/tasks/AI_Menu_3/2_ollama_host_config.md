# AI Menu 3 - Ollama Host Configuration System

## Implementation Task: Host Connection Management and Model Synchronization

### Architecture Overview
Implement Ollama host configuration system including IP address input validation, connection management, model synchronization, and real-time health monitoring.

### Core Components

#### Host Configuration Interface
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct OllamaHostConfiguration {
    pub host_input: Entity,              // Input field entity
    pub sync_button: Entity,             // "Sync Models" button entity
    pub info_button: Entity,             // Info "i" button entity
    pub connection_indicator: Entity,    // Visual connection status
    pub current_host: String,            // "127.0.0.1:11434"
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct HostInputField {
    pub current_value: String,
    pub placeholder: String,
    pub validation_state: ValidationState,
    pub input_focus: bool,
    pub cursor_position: usize,
    pub selection_range: Option<(usize, usize)>,
}

#[derive(Reflect)]
pub enum ValidationState {
    Valid,
    Invalid(String),     // Error message
    Validating,
    Unknown,
}
```

#### Connection Management System
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct OllamaConnectionManager {
    pub active_connection: Option<OllamaConnection>,
    pub connection_history: VecDeque<ConnectionAttempt>,
    pub health_monitor: HealthMonitor,
    pub retry_policy: RetryPolicy,
    pub connection_pool: ConnectionPool,
}

#[derive(Reflect)]
pub struct OllamaConnection {
    pub host: String,
    pub port: u16,
    pub connection_id: String,
    pub established_at: SystemTime,
    pub last_activity: SystemTime,
    pub api_version: Option<String>,
    pub server_info: Option<ServerInfo>,
    pub status: ConnectionStatus,
}

#[derive(Reflect)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Authenticated,
    Idle,
    Error(String),
    Disconnected,
    Timeout,
}
```

### Bevy Implementation References

#### Text Input with Validation
- **Text Input**: `docs/bevy/examples/input/text_input.rs`
  - IP address input field with real-time validation
  - Cursor positioning and text selection
  - Input focus management and visual feedback

#### Network Communication
- **Async HTTP**: `docs/bevy/examples/async_tasks/async_compute.rs`
  - Asynchronous HTTP requests to Ollama REST API
  - Connection timeout and retry handling
  - Background health checking and monitoring

#### Button Interactions
- **Button System**: `docs/bevy/examples/ui/button.rs`
  - "Sync Models" button with loading states
  - Info button with tooltip functionality
  - Button state management and visual feedback

#### Real-time Status Updates
- **Event System**: `docs/bevy/examples/ecs/send_and_receive_events.rs`
  - Connection status change events
  - Model synchronization progress events
  - Error and success notification events

### Host Input Validation

#### IP Address and Port Validation
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct HostValidator {
    pub validation_rules: Vec<ValidationRule>,
    pub format_patterns: Vec<FormatPattern>,
    pub port_range: (u16, u16),          // (1, 65535)
    pub default_port: u16,               // 11434
}

#[derive(Reflect)]
pub enum ValidationRule {
    ValidIPAddress,
    ValidPort,
    Reachable,
    OllamaService,
}

#[derive(Reflect)]
pub struct FormatPattern {
    pub pattern_name: String,
    pub regex: String,
    pub example: String,           // "127.0.0.1:11434"
}
```

#### Real-time Input Validation
- **Format Validation**: Check IP:port format as user types
- **Range Validation**: Validate port numbers within valid range
- **Accessibility**: Automatic port completion (default 11434)
- **Error Display**: Clear error messages for invalid input

### Model Synchronization System

#### Sync Models Implementation
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]  
pub struct ModelSynchronizer {
    pub sync_status: SyncStatus,
    pub discovered_models: Vec<OllamaModel>,
    pub sync_progress: SyncProgress,
    pub last_sync: Option<SystemTime>,
    pub auto_sync_enabled: bool,
}

#[derive(Reflect)]
pub enum SyncStatus {
    Idle,
    Connecting,
    Discovering,
    Syncing,
    Completed,
    Failed(String),
}

#[derive(Reflect)]
pub struct SyncProgress {
    pub current_step: String,        // "Discovering models..."
    pub completed_models: u32,
    pub total_models: u32,
    pub current_model: Option<String>,
    pub estimated_time_remaining: Option<Duration>,
}

#[derive(Reflect)]
pub struct OllamaModel {
    pub name: String,
    pub tag: String,
    pub size: u64,
    pub digest: String,
    pub modified_at: SystemTime,
    pub parameters: ModelParameters,
    pub template: Option<String>,
}
```

#### Sync Button Behavior
- **Loading State**: Visual loading indicator during sync
- **Progress Feedback**: Real-time progress updates
- **Error Handling**: Clear error messages for sync failures
- **Success Confirmation**: Visual confirmation of successful sync

### Connection Health Monitoring

#### Real-time Health Checks
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct HealthMonitor {
    pub health_status: HealthStatus,
    pub check_interval: Duration,       // Default: 30 seconds
    pub last_check: SystemTime,
    pub response_times: VecDeque<Duration>,
    pub error_history: VecDeque<HealthError>,
    pub consecutive_failures: u32,
}

#[derive(Reflect)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
    Unknown,
    Checking,
}

#[derive(Reflect)]
pub struct HealthError {
    pub timestamp: SystemTime,
    pub error_type: HealthErrorType,
    pub message: String,
    pub recovery_action: Option<RecoveryAction>,
}
```

#### Automatic Reconnection
- **Exponential Backoff**: Progressive retry delays for failed connections
- **Health Recovery**: Automatic reconnection when service becomes available
- **User Notification**: Clear status communication about connection issues
- **Manual Override**: User can manually retry connection attempts

### Visual Interface Implementation

#### Host Input Field Styling
- **Background**: Dark theme input field (#2a2a2a)
- **Text Color**: White text for valid input
- **Error State**: Red border and text for invalid input
- **Focus State**: Subtle border highlight when focused
- **Placeholder**: Light gray text for guidance

#### Sync Button Design  
- **Default State**: Standard button with darker background
- **Loading State**: Spinner or progress animation
- **Success State**: Brief success indicator
- **Error State**: Error styling with retry option
- **Disabled State**: Grayed out when no connection

#### Status Indicators
- **Connection Status**: Visual indicator next to host field
- **Model Count**: "5 models installed via Ollama" display
- **Last Sync**: Timestamp of last successful synchronization
- **Health Badge**: Color-coded health status indicator

### API Integration

#### Ollama REST API Client
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct OllamaAPIClient {
    pub base_url: String,
    pub http_client: HttpClient,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub rate_limiter: RateLimiter,
}

// API Endpoints
#[derive(Reflect)]
pub enum OllamaEndpoint {
    Health,              // GET /
    ListModels,          // GET /api/tags
    ShowModel,          // POST /api/show
    Generate,           // POST /api/generate
    Chat,              // POST /api/chat
    Create,            // POST /api/create
    Push,              // POST /api/push
    Pull,              // POST /api/pull
}
```

#### Error Handling and Recovery
```rust
#[derive(Event)]
pub struct OllamaConnectionError {
    pub host: String,
    pub error_type: ConnectionErrorType,
    pub error_message: String,
    pub retry_count: u32,
    pub recovery_suggestions: Vec<String>,
}

#[derive(Reflect)]
pub enum ConnectionErrorType {
    NetworkUnreachable,
    ConnectionRefused,
    Timeout,
    InvalidResponse,
    AuthenticationFailed,
    ServiceUnavailable,
    InvalidHost,
}
```

### Performance Optimization

#### Efficient Connection Management
- **Connection Pooling**: Reuse HTTP connections for multiple requests
- **Request Batching**: Batch multiple API calls when possible
- **Response Caching**: Cache model information to reduce API calls
- **Background Processing**: Perform sync operations without blocking UI

#### Resource Management
- **Memory Efficiency**: Efficient storage of model metadata
- **Network Optimization**: Minimize bandwidth usage for status checks
- **CPU Optimization**: Lightweight health checking and monitoring
- **Battery Optimization**: Intelligent scheduling of background operations

### Security Considerations

#### Network Security
- **TLS Validation**: Verify TLS certificates for HTTPS connections
- **Input Sanitization**: Sanitize host input to prevent injection attacks
- **Rate Limiting**: Prevent abuse of Ollama API endpoints
- **Error Message Sanitization**: Avoid exposing sensitive information in errors

#### Authentication Support
- **API Key Support**: Support for Ollama instances requiring authentication
- **Token Management**: Secure storage and renewal of authentication tokens
- **Permission Validation**: Validate user permissions for Ollama operations
- **Audit Logging**: Log connection attempts and configuration changes

### Integration Points

#### Model Management Integration
- **Model Discovery**: Automatically discover available models from Ollama
- **Installation Coordination**: Coordinate with model installation system
- **Usage Tracking**: Track model usage across different Ollama hosts
- **Version Management**: Handle model version updates and compatibility

#### UI System Integration
- **Real-time Updates**: Update UI immediately when connection status changes
- **Error Display**: Integrate error messages into UI notification system
- **Progress Display**: Show sync progress in appropriate UI components
- **Settings Persistence**: Save host configuration to user preferences

### Testing Requirements

#### Validation Testing
- **Input Validation**: Test all host input validation scenarios
- **Network Testing**: Test connection behavior under various network conditions
- **Error Handling**: Test recovery from all possible error conditions
- **Performance Testing**: Verify connection performance meets requirements

#### Integration Testing
- **API Compatibility**: Test with different versions of Ollama
- **Multi-host Testing**: Test switching between different Ollama hosts
- **Concurrent Operations**: Test multiple simultaneous operations
- **State Persistence**: Test configuration persistence across application restarts

### Implementation Files
- `ai_menu_3/ollama_host_config.rs` - Host configuration UI components
- `ai_menu_3/ollama_connection.rs` - Connection management and health monitoring
- `ai_menu_3/ollama_api_client.rs` - REST API client implementation
- `ai_menu_3/model_synchronizer.rs` - Model discovery and synchronization
- `ai_menu_3/host_validation.rs` - Input validation and format checking
- `ai_menu_3/connection_events.rs` - Connection-related event definitions

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all connection monitoring loops
- **Blazing-fast performance** - efficient network operations and validation
- **Production quality** - reliable Ollama host connection system
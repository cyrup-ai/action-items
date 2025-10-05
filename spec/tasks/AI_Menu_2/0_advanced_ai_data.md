# AI Menu 2 - Advanced AI Data Structures

## Implementation Task: AI Commands, MCP Server, and Tool Integration Data Models

### Architecture Overview
Implement comprehensive data structures for advanced AI features including AI Commands, Model Context Protocol (MCP) server integration, tool management, and secure API key storage.

### Core Components

#### AI Commands Data System
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AICommandsConfiguration {
    pub default_model: String,
    pub custom_commands: HashMap<String, CustomAICommand>,
    pub command_templates: Vec<CommandTemplate>,
    pub model_preferences: HashMap<CommandType, String>,
}

#[derive(Reflect, Clone)]
pub struct CustomAICommand {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub preferred_model: Option<String>,
    pub category: CommandCategory,
    pub usage_count: u64,
    pub last_used: SystemTime,
}

#[derive(Reflect)]
pub enum CommandCategory {
    Writing,
    Coding,
    Analysis,
    Translation,
    Custom,
}

#[derive(Reflect)]
pub struct CommandTemplate {
    pub name: String,
    pub instructions: String,
    pub category: CommandCategory,
    pub default_parameters: HashMap<String, String>,
}
```

#### MCP Server Management System
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MCPServerManager {
    pub registered_servers: HashMap<String, MCPServerConfig>,
    pub active_connections: HashMap<String, MCPConnection>,
    pub server_idle_timeout: Duration,
    pub auto_confirm_tools: bool,
    pub connection_status: HashMap<String, ServerStatus>,
}

#[derive(Reflect, Clone)]
pub struct MCPServerConfig {
    pub server_id: String,
    pub name: String,
    pub url: String,
    pub authentication: AuthenticationConfig,
    pub capabilities: ServerCapabilities,
    pub trust_level: TrustLevel,
    pub enabled: bool,
}

#[derive(Reflect)]
pub struct MCPConnection {
    pub server_id: String,
    pub connection_state: ConnectionState,
    pub last_activity: SystemTime,
    pub available_tools: Vec<ToolDefinition>,
    pub pending_operations: VecDeque<MCPOperation>,
}

#[derive(Reflect)]
pub enum ServerStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
    Idle,
}
```

#### Tool Integration System
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ToolIntegrationConfig {
    pub show_tool_call_info: bool,
    pub tool_confirmations_reset: bool,
    pub available_tools: HashMap<String, ToolDefinition>,
    pub tool_permissions: HashMap<String, ToolPermission>,
    pub execution_history: VecDeque<ToolExecution>,
}

#[derive(Reflect, Clone)]
pub struct ToolDefinition {
    pub tool_id: String,
    pub name: String,
    pub description: String,
    pub provider: ToolProvider,
    pub required_permissions: Vec<Permission>,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

#[derive(Reflect)]
pub enum ToolProvider {
    MCP(String),           // Server ID
    Built_in,
    Extension(String),     // Extension ID
}

#[derive(Reflect, Clone)]
pub struct ToolPermission {
    pub tool_id: String,
    pub permission_level: PermissionLevel,
    pub auto_confirm: bool,
    pub restrictions: Vec<ToolRestriction>,
    pub granted_by_user: bool,
}

#[derive(Reflect)]
pub struct ToolExecution {
    pub execution_id: String,
    pub tool_id: String,
    pub timestamp: SystemTime,
    pub input_data: serde_json::Value,
    pub output_data: Option<serde_json::Value>,
    pub status: ExecutionStatus,
    pub duration: Duration,
}
```

### Bevy Implementation References

#### Resource Management for Complex Data
- **Resource Systems**: `docs/bevy/examples/ecs/startup_system.rs`
  - Global MCP server manager initialization
  - Tool integration configuration resource setup
  - Shared state management across AI systems

#### Dynamic Data Loading
- **Asset Loading**: `docs/bevy/examples/asset/asset_loading.rs`
  - Dynamic loading of AI command templates
  - MCP server configuration loading from external sources
  - Tool definition discovery and registration

#### State Serialization
- **Reflection System**: `docs/bevy/examples/reflection/reflection.rs`
  - Configuration serialization for persistence
  - Dynamic introspection of AI command data
  - Type-safe state management with reflection

#### Event-Driven Updates
- **Event System**: `docs/bevy/examples/ecs/send_and_receive_events.rs`
  - MCP server status change events
  - Tool execution completion notifications
  - API key validation result events

### API Key Management System

#### Secure Storage Architecture
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct APIKeyManager {
    pub provider_configs: HashMap<String, ProviderConfig>,
    pub key_validation_status: HashMap<String, ValidationStatus>,
    pub usage_tracking: HashMap<String, UsageMetrics>,
    pub routing_preferences: RoutingConfig,
}

#[derive(Reflect, Clone)]
pub struct ProviderConfig {
    pub provider_id: String,
    pub provider_name: String,
    pub routing_type: RoutingType,
    pub capabilities: ProviderCapabilities,
    pub rate_limits: RateLimitConfig,
    pub cost_tracking: bool,
}

#[derive(Reflect)]
pub enum RoutingType {
    RaycastServers,    // Anthropic, Google, OpenAI
    DirectProvider,    // OpenRouter
    Custom(String),    // Custom routing endpoint
}

#[derive(Reflect)]
pub struct ProviderCapabilities {
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub max_context_length: usize,
    pub supported_models: Vec<ModelInfo>,
}

#[derive(Reflect)]
pub struct UsageMetrics {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub estimated_cost: f64,
    pub last_request: SystemTime,
    pub error_count: u64,
}
```

#### Security and Validation
```rust
#[derive(Reflect)]
pub struct SecureAPIKey {
    pub provider_id: String,
    pub key_id: String,           // Internal identifier
    pub encrypted_key: Vec<u8>,   // Encrypted API key data
    pub validation_hash: String,  // For integrity checking
    pub created_at: SystemTime,
    pub last_validated: SystemTime,
}

#[derive(Reflect)]
pub enum ValidationStatus {
    Valid,
    Invalid(String),
    Expired,
    RateLimited,
    Validating,
    NotValidated,
}
```

### Data Integration Architecture

#### Cross-System Dependencies
- **AI Commands ↔ Model Selection**: Commands reference available models
- **MCP Servers ↔ Tool Integration**: Tools provided by MCP servers  
- **API Keys ↔ Provider Selection**: API keys enable provider access
- **Tool Permissions ↔ Security**: Permission system governs tool access

#### Real-time Synchronization
```rust
#[derive(Event)]
pub struct MCPServerStatusChanged {
    pub server_id: String,
    pub old_status: ServerStatus,
    pub new_status: ServerStatus,
}

#[derive(Event)]
pub struct ToolAvailabilityChanged {
    pub server_id: String,
    pub available_tools: Vec<ToolDefinition>,
}

#[derive(Event)]
pub struct APIKeyValidationCompleted {
    pub provider_id: String,
    pub validation_result: ValidationStatus,
    pub provider_capabilities: Option<ProviderCapabilities>,
}
```

### Configuration Persistence

#### Settings Storage Architecture
- **Hierarchical Configuration**: Structured config with sections for each subsystem
- **Incremental Updates**: Only changed sections written to storage
- **Version Migration**: Automatic migration between configuration versions
- **Backup Integration**: Configuration backup with cloud sync support

#### Data Validation System
```rust
#[derive(Reflect)]
pub struct ConfigurationValidator {
    pub validation_rules: HashMap<String, ValidationRule>,
    pub schema_version: u32,
    pub migration_handlers: Vec<MigrationHandler>,
}

#[derive(Reflect)]
pub enum ValidationRule {
    Required,
    Range(i64, i64),
    Pattern(String),
    Enum(Vec<String>),
    Custom(String),    // Custom validation function name
}
```

### Performance Optimization

#### Efficient Data Structures
- **HashMap Usage**: Optimized for frequent lookups by ID
- **VecDeque for Queues**: Efficient queue operations for pending tasks
- **LRU Caches**: Memory-bounded caches for frequently accessed data
- **Copy vs Clone**: Minimal copying of large data structures

#### Memory Management
- **Resource Pooling**: Reuse of connection and execution contexts
- **Lazy Loading**: On-demand loading of heavy data structures
- **Garbage Collection**: Periodic cleanup of stale data
- **Zero-Copy Operations**: Minimize data copying in hot paths

### Error Handling Architecture

#### Comprehensive Error Types
```rust
#[derive(Reflect)]
pub enum AdvancedAIError {
    MCPConnectionFailed(String),
    ToolExecutionFailed(String, String),
    APIKeyValidationFailed(String),
    ConfigurationInvalid(String),
    PermissionDenied(String),
    NetworkError(String),
    SecurityViolation(String),
}

#[derive(Event)]
pub struct AdvancedAIErrorEvent {
    pub error: AdvancedAIError,
    pub timestamp: SystemTime,
    pub context: HashMap<String, String>,
    pub recovery_suggestions: Vec<String>,
}
```

#### Graceful Degradation
- **Fallback Providers**: Automatic fallback when primary provider fails
- **Offline Mode**: Cached data usage when network unavailable
- **Partial Functionality**: Core features work even if advanced features fail
- **User Notification**: Clear error communication without system crashes

### Testing Infrastructure

#### Data Model Testing
- **Serialization Roundtrip**: Verify all data structures serialize/deserialize correctly
- **Schema Evolution**: Test configuration migration between versions  
- **Validation Logic**: Comprehensive testing of all validation rules
- **Performance Benchmarks**: Ensure data operations meet performance requirements

#### Integration Testing
- **Cross-System Communication**: Test event propagation between subsystems
- **State Consistency**: Verify system state remains consistent across operations
- **Error Recovery**: Test graceful recovery from various error conditions
- **Security Testing**: Validate security constraints are enforced

### Implementation Files
- `ai_menu_2/advanced_ai_data.rs` - Core data structures and types
- `ai_menu_2/mcp_types.rs` - MCP-specific data structures and protocols
- `ai_menu_2/tool_types.rs` - Tool integration data structures
- `ai_menu_2/api_key_types.rs` - API key management data structures
- `ai_menu_2/validation.rs` - Configuration validation and migration
- `ai_menu_2/events.rs` - Event definitions for advanced AI features

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all data structure operations
- **Blazing-fast performance** - efficient HashMap and VecDeque usage
- **Production quality** - complete, secure data architecture

## Bevy Implementation Details

### Component Architecture
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AIMenuAdvancedData {
    pub ai_commands_config: AICommandsConfiguration,
    pub tool_integration_config: ToolIntegrationConfig,
    pub current_view: AdvancedAIView,
}

#[derive(Reflect, PartialEq)]
pub enum AdvancedAIView {
    Commands,
    MCPServers,
    Tools,
    APIKeys,
}
```

### System Architecture with SystemSets
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AdvancedAISystemSet {
    DataLoading,
    MCPManagement,
    ToolIntegration,
    APIKeyValidation,
    UIUpdate,
}

impl Plugin for AdvancedAIPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (
            AdvancedAISystemSet::DataLoading,
            AdvancedAISystemSet::MCPManagement,
            AdvancedAISystemSet::ToolIntegration,
            AdvancedAISystemSet::APIKeyValidation,
            AdvancedAISystemSet::UIUpdate,
        ).chain())
        .add_systems(Startup, setup_advanced_ai_data)
        .add_systems(Update, (
            load_ai_commands_data.in_set(AdvancedAISystemSet::DataLoading),
            manage_mcp_connections.in_set(AdvancedAISystemSet::MCPManagement),
            handle_tool_permissions.in_set(AdvancedAISystemSet::ToolIntegration),
            validate_api_keys.in_set(AdvancedAISystemSet::APIKeyValidation),
            update_advanced_ai_ui.in_set(AdvancedAISystemSet::UIUpdate),
        ));
    }
}
```

### Event-Driven Architecture
```rust
fn handle_advanced_ai_events(
    mut commands: Commands,
    mut ai_events: EventReader<AdvancedAIEvent>,
    mut mcp_manager: ResMut<MCPServerManager>,
    mut api_key_manager: ResMut<APIKeyManager>,
    time: Res<Time>,
) {
    for event in ai_events.read() {
        match event {
            AdvancedAIEvent::MCPServerConnected { server_id } => {
                if let Some(config) = mcp_manager.registered_servers.get(server_id) {
                    mcp_manager.connection_status.insert(
                        server_id.clone(), 
                        ServerStatus::Connected
                    );
                    // Trigger tool discovery
                    commands.trigger(ToolDiscoveryEvent { 
                        server_id: server_id.clone() 
                    });
                }
            }
            AdvancedAIEvent::APIKeyValidated { provider_id, status } => {
                api_key_manager.key_validation_status.insert(
                    provider_id.clone(), 
                    status.clone()
                );
            }
            AdvancedAIEvent::ToolExecutionCompleted { execution_id, result } => {
                // Update execution history
                // Trigger UI refresh
            }
        }
    }
}
```

### Resource Management for Async Operations
```rust
#[derive(Component)]
pub struct MCPConnectionTask(Task<Result<MCPConnection, MCPError>>);

fn spawn_mcp_connection_tasks(
    mut commands: Commands,
    mcp_manager: Res<MCPServerManager>,
    task_pool: Res<AsyncComputeTaskPool>,
) {
    for (server_id, config) in &mcp_manager.registered_servers {
        if config.enabled && !mcp_manager.active_connections.contains_key(server_id) {
            let server_config = config.clone();
            let task = task_pool.spawn(async move {
                establish_mcp_connection(server_config).await
            });
            
            commands.spawn(MCPConnectionTask(task));
        }
    }
}

fn poll_mcp_connection_tasks(
    mut commands: Commands,
    mut connection_tasks: Query<(Entity, &mut MCPConnectionTask)>,
    mut mcp_manager: ResMut<MCPServerManager>,
) {
    for (entity, mut task) in &mut connection_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            match result {
                Ok(connection) => {
                    mcp_manager.active_connections.insert(
                        connection.server_id.clone(), 
                        connection
                    );
                }
                Err(error) => {
                    error!("MCP connection failed: {:?}", error);
                }
            }
            commands.entity(entity).despawn();
        }
    }
}
```

### Query Optimization with Change Detection
```rust
fn update_tool_integration_ui(
    mut ui_query: Query<&mut ToolIntegrationConfig, Changed<ToolIntegrationConfig>>,
    mut ui_events: EventWriter<UIUpdateEvent>,
) {
    for config in &ui_query {
        // Only update UI when tool configuration actually changes
        ui_events.write(UIUpdateEvent::ToolsConfigurationChanged {
            available_tools: config.available_tools.len(),
            pending_confirmations: config.tool_permissions.values()
                .filter(|p| !p.granted_by_user)
                .count(),
        });
    }
}
```

### Flex-Based UI Layout for Complex Data Display
```rust
fn spawn_advanced_ai_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(12.0),
            max_width: Val::Px(800.0), // Prevent expansion
            flex_grow: 0.0, // Critical: prevent unwanted expansion
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
    ))
    .with_children(|parent| {
        // Header section
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(12.0)),
                flex_grow: 0.0,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        ));
        
        // Data sections with scrollable content
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0), // Fill remaining space
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(), // Handle content overflow
                max_height: Val::Px(600.0),
                flex_grow: 1.0, // Allow this to expand
                ..default()
            },
        ));
    });
}
```

### Testing Strategy for Complex Data Systems
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    
    #[test]
    fn test_advanced_ai_data_serialization() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AdvancedAIPlugin));
        
        let ai_config = AICommandsConfiguration {
            default_model: "claude-3-haiku".to_string(),
            custom_commands: HashMap::new(),
            command_templates: vec![],
            model_preferences: HashMap::new(),
        };
        
        // Test serialization roundtrip
        let serialized = serde_json::to_string(&ai_config).expect("Serialization failed");
        let deserialized: AICommandsConfiguration = serde_json::from_str(&serialized)
            .expect("Deserialization failed");
        
        assert_eq!(ai_config.default_model, deserialized.default_model);
    }
    
    #[test]
    fn test_mcp_connection_lifecycle() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AdvancedAIPlugin));
        
        // Test MCP server registration and connection lifecycle
        let mut mcp_manager = MCPServerManager::default();
        let server_config = MCPServerConfig {
            server_id: "test-server".to_string(),
            name: "Test MCP Server".to_string(),
            url: "http://localhost:8080".to_string(),
            authentication: AuthenticationConfig::None,
            capabilities: ServerCapabilities::default(),
            trust_level: TrustLevel::Trusted,
            enabled: true,
        };
        
        mcp_manager.registered_servers.insert("test-server".to_string(), server_config);
        app.insert_resource(mcp_manager);
        
        // Run one frame to process system
        app.update();
        
        let mcp_manager = app.world().resource::<MCPServerManager>();
        assert!(mcp_manager.registered_servers.contains_key("test-server"));
    }
}
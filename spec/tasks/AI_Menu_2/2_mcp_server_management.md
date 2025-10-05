# AI Menu 2 - Model Context Protocol (MCP) Server Management

## Implementation Task: MCP Server Integration and Management System

### Architecture Overview
Implement comprehensive Model Context Protocol server management including server discovery, connection handling, protocol validation, and real-time status monitoring.

### Core Components

#### MCP Protocol Implementation
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MCPProtocolHandler {
    pub protocol_version: String,
    pub supported_features: MCPFeatureSet,
    pub message_handlers: HashMap<String, MessageHandler>,
    pub request_timeout: Duration,
    pub max_concurrent_requests: usize,
}

#[derive(Reflect)]
pub struct MCPFeatureSet {
    pub supports_tools: bool,
    pub supports_resources: bool,
    pub supports_prompts: bool,
    pub supports_sampling: bool,
    pub supports_logging: bool,
    pub protocol_version: String,
}

#[derive(Reflect)]
pub struct MessageHandler {
    pub message_type: String,
    pub handler_function: String, // Function name for dynamic dispatch
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
}
```

#### Server Connection Management
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MCPConnectionManager {
    pub active_connections: HashMap<String, MCPServerConnection>,
    pub connection_pool: ConnectionPool,
    pub idle_timeout: Duration,
    pub max_connections_per_server: usize,
    pub health_check_interval: Duration,
}

#[derive(Reflect)]
pub struct MCPServerConnection {
    pub server_id: String,
    pub connection_id: String,
    pub websocket_url: String,
    pub connection_state: ConnectionState,
    pub last_heartbeat: SystemTime,
    pub capabilities: ServerCapabilities,
    pub message_queue: VecDeque<MCPMessage>,
    pub pending_requests: HashMap<String, PendingRequest>,
}

#[derive(Reflect)]
pub enum ConnectionState {
    Initializing,
    Connected,
    Authenticating,
    Ready,
    Idle,
    Reconnecting,
    Failed(String),
    Closed,
}
```

### Bevy Implementation References

#### WebSocket Communication
- **Network Communication**: `docs/bevy/examples/async_tasks/async_compute.rs`
  - Asynchronous WebSocket connection handling
  - Message sending and receiving with proper error handling
  - Connection state management and recovery

#### Real-time Status Updates
- **Event System**: `docs/bevy/examples/ecs/send_and_receive_events.rs`
  - Server status change events and propagation
  - Real-time UI updates for connection status
  - Event-driven architecture for MCP operations

#### Resource Management
- **Resource Systems**: `docs/bevy/examples/ecs/startup_system.rs`
  - Global MCP connection manager initialization
  - Shared connection pool management
  - Resource cleanup and lifecycle management

#### State Management
- **State Systems**: `docs/bevy/examples/state/states.rs`
  - Connection state transitions and validation
  - State-dependent behavior and UI updates
  - State persistence across application sessions

### Server Discovery and Registration

#### Discovery Mechanism
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MCPServerRegistry {
    pub registered_servers: HashMap<String, ServerRegistration>,
    pub discovery_sources: Vec<DiscoverySource>,
    pub auto_discovery_enabled: bool,
    pub trust_settings: TrustSettings,
}

#[derive(Reflect)]
pub struct ServerRegistration {
    pub server_id: String,
    pub display_name: String,
    pub description: String,
    pub websocket_url: String,
    pub authentication_required: bool,
    pub trust_level: TrustLevel,
    pub capabilities: ServerCapabilities,
    pub registration_source: RegistrationSource,
}

#[derive(Reflect)]
pub enum DiscoverySource {
    UserManual,
    SystemRegistry,
    NetworkBroadcast,
    ConfigurationFile(PathBuf),
    RemoteRegistry(String),
}
```

#### Server Management Interface
- **"Manage Servers" Button**: Opens comprehensive server configuration dialog
- **Server List Display**: Table view of registered servers with status indicators
- **Add Server Dialog**: Manual server registration with validation
- **Server Configuration**: Per-server settings and authentication
- **Bulk Operations**: Enable/disable multiple servers simultaneously

### Connection Lifecycle Management

#### Connection Establishment
1. **Initial Connection**: WebSocket connection establishment with timeout
2. **Protocol Handshake**: MCP protocol version negotiation
3. **Authentication**: Server authentication if required
4. **Capability Exchange**: Discovery of server tools and resources
5. **Ready State**: Server available for tool execution requests

#### Heartbeat and Health Monitoring
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ServerHealthMonitor {
    pub server_id: String,
    pub health_status: HealthStatus,
    pub last_heartbeat: SystemTime,
    pub response_times: VecDeque<Duration>,
    pub error_count: u32,
    pub consecutive_failures: u32,
}

#[derive(Reflect)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
    Unreachable,
}
```

#### Automatic Reconnection
- **Exponential Backoff**: Progressive retry delays for failed connections
- **Circuit Breaker**: Temporary suspension of connection attempts after repeated failures
- **Health Checks**: Periodic validation of connection health
- **Graceful Degradation**: Continue operation with remaining healthy servers

### Protocol Message Handling

#### Message Processing Pipeline
```rust
#[derive(Reflect)]
pub struct MCPMessage {
    pub message_id: String,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub timestamp: SystemTime,
    pub server_id: String,
    pub correlation_id: Option<String>,
}

#[derive(Reflect)]
pub enum MessageType {
    ToolCall,
    ToolResult,
    ResourceRequest,
    ResourceResponse,
    PromptRequest,
    PromptResponse,
    Notification,
    Error,
    Heartbeat,
}

#[derive(Reflect)]
pub struct PendingRequest {
    pub request_id: String,
    pub request_type: MessageType,
    pub timestamp: SystemTime,
    pub timeout: Duration,
    pub retry_count: u32,
    pub callback: String, // Callback function identifier
}
```

#### Request-Response Correlation
- **Unique Request IDs**: Track request-response pairs accurately
- **Timeout Management**: Handle requests that exceed timeout limits
- **Callback System**: Route responses to appropriate handlers
- **Error Propagation**: Proper error handling and user notification

### Server Configuration UI

#### Idle Time Configuration
- **"Server Idle Time" Dropdown**: "5 Minutes" (configurable options)
- **Options**: 1, 5, 15, 30 minutes, Never
- **Behavior**: Automatic connection closure after inactivity period
- **Resource Management**: Free up resources for inactive connections

#### Tool Call Automation
- **"Automatically confirm all tool calls" Checkbox**: Currently CHECKED
- **Warning Indicator**: Yellow/orange triangle warning icon
- **Security Implications**: Highlight potential security risks of auto-confirmation
- **Per-Server Override**: Allow per-server automation settings

### Security and Trust Management

#### Authentication System
```rust
#[derive(Reflect)]
pub enum AuthenticationMethod {
    None,
    APIKey(String),
    OAuth2(OAuth2Config),
    Certificate(CertificateConfig),
    Custom(CustomAuthConfig),
}

#[derive(Reflect)]
pub struct TrustSettings {
    pub require_authentication: bool,
    pub allowed_origins: Vec<String>,
    pub certificate_validation: CertificateValidation,
    pub trust_levels: HashMap<TrustLevel, PermissionSet>,
}

#[derive(Reflect)]
pub enum TrustLevel {
    Trusted,      // Full access to all capabilities
    Limited,      // Restricted access with user confirmation
    Sandboxed,    // Highly restricted execution environment
    Untrusted,    // Minimal access, extensive validation
}
```

#### Permission Management
- **Tool Execution Permissions**: Granular control over tool access
- **Resource Access Control**: Limit server access to system resources
- **Data Sharing Policies**: Control what data can be shared with servers
- **Audit Requirements**: Log all security-relevant operations

### Performance Optimization

#### Connection Pooling
- **Reusable Connections**: Maintain persistent connections for frequently used servers
- **Connection Limits**: Prevent resource exhaustion with connection caps
- **Load Balancing**: Distribute requests across multiple server instances
- **Connection Health**: Monitor and replace unhealthy connections

#### Message Queuing
- **Request Queuing**: Queue requests during temporary disconnections
- **Priority Handling**: Process high-priority requests first
- **Batch Processing**: Group similar requests for efficiency
- **Message Compression**: Reduce bandwidth usage for large messages

### Error Handling and Recovery

#### Connection Error Recovery
```rust
#[derive(Event)]
pub struct MCPConnectionError {
    pub server_id: String,
    pub error_type: ConnectionErrorType,
    pub error_message: String,
    pub recovery_action: RecoveryAction,
    pub timestamp: SystemTime,
}

#[derive(Reflect)]
pub enum ConnectionErrorType {
    NetworkError,
    AuthenticationFailed,
    ProtocolError,
    ServerUnavailable,
    TimeoutError,
    SecurityViolation,
}

#[derive(Reflect)]
pub enum RecoveryAction {
    Retry,
    Reconnect,
    SwitchServer,
    DisableServer,
    UserIntervention,
}
```

#### Graceful Degradation
- **Partial Functionality**: Continue with available servers when some fail
- **Error Boundaries**: Isolate server failures to prevent system-wide issues
- **User Notification**: Clear communication about server availability
- **Fallback Servers**: Automatic failover to backup servers

### Integration Points

#### Tool System Integration
- **Dynamic Tool Discovery**: Automatically discover tools from connected MCP servers
- **Tool Registration**: Register discovered tools with the main tool system
- **Tool Execution**: Route tool calls to appropriate MCP servers
- **Result Handling**: Process tool execution results and update UI

#### AI Command Integration
- **Command Routing**: Route AI commands to servers with appropriate capabilities
- **Context Passing**: Pass relevant context to MCP servers for tool execution
- **Result Integration**: Integrate tool results back into AI command responses
- **Performance Monitoring**: Track performance of server-based operations

### Testing Requirements

#### Protocol Compliance Testing
- **MCP Specification Adherence**: Verify strict compliance with MCP protocol
- **Message Format Validation**: Test all message types and formats
- **Error Handling**: Test protocol error scenarios and recovery
- **Version Compatibility**: Test with different MCP protocol versions

#### Connection Testing
- **Connection Reliability**: Test connection stability under various network conditions
- **Reconnection Logic**: Verify automatic reconnection works correctly
- **Load Testing**: Test with multiple concurrent connections and requests
- **Security Testing**: Verify authentication and authorization mechanisms

### Implementation Files
- `ai_menu_2/mcp_protocol.rs` - Core MCP protocol implementation
- `ai_menu_2/mcp_connections.rs` - Connection management and pooling
- `ai_menu_2/mcp_server_registry.rs` - Server discovery and registration
- `ai_menu_2/mcp_health_monitoring.rs` - Health monitoring and heartbeat systems
- `ai_menu_2/mcp_message_handling.rs` - Message processing and correlation
- `ai_menu_2/mcp_security.rs` - Authentication and trust management

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all message processing loops
- **Blazing-fast performance** - efficient WebSocket message handling
- **Production quality** - reliable MCP server integration system

## Bevy Implementation Details

### Component Architecture for MCP Management
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MCPServerManagerPanel {
    pub selected_server: Option<String>,
    pub connection_view: ConnectionView,
    pub show_health_details: bool,
}

#[derive(Component, Reflect)]
pub struct ServerConnectionTask {
    pub server_id: String,
    pub task: Task<Result<MCPServerConnection, MCPError>>,
    pub retry_count: u32,
}

#[derive(Reflect)]
pub enum ConnectionView {
    ServerList,
    ConnectionDetails,
    HealthMonitor,
    ConfigurationEditor,
}
```

### System Architecture with Ordered MCP Operations
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MCPSystemSet {
    Discovery,
    ConnectionManagement,
    MessageProcessing,
    HealthMonitoring,
    UIUpdate,
}

impl Plugin for MCPServerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (
            MCPSystemSet::Discovery,
            MCPSystemSet::ConnectionManagement,
            MCPSystemSet::MessageProcessing,
            MCPSystemSet::HealthMonitoring,
            MCPSystemSet::UIUpdate,
        ).chain())
        .add_systems(Startup, initialize_mcp_system)
        .add_systems(Update, (
            discover_mcp_servers.in_set(MCPSystemSet::Discovery),
            manage_mcp_connections.in_set(MCPSystemSet::ConnectionManagement),
            process_mcp_messages.in_set(MCPSystemSet::MessageProcessing),
            monitor_server_health.in_set(MCPSystemSet::HealthMonitoring),
            update_mcp_ui.in_set(MCPSystemSet::UIUpdate),
        ));
    }
}
```

### Async Connection Management with Task Polling
```rust
fn manage_mcp_connections(
    mut commands: Commands,
    mut connection_manager: ResMut<MCPConnectionManager>,
    mut server_registry: ResMut<MCPServerRegistry>,
    task_pool: Res<AsyncComputeTaskPool>,
    time: Res<Time>,
) {
    // Spawn connection tasks for unconnected servers
    for (server_id, registration) in &server_registry.registered_servers {
        if !connection_manager.active_connections.contains_key(server_id) {
            let server_config = registration.clone();
            let task = task_pool.spawn(async move {
                establish_mcp_connection(server_config).await
            });
            
            commands.spawn(ServerConnectionTask {
                server_id: server_id.clone(),
                task,
                retry_count: 0,
            });
        }
    }
}

fn poll_connection_tasks(
    mut commands: Commands,
    mut connection_tasks: Query<(Entity, &mut ServerConnectionTask)>,
    mut connection_manager: ResMut<MCPConnectionManager>,
    mut mcp_events: EventWriter<MCPServerEvent>,
) {
    for (entity, mut task) in &mut connection_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.task)) {
            match result {
                Ok(connection) => {
                    connection_manager.active_connections.insert(
                        connection.server_id.clone(), 
                        connection.clone()
                    );
                    mcp_events.write(MCPServerEvent::ServerConnected {
                        server_id: connection.server_id,
                        capabilities: connection.capabilities,
                    });
                }
                Err(error) => {
                    if task.retry_count < 3 {
                        task.retry_count += 1;
                        // Respawn with exponential backoff
                        continue;
                    }
                    mcp_events.write(MCPServerEvent::ConnectionFailed {
                        server_id: task.server_id.clone(),
                        error: error.to_string(),
                    });
                }
            }
            commands.entity(entity).despawn();
        }
    }
}
```

### Event-Driven MCP Protocol Handling
```rust
fn handle_mcp_events(
    mut mcp_events: EventReader<MCPServerEvent>,
    mut connection_manager: ResMut<MCPConnectionManager>,
    mut server_registry: ResMut<MCPServerRegistry>,
    mut ui_events: EventWriter<UIUpdateEvent>,
) {
    for event in mcp_events.read() {
        match event {
            MCPServerEvent::ServerConnected { server_id, capabilities } => {
                if let Some(connection) = connection_manager.active_connections.get_mut(server_id) {
                    connection.connection_state = ConnectionState::Ready;
                    connection.capabilities = capabilities.clone();
                }
                
                ui_events.write(UIUpdateEvent::ServerStatusChanged {
                    server_id: server_id.clone(),
                    status: ConnectionState::Ready,
                });
            }
            MCPServerEvent::MessageReceived { server_id, message } => {
                // Process incoming MCP message
                process_mcp_message(server_id, message, &mut connection_manager);
            }
            MCPServerEvent::ConnectionFailed { server_id, error } => {
                warn!("MCP server connection failed: {}: {}", server_id, error);
                
                ui_events.write(UIUpdateEvent::ServerError {
                    server_id: server_id.clone(),
                    error: error.clone(),
                });
            }
        }
    }
}
```

### Health Monitoring with Timer Components
```rust
#[derive(Component, Reflect)]
pub struct HealthCheckTimer {
    pub server_id: String,
    pub timer: Timer,
    pub consecutive_failures: u32,
}

fn monitor_server_health(
    mut health_timers: Query<&mut HealthCheckTimer>,
    mut connection_manager: ResMut<MCPConnectionManager>,
    mut health_events: EventWriter<ServerHealthEvent>,
    time: Res<Time>,
) {
    for mut health_timer in &mut health_timers {
        health_timer.timer.tick(time.delta());
        
        if health_timer.timer.just_finished() {
            if let Some(connection) = connection_manager.active_connections.get(&health_timer.server_id) {
                // Check if connection is responsive
                let health_status = check_connection_health(connection);
                
                match health_status {
                    HealthStatus::Healthy => {
                        health_timer.consecutive_failures = 0;
                    }
                    HealthStatus::Degraded(reason) => {
                        health_timer.consecutive_failures += 1;
                        warn!("MCP server {} degraded: {}", health_timer.server_id, reason);
                    }
                    HealthStatus::Unhealthy(reason) => {
                        health_timer.consecutive_failures += 1;
                        error!("MCP server {} unhealthy: {}", health_timer.server_id, reason);
                        
                        if health_timer.consecutive_failures > 3 {
                            health_events.write(ServerHealthEvent::ServerUnreachable {
                                server_id: health_timer.server_id.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
```

### Flex-Based UI for Server Management
```rust
fn spawn_mcp_server_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            max_width: Val::Px(1000.0),
            flex_grow: 0.0,
            ..default()
        },
    ))
    .with_children(|parent| {
        // Server list panel
        parent.spawn((
            Node {
                width: Val::Px(350.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::right(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(12.0)),
                overflow: Overflow::scroll_y(),
                flex_grow: 0.0,
                ..default()
            },
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        ))
        .with_children(|server_list| {
            // Server list header
            server_list.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(40.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::bottom(Val::Px(12.0)),
                    flex_grow: 0.0,
                    ..default()
                },
            ));
        });
        
        // Server details panel
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                max_width: Val::Px(650.0),
                flex_grow: 1.0,
                ..default()
            },
        ))
        .with_children(|details_panel| {
            // Connection status section
            details_panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(120.0),
                    flex_direction: FlexDirection::Column,
                    background_color: BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    padding: UiRect::all(Val::Px(16.0)),
                    margin: UiRect::bottom(Val::Px(16.0)),
                    flex_grow: 0.0,
                    ..default()
                },
            ));
        });
    });
}
```

### Testing Strategy for MCP Integration
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    
    #[test]
    fn test_mcp_connection_lifecycle() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, MCPServerPlugin));
        
        let server_registration = ServerRegistration {
            server_id: "test-mcp-server".to_string(),
            display_name: "Test MCP Server".to_string(),
            description: "Test server for unit tests".to_string(),
            websocket_url: "ws://localhost:8080/mcp".to_string(),
            authentication_required: false,
            trust_level: TrustLevel::Trusted,
            capabilities: ServerCapabilities::default(),
            registration_source: RegistrationSource::UserManual,
        };
        
        let mut registry = MCPServerRegistry::default();
        registry.registered_servers.insert("test-mcp-server".to_string(), server_registration);
        app.insert_resource(registry);
        
        // Run systems to process connection
        app.update();
        
        let connection_manager = app.world().resource::<MCPConnectionManager>();
        // In real test, would verify connection was attempted
        assert!(connection_manager.active_connections.is_empty()); // No real connection in test
    }
    
    #[test]
    fn test_server_health_monitoring() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, MCPServerPlugin));
        
        let health_timer = HealthCheckTimer {
            server_id: "test-server".to_string(),
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
            consecutive_failures: 0,
        };
        
        app.world_mut().spawn(health_timer);
        app.update();
        
        let timers: Vec<&HealthCheckTimer> = app.world().query::<&HealthCheckTimer>().iter(app.world()).collect();
        assert_eq!(timers.len(), 1);
        assert_eq!(timers[0].server_id, "test-server");
    }
}
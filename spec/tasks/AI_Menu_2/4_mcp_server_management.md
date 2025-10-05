# AI Menu 2 - MCP Server Management System

## Task: Implement Model Context Protocol Server Configuration

### File: `ui/src/ai/mcp/mod.rs` (new file)

Create comprehensive MCP server management with connection handling and protocol integration.

### Implementation Requirements

#### MCP Server Management Interface
- File: `ui/src/ai/mcp/server_manager.rs` (new file, line 1-134)
- "Manage Servers" button functionality for server configuration
- Server discovery and connection status monitoring
- Integration with MCP HTTP servers from experimental features
- Bevy Example Reference: [`async_tasks/async_compute.rs`](../../../docs/bevy/examples/async_tasks/async_compute.rs) - Asynchronous server operations

#### Server Idle Timeout System
- File: `ui/src/ai/mcp/idle_management.rs` (new file, line 1-67)
- Configurable server idle timeout (5 minutes default)
- Automatic server cleanup and resource management
- Connection health monitoring and recovery
- Background maintenance for optimal performance

#### MCP Protocol Implementation
```rust
#[derive(Resource, Debug, Clone)]
pub struct MCPServerConfig {
    pub active_servers: HashMap<String, ServerConnection>,
    pub idle_timeout: Duration,
    pub auto_confirm_tools: bool,
    pub connection_health: HashMap<String, HealthStatus>,
}
```

#### Automatic Tool Confirmation
- File: `ui/src/ai/mcp/auto_confirmation.rs` (new file, line 1-78)
- "Automatically confirm all tool calls" implementation with warning system
- Security implications display and user acknowledgment
- Integration with tools permission system from task 2
- Audit logging for automatic confirmations

### Architecture Notes
- Asynchronous MCP server communication
- Protocol-compliant server discovery and management
- Security-conscious automatic confirmation with warnings
- Resource-efficient connection pooling and cleanup
- Event-driven server status updates

### Integration Points
- `ui/src/ai/tools/permissions.rs` - Tool confirmation integration
- `core/src/runtime/` - Server communication runtime
- `app/src/preferences/` - MCP configuration persistence
- Experimental features integration for HTTP server support

### Event System Integration
```rust
#[derive(Event)]
pub enum MCPEvent {
    ServerConnected(String),
    ServerDisconnected(String),
    IdleTimeoutChanged(Duration),
    AutoConfirmToggled(bool),
    ServerHealthUpdated(String, HealthStatus),
}
```

### Bevy Example References
- **Async Operations**: [`async_tasks/async_compute.rs`](../../../docs/bevy/examples/async_tasks/async_compute.rs) - Server communication
- **Resource Management**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - MCP config resources
- **Event System**: [`ecs/event.rs`](../../../docs/bevy/examples/ecs/event.rs) - Server event patterns

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.
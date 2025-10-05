# AI Menu 2 - Tools Configuration System

## Task: Implement AI Tools Integration and Permission Management

### File: `ui/src/ai/tools/mod.rs` (new file)

Create comprehensive tools configuration system with permission management and tool call automation.

### Implementation Requirements

#### Tools Permission Management System
- File: `ui/src/ai/tools/permissions.rs` (new file, line 1-123)
- Implement tool call permission system with user confirmation workflows
- "Show Tool Call Info" checkbox integration for debugging visibility
- "Reset Tool Confirmations" functionality for permission reset
- Bevy Example Reference: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Button interaction patterns for tool controls

#### Tool Call Information Display
- File: `ui/src/ai/tools/tool_call_info.rs` (new file, line 1-89)
- Real-time tool call information display when enabled
- Tool execution logging and debugging interface
- Integration with existing AI response display system
- Performance monitoring for tool execution times

#### Tool Confirmation System
```rust
#[derive(Resource, Debug, Clone)]
pub struct ToolConfirmationState {
    pub confirmed_tools: HashSet<String>,
    pub auto_confirm_enabled: bool,
    pub show_tool_info: bool,
    pub pending_confirmations: Vec<ToolCallRequest>,
}
```

#### MCP Integration Foundation
- File: `ui/src/ai/tools/mcp_integration.rs` (new file, line 1-67)
- Model Context Protocol server connection management
- Server idle timeout configuration (5 minutes default)
- "Automatically confirm all tool calls" with security warning
- Integration with MCP HTTP servers from experimental features

### Architecture Notes
- Permission-based tool execution with explicit user consent
- Event-driven tool confirmation workflow
- Integration with existing AI provider system
- Security-first approach with opt-in tool automation
- Audit logging for all tool executions

### Integration Points
- `ui/src/ai/provider_bridge.rs` - Tool execution integration
- `app/src/preferences/` - Tool permission persistence  
- `core/src/runtime/` - Sandboxed tool execution environment
- UI components for tool confirmation dialogs

### Event System Integration
```rust
#[derive(Event)]
pub enum ToolEvent {
    PermissionRequested(String, ToolCapabilities),
    PermissionGranted(String),
    ToolExecuted(String, ToolResult),
    ConfirmationReset,
    AutoConfirmToggled(bool),
}
```

### Bevy Example References
- **UI Controls**: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Tool control interactions
- **Permission System**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Resource patterns
- **Event Handling**: [`ecs/event.rs`](../../../docs/bevy/examples/ecs/event.rs) - Tool event patterns

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Bevy Implementation Details

### Component Architecture for Tools Configuration
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ToolsConfigurationPanel {
    pub show_tool_info: bool,
    pub auto_confirm_enabled: bool,
    pub pending_confirmations: VecDeque<String>,
}

#[derive(Component, Reflect)]
pub struct ToolExecutionState {
    pub tool_id: String,
    pub status: ToolExecutionStatus,
    pub start_time: SystemTime,
    pub duration: Option<Duration>,
}
```

### System Architecture for Tool Management
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ToolSystemSet {
    PermissionManagement,
    ToolExecution,
    MCPIntegration,
    UIUpdate,
}

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (
            ToolSystemSet::PermissionManagement,
            ToolSystemSet::ToolExecution,
            ToolSystemSet::MCPIntegration,
            ToolSystemSet::UIUpdate,
        ).chain())
        .add_systems(Update, (
            manage_tool_permissions.in_set(ToolSystemSet::PermissionManagement),
            execute_confirmed_tools.in_set(ToolSystemSet::ToolExecution),
            sync_mcp_tools.in_set(ToolSystemSet::MCPIntegration),
            update_tools_ui.in_set(ToolSystemSet::UIUpdate),
        ));
    }
}
```

### Event-Driven Tool Confirmation
```rust
fn handle_tool_events(
    mut tool_events: EventReader<ToolEvent>,
    mut tool_config: ResMut<ToolConfirmationState>,
    mut ui_events: EventWriter<UIUpdateEvent>,
) {
    for event in tool_events.read() {
        match event {
            ToolEvent::PermissionRequested(tool_id, capabilities) => {
                if !tool_config.auto_confirm_enabled {
                    tool_config.pending_confirmations.push(ToolCallRequest {
                        tool_id: tool_id.clone(),
                        capabilities: capabilities.clone(),
                        timestamp: SystemTime::now(),
                    });
                    ui_events.write(UIUpdateEvent::ToolConfirmationRequired);
                }
            }
            ToolEvent::PermissionGranted(tool_id) => {
                tool_config.confirmed_tools.insert(tool_id.clone());
            }
            ToolEvent::ConfirmationReset => {
                tool_config.confirmed_tools.clear();
                tool_config.pending_confirmations.clear();
            }
        }
    }
}
```

### Flex-Based UI for Tool Configuration
```rust
fn spawn_tools_config_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            max_width: Val::Px(600.0),
            flex_grow: 0.0,
            padding: UiRect::all(Val::Px(16.0)),
            ..default()
        },
    ))
    .with_children(|parent| {
        // Tool confirmation settings
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                flex_grow: 0.0,
                ..default()
            },
        ));
    });
}
```
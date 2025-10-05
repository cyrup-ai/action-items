# Actions Menu - Command Execution Framework

## Implementation Task: Safe Command Execution with Sandboxing and Parameter Handling

### Architecture Overview
Implement a robust command execution system that safely executes user commands with parameter handling, sandboxed environments, and comprehensive error recovery.

### Core Components

#### Command Execution System
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CommandExecutor {
    pub execution_queue: VecDeque<CommandRequest>,
    pub active_executions: HashMap<CommandId, ExecutionContext>,
    pub execution_history: LRUCache<CommandId, ExecutionResult>,
    pub sandbox_config: SandboxConfiguration,
}

#[derive(Reflect, Clone)]
pub struct CommandRequest {
    pub command_id: CommandId,
    pub command_type: CommandType,
    pub parameters: HashMap<String, CommandParameter>,
    pub execution_mode: ExecutionMode,
    pub security_context: SecurityContext,
}

#[derive(Reflect)]
pub enum CommandType {
    Application(ApplicationCommand),
    SystemScript(ScriptCommand),
    Extension(ExtensionCommand),
    AIIntegrated(AICommand),
    File(FileCommand),
}

#[derive(Reflect)]
pub enum ExecutionMode {
    Synchronous,
    Asynchronous,
    Background,
    Sandboxed,
}
```

#### Security and Sandboxing System
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SandboxConfiguration {
    pub allowed_paths: Vec<PathBuf>,
    pub blocked_commands: HashSet<String>,
    pub permission_levels: HashMap<CommandType, PermissionLevel>,
    pub resource_limits: ResourceLimits,
}

#[derive(Reflect)]
pub struct SecurityContext {
    pub user_permissions: PermissionSet,
    pub command_trust_level: TrustLevel,
    pub sandbox_required: bool,
    pub audit_required: bool,
}

#[derive(Reflect)]
pub enum PermissionLevel {
    Public,     // No special permissions needed
    User,       // User-level file access
    Admin,      // Administrative privileges required
    System,     // System-level access
}
```

### Bevy Implementation References

#### Asynchronous Command Processing
- **Async Tasks**: `docs/bevy/examples/async_tasks/async_compute.rs`
  - Background command execution without blocking UI
  - Parallel command processing for independent operations
  - Result collection and UI update coordination

#### Event System for Command Flow
- **Event Handling**: `docs/bevy/examples/ecs/send_and_receive_events.rs`
  - Command execution request events
  - Result notification and error event propagation
  - Status update events for long-running commands

#### Resource Management
- **Resource Usage**: `docs/bevy/examples/ecs/startup_system.rs`
  - Command execution resource initialization
  - Shared execution context across systems
  - Cleanup and resource deallocation

#### Error Handling Systems
- **Error Management**: `docs/bevy/examples/ecs/error_handling.rs`
  - Graceful error recovery during command execution
  - Error event propagation to UI systems
  - Fallback mechanisms for failed commands

### Command Execution Pipeline

#### Request Validation Phase
- **Parameter Validation**: Comprehensive input sanitization and validation
- **Permission Checking**: Verify user permissions against command requirements
- **Security Scanning**: Detect potentially malicious command patterns
- **Resource Availability**: Confirm system resources available for execution

#### Pre-execution Phase
- **Sandbox Preparation**: Initialize sandboxed execution environment if required
- **Context Setup**: Prepare execution context with proper environment variables
- **Resource Allocation**: Reserve necessary system resources for execution
- **Audit Logging**: Log command execution attempt with full context

#### Execution Phase
- **Safe Execution**: Execute command in appropriate environment (sandboxed/direct)
- **Progress Monitoring**: Track execution progress for long-running commands
- **Resource Monitoring**: Monitor CPU, memory, and file system usage
- **Timeout Management**: Enforce execution time limits to prevent hanging

#### Post-execution Phase
- **Result Processing**: Capture and process command output and return codes
- **Resource Cleanup**: Release allocated resources and clean up temporary files
- **State Updates**: Update command history and execution statistics
- **User Notification**: Provide appropriate feedback and result display

### Parameter Handling System

#### Dynamic Parameter System
```rust
#[derive(Reflect, Clone)]
pub enum CommandParameter {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    FilePath(PathBuf),
    URL(Url),
    JSON(serde_json::Value),
    Array(Vec<CommandParameter>),
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ParameterCollector {
    pub required_params: Vec<ParameterDefinition>,
    pub optional_params: Vec<ParameterDefinition>,
    pub collected_values: HashMap<String, CommandParameter>,
    pub validation_errors: Vec<ValidationError>,
}
```

#### Interactive Parameter Collection
- **UI Generation**: Dynamically generate parameter input forms
- **Real-time Validation**: Validate parameters as user inputs them
- **Auto-completion**: Provide intelligent suggestions for parameter values
- **History Integration**: Remember previously used parameter values

### Sandbox Implementation

#### Process Isolation
- **Namespace Isolation**: Use process namespaces for execution isolation
- **File System Restriction**: Limit file system access to specified directories
- **Network Restriction**: Control network access based on command requirements
- **Resource Limits**: Enforce CPU, memory, and disk usage limits

#### Permission Management
- **Principle of Least Privilege**: Grant minimal necessary permissions
- **Dynamic Permission Escalation**: Request additional permissions when needed
- **User Consent**: Require explicit user approval for elevated permissions
- **Audit Trail**: Log all permission grants and usage

### Command Types Implementation

#### Application Commands
- **macOS Application Integration**: Launch and control macOS applications
- **Bundle Identifier Resolution**: Resolve applications by bundle ID
- **Application State Monitoring**: Monitor application launch and termination
- **Inter-Application Communication**: Handle AppleScript and URL schemes

#### System Script Commands
- **Shell Script Execution**: Secure execution of shell scripts and commands
- **Script Validation**: Analyze scripts for potentially dangerous operations
- **Environment Control**: Control script execution environment variables
- **Output Capture**: Capture stdout, stderr, and return codes

#### Extension Commands
- **Plugin System Integration**: Execute commands from installed extensions
- **Extension Sandboxing**: Isolate extension execution from system
- **API Boundary Control**: Control extension access to system APIs
- **Extension Lifecycle**: Manage extension loading and unloading

### Performance Optimization

#### Execution Queue Management
- **Priority Scheduling**: Execute high-priority commands first
- **Batch Processing**: Group similar commands for efficient execution
- **Resource Pooling**: Reuse execution contexts when possible
- **Load Balancing**: Distribute execution load across available resources

#### Result Caching
- **Deterministic Caching**: Cache results for deterministic commands
- **Cache Invalidation**: Intelligent cache invalidation based on dependencies
- **Partial Result Caching**: Cache intermediate results for complex operations
- **Memory Management**: Efficient cache memory usage and cleanup

### Error Recovery System

#### Execution Failure Recovery
```rust
#[derive(Event)]
pub struct CommandExecutionFailed {
    pub command_id: CommandId,
    pub error_type: ExecutionError,
    pub recovery_options: Vec<RecoveryAction>,
    pub retry_possible: bool,
}

#[derive(Reflect)]
pub enum RecoveryAction {
    Retry,
    RetryWithParameters,
    SkipCommand,
    RequestPermissions,
    SwitchToAlternative,
}
```

#### Graceful Degradation
- **Fallback Commands**: Alternative command execution when primary fails
- **Partial Success Handling**: Handle partial success in multi-step commands
- **User Choice Integration**: Allow user to choose recovery actions
- **Automatic Retry**: Intelligent retry with exponential backoff

### Security Implementation

#### Input Sanitization
- **Command Injection Prevention**: Sanitize command parameters to prevent injection
- **Path Traversal Protection**: Validate file paths to prevent directory traversal
- **Special Character Escaping**: Properly escape shell special characters
- **Parameter Type Validation**: Ensure parameters match expected types

#### Audit and Logging
- **Comprehensive Logging**: Log all command execution attempts and results
- **Security Event Logging**: Special logging for security-relevant events
- **Privacy Protection**: Avoid logging sensitive user data
- **Log Rotation**: Efficient log file management and rotation

### Integration Points

#### Search System Coordination
- **Command Discovery**: Interface with search system for command availability
- **Parameter Completion**: Provide parameter suggestions during search
- **Execution Context**: Maintain context from search to execution
- **Result Feedback**: Update search rankings based on execution success

#### UI System Integration
- **Progress Indication**: Real-time progress updates for long-running commands
- **Result Display**: Appropriate display of command results and errors
- **Interactive Parameters**: UI for collecting required parameters
- **Status Notifications**: User notifications for background command completion

### Testing Requirements

#### Security Testing
- **Sandbox Escape Prevention**: Verify commands cannot escape sandbox environment
- **Permission Escalation Testing**: Test prevention of unauthorized permission escalation
- **Input Validation Testing**: Comprehensive testing of parameter validation
- **Injection Attack Prevention**: Test protection against various injection attacks

#### Performance Testing
- **Concurrent Execution**: Test performance with multiple simultaneous commands
- **Resource Usage**: Verify resource usage stays within acceptable limits
- **Memory Leak Prevention**: Test for memory leaks during extended operation
- **Cache Performance**: Verify result caching improves performance appropriately

### Implementation Files
- `actions_menu/command_executor.rs` - Core command execution system
- `actions_menu/sandbox.rs` - Sandboxing and security implementation
- `actions_menu/parameter_collector.rs` - Dynamic parameter collection system
- `actions_menu/execution_events.rs` - Command execution event definitions
- `security/command_security.rs` - Security validation and audit logging

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all command queuing operations
- **Blazing-fast performance** - efficient command execution pipeline
- **Production quality** - complete, secure command execution system## Bevy Implementation Details

### Command Execution Architecture

```rust
#[derive(Component, Reflect)]
pub struct CommandExecutor {
    pub execution_queue: VecDeque<Entity>, // Entities with CommandRequest components
    pub active_executions: HashMap<CommandId, Entity>,
    pub max_concurrent: usize,
    pub sandbox_enabled: bool,
}

#[derive(Component, Reflect)]
pub struct CommandRequest {
    pub command_id: String,
    pub command_type: CommandType,
    pub parameters: HashMap<String, String>,
    pub execution_mode: ExecutionMode,
    pub security_level: SecurityLevel,
    pub created_at: SystemTime,
}

#[derive(Component)]
pub struct ExecutionTask(Task<CommandExecutionResult>);

#[derive(Component, Reflect)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub resource_usage: ResourceUsage,
}
```

### Async Command Execution System

```rust
fn process_command_queue(
    mut commands: Commands,
    mut executor_query: Query<&mut CommandExecutor>,
    command_requests: Query<(Entity, &CommandRequest), Without<ExecutionTask>>,
    mut execution_events: EventWriter<ExecutionEvent>,
) {
    for mut executor in &mut executor_query {
        // Process queued commands up to concurrent limit
        let available_slots = executor.max_concurrent.saturating_sub(executor.active_executions.len());
        
        for _ in 0..available_slots {
            if let Some(request_entity) = executor.execution_queue.pop_front() {
                if let Ok((entity, request)) = command_requests.get(request_entity) {
                    let thread_pool = AsyncComputeTaskPool::get();
                    let request_clone = request.clone();
                    
                    let task = thread_pool.spawn(async move {
                        execute_command_safely(request_clone).await
                    });
                    
                    commands.entity(entity).insert(ExecutionTask(task));
                    executor.active_executions.insert(request.command_id.clone(), entity);
                    
                    execution_events.send(ExecutionEvent::Started {
                        command_id: request.command_id.clone(),
                        entity,
                    });
                }
            } else {
                break;
            }
        }
    }
}

fn poll_execution_tasks(
    mut commands: Commands,
    mut execution_tasks: Query<(Entity, &mut ExecutionTask, &CommandRequest)>,
    mut execution_events: EventWriter<ExecutionEvent>,
    mut executor_query: Query<&mut CommandExecutor>,
) {
    for (entity, mut task, request) in &mut execution_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            // Command execution completed
            commands.entity(entity)
                .remove::<ExecutionTask>()
                .insert(ExecutionResult {
                    success: result.exit_code == 0,
                    output: result.stdout,
                    error_message: if result.stderr.is_empty() { None } else { Some(result.stderr) },
                    execution_time_ms: result.duration.as_millis() as u64,
                    resource_usage: result.resource_usage,
                });
            
            // Remove from active executions
            for mut executor in &mut executor_query {
                executor.active_executions.remove(&request.command_id);
            }
            
            execution_events.send(ExecutionEvent::Completed {
                command_id: request.command_id.clone(),
                success: result.exit_code == 0,
                entity,
            });
        }
    }
}
```

### Security and Sandboxing System

```rust
#[derive(Resource, Reflect)]
pub struct SandboxConfiguration {
    pub enabled: bool,
    pub allowed_paths: Vec<String>,
    pub blocked_commands: HashSet<String>,
    pub resource_limits: ResourceLimits,
    pub permission_levels: HashMap<String, PermissionLevel>,
}

#[derive(Component, Reflect)]
pub struct SecurityValidation {
    pub validated: bool,
    pub risk_level: SecurityRisk,
    pub required_permissions: Vec<Permission>,
    pub validation_errors: Vec<String>,
}

fn validate_command_security(
    mut commands: Commands,
    unvalidated_requests: Query<(Entity, &CommandRequest), Without<SecurityValidation>>,
    sandbox_config: Res<SandboxConfiguration>,
) {
    for (entity, request) in &unvalidated_requests {
        let validation = perform_security_validation(request, &sandbox_config);
        
        commands.entity(entity).insert(SecurityValidation {
            validated: validation.is_safe,
            risk_level: validation.risk_level,
            required_permissions: validation.permissions,
            validation_errors: validation.errors,
        });
    }
}
```

### Command Execution Events

```rust
#[derive(Event, Debug)]
pub enum ExecutionEvent {
    Requested(String, Entity),
    Validated(String, bool),
    Started { command_id: String, entity: Entity },
    Progress { command_id: String, percent: f32 },
    Completed { command_id: String, success: bool, entity: Entity },
    Failed { command_id: String, error: String, entity: Entity },
}

fn handle_execution_events(
    mut execution_events: EventReader<ExecutionEvent>,
    mut ui_events: EventWriter<UiNotificationEvent>,
    execution_results: Query<&ExecutionResult>,
) {
    for event in execution_events.read() {
        match event {
            ExecutionEvent::Completed { command_id, success, entity } => {
                if let Ok(result) = execution_results.get(*entity) {
                    ui_events.send(UiNotificationEvent::ExecutionComplete {
                        command: command_id.clone(),
                        success: *success,
                        output: result.output.clone(),
                    });
                }
            },
            ExecutionEvent::Failed { command_id, error, .. } => {
                ui_events.send(UiNotificationEvent::ExecutionFailed {
                    command: command_id.clone(),
                    error: error.clone(),
                });
            },
            _ => {}
        }
    }
}
```
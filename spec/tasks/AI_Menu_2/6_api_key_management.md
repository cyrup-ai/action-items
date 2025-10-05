# AI Menu 2 - Custom API Key Management System

## Task: Implement Secure API Key Storage and Routing Configuration

### File: `ui/src/ai/api_keys/mod.rs` (new file)

Create comprehensive API key management with secure storage and intelligent routing.

### Implementation Requirements

#### API Key Storage System
- File: `ui/src/ai/api_keys/storage.rs` (new file, line 1-123)
- Secure API key storage using system keychain integration
- Provider-specific key management (Anthropic, Google, OpenAI, OpenRouter)
- Key validation and authentication testing
- Bevy Example Reference: [`reflection/reflection.rs`](../../../docs/bevy/examples/reflection/reflection.rs) - Secure configuration patterns

#### Request Routing System
- File: `ui/src/ai/api_keys/routing.rs` (new file, line 1-89)
- Intelligent request routing based on provider capabilities
- Anthropic/Google/OpenAI: Raycast server routing
- OpenRouter: Direct provider server routing  
- Cost optimization and usage tracking per provider

#### Provider Configuration Interface
```rust
#[derive(Resource, Debug, Clone)]
pub struct APIKeyConfig {
    pub provider_keys: HashMap<String, SecureKey>,
    pub routing_rules: HashMap<String, RoutingRule>,
    pub usage_tracking: HashMap<String, UsageMetrics>,
    pub validation_status: HashMap<String, ValidationResult>,
}
```

#### Usage Tracking and Analytics
- File: `ui/src/ai/api_keys/usage_tracking.rs` (new file, line 1-67)
- Per-provider usage monitoring and cost tracking
- Request volume analytics and optimization suggestions
- Integration with billing systems and cost alerts
- Privacy-conscious analytics with user control

### Architecture Notes
- System keychain integration for secure key storage
- Provider-agnostic routing with specific implementations
- Real-time key validation and status monitoring
- Cost-conscious request optimization
- Audit logging for all API key operations

### Integration Points
- System keychain APIs (Keychain Services, Windows Credential Store, Secret Service)
- `ui/src/ai/provider_bridge.rs` - Request routing integration
- `app/src/preferences/` - API key preference management
- Cost tracking and billing system integration

### Security Implementation
- File: `ui/src/ai/api_keys/security.rs` (new file, line 1-78)
- Encrypted API key storage and transmission
- Key access logging and audit trails
- Secure key validation without exposure
- Automatic key rotation support where available

### Event System Integration
```rust
#[derive(Event)]
pub enum APIKeyEvent {
    KeyAdded(String, ProviderType),
    KeyValidated(String, ValidationResult),
    UsageUpdated(String, UsageMetrics),
    RoutingRuleChanged(String, RoutingRule),
}
```

### Bevy Example References
- **Secure Storage**: [`reflection/reflection.rs`](../../../docs/bevy/examples/reflection/reflection.rs) - Configuration security
- **Resource Management**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - API key resources
- **Async Operations**: [`async_tasks/async_compute.rs`](../../../docs/bevy/examples/async_tasks/async_compute.rs) - Key validation

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Bevy Implementation Details

### Component Architecture for API Key Management
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct APIKeyManagerPanel {
    pub selected_provider: Option<String>,
    pub validation_status: HashMap<String, ValidationResult>,
    pub show_usage_stats: bool,
}

#[derive(Component, Reflect)]
pub struct APIKeyValidationTask {
    pub provider: String,
    pub task: Task<Result<ValidationResult, APIError>>,
}
```

### System Architecture with Security Focus
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum APIKeySystemSet {
    KeyValidation,
    UsageTracking,
    SecurityAudit,
    UIUpdate,
}

impl Plugin for APIKeyPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (
            APIKeySystemSet::KeyValidation,
            APIKeySystemSet::UsageTracking,
            APIKeySystemSet::SecurityAudit,
            APIKeySystemSet::UIUpdate,
        ).chain())
        .add_systems(Update, (
            validate_api_keys.in_set(APIKeySystemSet::KeyValidation),
            track_usage_metrics.in_set(APIKeySystemSet::UsageTracking),
            audit_key_access.in_set(APIKeySystemSet::SecurityAudit),
            update_api_key_ui.in_set(APIKeySystemSet::UIUpdate),
        ));
    }
}
```

### Async Key Validation with Task Management
```rust
fn spawn_key_validation_tasks(
    mut commands: Commands,
    api_config: Res<APIKeyConfig>,
    task_pool: Res<AsyncComputeTaskPool>,
) {
    for (provider, key) in &api_config.provider_keys {
        if requires_validation(key) {
            let provider_clone = provider.clone();
            let key_clone = key.clone();
            let task = task_pool.spawn(async move {
                validate_provider_key(provider_clone, key_clone).await
            });
            
            commands.spawn(APIKeyValidationTask {
                provider: provider.clone(),
                task,
            });
        }
    }
}

fn poll_validation_tasks(
    mut commands: Commands,
    mut validation_tasks: Query<(Entity, &mut APIKeyValidationTask)>,
    mut api_config: ResMut<APIKeyConfig>,
    mut api_events: EventWriter<APIKeyEvent>,
) {
    for (entity, mut task) in &mut validation_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.task)) {
            api_config.validation_status.insert(task.provider.clone(), result.clone());
            api_events.write(APIKeyEvent::KeyValidated(task.provider.clone(), result));
            commands.entity(entity).despawn();
        }
    }
}
```

### Flex-Based UI for Key Management
```rust
fn spawn_api_key_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            max_width: Val::Px(700.0),
            flex_grow: 0.0,
            padding: UiRect::all(Val::Px(16.0)),
            ..default()
        },
    ))
    .with_children(|parent| {
        // Provider list with key status
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
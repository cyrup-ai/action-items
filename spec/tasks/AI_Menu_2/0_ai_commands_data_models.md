# AI Menu 2 - AI Commands Data Models and Configuration System

## Task: Implement AI Commands Configuration Data Structures

### File: `ui/src/ai/commands/mod.rs` (new file)

Create comprehensive data models for AI Commands with custom instructions, model selection, and template management.

### Implementation Requirements

#### AI Commands Configuration Resource
- File: `ui/src/ai/commands/config.rs` (new file, line 1-89)
- Implement `AICommandsConfig` resource with default model selection
- Custom instruction template storage and management
- Integration with provider-specific model capabilities (Gemini 2.5 Pro, etc.)
- Bevy Example Reference: [`reflection/reflection.rs`](../../../docs/bevy/examples/reflection/reflection.rs) - Configuration serialization patterns

#### AI Command Template System
- File: `ui/src/ai/commands/templates.rs` (new file, line 1-134)
- Template storage for common tasks (writing improvement, code enhancement)
- Custom instruction parsing and validation
- Dynamic template loading and hot-reloading support
- Integration with existing AI provider system from AI_Menu

#### Model Selection and Management
```rust
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct AICommandsConfig {
    pub default_model: AIModel,
    pub provider_config: HashMap<String, ProviderConfig>,
    pub custom_commands: Vec<CustomCommand>,
    pub template_cache: HashMap<String, CommandTemplate>,
}
```

#### Custom Command Structure
- File: `ui/src/ai/commands/custom_command.rs` (new file, line 1-78)
- Individual command configuration with specific models and instructions
- Per-command model override capability
- Command execution history and performance metrics
- Integration with action execution system from Main Menu

#### Provider Integration Layer
- File: `ui/src/ai/commands/provider_integration.rs` (new file, line 1-67)
- Provider-specific model configuration (Google Gemini, OpenAI, Anthropic)
- Model capability detection and feature availability
- Dynamic provider icon loading and display
- Cost tracking and usage optimization per provider

### Architecture Notes
- Resource-based configuration with Bevy's change detection
- Template system with hot-reloading for development workflow
- Provider-agnostic command interface with specific implementations
- Integration with existing AI configuration and API key management
- Zero-allocation template processing where possible

### Integration Points
- `ui/src/settings/ai_config/` - Integration with base AI configuration system
- `core/src/runtime/` - Deno runtime integration for command execution (lines 89-167)
- `ui/src/ai/provider_bridge.rs` - Provider communication integration
- `app/src/preferences/` - AI command preference persistence

### Command Template Format
- File: `ui/src/ai/commands/template_parser.rs` (new file, line 1-56)
- Structured template format with variable substitution
- Input validation and sanitization for command parameters
- Template versioning and migration support
- Custom command sharing and import/export functionality

### Event System Integration
```rust
#[derive(Event)]
pub enum AICommandEvent {
    CommandCreated(CustomCommand),
    CommandExecuted(String, AIModel),
    ModelChanged(AIModel),
    TemplateLoaded(String),
    ProviderConfigUpdated(String, ProviderConfig),
}
```

### Bevy Example References
- **Configuration**: [`reflection/reflection.rs`](../../../docs/bevy/examples/reflection/reflection.rs) - Settings serialization
- **Resource Management**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Resource patterns
- **Asset Loading**: [`asset/asset_loading.rs`](../../../docs/bevy/examples/asset/asset_loading.rs) - Template loading
- **Event System**: [`ecs/event.rs`](../../../docs/bevy/examples/ecs/event.rs) - AI command events

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Bevy Implementation Details

### Component Architecture for AI Commands
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AICommandsPanel {
    pub active_command: Option<String>,
    pub template_filter: TemplateFilter,
    pub edit_mode: bool,
}

#[derive(Component, Reflect)]
pub struct CommandExecutionState {
    pub command_id: String,
    pub model: AIModel,
    pub progress: ExecutionProgress,
    pub start_time: SystemTime,
}

#[derive(Reflect)]
pub enum TemplateFilter {
    All,
    Category(CommandCategory),
    Recent,
    Favorites,
}
```

### System Architecture with Ordered Execution
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AICommandSystemSet {
    ConfigLoad,
    TemplateManagement,
    CommandExecution,
    ProviderSync,
    UIUpdate,
}

impl Plugin for AICommandsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (
            AICommandSystemSet::ConfigLoad,
            AICommandSystemSet::TemplateManagement,
            AICommandSystemSet::CommandExecution,
            AICommandSystemSet::ProviderSync,
            AICommandSystemSet::UIUpdate,
        ).chain())
        .add_systems(Startup, initialize_ai_commands_system)
        .add_systems(Update, (
            load_command_templates.in_set(AICommandSystemSet::ConfigLoad),
            manage_custom_commands.in_set(AICommandSystemSet::TemplateManagement),
            execute_ai_commands.in_set(AICommandSystemSet::CommandExecution),
            sync_provider_configs.in_set(AICommandSystemSet::ProviderSync),
            update_commands_ui.in_set(AICommandSystemSet::UIUpdate),
        ));
    }
}
```

### Resource Management for AI Commands
```rust
fn initialize_ai_commands_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Initialize AI Commands configuration resource
    let commands_config = AICommandsConfig {
        default_model: AIModel::Claude3Haiku,
        provider_config: HashMap::new(),
        custom_commands: Vec::new(),
        template_cache: HashMap::new(),
    };
    
    commands.insert_resource(commands_config);
    commands.insert_resource(CommandTemplateLoader::new());
}

fn load_command_templates(
    mut commands: Commands,
    mut template_loader: ResMut<CommandTemplateLoader>,
    mut ai_config: ResMut<AICommandsConfig>,
    asset_server: Res<AssetServer>,
) {
    if template_loader.needs_loading() {
        // Load default templates asynchronously
        let template_task = AsyncComputeTaskPool::get().spawn(async move {
            load_default_command_templates().await
        });
        
        commands.spawn(TemplateLoadingTask(template_task));
    }
}
```

### Event-Driven Command Management
```rust
fn handle_ai_command_events(
    mut command_events: EventReader<AICommandEvent>,
    mut ai_config: ResMut<AICommandsConfig>,
    mut execution_query: Query<&mut CommandExecutionState>,
    time: Res<Time>,
) {
    for event in command_events.read() {
        match event {
            AICommandEvent::CommandCreated(command) => {
                ai_config.custom_commands.push(command.clone());
                info!("Custom AI command created: {}", command.name);
            }
            AICommandEvent::CommandExecuted(command_id, model) => {
                // Update execution tracking
                for mut execution_state in &mut execution_query {
                    if execution_state.command_id == *command_id {
                        execution_state.progress = ExecutionProgress::Completed;
                        execution_state.model = model.clone();
                    }
                }
            }
            AICommandEvent::ModelChanged(new_model) => {
                ai_config.default_model = new_model.clone();
            }
        }
    }
}
```

### Async Template Loading with Task Management
```rust
#[derive(Component)]
pub struct TemplateLoadingTask(Task<Vec<CommandTemplate>>);

fn poll_template_loading_tasks(
    mut commands: Commands,
    mut loading_tasks: Query<(Entity, &mut TemplateLoadingTask)>,
    mut ai_config: ResMut<AICommandsConfig>,
) {
    for (entity, mut task) in &mut loading_tasks {
        if let Some(templates) = block_on(future::poll_once(&mut task.0)) {
            // Update template cache with loaded templates
            for template in templates {
                ai_config.template_cache.insert(template.name.clone(), template);
            }
            
            commands.entity(entity).despawn();
            info!("AI command templates loaded successfully");
        }
    }
}
```

### Flex-Based UI for Command Configuration
```rust
fn spawn_ai_commands_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            max_width: Val::Px(900.0),
            flex_grow: 0.0, // Prevent expansion
            ..default()
        },
    ))
    .with_children(|parent| {
        // Left panel: Command list
        parent.spawn((
            Node {
                width: Val::Px(300.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::right(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(12.0)),
                overflow: Overflow::scroll_y(),
                flex_grow: 0.0,
                ..default()
            },
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        ));
        
        // Right panel: Command editor
        parent.spawn((
            Node {
                width: Val::Percent(100.0), // Fill remaining space
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                max_width: Val::Px(600.0),
                flex_grow: 1.0, // Allow expansion of editor area
                ..default()
            },
        ));
    });
}
```

### Query Optimization with Change Detection
```rust
fn update_commands_ui(
    commands_query: Query<&AICommandsConfig, Changed<AICommandsConfig>>,
    mut ui_events: EventWriter<UIRefreshEvent>,
) {
    for config in &commands_query {
        // Only update UI when commands configuration changes
        ui_events.write(UIRefreshEvent::CommandsConfigChanged {
            custom_commands_count: config.custom_commands.len(),
            templates_loaded: config.template_cache.len(),
            default_model: config.default_model.clone(),
        });
    }
}
```

### Testing Strategy for Command System
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    
    #[test]
    fn test_ai_commands_config_serialization() {
        let config = AICommandsConfig {
            default_model: AIModel::GPT4,
            provider_config: HashMap::new(),
            custom_commands: vec![],
            template_cache: HashMap::new(),
        };
        
        let serialized = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: AICommandsConfig = serde_json::from_str(&serialized)
            .expect("Failed to deserialize");
        
        assert_eq!(config.default_model, deserialized.default_model);
    }
    
    #[test]
    fn test_command_execution_lifecycle() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AICommandsPlugin));
        
        let execution_state = CommandExecutionState {
            command_id: "test-command".to_string(),
            model: AIModel::Claude3Haiku,
            progress: ExecutionProgress::Pending,
            start_time: SystemTime::now(),
        };
        
        app.world_mut().spawn(execution_state);
        
        // Simulate command execution event
        app.world_mut().send_event(AICommandEvent::CommandExecuted(
            "test-command".to_string(),
            AIModel::Claude3Haiku,
        ));
        
        app.update();
        
        // Verify execution state updated
        let execution_query = app.world().query::<&CommandExecutionState>();
        let execution_states: Vec<&CommandExecutionState> = execution_query.iter(app.world()).collect();
        assert_eq!(execution_states.len(), 1);
        assert_eq!(execution_states[0].progress, ExecutionProgress::Completed);
    }
}
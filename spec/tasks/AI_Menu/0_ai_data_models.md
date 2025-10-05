# AI Menu - Data Models and State Management

## Task: Implement Comprehensive AI Configuration Data Structures

### File: `ui/src/settings/ai/mod.rs` (new file)

Create comprehensive data models for AI feature configuration with zero-allocation patterns, blazing-fast state management, and secure API key handling.

### Implementation Requirements

#### Core AI Settings Structure
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct AISettings {
    pub quick_ai: QuickAISettings,
    pub ai_model: AIModelSettings,
    pub ai_chat: AIChatSettings,
    pub provider: ProviderSettings,
    pub text_preferences: TextPreferences,
    pub privacy_indicators: PrivacyIndicators,
}
```

#### Quick AI Configuration
- File: `ui/src/settings/ai/quick_ai.rs` (new file, line 1-67)
- Implement `QuickAISettings` with trigger configuration
- Tab-to-Ask-AI hotkey management
- Root search hint display settings
- Integration with main search interface

#### AI Model Management
- File: `ui/src/settings/ai/ai_model.rs` (new file, line 1-89)
- Implement `AIModelSettings` for provider and model selection
- "Sonar Reasoning Pro" and other model configurations
- Web search capability toggles
- Model-specific feature availability tracking

#### AI Chat Configuration
- File: `ui/src/settings/ai/chat_settings.rs` (new file, line 1-123)
- Implement `AIChatSettings` with hotkey assignments
- Dedicated chat window timeout configuration
- Auto-close settings ("After 30 minutes")
- Chat session persistence and state management

#### Provider and Authentication
- File: `ui/src/settings/ai/provider_settings.rs` (new file, line 1-134)
- Implement `ProviderSettings` for AI service configuration
- "CYRUP (openai)" and other provider selections
- Secure API key storage and validation
- Provider-specific capability detection

#### Privacy and Security Framework
- File: `ui/src/settings/ai/privacy.rs` (new file, line 1-78)
- Implement `PrivacyIndicators` with data collection status
- Full control, No collection, Encrypted status tracking
- Data handling transparency and user control
- Audit trail for privacy setting changes

### Architecture Notes
- Use Bevy's `Reflect` trait for all AI settings structures
- Implement secure storage for API keys using system keychain
- Zero-allocation serialization with encrypted sensitive data
- Integration with existing AI systems if present

### Integration Points
- AI service APIs for provider communication
- Hotkey system for AI chat activation
- Main search interface for Quick AI integration
- Settings persistence with encrypted API key storage

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Bevy Implementation Details

### Component Architecture

```rust
// Core AI configuration components
#[derive(Component, Reflect, Clone, Debug)]
pub struct AiProvider {
    pub id: String,
    pub name: String,
    pub api_endpoint: Option<String>,
    pub status: ProviderStatus,
    pub capabilities: Vec<ProviderCapability>,
}

#[derive(Component, Reflect, Clone)]
pub struct AiModel {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub display_name: String,
    pub model_type: ModelType,
    pub context_length: u32,
    pub pricing: Option<ModelPricing>,
    pub status: ModelStatus,
}

#[derive(Component, Reflect)]
pub struct AiConfiguration {
    pub active_provider: String,
    pub active_model: String,
    pub api_key: Option<String>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub custom_instructions: Option<String>,
}

// UI state components
#[derive(Component, Reflect)]
pub struct AiMenuState {
    pub visible: bool,
    pub current_tab: AiMenuTab,
    pub selected_provider_index: usize,
    pub selected_model_index: usize,
    pub editing_config: bool,
}

// Data validation components
#[derive(Component, Reflect)]
pub struct ConfigValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

### Resource Management

```rust
// Global AI system resources
#[derive(Resource, Reflect)]
pub struct AiProviderRegistry {
    pub providers: Vec<Entity>, // Entities with AiProvider components
    pub models_by_provider: HashMap<String, Vec<Entity>>,
    pub default_provider: Option<String>,
}

#[derive(Resource, Reflect)]
pub struct AiConfigurationResource {
    pub current_config: Entity, // Entity with AiConfiguration component
    pub config_history: VecDeque<Entity>,
    pub auto_save: bool,
}

#[derive(Resource)]
pub struct AiApiClients {
    pub openai: Option<Arc<dyn AiApiClient>>,
    pub anthropic: Option<Arc<dyn AiApiClient>>,
    pub google: Option<Arc<dyn AiApiClient>>,
    pub local: Option<Arc<dyn AiApiClient>>,
}
```

### Event-Driven Architecture

```rust
// AI configuration events
#[derive(Event, Debug, Clone)]
pub enum AiConfigEvent {
    ProviderSelected(String),
    ModelSelected(String, String), // provider_id, model_id
    ConfigurationChanged(Entity),
    ApiKeyUpdated(String),
    ValidateConfiguration(Entity),
    SaveConfiguration,
    LoadConfiguration(String),
    ResetToDefaults,
}

#[derive(Event, Debug)]
pub enum AiProviderEvent {
    ProviderRegistered(String),
    ProviderStatusChanged(String, ProviderStatus),
    ModelListUpdated(String, Vec<String>),
    ApiTestCompleted(String, bool, Option<String>),
}

#[derive(Event, Debug)]
pub enum AiValidationEvent {
    ValidationRequested(Entity),
    ValidationCompleted(Entity, bool, Vec<String>),
    ConfigurationError(Entity, String),
}
```

### System Architecture with SystemSets

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AiMenuSystemSet {
    InputHandling,
    ConfigValidation,
    DataSerialization,
    UIUpdates,
}

impl Plugin for AiMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<AiProviderRegistry>()
            .init_resource::<AiConfigurationResource>()
            .init_resource::<AiApiClients>()
            
            // Events
            .add_event::<AiConfigEvent>()
            .add_event::<AiProviderEvent>()
            .add_event::<AiValidationEvent>()
            
            // Component registration for reflection
            .register_type::<AiProvider>()
            .register_type::<AiModel>()
            .register_type::<AiConfiguration>()
            .register_type::<AiMenuState>()
            .register_type::<ConfigValidation>()
            
            // System sets ordering
            .configure_sets(Update, (
                AiMenuSystemSet::InputHandling,
                AiMenuSystemSet::ConfigValidation,
                AiMenuSystemSet::DataSerialization,
                AiMenuSystemSet::UIUpdates,
            ).chain())
            
            // Systems
            .add_systems(Startup, setup_ai_system)
            .add_systems(Update, (
                handle_ai_config_events.in_set(AiMenuSystemSet::InputHandling),
                validate_ai_configurations.in_set(AiMenuSystemSet::ConfigValidation),
                serialize_ai_config.in_set(AiMenuSystemSet::DataSerialization),
                update_ai_menu_ui.in_set(AiMenuSystemSet::UIUpdates),
            ));
    }
}
```

### Async Integration Patterns

```rust
#[derive(Component)]
pub struct AiConfigValidationTask(Task<ConfigValidationResult>);

fn validate_ai_configurations(
    mut commands: Commands,
    mut validation_events: EventReader<AiValidationEvent>,
    mut completed_tasks: Query<(Entity, &mut AiConfigValidationTask)>,
    ai_configs: Query<&AiConfiguration>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    
    // Start new validation tasks
    for event in validation_events.read() {
        if let AiValidationEvent::ValidationRequested(entity) = event {
            if let Ok(config) = ai_configs.get(*entity) {
                let config_clone = config.clone();
                let task = thread_pool.spawn(async move {
                    validate_config_async(config_clone).await
                });
                
                commands.entity(*entity).insert(AiConfigValidationTask(task));
            }
        }
    }
    
    // Poll completed tasks
    for (entity, mut task) in &mut completed_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity)
                .remove::<AiConfigValidationTask>()
                .insert(ConfigValidation {
                    is_valid: result.is_valid,
                    errors: result.errors,
                    warnings: result.warnings,
                });
        }
    }
}
```

### Query Optimization with Changed<T>

```rust
fn update_ai_menu_ui(
    mut commands: Commands,
    ai_menu_query: Query<(Entity, &AiMenuState), Changed<AiMenuState>>,
    config_query: Query<&AiConfiguration, Changed<AiConfiguration>>,
    validation_query: Query<&ConfigValidation, Changed<ConfigValidation>>,
    mut ui_events: EventWriter<UiUpdateEvent>,
) {
    // Only process entities where AI menu state has changed
    for (entity, menu_state) in &ai_menu_query {
        ui_events.send(UiUpdateEvent::AiMenuStateChanged {
            entity,
            visible: menu_state.visible,
            tab: menu_state.current_tab,
        });
    }
    
    // React to configuration changes
    for config in &config_query {
        ui_events.send(UiUpdateEvent::ConfigurationUpdated {
            provider: config.active_provider.clone(),
            model: config.active_model.clone(),
        });
    }
}
```

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;
    
    #[test]
    fn test_ai_provider_registration() {
        let mut app = App::new();
        app.add_plugins(AiMenuPlugin);
        
        let mut system_state = SystemState::<(
            Commands,
            ResMut<AiProviderRegistry>,
        )>::new(&mut app.world);
        
        let (mut commands, mut registry) = system_state.get_mut(&mut app.world);
        
        // Spawn test provider
        let provider_entity = commands.spawn(AiProvider {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
            api_endpoint: None,
            status: ProviderStatus::Available,
            capabilities: vec![],
        }).id();
        
        registry.providers.push(provider_entity);
        
        assert_eq!(registry.providers.len(), 1);
    }
    
    #[test]
    fn test_config_validation() {
        let mut app = App::new();
        app.add_plugins(AiMenuPlugin);
        
        // Test configuration validation logic
        let config = AiConfiguration {
            active_provider: "openai".to_string(),
            active_model: "gpt-4".to_string(),
            api_key: Some("test_key".to_string()),
            temperature: 0.7,
            max_tokens: Some(1000),
            custom_instructions: None,
        };
        
        let entity = app.world.spawn(config).id();
        app.world.send_event(AiValidationEvent::ValidationRequested(entity));
        
        // Run validation systems
        app.update();
        
        // Check that validation was applied
        assert!(app.world.get::<ConfigValidation>(entity).is_some());
    }
}
```
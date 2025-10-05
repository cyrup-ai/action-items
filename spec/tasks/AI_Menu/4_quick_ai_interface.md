# AI Menu - Quick AI Interface System

## Implementation Task: Tab-to-Ask-AI Trigger and Configuration System

### Architecture Overview
Implement the Quick AI interface system that enables instant AI responses from root search with configurable triggers, hints, and model selection.

### Core Components

#### Quick AI Configuration Component
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct QuickAIConfiguration {
    pub trigger_method: TriggerMethod,
    pub show_hint_in_search: bool,
    pub selected_model: String,
    pub web_search_enabled: bool,
    pub provider_icon: Handle<Image>,
}

#[derive(Reflect, Default)]
pub enum TriggerMethod {
    #[default]
    TabToAskAI,
    DoubleTap,
    Keyword,
    Hotkey,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct QuickAITriggerDropdown {
    pub expanded: bool,
    pub selected_index: usize,
    pub options: Vec<TriggerOption>,
}

#[derive(Reflect)]
pub struct TriggerOption {
    pub display_name: String,
    pub trigger_method: TriggerMethod,
    pub description: String,
}
```

#### Search Integration System
- **Root Search Integration**: Seamless integration with main launcher search
- **Trigger Detection**: Monitor Tab key presses in search context
- **Hint Display**: Optional "Tab to Ask AI" hint in search interface
- **Model Selection**: Dynamic model dropdown with provider branding

### Bevy Implementation References

#### Dropdown Menu System
- **UI Dropdown**: `docs/bevy/examples/ui/button.rs`
  - Expandable trigger method dropdown implementation
  - Selection state management for trigger options
  - Smooth expand/collapse animations

#### Input Detection System
- **Keyboard Input**: `docs/bevy/examples/input/keyboard_input_events.rs`
  - Tab key detection for trigger activation
  - Input event filtering for search context
  - Modifier key combination handling

#### Dynamic UI Updates
- **UI Layout**: `docs/bevy/examples/ui/flex_layout.rs`
  - Two-column layout for labels and controls
  - Dynamic content updates for model selection
  - Flexible width adjustments for dropdown content

#### Asset Loading for Icons
- **Image Assets**: `docs/bevy/examples/asset/asset_loading.rs`
  - Provider icon loading and caching
  - Dynamic icon updates based on model selection
  - Efficient asset management for UI elements

### Quick AI Interface Layout

#### Trigger Configuration Section
- **Label**: "Trigger" positioned in left column
- **Dropdown**: "Tab to Ask AI" (default) in right column
- **Width**: Spans majority of right column width
- **Style**: Dark background (#2a2a2a) with down arrow indicator
- **Options**: Tab to Ask AI, Double Tap, Keyword Trigger, Hotkey

#### Hint Display Toggle
- **Component**: Checkbox with text label
- **Label**: "Show Ask AI hint in root search"
- **State**: Checked by default (visible checkmark)
- **Position**: Below trigger dropdown, full width
- **Functionality**: Controls hint visibility in main search interface

#### Model Selection System
- **Label**: "Quick AI Model" in left column
- **Dropdown**: Provider-specific model selection
- **Current**: "Sonar Reasoning Pro" with provider icon
- **Icon**: Left-aligned provider branding within dropdown
- **Info Button**: Circular "i" for model capabilities and details

#### Web Search Integration
- **Component**: Checkbox below model selection
- **Label**: "Web Search" capability toggle
- **State**: Checked by default (visible checkmark)
- **Position**: Left-aligned with model dropdown
- **Functionality**: Enables/disables web search capability for selected model

### Data Integration Points

#### Search System Integration
```rust
#[derive(Event)]
pub struct QuickAITriggered {
    pub query: String,
    pub search_context: SearchContext,
    pub trigger_source: TriggerMethod,
}

#[derive(Event)]
pub struct ModelSelectionChanged {
    pub previous_model: String,
    pub new_model: String,
    pub provider_changed: bool,
}
```

#### Provider Model Management
- **Dynamic Model List**: Populate dropdown from available AI providers
- **Provider Icons**: Load and cache provider-specific branding
- **Capability Detection**: Show/hide web search toggle based on model capabilities
- **Authentication**: Verify model access and update availability

### Interactive Behavior Implementation

#### Trigger Method Selection
- **Dropdown Interaction**: Click to expand available trigger options
- **Selection Handling**: Update configuration on option selection
- **Visual Feedback**: Highlight selected option with distinct styling
- **Keyboard Navigation**: Arrow key navigation through options

#### Model Selection Interface
- **Provider Integration**: Real-time model list from active providers
- **Icon Loading**: Asynchronous provider icon loading with fallbacks
- **Capability Updates**: Dynamic web search toggle based on model support
- **Info Dialog**: Expandable model details and capability information

#### Search Hint System
- **Root Search Integration**: Conditional hint display in main search
- **Visual Styling**: Subtle hint text that doesn't interfere with search
- **Animation**: Smooth fade-in/out when toggling hint display
- **Context Awareness**: Show hint only when appropriate (empty search, etc.)

### Visual Implementation Details

#### Dropdown Styling
- **Background**: Dark theme (#2a2a2a) with 1px border
- **Text Color**: White (#ffffff) for selected option
- **Arrow Icon**: Down-facing chevron on right side
- **Hover State**: Slightly lighter background on hover
- **Expanded State**: Dropdown list with scroll if needed

#### Icon Integration
- **Provider Icons**: 16x16px icons within dropdown
- **Loading States**: Placeholder while icons load
- **Error Fallbacks**: Default icon for missing provider icons
- **High DPI Support**: Scalable vector icons for retina displays

### Performance Considerations

#### Efficient Model Loading
- **Lazy Loading**: Load provider models only when dropdown expands
- **Caching Strategy**: Cache frequently accessed model information
- **Background Updates**: Refresh model availability asynchronously
- **Memory Management**: Release unused provider icons periodically

#### Real-time Updates
- **Change Detection**: Efficient updates only when configuration changes
- **Event Batching**: Batch related configuration changes to minimize updates
- **UI Debouncing**: Prevent excessive UI updates during rapid changes
- **Asset Preloading**: Preload common provider icons for instant display

### Integration Points

#### Search System Coordination
- Monitor search input for trigger detection
- Integrate with main launcher search architecture
- Coordinate with favorites and contextual action systems
- Handle search context handoff to AI processing

#### AI Provider Communication
- Interface with multiple AI provider systems simultaneously
- Handle authentication state changes gracefully
- Provide fallback options when primary provider unavailable
- Monitor provider service health and capabilities

### Testing Requirements

#### Functional Testing
- Verify trigger method selection updates configuration correctly
- Test model selection updates provider integration
- Validate web search toggle affects AI capabilities appropriately
- Confirm hint display toggle updates root search interface

#### Integration Testing
- Test Quick AI trigger from root search interface
- Verify provider icon loading and display
- Validate model capability detection and UI updates
- Test configuration persistence across application sessions

### Implementation Files
- `ai_menu/quick_ai_config.rs` - Quick AI configuration components
- `ai_menu/trigger_systems.rs` - Trigger detection and handling systems
- `ai_menu/model_selection.rs` - Model dropdown and provider integration
- `ui/quick_ai_interface.rs` - UI components and layout systems

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)  
- **Zero-allocation patterns** for all trigger detection loops
- **Blazing-fast performance** - efficient input handling only
- **Production quality** - complete, robust implementation## Bevy Implementation Details

### Quick AI Component System

```rust
#[derive(Component, Reflect)]
pub struct QuickAiTriggerSystem {
    pub active_triggers: Vec<TriggerMethod>,
    pub search_context: Option<Entity>,
    pub last_trigger_time: Option<Instant>,
    pub debounce_duration: Duration,
}

#[derive(Component, Reflect)]
pub struct ModelSelectionUI {
    pub dropdown_entity: Entity,
    pub selected_model: String,
    pub provider_icon_entity: Option<Entity>,
    pub web_search_enabled: bool,
    pub capabilities_info: Vec<ModelCapability>,
}

#[derive(Event)]
pub enum QuickAiEvent {
    TriggerActivated(TriggerMethod, String),
    ModelChanged(String, String), // old, new
    WebSearchToggled(bool),
    ProviderIconLoaded(String, Handle<Image>),
    CapabilityUpdated(String, Vec<ModelCapability>),
}
```

### Input Handling System

```rust
fn handle_quick_ai_triggers(
    mut trigger_events: EventWriter<QuickAiEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    search_context: Query<&SearchContext>,
    quick_ai_config: Query<&QuickAIConfiguration>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        if let Ok(context) = search_context.get_single() {
            if context.is_active && !context.query.is_empty() {
                trigger_events.send(QuickAiEvent::TriggerActivated(
                    TriggerMethod::TabToAskAI,
                    context.query.clone(),
                ));
            }
        }
    }
}
```

### Flex-Based UI Layout

```rust
fn setup_quick_ai_interface(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(12.0),
            flex_grow: 0.0,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        QuickAiInterface,
    )).with_children(|parent| {
        // Trigger method row
        parent.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        }).with_children(|row| {
            // Label
            row.spawn((
                Text::new("Trigger"),
                TextFont { size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
            
            // Dropdown
            row.spawn((
                Node {
                    width: Val::Percent(60.0),
                    height: Val::Px(32.0),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
                TriggerDropdown::default(),
            ));
        });
    });
}
```

### Async Provider Integration

```rust
#[derive(Component)]
pub struct ModelLoadingTask(Task<Vec<ProviderModel>>);

fn load_provider_models(
    mut commands: Commands,
    mut loading_tasks: Query<(Entity, &mut ModelLoadingTask)>,
    providers: Res<AiProviderRegistry>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    
    // Poll completed loading tasks
    for (entity, mut task) in &mut loading_tasks {
        if let Some(models) = block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity)
                .remove::<ModelLoadingTask>()
                .insert(AvailableModels { models });
        }
    }
}
```
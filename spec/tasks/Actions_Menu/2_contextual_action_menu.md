# Actions Menu - Contextual Action Menu System

## Task: Implement Context-Sensitive Action Menu Interface

### File: `ui/src/launcher/contextual_actions.rs` (new file)

Create comprehensive contextual action menu system with dynamic action availability, keyboard shortcuts, and smooth animation transitions.

### Implementation Requirements

#### Contextual Action Component
```rust
#[derive(Component)]
pub struct ContextualActionMenu {
    pub visible: bool,
    pub selected_command: Option<CommandId>,
    pub available_actions: Vec<ActionItem>,
    pub selected_action_index: usize,
    pub position: Vec2,
}
```

#### Action Menu System
- File: `ui/src/launcher/contextual_actions.rs` (line 1-167)
- Implement right-click and selection-triggered action menu
- Context-sensitive action availability based on command type
- Smooth slide-in animation from selected item
- Keyboard navigation with directional keys

#### Standard Action Implementation
- File: `ui/src/launcher/standard_actions.rs` (new file, line 1-123)
- Implement "Open Command" primary action with Enter key
- "Reset Ranking" functionality for usage-based ranking
- "Move Down in Favorites" with keyboard shortcut (⌃⌘↓)
- "Remove from Favorites" with keyboard shortcut (⇧⌘F)

#### Dynamic Action Discovery
- File: `ui/src/launcher/action_discovery.rs` (new file, line 1-89)
- Dynamic action availability based on command capabilities
- Context-aware action filtering for command types
- Plugin-extensible action system for third-party integrations
- Action validation and permission checking

#### Action Execution Framework
- File: `ui/src/launcher/action_execution.rs` (new file, line 1-134)
- Safe action execution with proper error handling
- Action parameter collection and validation
- Asynchronous action processing with progress feedback
- Action result handling and user notification

#### Search Integration
- File: `ui/src/launcher/action_search.rs` (new file, line 1-67)
- "Search for actions" secondary search functionality
- Real-time action filtering within contextual menu
- Action search with keyboard navigation
- Search scope limited to available actions for selected command

### Architecture Notes
- Zero-allocation action menu rendering
- Integration with command system for action execution
- Smooth animations using Bevy's animation system
- Keyboard-first design with mouse support

### Visual Requirements
- Slide-in animation from selected command position
- Clear action hierarchy with primary/secondary groupings
- Keyboard shortcut display for each action
- Visual feedback for action execution status

### Integration Points
- Command execution system for action processing
- Favorites management system for ranking operations
- Animation system for smooth menu transitions
- Keyboard input system for navigation shortcuts

### Security Considerations
- Action permission validation before execution
- Safe command parameter handling
- Prevent execution of dangerous or system-level actions
- User confirmation for destructive operations

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.## Bevy Implementation Details

### Contextual Action Components

```rust
#[derive(Component, Reflect)]
pub struct ContextualActionMenu {
    pub visible: bool,
    pub target_entity: Option<Entity>,
    pub available_actions: Vec<Entity>, // Entities with ActionItem components
    pub selected_action_index: usize,
    pub position: Vec3,
    pub animation_state: MenuAnimationState,
}

#[derive(Component, Reflect)]
pub struct ActionItem {
    pub action_id: String,
    pub display_name: String,
    pub keyboard_shortcut: Option<KeyCombination>,
    pub icon_handle: Option<Handle<Image>>,
    pub action_type: ActionType,
    pub enabled: bool,
    pub requires_confirmation: bool,
}

#[derive(Component, Reflect)]
pub struct ActionExecutionContext {
    pub target_command: Entity,
    pub action_item: Entity,
    pub parameters: HashMap<String, ActionParameter>,
    pub execution_status: ExecutionStatus,
}
```

### Action Discovery System

```rust
fn discover_contextual_actions(
    mut commands: Commands,
    selected_commands: Query<(Entity, &CommandType), With<SelectedCommand>>,
    action_registry: Res<ActionRegistry>,
    mut action_menu: Query<&mut ContextualActionMenu>,
) {
    for (command_entity, command_type) in &selected_commands {
        let available_actions = action_registry.get_actions_for_type(command_type);
        
        for mut menu in &mut action_menu {
            menu.target_entity = Some(command_entity);
            menu.available_actions.clear();
            
            // Spawn action item entities
            for action_def in available_actions {
                let action_entity = commands.spawn(ActionItem {
                    action_id: action_def.id.clone(),
                    display_name: action_def.display_name.clone(),
                    keyboard_shortcut: action_def.shortcut,
                    icon_handle: None,
                    action_type: action_def.action_type,
                    enabled: true,
                    requires_confirmation: action_def.destructive,
                }).id();
                
                menu.available_actions.push(action_entity);
            }
        }
    }
}
```

### Action Execution Framework

```rust
#[derive(Event)]
pub enum ActionEvent {
    Execute(Entity, Entity), // action_entity, target_entity
    ParameterRequired(Entity, ParameterRequest),
    ExecutionCompleted(Entity, ActionResult),
    ExecutionFailed(Entity, String),
}

fn execute_actions(
    mut action_events: EventReader<ActionEvent>,
    action_items: Query<&ActionItem>,
    mut execution_contexts: Query<&mut ActionExecutionContext>,
    mut commands: Commands,
) {
    for event in action_events.read() {
        match event {
            ActionEvent::Execute(action_entity, target_entity) => {
                if let Ok(action) = action_items.get(*action_entity) {
                    let context_entity = commands.spawn(ActionExecutionContext {
                        target_command: *target_entity,
                        action_item: *action_entity,
                        parameters: HashMap::new(),
                        execution_status: ExecutionStatus::Running,
                    }).id();
                    
                    // Execute action based on type
                    match action.action_type {
                        ActionType::OpenCommand => {
                            // Execute primary command action
                        },
                        ActionType::RemoveFromFavorites => {
                            // Remove from favorites list
                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
    }
}
```

### Contextual Menu Animation System

```rust
fn animate_contextual_menu(
    time: Res<Time>,
    mut menu_query: Query<(&mut ContextualActionMenu, &mut Transform), Changed<ContextualActionMenu>>,
) {
    for (mut menu, mut transform) in &mut menu_query {
        if menu.visible && menu.animation_state == MenuAnimationState::SlideIn {
            // Smooth slide-in animation from target position
            let target_scale = Vec3::ONE;
            let current_scale = transform.scale;
            
            if current_scale.distance(target_scale) > 0.01 {
                transform.scale = current_scale.lerp(target_scale, 8.0 * time.delta_secs());
            } else {
                menu.animation_state = MenuAnimationState::Visible;
                transform.scale = target_scale;
            }
        }
    }
}
```
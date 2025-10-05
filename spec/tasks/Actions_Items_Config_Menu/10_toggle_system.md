# Task 10: Enable/Disable Toggle System Implementation

## Objective
Implement the enable/disable toggle system for extensions and commands with iOS-style toggle switches, state management, dependency handling, and visual feedback for enabled/disabled states.

## Implementation Details

### Target Files
- `ui/src/ui/components/config/toggle_switch.rs:1-200` - Toggle switch component implementation
- `core/src/config/enable_state.rs:1-180` - Enable/disable state management
- `core/src/config/dependency_manager.rs:1-150` - Extension dependency handling
- `ui/src/ui/systems/toggle_interaction.rs:1-120` - Toggle interaction system

### Bevy Implementation Patterns

#### iOS-Style Toggle Switch Component
**Reference**: `./docs/bevy/examples/ui/button.rs:620-660` - Toggle button interaction and styling
**Reference**: `./docs/bevy/examples/ui/ui.rs:1400-1440` - Custom toggle switch implementation
```rust
// Toggle switch component
#[derive(Component)]
pub struct ToggleSwitch {
    pub item_id: String,
    pub enabled: bool,
    pub disabled_reason: Option<DisabledReason>,
    pub animation_timer: Timer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisabledReason {
    ParentDisabled,
    DependencyMissing(String),
    SystemRestriction,
    UserDisabled,
}

// Toggle switch container
ButtonBundle {
    style: Style {
        width: Val::Px(44.0),
        height: Val::Px(24.0),
        border: UiRect::all(Val::Px(1.0)),
        justify_content: if toggle.enabled {
            JustifyContent::FlexEnd
        } else {
            JustifyContent::FlexStart
        },
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(2.0)),
        ..default()
    },
    background_color: if toggle.enabled {
        Color::rgb(0.0, 0.48, 1.0).into() // Blue when enabled
    } else {
        Color::rgba(0.3, 0.3, 0.3, 1.0).into() // Gray when disabled
    },
    border_color: if toggle.disabled_reason.is_some() {
        Color::rgba(0.6, 0.3, 0.3, 1.0).into() // Red border if restricted
    } else {
        Color::rgba(0.4, 0.4, 0.4, 1.0).into()
    },
    border_radius: BorderRadius::all(Val::Px(12.0)),
    ..default()
}

// Toggle switch knob
NodeBundle {
    style: Style {
        width: Val::Px(20.0),
        height: Val::Px(20.0),
        ..default()
    },
    background_color: Color::WHITE.into(),
    border_radius: BorderRadius::all(Val::Px(10.0)),
    ..default()
}
```

#### Toggle Animation System
**Reference**: `./docs/bevy/examples/animation/animated_fox.rs:100-140` - Animation state management
```rust
// Toggle switch animation system
fn toggle_animation_system(
    mut toggle_query: Query<(&mut ToggleSwitch, &mut Style, &mut BackgroundColor)>,
    time: Res<Time>,
) {
    for (mut toggle, mut style, mut bg_color) in toggle_query.iter_mut() {
        toggle.animation_timer.tick(time.delta());
        
        if toggle.animation_timer.percent() < 1.0 {
            let progress = toggle.animation_timer.percent();
            
            // Animate justify_content transition
            let target_justify = if toggle.enabled {
                JustifyContent::FlexEnd
            } else {
                JustifyContent::FlexStart
            };
            
            // Smooth transition logic here
            style.justify_content = target_justify;
            
            // Animate background color transition
            let target_color = if toggle.enabled {
                Color::rgb(0.0, 0.48, 1.0)
            } else {
                Color::rgba(0.3, 0.3, 0.3, 1.0)
            };
            
            // Interpolate color
            *bg_color = target_color.into();
        }
    }
}
```

### Enable/Disable State Management

#### Global Enable State Resource
**Reference**: `./docs/bevy/examples/ecs/resources.rs:340-380` - Enable state resource management
```rust
// Global enable/disable state resource
#[derive(Resource, Clone, Debug)]
pub struct EnableStateRegistry {
    pub enabled_items: HashSet<String>,
    pub disabled_items: HashMap<String, DisabledReason>,
    pub dependencies: HashMap<String, Vec<String>>,
    pub dependents: HashMap<String, Vec<String>>,
    pub change_history: Vec<EnableStateChange>,
}

#[derive(Debug, Clone)]
pub struct EnableStateChange {
    pub timestamp: DateTime<Utc>,
    pub item_id: String,
    pub old_state: bool,
    pub new_state: bool,
    pub reason: ChangeReason,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeReason {
    UserAction,
    DependencyChange,
    SystemRestriction,
    ParentStateChange,
}

impl EnableStateRegistry {
    pub fn new() -> Self {
        Self {
            enabled_items: HashSet::new(),
            disabled_items: HashMap::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            change_history: Vec::new(),
        }
    }
    
    pub fn is_enabled(&self, item_id: &str) -> bool {
        self.enabled_items.contains(item_id) && 
        !self.disabled_items.contains_key(item_id)
    }
    
    pub fn set_enabled(&mut self, item_id: String, enabled: bool, reason: ChangeReason) -> Result<(), EnableError> {
        let old_state = self.is_enabled(&item_id);
        
        if enabled {
            // Check if dependencies are satisfied
            if let Some(dependencies) = self.dependencies.get(&item_id) {
                for dep in dependencies {
                    if !self.is_enabled(dep) {
                        return Err(EnableError::DependencyNotMet(dep.clone()));
                    }
                }
            }
            
            self.enabled_items.insert(item_id.clone());
            self.disabled_items.remove(&item_id);
        } else {
            self.enabled_items.remove(&item_id);
            
            // Disable dependents
            if let Some(dependents) = self.dependents.get(&item_id).cloned() {
                for dependent in dependents {
                    self.set_enabled(dependent, false, ChangeReason::DependencyChange)?;
                }
            }
        }
        
        // Record change
        self.change_history.push(EnableStateChange {
            timestamp: Utc::now(),
            item_id: item_id.clone(),
            old_state,
            new_state: enabled,
            reason,
        });
        
        Ok(())
    }
    
    pub fn can_be_enabled(&self, item_id: &str) -> Result<(), DisabledReason> {
        // Check if parent extension is enabled (for commands)
        if let Some(parent_id) = self.get_parent_extension(item_id) {
            if !self.is_enabled(&parent_id) {
                return Err(DisabledReason::ParentDisabled);
            }
        }
        
        // Check dependencies
        if let Some(dependencies) = self.dependencies.get(item_id) {
            for dep in dependencies {
                if !self.is_enabled(dep) {
                    return Err(DisabledReason::DependencyMissing(dep.clone()));
                }
            }
        }
        
        // Check system restrictions
        if self.is_system_restricted(item_id) {
            return Err(DisabledReason::SystemRestriction);
        }
        
        Ok(())
    }
    
    fn get_parent_extension(&self, item_id: &str) -> Option<String> {
        // Implementation to find parent extension for commands
        // This would integrate with the extension hierarchy system
        None // Placeholder
    }
    
    fn is_system_restricted(&self, _item_id: &str) -> bool {
        // Implementation to check system-level restrictions
        false // Placeholder
    }
}

#[derive(Debug, Clone)]
pub enum EnableError {
    DependencyNotMet(String),
    SystemRestricted,
    ParentDisabled,
    InvalidItem,
}
```

#### Toggle Interaction System
**Reference**: `./docs/bevy/examples/ui/button.rs:700-740` - Button interaction with state changes
```rust
// Toggle switch interaction system
fn toggle_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut ToggleSwitch),
        (Changed<Interaction>, With<Button>),
    >,
    mut enable_state: ResMut<EnableStateRegistry>,
    mut toggle_events: EventWriter<ToggleStateEvent>,
    mut confirmation_events: EventWriter<ConfirmationRequiredEvent>,
) {
    for (interaction, mut toggle) in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            let new_state = !toggle.enabled;
            
            // Check if toggle is allowed
            if new_state {
                match enable_state.can_be_enabled(&toggle.item_id) {
                    Ok(()) => {
                        // Enable allowed
                        if let Ok(()) = enable_state.set_enabled(
                            toggle.item_id.clone(), 
                            true, 
                            ChangeReason::UserAction
                        ) {
                            toggle.enabled = true;
                            toggle.animation_timer.reset();
                            
                            toggle_events.send(ToggleStateEvent {
                                item_id: toggle.item_id.clone(),
                                enabled: true,
                            });
                        }
                    }
                    Err(reason) => {
                        // Show why it can't be enabled
                        toggle.disabled_reason = Some(reason.clone());
                        confirmation_events.send(ConfirmationRequiredEvent {
                            message: format!("Cannot enable: {:?}", reason),
                            action: None,
                        });
                    }
                }
            } else {
                // Disabling - check if it will disable dependents
                let dependents = enable_state.dependents.get(&toggle.item_id).cloned().unwrap_or_default();
                
                if !dependents.is_empty() {
                    confirmation_events.send(ConfirmationRequiredEvent {
                        message: format!("Disabling this will also disable {} dependent items. Continue?", dependents.len()),
                        action: Some(ConfirmationAction::DisableWithDependents {
                            item_id: toggle.item_id.clone(),
                        }),
                    });
                } else {
                    // Safe to disable
                    if let Ok(()) = enable_state.set_enabled(
                        toggle.item_id.clone(),
                        false,
                        ChangeReason::UserAction
                    ) {
                        toggle.enabled = false;
                        toggle.animation_timer.reset();
                        
                        toggle_events.send(ToggleStateEvent {
                            item_id: toggle.item_id.clone(),
                            enabled: false,
                        });
                    }
                }
            }
        }
    }
}
```

### Dependency Management System

#### Extension Dependency Tracking
**Reference**: `./docs/bevy/examples/ecs/change_detection.rs:80-120` - Dependency change propagation
```rust
// Extension dependency management
#[derive(Component)]
pub struct ExtensionDependencies {
    pub required: Vec<String>,
    pub optional: Vec<String>,
    pub conflicts: Vec<String>,
}

// Dependency validation system
fn dependency_validation_system(
    mut enable_state: ResMut<EnableStateRegistry>,
    dependency_query: Query<(&ExtensionDependencies, Entity), Changed<ExtensionDependencies>>,
    mut dependency_events: EventWriter<DependencyChangedEvent>,
) {
    for (dependencies, entity) in dependency_query.iter() {
        // Validate required dependencies
        for required_dep in &dependencies.required {
            if !enable_state.is_enabled(required_dep) {
                dependency_events.send(DependencyChangedEvent {
                    dependent_id: format!("entity_{:?}", entity),
                    dependency_id: required_dep.clone(),
                    dependency_type: DependencyType::Required,
                    available: false,
                });
            }
        }
        
        // Check for conflicts
        for conflict in &dependencies.conflicts {
            if enable_state.is_enabled(conflict) {
                dependency_events.send(DependencyChangedEvent {
                    dependent_id: format!("entity_{:?}", entity),
                    dependency_id: conflict.clone(),
                    dependency_type: DependencyType::Conflict,
                    available: true,
                });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    Required,
    Optional,
    Conflict,
}
```

### Bulk Toggle Operations

#### Multi-Selection Toggle System
**Reference**: `./docs/bevy/examples/ui/ui.rs:1500-1540` - Bulk operations interface
```rust
// Bulk toggle operations for multi-selection
#[derive(Component)]
pub struct BulkToggleControls {
    pub selected_items: HashSet<String>,
    pub operation: BulkOperation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BulkOperation {
    EnableAll,
    DisableAll,
    ToggleAll,
}

// Bulk toggle system
fn bulk_toggle_system(
    mut bulk_query: Query<&mut BulkToggleControls>,
    mut enable_state: ResMut<EnableStateRegistry>,
    mut bulk_events: EventWriter<BulkToggleEvent>,
    table_state: Res<TableState>,
) {
    for mut bulk_controls in bulk_query.iter_mut() {
        if !bulk_controls.selected_items.is_empty() {
            match bulk_controls.operation {
                BulkOperation::EnableAll => {
                    let mut enabled_count = 0;
                    for item_id in &bulk_controls.selected_items {
                        if let Ok(()) = enable_state.set_enabled(
                            item_id.clone(),
                            true,
                            ChangeReason::UserAction
                        ) {
                            enabled_count += 1;
                        }
                    }
                    
                    bulk_events.send(BulkToggleEvent {
                        operation: BulkOperation::EnableAll,
                        affected_count: enabled_count,
                        total_count: bulk_controls.selected_items.len(),
                    });
                }
                BulkOperation::DisableAll => {
                    let mut disabled_count = 0;
                    for item_id in &bulk_controls.selected_items {
                        if let Ok(()) = enable_state.set_enabled(
                            item_id.clone(),
                            false,
                            ChangeReason::UserAction
                        ) {
                            disabled_count += 1;
                        }
                    }
                    
                    bulk_events.send(BulkToggleEvent {
                        operation: BulkOperation::DisableAll,
                        affected_count: disabled_count,
                        total_count: bulk_controls.selected_items.len(),
                    });
                }
                BulkOperation::ToggleAll => {
                    let mut toggled_count = 0;
                    for item_id in &bulk_controls.selected_items {
                        let current_state = enable_state.is_enabled(item_id);
                        if let Ok(()) = enable_state.set_enabled(
                            item_id.clone(),
                            !current_state,
                            ChangeReason::UserAction
                        ) {
                            toggled_count += 1;
                        }
                    }
                    
                    bulk_events.send(BulkToggleEvent {
                        operation: BulkOperation::ToggleAll,
                        affected_count: toggled_count,
                        total_count: bulk_controls.selected_items.len(),
                    });
                }
            }
            
            bulk_controls.selected_items.clear();
        }
    }
}
```

### Event System Integration

#### Toggle State Events
**Reference**: `./docs/bevy/examples/ecs/event.rs:280-310` - Toggle and state change events
```rust
// Toggle system events
#[derive(Event)]
pub struct ToggleStateEvent {
    pub item_id: String,
    pub enabled: bool,
}

#[derive(Event)]
pub struct DependencyChangedEvent {
    pub dependent_id: String,
    pub dependency_id: String,
    pub dependency_type: DependencyType,
    pub available: bool,
}

#[derive(Event)]
pub struct BulkToggleEvent {
    pub operation: BulkOperation,
    pub affected_count: usize,
    pub total_count: usize,
}

#[derive(Event)]
pub struct ConfirmationRequiredEvent {
    pub message: String,
    pub action: Option<ConfirmationAction>,
}

#[derive(Debug, Clone)]
pub enum ConfirmationAction {
    DisableWithDependents { item_id: String },
    EnableWithDependencies { item_id: String },
}
```

### Architecture Notes

#### Component Structure
- **ToggleSwitch**: Individual toggle switch component with animation
- **EnableStateRegistry**: Global state tracking for all enable/disable states
- **ExtensionDependencies**: Dependency tracking for extensions and commands
- **BulkToggleControls**: Multi-selection bulk operation management

#### State Management Strategy
- **Hierarchical Dependencies**: Parent extensions control child commands
- **Dependency Validation**: Real-time dependency checking and enforcement
- **Change History**: Complete audit trail of enable/disable changes
- **Conflict Resolution**: User confirmation for potentially disruptive changes

#### Visual Feedback System
- **iOS-style Toggles**: Familiar toggle switch interface with smooth animations
- **Color Coding**: Blue for enabled, gray for disabled, red border for restricted
- **Disabled State Indication**: Visual feedback for why items cannot be enabled
- **Bulk Operation Feedback**: Progress indicators for multi-item operations

### Quality Standards
- Smooth toggle animations with 60fps performance
- Reliable dependency tracking and enforcement
- Clear visual feedback for all toggle states and restrictions
- Efficient bulk operations with progress feedback
- Comprehensive error handling for dependency conflicts

### Integration Points
- Table interface integration for toggle switch display
- Extension management system for dependency resolution
- Detail panel integration for individual toggle configuration
- Search and filter system integration for state-based filtering
# Task 8: Alias Management System Implementation

## Objective
Implement the alias management system with custom alias creation, uniqueness validation, "Add Alias" functionality, and alias display in table and detail panel interfaces.

## Implementation Details

### Target Files
- `ui/src/ui/components/config/alias_manager.rs:1-200` - Alias creation and editing interface
- `core/src/aliases/registry.rs:1-250` - Global alias management and validation
- `ui/src/ui/components/config/alias_input.rs:1-150` - Alias input field component
- `core/src/aliases/validation.rs:1-120` - Alias validation rules and conflict detection

### Bevy Implementation Patterns

#### Alias Input Component
**Reference**: `./docs/bevy/examples/ui/text_input.rs:180-220` - Text input with validation
**Reference**: `./docs/bevy/examples/input/text_input.rs:100-140` - Real-time input handling
```rust
// Alias input component
#[derive(Component)]
pub struct AliasInput {
    pub item_id: String,
    pub current_value: String,
    pub validation_state: ValidationState,
    pub placeholder: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationState {
    Valid,
    Invalid(ValidationError),
    Checking,
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    AlreadyExists(String), // Conflicting item ID
    InvalidCharacters,
    TooShort,
    TooLong,
    SystemReserved,
}

// Alias input field UI
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        gap: Size::all(Val::Px(4.0)),
        margin: UiRect::vertical(Val::Px(8.0)),
        ..default()
    },
    ..default()
}

// Input label and validation indicator
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    },
    ..default()
}

// Alias label
TextBundle::from_section(
    "Alias:",
    TextStyle {
        font: font_regular.clone(),
        font_size: 14.0,
        color: Color::rgba(0.8, 0.8, 0.8, 1.0),
    },
)

// Validation status indicator
TextBundle::from_section(
    match alias_input.validation_state {
        ValidationState::Valid => "✓",
        ValidationState::Invalid(_) => "✗",
        ValidationState::Checking => "...",
        ValidationState::Empty => "",
    },
    TextStyle {
        font: font_regular.clone(),
        font_size: 12.0,
        color: match alias_input.validation_state {
            ValidationState::Valid => Color::rgb(0.2, 0.8, 0.2),
            ValidationState::Invalid(_) => Color::rgb(0.8, 0.2, 0.2),
            _ => Color::rgba(0.6, 0.6, 0.6, 1.0),
        },
    },
)

// Alias input field
NodeBundle {
    style: Style {
        width: Val::Px(150.0),
        height: Val::Px(32.0),
        border: UiRect::all(Val::Px(1.0)),
        padding: UiRect::horizontal(Val::Px(8.0)),
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: Color::rgba(0.15, 0.15, 0.15, 1.0).into(),
    border_color: match alias_input.validation_state {
        ValidationState::Valid => Color::rgb(0.2, 0.8, 0.2).into(),
        ValidationState::Invalid(_) => Color::rgb(0.8, 0.2, 0.2).into(),
        _ => Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
    },
    border_radius: BorderRadius::all(Val::Px(4.0)),
    ..default()
}
```

#### Add Alias Button System
**Reference**: `./docs/bevy/examples/ui/button.rs:560-590` - Add alias button functionality
```rust
// Add Alias button component
#[derive(Component)]
pub struct AddAliasButton {
    pub item_id: String,
    pub alias_added: bool,
}

// Add Alias button (shown when no alias exists)
ButtonBundle {
    style: Style {
        padding: UiRect {
            left: Val::Px(8.0),
            right: Val::Px(8.0),
            top: Val::Px(4.0),
            bottom: Val::Px(4.0),
        },
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: Color::TRANSPARENT.into(),
    ..default()
}

// Add Alias button text
TextBundle::from_section(
    "Add Alias",
    TextStyle {
        font: font_regular.clone(),
        font_size: 12.0,
        color: Color::rgba(0.6, 0.6, 0.6, 1.0),
    },
)

// Add alias button interaction system
fn add_alias_button_system(
    mut interaction_query: Query<(&Interaction, &mut AddAliasButton), Changed<Interaction>>,
    mut alias_events: EventWriter<AddAliasEvent>,
) {
    for (interaction, mut add_button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked && !add_button.alias_added {
            add_button.alias_added = true;
            alias_events.send(AddAliasEvent {
                item_id: add_button.item_id.clone(),
            });
        }
    }
}
```

### Alias Validation System

#### Real-time Alias Validation
**Reference**: `./docs/bevy/examples/input/text_input.rs:160-200` - Input validation and debouncing
```rust
// Alias validation system
fn alias_validation_system(
    mut alias_query: Query<&mut AliasInput, Changed<AliasInput>>,
    alias_registry: Res<AliasRegistry>,
    mut validation_events: EventWriter<AliasValidationEvent>,
) {
    for mut alias_input in alias_query.iter_mut() {
        if alias_input.current_value.is_empty() {
            alias_input.validation_state = ValidationState::Empty;
            continue;
        }
        
        // Set checking state
        alias_input.validation_state = ValidationState::Checking;
        
        // Perform validation
        match validate_alias(&alias_input.current_value, &alias_input.item_id, &alias_registry) {
            Ok(()) => {
                alias_input.validation_state = ValidationState::Valid;
            }
            Err(error) => {
                alias_input.validation_state = ValidationState::Invalid(error);
            }
        }
        
        validation_events.send(AliasValidationEvent {
            item_id: alias_input.item_id.clone(),
            alias: alias_input.current_value.clone(),
            state: alias_input.validation_state.clone(),
        });
    }
}

// Alias validation function
fn validate_alias(alias: &str, item_id: &str, registry: &AliasRegistry) -> Result<(), ValidationError> {
    // Check length constraints
    if alias.len() < 2 {
        return Err(ValidationError::TooShort);
    }
    if alias.len() > 20 {
        return Err(ValidationError::TooLong);
    }
    
    // Check character constraints
    if !alias.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(ValidationError::InvalidCharacters);
    }
    
    // Check for system reserved aliases
    if registry.is_system_reserved(alias) {
        return Err(ValidationError::SystemReserved);
    }
    
    // Check for conflicts with existing aliases
    if let Some(conflicting_item) = registry.get_item_by_alias(alias) {
        if conflicting_item != item_id {
            return Err(ValidationError::AlreadyExists(conflicting_item));
        }
    }
    
    Ok(())
}
```

### Global Alias Registry

#### Alias Registry Management
**Reference**: `./docs/bevy/examples/ecs/resources.rs:280-320` - Global alias state management
```rust
// Global alias registry resource
#[derive(Resource, Clone, Debug)]
pub struct AliasRegistry {
    pub assignments: HashMap<String, String>, // alias -> item_id
    pub reverse_lookup: HashMap<String, String>, // item_id -> alias
    pub system_reserved: HashSet<String>,
    pub alias_history: Vec<AliasChange>,
}

#[derive(Debug, Clone)]
pub struct AliasChange {
    pub timestamp: DateTime<Utc>,
    pub item_id: String,
    pub old_alias: Option<String>,
    pub new_alias: Option<String>,
    pub change_type: AliasChangeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AliasChangeType {
    Added,
    Modified,
    Removed,
}

impl AliasRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            assignments: HashMap::new(),
            reverse_lookup: HashMap::new(),
            system_reserved: HashSet::new(),
            alias_history: Vec::new(),
        };
        
        // Initialize system reserved aliases
        registry.init_system_reserved();
        registry
    }
    
    pub fn assign_alias(&mut self, item_id: String, alias: String) -> Result<(), AliasError> {
        // Validate alias
        if self.is_system_reserved(&alias) {
            return Err(AliasError::SystemReserved);
        }
        
        if let Some(existing_item) = self.assignments.get(&alias) {
            if existing_item != &item_id {
                return Err(AliasError::AlreadyExists(existing_item.clone()));
            }
        }
        
        // Remove existing alias for this item
        let old_alias = self.reverse_lookup.remove(&item_id);
        if let Some(ref old) = old_alias {
            self.assignments.remove(old);
        }
        
        // Assign new alias
        self.assignments.insert(alias.clone(), item_id.clone());
        self.reverse_lookup.insert(item_id.clone(), alias.clone());
        
        // Record change
        self.alias_history.push(AliasChange {
            timestamp: Utc::now(),
            item_id,
            old_alias,
            new_alias: Some(alias),
            change_type: AliasChangeType::Added,
        });
        
        Ok(())
    }
    
    pub fn remove_alias(&mut self, item_id: &str) -> Option<String> {
        if let Some(alias) = self.reverse_lookup.remove(item_id) {
            self.assignments.remove(&alias);
            
            // Record change
            self.alias_history.push(AliasChange {
                timestamp: Utc::now(),
                item_id: item_id.to_string(),
                old_alias: Some(alias.clone()),
                new_alias: None,
                change_type: AliasChangeType::Removed,
            });
            
            Some(alias)
        } else {
            None
        }
    }
    
    pub fn get_alias_by_item(&self, item_id: &str) -> Option<&String> {
        self.reverse_lookup.get(item_id)
    }
    
    pub fn get_item_by_alias(&self, alias: &str) -> Option<String> {
        self.assignments.get(alias).cloned()
    }
    
    pub fn is_system_reserved(&self, alias: &str) -> bool {
        self.system_reserved.contains(alias)
    }
    
    fn init_system_reserved(&mut self) {
        // Common system reserved aliases
        let reserved = vec![
            "help", "quit", "exit", "settings", "config", "admin", "root",
            "system", "user", "app", "application", "launcher", "search",
        ];
        
        for alias in reserved {
            self.system_reserved.insert(alias.to_string());
        }
    }
}

#[derive(Debug, Clone)]
pub enum AliasError {
    AlreadyExists(String),
    SystemReserved,
    InvalidFormat,
    TooShort,
    TooLong,
}
```

### Alias Display Components

#### Table Alias Display
**Reference**: `./docs/bevy/examples/ui/ui.rs:1300-1340` - Alias display in table cells
```rust
// Alias display in table cell
fn create_alias_cell(parent: &mut ChildBuilder, item: &ExtensionItem, alias_registry: &AliasRegistry) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(15.0), // Alias column width
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(8.0)),
            ..default()
        },
        ..default()
    })
    .with_children(|cell| {
        if let Some(alias) = alias_registry.get_alias_by_item(&item.id) {
            // Display existing alias
            cell.spawn(TextBundle::from_section(
                alias.clone(),
                TextStyle {
                    font: font_mono.clone(),
                    font_size: 12.0,
                    color: Color::rgba(0.9, 0.9, 0.9, 1.0),
                },
            ));
        } else {
            // Show "Add Alias" button
            cell.spawn(ButtonBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                background_color: Color::TRANSPARENT.into(),
                ..default()
            })
            .insert(AddAliasButton {
                item_id: item.id.clone(),
                alias_added: false,
            })
            .with_children(|button| {
                button.spawn(TextBundle::from_section(
                    "Add Alias",
                    TextStyle {
                        font: font_regular.clone(),
                        font_size: 11.0,
                        color: Color::rgba(0.6, 0.6, 0.6, 1.0),
                    },
                ));
            });
        }
    });
}
```

### Alias Input Processing

#### Text Input System for Aliases
**Reference**: `./docs/bevy/examples/input/text_input.rs:240-280` - Character input processing for aliases
```rust
// Alias text input system
fn alias_text_input_system(
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut key_input_events: EventReader<KeyboardInput>,
    mut alias_query: Query<&mut AliasInput>,
    focused_input: Res<FocusedAliasInput>,
) {
    if let Some(focused_item_id) = &focused_input.item_id {
        for mut alias_input in alias_query.iter_mut() {
            if alias_input.item_id == *focused_item_id {
                // Handle character input
                for event in char_input_events.iter() {
                    if event.char.is_control() {
                        continue;
                    }
                    
                    // Only allow valid alias characters
                    if event.char.is_alphanumeric() || event.char == '_' || event.char == '-' {
                        alias_input.current_value.push(event.char);
                    }
                }
                
                // Handle special keys
                for event in key_input_events.iter() {
                    if event.state == ButtonState::Pressed {
                        match event.key_code {
                            Some(KeyCode::Back) => {
                                alias_input.current_value.pop();
                            }
                            Some(KeyCode::Delete) => {
                                alias_input.current_value.clear();
                            }
                            Some(KeyCode::Return) => {
                                // Confirm alias if valid
                                if alias_input.validation_state == ValidationState::Valid {
                                    // Submit alias
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

// Focused alias input tracking
#[derive(Resource, Default)]
pub struct FocusedAliasInput {
    pub item_id: Option<String>,
}
```

### Event System Integration

#### Alias Management Events
**Reference**: `./docs/bevy/examples/ecs/event.rs:220-250` - Alias system events
```rust
// Alias management events
#[derive(Event)]
pub struct AddAliasEvent {
    pub item_id: String,
}

#[derive(Event)]
pub struct AliasValidationEvent {
    pub item_id: String,
    pub alias: String,
    pub state: ValidationState,
}

#[derive(Event)]
pub struct AliasAssignedEvent {
    pub item_id: String,
    pub alias: String,
}

#[derive(Event)]
pub struct AliasRemovedEvent {
    pub item_id: String,
    pub old_alias: String,
}
```

### Architecture Notes

#### Component Structure
- **AliasInput**: Text input component for alias editing
- **AliasRegistry**: Global alias management and validation
- **AddAliasButton**: Button for adding aliases to items
- **ValidationState**: Real-time validation feedback system

#### Validation Strategy
- **Real-time Validation**: Immediate feedback as user types
- **Uniqueness Checking**: Global uniqueness enforcement
- **Character Constraints**: Allow alphanumeric, underscore, hyphen only
- **System Reserved**: Protect system-reserved alias names

#### User Experience Flow
- **Add Alias**: Click "Add Alias" to reveal input field
- **Type Alias**: Real-time validation with visual feedback
- **Confirm**: Enter key or focus loss confirms valid alias
- **Edit Existing**: Click existing alias to edit in-place

### Quality Standards
- Real-time validation with clear visual feedback
- Comprehensive conflict detection and prevention
- User-friendly error messages for validation failures
- Efficient alias lookup with minimal performance impact
- Consistent alias display across table and detail interfaces

### Integration Points
- Table interface integration for alias display and editing
- Detail panel integration for alias configuration
- Extension management system for alias persistence
- Search system integration for alias-based command lookup
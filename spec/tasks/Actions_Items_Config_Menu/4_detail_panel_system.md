# Task 4: Detail Panel Configuration System Implementation

## Objective
Implement the detail panel (30% width) that displays configuration options for selected extensions/commands including basic information, action configuration, and extension-specific settings.

## Implementation Details

### Target Files
- `ui/src/ui/components/config/detail_panel.rs:1-300` - Detail panel container and layout
- `ui/src/ui/components/config/config_sections.rs:1-250` - Configuration section components
- `ui/src/ui/components/config/action_dropdown.rs:1-150` - Action configuration dropdown
- `core/src/config/item_configuration.rs:1-200` - Configuration data management

### Bevy Implementation Patterns

#### Detail Panel Container
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:280-310` - Right panel layout (30% width)
**Reference**: `./docs/bevy/examples/ui/ui.rs:600-640` - Scrollable content panel
```rust
// Detail panel container (30% of split layout)
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(30.0),
        height: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(20.0)),
        gap: Size::all(Val::Px(16.0)),
        overflow: Overflow::clip_y(),
        ..default()
    },
    background_color: Color::rgba(0.12, 0.12, 0.12, 1.0).into(),
    ..default()
}

// No selection state message
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    ..default()
}
```

#### Selected Item Display Header
**Reference**: `./docs/bevy/examples/ui/ui.rs:680-720` - Item header with icon and title
**Reference**: `./docs/bevy/examples/asset_loading/asset_loading.rs:160-190` - Large icon display
```rust
// Selected item header section
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        gap: Size::all(Val::Px(12.0)),
        margin: UiRect::bottom(Val::Px(20.0)),
        ..default()
    },
    ..default()
}

// Large extension/command icon
ImageBundle {
    style: Style {
        width: Val::Px(64.0),
        height: Val::Px(64.0),
        ..default()
    },
    image: selected_item.icon_handle.clone().into(),
    ..default()
}

// Item title
TextBundle::from_section(
    selected_item.name.clone(),
    TextStyle {
        font: font_bold.clone(),
        font_size: 20.0,
        color: Color::WHITE,
    },
).with_text_alignment(TextAlignment::Center)

// Item description
TextBundle::from_section(
    selected_item.description.clone(),
    TextStyle {
        font: font_regular.clone(),
        font_size: 14.0,
        color: Color::rgba(0.8, 0.8, 0.8, 1.0),
    },
).with_text_alignment(TextAlignment::Center)
```

#### Configuration Sections Layout
**Reference**: `./docs/bevy/examples/ui/ui.rs:760-800` - Sectioned configuration layout
```rust
// Configuration sections container
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        gap: Size::all(Val::Px(20.0)),
        ..default()
    },
    ..default()
}

// Section header component
fn create_section_header(parent: &mut ChildBuilder, title: &str) {
    parent.spawn(TextBundle::from_section(
        title,
        TextStyle {
            font: font_medium.clone(),
            font_size: 16.0,
            color: Color::rgba(0.9, 0.9, 0.9, 1.0),
        },
    )).insert(SectionHeader);
}

// Section content container
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        gap: Size::all(Val::Px(8.0)),
        padding: UiRect::left(Val::Px(12.0)),
        ..default()
    },
    ..default()
}
```

### Primary Action Configuration

#### Action Dropdown Component
**Reference**: `./docs/bevy/examples/ui/button.rs:400-440` - Dropdown button with options
**Reference**: `./docs/bevy/examples/ui/ui.rs:840-880` - Dropdown menu implementation
```rust
// Action configuration dropdown
#[derive(Component)]
pub struct ActionDropdown {
    pub selected_action: ActionType,
    pub available_actions: Vec<ActionType>,
    pub open: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    PasteToActiveApp,
    CopyToClipboard,
    ShowInFinder,
    OpenWith(String),
    RunScript(String),
    Custom(String),
}

impl ActionType {
    pub fn display_name(&self) -> String {
        match self {
            ActionType::PasteToActiveApp => "Paste to Active App".to_string(),
            ActionType::CopyToClipboard => "Copy to Clipboard".to_string(),
            ActionType::ShowInFinder => "Show in Finder".to_string(),
            ActionType::OpenWith(app) => format!("Open with {}", app),
            ActionType::RunScript(name) => format!("Run Script: {}", name),
            ActionType::Custom(name) => name.clone(),
        }
    }
}

// Dropdown button
ButtonBundle {
    style: Style {
        width: Val::Percent(100.0),
        height: Val::Px(36.0),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(12.0)),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    },
    background_color: Color::rgba(0.15, 0.15, 0.15, 1.0).into(),
    border_color: Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(4.0)),
    ..default()
}

// Dropdown selected text
TextBundle::from_section(
    action_dropdown.selected_action.display_name(),
    TextStyle {
        font: font_regular.clone(),
        font_size: 14.0,
        color: Color::WHITE,
    },
)

// Dropdown arrow
TextBundle::from_section(
    if action_dropdown.open { "▲" } else { "▼" },
    TextStyle {
        font: font_regular.clone(),
        font_size: 12.0,
        color: Color::rgba(0.6, 0.6, 0.6, 1.0),
    },
)
```

#### Dropdown Menu System
**Reference**: `./docs/bevy/examples/ui/ui.rs:920-970` - Overlay dropdown menu
```rust
// Dropdown menu overlay
#[derive(Component)]
pub struct DropdownMenu {
    pub dropdown_id: String,
    pub options: Vec<ActionType>,
}

fn spawn_dropdown_menu(
    commands: &mut Commands,
    dropdown_id: String,
    options: Vec<ActionType>,
    position: Vec2,
) -> Entity {
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                ..default()
            },
            flex_direction: FlexDirection::Column,
            width: Val::Px(200.0),
            max_height: Val::Px(200.0),
            overflow: Overflow::clip_y(),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        background_color: Color::rgba(0.18, 0.18, 0.18, 1.0).into(),
        border_color: Color::rgba(0.4, 0.4, 0.4, 1.0).into(),
        border_radius: BorderRadius::all(Val::Px(4.0)),
        ..default()
    })
    .insert(DropdownMenu { dropdown_id, options: options.clone() })
    .with_children(|menu| {
        for option in options {
            spawn_dropdown_option(menu, option);
        }
    }).id()
}

// Individual dropdown option
fn spawn_dropdown_option(parent: &mut ChildBuilder, action_type: ActionType) {
    parent.spawn(ButtonBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(32.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(12.0)),
            ..default()
        },
        background_color: Color::TRANSPARENT.into(),
        ..default()
    })
    .insert(DropdownOption { action_type: action_type.clone() })
    .with_children(|option| {
        option.spawn(TextBundle::from_section(
            action_type.display_name(),
            TextStyle {
                font: font_regular.clone(),
                font_size: 13.0,
                color: Color::WHITE,
            },
        ));
    });
}
```

### Extension-Specific Configuration

#### Dynamic Configuration Sections
**Reference**: `./docs/bevy/examples/ui/ui.rs:1000-1050` - Dynamic UI generation based on item type
```rust
// Extension-specific configuration component
#[derive(Component)]
pub struct ConfigurationSection {
    pub section_type: ConfigSectionType,
    pub extension_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigSectionType {
    BasicInformation,
    ActionConfiguration,
    HotkeySettings,
    AliasSettings,
    AdvancedOptions,
    ExtensionSpecific(String),
}

// Dynamic configuration rendering system
fn render_configuration_sections(
    parent: &mut ChildBuilder,
    selected_item: &ExtensionItem,
    config_data: &ConfigurationData,
) {
    // Basic Information section
    create_basic_info_section(parent, selected_item);
    
    // Action Configuration section
    create_action_config_section(parent, &selected_item.id, &config_data.action_config);
    
    // Hotkey Settings section
    if selected_item.supports_hotkeys() {
        create_hotkey_section(parent, &selected_item.id, &config_data.hotkey_config);
    }
    
    // Alias Settings section
    if selected_item.supports_aliases() {
        create_alias_section(parent, &selected_item.id, &config_data.alias_config);
    }
    
    // Extension-specific sections
    for custom_section in &selected_item.custom_config_sections {
        create_custom_section(parent, custom_section, config_data);
    }
    
    // Advanced Options section
    if config_data.has_advanced_options() {
        create_advanced_section(parent, &selected_item.id, &config_data.advanced_config);
    }
}
```

#### Configuration Form Controls
**Reference**: `./docs/bevy/examples/ui/text_input.rs:100-140` - Form input controls
```rust
// Text input for alias configuration
#[derive(Component)]
pub struct AliasInput {
    pub extension_id: String,
    pub current_value: String,
}

// Alias input field
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        gap: Size::all(Val::Px(8.0)),
        margin: UiRect::vertical(Val::Px(4.0)),
        ..default()
    },
    ..default()
}

// Input label
TextBundle::from_section(
    "Alias:",
    TextStyle {
        font: font_regular.clone(),
        font_size: 14.0,
        color: Color::rgba(0.8, 0.8, 0.8, 1.0),
    },
)

// Input field
NodeBundle {
    style: Style {
        width: Val::Px(120.0),
        height: Val::Px(28.0),
        border: UiRect::all(Val::Px(1.0)),
        padding: UiRect::horizontal(Val::Px(8.0)),
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: Color::rgba(0.15, 0.15, 0.15, 1.0).into(),
    border_color: Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(3.0)),
    ..default()
}

// Toggle switch for enable/disable
#[derive(Component)]
pub struct EnableToggle {
    pub extension_id: String,
    pub enabled: bool,
}

// Toggle switch implementation
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
    border_color: Color::rgba(0.4, 0.4, 0.4, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(12.0)),
    ..default()
}
```

### Configuration Data Management

#### Configuration State Resource
**Reference**: `./docs/bevy/examples/ecs/resources.rs:160-200` - Configuration state management
```rust
// Configuration data resource
#[derive(Resource, Clone, Debug)]
pub struct ConfigurationData {
    pub selected_item_id: Option<String>,
    pub action_config: HashMap<String, ActionType>,
    pub hotkey_config: HashMap<String, String>,
    pub alias_config: HashMap<String, String>,
    pub advanced_config: HashMap<String, AdvancedConfig>,
    pub custom_sections: HashMap<String, CustomSectionData>,
    pub dirty_fields: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct AdvancedConfig {
    pub timeout: Option<u32>,
    pub retry_count: Option<u32>,
    pub custom_args: HashMap<String, String>,
    pub environment_vars: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CustomSectionData {
    pub section_name: String,
    pub fields: HashMap<String, ConfigFieldValue>,
}

#[derive(Debug, Clone)]
pub enum ConfigFieldValue {
    String(String),
    Boolean(bool),
    Number(f64),
    List(Vec<String>),
}

impl ConfigurationData {
    pub fn get_config_for_item(&self, item_id: &str) -> ItemConfiguration {
        ItemConfiguration {
            action: self.action_config.get(item_id).cloned().unwrap_or(ActionType::PasteToActiveApp),
            hotkey: self.hotkey_config.get(item_id).cloned(),
            alias: self.alias_config.get(item_id).cloned(),
            advanced: self.advanced_config.get(item_id).cloned(),
            custom: self.custom_sections.get(item_id).cloned(),
        }
    }
    
    pub fn update_field(&mut self, item_id: &str, field: ConfigField, value: ConfigFieldValue) {
        match field {
            ConfigField::Action(action) => {
                self.action_config.insert(item_id.to_string(), action);
            }
            ConfigField::Hotkey(hotkey) => {
                if hotkey.is_empty() {
                    self.hotkey_config.remove(item_id);
                } else {
                    self.hotkey_config.insert(item_id.to_string(), hotkey);
                }
            }
            ConfigField::Alias(alias) => {
                if alias.is_empty() {
                    self.alias_config.remove(item_id);
                } else {
                    self.alias_config.insert(item_id.to_string(), alias);
                }
            }
        }
        self.dirty_fields.insert(format!("{}:{:?}", item_id, field));
    }
}
```

### Architecture Notes

#### Component Structure
- **DetailPanel**: Main container for configuration interface
- **ConfigurationSection**: Individual configuration section components
- **ActionDropdown**: Primary action selection dropdown
- **ConfigurationData**: Centralized state for all configuration values

#### Dynamic Content Strategy
- **Item-Specific Rendering**: Configuration sections adapt to selected item type
- **Context-Sensitive Controls**: Form controls appear based on item capabilities
- **Real-time Updates**: Configuration changes reflected immediately in UI
- **Validation Feedback**: Input validation with real-time error display

#### Data Binding and Updates
- **Two-way Binding**: UI controls reflect and update configuration state
- **Change Detection**: Automatic detection of configuration modifications
- **Save State Management**: Track dirty fields for efficient saving
- **Undo/Redo Support**: Support for reverting configuration changes

### Quality Standards
- Responsive configuration interface with smooth transitions
- Clear visual hierarchy for configuration sections
- Real-time validation feedback for all input fields
- Efficient state management with minimal re-rendering
- Accessibility support for all configuration controls

### Integration Points
- Table interface integration for selection-driven configuration display
- Extension management system for configuration persistence
- Hotkey system integration for hotkey assignment and validation
- Action system integration for action type configuration
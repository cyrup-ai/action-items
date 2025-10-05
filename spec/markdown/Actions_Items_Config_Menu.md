# Actions Items Config Menu Specification

## Overview  
The Actions Items Config Menu represents the Extensions management interface, providing comprehensive control over commands, scripts, applications, quicklinks, and snippets. This interface serves as the central hub for configuring, enabling, and customizing all extensible functionality within the launcher.

## Layout Architecture
- **Base Layout**: Extensions tab active in primary navigation
- **Secondary Navigation**: Filter tabs for content categorization
- **Split Layout**: Main table view (70%) and detail panel (30%)
- **Search Integration**: Universal search across all extension items

## Navigation Structure

### Primary Tab Navigation
- **Extensions Tab**: Currently active, highlighted state
- **Integration**: Part of main settings navigation system
- **Badge Support**: Optional badges for extension counts or updates

### Secondary Filter Navigation
- **All**: Complete list of all extension items
- **Commands**: Filtered view of command-type items
- **Scripts**: Script-specific extension filtering
- **Apps**: Application integration filtering
- **Quicklinks**: Quick access link management
- **Sort/Filter Control**: Dropdown for additional sorting and filtering options
- **Add Button**: Plus icon for adding new extensions or items

### Search System
- **Search Input**: Full-width search bar with magnifying glass icon
- **Real-time Filtering**: Live search results as user types
- **Scope**: Searches across names, aliases, descriptions, and metadata
- **Advanced Search**: Support for filtering by type, status, or other attributes

## Main Table Interface

### Table Structure
- **Hierarchical Display**: Parent extensions with collapsible child commands
- **Column Layout**: Name, Type, Alias, Hotkey, Enabled status
- **Row Selection**: Single and multiple selection support
- **Sorting**: Clickable column headers for sorting functionality

#### Column Specifications

##### Name Column
- **Extension Headers**: Collapsible sections with expansion arrows
- **Command Items**: Indented items under parent extensions
- **Icons**: Application or extension-specific icons for visual identification
- **Typography**: Hierarchy indicated through font weight and indentation

##### Type Column  
- **Extension Types**: "Extension", "Command", "AI Extension"
- **Visual Consistency**: Consistent type labeling system
- **Filtering Integration**: Types used for secondary navigation filtering

##### Alias Column
- **Custom Aliases**: User-defined short codes for quick access
- **System Aliases**: Pre-defined aliases for common commands
- **Validation**: Real-time validation of alias uniqueness
- **Editing**: In-place editing or dedicated configuration interface

##### Hotkey Column
- **Hotkey Display**: Visual representation of assigned key combinations
- **Recording Interface**: "Record Hotkey" buttons for new assignment
- **Conflict Detection**: Real-time detection and resolution of hotkey conflicts
- **Clear/Reset**: Options to remove or reset hotkey assignments

##### Enabled Column
- **Toggle Switches**: iOS-style enable/disable toggles
- **State Indication**: Clear visual distinction between enabled/disabled states
- **Bulk Operations**: Support for enabling/disabling multiple items
- **Dependencies**: Automatic handling of dependent command states

## Detail Panel Interface

### Selected Item Display
- **Current Selection**: "Search Snippets" with distinctive red icon
- **Dynamic Content**: Panel updates based on table selection
- **Context Sensitivity**: Content adapts to selected item type

### Configuration Sections

#### Basic Information
- **Title**: Selected command or extension name
- **Description**: "Search for your snippets created in Raycast"
- **Icon Display**: Large icon representation of selected item
- **Type Information**: Command type and category details

#### Primary Action Configuration
- **Action Dropdown**: "Paste to Active App" selection
- **Action Types**: Various action types based on command capabilities
- **Default Behavior**: Configurable default action for command execution
- **Custom Actions**: Support for user-defined action sequences

### Extension-Specific Configuration
- **Command Options**: Configuration specific to individual commands
- **Extension Settings**: Parent extension configuration options
- **Integration Settings**: External service integration configuration
- **Advanced Options**: Expert-level configuration for power users

## Data Examples from Screenshot

### Slack Extension Hierarchy
- **Parent**: Slack (Extension type)
- **Children**: 
  - Ask Slack (AI Extension, alias: sla)
  - Open Channel (Command, alias: slc) 
  - Open Unread Messages (Command, alias: slo)
  - Search Emojis (Command, with "Add Alias" option)
  - Search Messages (Command, alias: sls)
  - Send Message (Command, with "Add Alias" option)
  - Set Presence (Command, alias: slp)
  - Set Snooze (Command, alias: slz)
  - Unread Messages (Command, alias: slu, currently disabled)

### Snippets Extension Hierarchy  
- **Parent**: Snippets (Extension type)
- **Children**:
  - Create Snippet (Command, alias: snipcr, hotkey: ⌃ ⌘ ⌥ [)
  - Export Snippets (Command, alias: snipex, hotkey: ⌃ ⌘ ⌥ E)
  - Import Snippets (Command, alias: snipim, hotkey: ⌃ ⌘ ⌥ I)
  - Search Snippets (Command, alias: snip, hotkey: ⌃ ⌘ ⌥ ]) - Currently selected in blue highlight

## Visual Design Specifications

### Navigation Tab Styling
- **Extensions Tab**: Dark background indicating active state
- **Inactive Tabs**: Light gray text with standard backgrounds
- **Tab Icons**: Distinctive icons for each section (puzzle piece for Extensions)
- **Tab Transitions**: Smooth transitions between tab selections

### Search Interface Design
- **Search Bar**: Full width with rounded corners and dark background
- **Placeholder Text**: "Search..." in medium gray
- **Search Icon**: Magnifying glass icon on right side
- **Focus State**: Visual highlighting when search field is active

### Filter Tab Bar
- **Active Filter**: "All" tab with darker background
- **Inactive Filters**: Commands, Scripts, Apps, Quicklinks with lighter styling
- **Filter Dropdown**: Three-line menu icon with dropdown arrow
- **Add Button**: Plus "+" icon for adding new items
- **Consistent Spacing**: Uniform padding and margins between filter tabs

### Table Layout Specifications
- **Column Headers**: "Name", "Type", "Alias", "Hotkey", "Enabled"
- **Header Styling**: Medium gray text, smaller font size
- **Column Widths**: 
  - Name: ~40% (expandable for hierarchy)
  - Type: ~20%
  - Alias: ~15%
  - Hotkey: ~15%
  - Enabled: ~10%

### Row Styling and States
- **Default Rows**: Dark background with subtle borders
- **Selected Row**: Blue background highlight (Search Snippets example)
- **Expansion Indicators**: Chevron arrows (▼ expanded, ▶ collapsed)
- **Indentation**: Child items indented under parent extensions
- **Icon Integration**: Extension/app icons displayed left of names

### Extension Hierarchy Visual
- **Parent Extensions**: Bold text, chevron controls, extension icons
- **Child Commands**: Regular weight, indented, smaller icons
- **Type Indicators**: Clear labeling (Extension, Command, AI Extension)
- **Alias Display**: Monospace or distinct font for alias values
- **Hotkey Display**: Keyboard symbol formatting (⌃ ⌘ ⌥ with letters)

### Enable/Disable Control States
- **Enabled State**: Checkmark in circle (✓)
- **Disabled State**: Empty circle (○)
- **Hover States**: Visual feedback on hover
- **Click Feedback**: Immediate visual response to toggle interactions

### Detail Panel Design
- **Panel Width**: Approximately 30% of total interface width
- **Background**: Consistent dark theme matching table
- **Icon Display**: Large extension icon at top (red snippets icon)
- **Title Text**: Bold, larger font for selected item name
- **Description Text**: Regular weight, medium gray color
- **Dropdown Styling**: Dark background matching other controls

### Status Indicators
- **"Record Hotkey" Buttons**: Gray text indicating available action
- **"Add Alias" Links**: Gray text for expandable functionality
- **Selection Highlight**: Blue background for currently selected row
- **Hover Effects**: Subtle highlighting on interactive elements

### Color Palette
- **Background**: Dark theme (#1a1a1a or similar)
- **Text Primary**: White/light gray (#ffffff or #f0f0f0)
- **Text Secondary**: Medium gray (#888888) for descriptions and labels
- **Selection Blue**: Bright blue (#007AFF) for selected rows
- **Border Colors**: Subtle dark borders between rows and columns
- **Icon Colors**: Preserve original extension/app branding colors

### Typography Hierarchy
- **Extension Names**: Bold, white text
- **Command Names**: Regular weight, white text with indentation
- **Column Headers**: Small, medium gray text
- **Type Labels**: Regular weight, medium gray
- **Aliases**: Monospace or distinct font, lighter gray
- **Descriptions**: Regular weight, medium gray in detail panel

### Interactive Element Specifications
- **Expandable Sections**: Smooth animation when expanding/collapsing
- **Row Selection**: Single-click selection with immediate highlight
- **Toggle Interactions**: Instant state change on click
- **Search Input**: Real-time filtering with smooth list updates
- **Dropdown Menus**: Standard dropdown styling with dark backgrounds

## Functional Requirements

### Extension Management System
- **Installation Pipeline**: Automated extension installation and updates
- **Dependency Resolution**: Automatic handling of extension dependencies  
- **Version Control**: Support for multiple extension versions
- **Rollback Capability**: Safe rollback to previous extension versions

### Command Configuration System
- **Hotkey Assignment**: Global hotkey assignment with conflict resolution
- **Alias Management**: Custom alias creation and validation
- **Action Binding**: Flexible action assignment for command results
- **State Persistence**: Reliable save/restore of all configuration changes

### Search and Filtering System
- **Real-time Search**: Instant filtering as user types search queries
- **Advanced Filtering**: Multi-criteria filtering by type, status, source
- **Saved Searches**: Optional saved search functionality for power users
- **Search History**: Recent search term history and suggestions

### Bulk Operations
- **Multi-Selection**: Checkbox-based selection of multiple items
- **Bulk Enable/Disable**: Mass operations on selected items
- **Bulk Hotkey Assignment**: Batch hotkey configuration
- **Export/Import**: Configuration export/import for backup and sharing

## Bevy Implementation Examples

### Hierarchical Table Display
- Reference: `./docs/bevy/examples/ui/ui.rs` - Nested UI hierarchy and indentation
- Reference: `./docs/bevy/examples/ui/flex_layout.rs` - Flexible row and column layouts

### Search Input Interface
- Reference: `./docs/bevy/examples/ui/text_input.rs` - Search input with real-time filtering
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Search query processing

### Toggle Switch Implementation
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Toggle switch states and animations
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Toggle interaction handling

### Hotkey Recording Interface
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Keyboard combination capture
- Reference: `./docs/bevy/examples/ui/button.rs` - Record button states and feedback

### Table Selection and Navigation
- Reference: `./docs/bevy/examples/ui/ui.rs` - Row selection and highlighting
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Click and selection handling

### Collapsible Sections
- Reference: `./docs/bevy/examples/ui/flex_layout.rs` - Dynamic content showing/hiding
- Reference: `./docs/bevy/examples/animation/animated_fox.rs` - Smooth expand/collapse animations

### Icon Management System
- Reference: `./docs/bevy/examples/asset_loading/asset_loading.rs` - Dynamic icon loading
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Icon atlas management

## State Management Requirements

### Extension State Tracking
- **Installation Status**: Real-time tracking of extension installation and updates
- **Enable/Disable State**: Persistent storage of individual command states
- **Configuration Changes**: Automatic saving of configuration modifications
- **Dependency Tracking**: Monitoring of extension dependencies and conflicts

### Search and Filter State
- **Search Query Persistence**: Remember search terms across sessions
- **Filter State Persistence**: Maintain filter selections between uses
- **Selection State**: Preserve selected items during navigation
- **Sort Preferences**: Remember user sorting preferences

### Hotkey Management State
- **Global Hotkey Registry**: System-wide hotkey assignment tracking
- **Conflict Resolution**: Real-time conflict detection and user notification
- **Recording State**: Modal state management during hotkey recording
- **Validation State**: Real-time validation feedback for hotkey assignments

## Security and Permissions

### Extension Security
- **Code Signing**: Verification of extension authenticity and integrity
- **Sandboxing**: Isolated execution environment for extension code
- **Permission System**: Granular permission control for extension capabilities
- **Audit Logging**: Comprehensive logging of extension installations and changes

### Command Execution Security
- **Input Validation**: Sanitization of command inputs and parameters
- **Output Sanitization**: Safe handling of command outputs and results
- **Resource Limits**: Enforcement of resource usage limits for commands
- **Safe Execution**: Protected execution environment for potentially unsafe operations

### Configuration Security
- **Change Validation**: Validation of configuration changes before application
- **Rollback Protection**: Safe rollback mechanisms for problematic configurations
- **Backup Integrity**: Secure backup and restore of configuration data
- **Access Control**: User authentication for sensitive configuration changes

## Performance Optimization

### Large Dataset Handling
- **Virtual Scrolling**: Efficient rendering of large extension lists
- **Lazy Loading**: On-demand loading of extension metadata and icons
- **Search Optimization**: Efficient search algorithms for large datasets
- **Caching Strategy**: Intelligent caching of extension information and states

### Real-time Updates
- **Incremental Updates**: Efficient handling of extension state changes
- **Debounced Search**: Optimized search performance with input debouncing
- **Background Processing**: Non-blocking processing of extension operations
- **Progressive Loading**: Incremental loading of extension information

### Memory Management
- **Resource Cleanup**: Proper cleanup of unused extension resources
- **Icon Optimization**: Efficient storage and rendering of extension icons
- **State Compression**: Compressed storage of extension configuration data
- **Garbage Collection**: Automatic cleanup of temporary extension data

## Error Handling and Recovery

### Extension Management Errors
- **Installation Failures**: Clear feedback and recovery options for failed installations
- **Dependency Conflicts**: User-friendly resolution of extension dependency issues
- **Corruption Recovery**: Automatic detection and repair of corrupted extensions
- **Network Failures**: Graceful handling of network connectivity issues

### Configuration Errors
- **Validation Failures**: Real-time feedback for invalid configuration attempts
- **Hotkey Conflicts**: Clear resolution workflow for hotkey assignment conflicts
- **State Corruption**: Automatic recovery from corrupted configuration states
- **Backup Restoration**: Reliable restoration of configuration from backups

### Runtime Errors
- **Command Execution Failures**: Graceful handling of failed command execution
- **Search Performance Issues**: Fallback options for search performance problems
- **UI Responsiveness**: Recovery mechanisms for UI freezing or unresponsiveness
- **Memory Issues**: Automatic handling of memory pressure and cleanup

## Bevy Implementation Details

### Component Architecture

#### Extension Configuration Components
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Component, Reflect)]
pub struct ExtensionItem {
    pub id: String,
    pub name: String,
    pub extension_type: ExtensionType,
    pub icon: Handle<Image>,
    pub enabled: bool,
    pub parent_extension: Option<String>,
    pub children: Vec<String>,
}

#[derive(Component, Reflect)]
pub struct ExtensionAlias {
    pub value: String,
    pub valid: bool,
    pub conflicts: Vec<String>,
}

#[derive(Component, Reflect)]
pub struct ExtensionHotkey {
    pub combination: KeyCombination,
    pub registered: bool,
    pub conflicts: Vec<String>,
}

#[derive(Component, Reflect)]
pub struct ExtensionToggle {
    pub enabled: bool,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

#[derive(Reflect, Clone, PartialEq)]
pub enum ExtensionType {
    Extension,
    Command,
    AIExtension,
}

#[derive(Component, Reflect)]
pub struct HierarchicalTable {
    pub expanded_items: HashSet<String>,
    pub selected_item: Option<String>,
    pub sort_column: TableColumn,
    pub sort_ascending: bool,
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum TableColumn {
    Name,
    Type,
    Alias,
    Hotkey,
    Enabled,
}
```

#### UI State Components
```rust
#[derive(Component, Reflect)]
pub struct SearchInput {
    pub text: String,
    pub focused: bool,
    pub placeholder: String,
}

#[derive(Component, Reflect)]
pub struct FilterTabs {
    pub active_filter: ExtensionFilter,
    pub show_dropdown: bool,
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum ExtensionFilter {
    All,
    Commands,
    Scripts,
    Apps,
    Quicklinks,
}

#[derive(Component, Reflect)]
pub struct DetailPanel {
    pub selected_extension: Option<String>,
    pub action_dropdown: String,
    pub configuration_visible: bool,
}
```

### Resource Management
```rust
#[derive(Resource, Default, Reflect)]
pub struct ExtensionRegistry {
    pub extensions: HashMap<String, ExtensionData>,
    pub filtered_results: Vec<String>,
    pub search_index: SearchIndex,
}

#[derive(Resource, Reflect)]
pub struct ExtensionConfigState {
    pub modified_extensions: HashSet<String>,
    pub pending_saves: HashMap<String, ExtensionConfig>,
    pub validation_errors: HashMap<String, Vec<String>>,
}

#[derive(Reflect)]
pub struct ExtensionData {
    pub manifest: ExtensionManifest,
    pub commands: Vec<CommandData>,
    pub configuration: ExtensionConfig,
    pub status: ExtensionStatus,
}

#[derive(Resource, Reflect)]
pub struct HotkeyRegistry {
    pub registered_hotkeys: HashMap<KeyCombination, String>,
    pub pending_registrations: HashMap<String, KeyCombination>,
    pub conflicts: Vec<HotkeyConflict>,
}
```

### Event System
```rust
#[derive(Event, Reflect)]
pub enum ExtensionConfigEvent {
    SearchChanged(String),
    FilterChanged(ExtensionFilter),
    ItemSelected(String),
    ItemToggled { id: String, enabled: bool },
    AliasChanged { id: String, alias: String },
    HotkeyRecorded { id: String, hotkey: KeyCombination },
    HotkeyCleared(String),
    ExtensionExpanded { id: String, expanded: bool },
    ConfigurationSaved(String),
    BulkOperation { action: BulkAction, items: Vec<String> },
}

#[derive(Event, Reflect)]
pub enum ValidationEvent {
    AliasValidation { extension_id: String, alias: String, result: ValidationResult },
    HotkeyValidation { extension_id: String, hotkey: KeyCombination, result: ValidationResult },
}
```

### System Implementation

#### Search and Filtering System
```rust
fn extension_search_system(
    mut registry: ResMut<ExtensionRegistry>,
    mut search_inputs: Query<&mut SearchInput, Changed<SearchInput>>,
    mut events: EventWriter<ExtensionConfigEvent>,
) {
    for search_input in search_inputs.iter_mut() {
        if search_input.is_changed() {
            // Real-time search implementation
            let query = &search_input.text.to_lowercase();
            registry.filtered_results = registry.extensions.iter()
                .filter(|(_, extension)| {
                    extension.manifest.name.to_lowercase().contains(query) ||
                    extension.manifest.description.to_lowercase().contains(query) ||
                    extension.commands.iter().any(|cmd| 
                        cmd.name.to_lowercase().contains(query) ||
                        cmd.alias.as_ref().map_or(false, |a| a.contains(query))
                    )
                })
                .map(|(id, _)| id.clone())
                .collect();
            
            events.send(ExtensionConfigEvent::SearchChanged(search_input.text.clone()));
        }
    }
}

fn filter_tabs_system(
    mut filter_tabs: Query<&mut FilterTabs>,
    mut registry: ResMut<ExtensionRegistry>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<(&Interaction, &FilterTab), Changed<Interaction>>,
) {
    for (interaction, filter_tab) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            for mut tabs in filter_tabs.iter_mut() {
                tabs.active_filter = filter_tab.filter_type;
                
                // Apply filter to registry
                registry.filtered_results = match filter_tab.filter_type {
                    ExtensionFilter::All => registry.extensions.keys().cloned().collect(),
                    ExtensionFilter::Commands => registry.extensions.iter()
                        .filter(|(_, ext)| ext.commands.len() > 0)
                        .map(|(id, _)| id.clone())
                        .collect(),
                    ExtensionFilter::Scripts => registry.extensions.iter()
                        .filter(|(_, ext)| matches!(ext.manifest.extension_type, ExtensionType::Script))
                        .map(|(id, _)| id.clone())
                        .collect(),
                    // Additional filters...
                    _ => vec![],
                };
            }
        }
    }
}
```

#### Hierarchical Table System with Flex Layout
```rust
fn spawn_extensions_config_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            // Search bar with proper flex constraints
            parent.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                flex_grow: 0.0, // Prevent expansion
                ..default()
            }).with_children(|parent| {
                // Search input
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(32.0),
                        max_width: Val::Px(400.0), // Constrain width
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                    SearchInput {
                        text: String::new(),
                        focused: false,
                        placeholder: "Search...".to_string(),
                    },
                ));
            });
            
            // Main content area with 70/30 split
            parent.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0,
                ..default()
            }).with_children(|parent| {
                // Table section (70% with max width constraint)
                spawn_table_section(parent);
                
                // Detail panel (30% with max width constraint)
                spawn_detail_panel(parent);
            });
        });
}

fn spawn_table_section(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Percent(70.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            overflow: Overflow::clip(), // Critical for content clipping
            flex_grow: 0.0, // Prevent expansion beyond 70%
            max_width: Val::Px(800.0), // Constrain maximum width
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        HierarchicalTable::default(),
    )).with_children(|parent| {
        // Filter tabs
        spawn_filter_tabs(parent);
        
        // Table headers
        parent.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(32.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            border: UiRect::bottom(Val::Px(2.0)),
            flex_grow: 0.0,
            ..default()
        }).with_children(|parent| {
            spawn_header_cell(parent, "Name", Val::Percent(40.0));
            spawn_header_cell(parent, "Type", Val::Percent(20.0));
            spawn_header_cell(parent, "Alias", Val::Percent(15.0));
            spawn_header_cell(parent, "Hotkey", Val::Percent(15.0));
            spawn_header_cell(parent, "Enabled", Val::Percent(10.0));
        });
        
        // Scrollable table content
        parent.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip_y(), // Enable vertical scrolling only
            flex_grow: 1.0,
            ..default()
        }).with_children(|parent| {
            // Table rows will be dynamically spawned here
        });
    });
}

fn spawn_detail_panel(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Percent(30.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::left(Val::Px(1.0)),
            flex_grow: 0.0, // Prevent expansion beyond 30%
            max_width: Val::Px(400.0), // Constrain maximum width
            overflow: Overflow::clip(), // Prevent content overflow
            ..default()
        },
        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
        DetailPanel::default(),
    )).with_children(|parent| {
        // Selected extension icon
        parent.spawn(Node {
            width: Val::Px(64.0),
            height: Val::Px(64.0),
            margin: UiRect::bottom(Val::Px(16.0)),
            flex_grow: 0.0,
            ..default()
        });
        
        // Extension details with constrained text
        parent.spawn((
            Text::new("Selected Extension"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(350.0), // Prevent text overflow
                ..default()
            },
        ));
    });
}
```

#### Hotkey Recording System with Button Interaction
```rust
fn hotkey_recording_system(
    mut hotkey_registry: ResMut<HotkeyRegistry>,
    mut recording_state: Local<Option<String>>, // Extension ID being recorded
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut button_query: Query<(&Interaction, &HotkeyRecordButton), Changed<Interaction>>,
    mut text_query: Query<&mut Text>,
    mut events: EventWriter<ValidationEvent>,
) {
    // Handle record button clicks
    for (interaction, record_button) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            *recording_state = Some(record_button.extension_id.clone());
            
            // Update button text to show recording state
            if let Ok(mut text) = text_query.get_mut(record_button.text_entity) {
                **text = "Recording...".to_string();
            }
        }
    }
    
    // Handle keyboard input during recording
    if let Some(extension_id) = recording_state.as_ref() {
        let mut modifiers = Vec::new();
        let mut key = None;
        
        // Capture modifier keys
        if keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight) {
            modifiers.push(Modifier::Control);
        }
        if keyboard_input.pressed(KeyCode::AltLeft) || keyboard_input.pressed(KeyCode::AltRight) {
            modifiers.push(Modifier::Alt);
        }
        if keyboard_input.pressed(KeyCode::SuperLeft) || keyboard_input.pressed(KeyCode::SuperRight) {
            modifiers.push(Modifier::Command);
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
            modifiers.push(Modifier::Shift);
        }
        
        // Capture regular key
        for keycode in keyboard_input.get_just_pressed() {
            if !matches!(keycode, 
                KeyCode::ControlLeft | KeyCode::ControlRight |
                KeyCode::AltLeft | KeyCode::AltRight |
                KeyCode::SuperLeft | KeyCode::SuperRight |
                KeyCode::ShiftLeft | KeyCode::ShiftRight |
                KeyCode::Escape // Allow escape to cancel
            ) {
                key = Some(*keycode);
                break;
            }
        }
        
        // Handle escape key to cancel recording
        if keyboard_input.just_pressed(KeyCode::Escape) {
            *recording_state = None;
            // Reset button text
            return;
        }
        
        if let Some(key_code) = key {
            let combination = KeyCombination {
                modifiers,
                key: key_code,
            };
            
            // Validate hotkey
            let validation_result = validate_hotkey(&combination, &hotkey_registry);
            events.send(ValidationEvent::HotkeyValidation {
                extension_id: extension_id.clone(),
                hotkey: combination.clone(),
                result: validation_result.clone(),
            });
            
            if validation_result.is_valid {
                hotkey_registry.registered_hotkeys.insert(combination, extension_id.clone());
            }
            
            *recording_state = None; // End recording
        }
    }
}

#[derive(Component)]
pub struct HotkeyRecordButton {
    pub extension_id: String,
    pub text_entity: Entity,
}
```

#### Toggle System with Animation
```rust
fn toggle_animation_system(
    time: Res<Time>,
    mut toggle_query: Query<(&mut ExtensionToggle, &mut Node, &mut BackgroundColor), Changed<ExtensionToggle>>,
) {
    for (toggle, mut style, mut bg_color) in toggle_query.iter_mut() {
        // Animate toggle switch appearance
        let target_color = if toggle.enabled {
            Color::srgb(0.2, 0.8, 0.2) // Green for enabled
        } else {
            Color::srgb(0.3, 0.3, 0.3) // Gray for disabled
        };
        
        // Smooth color transition
        let current = bg_color.0;
        let lerp_factor = 8.0 * time.delta_secs();
        bg_color.0 = Color::srgb(
            current.red() + (target_color.red() - current.red()) * lerp_factor,
            current.green() + (target_color.green() - current.green()) * lerp_factor,
            current.blue() + (target_color.blue() - current.blue()) * lerp_factor,
        );
        
        // Update justify_content for toggle switch position
        style.justify_content = if toggle.enabled {
            JustifyContent::FlexEnd
        } else {
            JustifyContent::FlexStart
        };
    }
}
```

### SystemSet Organization with Proper Ordering
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ExtensionConfigSystems {
    Input,
    Search,
    Validation,
    UI,
    Persistence,
}

impl Plugin for ExtensionConfigPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ExtensionItem>()
            .register_type::<ExtensionAlias>()
            .register_type::<ExtensionHotkey>()
            .register_type::<HierarchicalTable>()
            .register_type::<SearchInput>()
            .register_type::<FilterTabs>()
            .register_type::<DetailPanel>()
            
            .init_resource::<ExtensionRegistry>()
            .init_resource::<ExtensionConfigState>()
            .init_resource::<HotkeyRegistry>()
            
            .add_event::<ExtensionConfigEvent>()
            .add_event::<ValidationEvent>()
            
            // Ensure proper system ordering for responsive UI
            .configure_sets(Update, (
                ExtensionConfigSystems::Input,
                ExtensionConfigSystems::Search,
                ExtensionConfigSystems::Validation,
                ExtensionConfigSystems::UI,
                ExtensionConfigSystems::Persistence,
            ).chain())
            
            .add_systems(Update, (
                // Input handling - highest priority
                (
                    keyboard_navigation_system,
                    mouse_interaction_system,
                    hotkey_recording_system,
                ).in_set(ExtensionConfigSystems::Input),
                
                // Search and filtering
                (
                    extension_search_system,
                    filter_tabs_system,
                ).in_set(ExtensionConfigSystems::Search),
                
                // Validation with Changed<T> optimization
                (
                    alias_validation_system,
                    hotkey_validation_system,
                ).in_set(ExtensionConfigSystems::Validation),
                
                // UI updates with efficient queries
                (
                    hierarchical_table_system,
                    detail_panel_system,
                    toggle_animation_system,
                ).in_set(ExtensionConfigSystems::UI),
                
                // Data persistence - lowest priority
                (
                    configuration_save_system,
                    extension_state_sync_system,
                ).in_set(ExtensionConfigSystems::Persistence),
            ));
    }
}
```

### Query Optimization with Changed Detection
```rust
fn optimized_extension_update_system(
    // Only process changed search inputs
    search_inputs: Query<&SearchInput, Changed<SearchInput>>,
    
    // Only process modified extensions
    extensions: Query<&ExtensionItem, Changed<ExtensionItem>>,
    
    // Use Or filter for multiple change types efficiently
    ui_elements: Query<Entity, Or<(
        Changed<ExtensionToggle>,
        Changed<ExtensionAlias>, 
        Changed<ExtensionHotkey>,
    )>>,
    
    // Efficiently query only visible table rows
    visible_rows: Query<&TableRowMarker, (With<Visibility>, Without<Hidden>)>,
) {
    // Only execute expensive operations when actual changes occur
    if !search_inputs.is_empty() {
        // Process search changes efficiently
        for search_input in search_inputs.iter() {
            // Real-time search with debouncing
        }
    }
    
    // Process only changed extensions to minimize work
    for extension in extensions.iter() {
        // Update only modified extension UI elements
    }
}
```

This comprehensive Bevy implementation provides:

1. **Flex-based UI layouts** with proper constraints to prevent expansion
2. **Component-driven architecture** with proper reflection support  
3. **Event-driven patterns** for all menu interactions
4. **Resource management** for persistent state
5. **Query optimization** using `Changed<T>` filters
6. **SystemSet organization** with proper ordering
7. **Animation systems** for smooth UI transitions
8. **Efficient search and filtering** with real-time updates
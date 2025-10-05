# Task 2: Main Table Interface System Implementation

## Objective
Implement the hierarchical table interface with columns (Name, Type, Alias, Hotkey, Enabled), row selection, sorting capabilities, and expandable parent-child extension relationships.

## Implementation Details

### Target Files
- `ui/src/ui/components/config/table_interface.rs:1-350` - Main table component
- `ui/src/ui/components/config/table_row.rs:1-200` - Individual table row components
- `ui/src/ui/components/config/table_header.rs:1-150` - Table header with sorting
- `core/src/config/table_state.rs:1-180` - Table state and data management

### Bevy Implementation Patterns

#### Table Container and Layout
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:150-180` - Table layout with fixed columns
**Reference**: `./docs/bevy/examples/ui/ui.rs:200-240` - Scrollable table container
```rust
// Main table container (70% of split layout)
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(70.0),
        height: Val::Percent(100.0),
        overflow: Overflow::clip_y(),
        ..default()
    },
    background_color: Color::rgba(0.1, 0.1, 0.1, 1.0).into(),
    ..default()
}

// Table header row
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        width: Val::Percent(100.0),
        height: Val::Px(40.0),
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(16.0)),
        border_color: Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
        border: UiRect::bottom(Val::Px(1.0)),
        ..default()
    },
    background_color: Color::rgba(0.12, 0.12, 0.12, 1.0).into(),
    ..default()
}
```

#### Column Header System
**Reference**: `./docs/bevy/examples/ui/button.rs:220-250` - Sortable column headers
**Reference**: `./docs/bevy/examples/ui/ui.rs:300-340` - Column width distribution
```rust
// Column header component with sorting
#[derive(Component)]
pub struct ColumnHeader {
    pub field: SortField,
    pub width_percent: f32,
    pub sortable: bool,
}

// Column header styling
ButtonBundle {
    style: Style {
        width: Val::Percent(column_header.width_percent),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(8.0)),
        ..default()
    },
    background_color: Color::TRANSPARENT.into(),
    ..default()
}

// Column widths: Name(40%), Type(20%), Alias(15%), Hotkey(15%), Enabled(10%)
fn create_table_headers(parent: &mut ChildBuilder, asset_server: &AssetServer) {
    let headers = vec![
        ("Name", SortField::Name, 40.0, true),
        ("Type", SortField::Type, 20.0, true),
        ("Alias", SortField::Alias, 15.0, false),
        ("Hotkey", SortField::Hotkey, 15.0, false),
        ("Enabled", SortField::Enabled, 10.0, true),
    ];
    
    for (title, field, width, sortable) in headers {
        spawn_column_header(parent, title, field, width, sortable, asset_server);
    }
}
```

#### Hierarchical Row System
**Reference**: `./docs/bevy/examples/ui/ui.rs:400-450` - Nested UI hierarchy with indentation
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:220-260` - Expandable sections
```rust
// Extension parent row (expandable)
#[derive(Component)]
pub struct ExtensionParentRow {
    pub extension_id: String,
    pub expanded: bool,
    pub child_count: usize,
}

// Extension child row (command under parent)
#[derive(Component)]
pub struct ExtensionChildRow {
    pub parent_id: String,
    pub command_id: String,
    pub indent_level: usize,
}

// Parent row container
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        width: Val::Percent(100.0),
        height: Val::Px(44.0),
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(16.0)),
        border: UiRect::bottom(Val::Px(1.0)),
        ..default()
    },
    background_color: if is_selected {
        Color::rgb(0.0, 0.48, 1.0).into() // Blue selection
    } else {
        Color::TRANSPARENT.into()
    },
    border_color: Color::rgba(0.2, 0.2, 0.2, 1.0).into(),
    ..default()
}

// Expansion chevron for parent rows
ButtonBundle {
    style: Style {
        width: Val::Px(20.0),
        height: Val::Px(20.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        margin: UiRect::right(Val::Px(8.0)),
        ..default()
    },
    background_color: Color::TRANSPARENT.into(),
    ..default()
}

// Chevron text (▼ expanded, ▶ collapsed)
TextBundle::from_section(
    if parent_row.expanded { "▼" } else { "▶" },
    TextStyle {
        font: font_regular.clone(),
        font_size: 12.0,
        color: Color::rgba(0.6, 0.6, 0.6, 1.0),
    },
)
```

#### Table Row Data Display
**Reference**: `./docs/bevy/examples/ui/ui.rs:500-550` - Row content layout
**Reference**: `./docs/bevy/examples/asset_loading/asset_loading.rs:100-130` - Icon loading for extensions
```rust
// Name column with icon and text
fn create_name_cell(
    parent: &mut ChildBuilder,
    extension_item: &ExtensionItem,
    is_child: bool,
    asset_server: &AssetServer,
) {
    parent.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(40.0),
            align_items: AlignItems::Center,
            padding: UiRect::left(Val::Px(if is_child { 24.0 } else { 0.0 })),
            ..default()
        },
        ..default()
    })
    .with_children(|cell| {
        // Extension/app icon
        cell.spawn(ImageBundle {
            style: Style {
                width: Val::Px(20.0),
                height: Val::Px(20.0),
                margin: UiRect::right(Val::Px(8.0)),
                ..default()
            },
            image: asset_server.load(&extension_item.icon_path).into(),
            ..default()
        });
        
        // Extension/command name
        cell.spawn(TextBundle::from_section(
            extension_item.name.clone(),
            TextStyle {
                font: if is_child { 
                    font_regular.clone() 
                } else { 
                    font_bold.clone() 
                },
                font_size: 14.0,
                color: Color::WHITE,
            },
        ));
    });
}

// Type column display
fn create_type_cell(parent: &mut ChildBuilder, extension_type: &ExtensionType) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(20.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    })
    .with_children(|cell| {
        cell.spawn(TextBundle::from_section(
            extension_type.display_name(),
            TextStyle {
                font: font_regular.clone(),
                font_size: 13.0,
                color: Color::rgba(0.8, 0.8, 0.8, 1.0),
            },
        ));
    });
}
```

### Table State Management

#### Table Data Resource
**Reference**: `./docs/bevy/examples/ecs/resources.rs:80-120` - Table state resource
```rust
// Table state management resource
#[derive(Resource, Clone, Debug)]
pub struct TableState {
    pub extensions: Vec<ExtensionItem>,
    pub selected_items: HashSet<String>,
    pub expanded_extensions: HashSet<String>,
    pub sort_state: SortState,
    pub filter_state: FilterState,
}

#[derive(Debug, Clone)]
pub struct ExtensionItem {
    pub id: String,
    pub name: String,
    pub extension_type: ExtensionType,
    pub icon_path: String,
    pub alias: Option<String>,
    pub hotkey: Option<String>,
    pub enabled: bool,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub metadata: ExtensionMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionType {
    Extension,
    Command,
    AIExtension,
    Script,
    App,
    Quicklink,
    Snippet,
}

impl ExtensionType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ExtensionType::Extension => "Extension",
            ExtensionType::Command => "Command", 
            ExtensionType::AIExtension => "AI Extension",
            ExtensionType::Script => "Script",
            ExtensionType::App => "App",
            ExtensionType::Quicklink => "Quicklink",
            ExtensionType::Snippet => "Snippet",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SortState {
    pub field: SortField,
    pub order: SortOrder,
}

#[derive(Debug, Clone)]
pub struct FilterState {
    pub active_filter: FilterType,
    pub search_query: String,
}
```

#### Row Interaction System
**Reference**: `./docs/bevy/examples/ui/button.rs:280-320` - Row selection and expansion
**Reference**: `./docs/bevy/examples/input/mouse_input.rs:80-120` - Click handling for table rows
```rust
// Table row interaction system
fn table_row_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&ExtensionParentRow>, Option<&ExtensionChildRow>),
        (Changed<Interaction>, With<Button>),
    >,
    mut table_state: ResMut<TableState>,
    mut selection_events: EventWriter<TableSelectionEvent>,
    mut expansion_events: EventWriter<RowExpansionEvent>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for (interaction, mut color, parent_row, child_row) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                let item_id = if let Some(parent) = parent_row {
                    parent.extension_id.clone()
                } else if let Some(child) = child_row {
                    child.command_id.clone()
                } else {
                    continue;
                };
                
                // Handle multi-selection with Cmd/Ctrl
                let multi_select = keyboard_input.pressed(KeyCode::LWin) || 
                                  keyboard_input.pressed(KeyCode::LControl);
                
                if multi_select {
                    if table_state.selected_items.contains(&item_id) {
                        table_state.selected_items.remove(&item_id);
                    } else {
                        table_state.selected_items.insert(item_id.clone());
                    }
                } else {
                    table_state.selected_items.clear();
                    table_state.selected_items.insert(item_id.clone());
                }
                
                selection_events.send(TableSelectionEvent {
                    selected_items: table_state.selected_items.clone(),
                });
                
                // Handle expansion for parent rows
                if let Some(parent) = parent_row {
                    if table_state.expanded_extensions.contains(&parent.extension_id) {
                        table_state.expanded_extensions.remove(&parent.extension_id);
                    } else {
                        table_state.expanded_extensions.insert(parent.extension_id.clone());
                    }
                    
                    expansion_events.send(RowExpansionEvent {
                        extension_id: parent.extension_id.clone(),
                        expanded: table_state.expanded_extensions.contains(&parent.extension_id),
                    });
                }
            }
            Interaction::Hovered => {
                *color = Color::rgba(0.15, 0.15, 0.15, 1.0).into();
            }
            Interaction::None => {
                let item_id = if let Some(parent) = parent_row {
                    &parent.extension_id
                } else if let Some(child) = child_row {
                    &child.command_id
                } else {
                    continue;
                };
                
                *color = if table_state.selected_items.contains(item_id) {
                    Color::rgb(0.0, 0.48, 1.0).into() // Selected blue
                } else {
                    Color::TRANSPARENT.into()
                };
            }
        }
    }
}
```

### Sorting System Implementation

#### Column Header Sorting
**Reference**: `./docs/bevy/examples/ui/button.rs:350-380` - Sortable header interactions
```rust
// Column header sorting system
fn column_header_sort_system(
    mut interaction_query: Query<(&Interaction, &ColumnHeader), Changed<Interaction>>,
    mut table_state: ResMut<TableState>,
    mut sort_events: EventWriter<TableSortEvent>,
) {
    for (interaction, column_header) in interaction_query.iter() {
        if *interaction == Interaction::Clicked && column_header.sortable {
            let new_order = if table_state.sort_state.field == column_header.field {
                // Toggle order if same field
                match table_state.sort_state.order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                }
            } else {
                SortOrder::Ascending
            };
            
            table_state.sort_state = SortState {
                field: column_header.field,
                order: new_order,
            };
            
            sort_events.send(TableSortEvent {
                field: column_header.field,
                order: new_order,
            });
        }
    }
}

// Table sorting implementation
fn apply_table_sort(extensions: &mut Vec<ExtensionItem>, sort_state: &SortState) {
    extensions.sort_by(|a, b| {
        let comparison = match sort_state.field {
            SortField::Name => a.name.cmp(&b.name),
            SortField::Type => a.extension_type.display_name().cmp(&b.extension_type.display_name()),
            SortField::Enabled => a.enabled.cmp(&b.enabled),
            SortField::Alias => a.alias.as_deref().unwrap_or("").cmp(&b.alias.as_deref().unwrap_or("")),
            SortField::Hotkey => a.hotkey.as_deref().unwrap_or("").cmp(&b.hotkey.as_deref().unwrap_or("")),
        };
        
        match sort_state.order {
            SortOrder::Ascending => comparison,
            SortOrder::Descending => comparison.reverse(),
        }
    });
}
```

### Event System Integration

#### Table Events
**Reference**: `./docs/bevy/examples/ecs/event.rs:100-130` - Table interaction events
```rust
// Table interaction events
#[derive(Event)]
pub struct TableSelectionEvent {
    pub selected_items: HashSet<String>,
}

#[derive(Event)] 
pub struct RowExpansionEvent {
    pub extension_id: String,
    pub expanded: bool,
}

#[derive(Event)]
pub struct TableSortEvent {
    pub field: SortField,
    pub order: SortOrder,
}
```

### Architecture Notes

#### Component Structure
- **TableInterface**: Main table container component
- **ExtensionParentRow**: Expandable parent extension rows
- **ExtensionChildRow**: Child command rows with indentation
- **ColumnHeader**: Sortable column header components

#### Data Flow Strategy
- **Hierarchical Display**: Parent extensions contain child commands
- **State-Driven Rendering**: Table renders based on centralized state
- **Event-Driven Updates**: User interactions trigger state changes
- **Performance Optimization**: Virtual scrolling for large datasets

#### Selection and Expansion
- **Multi-Selection**: Support for selecting multiple items with modifier keys
- **Visual Feedback**: Clear selection highlighting with blue background
- **Expansion State**: Persistent expansion state for parent extensions
- **Keyboard Navigation**: Full keyboard accessibility support

### Quality Standards
- Smooth scrolling performance with large extension lists
- Consistent visual hierarchy with proper indentation
- Efficient sorting and filtering with minimal performance impact
- Clear selection feedback with accessible color contrast
- Responsive layout adaptation to different window sizes

### Integration Points
- Detail panel integration for displaying selected item configuration
- Search system integration for real-time filtering
- Extension management system for data updates
- Navigation system integration for filter application
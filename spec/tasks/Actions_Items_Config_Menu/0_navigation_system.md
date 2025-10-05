# Task 0: Navigation System Implementation

## Objective
Implement the dual-level navigation system with primary Extensions tab integration and secondary filter navigation (All, Commands, Scripts, Apps, Quicklinks) with search bar and add functionality.

## Implementation Details

### Target Files
- `ui/src/ui/components/config/navigation_bar.rs:1-200` - Primary and secondary navigation
- `ui/src/ui/components/config/filter_tabs.rs:1-150` - Filter tab system
- `ui/src/ui/components/config/search_bar.rs:1-120` - Search input component
- `core/src/config/navigation_state.rs:1-100` - Navigation state management

### Bevy Implementation Patterns

#### Primary Tab Navigation
**Reference**: `./docs/bevy/examples/ui/ui.rs:50-80` - Tab navigation with active state highlighting
**Reference**: `./docs/bevy/examples/ui/button.rs:40-70` - Tab button interaction and styling
```rust
// Primary navigation container
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        width: Val::Percent(100.0),
        height: Val::Px(48.0),
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: Color::rgba(0.1, 0.1, 0.1, 1.0).into(),
    ..default()
}

// Extensions tab (active state)
ButtonBundle {
    style: Style {
        padding: UiRect {
            left: Val::Px(16.0),
            right: Val::Px(16.0),
            top: Val::Px(8.0),
            bottom: Val::Px(8.0),
        },
        margin: UiRect::right(Val::Px(4.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: Color::rgba(0.2, 0.2, 0.2, 1.0).into(), // Dark active background
    border_radius: BorderRadius::all(Val::Px(4.0)),
    ..default()
}
```

#### Secondary Filter Navigation
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:100-130` - Horizontal filter layout
**Reference**: `./docs/bevy/examples/ui/ui.rs:150-180` - Filter state management
```rust
// Secondary navigation filter bar
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        width: Val::Percent(100.0),
        height: Val::Px(44.0),
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(16.0)),
        gap: Size::all(Val::Px(8.0)),
        ..default()
    },
    background_color: Color::rgba(0.12, 0.12, 0.12, 1.0).into(),
    ..default()
}

// Filter tabs (All, Commands, Scripts, Apps, Quicklinks)
ButtonBundle {
    style: Style {
        padding: UiRect {
            left: Val::Px(12.0),
            right: Val::Px(12.0),
            top: Val::Px(6.0),
            bottom: Val::Px(6.0),
        },
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: if is_active {
        Color::rgba(0.25, 0.25, 0.25, 1.0).into() // Active filter background
    } else {
        Color::TRANSPARENT.into()
    },
    border_radius: BorderRadius::all(Val::Px(4.0)),
    ..default()
}
```

#### Search Bar Integration
**Reference**: `./docs/bevy/examples/ui/text_input.rs:30-60` - Search input with real-time filtering
**Reference**: `./docs/bevy/examples/input/keyboard_input_events.rs:25-55` - Input handling for search
```rust
// Search bar container in secondary navigation
NodeBundle {
    style: Style {
        flex_grow: 1.0,
        height: Val::Px(32.0),
        margin: UiRect::horizontal(Val::Px(16.0)),
        border: UiRect::all(Val::Px(1.0)),
        padding: UiRect::all(Val::Px(8.0)),
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: Color::rgba(0.15, 0.15, 0.15, 1.0).into(),
    border_color: Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(6.0)),
    ..default()
}

// Search input field
TextBundle::from_section(
    "",
    TextStyle {
        font: font_regular.clone(),
        font_size: 14.0,
        color: Color::WHITE,
    },
).with_style(Style {
    flex_grow: 1.0,
    ..default()
})

// Search icon
ImageBundle {
    style: Style {
        width: Val::Px(16.0),
        height: Val::Px(16.0),
        margin: UiRect::left(Val::Px(8.0)),
        ..default()
    },
    image: search_icon_handle.clone().into(),
    ..default()
}
```

### Navigation State Management

#### Filter State Resource
**Reference**: `./docs/bevy/examples/ecs/resources.rs:40-70` - Navigation state resource
```rust
// Navigation state resource for config menu
#[derive(Resource, Clone, Debug)]
pub struct ConfigNavigationState {
    pub active_primary_tab: PrimaryTab,
    pub active_filter: FilterType,
    pub search_query: String,
    pub search_active: bool,
    pub sort_options: SortOptions,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimaryTab {
    Extensions,
    // Future tabs can be added here
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    All,
    Commands,
    Scripts,
    Apps,
    Quicklinks,
}

#[derive(Debug, Clone)]
pub struct SortOptions {
    pub sort_by: SortField,
    pub sort_order: SortOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortField {
    Name,
    Type,
    Enabled,
    LastUsed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for ConfigNavigationState {
    fn default() -> Self {
        Self {
            active_primary_tab: PrimaryTab::Extensions,
            active_filter: FilterType::All,
            search_query: String::new(),
            search_active: false,
            sort_options: SortOptions {
                sort_by: SortField::Name,
                sort_order: SortOrder::Ascending,
            },
        }
    }
}
```

#### Filter Tab Interaction System
**Reference**: `./docs/bevy/examples/ui/button.rs:100-130` - Button interaction for filter changes
**Reference**: `./docs/bevy/examples/ecs/change_detection.rs:30-60` - State change detection
```rust
// Filter tab component for identification
#[derive(Component)]
pub struct FilterTab {
    pub filter_type: FilterType,
}

// Filter tab interaction system
fn filter_tab_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &FilterTab, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut navigation_state: ResMut<ConfigNavigationState>,
    mut filter_events: EventWriter<FilterChangedEvent>,
) {
    for (interaction, filter_tab, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                if navigation_state.active_filter != filter_tab.filter_type {
                    navigation_state.active_filter = filter_tab.filter_type.clone();
                    filter_events.send(FilterChangedEvent {
                        new_filter: filter_tab.filter_type.clone(),
                        search_query: navigation_state.search_query.clone(),
                    });
                }
            }
            Interaction::Hovered => {
                *color = Color::rgba(0.2, 0.2, 0.2, 1.0).into();
            }
            Interaction::None => {
                *color = if navigation_state.active_filter == filter_tab.filter_type {
                    Color::rgba(0.25, 0.25, 0.25, 1.0).into()
                } else {
                    Color::TRANSPARENT.into()
                };
            }
        }
    }
}
```

### Search System Integration

#### Real-time Search Processing
**Reference**: `./docs/bevy/examples/input/text_input.rs:50-80` - Text input handling
**Reference**: `./docs/bevy/examples/async_tasks/async_compute.rs:60-90` - Debounced search processing
```rust
// Search input component
#[derive(Component)]
pub struct SearchInput {
    pub placeholder: String,
    pub debounce_timer: Timer,
}

// Search input system
fn search_input_system(
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut key_input_events: EventReader<KeyboardInput>,
    mut search_query: Query<&mut Text, With<SearchInput>>,
    mut navigation_state: ResMut<ConfigNavigationState>,
    mut search_events: EventWriter<SearchQueryEvent>,
    time: Res<Time>,
) {
    let mut query_changed = false;
    
    // Handle character input
    for event in char_input_events.iter() {
        if event.char.is_control() {
            continue;
        }
        
        navigation_state.search_query.push(event.char);
        query_changed = true;
    }
    
    // Handle special keys (backspace, delete)
    for event in key_input_events.iter() {
        if event.state == ButtonState::Pressed {
            match event.key_code {
                Some(KeyCode::Back) => {
                    navigation_state.search_query.pop();
                    query_changed = true;
                }
                Some(KeyCode::Delete) => {
                    navigation_state.search_query.clear();
                    query_changed = true;
                }
                _ => {}
            }
        }
    }
    
    // Update search input display
    if query_changed {
        for mut text in search_query.iter_mut() {
            text.sections[0].value = if navigation_state.search_query.is_empty() {
                "Search...".to_string()
            } else {
                navigation_state.search_query.clone()
            };
        }
        
        // Send debounced search event
        search_events.send(SearchQueryEvent {
            query: navigation_state.search_query.clone(),
            filter: navigation_state.active_filter.clone(),
        });
    }
}
```

### Add Functionality Integration

#### Add Button Component
**Reference**: `./docs/bevy/examples/ui/button.rs:160-190` - Add button with dropdown integration
```rust
// Add button in secondary navigation
ButtonBundle {
    style: Style {
        width: Val::Px(36.0),
        height: Val::Px(32.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    },
    background_color: Color::rgba(0.2, 0.4, 0.8, 1.0).into(), // Blue add button
    border_color: Color::rgba(0.3, 0.5, 0.9, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(4.0)),
    ..default()
}

// Plus icon for add button
TextBundle::from_section(
    "+",
    TextStyle {
        font: font_medium.clone(),
        font_size: 18.0,
        color: Color::WHITE,
    },
)

// Add button component
#[derive(Component)]
pub struct AddButton {
    pub dropdown_open: bool,
}

// Add button interaction system
fn add_button_system(
    mut interaction_query: Query<(&Interaction, &mut AddButton), Changed<Interaction>>,
    mut add_events: EventWriter<AddExtensionEvent>,
) {
    for (interaction, mut add_button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            add_button.dropdown_open = !add_button.dropdown_open;
            add_events.send(AddExtensionEvent::ShowOptions);
        }
    }
}
```

### Navigation Event System

#### Navigation Events
**Reference**: `./docs/bevy/examples/ecs/event.rs:40-70` - Navigation change events
```rust
// Navigation events
#[derive(Event)]
pub struct FilterChangedEvent {
    pub new_filter: FilterType,
    pub search_query: String,
}

#[derive(Event)]
pub struct SearchQueryEvent {
    pub query: String,
    pub filter: FilterType,
}

#[derive(Event)]
pub struct AddExtensionEvent {
    pub action: AddAction,
}

#[derive(Debug, Clone)]
pub enum AddAction {
    ShowOptions,
    AddCommand,
    AddScript,
    AddQuicklink,
    AddSnippet,
    ImportExtension,
}
```

### Architecture Notes

#### Component Structure
- **ConfigNavigationBar**: Primary navigation container
- **FilterTabContainer**: Secondary filter navigation
- **SearchInput**: Search input field with debouncing
- **AddButton**: Add functionality with dropdown options

#### State Management Strategy
- **Centralized Navigation State**: Single resource for all navigation state
- **Event-Driven Updates**: Navigation changes trigger events for other systems
- **Debounced Search**: Performance-optimized search with input debouncing
- **Filter Persistence**: Navigation state persists across sessions

#### Visual Hierarchy
- **Primary Tab Prominence**: Extensions tab clearly indicated as active
- **Filter Tab Distinction**: Clear active/inactive state for filter tabs
- **Search Integration**: Search bar seamlessly integrated into navigation flow
- **Add Button Accessibility**: Clear call-to-action for adding new items

### Quality Standards
- Smooth navigation transitions with proper state management
- Real-time search with performance optimization through debouncing
- Clear visual feedback for all interactive navigation elements
- Accessible keyboard navigation support for all navigation components
- Consistent styling with overall application theme

### Integration Points
- Main table system integration for filter application
- Detail panel integration for navigation-driven content updates
- Extension management system integration for add functionality
- Search system integration for real-time filtering
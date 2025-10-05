# Main Menu Specification

## Overview
The Main Menu is the primary interface of the Action Items launcher, providing a search-driven interface for discovering and executing commands, applications, and actions from various sources.

## Visual Design

### Layout Structure
- **Search Bar** (top): Primary input field with placeholder "Search for apps and commands..."
- **AI Assistant Button**: "Ask AI | Tab" button in top-right of search bar
- **Favorites Section**: Header labeled "Favorites" above the action list
- **Action List**: Scrollable vertical list of available actions
- **Bottom Bar**: Navigation controls with "Open Command" and "Actions ⌘K"

### Color Scheme & Theme
- Dark theme with charcoal gray background (#2d2d2d)
- Primary text: light gray/white (#ffffff, #f0f0f0)
- Secondary text: medium gray (#999999, #b0b0b0)
- Interactive elements: subtle hover states with background highlights

### Typography
- Primary titles: Medium weight, responsive sizing (~1.8% viewport height)
- Secondary descriptions: Regular weight, responsive sizing (~1.5% viewport height)
- Search placeholder: Regular weight, italic styling
- Tags/badges: Small, responsive sizing (~1.2% viewport height), monospace font

## Functional Requirements

### Search Functionality
1. **Real-time Search**: Filter actions as user types in search bar
2. **Fuzzy Matching**: Support partial and approximate string matching
3. **Multi-source Search**: Search across all registered plugin sources simultaneously
4. **Search History**: Remember and suggest recent searches
5. **Empty State**: Show all favorites when search is empty

### Action Item Display
Each action item must display:
1. **Icon**: Colored icon representing the action type/source (responsive, ~2% VMin)
2. **Primary Title**: Main action name in bold/medium weight
3. **Secondary Description**: Source or detailed description in lighter text
4. **Command Tag**: Dark pill-shaped badge showing command or shortcut
5. **Type Indicator**: "Command" label on the right side
6. **Hover State**: Background highlight and subtle scale effect

### Keyboard Navigation
1. **Arrow Keys**: Navigate up/down through action list
2. **Enter**: Execute selected action
3. **Tab**: Activate AI assistant
4. **⌘K**: Open Actions menu
5. **Escape**: Clear search or close launcher
6. **Letter Keys**: Focus search bar and start typing

### Action Sources Integration
Support for multiple plugin sources:
1. **Search Snippets**: Text snippet management (red icon)
2. **Kill Process**: System process management (yellow/orange icon)
3. **Create Quicklink**: Raycast extension integration (red/pink icon)
4. **Search Crates**: Cargo/crates.io package search (golden icon)
5. **Webpage to Markdown**: Web content conversion (teal icon)
6. **Things Integration**: Task management (blue checkbox icons)
   - Quick Add To-Do
   - Show Today List
   - Show Upcoming List
   - Add New To-Do

### AI Assistant Integration
1. **AI Button**: "Ask AI | Tab" button for natural language queries
2. **Context Awareness**: AI understands current search context
3. **Suggestion System**: AI can suggest relevant actions
4. **Command Generation**: AI can create new commands on demand

### Command Context Menu System (Main_Menu_2.png)
**Per-Command Management**: Right-side overlay for selected commands with actions:
1. **Copy Deeplink** (⌘C) - Copy direct link to command
2. **Configure Command** (⌘,) - Open command-specific settings  
3. **Configure Extension** (⌘,) - Access parent extension settings
4. **Disable Command** (⌘D) - Temporarily disable command
5. **Action Search**: "Search for actions..." field for filtering options
6. **Keyboard Shortcuts**: Full keyboard navigation with visual indicators

## Technical Implementation

### Core Systems Required

#### UI Rendering System
```rust
// Key Bevy examples to reference:
// - ui/scroll.rs - scrollable action list
// - ui/flex_layout.rs - main layout structure
// - ui/button.rs - interactive action items
// - ui/text.rs - text rendering and search input
```

#### Input Handling System
```rust
// Key Bevy examples to reference:
// - input/keyboard_input.rs - keyboard navigation
// - input/keyboard_input_events.rs - key event handling
// - input/text_input.rs - search input text handling
// - input/keyboard_modifiers.rs - modifier key combinations
```

#### Search & Filtering System
```rust
// Key Bevy examples to reference:
// - ecs/event.rs - search events and results
// - ecs/system_param.rs - search system parameters
```

#### Action Execution System
```rust
// Key Bevy examples to reference:
// - ecs/event.rs - action execution events
// - games/game_menu.rs - menu interaction patterns
```

### Component Architecture

#### Action Item Component
```rust
#[derive(Component)]
pub struct ActionItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: IconType,
    pub source: String,
    pub command_tag: String,
    pub action_type: ActionType,
    pub metadata: HashMap<String, String>,
}
```

#### Search State Component
```rust
#[derive(Resource)]
pub struct SearchState {
    pub query: String,
    pub results: Vec<ActionItem>,
    pub selected_index: usize,
    pub is_ai_mode: bool,
}
```

### Event System
```rust
#[derive(Event)]
pub enum LauncherEvent {
    SearchQuery(String),
    ExecuteAction(String),
    NavigateSelection(isize),
    ActivateAI,
    OpenActionsMenu,
}
```

## Performance Requirements

1. **Search Response Time**: < 50ms for query results
2. **Smooth Scrolling**: 60fps scrolling performance
3. **Memory Usage**: Efficient caching of action metadata
4. **Plugin Loading**: Lazy loading of plugin data

## Accessibility Features

1. **Keyboard Navigation**: Full keyboard accessibility
2. **Screen Reader Support**: Proper ARIA labels and descriptions
3. **High Contrast**: Support for high contrast themes
4. **Focus Indicators**: Clear visual focus states
5. **Text Scaling**: Support for system text scaling preferences

## Integration Points

### Plugin System
- **Plugin Registration**: Dynamic registration of action sources
- **Icon Management**: Centralized icon loading and caching
- **Metadata Sync**: Automatic synchronization of plugin data

### AI Integration
- **Natural Language Processing**: Query understanding and intent recognition
- **Context Preservation**: Maintain conversation context across queries
- **Action Suggestions**: Proactive action recommendations

### System Integration
- **Global Hotkeys**: System-wide activation shortcuts
- **Process Management**: Integration with system process controls
- **File System Access**: Integration with file and folder operations

## Related Bevy Examples

### Primary References
- [`ui/scroll.rs`](../../docs/bevy/examples/ui/scroll.rs) - Scrollable list implementation
- [`ui/button.rs`](../../docs/bevy/examples/ui/button.rs) - Interactive button components
- [`ui/text.rs`](../../docs/bevy/examples/ui/text.rs) - Text rendering and input
- [`ui/flex_layout.rs`](../../docs/bevy/examples/ui/flex_layout.rs) - Layout system
- [`input/keyboard_input.rs`](../../docs/bevy/examples/input/keyboard_input.rs) - Keyboard handling
- [`input/text_input.rs`](../../docs/bevy/examples/input/text_input.rs) - Text input systems
- [`games/game_menu.rs`](../../docs/bevy/examples/games/game_menu.rs) - Menu interaction patterns

### Supporting References
- [`ui/overflow.rs`](../../docs/bevy/examples/ui/overflow.rs) - Scrolling behavior
- [`input/keyboard_input_events.rs`](../../docs/bevy/examples/input/keyboard_input_events.rs) - Key event handling
- [`input/keyboard_modifiers.rs`](../../docs/bevy/examples/input/keyboard_modifiers.rs) - Modifier keys
- [`ecs/event.rs`](../../docs/bevy/examples/ecs/event.rs) - Event system patterns
- [`ecs/system_param.rs`](../../docs/bevy/examples/ecs/system_param.rs) - System parameters
- [`ui/ui_texture_atlas.rs`](../../docs/bevy/examples/ui/ui_texture_atlas.rs) - Icon rendering
- [`animation/animated_ui.rs`](../../docs/bevy/examples/animation/animated_ui.rs) - UI animations

## Implementation Priority

### Phase 1: Core Interface
1. Basic search bar with text input
2. Static action list with keyboard navigation
3. Action item display with icons and text
4. Basic search filtering

### Phase 2: Interaction Systems
1. Action execution on Enter key
2. Mouse hover and click support
3. Smooth scrolling implementation
4. Visual feedback and animations

### Phase 3: Advanced Features
1. AI assistant integration
2. Plugin system integration
3. Advanced search with fuzzy matching
4. Performance optimization and caching

### Phase 4: Polish & Accessibility
1. Accessibility enhancements
2. Theme system integration
3. Advanced animations and transitions
4. Error handling and edge cases

## **CRITICAL: Deno Runtime Integration for Main Menu**

**The Main Menu integrates with our existing Deno embedded runtime for extensible functionality:**

### **Core Integration Files**
- **[`core/src/deno_runtime.rs`](../../core/src/deno_runtime.rs)** - Main launcher actions can trigger Deno plugin execution
- **[`core/src/deno_runtime_init.js`](../../core/src/deno_runtime_init.js)** - Plugins have full access to ActionItems API for creating, updating, and searching tasks
- **[`deno-ops/src/lib.rs`](../../deno-ops/src/lib.rs)** - Async operations enable real-time action item management

### **Main Menu Actions via Bevy ECS**
Pure Rust implementation for main menu actions through Bevy systems:

```rust
// Main Menu actions as Bevy systems
#[derive(Resource)]
pub struct MainMenuActionManager {
    pub quick_task_creator: QuickTaskCreator,
    pub smart_search_engine: SmartSearchEngine,
    pub contextual_action_generator: ContextualActionGenerator,
}

#[derive(Component)]
pub struct QuickTaskCreationEvent {
    pub search_text: String,
    pub created_at: std::time::SystemTime,
    pub source: TaskSource,
}

// System that creates quick tasks from search text
pub fn quick_task_creation_system(
    mut action_manager: ResMut<MainMenuActionManager>,
    mut action_items: ResMut<ActionItemsDatabase>,
    mut creation_events: EventReader<QuickTaskCreationEvent>,
    mut task_events: EventWriter<TaskCreatedEvent>,
) {
    for event in creation_events.iter() {
        // Create task from search text
        let task = ActionItem {
            id: format!("quick-task-{}", std::time::SystemTime::now().elapsed().unwrap().as_secs()),
            title: event.search_text.clone(),
            description: "Created from main menu".to_string(),
            tags: vec!["quick-entry".to_string()],
            priority: ActionItemPriority::Medium,
            status: ActionItemStatus::Todo,
            created_at: event.created_at,
        };
        
        // Insert into ActionItems database
        action_items.items.insert(task.id.clone(), task.clone());
        
        // Log successful creation
        println!("Quick task created: {}", task.title);
        
        task_events.send(TaskCreatedEvent {
            task_id: task.id,
            source: TaskCreationSource::MainMenu,
        });
    }
}

// System that provides smart search with AI assistance
pub fn smart_search_system(
    mut action_manager: ResMut<MainMenuActionManager>,
    action_items: Res<ActionItemsDatabase>,
    mut search_events: EventReader<SmartSearchEvent>,
    mut search_results_events: EventWriter<SearchResultsEvent>,
) {
    for event in search_events.iter() {
        // Enhanced search across ActionItems database
        let query = &event.query;
        let search_results: Vec<_> = action_items.items.values()
            .filter(|item| {
                item.title.to_lowercase().contains(&query.to_lowercase()) ||
                item.description.to_lowercase().contains(&query.to_lowercase()) ||
                item.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
            })
            .cloned()
            .collect();
        
        // AI-powered result ranking and suggestions
        let enhanced_results = action_manager.smart_search_engine
            .enhance_search_results(search_results, query);
        
        search_results_events.send(SearchResultsEvent {
            query: query.clone(),
            results: enhanced_results,
            result_count: search_results.len(),
            enhanced: true,
        });
    }
}

// System that generates contextual actions based on selected items
pub fn contextual_actions_system(
    mut action_manager: ResMut<MainMenuActionManager>,
    action_items: Res<ActionItemsDatabase>,
    mut context_events: EventReader<ContextualActionRequestEvent>,
    mut context_results_events: EventWriter<ContextualActionsEvent>,
) {
    for event in context_events.iter() {
        let selected_item = &event.selected_item;
        
        // Find related tasks with matching tags or priority
        let related_tasks: Vec<_> = action_items.items.values()
            .filter(|item| {
                item.id != selected_item.id && (
                    item.tags.iter().any(|tag| selected_item.tags.contains(tag)) ||
                    item.priority == selected_item.priority
                )
            })
            .take(10) // Limit to 10 related items
            .cloned()
            .collect();
        
        // Generate contextual actions based on the selection and related tasks
        let contextual_actions = action_manager.contextual_action_generator
            .generate_actions(selected_item, &related_tasks);
        
        context_results_events.send(ContextualActionsEvent {
            source_item: selected_item.clone(),
            related_tasks_count: related_tasks.len(),
            suggested_actions: contextual_actions,
        });
    }
}

// System that manages main menu state and coordination
pub fn main_menu_coordination_system(
    mut search_state: ResMut<SearchState>,
    mut action_manager: ResMut<MainMenuActionManager>,
    keyboard_input: Res<Input<KeyCode>>,
    mut launcher_events: EventWriter<LauncherEvent>,
) {
    // Handle keyboard shortcuts for main menu actions
    if keyboard_input.just_pressed(KeyCode::Return) {
        if !search_state.query.is_empty() && search_state.results.is_empty() {
            // Create quick task if no search results found
            launcher_events.send(LauncherEvent::CreateQuickTask {
                text: search_state.query.clone(),
            });
        } else if search_state.selected_index < search_state.results.len() {
            // Execute selected action
            let selected_action = &search_state.results[search_state.selected_index];
            launcher_events.send(LauncherEvent::ExecuteAction(selected_action.id.clone()));
        }
    }
    
    // Handle Tab key for AI assistant
    if keyboard_input.just_pressed(KeyCode::Tab) {
        launcher_events.send(LauncherEvent::ActivateAI);
    }
    
    // Handle Command+K for actions menu
    if keyboard_input.pressed(KeyCode::LWin) && keyboard_input.just_pressed(KeyCode::K) {
        launcher_events.send(LauncherEvent::OpenActionsMenu);
    }
}
```

### **Real-time Action Item Integration**
The Main Menu provides live access to action items through the Deno runtime:
- **Dynamic Search**: Real-time search across action items using Deno async operations
- **Quick Actions**: Create, update, complete tasks directly from main interface
- **Smart Suggestions**: AI-powered task suggestions based on patterns and context
- **Plugin Extensions**: Third-party plugins can add custom main menu functionality

## Bevy Implementation Details

### Core Search Interface Components

```rust
use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct SearchInterface {
    pub query: String,
    pub focused: bool,
    pub ai_mode: bool,
    pub results: Vec<ActionItem>,
    pub selected_index: usize,
}

#[derive(Component, Reflect)]
pub struct ActionItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: Handle<Image>,
    pub source: String,
    pub alias: String,
}

#[derive(Resource)]
pub struct MainMenuState {
    pub favorites: Vec<String>,
    pub search_history: Vec<String>,
    pub ai_active: bool,
}

#[derive(Event)]
pub enum MainMenuEvent {
    SearchChanged(String),
    ActionSelected(String),
    ActionExecuted(String),
    AIToggled(bool),
}

// Main search interface with flex layout
fn spawn_main_menu_ui(mut commands: Commands) {
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        overflow: Overflow::clip(), // Prevent expansion
        ..default()
    }).with_children(|parent| {
        // Search bar at top
        parent.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            padding: UiRect::all(Val::Px(16.0)),
            flex_grow: 0.0, // Fixed height
            ..default()
        }).with_children(|parent| {
            // Search input with AI button
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(40.0),
                    max_width: Val::Px(600.0), // Constrain search width
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                SearchInterface {
                    query: String::new(),
                    focused: false,
                    ai_mode: false,
                    results: vec![],
                    selected_index: 0,
                },
            ));
        });
        
        // Scrollable results area
        parent.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip_y(), // Vertical scroll only
            padding: UiRect::all(Val::Px(16.0)),
            flex_grow: 1.0, // Fill remaining space
            ..default()
        });
    });
}

// Real-time search system
fn search_system(
    mut search_interfaces: Query<&mut SearchInterface, Changed<SearchInterface>>,
    mut main_menu_state: ResMut<MainMenuState>,
    mut events: EventWriter<MainMenuEvent>,
) {
    for search_interface in search_interfaces.iter_mut() {
        if search_interface.is_changed() {
            // Perform fuzzy search
            let results = perform_search(&search_interface.query, &main_menu_state.favorites);
            
            // Update results efficiently using Changed<T>
            // This system only runs when search input actually changes
            events.send(MainMenuEvent::SearchChanged(search_interface.query.clone()));
        }
    }
}

fn perform_search(query: &str, favorites: &[String]) -> Vec<ActionItem> {
    // Implement fuzzy search logic
    vec![]
}
```

### Keyboard Navigation System

```rust
fn keyboard_navigation_system(
    mut search_interfaces: Query<&mut SearchInterface>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<MainMenuEvent>,
) {
    for mut interface in search_interfaces.iter_mut() {
        // Arrow key navigation
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            if interface.selected_index < interface.results.len().saturating_sub(1) {
                interface.selected_index += 1;
            }
        }
        
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            if interface.selected_index > 0 {
                interface.selected_index -= 1;
            }
        }
        
        // Execute on Enter
        if keyboard_input.just_pressed(KeyCode::Enter) {
            if let Some(selected) = interface.results.get(interface.selected_index) {
                events.send(MainMenuEvent::ActionExecuted(selected.id.clone()));
            }
        }
        
        // Toggle AI mode on Tab
        if keyboard_input.just_pressed(KeyCode::Tab) {
            interface.ai_mode = !interface.ai_mode;
            events.send(MainMenuEvent::AIToggled(interface.ai_mode));
        }
    }
}
```

### SystemSet Organization

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MainMenuSystems {
    Input,
    Search,
    Navigation,
    AI,
    UI,
}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<SearchInterface>()
            .register_type::<ActionItem>()
            
            .init_resource::<MainMenuState>()
            
            .add_event::<MainMenuEvent>()
            
            .add_systems(Update, (
                // Input handling
                keyboard_navigation_system.in_set(MainMenuSystems::Input),
                
                // Search with Changed<T> optimization
                search_system.in_set(MainMenuSystems::Search),
                
                // UI updates only when needed
                ui_update_system.in_set(MainMenuSystems::UI)
                    .run_if(any_component_changed::<SearchInterface>()),
            ));
    }
}
```
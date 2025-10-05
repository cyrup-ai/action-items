# Actions Menu Specification

## Overview
The Actions Menu represents the primary launcher interface, showcasing the core search and command execution functionality. This interface serves as the main user interaction point for accessing commands, managing favorites, and performing contextual actions on selected items.

## Layout Architecture
- **Top Search Bar**: Universal search interface with AI integration
- **Left Panel**: Favorites and command list (primary content area)
- **Right Panel**: Contextual action menu for selected commands
- **Bottom Action Bar**: Primary action buttons and secondary controls

## Search Interface

### Primary Search Bar
- **Search Input**: Full-width text input with placeholder "Search for apps and commands..."
- **Background**: Dark background with subtle border
- **Font**: Medium gray placeholder text
- **AI Integration**: "Ask AI" and "Tab" buttons positioned right side
- **Real-time Results**: Dynamic filtering and ranking as user types
- **Search Scope**: Commands, applications, files, snippets, and extensions

### AI Search Integration
- **Ask AI Button**: Compact tab-style button with "Ask AI" text
- **Tab Button**: Secondary "Tab" button for keyboard hints
- **Button Styling**: Subtle dark background, consistent with interface theme
- **Positioning**: Right-aligned within search bar container

## Favorites Management System

### Favorites List Structure
- **Section Header**: "Favorites" with optional count or management controls
- **Ranked List**: User-customizable order based on usage and preferences
- **Visual Hierarchy**: Clear distinction between different command types
- **Quick Access**: Optimized for keyboard and mouse interaction

### Command Item Structure
Each favorite item displays:

#### Visual Elements
- **Icon**: Application or command-specific icon for visual identification
- **Command Name**: Primary display name (e.g., "Search Snippets", "Kill Process")
- **Source Information**: Extension or application source (e.g., "Snippets", "Raycast")
- **Alias/Hotkey**: Quick access identifier (e.g., "snip", "kill", "/quicklink")
- **Type Indicator**: Command type classification ("Command")

#### Example Commands from Interface
1. **Search Snippets**
   - **Source**: Snippets
   - **Alias**: snip
   - **Icon**: Red snippets icon
   - **Type**: Command

2. **Kill Process**
   - **Source**: Kill Process
   - **Alias**: kill
   - **Icon**: Yellow warning icon
   - **Type**: Command

3. **Create Quicklink**
   - **Source**: Raycast
   - **Alias**: /quicklink
   - **Icon**: Red link icon
   - **Type**: Command

4. **Search Crates**
   - **Source**: crates.io
   - **Alias**: /cargo-search
   - **Icon**: Yellow package icon
   - **Type**: Command

5. **Webpage to Markdown**
   - **Source**: Convert a webpage to markdown
   - **Icon**: Green conversion icon
   - **Type**: Command

6. **Task Management Commands**
   - **Quick Add To-Do**: Things integration
   - **Show Today List**: Things integration
   - **Show Upcoming List**: Things integration
   - **Add New To-Do**: Things integration

## Contextual Action Menu

### Action Menu Structure
- **Trigger**: Right-click or selection of command item
- **Context Sensitivity**: Actions vary based on selected command type
- **Keyboard Navigation**: Full keyboard accessibility with shortcuts
- **Action Hierarchy**: Grouped actions with clear visual separation

### Standard Actions Available

#### Primary Actions
- **Open Command**: Execute the selected command (⏎ shortcut)
- **Keyboard Shortcut**: Enter key for quick execution

#### Favorites Management
- **Reset Ranking**: Reset usage-based ranking for the command
- **Move Down in Favorites**: Lower priority in favorites list (⌃⌘↓)
- **Remove from Favorites**: Remove from favorites list (⌃⌘F)
- **Move Up**: Increase priority in favorites list (implied functionality)

#### Action Search
- **Search for actions**: Secondary search within available actions
- **Action Filtering**: Real-time filtering of available actions
- **Contextual Actions**: Actions specific to command type and capabilities

## Bottom Action Bar

### Primary Controls
- **Open Command**: Primary execution button with Enter key indicator (⏎)
- **Actions Menu**: Secondary actions button with keyboard shortcut (⌘K)
- **Navigation Hints**: Clear indication of keyboard shortcuts for power users

### Keyboard Navigation
- **Enter Key**: Execute primary action for selected command
- **Command+K**: Open actions menu for additional options
- **Arrow Keys**: Navigate through favorites list
- **Tab Navigation**: Switch between search modes and interface elements

## Visual Design Specifications

### Interface Layout and Theming
- **Background**: Dark theme throughout (#1a1a1a or similar)
- **Search Bar**: Full-width dark input field with rounded corners
- **List Background**: Consistent dark background with subtle item separation
- **Typography**: White primary text, gray secondary text throughout

### Search Interface Styling
- **Search Input**: 
  - Placeholder text: "Search for apps and commands..."
  - Medium gray placeholder color
  - White text on focus
  - Subtle dark border
- **Right Buttons**:
  - "Ask AI" button: Compact, tab-style with subtle background
  - "Tab" button: Matching style, indicating keyboard shortcut

### Favorites List Visual Structure
- **Section Header**: "Favorites" in medium gray text
- **List Items**: Consistent row layout with hover states
- **Item Spacing**: Uniform vertical spacing between items
- **Visual Hierarchy**: Clear distinction between icons, names, and metadata

### Command Item Visual Components
- **Icon Design**:
  - Consistent sizing across all items
  - Colorful icons maintaining app/command branding
  - Sharp, high-resolution rendering
- **Text Layout**:
  - **Primary Name**: White text, medium weight
  - **Source/App**: Gray text, smaller font size
  - **Alias/Shortcut**: Monospace or distinct font in subtle background pill
  - **Type Label**: "Command" in right-aligned gray text

### Context Menu Design System
- **Menu Background**: Dark overlay with rounded corners
- **Menu Items**: 
  - White text for action names
  - Keyboard shortcuts in gray, right-aligned
  - Icon indicators for actions (enter key symbol, etc.)
- **Menu Structure**:
  - Title section: "Search Snippets" with icon
  - Primary action: "Open Command" with enter key icon
  - Secondary actions: Reset, Move, Remove with shortcuts
- **Visual Effects**: Subtle drop shadow and blur for depth

### Bottom Action Bar Styling
- **Background**: Consistent dark theme with top border separation
- **Search Field**: "Search for actions..." with same styling as main search
- **Action Buttons**:
  - "Open Command": Left-aligned with enter key indicator
  - "Actions ⌘K": Right-aligned with keyboard shortcut

### Interactive Element States
- **Hover Effects**: Subtle background lightening on interactive elements
- **Selection State**: Clear visual indication of selected/focused items
- **Keyboard Focus**: Distinct focus rings for accessibility
- **Button States**: Subtle state changes for buttons and clickable elements

### Keyboard Shortcut Visualization
- **Shortcut Pills**: Dark background containers for shortcut text
- **Modifier Keys**: Proper representation of ⌃ ⌘ ⇧ symbols
- **Consistent Styling**: Uniform appearance across all shortcut displays
- **Right Alignment**: Shortcuts consistently right-aligned in lists

### Color Palette Application
- **Primary Text**: Pure white (#FFFFFF) for command names
- **Secondary Text**: Medium gray for sources, descriptions
- **Accent Elements**: App-specific colors preserved in icons
- **Interactive Elements**: Subtle blue accents for focus states
- **Background Hierarchy**: Various dark gray shades for depth

## Functional Requirements

### Search and Discovery System
- **Fuzzy Search**: Intelligent matching of partial queries to commands
- **Ranking Algorithm**: Usage-based ranking with personalization
- **Real-time Indexing**: Dynamic indexing of available commands and applications
- **Search History**: Optional search query history and suggestions

### Favorites Management
- **Automatic Favorites**: AI-powered automatic addition based on usage patterns
- **Manual Management**: User control over favorites list composition and order
- **Usage Analytics**: Tracking of command usage for intelligent ranking
- **Sync Across Devices**: Cloud synchronization of favorites and preferences

### Command Execution System
- **Safe Execution**: Sandboxed execution environment for commands
- **Parameter Handling**: Support for commands requiring additional parameters
- **Output Management**: Proper handling and display of command results
- **Error Recovery**: Graceful error handling and user feedback

### Action System Framework
- **Dynamic Actions**: Context-sensitive action availability
- **Plugin System**: Extensible action system for third-party integrations
- **Batch Operations**: Support for bulk operations on multiple commands
- **Macro Support**: Action sequence recording and playback

## Bevy Implementation Examples

### Search Interface Implementation
- Reference: `./docs/bevy/examples/ui/text_input.rs` - Search input with real-time filtering
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Keyboard input handling and shortcuts

### List Management and Rendering
- Reference: `./docs/bevy/examples/ui/ui.rs` - Dynamic list rendering and management
- Reference: `./docs/bevy/examples/ui/flex_layout.rs` - Flexible list layout with proper spacing

### Icon Management System
- Reference: `./docs/bevy/examples/asset_loading/asset_loading.rs` - Dynamic icon loading for commands
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Efficient icon atlas management

### Contextual Menu System
- Reference: `./docs/bevy/examples/ui/ui.rs` - Dynamic menu generation and positioning
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Right-click and context menu handling

### Keyboard Navigation
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Comprehensive keyboard navigation
- Reference: `./docs/bevy/examples/ui/ui.rs` - Focus management and visual indicators

### Command Execution Framework
- Reference: `./docs/bevy/examples/async_tasks/async_compute.rs` - Asynchronous command execution
- Reference: `./docs/bevy/examples/app/return_after_run.rs` - Command result handling

### Favorites Persistence
- Reference: `./docs/bevy/examples/reflection/reflection.rs` - Favorites state serialization
- Reference: `./docs/bevy/examples/app/return_after_run.rs` - Application state management

## State Management Requirements

### Search State Management
- **Query Persistence**: Maintain search state during navigation
- **Result Caching**: Cache search results for performance optimization
- **History Management**: Search query history with intelligent suggestions
- **Filter State**: Persistent filter preferences across sessions

### Favorites State Synchronization
- **Real-time Updates**: Immediate reflection of favorites changes
- **Cloud Sync**: Bidirectional synchronization with cloud storage
- **Conflict Resolution**: Handling of concurrent favorites modifications
- **Backup and Recovery**: Reliable backup and restoration of favorites data

### Command State Tracking
- **Execution History**: Tracking of command execution patterns and frequency
- **Performance Metrics**: Monitoring command execution times and success rates
- **Error State Management**: Persistent tracking of command failures and recovery
- **Permission State**: Dynamic permission tracking for command execution

## Performance Optimization Requirements

### Search Performance
- **Incremental Search**: Optimized incremental search with debouncing
- **Index Optimization**: Efficient search index management and updates
- **Result Ranking**: Fast ranking algorithms for search result ordering
- **Memory Management**: Efficient memory usage for large command databases

### List Rendering Optimization
- **Virtual Scrolling**: Efficient rendering of large command lists
- **Lazy Loading**: On-demand loading of command metadata and icons
- **Update Optimization**: Minimal re-rendering for list state changes
- **Animation Performance**: Smooth animations without performance impact

### Command Execution Optimization
- **Parallel Execution**: Support for concurrent command execution
- **Resource Management**: Efficient resource allocation and cleanup
- **Caching Strategy**: Intelligent caching of command results and metadata
- **Background Processing**: Non-blocking execution of long-running commands

## Security and Safety Requirements

### Command Execution Security
- **Input Validation**: Comprehensive validation of command parameters
- **Execution Sandboxing**: Isolated execution environment for unsafe commands
- **Permission System**: Granular permission control for system access
- **Audit Logging**: Complete audit trail of command execution and results

### Data Protection
- **Search Privacy**: Protection of search query data and patterns
- **Command History**: Secure storage and optional encryption of command history
- **User Data Protection**: Proper handling of sensitive user data in commands
- **Network Security**: Secure communication for cloud-based features

### System Integration Security
- **Application Access**: Controlled access to system applications and files
- **External Integration**: Secure communication with external services
- **Plugin Security**: Safe execution of third-party plugins and extensions
- **Update Security**: Secure update mechanism for commands and extensions

## Accessibility Requirements

### Keyboard Navigation
- **Complete Keyboard Access**: Full functionality accessible via keyboard
- **Logical Tab Order**: Intuitive keyboard navigation flow
- **Shortcut Consistency**: Consistent keyboard shortcuts across interface
- **Custom Shortcuts**: User-customizable keyboard shortcuts

### Screen Reader Support
- **Semantic Markup**: Proper semantic structure for screen readers
- **Dynamic Announcements**: Real-time announcement of search results and changes
- **Action Descriptions**: Clear descriptions of available actions and their effects
- **Status Updates**: Accessible status information for command execution

### Visual Accessibility
- **High Contrast**: Support for high contrast display modes
- **Font Scaling**: Proper scaling behavior for increased font sizes
- **Color Independence**: Functionality not dependent on color alone
- **Focus Indicators**: Clear visual focus indicators for all interactive elements

## Error Handling and Recovery

### Search Error Handling
- **Index Corruption**: Recovery mechanisms for corrupted search indexes
- **Performance Degradation**: Fallback options for search performance issues
- **Network Failures**: Graceful handling of network-dependent search features
- **Memory Issues**: Recovery from memory pressure during search operations

### Command Execution Error Handling
- **Execution Failures**: User-friendly error messages for failed commands
- **Permission Errors**: Clear guidance for permission-related failures
- **Resource Conflicts**: Handling of resource conflicts during command execution
- **Timeout Management**: Proper timeout handling for long-running commands

### State Management Error Recovery
- **Corrupted State**: Automatic recovery from corrupted application state
- **Sync Failures**: Recovery mechanisms for cloud synchronization failures
- **Backup Restoration**: Reliable restoration from backup data
- **Data Migration**: Safe migration of user data between application versions

## Bevy Implementation Details

### Component Architecture

#### Core Action System Components
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Component, Reflect)]
pub struct ActionItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub alias: String,
    pub icon: Handle<Image>,
    pub action_type: ActionType,
    pub metadata: HashMap<String, String>,
}

#[derive(Component, Reflect)]
pub struct FavoriteItem {
    pub rank: u32,
    pub usage_count: u64,
    pub last_used: std::time::SystemTime,
    pub pinned: bool,
}

#[derive(Component, Reflect)]
pub struct SearchInterface {
    pub query: String,
    pub focused: bool,
    pub ai_mode: bool,
    pub placeholder: String,
}

#[derive(Component, Reflect)]
pub struct ContextualActionMenu {
    pub visible: bool,
    pub target_item: Option<String>,
    pub position: Vec2,
    pub available_actions: Vec<ContextAction>,
    pub selected_index: usize,
}

#[derive(Reflect, Clone)]
pub struct ContextAction {
    pub id: String,
    pub name: String,
    pub shortcut: String,
    pub icon: String,
}
```

#### Action Menu State Management
```rust
#[derive(Resource, Default, Reflect)]
pub struct ActionMenuState {
    pub favorites: Vec<String>, // Action IDs in order
    pub current_selection: Option<String>,
    pub search_results: Vec<String>,
    pub ai_suggestions: Vec<String>,
    pub context_menu_open: bool,
}

#[derive(Resource, Reflect)]
pub struct ActionRegistry {
    pub actions: HashMap<String, ActionItem>,
    pub search_index: SearchIndex,
    pub favorites_dirty: bool,
}

#[derive(Resource, Reflect)]
pub struct AIAssistant {
    pub active: bool,
    pub context: String,
    pub suggestions: Vec<AISuggestion>,
    pub processing: bool,
}
```

### Event System for Action Management
```rust
#[derive(Event, Reflect)]
pub enum ActionMenuEvent {
    SearchQueryChanged(String),
    ActionSelected(String),
    ActionExecuted(String),
    AIActivated,
    AIDeactivated,
    FavoriteAdded(String),
    FavoriteRemoved(String),
    FavoriteReordered { from: usize, to: usize },
    ContextMenuRequested { action_id: String, position: Vec2 },
    ContextMenuClosed,
    ContextActionExecuted { action_id: String, context_action: String },
}

#[derive(Event, Reflect)]
pub enum ActionExecutionEvent {
    Execute(String),
    ExecuteWithParameters { action_id: String, params: HashMap<String, String> },
    ExecutionCompleted { action_id: String, result: ExecutionResult },
    ExecutionFailed { action_id: String, error: String },
}
```

### Search and Filtering System
```rust
fn action_search_system(
    mut search_interfaces: Query<&mut SearchInterface, Changed<SearchInterface>>,
    mut action_menu_state: ResMut<ActionMenuState>,
    action_registry: Res<ActionRegistry>,
    mut events: EventWriter<ActionMenuEvent>,
) {
    for search_interface in search_interfaces.iter_mut() {
        if search_interface.is_changed() {
            let query = &search_interface.query.to_lowercase();
            
            if query.is_empty() {
                // Show favorites when search is empty
                action_menu_state.search_results = action_menu_state.favorites.clone();
            } else {
                // Perform fuzzy search across all actions
                action_menu_state.search_results = action_registry.actions.iter()
                    .filter(|(_, action)| {
                        action.title.to_lowercase().contains(query) ||
                        action.description.to_lowercase().contains(query) ||
                        action.source.to_lowercase().contains(query) ||
                        action.alias.to_lowercase().contains(query) ||
                        action.metadata.values().any(|v| v.to_lowercase().contains(query))
                    })
                    .map(|(id, _)| id.clone())
                    .collect();
                
                // Sort by relevance (title matches first, then description, etc.)
                action_menu_state.search_results.sort_by(|a, b| {
                    let action_a = &action_registry.actions[a];
                    let action_b = &action_registry.actions[b];
                    
                    let score_a = calculate_relevance_score(action_a, query);
                    let score_b = calculate_relevance_score(action_b, query);
                    
                    score_b.partial_cmp(&score_a).unwrap()
                });
            }
            
            events.send(ActionMenuEvent::SearchQueryChanged(search_interface.query.clone()));
        }
    }
}

fn calculate_relevance_score(action: &ActionItem, query: &str) -> f32 {
    let mut score = 0.0;
    
    // Title match gets highest score
    if action.title.to_lowercase().contains(query) {
        score += 10.0;
        if action.title.to_lowercase().starts_with(query) {
            score += 5.0;
        }
    }
    
    // Alias match gets second highest
    if action.alias.to_lowercase().contains(query) {
        score += 8.0;
    }
    
    // Source and description matches
    if action.source.to_lowercase().contains(query) {
        score += 3.0;
    }
    if action.description.to_lowercase().contains(query) {
        score += 2.0;
    }
    
    score
}
```

### Favorites Management System
```rust
fn favorites_management_system(
    mut action_menu_state: ResMut<ActionMenuState>,
    mut favorite_items: Query<&mut FavoriteItem>,
    mut events: EventReader<ActionMenuEvent>,
    time: Res<Time>,
) {
    for event in events.read() {
        match event {
            ActionMenuEvent::ActionExecuted(action_id) => {
                // Update usage statistics for favorites ranking
                if let Some(favorite_entity) = find_favorite_entity(action_id) {
                    if let Ok(mut favorite) = favorite_items.get_mut(favorite_entity) {
                        favorite.usage_count += 1;
                        favorite.last_used = std::time::SystemTime::now();
                    }
                } else {
                    // Auto-add frequently used actions to favorites
                    let usage_count = get_action_usage_count(action_id);
                    if usage_count > 5 && !action_menu_state.favorites.contains(action_id) {
                        action_menu_state.favorites.push(action_id.clone());
                    }
                }
            },
            ActionMenuEvent::FavoriteAdded(action_id) => {
                if !action_menu_state.favorites.contains(action_id) {
                    action_menu_state.favorites.push(action_id.clone());
                }
            },
            ActionMenuEvent::FavoriteRemoved(action_id) => {
                action_menu_state.favorites.retain(|id| id != action_id);
            },
            ActionMenuEvent::FavoriteReordered { from, to } => {
                if *from < action_menu_state.favorites.len() && *to < action_menu_state.favorites.len() {
                    let item = action_menu_state.favorites.remove(*from);
                    action_menu_state.favorites.insert(*to, item);
                }
            },
            _ => {}
        }
    }
}
```

### Contextual Action Menu System
```rust
fn contextual_menu_system(
    mut context_menus: Query<&mut ContextualActionMenu>,
    mut events: EventReader<ActionMenuEvent>,
    action_registry: Res<ActionRegistry>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
) {
    for event in events.read() {
        match event {
            ActionMenuEvent::ContextMenuRequested { action_id, position } => {
                for mut menu in context_menus.iter_mut() {
                    menu.visible = true;
                    menu.target_item = Some(action_id.clone());
                    menu.position = *position;
                    menu.selected_index = 0;
                    
                    // Generate contextual actions based on the selected action
                    if let Some(action) = action_registry.actions.get(action_id) {
                        menu.available_actions = generate_context_actions(action);
                    }
                }
            },
            ActionMenuEvent::ContextMenuClosed => {
                for mut menu in context_menus.iter_mut() {
                    menu.visible = false;
                    menu.target_item = None;
                    menu.available_actions.clear();
                }
            },
            _ => {}
        }
    }
    
    // Handle keyboard navigation in context menu
    for mut menu in context_menus.iter_mut() {
        if menu.visible {
            if keyboard_input.just_pressed(KeyCode::ArrowUp) {
                if menu.selected_index > 0 {
                    menu.selected_index -= 1;
                }
            }
            if keyboard_input.just_pressed(KeyCode::ArrowDown) {
                if menu.selected_index < menu.available_actions.len().saturating_sub(1) {
                    menu.selected_index += 1;
                }
            }
            if keyboard_input.just_pressed(KeyCode::Enter) {
                if let Some(action) = menu.available_actions.get(menu.selected_index) {
                    // Execute selected context action
                }
            }
            if keyboard_input.just_pressed(KeyCode::Escape) {
                menu.visible = false;
            }
        }
    }
    
    // Close context menu when clicking outside
    if mouse_input.just_pressed(MouseButton::Left) {
        for mut menu in context_menus.iter_mut() {
            if menu.visible {
                // Check if click is outside menu bounds
                let window = windows.single();
                if let Some(cursor_pos) = window.cursor_position() {
                    // Implement bounds checking logic
                    menu.visible = false;
                }
            }
        }
    }
}

fn generate_context_actions(action: &ActionItem) -> Vec<ContextAction> {
    let mut actions = vec![
        ContextAction {
            id: "open".to_string(),
            name: "Open Command".to_string(),
            shortcut: "⏎".to_string(),
            icon: "arrow-right".to_string(),
        },
    ];
    
    // Add ranking actions for favorites
    actions.push(ContextAction {
        id: "reset_ranking".to_string(),
        name: "Reset Ranking".to_string(),
        shortcut: "".to_string(),
        icon: "refresh".to_string(),
    });
    
    actions.push(ContextAction {
        id: "move_down".to_string(),
        name: "Move Down in Favorites".to_string(),
        shortcut: "⌃⌘↓".to_string(),
        icon: "arrow-down".to_string(),
    });
    
    actions.push(ContextAction {
        id: "remove_favorite".to_string(),
        name: "Remove from Favorites".to_string(),
        shortcut: "⌃⌘F".to_string(),
        icon: "heart-minus".to_string(),
    });
    
    actions
}
```

### AI Assistant Integration System
```rust
fn ai_assistant_system(
    mut ai_assistant: ResMut<AIAssistant>,
    mut search_interfaces: Query<&mut SearchInterface>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<ActionMenuEvent>,
    action_registry: Res<ActionRegistry>,
) {
    // Handle Tab key to activate AI mode
    if keyboard_input.just_pressed(KeyCode::Tab) {
        ai_assistant.active = !ai_assistant.active;
        
        for mut search_interface in search_interfaces.iter_mut() {
            search_interface.ai_mode = ai_assistant.active;
            if ai_assistant.active {
                search_interface.placeholder = "Ask AI...".to_string();
                events.send(ActionMenuEvent::AIActivated);
            } else {
                search_interface.placeholder = "Search for apps and commands...".to_string();
                events.send(ActionMenuEvent::AIDeactivated);
            }
        }
    }
    
    // Process AI suggestions when in AI mode
    if ai_assistant.active && !ai_assistant.processing {
        for search_interface in search_interfaces.iter() {
            if !search_interface.query.is_empty() {
                ai_assistant.processing = true;
                ai_assistant.context = search_interface.query.clone();
                
                // Generate AI suggestions (this would call actual AI service)
                ai_assistant.suggestions = generate_ai_suggestions(
                    &search_interface.query,
                    &action_registry.actions
                );
                
                ai_assistant.processing = false;
            }
        }
    }
}

fn generate_ai_suggestions(query: &str, actions: &HashMap<String, ActionItem>) -> Vec<AISuggestion> {
    // This would integrate with actual AI service
    // For now, return semantic matches
    vec![]
}

#[derive(Reflect, Clone)]
pub struct AISuggestion {
    pub action_id: String,
    pub confidence: f32,
    pub explanation: String,
}
```

### Flex-Based UI Layout Implementation
```rust
fn spawn_action_menu_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            // Top search bar
            spawn_search_bar(parent);
            
            // Main content area
            parent.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0,
                ..default()
            }).with_children(|parent| {
                // Left panel - Favorites list (70%)
                spawn_favorites_panel(parent);
                
                // Right panel - Contextual actions (30%) 
                spawn_actions_panel(parent);
            });
            
            // Bottom action bar
            spawn_bottom_action_bar(parent);
        });
}

fn spawn_search_bar(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(56.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(16.0)),
        flex_grow: 0.0, // Prevent expansion
        ..default()
    }).with_children(|parent| {
        // Search input field
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                max_width: Val::Px(600.0), // Constrain max width
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                flex_grow: 1.0,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            SearchInterface {
                query: String::new(),
                focused: false,
                ai_mode: false,
                placeholder: "Search for apps and commands...".to_string(),
            },
        ));
        
        // AI button
        parent.spawn((
            Node {
                width: Val::Px(100.0),
                height: Val::Px(32.0),
                margin: UiRect::left(Val::Px(12.0)),
                padding: UiRect::all(Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0, // Prevent expansion
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            Button,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Ask AI | Tab"),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
    });
}

fn spawn_favorites_panel(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(70.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(16.0)),
        overflow: Overflow::clip(), // Prevent expansion
        flex_grow: 0.0, // Prevent expansion beyond 70%
        max_width: Val::Px(500.0), // Constrain maximum width
        ..default()
    }).with_children(|parent| {
        // Favorites header
        parent.spawn((
            Text::new("Favorites"),
            TextFont { 
                font_size: 18.0,
                ..default() 
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ));
        
        // Scrollable favorites list
        parent.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::clip_y(), // Enable vertical scrolling
            flex_grow: 1.0,
            ..default()
        }).with_children(|parent| {
            // Action items will be dynamically spawned here
        });
    });
}

fn spawn_action_item(
    parent: &mut ChildBuilder,
    action: &ActionItem,
    is_selected: bool,
) {
    let bg_color = if is_selected {
        Color::srgb(0.0, 0.5, 1.0) // Blue selection
    } else {
        Color::srgb(0.12, 0.12, 0.12)
    };
    
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(64.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(12.0)),
            margin: UiRect::bottom(Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(bg_color),
        Interaction::default(),
        Button,
    )).with_children(|parent| {
        // Icon
        parent.spawn(Node {
            width: Val::Px(32.0),
            height: Val::Px(32.0),
            margin: UiRect::right(Val::Px(12.0)),
            flex_grow: 0.0,
            ..default()
        });
        
        // Text content
        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            flex_grow: 1.0,
            max_width: Val::Px(300.0), // Prevent text overflow
            overflow: Overflow::clip(),
            ..default()
        }).with_children(|parent| {
            // Action title
            parent.spawn((
                Text::new(&action.title),
                TextFont { 
                    font_size: 16.0,
                    ..default() 
                },
                TextColor(Color::WHITE),
            ));
            
            // Action source
            parent.spawn((
                Text::new(&action.source),
                TextFont { 
                    font_size: 13.0,
                    ..default() 
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
        
        // Alias badge
        if !action.alias.is_empty() {
            parent.spawn((
                Node {
                    padding: UiRect::all(Val::Px(6.0)),
                    margin: UiRect::left(Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    flex_grow: 0.0,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(&action.alias),
                    TextFont { 
                        font_size: 11.0,
                        ..default() 
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });
        }
        
        // Type indicator
        parent.spawn((
            Text::new("Command"),
            TextFont { 
                font_size: 12.0,
                ..default() 
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
            Node {
                margin: UiRect::left(Val::Px(12.0)),
                flex_grow: 0.0,
                ..default()
            },
        ));
    });
}
```

### Bottom Action Bar System
```rust
fn spawn_bottom_action_bar(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(48.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::all(Val::Px(16.0)),
        border: UiRect::top(Val::Px(1.0)),
        flex_grow: 0.0, // Prevent expansion
        ..default()
    }).with_children(|parent| {
        // Primary action button
        parent.spawn((
            Node {
                padding: UiRect::all(Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            Button,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Open Command ⏎"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
        
        // Secondary actions button
        parent.spawn((
            Node {
                padding: UiRect::all(Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            Button,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Actions ⌘K"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
    });
}
```

### SystemSet Organization with Proper Event Handling
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ActionMenuSystems {
    Input,
    Search,
    AI,
    Favorites,
    ContextMenu,
    Execution,
    UI,
}

impl Plugin for ActionMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ActionItem>()
            .register_type::<FavoriteItem>()
            .register_type::<SearchInterface>()
            .register_type::<ContextualActionMenu>()
            
            .init_resource::<ActionMenuState>()
            .init_resource::<ActionRegistry>()
            .init_resource::<AIAssistant>()
            
            .add_event::<ActionMenuEvent>()
            .add_event::<ActionExecutionEvent>()
            
            .configure_sets(Update, (
                ActionMenuSystems::Input,
                ActionMenuSystems::Search,
                ActionMenuSystems::AI,
                ActionMenuSystems::Favorites,
                ActionMenuSystems::ContextMenu,
                ActionMenuSystems::Execution,
                ActionMenuSystems::UI,
            ).chain())
            
            .add_systems(Update, (
                // Input handling
                (
                    keyboard_navigation_system,
                    search_input_system,
                ).in_set(ActionMenuSystems::Input),
                
                // Search and filtering with Changed<T> optimization
                (
                    action_search_system,
                    search_results_update_system,
                ).in_set(ActionMenuSystems::Search),
                
                // AI assistant integration
                (
                    ai_assistant_system,
                    ai_suggestions_system,
                ).in_set(ActionMenuSystems::AI),
                
                // Favorites management
                (
                    favorites_management_system,
                    favorites_ranking_system,
                ).in_set(ActionMenuSystems::Favorites),
                
                // Context menu handling
                (
                    contextual_menu_system,
                    context_menu_positioning_system,
                ).in_set(ActionMenuSystems::ContextMenu),
                
                // Action execution
                (
                    action_execution_system,
                    execution_result_system,
                ).in_set(ActionMenuSystems::Execution),
                
                // UI updates
                (
                    action_list_update_system,
                    selection_highlight_system,
                    animation_system,
                ).in_set(ActionMenuSystems::UI),
            ));
    }
}
```

This comprehensive Bevy implementation provides:

1. **Flex-based UI layouts** preventing unwanted expansion with proper constraints
2. **Event-driven architecture** for all menu interactions and state changes
3. **Component-driven design** with full reflection support for debugging
4. **Real-time search** with fuzzy matching and relevance scoring
5. **AI assistant integration** with contextual suggestions
6. **Favorites management** with automatic ranking based on usage
7. **Contextual action menus** with keyboard navigation support
8. **Query optimization** using `Changed<T>` filters for performance
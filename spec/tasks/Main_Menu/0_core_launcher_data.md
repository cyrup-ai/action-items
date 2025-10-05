# Main Menu - Core Launcher Data Models and State Management

## Task: Implement Core Data Structures for Main Launcher Interface

### File: `ui/src/launcher/mod.rs` (new file)

Create comprehensive data models for main launcher functionality with zero-allocation patterns and blazing-fast search performance.

### Implementation Requirements

#### Action Item Core Structure
- File: `ui/src/launcher/action_item.rs` (new file, line 1-89)
- Implement `ActionItem` component with icon, title, description, command tag, metadata
- Bevy Example Reference: [`ecs/component.rs`](../../../docs/bevy/examples/ecs/component.rs) - Component architecture patterns
- Integration with existing plugin system for multi-source action discovery
- Zero-allocation metadata handling with efficient string storage

#### Search State Management
- File: `ui/src/launcher/search_state.rs` (new file, line 1-67)
- Implement `SearchState` resource with query, results, selected index, AI mode
- Real-time search result updates with change detection
- Bevy Example Reference: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Resource management patterns
- Fuzzy matching integration with configurable sensitivity

#### Plugin Source Integration  
```rust
#[derive(Component, Debug, Clone)]
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

#### Favorites Management System
- File: `ui/src/launcher/favorites.rs` (new file, line 1-78)
- Dynamic favorites list based on usage patterns and manual curation
- AI-powered automatic favorites based on user behavior
- Integration with existing action_items database for persistent storage

#### AI Assistant Integration Data
- File: `ui/src/launcher/ai_integration.rs` (new file, line 1-56)
- Context-aware AI query processing with current search context
- Natural language command generation and suggestion system
- Integration with existing Deno runtime for AI-powered actions

### Architecture Notes
- Use Bevy ECS components for action items with efficient querying
- Resource-based search state with atomic updates
- Integration with existing `core/src/plugins/` system for action discovery
- Event-driven architecture for real-time search and selection
- Zero-allocation string handling for high-performance search

### Integration Points
- `core/src/plugins/` - Plugin action discovery integration (lines 45-123)
- `core/src/search/` - Existing search system integration (lines 23-89) 
- `core/src/runtime/` - Deno runtime integration for dynamic actions (lines 67-134)
- `app/src/events/` - Event system integration for launcher actions

### Multi-Source Action Support
Integration with existing plugin sources:
- Search Snippets (red icon)
- Kill Process (yellow/orange icon) 
- Create Quicklink (red/pink icon)
- Search Crates (golden icon)
- Webpage to Markdown (teal icon)
- Things Integration (blue checkbox icons)

### Bevy Example References
- **Component System**: [`ecs/component.rs`](../../../docs/bevy/examples/ecs/component.rs) - Action item component architecture
- **Resource Management**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Search state resources
- **Event System**: [`ecs/event.rs`](../../../docs/bevy/examples/ecs/event.rs) - Launcher event patterns
- **Query Systems**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Efficient action item queries

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Bevy Implementation Details

### Component Architecture for Core Launcher
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CoreLauncherPanel {
    pub search_query: String,
    pub selected_index: Option<usize>,
    pub ai_mode_active: bool,
    pub search_results: Vec<ActionItem>,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LauncherSystemSet {
    SearchProcessing,
    ActionDiscovery,
    ResultFiltering,
    UIUpdate,
}

impl Plugin for CoreLauncherPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (
            LauncherSystemSet::SearchProcessing,
            LauncherSystemSet::ActionDiscovery,
            LauncherSystemSet::ResultFiltering,
            LauncherSystemSet::UIUpdate,
        ).chain())
        .add_systems(Update, (
            process_search_queries.in_set(LauncherSystemSet::SearchProcessing),
            discover_plugin_actions.in_set(LauncherSystemSet::ActionDiscovery),
            filter_search_results.in_set(LauncherSystemSet::ResultFiltering),
            update_launcher_ui.in_set(LauncherSystemSet::UIUpdate),
        ));
    }
}
```

### Search State Management with Change Detection
```rust
fn process_search_queries(
    mut search_state: ResMut<SearchState>,
    mut search_events: EventReader<SearchEvent>,
    action_items: Query<&ActionItem>,
) {
    for event in search_events.read() {
        match event {
            SearchEvent::QueryChanged(query) => {
                search_state.current_query = query.clone();
                // Trigger search with fuzzy matching
                let results = perform_fuzzy_search(query, &action_items);
                search_state.results = results;
            }
            SearchEvent::SelectionChanged(index) => {
                search_state.selected_index = *index;
            }
        }
    }
}
```
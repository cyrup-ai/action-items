# Actions Menu - Search and Favorites System

## Task: Implement Core Launcher Search Interface with AI Integration

### File: `ui/src/launcher/search_system.rs` (new file)

Create the primary launcher search interface with real-time filtering, AI integration, and blazing-fast favorites management.

### Implementation Requirements

#### Search Interface Component
```rust
#[derive(Component)]
pub struct SearchInterface {
    pub search_query: String,
    pub search_results: Vec<CommandResult>,
    pub ai_mode_active: bool,
    pub favorites_list: Vec<FavoriteCommand>,
    pub selected_index: usize,
}
```

#### Real-time Search System
- File: `ui/src/launcher/search_system.rs` (line 1-156)
- Implement fuzzy search with blazing-fast pattern matching
- Real-time result filtering as user types
- Search scope: commands, applications, files, snippets, extensions
- Intelligent ranking based on usage patterns and relevance

#### AI Integration Interface
- File: `ui/src/launcher/ai_integration.rs` (new file, line 1-89)
- "Ask AI" tab button for switching to AI-powered search
- Natural language query processing
- Contextual AI assistance based on current search state
- Smart suggestions and command recommendations

#### Favorites Management System
- File: `ui/src/launcher/favorites_manager.rs` (new file, line 1-134)
- Dynamic favorites list with user-customizable ordering
- Usage-based automatic favorites addition
- Manual favorites management (add/remove/reorder)
- Cross-device favorites synchronization support

#### Command Result Processing
- File: `ui/src/launcher/command_results.rs` (new file, line 1-123)
- Implement `CommandResult` struct with metadata
- Icon loading and caching for command results
- Source information display (extension, application)
- Command type classification and visual indicators

#### Search Performance Optimization
- File: `ui/src/launcher/search_optimization.rs` (new file, line 1-78)
- Incremental search with debouncing
- Efficient search index management
- Result caching for performance
- Memory-efficient result storage

### Architecture Notes
- Zero-allocation search patterns with string interning
- Integration with existing command system
- Blazing-fast keyboard navigation
- Real-time result updates without UI stuttering

### Integration Points
- Command execution system for result actions
- AI service APIs for intelligent search assistance
- Settings system for search preferences
- Hotkey system for search activation

### Visual Requirements
- Full-width search input with placeholder text
- AI toggle tab with smooth transition animations
- Favorites list with icons, names, and metadata
- Real-time result highlighting and selection

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.## Bevy Implementation Details

### Search Component Architecture

```rust
#[derive(Component, Reflect)]
pub struct SearchIndex {
    pub indexed_commands: HashMap<String, Entity>,
    pub search_trie: SearchTrie,
    pub fuzzy_matcher: FuzzyMatcher,
    pub recent_searches: VecDeque<String>,
    pub search_performance_cache: LRUCache<String, Vec<Entity>>,
}

#[derive(Component, Reflect)]
pub struct SearchState {
    pub current_query: String,
    pub query_cursor: usize,
    pub results_visible: bool,
    pub selected_result_index: usize,
    pub ai_mode_active: bool,
    pub search_scope: SearchScope,
}

#[derive(Component, Reflect)]
pub struct FavoritesList {
    pub favorites: Vec<Entity>, // Entities with FavoriteCommand components
    pub custom_order: Vec<String>,
    pub auto_favorites_enabled: bool,
    pub sync_status: SyncStatus,
}
```

### Real-Time Search System

```rust
fn update_search_results(
    mut commands: Commands,
    mut search_query: Query<(&mut SearchState, &SearchInterface), Changed<SearchState>>,
    search_index: Res<SearchIndex>,
    mut result_events: EventWriter<SearchResultEvent>,
) {
    for (mut state, interface) in &mut search_query {
        if !state.current_query.is_empty() {
            let results = search_index.fuzzy_matcher.find_matches(&state.current_query);
            
            // Update results without allocation in hot path
            result_events.send(SearchResultEvent::ResultsUpdated {
                query: state.current_query.clone(),
                results: results.into_iter().take(10).collect(),
                search_time_ms: 0, // Measured by system
            });
        }
    }
}
```

### Flex-Based Search UI Layout

```rust
fn setup_search_interface(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(600.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            flex_grow: 0.0,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
        SearchInterfaceUI,
    )).with_children(|parent| {
        // Search input field
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                padding: UiRect::horizontal(Val::Px(16.0)),
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
            BorderColor(Color::srgba(0.4, 0.4, 0.8, 1.0)),
            SearchInput,
        ));
        
        // AI toggle button
        parent.spawn((
            Node {
                width: Val::Px(120.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.3, 0.3, 0.8, 1.0)),
            AiToggleButton,
            Interaction::default(),
        ));
    });
}
```

### Search Event System

```rust
#[derive(Event, Debug)]
pub enum SearchEvent {
    QueryChanged(String),
    ResultSelected(Entity),
    AiModeToggled(bool),
    FavoriteAdded(Entity),
    FavoriteRemoved(Entity),
}

fn handle_search_events(
    mut search_events: EventReader<SearchEvent>,
    mut search_state: Query<&mut SearchState>,
    mut favorites: Query<&mut FavoritesList>,
) {
    for event in search_events.read() {
        match event {
            SearchEvent::QueryChanged(query) => {
                for mut state in &mut search_state {
                    state.current_query = query.clone();
                    state.selected_result_index = 0;
                }
            },
            SearchEvent::FavoriteAdded(entity) => {
                for mut fav_list in &mut favorites {
                    if !fav_list.favorites.contains(entity) {
                        fav_list.favorites.push(*entity);
                    }
                }
            },
            _ => {}
        }
    }
}
```
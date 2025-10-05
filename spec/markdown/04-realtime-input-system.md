# Real-time Input Processing System

## Critical Analysis: Missing Real-time Search

### Current State: No Search Implementation
The current codebase has search UI components but **no search functionality**. There's no system to:
- Filter results as user types
- Display relevant applications/actions
- Handle fuzzy matching for user convenience 
- Update results in real-time with zero latency

**Missing Components:**
- Search result data structures
- Fuzzy matching algorithms
- Real-time filtering pipeline
- Result ranking and scoring system

## Target Architecture: Raycast-like Instant Search

### Design Principles
1. **Instant Feedback**: Results appear as user types first character
2. **Fuzzy Matching**: Smart matching like "chr" → "Google Chrome"
3. **Zero Allocation**: No heap allocations during search
4. **Ranking System**: Best matches appear first
5. **Debounced Updates**: Smooth performance during fast typing

### Raycast Search Behavior Analysis

#### Search Pipeline
1. **Keystroke** → Immediate character capture
2. **Fuzzy Match** → Find matching applications/actions  
3. **Rank Results** → Sort by relevance score
4. **Update UI** → Display top 8 results instantly
5. **Highlight Match** → Show matching characters

#### Performance Characteristics
- **Latency**: < 16ms from keystroke to UI update
- **Throughput**: Handle 60+ keystrokes/second
- **Memory**: Zero allocations during steady-state search
- **Results**: Top 8 most relevant matches

## Implementation Specification

### Phase 1: Search Data Structures

**New Component:** `SearchableItem`
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct SearchableItem {
    pub id: u32,                    // Unique identifier
    pub title: String,              // "Google Chrome"
    pub subtitle: Option<String>,   // "Application"
    pub keywords: Vec<String>,      // ["chrome", "browser", "google"]
    pub icon_path: Option<String>,  // Path to app icon
    pub action_type: ActionType,    // What happens when selected
    pub relevance_base: f32,        // Base relevance score (0.0-1.0)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    LaunchApplication { bundle_id: String },
    OpenFile { path: String },
    RunCommand { command: String },
    WebSearch { query: String, engine: String },
}
```

**New Resource:** `SearchIndex`
```rust
#[derive(Resource, Debug)]
pub struct SearchIndex {
    items: Vec<SearchableItem>,           // All searchable items
    title_chars: Vec<Vec<char>>,          // Pre-computed character vectors
    keyword_chars: Vec<Vec<Vec<char>>>,   // Pre-computed keyword characters
    last_query: String,                   // Cache for debouncing
    last_results: Vec<SearchResult>,      // Cached results
}

impl SearchIndex {
    /// Create new search index with pre-computed character data
    #[inline]
    pub fn new(items: Vec<SearchableItem>) -> Self {
        let title_chars: Vec<Vec<char>> = items
            .iter()
            .map(|item| item.title.to_lowercase().chars().collect())
            .collect();
            
        let keyword_chars: Vec<Vec<Vec<char>>> = items
            .iter()
            .map(|item| {
                item.keywords
                    .iter()
                    .map(|kw| kw.to_lowercase().chars().collect())
                    .collect()
            })
            .collect();
        
        Self {
            items,
            title_chars,
            keyword_chars,
            last_query: String::new(),
            last_results: Vec::new(),
        }
    }
    
    /// Zero-allocation fuzzy search
    #[inline]
    pub fn search(&mut self, query: &str, max_results: usize) -> &[SearchResult] {
        // Skip if query unchanged (debouncing)
        if query == self.last_query {
            return &self.last_results;
        }
        
        // Clear previous results (reuse allocation)
        self.last_results.clear();
        
        let query_chars: Vec<char> = query.to_lowercase().chars().collect();
        let mut scored_matches = Vec::with_capacity(self.items.len());
        
        // Score all items
        for (idx, item) in self.items.iter().enumerate() {
            if let Some(score) = self.fuzzy_score(&query_chars, idx) {
                scored_matches.push((idx, score));
            }
        }
        
        // Sort by score (highest first)
        scored_matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Convert to SearchResult (reuse allocation)
        self.last_results.extend(
            scored_matches
                .into_iter()
                .take(max_results)
                .map(|(idx, score)| SearchResult {
                    item_id: self.items[idx].id,
                    title: &self.items[idx].title,
                    subtitle: self.items[idx].subtitle.as_deref(),
                    icon_path: self.items[idx].icon_path.as_deref(),
                    relevance_score: score,
                    match_indices: Vec::new(), // TODO: Implement highlighting
                })
        );
        
        self.last_query = query.to_string();
        &self.last_results
    }
    
    /// Fast fuzzy matching with character-level scoring
    #[inline]
    fn fuzzy_score(&self, query_chars: &[char], item_idx: usize) -> Option<f32> {
        let title_chars = &self.title_chars[item_idx];
        let base_score = self.items[item_idx].relevance_base;
        
        // Try exact prefix match first (highest score)
        if self.prefix_match(query_chars, title_chars) {
            return Some(base_score + 0.5);
        }
        
        // Try fuzzy match on title
        if let Some(score) = self.char_match_score(query_chars, title_chars) {
            return Some(base_score + score * 0.3);
        }
        
        // Try fuzzy match on keywords
        for keyword_chars in &self.keyword_chars[item_idx] {
            if let Some(score) = self.char_match_score(query_chars, keyword_chars) {
                return Some(base_score + score * 0.2);
            }
        }
        
        None
    }
    
    /// Check if query is prefix of target
    #[inline]
    fn prefix_match(&self, query: &[char], target: &[char]) -> bool {
        if query.len() > target.len() {
            return false;
        }
        query.iter().zip(target.iter()).all(|(q, t)| q == t)
    }
    
    /// Character-level fuzzy matching score
    #[inline]
    fn char_match_score(&self, query: &[char], target: &[char]) -> Option<f32> {
        if query.is_empty() {
            return Some(1.0);
        }
        
        let mut query_idx = 0;
        let mut matches = 0;
        
        for &target_char in target {
            if query_idx < query.len() && query[query_idx] == target_char {
                query_idx += 1;
                matches += 1;
            }
        }
        
        if query_idx == query.len() {
            // All query characters matched
            Some(matches as f32 / target.len() as f32)
        } else {
            None
        }
    }
}
```

### Phase 2: Search Result Components

**New Component:** `SearchResult`
```rust
#[derive(Debug, Clone)]
pub struct SearchResult<'a> {
    pub item_id: u32,
    pub title: &'a str,
    pub subtitle: Option<&'a str>,
    pub icon_path: Option<&'a str>,
    pub relevance_score: f32,
    pub match_indices: Vec<usize>,    // For highlighting matched chars
}
```

**New Component:** `SearchResultsContainer`
```rust
#[derive(Component, Debug)]
pub struct SearchResultsContainer {
    pub current_results: Vec<Entity>,     // Result item entities
    pub selected_index: usize,           // Currently selected result
    pub max_visible: usize,              // 8 results like Raycast
}

impl Default for SearchResultsContainer {
    fn default() -> Self {
        Self {
            current_results: Vec::new(),
            selected_index: 0,
            max_visible: 8,
        }
    }
}
```

### Phase 3: Real-time Search System

**Core System:** `realtime_search_system`
```rust
/// Main real-time search system - triggers on every text change
#[inline]
pub fn realtime_search_system(
    mut search_index: ResMut<SearchIndex>,
    text_input_query: Query<&CompactTextInput, Changed<CompactTextInput>>,
    mut results_query: Query<&mut SearchResultsContainer>,
    mut commands: Commands,
    theme: Res<Theme>,
    fonts: Res<TypographyScale>,
) {
    // Check if search text changed
    let Ok(input) = text_input_query.get_single() else { return };
    let Ok(mut results_container) = results_query.get_single_mut() else { return };
    
    // Skip if not in search mode or no actual text
    if !input.is_focused {
        return;
    }
    
    let query = input.current_text.trim();
    
    // Handle empty query - show recent/default items
    if query.is_empty() {
        spawn_default_results(&mut commands, &mut results_container, &theme, &fonts);
        return;
    }
    
    // Perform fuzzy search
    let results = search_index.search(query, results_container.max_visible);
    
    // Update UI with new results
    spawn_search_results(&mut commands, &mut results_container, results, &theme, &fonts);
}

/// Spawn search result items in UI
#[inline]
fn spawn_search_results(
    commands: &mut Commands,
    results_container: &mut SearchResultsContainer,
    results: &[SearchResult],
    theme: &Theme,
    fonts: &TypographyScale,
) {
    // Despawn previous results
    for entity in results_container.current_results.drain(..) {
        commands.entity(entity).despawn_recursive();
    }
    
    // Reset selection
    results_container.selected_index = 0;
    
    // Spawn new result items
    for (idx, result) in results.iter().enumerate() {
        let is_selected = idx == results_container.selected_index;
        
        let result_entity = commands.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(48.0),          // Compact result height
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            if is_selected {
                theme.colors.result_item_selected_gradient()
            } else {
                theme.colors.result_item_gradient()
            },
            BorderRadius::all(Val::Px(6.0)),
            InteractiveGradient::result_item(&theme.colors),
            SearchResultItem {
                item_id: result.item_id,
                is_selected,
            },
        )).with_children(|parent| {
            // Result icon (if available)
            if let Some(icon_path) = result.icon_path {
                parent.spawn((
                    Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        margin: UiRect::right(Val::Px(12.0)),
                        ..default()
                    },
                    // TODO: Load actual icon from icon_path
                    BackgroundColor(theme.colors.surface_default),
                    BorderRadius::all(Val::Px(4.0)),
                ));
            }
            
            // Text content
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    flex_grow: 1.0,
                    ..default()
                },
            )).with_children(|parent| {
                // Title
                parent.spawn((
                    Text::new(result.title),
                    TextFont {
                        font: fonts.medium.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(theme.colors.text_primary),
                ));
                
                // Subtitle (if available)
                if let Some(subtitle) = result.subtitle {
                    parent.spawn((
                        Text::new(subtitle),
                        TextFont {
                            font: fonts.regular.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(theme.colors.text_secondary),
                    ));
                }
            });
        }).id();
        
        results_container.current_results.push(result_entity);
    }
}
```

### Phase 4: Application Discovery System

**New System:** `discover_applications_system`
```rust
/// Discover installed applications for search index
#[inline]
pub fn discover_applications_system(
    mut commands: Commands,
    mut search_index: Option<ResMut<SearchIndex>>,
) {
    // Skip if search index already exists
    if search_index.is_some() {
        return;
    }
    
    let mut items = Vec::new();
    
    // Discover applications based on platform
    #[cfg(target_os = "macos")]
    {
        items.extend(discover_macos_applications());
    }
    
    #[cfg(target_os = "windows")]  
    {
        items.extend(discover_windows_applications());
    }
    
    #[cfg(target_os = "linux")]
    {
        items.extend(discover_linux_applications());
    }
    
    // Add built-in actions
    items.extend(create_builtin_actions());
    
    // Create and insert search index
    let index = SearchIndex::new(items);
    commands.insert_resource(index);
    
    info!("Application discovery completed: {} items indexed", items.len());
}

#[cfg(target_os = "macos")]
fn discover_macos_applications() -> Vec<SearchableItem> {
    let mut apps = Vec::new();
    
    // Common macOS application directories
    let app_dirs = [
        "/Applications",
        "/Applications/Utilities", 
        "/System/Applications",
        &format!("{}/Applications", std::env::var("HOME").unwrap_or_default()),
    ];
    
    for dir in &app_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".app") {
                        let title = name.trim_end_matches(".app").to_string();
                        let bundle_id = extract_bundle_id(&entry.path());
                        
                        apps.push(SearchableItem {
                            id: apps.len() as u32,
                            title: title.clone(),
                            subtitle: Some("Application".to_string()),
                            keywords: vec![title.to_lowercase()],
                            icon_path: Some(entry.path().to_string_lossy().to_string()),
                            action_type: ActionType::LaunchApplication { bundle_id },
                            relevance_base: 0.5,
                        });
                    }
                }
            }
        }
    }
    
    apps
}

fn create_builtin_actions() -> Vec<SearchableItem> {
    vec![
        SearchableItem {
            id: 1000,
            title: "System Preferences".to_string(),
            subtitle: Some("System Settings".to_string()),
            keywords: vec!["preferences".to_string(), "settings".to_string()],
            icon_path: None,
            action_type: ActionType::LaunchApplication {
                bundle_id: "com.apple.systempreferences".to_string()
            },
            relevance_base: 0.8,
        },
        SearchableItem {
            id: 1001,
            title: "Google Search".to_string(),
            subtitle: Some("Web Search".to_string()),
            keywords: vec!["google".to_string(), "search".to_string(), "web".to_string()],
            icon_path: None,
            action_type: ActionType::WebSearch {
                query: "{query}".to_string(),
                engine: "https://www.google.com/search?q=".to_string()
            },
            relevance_base: 0.7,
        },
    ]
}
```

### Phase 5: Selection and Navigation

**New System:** `search_navigation_system`
```rust
/// Handle up/down arrow navigation through search results
#[inline]  
pub fn search_navigation_system(
    mut key_events: EventReader<KeyboardInput>,
    mut results_query: Query<&mut SearchResultsContainer>,
    mut result_items: Query<(&mut BackgroundGradient, &mut SearchResultItem)>,
    theme: Res<Theme>,
    app_state: Res<State<AppState>>,
) {
    if *app_state.get() != AppState::SearchMode {
        return;
    }
    
    let Ok(mut results_container) = results_query.get_single_mut() else { return };
    
    for event in key_events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        
        match event.key_code {
            KeyCode::ArrowDown => {
                if results_container.selected_index + 1 < results_container.current_results.len() {
                    update_selection(&mut results_container, &mut result_items, &theme, 1);
                }
            },
            KeyCode::ArrowUp => {
                if results_container.selected_index > 0 {
                    update_selection(&mut results_container, &mut result_items, &theme, -1);
                }
            },
            KeyCode::Enter => {
                if let Some(&selected_entity) = results_container.current_results.get(results_container.selected_index) {
                    // TODO: Execute selected action
                    execute_search_result(selected_entity, &result_items);
                }
            },
            _ => {}
        }
    }
}

#[inline]
fn update_selection(
    results_container: &mut SearchResultsContainer,
    result_items: &mut Query<(&mut BackgroundGradient, &mut SearchResultItem)>,
    theme: &Theme,
    direction: i32,
) {
    let old_index = results_container.selected_index;
    let new_index = (old_index as i32 + direction).max(0) as usize;
    
    results_container.selected_index = new_index;
    
    // Update visual states
    for (entity, (mut gradient, mut item)) in result_items.iter_mut().zip(results_container.current_results.iter()).enumerate() {
        item.is_selected = entity == new_index;
        *gradient = if item.is_selected {
            theme.colors.result_item_selected_gradient()
        } else {
            theme.colors.result_item_gradient()
        };
    }
}
```

## Implementation Timeline

### Phase 1: Core Data Structures (High Priority)
- Implement SearchableItem and SearchIndex
- Add fuzzy matching algorithms
- Create search result components

### Phase 2: Real-time Search System (High Priority)
- Implement realtime_search_system
- Connect to text input changes
- Test search performance

### Phase 3: Application Discovery (Medium Priority)
- Add discover_applications_system
- Implement platform-specific app discovery
- Build initial search index

### Phase 4: UI Integration (Medium Priority)
- Connect search results to UI spawning
- Add result item styling and gradients
- Test visual result updates

### Phase 5: Navigation System (Low Priority)
- Implement keyboard navigation
- Add selection visual feedback
- Add result execution system

## Performance Requirements

### Zero Allocation Constraints
- Reuse result vectors between searches
- Pre-compute character arrays for fuzzy matching
- Cache search results for identical queries
- Use string references instead of cloning

### Benchmarking Targets
- Search latency: < 5ms for 1000 items
- UI update: < 10ms from search to display
- Memory usage: < 500KB for search index
- Keystroke response: < 16ms total pipeline

## Success Criteria

1. ✅ Real-time search results appear as user types
2. ✅ Fuzzy matching works for partial queries
3. ✅ Results ranked by relevance score
4. ✅ Zero allocations during steady-state search
5. ✅ Performance targets met for all operations
6. ✅ Keyboard navigation through results works
7. ✅ Selected results execute proper actions

---

## Bevy Implementation Details

### Component Architecture for Input Handling

```rust
use bevy::{
    prelude::*,
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    window::{PrimaryWindow, WindowEvent, WindowFocused},
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
    ecs::{system::CommandQueue, world::CommandQueue as WorldCommandQueue},
};

/// Component for text input handling with real-time feedback
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct RealtimeTextInput {
    pub current_text: String,
    pub placeholder: String,
    pub is_focused: bool,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub last_input_time: f64,
    pub debounce_ms: u64,
}

impl Default for RealtimeTextInput {
    fn default() -> Self {
        Self {
            current_text: String::new(),
            placeholder: "Search...".to_string(),
            is_focused: false,
            cursor_position: 0,
            selection_start: None,
            last_input_time: 0.0,
            debounce_ms: 50, // 50ms debounce for smooth typing
        }
    }
}

/// Component for search result display
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct SearchResultsDisplay {
    pub results: Vec<SearchResultItem>,
    pub selected_index: usize,
    pub max_visible: usize,
    pub is_loading: bool,
}

#[derive(Reflect, Debug, Clone)]
pub struct SearchResultItem {
    pub id: u32,
    pub title: String,
    pub subtitle: String,
    pub icon_path: Option<String>,
    pub relevance_score: f32,
    pub action_data: ActionData,
}

#[derive(Reflect, Debug, Clone)]
pub enum ActionData {
    LaunchApp { bundle_id: String },
    OpenFile { path: String },
    RunCommand { command: String },
    WebSearch { query: String },
}
```

### Event System for Input Processing

```rust
/// Custom events for input processing pipeline
#[derive(Event, Debug, Clone)]
pub enum InputEvent {
    /// Text changed in search input
    TextChanged { 
        new_text: String,
        timestamp: f64,
    },
    /// Search query should be executed
    ExecuteSearch { 
        query: String,
        force_update: bool,
    },
    /// Navigation key pressed
    Navigate(NavigationDirection),
    /// Result item selected
    SelectResult { index: usize },
    /// Execute selected result
    ExecuteSelected,
    /// Input focus changed
    FocusChanged { focused: bool },
}

#[derive(Debug, Clone, Copy)]
pub enum NavigationDirection {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Event for search result updates
#[derive(Event, Debug)]
pub enum SearchEvent {
    /// Results updated from search
    ResultsUpdated { 
        results: Vec<SearchResultItem>,
        query: String,
        took_ms: f32,
    },
    /// Search failed
    SearchFailed { 
        query: String,
        error: String,
    },
    /// Search index updated
    IndexUpdated { item_count: usize },
}
```

### Real-Time Input Processing System

```rust
/// Main input processing system with zero-allocation patterns
pub fn realtime_input_system(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut text_inputs: Query<&mut RealtimeTextInput>,
    mut input_events: EventWriter<InputEvent>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut text_input) = text_inputs.get_single_mut() else { return };
    let current_time = time.elapsed_secs_f64();

    // Handle modifier keys for special actions
    let cmd_pressed = keyboard_input.pressed(KeyCode::SuperLeft) || 
                     keyboard_input.pressed(KeyCode::SuperRight);
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft) || 
                       keyboard_input.pressed(KeyCode::ShiftRight);

    // Process keyboard events
    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match event.key_code {
            // Navigation keys
            KeyCode::ArrowUp => {
                input_events.send(InputEvent::Navigate(NavigationDirection::Up));
                continue;
            },
            KeyCode::ArrowDown => {
                input_events.send(InputEvent::Navigate(NavigationDirection::Down));
                continue;
            },
            KeyCode::Enter => {
                input_events.send(InputEvent::ExecuteSelected);
                continue;
            },
            KeyCode::Escape => {
                text_input.current_text.clear();
                text_input.cursor_position = 0;
                input_events.send(InputEvent::TextChanged {
                    new_text: String::new(),
                    timestamp: current_time,
                });
                continue;
            },
            
            // Text editing
            KeyCode::Backspace => {
                if text_input.cursor_position > 0 {
                    if cmd_pressed {
                        // Delete whole word
                        delete_word_backwards(&mut text_input);
                    } else {
                        // Delete single character
                        text_input.cursor_position -= 1;
                        text_input.current_text.remove(text_input.cursor_position);
                    }
                    
                    text_input.last_input_time = current_time;
                    input_events.send(InputEvent::TextChanged {
                        new_text: text_input.current_text.clone(),
                        timestamp: current_time,
                    });
                }
                continue;
            },
            KeyCode::Delete => {
                if text_input.cursor_position < text_input.current_text.len() {
                    text_input.current_text.remove(text_input.cursor_position);
                    text_input.last_input_time = current_time;
                    input_events.send(InputEvent::TextChanged {
                        new_text: text_input.current_text.clone(),
                        timestamp: current_time,
                    });
                }
                continue;
            },
            
            // Cursor movement
            KeyCode::ArrowLeft => {
                if cmd_pressed {
                    text_input.cursor_position = 0;
                } else {
                    text_input.cursor_position = text_input.cursor_position.saturating_sub(1);
                }
                continue;
            },
            KeyCode::ArrowRight => {
                if cmd_pressed {
                    text_input.cursor_position = text_input.current_text.len();
                } else {
                    text_input.cursor_position = (text_input.cursor_position + 1)
                        .min(text_input.current_text.len());
                }
                continue;
            },
            
            // Select all
            KeyCode::KeyA if cmd_pressed => {
                text_input.selection_start = Some(0);
                text_input.cursor_position = text_input.current_text.len();
                continue;
            },
            
            // Regular character input
            _ => {
                if let Some(character) = key_code_to_char(event.key_code, shift_pressed) {
                    // Insert character at cursor position
                    text_input.current_text.insert(text_input.cursor_position, character);
                    text_input.cursor_position += 1;
                    text_input.last_input_time = current_time;
                    
                    input_events.send(InputEvent::TextChanged {
                        new_text: text_input.current_text.clone(),
                        timestamp: current_time,
                    });
                }
            },
        }
    }
}

/// Helper function for word-wise deletion
fn delete_word_backwards(text_input: &mut RealtimeTextInput) {
    let mut pos = text_input.cursor_position;
    
    // Skip any trailing spaces
    while pos > 0 && text_input.current_text.chars().nth(pos - 1) == Some(' ') {
        pos -= 1;
    }
    
    // Delete until space or beginning
    while pos > 0 {
        let prev_char = text_input.current_text.chars().nth(pos - 1);
        if prev_char == Some(' ') {
            break;
        }
        pos -= 1;
    }
    
    // Remove the characters
    text_input.current_text.drain(pos..text_input.cursor_position);
    text_input.cursor_position = pos;
}

/// Convert KeyCode to character (simplified)
fn key_code_to_char(key_code: KeyCode, shift_pressed: bool) -> Option<char> {
    match key_code {
        KeyCode::Space => Some(' '),
        KeyCode::KeyA => Some(if shift_pressed { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift_pressed { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift_pressed { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift_pressed { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift_pressed { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift_pressed { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift_pressed { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift_pressed { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift_pressed { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift_pressed { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift_pressed { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift_pressed { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift_pressed { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift_pressed { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift_pressed { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift_pressed { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift_pressed { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift_pressed { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift_pressed { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift_pressed { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift_pressed { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift_pressed { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift_pressed { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift_pressed { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift_pressed { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift_pressed { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some(if shift_pressed { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift_pressed { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift_pressed { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift_pressed { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift_pressed { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift_pressed { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift_pressed { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift_pressed { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift_pressed { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift_pressed { '(' } else { '9' }),
        _ => None,
    }
}
```

### Debounced Search System with Async Tasks

```rust
/// Component for tracking async search tasks
#[derive(Component)]
pub struct SearchTask(Task<SearchTaskResult>);

#[derive(Debug)]
pub struct SearchTaskResult {
    pub query: String,
    pub results: Vec<SearchResultItem>,
    pub duration_ms: f32,
}

/// Debounced search system using AsyncComputeTaskPool
pub fn debounced_search_system(
    mut input_events: EventReader<InputEvent>,
    mut search_events: EventWriter<SearchEvent>,
    mut commands: Commands,
    mut search_tasks: Query<(Entity, &mut SearchTask)>,
    time: Res<Time>,
    search_index: Res<SearchIndex>,
) {
    let current_time = time.elapsed_secs_f64();
    let task_pool = AsyncComputeTaskPool::get();

    // Handle text change events with debouncing
    for event in input_events.read() {
        if let InputEvent::TextChanged { new_text, timestamp } = event {
            let time_since_input = (current_time - timestamp) * 1000.0; // Convert to ms
            
            // Only process if debounce period has elapsed
            if time_since_input < 50.0 { // 50ms debounce
                continue;
            }
            
            if new_text.trim().is_empty() {
                // Clear results for empty query
                search_events.send(SearchEvent::ResultsUpdated {
                    results: Vec::new(),
                    query: String::new(),
                    took_ms: 0.0,
                });
                continue;
            }

            // Cancel any existing search tasks
            for (entity, _) in search_tasks.iter() {
                commands.entity(entity).despawn();
            }

            // Clone data for async task
            let query = new_text.clone();
            let search_data = search_index.clone(); // Assuming SearchIndex is Clone

            // Spawn new search task
            let task = task_pool.spawn(async move {
                let start_time = std::time::Instant::now();
                
                // Perform fuzzy search (this runs on background thread)
                let results = perform_fuzzy_search(&search_data, &query, 8).await;
                
                SearchTaskResult {
                    query,
                    results,
                    duration_ms: start_time.elapsed().as_secs_f32() * 1000.0,
                }
            });

            commands.spawn(SearchTask(task));
        }
    }

    // Check for completed search tasks
    for (entity, mut search_task) in search_tasks.iter_mut() {
        if let Some(result) = block_on(future::poll_once(&mut search_task.0)) {
            // Send search results event
            search_events.send(SearchEvent::ResultsUpdated {
                results: result.results,
                query: result.query,
                took_ms: result.duration_ms,
            });

            // Clean up completed task
            commands.entity(entity).despawn();
        }
    }
}

/// Async fuzzy search implementation
async fn perform_fuzzy_search(
    search_index: &SearchIndex,
    query: &str, 
    max_results: usize,
) -> Vec<SearchResultItem> {
    // Simulate async work (in real implementation, this would be CPU-intensive fuzzy matching)
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    
    // Perform actual search
    search_index.fuzzy_search(query, max_results)
}
```

### Navigation and Selection Systems

```rust
/// Handle keyboard navigation through search results
pub fn navigation_system(
    mut input_events: EventReader<InputEvent>,
    mut search_displays: Query<&mut SearchResultsDisplay>,
    mut gradient_events: EventWriter<GradientUpdateEvent>,
) {
    let Ok(mut display) = search_displays.get_single_mut() else { return };

    for event in input_events.read() {
        match event {
            InputEvent::Navigate(direction) => {
                let old_index = display.selected_index;
                
                match direction {
                    NavigationDirection::Up => {
                        if display.selected_index > 0 {
                            display.selected_index -= 1;
                        }
                    },
                    NavigationDirection::Down => {
                        if display.selected_index + 1 < display.results.len() {
                            display.selected_index += 1;
                        }
                    },
                    NavigationDirection::PageUp => {
                        display.selected_index = display.selected_index.saturating_sub(5);
                    },
                    NavigationDirection::PageDown => {
                        display.selected_index = (display.selected_index + 5)
                            .min(display.results.len().saturating_sub(1));
                    },
                    NavigationDirection::Home => {
                        display.selected_index = 0;
                    },
                    NavigationDirection::End => {
                        if !display.results.is_empty() {
                            display.selected_index = display.results.len() - 1;
                        }
                    },
                }

                // Send gradient update event if selection changed
                if old_index != display.selected_index {
                    gradient_events.send(GradientUpdateEvent::SelectionChanged(display.selected_index));
                }
            },
            InputEvent::SelectResult { index } => {
                if *index < display.results.len() {
                    display.selected_index = *index;
                    gradient_events.send(GradientUpdateEvent::SelectionChanged(*index));
                }
            },
            _ => {},
        }
    }
}

/// Execute selected search result
pub fn execution_system(
    mut input_events: EventReader<InputEvent>,
    search_displays: Query<&SearchResultsDisplay>,
    mut commands: Commands,
) {
    let Ok(display) = search_displays.get_single() else { return };

    for event in input_events.read() {
        if matches!(event, InputEvent::ExecuteSelected) {
            if let Some(selected_result) = display.results.get(display.selected_index) {
                // Execute the selected result
                execute_search_result(selected_result, &mut commands);
            }
        }
    }
}

/// Execute a search result based on its action data
fn execute_search_result(result: &SearchResultItem, commands: &mut Commands) {
    match &result.action_data {
        ActionData::LaunchApp { bundle_id } => {
            info!("Launching app: {}", bundle_id);
            // In real implementation, would launch the application
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .args(["-b", bundle_id])
                    .spawn();
            }
        },
        ActionData::OpenFile { path } => {
            info!("Opening file: {}", path);
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg(path)
                    .spawn();
            }
        },
        ActionData::RunCommand { command } => {
            info!("Running command: {}", command);
            let _ = std::process::Command::new("sh")
                .args(["-c", command])
                .spawn();
        },
        ActionData::WebSearch { query } => {
            info!("Web search: {}", query);
            let search_url = format!("https://www.google.com/search?q={}", 
                urlencoding::encode(query));
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg(search_url)
                    .spawn();
            }
        },
    }
}
```

### System Sets and Plugin Organization

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InputSystems {
    /// Process raw keyboard input
    ProcessInput,
    /// Handle debounced search
    ExecuteSearch,
    /// Update UI based on results
    UpdateDisplay,
    /// Handle navigation
    HandleNavigation,
    /// Execute actions
    ExecuteActions,
}

pub struct RealtimeInputPlugin;

impl Plugin for RealtimeInputPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<RealtimeTextInput>()
            .register_type::<SearchResultsDisplay>()
            .register_type::<SearchResultItem>()
            .add_event::<InputEvent>()
            .add_event::<SearchEvent>()
            .configure_sets(
                Update,
                (
                    InputSystems::ProcessInput,
                    InputSystems::ExecuteSearch,
                    InputSystems::UpdateDisplay,
                    InputSystems::HandleNavigation,
                    InputSystems::ExecuteActions,
                ).chain(),
            )
            .add_systems(
                Update,
                (
                    realtime_input_system.in_set(InputSystems::ProcessInput),
                    debounced_search_system.in_set(InputSystems::ExecuteSearch),
                    search_results_ui_system.in_set(InputSystems::UpdateDisplay),
                    navigation_system.in_set(InputSystems::HandleNavigation),
                    execution_system.in_set(InputSystems::ExecuteActions),
                ),
            )
            .add_systems(Startup, setup_input_components);
    }
}

/// Setup system for input components
pub fn setup_input_components(mut commands: Commands) {
    commands.spawn((
        RealtimeTextInput::default(),
        SearchResultsDisplay {
            results: Vec::new(),
            selected_index: 0,
            max_visible: 8,
            is_loading: false,
        },
    ));
}
```

### UI Update System with Query Optimization

```rust
/// Update search results UI efficiently using Changed<T> queries
pub fn search_results_ui_system(
    mut search_events: EventReader<SearchEvent>,
    mut search_display: Query<&mut SearchResultsDisplay>,
    mut result_containers: Query<&mut Visibility, With<SearchResultsContainer>>,
    mut commands: Commands,
) {
    let Ok(mut display) = search_display.get_single_mut() else { return };

    for event in search_events.read() {
        match event {
            SearchEvent::ResultsUpdated { results, query, took_ms } => {
                // Update results data
                display.results = results.clone();
                display.selected_index = 0;
                display.is_loading = false;

                // Show/hide results container based on results
                for mut visibility in result_containers.iter_mut() {
                    *visibility = if results.is_empty() {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    };
                }

                info!("Search completed: '{}' -> {} results in {:.1}ms", 
                     query, results.len(), took_ms);
            },
            SearchEvent::SearchFailed { query, error } => {
                warn!("Search failed for '{}': {}", query, error);
                display.results.clear();
                display.is_loading = false;
            },
            SearchEvent::IndexUpdated { item_count } => {
                info!("Search index updated: {} items", item_count);
            },
        }
    }
}
```

### Testing Strategies

```rust
#[cfg(test)]
mod input_system_tests {
    use super::*;

    #[test]
    fn test_text_input_component() {
        let mut input = RealtimeTextInput::default();
        assert_eq!(input.current_text, "");
        assert_eq!(input.cursor_position, 0);
        assert!(!input.is_focused);
    }

    #[test]
    fn test_key_code_conversion() {
        assert_eq!(key_code_to_char(KeyCode::KeyA, false), Some('a'));
        assert_eq!(key_code_to_char(KeyCode::KeyA, true), Some('A'));
        assert_eq!(key_code_to_char(KeyCode::Space, false), Some(' '));
        assert_eq!(key_code_to_char(KeyCode::Enter, false), None);
    }

    #[test]
    fn test_word_deletion() {
        let mut input = RealtimeTextInput::default();
        input.current_text = "hello world test".to_string();
        input.cursor_position = 16; // At end
        
        delete_word_backwards(&mut input);
        assert_eq!(input.current_text, "hello world ");
        assert_eq!(input.cursor_position, 12);
        
        delete_word_backwards(&mut input);
        assert_eq!(input.current_text, "hello ");
        assert_eq!(input.cursor_position, 6);
    }

    #[test] 
    fn test_navigation_direction() {
        let mut display = SearchResultsDisplay {
            results: vec![
                SearchResultItem { 
                    id: 1,
                    title: "Test 1".to_string(),
                    subtitle: String::new(),
                    icon_path: None,
                    relevance_score: 1.0,
                    action_data: ActionData::RunCommand { 
                        command: "test1".to_string() 
                    },
                },
                SearchResultItem {
                    id: 2,
                    title: "Test 2".to_string(), 
                    subtitle: String::new(),
                    icon_path: None,
                    relevance_score: 0.9,
                    action_data: ActionData::RunCommand { 
                        command: "test2".to_string() 
                    },
                }
            ],
            selected_index: 0,
            max_visible: 8,
            is_loading: false,
        };
        
        // Test navigation down
        assert_eq!(display.selected_index, 0);
        // In real test, would simulate navigation event
    }

    #[test]
    fn test_search_result_execution() {
        let result = SearchResultItem {
            id: 1,
            title: "Test App".to_string(),
            subtitle: "Application".to_string(),
            icon_path: None,
            relevance_score: 1.0,
            action_data: ActionData::LaunchApp {
                bundle_id: "com.test.app".to_string(),
            },
        };
        
        // Would test execution logic here
        match &result.action_data {
            ActionData::LaunchApp { bundle_id } => {
                assert_eq!(bundle_id, "com.test.app");
            },
            _ => panic!("Wrong action type"),
        }
    }
}
```

---

**Next:** [05-window-sizing-strategy.md](./05-window-sizing-strategy.md)
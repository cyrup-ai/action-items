# Main Menu - Search and Filtering System Implementation

## Task: Implement Real-time Search with Fuzzy Matching and Multi-source Integration

### File: `ui/src/systems/search_engine.rs` (new file)

Create comprehensive search system with real-time filtering, fuzzy matching, and intelligent ranking.

### Implementation Requirements

#### Real-time Search System
- File: `ui/src/systems/search_engine.rs` (new file, line 1-156)
- Implement `search_system` with < 50ms response time for query results
- Bevy Example Reference: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Efficient query patterns for action items
- Real-time filtering using Bevy's change detection for search input
- Integration with existing fuzzy search sensitivity from Advanced Menu

#### Fuzzy Matching Implementation
- File: `ui/src/search/fuzzy_matcher.rs` (new file, line 1-89)
- Advanced fuzzy matching with configurable sensitivity levels
- Integration with existing `core/src/search/` system for consistent behavior
- Support for partial and approximate string matching across all action sources
- Intelligent ranking based on match quality and usage patterns

#### Multi-Source Search Integration
```rust
pub fn unified_search_system(
    mut search_state: ResMut<SearchState>,
    action_query: Query<&ActionItem>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut search_events: EventWriter<SearchEvent>,
) {
    // Implementation for multi-source search
}
```

#### Search History and Suggestions
- File: `ui/src/systems/search_history.rs` (new file, line 1-67)
- Recent search term storage and intelligent suggestions
- Frequency-based ranking for commonly searched items
- Integration with AI system for contextual search suggestions
- Privacy-conscious history management with user control

#### Empty State Management
- File: `ui/src/systems/empty_state.rs` (new file, line 1-45)
- Show all favorites when search is empty
- Smooth transitions between search results and favorites
- Loading states during search index building
- Graceful handling of search system failures

### Architecture Notes
- Event-driven search with SearchEvent for decoupled processing
- Incremental search updates using Bevy's change detection
- Zero-allocation search processing where possible
- Integration with existing plugin discovery system
- Configurable search sensitivity from user preferences

### Integration Points
- `core/src/search/systems.rs` - Integration with existing search (lines 23-67)
- `ui/src/launcher/favorites.rs` - Favorites display integration
- `app/src/preferences/` - Search sensitivity preference integration
- Multi-source plugin integration for unified search results

### Search Performance Optimization
- Indexed search with pre-built search trees
- Debounced input processing to reduce computation
- Background search index updates for new actions
- Memory-efficient result caching with LRU eviction

### Event System Integration
```rust
#[derive(Event)]
pub enum SearchEvent {
    QueryChanged(String),
    ResultsUpdated(Vec<ActionItem>),
    SearchCleared,
    SuggestionSelected(String),
    HistoryRequested,
}
```

### Bevy Example References
- **System Parameters**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Efficient querying (lines 15-45)
- **Event Handling**: [`ecs/event.rs`](../../../docs/bevy/examples/ecs/event.rs) - Search event patterns
- **Input Processing**: [`input/keyboard_input.rs`](../../../docs/bevy/examples/input/keyboard_input.rs) - Real-time input handling
- **Resource Updates**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - SearchState updates

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.
# Actions_Items_Config_Menu Task 4: Universal Search System

## Task Overview
Implement comprehensive universal extension and command search system with fuzzy matching, multi-field search, real-time filtering, and intelligent ranking for the Actions Items Config interface.

## Implementation Requirements

### Core Components
```rust
// Universal search system
#[derive(Resource, Reflect, Debug)]
pub struct UniversalSearchResource {
    pub search_index: SearchIndex,
    pub search_state: SearchState,
    pub search_results: SearchResults,
    pub search_filters: SearchFilters,
    pub ranking_algorithm: RankingAlgorithm,
}

#[derive(Reflect, Debug)]
pub struct SearchIndex {
    pub extension_index: HashMap<ExtensionId, ExtensionSearchData>,
    pub command_index: HashMap<CommandId, CommandSearchData>,
    pub keyword_index: HashMap<String, Vec<SearchableItem>>,
    pub fuzzy_matcher: FuzzyMatcher,
}

#[derive(Reflect, Debug)]
pub struct ExtensionSearchData {
    pub extension_id: ExtensionId,
    pub name: String,
    pub description: String,
    pub author: String,
    pub keywords: Vec<String>,
    pub command_names: Vec<String>,
    pub search_weight: f32,
}

#[derive(Reflect, Debug)]
pub struct CommandSearchData {
    pub command_id: CommandId,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub aliases: Vec<String>,
    pub category: Option<String>,
    pub extension_name: String,
    pub usage_frequency: u32,
    pub search_weight: f32,
}

#[derive(Component, Reflect, Debug)]
pub struct SearchInterfaceComponent {
    pub search_input_entity: Entity,
    pub results_container_entity: Entity,
    pub filter_panel_entity: Option<Entity>,
    pub current_query: String,
    pub results_count: u32,
}
```

### Search Algorithm System
```rust
// Advanced search and ranking system
#[derive(Reflect, Debug)]
pub struct FuzzyMatcher {
    pub match_threshold: f32,
    pub word_boundary_bonus: f32,
    pub camel_case_bonus: f32,
    pub consecutive_bonus: f32,
    pub leading_letter_penalty: f32,
    pub max_leading_letter_penalty: f32,
}

#[derive(Reflect, Debug)]
pub enum SearchableItem {
    Extension(ExtensionId),
    Command(CommandId),
    Category(String),
}

#[derive(Reflect, Debug)]
pub struct SearchResult {
    pub item: SearchableItem,
    pub score: f32,
    pub match_ranges: Vec<MatchRange>,
    pub ranking_factors: RankingFactors,
}

#[derive(Reflect, Debug)]
pub struct MatchRange {
    pub start: u32,
    pub end: u32,
    pub field: SearchField,
}

#[derive(Reflect, Debug)]
pub enum SearchField {
    Name,
    Description,
    Keywords,
    Aliases,
    Category,
    Author,
}

pub fn universal_search_system(
    mut search_res: ResMut<UniversalSearchResource>,
    search_interface_query: Query<&SearchInterfaceComponent, Changed<SearchInterfaceComponent>>,
    extension_res: Res<ExtensionManagementResource>,
) {
    for search_interface in &search_interface_query {
        if !search_interface.current_query.is_empty() {
            let results = perform_search(
                &search_interface.current_query,
                &search_res.search_index,
                &search_res.search_filters,
            );
            
            search_res.search_results = rank_and_sort_results(
                results,
                &search_res.ranking_algorithm,
            );
        } else {
            search_res.search_results.clear();
        }
    }
}
```

### Real-time Filtering System
```rust
// Advanced filtering and categorization
#[derive(Reflect, Debug)]
pub struct SearchFilters {
    pub extension_filters: Vec<ExtensionFilter>,
    pub command_filters: Vec<CommandFilter>,
    pub category_filters: Vec<String>,
    pub status_filters: Vec<ExtensionStatus>,
    pub date_filters: DateRangeFilter,
}

#[derive(Reflect, Debug)]
pub enum ExtensionFilter {
    ByAuthor(String),
    ByVersion(VersionRange),
    ByInstallDate(DateRange),
    ByCommandCount(RangeFilter<u32>),
    ByStatus(ExtensionStatus),
}

#[derive(Reflect, Debug)]
pub enum CommandFilter {
    ByCategory(String),
    ByHotkey(bool),
    ByAlias(bool),
    ByUsageFrequency(RangeFilter<u32>),
    ByLastUsed(DateRange),
}

pub fn search_filtering_system(
    mut search_res: ResMut<UniversalSearchResource>,
    filter_events: EventReader<SearchFilterEvent>,
) {
    for filter_event in filter_events.read() {
        match filter_event {
            SearchFilterEvent::AddFilter(filter) => {
                add_search_filter(&mut search_res.search_filters, filter);
            }
            SearchFilterEvent::RemoveFilter(filter_id) => {
                remove_search_filter(&mut search_res.search_filters, *filter_id);
            }
            SearchFilterEvent::ClearFilters => {
                clear_all_filters(&mut search_res.search_filters);
            }
        }
        
        // Re-apply search with new filters
        reapply_search_with_filters(&mut search_res);
    }
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `input/text_input.rs` - Real-time search input handling
- `ui/text.rs` - Search result highlighting
- `async_compute/async_compute.rs` - Async search operations

### Implementation Pattern
```rust
// Based on text_input.rs for search input
fn search_input_system(
    mut char_events: EventReader<ReceivedCharacter>,
    mut key_events: EventReader<KeyboardInput>,
    mut search_query: Query<&mut Text, With<SearchInputComponent>>,
    mut search_events: EventWriter<SearchEvent>,
) {
    for mut text in &mut search_query {
        for char_event in char_events.read() {
            if !char_event.char.is_control() {
                text.sections[0].value.push(char_event.char);
                search_events.send(SearchEvent::QueryChanged {
                    query: text.sections[0].value.clone(),
                });
            }
        }
    }
}

// Based on ui/text.rs for result highlighting
fn search_highlight_system(
    mut text_query: Query<&mut Text, With<SearchResultComponent>>,
    search_res: Res<UniversalSearchResource>,
) {
    if search_res.is_changed() {
        for (mut text, result_data) in text_query.iter_mut().zip(&search_res.search_results.items) {
            apply_search_highlighting(&mut text, &result_data.match_ranges);
        }
    }
}
```

## Search Optimization
- Incremental search index building
- Debounced search execution for performance
- Cached search results for repeated queries  
- Background index updates for large extension sets

## Performance Constraints
- **ZERO ALLOCATIONS** during search execution
- Efficient fuzzy matching algorithms
- Optimized index structure for fast lookups
- Minimal UI blocking during search operations

## Success Criteria
- Complete universal search implementation
- Fast and accurate fuzzy matching
- No unwrap()/expect() calls in production code
- Zero-allocation search operations
- Comprehensive filtering and ranking system

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for fuzzy matching accuracy
- Integration tests for search and filtering
- Performance tests for large dataset searches
- User experience tests for search responsiveness
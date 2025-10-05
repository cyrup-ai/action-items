# Task 10: Search Sensitivity and Filtering System

## Implementation Details

**File**: `ui/src/ui/search_sensitivity.rs`  
**Lines**: 300-385  
**Architecture**: Configurable search sensitivity with real-time filtering  
**Integration**: SearchSystem, SettingsSystem, FilterManager  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct SearchSensitivitySettings {
    pub case_sensitivity: CaseSensitivity,
    pub accent_sensitivity: bool,
    pub fuzzy_matching: FuzzyMatchingLevel,
    pub word_boundaries: bool,
    pub substring_matching: bool,
    pub minimum_query_length: usize,
    pub debounce_delay: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaseSensitivity {
    Insensitive,
    Sensitive,
    Smart, // Sensitive if query contains uppercase
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FuzzyMatchingLevel {
    Disabled,
    Low,    // Requires 90% similarity
    Medium, // Requires 75% similarity  
    High,   // Requires 60% similarity
}

#[derive(Component, Clone, Debug)]
pub struct SearchFilter {
    pub query: String,
    pub normalized_query: String,
    pub active_filters: Vec<FilterType>,
    pub score_threshold: f32,
    pub last_updated: Instant,
}

pub fn search_sensitivity_system(
    mut search_filters: Query<&mut SearchFilter>,
    sensitivity_settings: Res<SearchSensitivitySettings>,
    mut search_events: EventReader<SearchQueryEvent>,
    mut result_events: EventWriter<SearchResultEvent>,
    search_index: Res<SearchIndex>,
    time: Res<Time>,
) {
    // Handle search query updates with debouncing
    for event in search_events.read() {
        for mut filter in search_filters.iter_mut() {
            // Apply debounce delay
            if filter.last_updated.elapsed().as_secs_f32() < sensitivity_settings.debounce_delay {
                continue;
            }

            filter.query = event.query.clone();
            filter.normalized_query = normalize_search_query(&event.query, &sensitivity_settings);
            filter.last_updated = Instant::now();

            // Skip search if query is too short
            if filter.query.len() < sensitivity_settings.minimum_query_length {
                result_events.send(SearchResultEvent::ClearResults);
                continue;
            }

            // Perform search with configured sensitivity
            let results = perform_sensitive_search(&filter, &search_index, &sensitivity_settings);
            result_events.send(SearchResultEvent::UpdateResults(results));
        }
    }
}

fn normalize_search_query(query: &str, settings: &SearchSensitivitySettings) -> String {
    let mut normalized = query.to_string();

    // Handle case sensitivity
    match settings.case_sensitivity {
        CaseSensitivity::Insensitive => {
            normalized = normalized.to_lowercase();
        }
        CaseSensitivity::Smart => {
            if !query.chars().any(|c| c.is_uppercase()) {
                normalized = normalized.to_lowercase();
            }
        }
        CaseSensitivity::Sensitive => {
            // Keep original case
        }
    }

    // Handle accent sensitivity
    if !settings.accent_sensitivity {
        normalized = remove_accents(&normalized);
    }

    normalized
}

fn perform_sensitive_search(
    filter: &SearchFilter,
    index: &SearchIndex,
    settings: &SearchSensitivitySettings,
) -> Vec<SearchResult> {
    let mut results = Vec::new();

    for item in &index.items {
        let score = calculate_match_score(
            &filter.normalized_query,
            &normalize_text(&item.text, settings),
            settings,
        );

        if score >= filter.score_threshold {
            results.push(SearchResult {
                item: item.clone(),
                score,
                match_positions: find_match_positions(&filter.normalized_query, &item.text),
            });
        }
    }

    // Sort by score descending
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    results
}
```

### Fuzzy Matching Implementation

**Reference**: `./docs/bevy/examples/string_matching.rs:145-178`

```rust
fn calculate_match_score(query: &str, text: &str, settings: &SearchSensitivitySettings) -> f32 {
    match settings.fuzzy_matching {
        FuzzyMatchingLevel::Disabled => {
            if settings.substring_matching {
                if text.contains(query) { 1.0 } else { 0.0 }
            } else {
                if settings.word_boundaries {
                    calculate_word_boundary_score(query, text)
                } else {
                    if text == query { 1.0 } else { 0.0 }
                }
            }
        }
        level => {
            let similarity = calculate_fuzzy_similarity(query, text);
            let threshold = match level {
                FuzzyMatchingLevel::High => 0.6,
                FuzzyMatchingLevel::Medium => 0.75,
                FuzzyMatchingLevel::Low => 0.9,
                FuzzyMatchingLevel::Disabled => unreachable!(),
            };
            
            if similarity >= threshold { similarity } else { 0.0 }
        }
    }
}

fn calculate_fuzzy_similarity(query: &str, text: &str) -> f32 {
    let query_chars: Vec<char> = query.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    
    if query_chars.is_empty() || text_chars.is_empty() {
        return 0.0;
    }

    // Levenshtein distance algorithm
    let mut dp = vec![vec![0; text_chars.len() + 1]; query_chars.len() + 1];
    
    for i in 0..=query_chars.len() {
        dp[i][0] = i;
    }
    for j in 0..=text_chars.len() {
        dp[0][j] = j;
    }

    for i in 1..=query_chars.len() {
        for j in 1..=text_chars.len() {
            let cost = if query_chars[i - 1] == text_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = std::cmp::min(
                std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                dp[i - 1][j - 1] + cost,
            );
        }
    }

    let max_len = std::cmp::max(query_chars.len(), text_chars.len());
    1.0 - (dp[query_chars.len()][text_chars.len()] as f32) / (max_len as f32)
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui_slider.rs:188-225`

```rust
// Search sensitivity settings section
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(16.0)),
        row_gap: Val::Px(12.0),
        ..default()
    },
    background_color: Color::rgba(0.08, 0.08, 0.08, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(8.0)),
    ..default()
},
children: &[
    // Case sensitivity dropdown
    (SettingsRowBundle {
        label: "Case Sensitivity".to_string(),
        control: ControlType::Dropdown {
            options: vec!["Insensitive", "Smart", "Sensitive"],
            selected: match sensitivity_settings.case_sensitivity {
                CaseSensitivity::Insensitive => 0,
                CaseSensitivity::Smart => 1,
                CaseSensitivity::Sensitive => 2,
            },
        },
        ..default()
    },),
    
    // Fuzzy matching level slider
    (SettingsRowBundle {
        label: "Fuzzy Matching".to_string(),
        control: ControlType::Slider {
            value: match sensitivity_settings.fuzzy_matching {
                FuzzyMatchingLevel::Disabled => 0.0,
                FuzzyMatchingLevel::High => 0.25,
                FuzzyMatchingLevel::Medium => 0.5,
                FuzzyMatchingLevel::Low => 0.75,
            },
            min: 0.0,
            max: 1.0,
            step: 0.25,
        },
        ..default()
    },),
    
    // Debounce delay slider
    (SettingsRowBundle {
        label: "Search Delay (ms)".to_string(),
        control: ControlType::Slider {
            value: sensitivity_settings.debounce_delay * 1000.0,
            min: 50.0,
            max: 1000.0,
            step: 50.0,
        },
        tooltip: Some("Delay before search starts while typing".to_string()),
        ..default()
    },),
]
```

### Architecture Notes

- Configurable search sensitivity with multiple matching algorithms
- Real-time debouncing prevents excessive search operations
- Fuzzy matching with adjustable similarity thresholds
- Accent-insensitive search for international text
- Word boundary detection for precise matching
- Score-based result ranking with customizable thresholds
- Efficient search indexing for large datasets

**Bevy Examples**: `./docs/bevy/examples/text_search.rs:125-158`, `./docs/bevy/examples/ui_slider.rs:88-115`  
**Integration Points**: SearchSystem, IndexingSystem, SettingsSystem  
**Dependencies**: SearchIndex, SettingsResource, EventSystems
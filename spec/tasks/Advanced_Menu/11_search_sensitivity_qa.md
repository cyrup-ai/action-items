# Task 11: QA Validation - Search Sensitivity and Filtering System

## Comprehensive Testing Protocol

**File**: `tests/ui/search_sensitivity_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: SearchSystem, FilteringSystem, SettingsSystem  

### Test Categories

#### 1. Case Sensitivity Testing
**Reference**: `./docs/bevy/examples/text_search.rs:185-212`
```rust
#[test]
fn test_case_sensitivity_modes() {
    let test_cases = vec![
        // (query, text, case_mode, should_match)
        ("hello", "Hello World", CaseSensitivity::Insensitive, true),
        ("hello", "Hello World", CaseSensitivity::Sensitive, false),
        ("Hello", "Hello World", CaseSensitivity::Sensitive, true),
        ("hello", "Hello World", CaseSensitivity::Smart, true), // lowercase query
        ("Hello", "hello world", CaseSensitivity::Smart, false), // uppercase query
    ];

    for (query, text, case_mode, should_match) in test_cases {
        let settings = SearchSensitivitySettings {
            case_sensitivity: case_mode,
            fuzzy_matching: FuzzyMatchingLevel::Disabled,
            substring_matching: true,
            ..default()
        };
        
        let score = calculate_match_score(query, text, &settings);
        
        if should_match {
            assert!(score > 0.0, "Query '{}' should match '{}' with {:?}", query, text, case_mode);
        } else {
            assert_eq!(score, 0.0, "Query '{}' should not match '{}' with {:?}", query, text, case_mode);
        }
    }
}
```

#### 2. Fuzzy Matching Accuracy Testing
```rust
#[test]
fn test_fuzzy_matching_levels() {
    let test_cases = vec![
        // (query, text, expected_similarity)
        ("hello", "helo", 0.8),      // Missing letter
        ("hello", "heloo", 0.8),     // Extra letter
        ("hello", "helol", 0.8),     // Transposed letters
        ("hello", "world", 0.0),     // Completely different
        ("hello", "hello", 1.0),     // Exact match
    ];

    for (query, text, expected_similarity) in test_cases {
        let calculated_similarity = calculate_fuzzy_similarity(query, text);
        
        assert!(
            (calculated_similarity - expected_similarity).abs() < 0.1,
            "Fuzzy similarity for '{}' vs '{}' was {}, expected {}",
            query, text, calculated_similarity, expected_similarity
        );
    }
}
```

#### 3. Accent Sensitivity Testing
**Reference**: `./docs/bevy/examples/unicode_search.rs:95-122`
```rust
#[test]
fn test_accent_sensitivity() {
    let test_cases = vec![
        // (query, text, accent_sensitive, should_match)
        ("cafe", "café", false, true),
        ("cafe", "café", true, false),
        ("naïve", "naive", false, true),
        ("naïve", "naive", true, false),
        ("résumé", "resume", false, true),
        ("résumé", "resume", true, false),
    ];

    for (query, text, accent_sensitive, should_match) in test_cases {
        let settings = SearchSensitivitySettings {
            accent_sensitivity: accent_sensitive,
            case_sensitivity: CaseSensitivity::Insensitive,
            substring_matching: true,
            fuzzy_matching: FuzzyMatchingLevel::Disabled,
            ..default()
        };
        
        let normalized_query = normalize_search_query(query, &settings);
        let normalized_text = normalize_text(text, &settings);
        let matches = normalized_text.contains(&normalized_query);
        
        assert_eq!(matches, should_match, 
            "Query '{}' vs '{}' with accent_sensitive={} should match={}",
            query, text, accent_sensitive, should_match);
    }
}
```

#### 4. Word Boundary Detection Testing
```rust
#[test]
fn test_word_boundary_matching() {
    let test_cases = vec![
        // (query, text, word_boundaries, should_match, expected_score)
        ("cat", "cat", true, true, 1.0),
        ("cat", "catch", true, false, 0.0),
        ("cat", "wildcat", true, false, 0.0),
        ("cat", "cat dog", true, true, 1.0),
        ("cat", "catch", false, true, 0.6), // Partial match without boundaries
    ];

    for (query, text, word_boundaries, should_match, expected_score) in test_cases {
        let settings = SearchSensitivitySettings {
            word_boundaries,
            substring_matching: !word_boundaries,
            case_sensitivity: CaseSensitivity::Insensitive,
            fuzzy_matching: FuzzyMatchingLevel::Disabled,
            ..default()
        };
        
        let score = calculate_match_score(query, text, &settings);
        
        if should_match {
            assert!(score >= expected_score * 0.9, 
                "Word boundary test failed: '{}' in '{}' scored {}, expected >= {}",
                query, text, score, expected_score);
        } else {
            assert_eq!(score, 0.0, 
                "Word boundary test failed: '{}' in '{}' should not match",
                query, text);
        }
    }
}
```

#### 5. Debouncing and Performance Testing
**Reference**: `./docs/bevy/examples/timers.rs:225-252`
```rust
#[test]
fn test_search_debouncing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, search_sensitivity_system)
       .add_event::<SearchQueryEvent>()
       .add_event::<SearchResultEvent>();

    let settings = SearchSensitivitySettings {
        debounce_delay: 0.1, // 100ms debounce
        minimum_query_length: 2,
        ..default()
    };
    
    app.world_mut().insert_resource(settings);
    
    let search_filter = SearchFilter {
        query: String::new(),
        normalized_query: String::new(),
        active_filters: Vec::new(),
        score_threshold: 0.1,
        last_updated: Instant::now() - Duration::from_millis(200), // Old timestamp
    };
    
    app.world_mut().spawn(search_filter);
    
    // Send rapid search queries
    let mut search_events = app.world_mut().resource_mut::<Events<SearchQueryEvent>>();
    search_events.send(SearchQueryEvent { query: "te".to_string() });
    search_events.send(SearchQueryEvent { query: "tes".to_string() });
    search_events.send(SearchQueryEvent { query: "test".to_string() });
    
    app.update();
    
    // Should only process the last query due to debouncing
    let result_events = app.world().resource::<Events<SearchResultEvent>>();
    assert!(result_events.len() <= 1, "Debouncing failed, too many search operations");
}
```

#### 6. Performance Stress Testing
```rust
#[test]
fn test_search_performance_stress() {
    let settings = SearchSensitivitySettings {
        fuzzy_matching: FuzzyMatchingLevel::Medium,
        case_sensitivity: CaseSensitivity::Insensitive,
        substring_matching: true,
        ..default()
    };
    
    // Create large search index
    let large_index = SearchIndex {
        items: (0..10000).map(|i| SearchItem {
            id: i,
            text: format!("Item number {} with some descriptive text", i),
            category: "test".to_string(),
        }).collect(),
    };
    
    let filter = SearchFilter {
        query: "item 500".to_string(),
        normalized_query: "item 500".to_string(),
        active_filters: Vec::new(),
        score_threshold: 0.1,
        last_updated: Instant::now(),
    };
    
    let start_time = Instant::now();
    let results = perform_sensitive_search(&filter, &large_index, &settings);
    let search_duration = start_time.elapsed();
    
    assert!(search_duration.as_millis() < 100, 
        "Search took too long: {}ms", search_duration.as_millis());
    assert!(!results.is_empty(), "Should find results in large dataset");
    assert!(results.len() <= 100, "Should limit result count for performance");
}
```

### Edge Case Testing

#### 7. Unicode and Emoji Testing
**Reference**: `./docs/bevy/examples/unicode_search.rs:155-182`
```rust
#[test]
fn test_unicode_emoji_search() {
    let test_cases = vec![
        ("🔥", "fire 🔥 emoji", true),
        ("fire", "🔥 flame", false),
        ("café", "café latte", true),
        ("北京", "北京大学", true),
        ("ひらがな", "ひらがなとカタカナ", true),
    ];

    let settings = SearchSensitivitySettings {
        case_sensitivity: CaseSensitivity::Insensitive,
        accent_sensitivity: false,
        substring_matching: true,
        fuzzy_matching: FuzzyMatchingLevel::Disabled,
        ..default()
    };

    for (query, text, should_match) in test_cases {
        let score = calculate_match_score(query, text, &settings);
        
        if should_match {
            assert!(score > 0.0, "Unicode search failed: '{}' should match '{}'", query, text);
        } else {
            assert_eq!(score, 0.0, "Unicode search failed: '{}' should not match '{}'", query, text);
        }
    }
}
```

#### 8. Settings Persistence Testing
```rust
#[test]
fn test_search_settings_persistence() {
    let original_settings = SearchSensitivitySettings {
        case_sensitivity: CaseSensitivity::Smart,
        accent_sensitivity: false,
        fuzzy_matching: FuzzyMatchingLevel::Medium,
        word_boundaries: true,
        substring_matching: false,
        minimum_query_length: 3,
        debounce_delay: 0.2,
    };
    
    // Test serialization/deserialization
    let serialized = serde_json::to_string(&original_settings).unwrap();
    let deserialized: SearchSensitivitySettings = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(original_settings.case_sensitivity, deserialized.case_sensitivity);
    assert_eq!(original_settings.fuzzy_matching, deserialized.fuzzy_matching);
    assert_eq!(original_settings.debounce_delay, deserialized.debounce_delay);
}
```

### Manual Testing Checklist

- [ ] Case sensitivity toggles work correctly in real-time
- [ ] Fuzzy matching finds relevant results at all levels
- [ ] Accent insensitive search works with international text
- [ ] Word boundary detection prevents false positives
- [ ] Search debouncing prevents lag during rapid typing
- [ ] Large datasets search quickly (< 100ms for 10K items)
- [ ] Unicode and emoji characters search correctly
- [ ] Settings persist across application restarts
- [ ] Minimum query length prevents excessive searches
- [ ] Search results are ranked by relevance score

**Bevy Examples**: `./docs/bevy/examples/unicode_search.rs:205-232`, `./docs/bevy/examples/text_search.rs:278-305`  
**Integration Points**: All search sensitivity components  
**Success Criteria**: All tests pass, search responds < 100ms, zero false positives with word boundaries
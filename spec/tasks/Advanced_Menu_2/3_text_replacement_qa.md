# Task 3: QA Validation - Advanced Text Replacement System

## Comprehensive Testing Protocol

**File**: `tests/ui/text_replacement_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: TextReplacementSystem, PatternMatcher, SettingsSystem  

### Test Categories

#### 1. Modifier Notation Replacement Testing
**Reference**: `./docs/bevy/examples/ui/text_input.rs:385-412`
```rust
#[test]
fn test_modifier_notation_replacement() {
    let mut replacement_manager = TextReplacementManager::default();
    
    // Add modifier notation rule
    let modifier_rule = ReplacementRule {
        id: "modifier_rule".to_string(),
        pattern: "^⌃⇧⌘".to_string(),
        replacement: "⌃".to_string(),
        rule_type: ReplacementType::ModifierNotation,
        enabled: true,
        case_sensitive: false,
        word_boundaries: false,
        created_at: chrono::Utc::now(),
    };
    
    replacement_manager.active_rules.insert(modifier_rule.id.clone(), modifier_rule);
    
    let test_cases = vec![
        // (input, expected_output)
        ("Press ^⌃⇧⌘K for search", "Press ⌃K for search"),
        ("Use ^⌃⇧⌘+Space to open", "Use ⌃+Space to open"),
        ("The ^⌃⇧⌘ combination", "The ⌃ combination"),
        ("No replacement here", "No replacement here"), // Should remain unchanged
    ];
    
    for (input, expected) in test_cases {
        let result = process_text_replacement(input, &mut replacement_manager);
        assert_eq!(result, expected, 
            "Modifier notation replacement failed for input: '{}'", input);
    }
}
```

#### 2. Pattern Matching Accuracy Testing
```rust
#[test]
fn test_pattern_matching_accuracy() {
    let mut replacement_manager = TextReplacementManager::default();
    
    // Test word boundary enforcement
    let word_boundary_rule = ReplacementRule {
        id: "word_boundary_test".to_string(),
        pattern: "cat".to_string(),
        replacement: "dog".to_string(),
        rule_type: ReplacementType::SimpleText,
        enabled: true,
        case_sensitive: false,
        word_boundaries: true,
        created_at: chrono::Utc::now(),
    };
    
    replacement_manager.active_rules.insert(word_boundary_rule.id.clone(), word_boundary_rule);
    
    let test_cases = vec![
        // (input, expected_output, description)
        ("I have a cat", "I have a dog", "Whole word replacement"),
        ("The cat runs", "The dog runs", "Word at start of sentence"),
        ("cats are nice", "cats are nice", "Partial word should not match"),
        ("catch the ball", "catch the ball", "Substring should not match"),
        ("CAT in caps", "DOG in caps", "Case insensitive matching"),
    ];
    
    for (input, expected, description) in test_cases {
        let result = process_text_replacement(input, &mut replacement_manager);
        assert_eq!(result, expected, "{}: input '{}' should become '{}'", 
                   description, input, expected);
    }
}
```

#### 3. Unicode Symbol Replacement Testing
**Reference**: `./docs/bevy/examples/unicode_handling.rs:125-152`
```rust
#[test]
fn test_unicode_symbol_replacement() {
    let mut replacement_manager = TextReplacementManager::default();
    
    let unicode_mappings = vec![
        ("->", "→", "Right arrow"),
        ("<-", "←", "Left arrow"),
        ("<=", "≤", "Less than or equal"),
        (">=", "≥", "Greater than or equal"),
        ("!=", "≠", "Not equal"),
        ("+-", "±", "Plus minus"),
        ("...", "…", "Ellipsis"),
        ("(c)", "©", "Copyright"),
        ("(r)", "®", "Registered"),
        ("(tm)", "™", "Trademark"),
    ];
    
    for (pattern, replacement, description) in unicode_mappings {
        let rule = ReplacementRule {
            id: format!("unicode_{}", pattern.replace(['(', ')', '<', '>', '.'], "")),
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
            rule_type: ReplacementType::UnicodeSymbol,
            enabled: true,
            case_sensitive: false,
            word_boundaries: false,
            created_at: chrono::Utc::now(),
        };
        
        replacement_manager.active_rules.insert(rule.id.clone(), rule);
    }
    
    let test_input = "Arrow -> points right, <- points left. Check if 5 >= 3 and 2 <= 4. Also x != y and a +- b. Use ... for ellipsis. Copyright (c) and trademark (tm) symbols.";
    let expected = "Arrow → points right, ← points left. Check if 5 ≥ 3 and 2 ≤ 4. Also x ≠ y and a ± b. Use … for ellipsis. Copyright © and trademark ™ symbols.";
    
    let result = process_text_replacement(test_input, &mut replacement_manager);
    assert_eq!(result, expected, "Unicode symbol replacement failed");
}
```

#### 4. Regular Expression Replacement Testing
```rust
#[test]
fn test_regex_replacement() {
    let mut replacement_manager = TextReplacementManager::default();
    
    let regex_rule = ReplacementRule {
        id: "email_obfuscation".to_string(),
        pattern: r"\b[\w\.-]+@[\w\.-]+\.\w+\b".to_string(), // Email pattern
        replacement: "[email protected]".to_string(),
        rule_type: ReplacementType::RegularExpression,
        enabled: true,
        case_sensitive: false,
        word_boundaries: false,
        created_at: chrono::Utc::now(),
    };
    
    replacement_manager.active_rules.insert(regex_rule.id.clone(), regex_rule);
    
    let test_cases = vec![
        ("Contact us at support@example.com for help", "Contact us at [email protected] for help"),
        ("Email: user123@domain.co.uk", "Email: [email protected]"),
        ("No email in this text", "No email in this text"),
        ("user@site.com and admin@test.org", "[email protected] and [email protected]"),
    ];
    
    for (input, expected) in test_cases {
        let result = process_text_replacement(input, &mut replacement_manager);
        assert_eq!(result, expected, "Regex replacement failed for: '{}'", input);
    }
}
```

#### 5. Replacement History Tracking Testing
**Reference**: `./docs/bevy/examples/collections.rs:185-212`
```rust
#[test]
fn test_replacement_history_tracking() {
    let mut replacement_manager = TextReplacementManager::default();
    
    let simple_rule = ReplacementRule {
        id: "hello_rule".to_string(),
        pattern: "hello".to_string(),
        replacement: "hi".to_string(),
        rule_type: ReplacementType::SimpleText,
        enabled: true,
        case_sensitive: false,
        word_boundaries: true,
        created_at: chrono::Utc::now(),
    };
    
    replacement_manager.active_rules.insert(simple_rule.id.clone(), simple_rule);
    
    // Perform replacements
    let input1 = "Say hello to everyone";
    let result1 = process_text_replacement(input1, &mut replacement_manager);
    assert_eq!(result1, "Say hi to everyone");
    
    let input2 = "hello world";
    let result2 = process_text_replacement(input2, &mut replacement_manager);
    assert_eq!(result2, "hi world");
    
    // Check history tracking
    assert_eq!(replacement_manager.replacement_history.len(), 2);
    
    let history1 = &replacement_manager.replacement_history[0];
    assert_eq!(history1.original, input1);
    assert_eq!(history1.replaced, result1);
    assert_eq!(history1.rule_id, "hello_rule");
    
    let history2 = &replacement_manager.replacement_history[1];
    assert_eq!(history2.original, input2);
    assert_eq!(history2.replaced, result2);
}
```

#### 6. Performance and Scalability Testing
```rust
#[test]
fn test_replacement_performance() {
    let mut replacement_manager = TextReplacementManager::default();
    
    // Add many replacement rules
    for i in 0..100 {
        let rule = ReplacementRule {
            id: format!("rule_{}", i),
            pattern: format!("pattern{}", i),
            replacement: format!("replacement{}", i),
            rule_type: ReplacementType::SimpleText,
            enabled: true,
            case_sensitive: false,
            word_boundaries: true,
            created_at: chrono::Utc::now(),
        };
        replacement_manager.active_rules.insert(rule.id.clone(), rule);
    }
    
    // Large text input for performance testing
    let large_input = "pattern0 pattern1 pattern2 ".repeat(1000);
    
    let start_time = std::time::Instant::now();
    let result = process_text_replacement(&large_input, &mut replacement_manager);
    let duration = start_time.elapsed();
    
    // Should complete quickly even with many rules
    assert!(duration.as_millis() < 100, 
        "Text replacement took too long: {}ms", duration.as_millis());
    
    // Verify replacements were applied
    assert!(result.contains("replacement0"));
    assert!(result.contains("replacement1"));
    assert!(result.contains("replacement2"));
}
```

### Edge Case Testing

#### 7. Rule Priority and Conflict Testing
```rust
#[test]
fn test_rule_priority_and_conflicts() {
    let mut replacement_manager = TextReplacementManager::default();
    
    // Add conflicting rules (longer patterns should take priority)
    let short_rule = ReplacementRule {
        id: "short".to_string(),
        pattern: "test".to_string(),
        replacement: "short".to_string(),
        rule_type: ReplacementType::SimpleText,
        enabled: true,
        case_sensitive: false,
        word_boundaries: false,
        created_at: chrono::Utc::now(),
    };
    
    let long_rule = ReplacementRule {
        id: "long".to_string(),
        pattern: "test case".to_string(),
        replacement: "long".to_string(),
        rule_type: ReplacementType::SimpleText,
        enabled: true,
        case_sensitive: false,
        word_boundaries: false,
        created_at: chrono::Utc::now(),
    };
    
    replacement_manager.active_rules.insert(short_rule.id.clone(), short_rule);
    replacement_manager.active_rules.insert(long_rule.id.clone(), long_rule);
    
    let input = "This is a test case for rules";
    let result = process_text_replacement(input, &mut replacement_manager);
    
    // Longer pattern should win
    assert_eq!(result, "This is a long for rules");
}
```

#### 8. Settings Persistence Testing
```rust
#[test]
fn test_replacement_settings_persistence() {
    let original_rule = ReplacementRule {
        id: "persistent_rule".to_string(),
        pattern: "orig".to_string(),
        replacement: "repl".to_string(),
        rule_type: ReplacementType::SimpleText,
        enabled: true,
        case_sensitive: true,
        word_boundaries: false,
        created_at: chrono::Utc::now(),
    };
    
    let replacement_settings = TextReplacementSettings {
        global_enabled: true,
        processing_enabled: true,
        rules: vec![original_rule.clone()],
        history_limit: 1000,
    };
    
    // Test serialization/deserialization
    let serialized = serde_json::to_string(&replacement_settings).unwrap();
    let deserialized: TextReplacementSettings = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(replacement_settings.global_enabled, deserialized.global_enabled);
    assert_eq!(replacement_settings.rules.len(), deserialized.rules.len());
    assert_eq!(replacement_settings.rules[0].pattern, deserialized.rules[0].pattern);
    assert_eq!(replacement_settings.rules[0].case_sensitive, deserialized.rules[0].case_sensitive);
}
```

### Manual Testing Checklist

- [ ] Modifier notation checkbox toggles correctly
- [ ] Text replacement works in real-time in text inputs
- [ ] Unicode symbol replacements render correctly
- [ ] Word boundary detection prevents partial matches
- [ ] Case sensitivity toggle works as expected
- [ ] Rule addition/deletion works through UI
- [ ] Replacement history is tracked accurately
- [ ] Performance remains good with many rules
- [ ] Settings persist across application restarts
- [ ] Error handling works for invalid regex patterns

**Bevy Examples**: `./docs/bevy/examples/ui/text_input.rs:445-478`, `./docs/bevy/examples/regex.rs:155-182`  
**Integration Points**: All text replacement components  
**Success Criteria**: All tests pass, sub-100ms replacement time, zero replacement conflicts
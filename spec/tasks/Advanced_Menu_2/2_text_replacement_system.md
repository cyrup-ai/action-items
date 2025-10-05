# Task 2: Advanced Text Replacement System

## Implementation Details

**File**: `ui/src/ui/text_replacement.rs`  
**Lines**: 125-210  
**Architecture**: Real-time text replacement engine with pattern matching  
**Integration**: TextInputSystem, SettingsSystem, PatternMatcher  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct TextReplacementManager {
    pub active_rules: HashMap<String, ReplacementRule>,
    pub global_enabled: bool,
    pub processing_enabled: bool,
    pub replacement_history: VecDeque<ReplacementHistory>,
    pub pattern_matcher: PatternMatcher,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReplacementRule {
    pub id: String,
    pub pattern: String,
    pub replacement: String,
    pub rule_type: ReplacementType,
    pub enabled: bool,
    pub case_sensitive: bool,
    pub word_boundaries: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReplacementType {
    SimpleText,
    RegularExpression,
    ModifierNotation,
    UnicodeSymbol,
}

#[derive(Clone, Debug)]
pub struct ReplacementHistory {
    pub original: String,
    pub replaced: String,
    pub rule_id: String,
    pub timestamp: Instant,
    pub context: String,
}

pub fn text_replacement_system(
    mut replacement_manager: ResMut<TextReplacementManager>,
    mut text_input_events: EventReader<TextInputEvent>,
    mut text_output_events: EventWriter<TextOutputEvent>,
    mut replacement_events: EventWriter<TextReplacementEvent>,
) {
    if !replacement_manager.processing_enabled {
        // Pass through text unchanged when disabled
        for event in text_input_events.read() {
            if let TextInputEvent::CharacterInput(ch) = event {
                text_output_events.send(TextOutputEvent::CharacterOutput(*ch));
            }
        }
        return;
    }

    for event in text_input_events.read() {
        match event {
            TextInputEvent::CharacterInput(ch) => {
                let processed_char = process_character_replacement(*ch, &replacement_manager);
                text_output_events.send(TextOutputEvent::CharacterOutput(processed_char));
            }
            TextInputEvent::TextInput(text) => {
                let processed_text = process_text_replacement(text, &mut replacement_manager);
                text_output_events.send(TextOutputEvent::TextOutput(processed_text));
                
                // Track replacements for history
                if text != &processed_text {
                    replacement_events.send(TextReplacementEvent::ReplacementApplied {
                        original: text.clone(),
                        replaced: processed_text.clone(),
                        rules_applied: replacement_manager.get_applied_rules(text, &processed_text),
                    });
                }
            }
            _ => {}
        }
    }
}

fn process_text_replacement(
    input: &str, 
    manager: &mut TextReplacementManager
) -> String {
    let mut result = input.to_string();
    let mut applied_rules = Vec::new();
    
    // Apply replacement rules in priority order
    let mut sorted_rules: Vec<_> = manager.active_rules.values().collect();
    sorted_rules.sort_by(|a, b| b.pattern.len().cmp(&a.pattern.len())); // Longest patterns first
    
    for rule in sorted_rules {
        if !rule.enabled {
            continue;
        }
        
        let new_result = match rule.rule_type {
            ReplacementType::SimpleText => {
                apply_simple_replacement(&result, rule)
            }
            ReplacementType::RegularExpression => {
                apply_regex_replacement(&result, rule)
            }
            ReplacementType::ModifierNotation => {
                apply_modifier_notation_replacement(&result, rule)
            }
            ReplacementType::UnicodeSymbol => {
                apply_unicode_replacement(&result, rule)
            }
        };
        
        if new_result != result {
            applied_rules.push(rule.id.clone());
            result = new_result;
        }
    }
    
    // Record replacement history
    if !applied_rules.is_empty() {
        manager.replacement_history.push_back(ReplacementHistory {
            original: input.to_string(),
            replaced: result.clone(),
            rule_id: applied_rules.join(","),
            timestamp: Instant::now(),
            context: "text_input".to_string(),
        });
        
        // Limit history size
        if manager.replacement_history.len() > 1000 {
            manager.replacement_history.pop_front();
        }
    }
    
    result
}
```

### Modifier Notation Replacement

**Reference**: `./docs/bevy/examples/ui/text_input.rs:245-278`

```rust
fn apply_modifier_notation_replacement(input: &str, rule: &ReplacementRule) -> String {
    // Handle complex modifier combinations like ^⌃⇧⌘ -> ⌃
    let modifier_patterns = vec![
        ("^⌃⇧⌘", "⌃"),  // Control+Shift+Command -> Control
        ("^⌥⇧⌘", "⌥"),  // Option+Shift+Command -> Option  
        ("⌃⌥⇧⌘", "⌘"),  // All modifiers -> Command only
        ("⌃⇧", "⌃"),     // Control+Shift -> Control
        ("⌥⇧", "⌥"),     // Option+Shift -> Option
        ("⇧⌘", "⌘"),     // Shift+Command -> Command
    ];
    
    let mut result = input.to_string();
    
    for (pattern, replacement) in modifier_patterns {
        if rule.pattern == pattern {
            if rule.case_sensitive {
                result = result.replace(pattern, replacement);
            } else {
                result = result.replace(&pattern.to_lowercase(), replacement);
                result = result.replace(&pattern.to_uppercase(), replacement);
            }
        }
    }
    
    result
}

fn apply_simple_replacement(input: &str, rule: &ReplacementRule) -> String {
    if rule.word_boundaries {
        // Use word boundary regex for precise matching
        let pattern = if rule.case_sensitive {
            format!(r"\b{}\b", regex::escape(&rule.pattern))
        } else {
            format!(r"(?i)\b{}\b", regex::escape(&rule.pattern))
        };
        
        if let Ok(re) = regex::Regex::new(&pattern) {
            re.replace_all(input, &rule.replacement).to_string()
        } else {
            input.to_string()
        }
    } else {
        // Simple string replacement
        if rule.case_sensitive {
            input.replace(&rule.pattern, &rule.replacement)
        } else {
            replace_case_insensitive(input, &rule.pattern, &rule.replacement)
        }
    }
}

fn apply_unicode_replacement(input: &str, rule: &ReplacementRule) -> String {
    // Handle Unicode symbol replacements
    let unicode_mappings = vec![
        ("->", "→"),
        ("<-", "←"), 
        ("<=", "≤"),
        (">=", "≥"),
        ("!=", "≠"),
        ("+-", "±"),
        ("...", "…"),
        ("(c)", "©"),
        ("(r)", "®"),
        ("(tm)", "™"),
    ];
    
    let mut result = input.to_string();
    
    for (pattern, replacement) in unicode_mappings {
        if rule.pattern == pattern {
            result = result.replace(pattern, replacement);
        }
    }
    
    result
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui/ui_checkbox.rs:188-225`

```rust
// Text replacement settings section
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
    // Main toggle for text replacement
    (SettingsRowBundle {
        label: "Replace occurrences of ^⌃⇧⌘ with ⌃".to_string(),
        control: ControlType::Checkbox {
            checked: replacement_manager.active_rules
                .values()
                .any(|rule| rule.pattern == "^⌃⇧⌘" && rule.enabled),
        },
        tooltip: Some("Simplify complex modifier key notation in documentation and help text".to_string()),
        ..default()
    },),
    
    // Advanced replacement rules section
    (ExpansionPanelBundle {
        header: "Advanced Text Replacement Rules".to_string(),
        expanded: false,
        content: NodeBundle {
            children: replacement_manager.active_rules.values()
                .filter(|rule| rule.rule_type != ReplacementType::ModifierNotation)
                .map(|rule| {
                    (ReplacementRuleRowBundle {
                        rule: rule.clone(),
                        editable: true,
                        deletable: true,
                        ..default()
                    },)
                })
                .collect(),
            ..default()
        },
        ..default()
    },),
    
    // Add new rule button
    (ButtonBundle {
        style: Style {
            width: Val::Px(150.0),
            height: Val::Px(32.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        },
        background_color: Color::rgb(0.2, 0.5, 0.8).into(),
        ..default()
    },
    children: &[
        (TextBundle::from_section(
            "Add Rule",
            TextStyle {
                font: asset_server.load("fonts/Inter-Medium.ttf"),
                font_size: 12.0,
                color: Color::WHITE,
            },
        ),),
    ]),
]
```

### Rule Management System

**Reference**: `./docs/bevy/examples/ui/ui_table.rs:128-165`

```rust
pub fn replacement_rule_management_system(
    mut replacement_manager: ResMut<TextReplacementManager>,
    mut rule_events: EventReader<ReplacementRuleEvent>,
    mut ui_events: EventWriter<UIUpdateEvent>,
) {
    for event in rule_events.read() {
        match event {
            ReplacementRuleEvent::AddRule { pattern, replacement, rule_type } => {
                let rule_id = generate_rule_id();
                let new_rule = ReplacementRule {
                    id: rule_id.clone(),
                    pattern: pattern.clone(),
                    replacement: replacement.clone(),
                    rule_type: *rule_type,
                    enabled: true,
                    case_sensitive: false,
                    word_boundaries: true,
                    created_at: chrono::Utc::now(),
                };
                
                replacement_manager.active_rules.insert(rule_id, new_rule);
                ui_events.send(UIUpdateEvent::ReplacementRulesChanged);
            }
            
            ReplacementRuleEvent::DeleteRule { rule_id } => {
                replacement_manager.active_rules.remove(rule_id);
                ui_events.send(UIUpdateEvent::ReplacementRulesChanged);
            }
            
            ReplacementRuleEvent::ToggleRule { rule_id, enabled } => {
                if let Some(rule) = replacement_manager.active_rules.get_mut(rule_id) {
                    rule.enabled = *enabled;
                    ui_events.send(UIUpdateEvent::ReplacementRulesChanged);
                }
            }
        }
    }
}
```

### Architecture Notes

- Real-time text replacement with minimal processing overhead
- Rule priority system (longest patterns matched first to prevent conflicts)
- Comprehensive replacement history for debugging and undo functionality
- Multiple replacement types: simple text, regex, modifier notation, Unicode symbols
- Word boundary detection prevents partial word replacements
- Case sensitivity control for precise matching requirements
- Efficient pattern matching with optimized algorithms

**Bevy Examples**: `./docs/bevy/examples/ui/text_input.rs:315-352`, `./docs/bevy/examples/regex.rs:75-102`  
**Integration Points**: TextInputSystem, SettingsSystem, UISystem  
**Dependencies**: RegexEngine, PatternMatcher, SettingsResource
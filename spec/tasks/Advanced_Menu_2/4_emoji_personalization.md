# Advanced_Menu_2 Task 4: Emoji Personalization System

## Task Overview
Implement comprehensive emoji skin tone selection system with user preferences, automatic emoji variant application, and intelligent emoji suggestions based on user patterns.

## Implementation Requirements

### Core Components
```rust
// Emoji personalization system
#[derive(Resource, Reflect, Debug)]
pub struct EmojiPersonalizationResource {
    pub skin_tone_preferences: SkinTonePreferences,
    pub emoji_usage_patterns: EmojiUsagePatterns,
    pub personalization_settings: PersonalizationSettings,
    pub emoji_variants: EmojiVariantDatabase,
}

#[derive(Reflect, Debug, Clone)]
pub struct SkinTonePreferences {
    pub default_skin_tone: SkinTone,
    pub per_emoji_preferences: HashMap<String, SkinTone>,
    pub auto_apply_skin_tone: bool,
    pub remember_choices: bool,
    pub skin_tone_history: Vec<SkinToneUsage>,
}

#[derive(Reflect, Debug, Clone, Copy)]
pub enum SkinTone {
    Default,      // 🙂 (no modifier)
    Light,        // 🙂🏻 (U+1F3FB)
    MediumLight,  // 🙂🏼 (U+1F3FC)
    Medium,       // 🙂🏽 (U+1F3FD)
    MediumDark,   // 🙂🏾 (U+1F3FE)
    Dark,         // 🙂🏿 (U+1F3FF)
}

#[derive(Component, Reflect, Debug)]
pub struct EmojiPersonalizationComponent {
    pub skin_tone_selector: Entity,
    pub emoji_preview: Entity,
    pub usage_statistics: Entity,
    pub personalization_toggles: Vec<Entity>,
}

#[derive(Reflect, Debug)]
pub struct EmojiUsagePatterns {
    pub frequently_used: HashMap<String, u32>,
    pub recent_usage: VecDeque<EmojiUsageRecord>,
    pub contextual_patterns: HashMap<String, Vec<String>>,
    pub time_based_patterns: HashMap<TimeContext, Vec<String>>,
}

#[derive(Reflect, Debug)]
pub struct EmojiUsageRecord {
    pub emoji: String,
    pub skin_tone: SkinTone,
    pub timestamp: DateTime<Utc>,
    pub context: UsageContext,
}

pub fn emoji_personalization_system(
    mut emoji_res: ResMut<EmojiPersonalizationResource>,
    emoji_events: EventReader<EmojiEvent>,
    mut personalized_emoji_events: EventWriter<PersonalizedEmojiEvent>,
) {
    for emoji_event in emoji_events.read() {
        match emoji_event {
            EmojiEvent::EmojiRequested { base_emoji } => {
                let personalized_emoji = apply_skin_tone_preference(
                    base_emoji,
                    &emoji_res.skin_tone_preferences,
                    &emoji_res.emoji_variants,
                );
                
                personalized_emoji_events.send(PersonalizedEmojiEvent {
                    original: base_emoji.clone(),
                    personalized: personalized_emoji,
                    skin_tone: emoji_res.skin_tone_preferences.default_skin_tone,
                });
            }
            EmojiEvent::SkinToneSelected { emoji, skin_tone } => {
                update_skin_tone_preference(&mut emoji_res, emoji, *skin_tone);
            }
        }
    }
}
```

### Emoji Variant Database
```rust
// Comprehensive emoji variant management
#[derive(Resource, Reflect, Debug)]
pub struct EmojiVariantDatabase {
    pub base_to_variants: HashMap<String, EmojiVariants>,
    pub variant_to_base: HashMap<String, String>,
    pub skin_tone_modifiers: HashMap<SkinTone, String>,
    pub supported_emojis: HashSet<String>,
}

#[derive(Reflect, Debug)]
pub struct EmojiVariants {
    pub base_emoji: String,
    pub skin_tone_variants: HashMap<SkinTone, String>,
    pub has_skin_tone_support: bool,
    pub unicode_version: String,
}

fn apply_skin_tone_preference(
    base_emoji: &str,
    preferences: &SkinTonePreferences,
    variant_db: &EmojiVariantDatabase,
) -> String {
    // Check for per-emoji preference first
    if let Some(preferred_tone) = preferences.per_emoji_preferences.get(base_emoji) {
        if let Some(variant) = get_emoji_variant(base_emoji, *preferred_tone, variant_db) {
            return variant;
        }
    }
    
    // Apply default skin tone if emoji supports it
    if preferences.auto_apply_skin_tone {
        if let Some(variant) = get_emoji_variant(base_emoji, preferences.default_skin_tone, variant_db) {
            return variant;
        }
    }
    
    // Return original emoji if no preferences or variants available
    base_emoji.to_string()
}

fn get_emoji_variant(
    base_emoji: &str,
    skin_tone: SkinTone,
    variant_db: &EmojiVariantDatabase,
) -> Option<String> {
    variant_db.base_to_variants
        .get(base_emoji)?
        .skin_tone_variants
        .get(&skin_tone)
        .cloned()
}
```

### Smart Emoji Suggestions
```rust
// Intelligent emoji suggestion system
#[derive(Reflect, Debug)]
pub struct EmojiSuggestionEngine {
    pub suggestion_algorithm: SuggestionAlgorithm,
    pub context_analyzer: ContextAnalyzer,
    pub learning_model: LearningModel,
    pub suggestion_cache: SuggestionCache,
}

#[derive(Reflect, Debug)]
pub enum SuggestionAlgorithm {
    FrequencyBased,
    ContextAware,
    MachineLearning,
    Hybrid,
}

pub fn emoji_suggestion_system(
    emoji_res: Res<EmojiPersonalizationResource>,
    suggestion_engine: Res<EmojiSuggestionEngine>,
    suggestion_events: EventReader<EmojiSuggestionRequestEvent>,
    mut suggestion_response_events: EventWriter<EmojiSuggestionResponseEvent>,
) {
    for request in suggestion_events.read() {
        let suggestions = generate_emoji_suggestions(
            &request.context,
            &emoji_res.emoji_usage_patterns,
            &suggestion_engine,
        );
        
        suggestion_response_events.send(EmojiSuggestionResponseEvent {
            request_id: request.request_id,
            suggestions,
        });
    }
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `ui/text.rs` - Emoji text rendering
- `ui/ui.rs` - Emoji picker UI components
- `reflection/reflection.rs` - Preference serialization

### Implementation Pattern
```rust
// Based on ui/text.rs for emoji rendering
fn emoji_text_system(
    mut text_query: Query<&mut Text, With<EmojiTextComponent>>,
    emoji_res: Res<EmojiPersonalizationResource>,
) {
    for mut text in &mut text_query {
        for section in &mut text.sections {
            section.value = apply_emoji_personalization(
                &section.value,
                &emoji_res.skin_tone_preferences,
            );
        }
    }
}

// Based on ui/ui.rs for emoji picker
fn emoji_picker_system(
    mut commands: Commands,
    emoji_res: Res<EmojiPersonalizationResource>,
    picker_query: Query<&EmojiPickerComponent>,
) {
    for picker in &picker_query {
        let frequently_used = get_frequently_used_emojis(&emoji_res.emoji_usage_patterns);
        update_emoji_picker_suggestions(&mut commands, picker, &frequently_used);
    }
}
```

## Unicode Compliance
- Full Unicode emoji standard compliance
- Support for latest emoji versions
- Proper skin tone modifier handling
- ZWJ sequence support for complex emojis

## Performance Constraints
- **ZERO ALLOCATIONS** during emoji application
- Efficient emoji variant lookup
- Optimized suggestion generation
- Minimal impact on text rendering performance

## Success Criteria
- Complete emoji personalization system implementation
- Accurate skin tone application and preferences
- No unwrap()/expect() calls in production code
- Zero-allocation emoji processing
- Intelligent suggestion algorithms

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for skin tone application logic
- Integration tests for emoji variant database
- Performance tests for emoji processing speed
- Unicode compliance tests for emoji standards
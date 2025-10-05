# Task 5: QA Validation - Favicon Provider and Emoji Skin Tone Systems

## Comprehensive Testing Protocol

**File**: `tests/ui/personalization_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: FaviconSystem, EmojiPersonalization, AssetManager  

### Test Categories

#### 1. Favicon Provider Selection Testing
**Reference**: `./docs/bevy/examples/asset_loading/asset_loading.rs:285-312`
```rust
#[test]
fn test_favicon_provider_selection() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, favicon_provider_system)
       .add_event::<FaviconRequestEvent>()
       .add_event::<FaviconResponseEvent>();

    let providers = vec![
        FaviconProviderInfo {
            provider: FaviconProvider::Raycast,
            display_name: "Raycast".to_string(),
            description: "Internal service".to_string(),
            privacy_friendly: true,
            response_time_ms: Some(50),
            reliability_score: 0.98,
        },
        FaviconProviderInfo {
            provider: FaviconProvider::GoogleAPI,
            display_name: "Google Favicon API".to_string(),
            description: "External service".to_string(),
            privacy_friendly: false,
            response_time_ms: Some(150),
            reliability_score: 0.95,
        },
    ];

    let mut favicon_manager = FaviconProviderManager {
        active_provider: FaviconProvider::Raycast,
        available_providers: providers.clone(),
        cache: FaviconCache::default(),
        fallback_enabled: true,
        privacy_mode: PrivacyMode::Balanced,
    };

    app.world_mut().insert_resource(favicon_manager);

    // Test provider switching
    let mut favicon_events = app.world_mut().resource_mut::<Events<FaviconRequestEvent>>();
    favicon_events.send(FaviconRequestEvent {
        domain: "example.com".to_string(),
        size: 32,
    });

    app.update();

    let response_events = app.world().resource::<Events<FaviconResponseEvent>>();
    assert!(!response_events.is_empty(), "Favicon request should generate response");
}
```

#### 2. Favicon Caching System Testing
```rust
#[test]
fn test_favicon_caching() {
    let mut cache = FaviconCache {
        icons: HashMap::new(),
        cache_size_limit: 100,
        cache_ttl: Duration::from_secs(3600),
        last_cleanup: Instant::now(),
    };

    let test_favicon = CachedFavicon {
        url: "https://example.com/favicon.ico".to_string(),
        image_data: vec![0x89, 0x50, 0x4E, 0x47], // PNG header
        format: ImageFormat::Png,
        cached_at: Instant::now(),
        access_count: 1,
        last_accessed: Instant::now(),
    };

    let cache_key = "example.com:32";
    cache.icons.insert(cache_key.to_string(), test_favicon.clone());

    // Test cache hit
    assert!(cache.icons.contains_key(cache_key));
    
    // Test TTL expiration
    let expired_favicon = CachedFavicon {
        cached_at: Instant::now() - Duration::from_secs(7200), // 2 hours ago
        ..test_favicon.clone()
    };
    
    cache.icons.insert("expired.com:32".to_string(), expired_favicon);
    
    // Cleanup should remove expired entries
    cleanup_favicon_cache(&mut cache);
    assert!(!cache.icons.contains_key("expired.com:32"));
    assert!(cache.icons.contains_key(cache_key)); // Fresh entry should remain
}
```

#### 3. Privacy Mode Enforcement Testing
**Reference**: `./docs/bevy/examples/networking/http_client.rs:185-212`
```rust
#[test]
fn test_privacy_mode_enforcement() {
    let test_cases = vec![
        (PrivacyMode::Strict, FaviconProvider::GoogleAPI, false), // Should block
        (PrivacyMode::Strict, FaviconProvider::Raycast, true),    // Should allow
        (PrivacyMode::Balanced, FaviconProvider::GoogleAPI, true), // Should allow with consent
        (PrivacyMode::Permissive, FaviconProvider::GoogleAPI, true), // Should allow
    ];

    for (privacy_mode, provider, should_allow) in test_cases {
        let mut favicon_manager = FaviconProviderManager {
            active_provider: provider.clone(),
            available_providers: Vec::new(),
            cache: FaviconCache::default(),
            fallback_enabled: true,
            privacy_mode,
        };

        let request = FaviconRequestEvent {
            domain: "test.com".to_string(),
            size: 32,
        };

        let result = should_allow_request(&request, &favicon_manager);
        assert_eq!(result, should_allow, 
            "Privacy enforcement failed for {:?} with {:?}", privacy_mode, provider);
    }
}
```

#### 4. Emoji Skin Tone Application Testing
**Reference**: `./docs/bevy/examples/ui/ui_texture_atlas.rs:385-418`
```rust
#[test]
fn test_emoji_skin_tone_application() {
    let emoji_settings = EmojiPersonalization {
        selected_skin_tone: SkinTone::Medium,
        available_tones: vec![
            SkinToneOption {
                tone: SkinTone::Medium,
                display_name: "Medium".to_string(),
                unicode_modifier: Some('\u{1F3FD}'),
                example_emoji: "👋🏽".to_string(),
                accessibility_description: "Waving hand with medium skin tone".to_string(),
            },
        ],
        emoji_preferences: HashMap::new(),
        auto_apply: true,
        accessibility_mode: false,
    };

    let test_cases = vec![
        // (input_emoji, expected_output)
        ("👋", "👋🏽"), // Should apply medium skin tone
        ("👍", "👍🏽"), // Should apply medium skin tone
        ("❤️", "❤️"),  // Should not modify (no skin tone support)
        ("🚀", "🚀"),  // Should not modify (no skin tone support)
    ];

    for (input, expected) in test_cases {
        let result = apply_skin_tone_preference(
            input,
            emoji_settings.selected_skin_tone,
            &emoji_settings.emoji_preferences
        );
        
        assert_eq!(result, expected, 
            "Skin tone application failed for emoji '{}', got '{}', expected '{}'", 
            input, result, expected);
    }
}
```

#### 5. Emoji Skin Tone Support Detection Testing
```rust
#[test]
fn test_emoji_skin_tone_support_detection() {
    let supported_emojis = vec![
        "👋", "👍", "👎", "👊", "✊", "🤚", "🖐", "✋", "🖖", "👌",
        "👶", "🧒", "👦", "👧", "🧑", "👱", "👨", "🧔", "👩", "💪",
    ];
    
    let unsupported_emojis = vec![
        "❤️", "🚀", "🎉", "🌟", "🍕", "🎵", "⚡", "🔥", "💡", "🌈",
    ];

    for emoji in supported_emojis {
        assert!(supports_skin_tone(emoji), 
            "Emoji '{}' should support skin tone modifiers", emoji);
    }

    for emoji in unsupported_emojis {
        assert!(!supports_skin_tone(emoji), 
            "Emoji '{}' should not support skin tone modifiers", emoji);
    }
}
```

#### 6. Custom Emoji Preferences Testing
```rust
#[test]
fn test_custom_emoji_preferences() {
    let mut preferences = HashMap::new();
    preferences.insert("👋".to_string(), "👋🏿".to_string()); // Custom dark skin tone for wave
    preferences.insert("👍".to_string(), "👍🏻".to_string()); // Custom light skin tone for thumbs up

    let emoji_settings = EmojiPersonalization {
        selected_skin_tone: SkinTone::Medium, // Default medium, but preferences override
        available_tones: Vec::new(),
        emoji_preferences: preferences,
        auto_apply: true,
        accessibility_mode: false,
    };

    // Custom preferences should override default skin tone
    let wave_result = apply_skin_tone_preference(
        "👋",
        emoji_settings.selected_skin_tone,
        &emoji_settings.emoji_preferences
    );
    assert_eq!(wave_result, "👋🏿");

    let thumbs_result = apply_skin_tone_preference(
        "👍",
        emoji_settings.selected_skin_tone,
        &emoji_settings.emoji_preferences
    );
    assert_eq!(thumbs_result, "👍🏻");

    // Non-custom emoji should use default skin tone
    let fist_result = apply_skin_tone_preference(
        "👊",
        emoji_settings.selected_skin_tone,
        &emoji_settings.emoji_preferences
    );
    assert_eq!(fist_result, "👊🏽"); // Medium skin tone applied
}
```

### Edge Case Testing

#### 7. Favicon Fallback Testing
```rust
#[test]
fn test_favicon_fallback_mechanism() {
    let mut favicon_manager = FaviconProviderManager {
        active_provider: FaviconProvider::CustomService("http://unreachable.service".to_string()),
        available_providers: vec![
            FaviconProviderInfo {
                provider: FaviconProvider::Raycast,
                display_name: "Raycast".to_string(),
                description: "Fallback provider".to_string(),
                privacy_friendly: true,
                response_time_ms: Some(50),
                reliability_score: 0.98,
            },
        ],
        cache: FaviconCache::default(),
        fallback_enabled: true,
        privacy_mode: PrivacyMode::Balanced,
    };

    // Simulate primary provider failure
    let request = FaviconRequestEvent {
        domain: "example.com".to_string(),
        size: 32,
    };

    // Should automatically fallback to Raycast provider
    let fallback_result = handle_favicon_request_with_fallback(&request, &mut favicon_manager);
    assert!(fallback_result.is_ok(), "Fallback mechanism should handle provider failure");
}
```

#### 8. Unicode Emoji Validation Testing
**Reference**: `./docs/bevy/examples/unicode_handling.rs:225-252`
```rust
#[test]
fn test_unicode_emoji_validation() {
    let test_cases = vec![
        // (input, is_valid_emoji)
        ("👋", true),
        ("🏽", false), // Skin tone modifier alone is not a complete emoji
        ("👋🏽", true), // Base + modifier is valid
        ("👋🏽🏽", false), // Double modifier is invalid
        ("Hello", false), // Text is not emoji
        ("👨‍💻", true), // Complex emoji with ZWJ sequences
        ("🇺🇸", true), // Flag emoji
    ];

    for (input, should_be_valid) in test_cases {
        let is_valid = validate_emoji_sequence(input);
        assert_eq!(is_valid, should_be_valid, 
            "Emoji validation failed for '{}' (bytes: {:?})", 
            input, input.as_bytes());
    }
}
```

#### 9. Settings Persistence Testing
```rust
#[test]
fn test_personalization_settings_persistence() {
    let original_emoji_settings = EmojiPersonalization {
        selected_skin_tone: SkinTone::MediumDark,
        available_tones: Vec::new(),
        emoji_preferences: {
            let mut prefs = HashMap::new();
            prefs.insert("👋".to_string(), "👋🏿".to_string());
            prefs
        },
        auto_apply: true,
        accessibility_mode: false,
    };

    let original_favicon_settings = FaviconProviderSettings {
        active_provider: FaviconProvider::Raycast,
        privacy_mode: PrivacyMode::Strict,
        cache_ttl_hours: 24,
        fallback_enabled: true,
    };

    // Test serialization/deserialization
    let emoji_serialized = serde_json::to_string(&original_emoji_settings).unwrap();
    let emoji_deserialized: EmojiPersonalization = serde_json::from_str(&emoji_serialized).unwrap();

    let favicon_serialized = serde_json::to_string(&original_favicon_settings).unwrap();
    let favicon_deserialized: FaviconProviderSettings = serde_json::from_str(&favicon_serialized).unwrap();

    // Verify emoji settings
    assert_eq!(original_emoji_settings.selected_skin_tone, emoji_deserialized.selected_skin_tone);
    assert_eq!(original_emoji_settings.auto_apply, emoji_deserialized.auto_apply);
    assert_eq!(original_emoji_settings.emoji_preferences.len(), emoji_deserialized.emoji_preferences.len());

    // Verify favicon settings
    assert_eq!(original_favicon_settings.active_provider, favicon_deserialized.active_provider);
    assert_eq!(original_favicon_settings.privacy_mode, favicon_deserialized.privacy_mode);
}
```

### Manual Testing Checklist

- [ ] Favicon provider dropdown shows all available options
- [ ] Provider switching updates favicon requests correctly
- [ ] Privacy mode blocks/allows requests as configured
- [ ] Favicon caching reduces redundant network requests
- [ ] Emoji skin tone selector shows all 6 tone options
- [ ] Skin tone selection updates emoji display in real-time
- [ ] Custom emoji preferences override default skin tone
- [ ] Unsupported emojis remain unchanged when skin tone is applied
- [ ] Settings persist across application restarts
- [ ] Fallback mechanisms work when primary provider fails

**Bevy Examples**: `./docs/bevy/examples/ui/ui_texture_atlas.rs:445-478`, `./docs/bevy/examples/asset_loading/asset_loading.rs:385-412`  
**Integration Points**: All personalization system components  
**Success Criteria**: All tests pass, reliable provider fallback, accurate emoji skin tone application
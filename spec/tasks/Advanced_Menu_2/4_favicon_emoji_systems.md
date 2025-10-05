# Task 4: Favicon Provider Configuration and Emoji Skin Tone Systems

## Implementation Details

**File**: `ui/src/ui/personalization.rs`  
**Lines**: 225-335  
**Architecture**: Dual personalization system with favicon management and emoji customization  
**Integration**: AssetManager, SettingsSystem, LocalizationManager  

### Favicon Provider System

```rust
#[derive(Resource, Clone, Debug)]
pub struct FaviconProviderManager {
    pub active_provider: FaviconProvider,
    pub available_providers: Vec<FaviconProviderInfo>,
    pub cache: FaviconCache,
    pub fallback_enabled: bool,
    pub privacy_mode: PrivacyMode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FaviconProvider {
    Raycast,
    GoogleAPI,
    CustomService(String),
    LocalCache,
}

#[derive(Clone, Debug)]
pub struct FaviconProviderInfo {
    pub provider: FaviconProvider,
    pub display_name: String,
    pub description: String,
    pub privacy_friendly: bool,
    pub response_time_ms: Option<u64>,
    pub reliability_score: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct FaviconCache {
    pub icons: HashMap<String, CachedFavicon>,
    pub cache_size_limit: usize,
    pub cache_ttl: Duration,
    pub last_cleanup: Instant,
}

#[derive(Clone, Debug)]
pub struct CachedFavicon {
    pub url: String,
    pub image_data: Vec<u8>,
    pub format: ImageFormat,
    pub cached_at: Instant,
    pub access_count: u32,
    pub last_accessed: Instant,
}

pub fn favicon_provider_system(
    mut favicon_manager: ResMut<FaviconProviderManager>,
    mut favicon_requests: EventReader<FaviconRequestEvent>,
    mut favicon_responses: EventWriter<FaviconResponseEvent>,
    asset_server: Res<AssetServer>,
    http_client: Res<HttpClient>,
) {
    // Process favicon requests
    for request in favicon_requests.read() {
        let cache_key = format!("{}:{}", request.domain, request.size);
        
        // Check cache first
        if let Some(cached) = favicon_manager.cache.icons.get(&cache_key) {
            if cached.cached_at.elapsed() < favicon_manager.cache.cache_ttl {
                favicon_responses.send(FaviconResponseEvent::CacheHit {
                    domain: request.domain.clone(),
                    image_handle: asset_server.load_from_bytes(&cached.image_data),
                });
                continue;
            }
        }
        
        // Fetch from active provider
        match favicon_manager.active_provider {
            FaviconProvider::Raycast => {
                let url = format!("raycast://favicon/{}/{}", request.domain, request.size);
                favicon_responses.send(FaviconResponseEvent::FetchStarted {
                    domain: request.domain.clone(),
                    provider: FaviconProvider::Raycast,
                });
            }
            FaviconProvider::GoogleAPI => {
                if favicon_manager.privacy_mode != PrivacyMode::Strict {
                    let url = format!("https://www.google.com/s2/favicons?domain={}&sz={}", 
                                     request.domain, request.size);
                    fetch_favicon_async(&url, request.domain.clone(), http_client);
                } else {
                    favicon_responses.send(FaviconResponseEvent::PrivacyBlocked {
                        domain: request.domain.clone(),
                    });
                }
            }
            FaviconProvider::LocalCache => {
                fetch_local_favicon(&request.domain, &asset_server);
            }
            FaviconProvider::CustomService(ref service_url) => {
                let url = format!("{}/favicon/{}/{}", service_url, request.domain, request.size);
                fetch_favicon_async(&url, request.domain.clone(), http_client);
            }
        }
    }
    
    // Periodic cache cleanup
    if favicon_manager.cache.last_cleanup.elapsed() > Duration::from_secs(3600) {
        cleanup_favicon_cache(&mut favicon_manager.cache);
    }
}
```

### Emoji Skin Tone System

**Reference**: `./docs/bevy/examples/ui/ui_texture_atlas.rs:185-218`

```rust
#[derive(Resource, Clone, Debug)]
pub struct EmojiPersonalization {
    pub selected_skin_tone: SkinTone,
    pub available_tones: Vec<SkinToneOption>,
    pub emoji_preferences: HashMap<String, String>,
    pub auto_apply: bool,
    pub accessibility_mode: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SkinTone {
    Default,     // 👋 (yellow/default)
    Light,       // 👋🏻 (light skin tone)
    MediumLight, // 👋🏼 (medium-light skin tone)
    Medium,      // 👋🏽 (medium skin tone)
    MediumDark,  // 👋🏾 (medium-dark skin tone)
    Dark,        // 👋🏿 (dark skin tone)
}

#[derive(Clone, Debug)]
pub struct SkinToneOption {
    pub tone: SkinTone,
    pub display_name: String,
    pub unicode_modifier: Option<char>,
    pub example_emoji: String,
    pub accessibility_description: String,
}

pub fn emoji_personalization_system(
    emoji_settings: Res<EmojiPersonalization>,
    mut text_events: EventReader<EmojiInputEvent>,
    mut personalized_events: EventWriter<PersonalizedEmojiEvent>,
) {
    for event in text_events.read() {
        match event {
            EmojiInputEvent::EmojiRequested { base_emoji, context } => {
                let personalized = apply_skin_tone_preference(
                    base_emoji, 
                    emoji_settings.selected_skin_tone,
                    &emoji_settings.emoji_preferences
                );
                
                personalized_events.send(PersonalizedEmojiEvent::EmojiPersonalized {
                    original: base_emoji.clone(),
                    personalized,
                    context: context.clone(),
                });
            }
        }
    }
}

fn apply_skin_tone_preference(
    base_emoji: &str,
    selected_tone: SkinTone,
    preferences: &HashMap<String, String>
) -> String {
    // Check for specific emoji preference first
    if let Some(preferred) = preferences.get(base_emoji) {
        return preferred.clone();
    }
    
    // Apply skin tone modifier if applicable
    if let Some(modifier) = get_skin_tone_modifier(selected_tone) {
        if supports_skin_tone(base_emoji) {
            return format!("{}{}", base_emoji, modifier);
        }
    }
    
    // Return original emoji if no modification applies
    base_emoji.to_string()
}

fn get_skin_tone_modifier(tone: SkinTone) -> Option<char> {
    match tone {
        SkinTone::Default => None,
        SkinTone::Light => Some('\u{1F3FB}'),       // 🏻
        SkinTone::MediumLight => Some('\u{1F3FC}'), // 🏼
        SkinTone::Medium => Some('\u{1F3FD}'),      // 🏽
        SkinTone::MediumDark => Some('\u{1F3FE}'),  // 🏾
        SkinTone::Dark => Some('\u{1F3FF}'),        // 🏿
    }
}

fn supports_skin_tone(emoji: &str) -> bool {
    // Define emojis that support skin tone modifiers
    const SKIN_TONE_SUPPORTED: &[&str] = &[
        "👋", "👍", "👎", "👊", "✊", "🤚", "🖐", "✋", "🖖", "👌",
        "🤌", "🤏", "✌", "🤞", "🤟", "🤘", "🤙", "👈", "👉", "👆",
        "🖕", "👇", "☝", "👏", "🙌", "👐", "🤲", "🤝", "🙏", "✍",
        "💅", "🤳", "💪", "🦵", "🦶", "👂", "🦻", "👃", "👶", "🧒",
        "👦", "👧", "🧑", "👱", "👨", "🧔", "👩", "🧓", "👴", "👵",
        // Add more as needed
    ];
    
    SKIN_TONE_SUPPORTED.contains(&emoji)
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui/ui_button_grid.rs:128-175`

```rust
// Favicon provider dropdown
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        width: Val::Percent(100.0),
        height: Val::Px(40.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::horizontal(Val::Px(16.0)),
        margin: UiRect::bottom(Val::Px(8.0)),
        ..default()
    },
    background_color: Color::rgba(0.12, 0.12, 0.12, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(6.0)),
    ..default()
},
children: &[
    (TextBundle::from_section(
        "Favicon Provider",
        TextStyle {
            font: asset_server.load("fonts/Inter-Medium.ttf"),
            font_size: 14.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    ),),
    (DropdownBundle {
        options: favicon_manager.available_providers.iter()
            .map(|provider| provider.display_name.clone())
            .collect(),
        selected_index: favicon_manager.available_providers.iter()
            .position(|p| p.provider == favicon_manager.active_provider)
            .unwrap_or(0),
        width: Val::Px(150.0),
        ..default()
    },),
    (InfoIconBundle {
        tooltip: "Choose favicon service for website icons. Raycast uses internal service for privacy.".to_string(),
        ..default()
    },),
],

// Emoji skin tone selector
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(16.0)),
        row_gap: Val::Px(8.0),
        ..default()
    },
    background_color: Color::rgba(0.08, 0.08, 0.08, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(8.0)),
    ..default()
},
children: &[
    (TextBundle::from_section(
        "Emoji Skin Tone",
        TextStyle {
            font: asset_server.load("fonts/Inter-Medium.ttf"),
            font_size: 14.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    ),),
    // Skin tone grid
    (NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            ..default()
        },
        ..default()
    },
    children: emoji_settings.available_tones.iter().enumerate().map(|(i, tone_option)| {
        (ButtonBundle {
            style: Style {
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: if tone_option.tone == emoji_settings.selected_skin_tone {
                Color::rgb(0.3, 0.6, 0.9)
            } else {
                Color::rgba(0.2, 0.2, 0.2, 1.0)
            }.into(),
            border_color: if tone_option.tone == emoji_settings.selected_skin_tone {
                Color::rgb(0.4, 0.7, 1.0)
            } else {
                Color::rgba(0.4, 0.4, 0.4, 1.0)
            }.into(),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        children: &[
            (TextBundle::from_section(
                &tone_option.example_emoji,
                TextStyle {
                    font: asset_server.load("fonts/NotoColorEmoji.ttf"),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ),),
        ])
    }).collect::<Vec<_>>()),
]
```

### Architecture Notes

- Favicon provider abstraction supports multiple services with fallback mechanisms
- Privacy-aware favicon fetching with user control over external service usage
- Intelligent favicon caching with TTL and LRU eviction policies
- Unicode-compliant emoji skin tone system with comprehensive modifier support
- Personalization persistence with cross-device synchronization capabilities
- Accessibility integration for screen readers and inclusive emoji descriptions
- Performance-optimized caching reduces network requests and improves responsiveness

**Bevy Examples**: `./docs/bevy/examples/ui/ui_texture_atlas.rs:245-282`, `./docs/bevy/examples/asset_loading/asset_loading.rs:158-185`  
**Integration Points**: AssetManager, SettingsSystem, NetworkManager  
**Dependencies**: HttpClient, AssetServer, LocalizationResource
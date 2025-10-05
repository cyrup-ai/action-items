# Advanced_Menu_2 Task 2: Favicon Provider System

## Task Overview
Implement comprehensive favicon service configuration and caching system for displaying website icons in launcher results, with intelligent fetching, caching strategies, and fallback mechanisms.

## Implementation Requirements

### Core Components
```rust
// Favicon provider system
#[derive(Resource, Reflect, Debug)]
pub struct FaviconProviderResource {
    pub favicon_cache: FaviconCache,
    pub provider_config: FaviconProviderConfiguration,
    pub fetch_queue: FaviconFetchQueue,
    pub fallback_handler: FaviconFallbackHandler,
}

#[derive(Reflect, Debug)]
pub struct FaviconCache {
    pub cached_favicons: HashMap<String, CachedFavicon>,
    pub cache_policy: CachePolicy,
    pub storage_manager: FaviconStorageManager,
    pub cache_stats: CacheStatistics,
}

#[derive(Reflect, Debug, Clone)]
pub struct CachedFavicon {
    pub url: String,
    pub domain: String,
    pub favicon_data: Vec<u8>,
    pub content_type: String,
    pub cache_timestamp: DateTime<Utc>,
    pub expiry_timestamp: DateTime<Utc>,
    pub access_count: u32,
    pub last_accessed: DateTime<Utc>,
    pub size: u64,
}

#[derive(Reflect, Debug)]
pub struct FaviconProviderConfiguration {
    pub enabled_providers: Vec<FaviconProvider>,
    pub fetch_timeout: Duration,
    pub max_concurrent_fetches: u32,
    pub fallback_strategy: FallbackStrategy,
    pub quality_preference: QualityPreference,
}

#[derive(Reflect, Debug)]
pub enum FaviconProvider {
    Direct,
    GoogleFavicons,
    IconFinder,
    FaviconKit,
    DuckDuckGo,
    Custom { endpoint: String },
}

pub fn favicon_provider_system(
    mut favicon_res: ResMut<FaviconProviderResource>,
    mut fetch_events: EventReader<FaviconFetchEvent>,
    mut favicon_ready_events: EventWriter<FaviconReadyEvent>,
) {
    // Process favicon fetch requests
    for fetch_event in fetch_events.read() {
        if let Some(cached_favicon) = favicon_res.favicon_cache.cached_favicons.get(&fetch_event.url) {
            if !is_favicon_expired(cached_favicon) {
                favicon_ready_events.send(FaviconReadyEvent {
                    url: fetch_event.url.clone(),
                    favicon_data: cached_favicon.favicon_data.clone(),
                });
                continue;
            }
        }
        
        // Queue for fetching if not in cache or expired
        favicon_res.fetch_queue.queue_fetch(fetch_event.url.clone());
    }
}
```

### Async Favicon Fetching
```rust
// Asynchronous favicon fetching system
#[derive(Reflect, Debug)]
pub struct FaviconFetchQueue {
    pub pending_fetches: VecDeque<FaviconFetchRequest>,
    pub active_fetches: HashMap<String, FetchTask>,
    pub fetch_limiter: RateLimiter,
}

#[derive(Reflect, Debug)]
pub struct FaviconFetchRequest {
    pub url: String,
    pub domain: String,
    pub priority: FetchPriority,
    pub requested_at: DateTime<Utc>,
    pub retry_count: u8,
}

#[derive(Reflect, Debug)]
pub enum FetchPriority {
    High,    // Currently visible items
    Medium,  // Soon-to-be-visible items
    Low,     // Background prefetch
}

pub fn favicon_fetch_system(
    mut commands: Commands,
    mut favicon_res: ResMut<FaviconProviderResource>,
) {
    // Process fetch queue with concurrency limits
    while favicon_res.fetch_queue.can_start_fetch() {
        if let Some(fetch_request) = favicon_res.fetch_queue.pending_fetches.pop_front() {
            let task = commands.spawn_task(async move {
                fetch_favicon_async(fetch_request).await
            });
            
            favicon_res.fetch_queue.active_fetches.insert(
                fetch_request.url.clone(),
                FetchTask { task_handle: task, started_at: Utc::now() }
            );
        }
    }
}

async fn fetch_favicon_async(request: FaviconFetchRequest) -> Result<FaviconFetchResult, FaviconError> {
    // Try multiple favicon sources in order of preference
    for provider in get_enabled_providers() {
        match try_fetch_from_provider(&provider, &request.url).await {
            Ok(favicon_data) => {
                return Ok(FaviconFetchResult {
                    url: request.url,
                    favicon_data,
                    provider: provider.name(),
                    fetch_duration: calculate_fetch_duration(),
                });
            }
            Err(e) => {
                // Log error and try next provider
                log_provider_error(&provider, &e);
            }
        }
    }
    
    Err(FaviconError::AllProvidersFailed)
}
```

### Fallback System
```rust
// Intelligent fallback handling
#[derive(Reflect, Debug)]
pub struct FaviconFallbackHandler {
    pub fallback_icons: HashMap<String, FallbackIcon>,
    pub domain_patterns: Vec<DomainPattern>,
    pub default_icon: DefaultIconSettings,
}

#[derive(Reflect, Debug)]
pub struct FallbackIcon {
    pub pattern: String,
    pub icon_data: Vec<u8>,
    pub icon_type: FallbackIconType,
}

#[derive(Reflect, Debug)]
pub enum FallbackIconType {
    DomainBased,
    CategoryBased,
    Generated { seed: String },
    Default,
}

fn generate_fallback_favicon(
    domain: &str,
    fallback_handler: &FaviconFallbackHandler,
) -> Vec<u8> {
    // Generate or retrieve fallback favicon with zero allocations
    if let Some(pattern_match) = find_domain_pattern(domain, &fallback_handler.domain_patterns) {
        return pattern_match.icon_data.clone();
    }
    
    // Generate domain-based icon
    generate_domain_icon(domain, &fallback_handler.default_icon)
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `async_compute/async_compute.rs` - Async favicon fetching
- `asset/asset_loading.rs` - Favicon asset management
- `ui/ui_texture_atlas.rs` - Favicon display in UI

### Implementation Pattern
```rust
// Based on async_compute.rs for favicon fetching
fn async_favicon_system(
    mut commands: Commands,
    favicon_tasks: Query<Entity, With<FaviconFetchTask>>,
) {
    for task_entity in &favicon_tasks {
        let task = commands.spawn_task(async move {
            // Async favicon fetch with error handling
            fetch_and_cache_favicon().await
        });
    }
}

// Based on asset_loading.rs for favicon management
fn favicon_asset_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    favicon_res: Res<FaviconProviderResource>,
) {
    for (url, cached_favicon) in &favicon_res.favicon_cache.cached_favicons {
        let handle = create_favicon_texture(&cached_favicon.favicon_data);
        commands.spawn(FaviconBundle {
            texture: handle,
            url: url.clone(),
        });
    }
}
```

## Cache Management
- LRU eviction policy for favicon cache
- Configurable cache size and retention policies
- Disk-based cache persistence
- Cache warming strategies for popular domains

## Performance Constraints
- **ZERO ALLOCATIONS** during favicon display
- Efficient caching with minimal memory footprint
- Async fetching to prevent UI blocking
- Rate limiting to prevent service abuse

## Success Criteria
- Complete favicon provider system implementation
- Efficient caching and fetching mechanisms
- No unwrap()/expect() calls in production code
- Zero-allocation favicon display
- Robust fallback handling for failed fetches

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for favicon fetching logic
- Integration tests for cache management
- Performance tests for fetch queue efficiency
- Network resilience tests for fetch failures
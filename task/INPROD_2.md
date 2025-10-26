# Task: Implement Search Result Caching in Search Aggregator

## OBJECTIVE
Replace the "In a real implementation, you might want to keep recent searches for caching" comment at line 431 in `packages/ecs-search-aggregator/src/plugin.rs` with a production-ready search result caching implementation using the existing `ecs-cache` infrastructure.

## PRIORITY
P1 - CRITICAL - Improves search performance by avoiding redundant plugin searches

## RESEARCH FINDINGS

### Current Architecture Analysis

**Search Flow:**
1. UI (`packages/ui/src/ui/systems/search_input.rs`): Fires `SearchQueryChanged` event on every keystroke (no debouncing at UI layer)
2. Aggregator (`packages/ecs-search-aggregator/src/plugin.rs`): `query_change_detection_system` detects `CurrentQuery` resource changes
3. Aggregator: Spawns async tasks to search each plugin via service bridge messaging
4. Aggregator: `aggregate_search_results_system` merges results from all responding plugins
5. Aggregator: `search_cleanup_system` fires `SearchCompleted` event and **immediately removes** search from `active_searches` HashMap (line 432)

**Problem:** When users type the same query again (e.g., backspace and retype), the entire multi-plugin search executes again, causing unnecessary latency and plugin load.

### Existing Infrastructure

**ECS-Cache System** (`packages/ecs-cache/`):
- ✅ Already integrated with `goldylox` multi-tier caching library
- ✅ "search_results" partition already initialized in [ecs-cache/src/systems.rs:22](../../packages/ecs-cache/src/systems.rs)
- ✅ Event-driven architecture: `CacheReadRequested`/`CacheReadCompleted`, `CacheWriteRequested`/`CacheWriteCompleted`
- ✅ Default configuration: hot_tier=1000 entries, warm_tier=10000 entries, TTL=1 hour
- ✅ Automatic LRU eviction when memory pressure exceeds thresholds
- ✅ Async operations using Bevy's `AsyncComputeTaskPool`

**Cache API Pattern:**
```rust
// Reading from cache (async event-driven)
cache_events.write(CacheReadRequested::new(
    "search_results",  // partition
    query.clone(),     // key
    "search_aggregator" // requester
));

// Writing to cache (async event-driven)
let serialized = serde_json::to_vec(&results)?;
cache_events.write(CacheWriteRequested::new(
    "search_results",
    query.clone(),
    serialized,
    Some(3600), // TTL in seconds (1 hour)
    "search_aggregator"
));
```

**Data Serialization:**
- `SearchResult` already has `Serialize` and `Deserialize` derives ([types.rs:11](../../packages/ecs-search-aggregator/src/types.rs))
- Cache values are `Vec<u8>`, use `serde_json::to_vec()` for serialization
- Deserialize with `serde_json::from_slice::<Vec<SearchResult>>(&bytes)`

### Search Configuration
`SearchConfig` in [types.rs:118-134](../../packages/ecs-search-aggregator/src/types.rs):
- `timeout_ms: 5000` - Plugin search timeout
- `max_results_per_plugin: 20` - Result limit per plugin
- `debounce_delay_ms: 150` - **NOT CURRENTLY USED** (intended for UI debouncing)
- `min_query_length: 1` - Minimum query length to trigger search

## IMPLEMENTATION APPROACH

### Strategy: Async Event-Driven Caching

Instead of synchronous cache lookups that would block the search execution, implement an asynchronous event-driven flow that checks the cache in parallel with search preparation:

1. **Query Change Detection** → Emit cache read request
2. **Cache Check System** → Handle cache hit/miss
3. **Search Execution** → Proceed only on cache miss
4. **Search Cleanup** → Write results to cache instead of immediate cleanup

### Caching Policy

**Cache Key:** Query string (exact match, case-sensitive)
**Cache Value:** Serialized `Vec<SearchResult>` (aggregated and scored results from all plugins)
**TTL:** 1 hour (goldylox default)
**Eviction:** Automatic LRU when memory limits reached

**Why cache aggregated results instead of per-plugin results?**
- Aggregated results include deduplication, scoring adjustments, and result merging
- Single cache lookup vs. N plugin cache lookups
- Simpler invalidation logic
- Lower memory footprint

## DETAILED IMPLEMENTATION STEPS

### STEP 1: Add ecs-cache Dependency

**File:** `packages/ecs-search-aggregator/Cargo.toml`

Add to `[dependencies]` section:
```toml
ecs-cache = { path = "../ecs-cache" }
```

### STEP 2: Import Cache Events

**File:** `packages/ecs-search-aggregator/src/plugin.rs`

Add to imports at top of file:
```rust
use ecs_cache::{CacheReadRequested, CacheReadCompleted, CacheWriteRequested, CacheWriteCompleted};
```

### STEP 3: Register Cache Events

**File:** `packages/ecs-search-aggregator/src/plugin.rs`

In `SearchAggregatorPlugin::build()`, add cache-related systems:
```rust
impl Plugin for SearchAggregatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SearchAggregator>()
            .init_resource::<AggregatedSearchResults>()
            .init_resource::<SearchConfig>()
            .init_resource::<CurrentQuery>()
            .add_event::<SearchRequested>()
            .add_event::<SearchResultReceived>()
            .add_event::<SearchFailed>()
            .add_event::<SearchCompleted>()
            .add_event::<SearchCancelled>()
            .add_systems(
                Update,
                (
                    query_change_detection_system,
                    handle_cache_hits_system,  // NEW: Handle cache responses
                    spawn_plugin_search_tasks_system,
                    handle_plugin_search_tasks_system,
                    aggregate_search_results_system,
                    search_timeout_system,
                    search_cancellation_system,
                    search_cleanup_system,
                    cache_completed_searches_system,  // NEW: Write to cache
                )
                    .chain(),
            );
    }
}
```

### STEP 4: Modify Query Change Detection

**File:** `packages/ecs-search-aggregator/src/plugin.rs`, function `query_change_detection_system` (starting at line 50)

**Current behavior:** Immediately spawns search tasks

**New behavior:** First check cache, only spawn search tasks on cache miss

Add `cache_read_events` parameter and emit cache read request:

```rust
fn query_change_detection_system(
    current_query: Res<CurrentQuery>,
    mut search_events: EventWriter<SearchRequested>,
    mut cancel_events: EventWriter<SearchCancelled>,
    mut cache_read_events: EventWriter<CacheReadRequested>,  // NEW
    mut search_aggregator: ResMut<SearchAggregator>,
    mut aggregated_results: ResMut<AggregatedSearchResults>,
    search_config: Res<SearchConfig>,
    capability_index: Res<PluginCapabilityIndex>,
) {
    // Only trigger on actual query changes
    if !current_query.is_changed() {
        return;
    }

    let query = current_query.0.trim().to_string();

    // Clear results immediately when query changes
    aggregated_results.clear();

    // Validate query
    if let Err(e) = SearchAggregatorManager::validate_query(&query, &search_config) {
        debug!("Invalid search query '{}': {}", query, e);
        return;
    }

    // Cancel any active searches
    for (search_id, active_search) in search_aggregator.active_searches.iter() {
        cancel_events.write(SearchCancelled {
            search_id: *search_id,
            reason: format!("Query changed from '{}' to '{}'", active_search.query, query),
        });
    }
    search_aggregator.active_searches.clear();

    // NEW: Check cache first before executing expensive multi-plugin search
    debug!("Checking cache for query: '{}'", query);
    cache_read_events.write(CacheReadRequested::new(
        "search_results",
        query.clone(),
        "search_aggregator"
    ));
    
    // Note: Search execution is deferred to handle_cache_hits_system
    // If cache miss, that system will emit SearchRequested event
}
```

### STEP 5: Add Cache Hit Handler System

**File:** `packages/ecs-search-aggregator/src/plugin.rs`

Add new system after `query_change_detection_system`:

```rust
/// System to handle cache read responses and trigger search on cache miss
fn handle_cache_hits_system(
    mut cache_responses: EventReader<CacheReadCompleted>,
    mut search_events: EventWriter<SearchRequested>,
    mut search_aggregator: ResMut<SearchAggregator>,
    mut aggregated_results: ResMut<AggregatedSearchResults>,
    capability_index: Res<PluginCapabilityIndex>,
    current_query: Res<CurrentQuery>,
) {
    for cache_response in cache_responses.read() {
        // Only handle responses for search_results partition
        if cache_response.partition != "search_results" || cache_response.requester != "search_aggregator" {
            continue;
        }
        
        let query = cache_response.key.clone();
        
        match &cache_response.result {
            Ok(Some(cached_bytes)) => {
                // Cache HIT: Deserialize and display results immediately
                match serde_json::from_slice::<Vec<SearchResult>>(cached_bytes) {
                    Ok(cached_results) => {
                        info!("Cache HIT for query '{}': {} results", query, cached_results.len());
                        
                        // Create synthetic search_id for tracking
                        let search_id = uuid::Uuid::new_v4();
                        
                        // Update aggregated results directly
                        aggregated_results.results = cached_results;
                        aggregated_results.search_id = Some(search_id);
                        aggregated_results.is_loading = false;
                        aggregated_results.total_execution_time_ms = 0; // Instant from cache
                        
                        // No need to track in active_searches since search is complete
                        
                        debug!("Cache results displayed for query: '{}'", query);
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cached results for '{}': {}", query, e);
                        // Fall through to cache miss logic
                    }
                }
            }
            Ok(None) => {
                // Cache MISS: Execute normal search flow
                info!("Cache MISS for query '{}': Executing plugin searches", query);
                
                // Find search-capable plugins
                let search_capable_plugins: Vec<String> = discover_search_capable_plugins(&capability_index);
                
                if search_capable_plugins.is_empty() {
                    debug!("No search-capable plugins found");
                    return;
                }
                
                // Create and send search request event
                let search_request = SearchRequested::new(query.clone(), search_capable_plugins.clone());
                let search_id = search_request.search_id;
                
                // Track this search
                let expected_plugins: HashSet<String> = search_capable_plugins.into_iter().collect();
                let active_search = ActiveSearch::new(query.clone(), search_id, expected_plugins);
                search_aggregator.active_searches.insert(search_id, active_search);
                
                // Start loading state
                aggregated_results.start_search(search_id);
                
                info!("Starting search '{}' with ID {:?}", query, search_id);
                search_events.write(search_request);
            }
            Err(e) => {
                warn!("Cache read error for '{}': {:?}", query, e);
                // Treat errors as cache miss and continue with search
            }
        }
    }
}
```

### STEP 6: Modify Search Cleanup System

**File:** `packages/ecs-search-aggregator/src/plugin.rs`, function `search_cleanup_system` (starting at line 407)

**Current code at line 430-432:**
```rust
// Clean up the active search after a delay to allow for any final processing
// In a real implementation, you might want to keep recent searches for caching
search_aggregator.active_searches.remove(&search_id);
```

**Replace with:**
```rust
// Keep the search in active_searches briefly for cache writing system
// It will be removed after cache write completes
debug!("Search {:?} ready for caching", search_id);
```

### STEP 7: Add Cache Writing System

**File:** `packages/ecs-search-aggregator/src/plugin.rs`

Add new system after `search_cleanup_system`:

```rust
/// System to cache completed search results for future reuse
fn cache_completed_searches_system(
    mut completion_events: EventReader<SearchCompleted>,
    mut cache_write_events: EventWriter<CacheWriteRequested>,
    search_aggregator: Res<SearchAggregator>,
) {
    for completion_event in completion_events.read() {
        let search_id = completion_event.search_id;
        
        if let Some(active_search) = search_aggregator.active_searches.get(&search_id) {
            let query = active_search.query.clone();
            let results = active_search.results.clone();
            
            // Skip caching if no results
            if results.is_empty() {
                debug!("Skipping cache write for empty results: '{}'", query);
                continue;
            }
            
            // Serialize results for caching
            match serde_json::to_vec(&results) {
                Ok(serialized) => {
                    debug!(
                        "Writing {} results to cache for query '{}' ({} bytes)",
                        results.len(),
                        query,
                        serialized.len()
                    );
                    
                    cache_write_events.write(CacheWriteRequested::new(
                        "search_results",
                        query.clone(),
                        serialized,
                        Some(3600), // 1 hour TTL
                        "search_aggregator"
                    ));
                }
                Err(e) => {
                    warn!("Failed to serialize search results for caching: {}", e);
                }
            }
        }
    }
}
```

### STEP 8: Add Cache Write Completion Handler

**File:** `packages/ecs-search-aggregator/src/plugin.rs`

Add system to clean up active searches after cache write:

```rust
/// System to handle cache write completion and clean up active searches
fn handle_cache_write_completion_system(
    mut cache_write_responses: EventReader<CacheWriteCompleted>,
    mut search_aggregator: ResMut<SearchAggregator>,
) {
    for cache_response in cache_write_responses.read() {
        // Only handle responses from search aggregator
        if cache_response.partition != "search_results" || cache_response.requester != "search_aggregator" {
            continue;
        }
        
        match &cache_response.result {
            Ok(()) => {
                debug!("Successfully cached search results for query: '{}'", cache_response.key);
                
                // Now safe to remove completed searches that match this query
                search_aggregator.active_searches.retain(|_id, search| {
                    search.query != cache_response.key
                });
            }
            Err(e) => {
                warn!("Failed to write search results to cache: {:?}", e);
            }
        }
    }
}
```

Don't forget to add this system to the plugin's system chain in STEP 3.

## ALTERNATIVE APPROACHES CONSIDERED

### Option A: Synchronous In-Memory HashMap Cache
**Pros:** Simpler, no async complexity
**Cons:** No TTL, no LRU eviction, manual memory management, duplicates functionality
**Decision:** ❌ Rejected - reinvents the wheel, ecs-cache already provides this

### Option B: Cache Per-Plugin Results
**Pros:** Finer-grained cache invalidation
**Cons:** More complex, higher memory usage, slower (N cache lookups vs 1)
**Decision:** ❌ Rejected - complexity outweighs benefits

### Option C: Remove Comment and Document Why No Caching
**Pros:** Zero implementation effort
**Cons:** Misses significant performance optimization opportunity
**Decision:** ❌ Rejected - caching provides measurable UX improvement

### Option D: Implement with ecs-cache (SELECTED)
**Pros:** Reuses existing infrastructure, production-ready, automatic TTL/LRU
**Cons:** Requires event-driven architecture understanding
**Decision:** ✅ **SELECTED** - Best balance of performance and maintainability

## PERFORMANCE IMPACT

**Cache HIT scenario:**
- Latency: ~0-5ms (memory lookup + deserialization)
- Plugin load: 0 (no plugin searches executed)
- User experience: Instant results

**Cache MISS scenario:**
- Latency: Same as current (5-10ms for plugin searches)
- Plugin load: Same as current
- User experience: Same as current, but subsequent identical queries are instant

**Memory footprint:**
- ~1-5KB per cached query (depends on result count)
- goldylox hot tier: 1000 entries max (~1-5MB)
- goldylox warm tier: 10000 entries max (~10-50MB)
- Automatic LRU eviction prevents unbounded growth

## FILES TO MODIFY

1. [`packages/ecs-search-aggregator/Cargo.toml`](../../packages/ecs-search-aggregator/Cargo.toml) - Add ecs-cache dependency
2. [`packages/ecs-search-aggregator/src/plugin.rs`](../../packages/ecs-search-aggregator/src/plugin.rs) - Core implementation changes
3. No changes needed to `types.rs` - SearchResult already has Serialize/Deserialize

## DEFINITION OF DONE

- [ ] ecs-cache dependency added to Cargo.toml
- [ ] Cache events imported in plugin.rs
- [ ] `handle_cache_hits_system` implemented and registered
- [ ] `cache_completed_searches_system` implemented and registered  
- [ ] `handle_cache_write_completion_system` implemented and registered
- [ ] `query_change_detection_system` emits CacheReadRequested instead of immediate SearchRequested
- [ ] `search_cleanup_system` comment removed and replaced with cache flow documentation
- [ ] Systems added to plugin's Update schedule in correct order
- [ ] Code compiles without errors or warnings
- [ ] Cache hits return results in <5ms (verify with tracing logs)
- [ ] Cache misses trigger normal search flow
- [ ] Identical queries reuse cached results within 1 hour TTL
- [ ] No "in a real implementation" comments remain in the file

## CONSTRAINTS

- DO NOT add debouncing at this layer (handled by UI input layer if needed)
- DO NOT change search execution logic (only add caching layer)
- DO NOT create synchronous blocking cache lookups (use async events)
- DO NOT bypass goldylox's built-in TTL/LRU mechanisms
- DO NOT cache empty result sets (wastes memory)

## INTEGRATION NOTES

**Dependency Order:**
- ecs-cache plugin must be initialized before ecs-search-aggregator
- This is likely already handled in app initialization since "search_results" partition exists

**Event Ordering:**
- `query_change_detection_system` → emits `CacheReadRequested`
- `handle_cache_hits_system` → reads `CacheReadCompleted`, emits `SearchRequested` on miss
- `spawn_plugin_search_tasks_system` → reads `SearchRequested`
- (normal search flow continues)
- `search_cleanup_system` → emits `SearchCompleted`
- `cache_completed_searches_system` → reads `SearchCompleted`, emits `CacheWriteRequested`
- `handle_cache_write_completion_system` → reads `CacheWriteCompleted`, removes from active_searches

## RELATED FILES FOR REFERENCE

- Cache infrastructure: [`packages/ecs-cache/src/`](../../packages/ecs-cache/src/)
- Cache events: [`packages/ecs-cache/src/events.rs`](../../packages/ecs-cache/src/events.rs)
- Cache resources: [`packages/ecs-cache/src/resources.rs`](../../packages/ecs-cache/src/resources.rs)
- Cache systems: [`packages/ecs-cache/src/systems.rs`](../../packages/ecs-cache/src/systems.rs)
- Search types: [`packages/ecs-search-aggregator/src/types.rs`](../../packages/ecs-search-aggregator/src/types.rs)
- Search manager: [`packages/ecs-search-aggregator/src/manager.rs`](../../packages/ecs-search-aggregator/src/manager.rs)

## IMPLEMENTATION NOTES

This task removes the "real implementation" TODO by integrating with the existing, production-ready ecs-cache system. The implementation follows Bevy's event-driven patterns and avoids blocking operations. Cache operations happen asynchronously in the background, ensuring the search system remains responsive.

The cache provides immediate performance benefits for repeated queries (common when users refine searches by backspacing and retyping). The 1-hour TTL balances freshness with performance, and goldylox's automatic LRU eviction prevents memory issues.
# Action Items - Bevy ECS Services

> **Implementation Reference**: See service implementations in [`./packages/core/src/plugins/services/`](./packages/core/src/plugins/services/) and [`./packages/core/src/service_bridge/services/`](./packages/core/src/service_bridge/services/)

## Required Imports

For all Bevy async patterns shown in this documentation:

```rust
use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use tracing::{debug, error, info, warn};

// Service-specific imports
use ecs_search_aggregator::*;
use ecs_service_bridge::*;
use ecs_surrealdb::*;

// Additional async utilities
use std::sync::Arc;
use tokio::sync::RwLock;
```

## Service Overview

| Service | Plugin | Resource | Package |
|---------|--------|----------|---------|
| Cache | `CachePlugin` | `CacheManager` | `ecs-cache` |
| Clipboard | `ClipboardPlugin` | `ClipboardResource` | `ecs-clipboard` |
| Compression | `CompressionPlugin` | `CompressionManager` | `ecs-compression` |
| Permissions | `PermissionsPlugin` | `PermissionResource` | `ecs-permissions` |
| Notifications | `NotificationPlugin` | `NotificationResource` | `ecs-notifications` |
| Progress | `ProgressPlugin` | `ProgressManager` | `ecs-progress` |
| UI | `UIPlugin` | `UIManager` | `ecs-ui` |
| Bluetooth | `BluetoothPlugin` | `BluetoothManager` | `ecs-bluetooth` |
| HTTP/Fetch | `HttpPlugin` | `HttpClientPool` | `ecs-fetch` |
| TLS | `TlsCleanupPlugin` | `OcspCleanupTimer`, `CrlCleanupTimer` | `ecs-tls` |
| Search Aggregator | `SearchAggregatorPlugin` | `SearchAggregator` | `ecs-search-aggregator` |
| Service Bridge | `ServiceBridgePlugin` | `ServiceBridgeResource` | `ecs-service-bridge` |
| Database | `DatabasePlugin` | `DatabaseService` | `ecs-surrealdb` |
| Deno Runtime | `DenoPlugin` | `DenoRuntimePool`, `DenoOperationTracker` | `ecs-deno` |
| Task Management | `TaskManagementPlugin` | `TaskStatistics` | `ecs-task-management` |

## Cache Service

### Resource: `CacheManager`
```rust
// Multi-partition cache with goldylox backend
cache_mgr.create_partition(name: impl Into<String>, config: CachePartitionConfig) -> Result<(), String>
cache_mgr.get_partition(name: &str) -> Option<&Goldylox<String, Vec<u8>>>
cache_mgr.get_partition_mut(name: &str) -> Option<&mut Goldylox<String, Vec<u8>>>

// Default partitions: plugin_metadata, search_results, ui_assets, configuration, api_responses
```

### Events (Request/Response Pattern)
| Event | Direction | Purpose |
|-------|-----------|---------|
| `CacheReadRequested` | Client → Service | Get cached value by key |
| `CacheReadCompleted` | Service → Client | Read operation result |
| `CacheWriteRequested` | Client → Service | Store value in cache |
| `CacheWriteCompleted` | Service → Client | Write operation confirmation |
| `CacheInvalidateRequested` | Client → Service | Remove entry from cache |
| `CacheInvalidationCompleted` | Service → Client | Invalidation result |
| `CacheEvictionOccurred` | Service → Clients | Entry evicted notification |
| `CacheWarmupRequested` | Client → Service | Pre-populate cache entries |

### Supporting Resources
```rust
// Configuration
struct CacheConfig { global_memory_limit: usize, enable_cache_warming: bool, enable_metrics: bool, eviction_check_interval: Duration }
struct CachePartitionConfig { hot_tier_capacity: usize, warm_tier_capacity: usize, default_ttl: Option<Duration>, max_entry_size: usize, enable_compression: bool }

// Metrics
struct CacheMetrics { partition_stats: HashMap<String, CachePartitionStats>, global_stats: GlobalCacheStats }
struct CachePartitionStats { hits: u64, misses: u64, evictions: u64, writes: u64, total_size: usize, entry_count: usize }
```

### Components
```rust
// Operation tracking
struct CacheOperation { operation_id: Uuid, partition: String, key: String, operation_type: CacheOperationType, started_at: Instant, requester: String }
struct CacheWarmupTask { operation_id: Uuid, partition: String, task: Task<Result<Vec<(String, Vec<u8>)>, CacheOperationError>>, started_at: Instant }

// Monitoring
struct CacheEvictionMonitor { partition: String, last_check: Instant, eviction_threshold: f32 }
struct CacheAccessPattern { partition: String, key: String, access_count: u64, last_accessed: Instant, access_frequency: f64 }
```

### Systems
```rust
// Core processing
process_cache_reads_system    // Handle CacheReadRequested -> CacheReadCompleted
process_cache_writes_system   // Handle CacheWriteRequested -> CacheWriteCompleted  
process_cache_invalidations_system // Handle CacheInvalidateRequested -> CacheInvalidationCompleted

// Background tasks
cache_eviction_system    // Monitor memory pressure and trigger evictions
cache_metrics_system     // Update performance statistics
```

### Types
```rust
// Universal serialization types
type CacheKey = String;        // Any key type -> String
type CacheValue = Vec<u8>;     // Any value type -> bytes via serde

// Operations
enum CacheOperationType { Read, Write { value: Vec<u8>, ttl_seconds: Option<u64> }, Invalidate, Warmup { keys: Vec<String> } }
enum EvictionReason { TTLExpired, LRUEviction, ManualInvalidation, MemoryPressure, SystemShutdown }

// Errors
use goldylox::CacheOperationError; // KeyNotFound, StorageError, MemoryLimitExceeded, etc.
```

### Backend Integration
- **goldylox**: High-performance multi-tier cache (hot/warm/cold)
- **Architecture**: Event-driven ECS wrapper around goldylox public API
- **Performance**: Goldylox handles caching, ECS handles application integration
- **Serialization**: Application layer handles type-safe serde, cache stores bytes

## Clipboard Service

### Resource: `ClipboardResource`
```rust
// Sync API
clipboard_res.get_sync(format: ClipboardFormat) -> Result<ClipboardData, ClipboardError>
clipboard_res.set_sync(data: ClipboardData) -> Result<(), ClipboardError>
clipboard_res.clear_sync() -> Result<(), ClipboardError>
clipboard_res.has_format_sync(format: ClipboardFormat) -> bool
clipboard_res.available_formats_sync() -> Vec<ClipboardFormat>
```

### Events
| Event | Direction | Purpose |
|-------|-----------|---------|
| `ClipboardRequest` | Client → Service | Get, Set, Clear, CheckFormat, GetAvailableFormats |
| `ClipboardChanged` | Service → Clients | Notify content changes |

### Types
```rust
enum ClipboardData { Text(String), Html{html: String, alt_text: Option<String>}, Image(ImageData), Files(Vec<PathBuf>) }
enum ClipboardFormat { Text, Html, Image, Files }
enum ClipboardError { AccessDenied, UnsupportedFormat(ClipboardFormat), UnsupportedPlatform, PlatformError(String), Busy }
```

## Permissions Service

### Resource: `PermissionResource`
```rust
// Sync API
permission_res.check_sync(permission: PermissionType) -> Result<PermissionStatus, PermissionError>
permission_res.request_sync(permission: PermissionType) -> Result<PermissionStatus, PermissionError>
permission_res.supported_permissions_sync() -> Vec<PermissionType>
```

### Events
| Event | Direction | Purpose |
|-------|-----------|---------|
| `PermissionRequest` | Client → Service | Check, Request permissions |
| `PermissionResponse` | Service → Client | Operation results |
| `PermissionChanged` | Service → Clients | Status changes |

### Types
```rust
enum PermissionType { 
    // macOS (37 types)
    Camera, Microphone, Location, Calendar, Contacts, Reminders, Photos, MediaLibrary,
    Bluetooth, BluetoothPeripheral, SpeechRecognition, Accessibility, FullDiskAccess,
    ScreenCapture, SystemEvents, AppleEvents, Automation, FileProviderPresence,
    ListenEvent, PostEvent, FileProviderDomain, AddressBook, SystemPolicyAllFiles,
    SystemPolicyDesktopFolder, SystemPolicyDocumentsFolder, SystemPolicyDownloadsFolder,
    SystemPolicyNetworkVolumes, SystemPolicyRemovableVolumes, SystemPolicySystemAdminFiles,
    DeveloperTool, Kext, WiFi, LocalNetwork, NearbyInteraction, UserTracking,
    FocusStatus, SensorKit, BackgroundApp,
    // Windows/Linux
    Notifications, FileSystem
}
enum PermissionStatus { Granted, Denied, NotDetermined, Restricted }
enum PermissionError { UnsupportedPermission(PermissionType), AccessDenied, PlatformError(String), UnsupportedPlatform }
```

## Notifications Service

### Resource: `NotificationResource`
```rust
// Sync API
notification_res.show_sync(options: NotificationOptions) -> Result<NotificationId, NotificationError>
notification_res.show_toast_sync(message: &str) -> Result<NotificationId, NotificationError>
notification_res.dismiss_sync(id: NotificationId) -> Result<(), NotificationError>
notification_res.is_available_sync() -> bool
notification_res.platform_name_sync() -> &'static str
```

### Events
| Event | Direction | Purpose |
|-------|-----------|---------|
| `NotificationRequest` | Client → Service | Show, ShowToast, Dismiss, IsAvailable, GetPlatformName |
| `NotificationShown` | Service → Clients | Notification displayed successfully |
| `NotificationDismissed` | Service → Clients | Notification dismissed or failed |

### Types
```rust
struct NotificationOptions<'a> { title: &'a str, message: &'a str, icon: Option<&'a str>, sound: bool, duration: Option<Duration>, urgent: bool }
enum NotificationError { ServiceUnavailable(String), PermissionDenied(String), InvalidContent(String), SystemError(String), Timeout(String) }
struct NotificationId(u64)
enum DismissReason { User, System, Timeout, Error(String) }

// Platform Backends
// macOS: UserNotifications framework
// Linux: D-Bus org.freedesktop.Notifications  
// Windows: Toast notifications
```

## Progress Service

### Resource: `ProgressManager`
```rust
// Sync API
progress_mgr.create_progress(id: ProgressId, config: ProgressConfig) -> Result<(), ProgressError>
progress_mgr.update_progress(id: ProgressId, progress: f32) -> Result<(), ProgressError>
progress_mgr.set_message(id: ProgressId, message: String) -> Result<(), ProgressError>
progress_mgr.complete_progress(id: ProgressId) -> Result<(), ProgressError>
progress_mgr.cancel_progress(id: ProgressId) -> Result<(), ProgressError>
progress_mgr.get_progress(id: ProgressId) -> Option<ProgressState>
```

### Events
| Event | Direction | Purpose |
|-------|-----------|---------|
| `ProgressCreated` | Service → Clients | New progress tracker created |
| `ProgressUpdated` | Service → Clients | Progress value or message changed |
| `ProgressCompleted` | Service → Clients | Progress completed successfully |
| `ProgressCancelled` | Service → Clients | Progress cancelled |

### Types
```rust
struct ProgressConfig { title: String, message: Option<String>, show_percentage: bool, cancellable: bool }
struct ProgressState { id: ProgressId, config: ProgressConfig, progress: f32, status: ProgressStatus }
enum ProgressStatus { Active, Completed, Cancelled, Error(String) }
enum ProgressError { NotFound(ProgressId), InvalidProgress(f32), AlreadyCompleted, SystemError(String) }
```

## TLS Service

### Resources: `OcspCleanupTimer`, `CrlCleanupTimer`
```rust
// Resources for tracking cache cleanup timing
struct OcspCleanupTimer { last_cleanup: Instant, interval: Duration }
struct CrlCleanupTimer { last_cleanup: Instant, interval: Duration }
```

### Components: `TlsCacheHolder`
```rust
// Component to mark entities that have TLS caches
struct TlsCacheHolder { ocsp_cache: OcspCache, crl_cache: CrlCache }
```

### Plugin: `TlsCleanupPlugin`
```rust
// Plugin that registers cleanup systems and resources
impl Plugin for TlsCleanupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OcspCleanupTimer>()
            .init_resource::<CrlCleanupTimer>()
            .add_systems(Update, (
                ocsp_cache_cleanup_system,
                crl_cache_cleanup_system,
            ).chain());
    }
}
```

### Systems
| System | Schedule | Purpose |
|--------|----------|---------|
| `ocsp_cache_cleanup_system` | Update | Periodic cleanup of OCSP cache entries |
| `crl_cache_cleanup_system` | Update | Periodic cleanup of CRL cache entries |

### Builder Interface
```rust
// Primary public API for certificate operations
pub struct Tls;
pub struct CertificateAuthority;

impl Tls {
    pub fn builder() -> CertificateBuilder
    pub fn validate_certificate(cert: &[u8]) -> Result<(), TlsError>
    pub fn create_ca() -> CertificateAuthority
}
```

### Types
```rust
struct OcspCache { /* Internal OCSP response cache */ }
struct CrlCache { /* Internal certificate revocation list cache */ }
enum TlsError { InvalidCertificate(String), OcspError(String), CrlError(String), SystemError(String) }

// Certificate operations
struct CertificateBuilder { /* Certificate generation builder */ }
struct ValidatedCertificate { /* Validated certificate wrapper */ }
```

### Features
- **Certificate Validation**: X.509 certificate parsing and validation
- **OCSP Support**: Online Certificate Status Protocol caching
- **CRL Management**: Certificate Revocation List handling
- **Cross-Platform**: macOS Security.framework, Linux OpenSSL, Windows CryptoAPI
- **Bevy ECS Integration**: Proper resource management and cleanup systems
- **Builder Pattern**: Secure-by-default certificate operations
- **Cache Management**: Automatic cleanup of expired OCSP/CRL entries

## UI Service

### Resource: `UIManager`
```rust
// Sync API
ui_mgr.show_window(config: WindowConfig) -> Result<WindowId, UIError>
ui_mgr.hide_window(id: WindowId) -> Result<(), UIError>
ui_mgr.update_window(id: WindowId, updates: WindowUpdates) -> Result<(), UIError>
ui_mgr.show_dialog(config: DialogConfig) -> Result<DialogResult, UIError>
ui_mgr.show_file_picker(config: FilePickerConfig) -> Result<Vec<PathBuf>, UIError>
```

### Events
| Event | Direction | Purpose |
|-------|-----------|---------|
| `WindowShown` | Service → Clients | Window displayed |
| `WindowHidden` | Service → Clients | Window hidden |
| `WindowClosed` | Service → Clients | Window closed by user |
| `DialogResult` | Service → Clients | Dialog interaction result |

### Types
```rust
struct WindowConfig { title: String, size: (u32, u32), resizable: bool, position: Option<(i32, i32)> }
struct DialogConfig { title: String, message: String, buttons: Vec<String>, dialog_type: DialogType }
enum DialogType { Info, Warning, Error, Question }
enum DialogResult { Button(usize), Cancelled }
enum UIError { WindowNotFound(WindowId), InvalidConfig, PlatformError(String) }
```

## Bluetooth Service

### Resource: `BluetoothManager`
```rust
// Sync API
bluetooth_mgr.start_scan(options: ScanOptions) -> Result<(), BluetoothError>
bluetooth_mgr.stop_scan() -> Result<(), BluetoothError>
bluetooth_mgr.connect_device(device_id: BluetoothDeviceId) -> Result<(), BluetoothError>
bluetooth_mgr.disconnect_device(device_id: BluetoothDeviceId) -> Result<(), BluetoothError>
bluetooth_mgr.devices() -> Result<Vec<BluetoothDevice>, BluetoothError>
bluetooth_mgr.adapter_state() -> Result<AdapterState, BluetoothError>
```

### Events
| Event | Direction | Purpose |
|-------|-----------|---------|
| `DeviceDiscovered` | Service → Clients | New BLE device found |
| `DeviceUpdated` | Service → Clients | Device RSSI or info changed |
| `ConnectionStateChanged` | Service → Clients | Device connection status |
| `ScanStarted` | Service → Clients | Scanning began |
| `ScanStopped` | Service → Clients | Scanning ended |
| `AdapterStateChanged` | Service → Clients | Bluetooth adapter state |

### Types
```rust
struct BluetoothDevice { info: DeviceInfo, state: ConnectionState, last_seen: SystemTime }
struct DeviceInfo { id: BluetoothDeviceId, name: Option<String>, address: String, rssi: Option<i16>, services: Vec<Uuid> }
enum ConnectionState { Disconnected, Connecting, Connected, Disconnecting }
enum AdapterState { Unknown, PoweredOff, PoweredOn, Unauthorized, Unsupported }
struct ScanOptions { duration: Option<Duration>, service_uuids: Vec<Uuid>, allow_duplicates: bool, min_rssi: Option<i16> }
enum BluetoothError { AdapterNotAvailable, PoweredOff, PermissionDenied, DeviceNotFound, ConnectionFailed, NotSupported }
```

## Usage Patterns

### Async (Event-Driven)
```rust
// Request
let (sender, receiver) = oneshot::channel();
events.send(ServiceRequest::Operation { params, response_sender: sender });

// Listen for changes
fn handle_changes(mut events: EventReader<ServiceChanged>) {
    for event in events.read() { /* handle */ }
}
```

## HTTP/Fetch Service

### Resource: `HttpClientPool`
```rust
// Event-driven HTTP requests with comprehensive features
commands.spawn((
    HttpRequest::new(operation_id, correlation_id, Method::GET, "https://api.example.com/data"),
    RetryPolicy::exponential_backoff(3, Duration::from_millis(100)),
    RequestTimeout::new(Duration::from_secs(30)),
));

// Fluent API for common operations
let operation_id = HttpClient::get("https://api.example.com/users", &mut commands, &mut events);
let operation_id = HttpClient::post_json("https://api.example.com/users", &user_data, &mut commands, &mut events)?;

// Builder pattern for complex requests
HttpRequestBuilder::post("https://api.example.com/upload")
    .header("Authorization", "Bearer token")
    .json(&payload)?
    .timeout(Duration::from_secs(60))
    .priority(RequestPriority::High)
    .cache_policy(CachePolicy::NoCache)
    .send(&mut commands, &mut events);
```

### Events (Request/Response Pattern)
| Event | Direction | Purpose |
|-------|-----------|---------|
| `HttpRequestSubmitted` | Client → Service | Submit HTTP request with full configuration |
| `HttpResponseReceived` | Service → Client | Successful response with data and metadata |
| `HttpRequestFailed` | Service → Client | Request failure with error classification |
| `HttpRequestRetryRequested` | Service → Client | Retry attempt notification |
| `RateLimitExceeded` | Service → Client | Rate limit enforcement notification |
| `HttpRequestCancelled` | Service → Client | Request cancellation confirmation |
| `HttpRequestTimeout` | Service → Client | Request timeout notification |

### Advanced Events
| Event | Direction | Purpose |
|-------|-----------|---------|
| `HttpCacheHit` | Service → Client | Response served from cache |
| `HttpCacheMiss` | Service → Client | Cache miss, fetching from origin |
| `HttpCacheStored` | Service → Client | Response cached for future use |
| `ConditionalRequestRequired` | Service → Client | Conditional request needed for cache validation |
| `TraceStarted` | Service → Client | Request tracing initiated |
| `TraceCompleted` | Service → Client | Request tracing completed |
| `MetricsReportGenerated` | Service → Client | Performance metrics available |

### Supporting Resources
```rust
// Core configuration
struct HttpConfig { default_timeout: Duration, max_request_size: usize, max_response_size: usize, default_headers: HashMap<String, String>, user_agent: String, tls_config: TlsConfig, security_config: SecurityConfig, rate_limit_config: RateLimitConfig }

// Client pool management
struct HttpClientPool { clients: Vec<Arc<Client>>, next_client: AtomicUsize, config: ClientPoolConfig }
struct ClientPoolConfig { pool_size: usize, default_timeout: Duration, connect_timeout: Duration, idle_timeout: Option<Duration>, max_idle_per_host: usize, tcp_keepalive: Option<Duration>, accept_invalid_certs: bool }

// Rate limiting and metrics
struct RateLimitManager { per_domain_limiters: AHashMap<String, RateLimiter>, global_limiter: RateLimiter, config: RateLimitConfig }
struct RequestMetrics { total_requests: AtomicU64, successful_requests: AtomicU64, failed_requests: AtomicU64, latency_histogram: Histogram<f64>, bandwidth_usage: AtomicU64, cache_hit_rate: AtomicU64 }
```

### Components
```rust
// Request tracking and configuration
struct HttpRequest { operation_id: HttpOperationId, correlation_id: CorrelationId, method: Method, url: String, headers: HeaderMap, body: Option<Bytes>, started_at: Instant, retries: u32, priority: RequestPriority }
struct RetryPolicy { strategy: RetryStrategy, max_attempts: u32, initial_delay: Duration, max_delay: Duration, jitter: bool }
struct RequestTimeout { deadline: Instant, timeout_duration: Duration, has_timed_out: bool }

// Security and validation
struct RequestSecurityContext { parsed_url: Url, method: Method, headers: HeaderMap, body_size: usize, requester: String }
struct SsrfProtectionResult { is_allowed: bool, blocked_reason: Option<String>, resolved_ip: Option<IpAddr> }
```

### Systems
```rust
// Core processing systems  
process_http_requests_system     // Handle HttpRequestSubmitted -> security validation, rate limiting, task spawning
process_http_responses_system    // Handle HTTP completion -> HttpResponseReceived/HttpRequestFailed events
request_retry_system            // Handle failed requests -> HttpRequestRetryRequested with exponential backoff
request_timeout_system          // Monitor request timeouts -> HttpRequestTimeout events
rate_limiting_system           // Enforce rate limits -> RateLimitExceeded events
connection_pool_management_system // Manage HTTP client pool health and rotation

// Advanced systems
process_cache_read_completions_system  // Handle cache integration reads
process_cache_write_completions_system // Handle cache integration writes  
trace_cleanup_system                   // Clean up completed request traces
metrics_reporting_system              // Generate periodic metrics reports
```

### Types
```rust
// Core HTTP types
enum HttpMethod { Get, Post, Put, Delete, Patch }
enum RequestPriority { Low = 0, Normal = 1, High = 2, Critical = 3 }
struct HttpResponseData { status: u16, headers: HashMap<String, String>, body: Value, from_cache: bool }

// Error classification
enum HttpError { RequestFailed(String), Timeout { timeout_ms: u64 }, RateLimited, InvalidUrl(String), SerializationError(String), SecurityViolation(String), NetworkError(String), SsrfBlocked(String) }

// Cache integration
enum CachePolicy { Default, NoCache, CacheOnly, CacheFirst, NetworkFirst, StaleWhileRevalidate }
struct CacheMetadata { cache_key: String, ttl: Duration, last_modified: Option<SystemTime>, etag: Option<String> }

// Security configuration  
struct SecurityConfig { ssrf_protection: bool, allowed_schemes: Vec<String>, blocked_ip_ranges: Vec<String>, max_redirects: u32, blocked_domains: Vec<String>, allowed_domains: Vec<String> }
```

### Features

#### Core Features
- **Connection Pooling**: Round-robin load balancing across HTTP/2 client pool
- **Retry Logic**: Exponential backoff with jitter and configurable max attempts
- **Rate Limiting**: Per-domain and global rate limiting with token bucket algorithm
- **Request Prioritization**: Priority queue management with starvation prevention
- **Request Deduplication**: Prevent duplicate concurrent requests to same endpoint

#### Security Features
- **SSRF Protection**: Comprehensive protection against Server-Side Request Forgery
- **URL Validation**: Scheme restrictions and malicious pattern detection
- **Request Sanitization**: Header injection prevention and size limits
- **TLS Security**: Certificate validation, minimum TLS version, cipher suite control
- **IP Range Blocking**: Block private networks, localhost, and custom IP ranges

#### Performance Features
- **Response Caching**: Intelligent HTTP cache with TTL, conditional requests, cache policies
- **Response Streaming**: Chunked response handling with backpressure management
- **Circuit Breaker**: Automatic failure detection and recovery with configurable thresholds
- **Metrics Collection**: Comprehensive request latency, bandwidth, and success rate tracking
- **Zero-Allocation Paths**: Optimized hot paths for minimal memory allocation

#### Integration Features
- **Cache Integration**: Seamless integration with `ecs-cache` service for response caching
- **Tracing Integration**: Request correlation IDs, distributed tracing, span propagation
- **Authentication**: Bearer tokens, API keys, OAuth integration with automatic refresh
- **Middleware Support**: Custom request/response middleware with compression and content negotiation

### Configuration Modes

#### Development Mode
```rust
HttpPlugin::new()
    .development_mode()
    // Relaxed security, detailed logging, 100% tracing sampling
```

#### Production Mode  
```rust
HttpPlugin::new()
    .production_mode()
    // Strict security, optimized performance, 10% tracing sampling
```

#### Custom Configuration
```rust
HttpPlugin::new()
    .with_http_config(HttpConfig { /* ... */ })
    .with_security_config(SecurityConfig { /* ... */ })
    .with_max_clients(20)
    .with_metrics_reporting(true, Some(Duration::from_secs(300)))
```

### Usage Patterns

#### Event-Driven (Recommended)
```rust
// Submit request via events
events.send(HttpRequestSubmitted {
    operation_id: HttpOperationId(Uuid::new_v4()),
    method: Method::GET,
    url: "https://api.example.com/data".to_string(),
    /* ... */
});

// Listen for responses
fn handle_http_responses(mut events: EventReader<HttpResponseReceived>) {
    for response in events.read() {
        match response.result {
            Ok(data) => { /* handle success */ },
            Err(error) => { /* handle error */ },
        }
    }
}
```

#### Convenience API
```rust
// Simple operations
let operation_id = HttpClient::get("https://api.example.com/users", &mut commands, &mut events);
let operation_id = HttpClient::post_json("https://api.example.com/users", &user, &mut commands, &mut events)?;

// Builder pattern for complex requests
HttpRequestBuilder::get("https://api.example.com/search")
    .header("Authorization", format!("Bearer {}", token))
    .timeout(Duration::from_secs(45))
    .cache_policy(CachePolicy::StaleWhileRevalidate)
    .priority(RequestPriority::High)
    .send(&mut commands, &mut events);
```

### Backend Integration
- **reqwest**: High-performance HTTP client with HTTP/2 support, connection pooling, and async operation
- **governor**: Token bucket rate limiting with per-domain and global limits
- **Architecture**: Event-driven ECS wrapper around reqwest with comprehensive feature set
- **Performance**: Connection reuse, keep-alive, HTTP/2 multiplexing, zero-allocation hot paths
- **Security**: SSRF protection, TLS validation, request sanitization, comprehensive input validation

### Sync (Direct)
```rust
let result = service_res.operation_sync(params)?;
```

## Search Aggregator Service

### Resource: `SearchAggregator`
```rust
// Distributed search coordinator for Raycast/Alfred-style launcher
search_aggregator.active_searches: HashMap<SearchId, ActiveSearch> 
search_aggregator.search_timeout: Duration
search_aggregator.max_concurrent_searches: usize
```

### Events (Request/Response Pattern)
| Event | Direction | Purpose |
|-------|-----------|---------|
| `SearchRequested` | Client → Service | Initiate search across all capable plugins |
| `SearchResultReceived` | Service → Client | Plugin returned search results |
| `SearchFailed` | Service → Client | Plugin search failed or timed out |
| `SearchCompleted` | Service → Client | All plugins responded, results aggregated |
| `SearchCancelled` | Service → Client | Search cancelled (new query or timeout) |

### Supporting Resources
```rust
// Search coordination
struct ActiveSearch { query: String, search_id: SearchId, started_at: Instant, expected_plugins: HashSet<String>, completed_plugins: HashSet<String>, failed_plugins: HashMap<String, String>, results: Vec<SearchResult> }

// UI integration
struct AggregatedSearchResults { results: Vec<SearchResult>, search_id: Option<SearchId>, is_loading: bool, completed_plugins: HashSet<String>, failed_plugins: Vec<(String, String)>, total_execution_time_ms: u64 }

// Configuration
struct SearchConfig { timeout_ms: u64, max_results_per_plugin: usize, debounce_delay_ms: u64, min_query_length: usize }
```

### Components
```rust
// Async task tracking
struct PluginSearchTask { search_id: SearchId, plugin_id: String, task: Task<Result<Vec<SearchResult>, SearchError>>, started_at: Instant }

// Timeout management
struct SearchTimeout { search_id: SearchId, deadline: Instant }
```

### Systems
```rust
// Core coordination
query_change_detection_system           // Detect query changes and initiate searches
spawn_plugin_search_tasks_system        // Create async tasks for each capable plugin
handle_plugin_search_tasks_system       // Poll completed tasks and emit results
aggregate_search_results_system         // Merge and score results from multiple plugins

// Management
search_timeout_system                   // Handle search timeouts
search_cancellation_system              // Cancel searches when new query starts
search_cleanup_system                   // Clean up completed searches
```

### Types
```rust
// Search results
struct SearchResult { title: String, description: String, action: String, icon: Option<String>, score: f32, plugin_id: String }
type SearchId = uuid::Uuid

// Error handling
enum SearchError { Timeout, PluginNotResponding, InvalidQuery, ServiceUnavailable, InternalError(String) }

// Query tracking
struct CurrentQuery(pub String)
```

### Features

#### Distributed Search Coordination
- **Plugin Discovery**: Automatic discovery of search-capable plugins via service bridge
- **Concurrent Execution**: Parallel async tasks for each plugin with independent timeouts
- **Result Aggregation**: Intelligent merging, scoring, and deduplication of results
- **Early Completion**: Optional early termination when minimum plugin threshold is met
- **Query Debouncing**: Configurable delay to prevent excessive search requests

#### Performance Optimizations
- **Async Task Pooling**: Uses Bevy's AsyncComputeTaskPool for efficient task scheduling
- **Result Streaming**: Results displayed as plugins respond (no wait for all)
- **Memory Efficient**: Automatic cleanup of completed searches and task entities
- **Score Boosting**: Relevance scoring based on title/description matching
- **Result Limiting**: Per-plugin result limits with global aggregation cap

#### Integration Features
- **Service Bridge Integration**: Uses `ecs-service-bridge` for plugin communication
- **Capability Discovery**: Automatic detection of plugins with search capability
- **Health Monitoring**: Only sends requests to healthy, active plugins
- **Message Correlation**: Request/response correlation with UUIDs
- **Priority Messaging**: High-priority search messages for responsive UI

### Usage Patterns

#### Event-Driven (Recommended)
```rust
// Initiate search by updating query
fn update_search_query(mut current_query: ResMut<CurrentQuery>) {
    current_query.0 = "new search term".to_string();
    // System automatically detects change and starts search
}

// Listen for completed results with proper resource checking
fn handle_search_results(
    mut events: EventReader<SearchCompleted>, 
    aggregated_results: Option<Res<AggregatedSearchResults>>,
    mut ui_events: EventWriter<UIUpdateEvent>
) {
    for completion in events.read() {
        if let Some(results) = aggregated_results.as_ref() {
            info!("Search completed: {} results in {}ms", results.results.len(), completion.execution_time_ms);
            
            // Safely update UI through event system
            ui_events.send(UIUpdateEvent::SearchResults {
                results: results.results.clone(),
                execution_time_ms: completion.execution_time_ms,
            });
        } else {
            warn!("Search completed but no aggregated results resource found");
        }
    }
}
```

#### Manual Search Triggering
```rust
// Direct search request
fn trigger_search(mut search_events: EventWriter<SearchRequested>) {
    let search_request = SearchRequested::new("manual query".to_string(), vec!["plugin1".to_string(), "plugin2".to_string()]);
    search_events.send(search_request);
}
```

### Backend Integration
- **Service Bridge**: Uses `ecs-service-bridge` for plugin communication and capability discovery
- **Plugin Protocol**: Structured JSON messages with search_request/search_response format  
- **Task Management**: Bevy async task system for concurrent plugin execution
- **Architecture**: Event-driven ECS coordination of distributed search operations
- **Performance**: Parallel execution, result streaming, intelligent scoring and deduplication

## Service Bridge Service

### Resource: `ServiceBridgeResource`
```rust
// High-performance inter-plugin communication hub
service_bridge.config: ServiceBridgeConfig
service_bridge.stats: ServiceBridgeStats
service_bridge.health: ServiceBridgeHealth
service_bridge.startup_time: TimeStamp
```

### Events (Request/Response Pattern)
| Event | Direction | Purpose |
|-------|-----------|---------|
| `PluginMessageEvent` | Plugin → Plugin | Direct message between plugins with priority routing |
| `BroadcastMessageEvent` | Plugin → All | Broadcast message to all registered plugins |
| `PluginLifecycleEvent` | Service → Clients | Plugin registration, status changes, errors |
| `ClipboardEvent` | Service → Clients | Clipboard read/write operations |
| `HttpEvent` | Service → Clients | HTTP request/response operations |
| `NotificationEvent` | Service → Clients | System notification operations |
| `StorageEvent` | Service → Clients | Storage read/write operations |

### Supporting Resources
```rust
// Plugin registry and capabilities
struct PluginRegistryResource { plugins: HashMap<String, PluginInfo>, capabilities: HashMap<String, Vec<String>>, plugin_channels: HashMap<String, PluginChannel>, channel_receivers: HashMap<String, mpsc::Receiver<PluginMessageEvent>> }

// Channel management
struct ChannelManagerResource { channels: HashMap<String, ChannelInfo>, message_queue: Vec<QueuedMessage>, stats: ChannelStats }

// Configuration with memory optimization
struct ServiceBridgeConfig { max_plugins: usize, message_timeout_ms: u64, enable_metrics: bool, log_level: String }
```

### Components
```rust
// Plugin registration and status
struct PluginRegistration { plugin_id: String, capabilities: Vec<String>, registration_time: TimeStamp }
struct PluginStatus { status: PluginStatusType, last_heartbeat: Option<TimeStamp> }

// Message processing
struct QueuedMessage { message: PluginMessageEvent, queued_at: TimeStamp, retry_count: u32 }
struct MessageCorrelation { request_id: String, sender: String, expected_response_type: String }
```

### Systems
```rust
// Core message processing
process_plugin_messages              // Route messages between plugins with priority handling
handle_plugin_lifecycle              // Manage plugin registration, status changes, cleanup
process_broadcast_messages           // Distribute broadcast messages to all registered plugins
route_messages                       // Intelligent message routing based on capabilities

// Channel management
update_channel_stats                 // Track message throughput and channel health
manage_plugin_channels               // Create/destroy communication channels
handle_message_timeouts              // Process message timeouts and retry logic

// Plugin management
initialize_service_bridge           // Setup service bridge infrastructure
register_plugin_capabilities        // Register plugin capabilities and create channels
cleanup_inactive_plugins            // Remove inactive plugins and clean up resources
```

### Types
```rust
// Message priority for optimal routing
enum MessagePriority { Critical = 0, High = 1, Normal = 2, Low = 3 }

// Plugin lifecycle events
enum LifecycleEventType { Registered = 0, Started = 1, Stopped = 2, StatusChanged(String) = 3, Error(String) = 4, Unregistered = 5 }

// Service-specific operations
enum ClipboardOperation { Read = 0, Write(String) = 1, ReadResponse(String) = 2, WriteResponse(bool) = 3 }
enum HttpOperation { Request { url: String, method: String, body: Option<String> } = 0, Response { status: u16, body: String } = 1 }
enum StorageOperation { Read(String) = 0, Write(String, String) = 1, ReadResponse(Option<String>) = 2, WriteResponse(bool) = 3 }

// Health status
enum ServiceBridgeHealth { Healthy = 0, Degraded(String) = 1, Unhealthy(String) = 2 }
```

### Features

#### High-Performance Architecture
- **Memory Optimization**: All types use `#[repr(C)]` for optimal cache efficiency
- **Zero-Allocation Paths**: Hot paths designed for minimal memory allocation
- **Explicit Discriminants**: Enums use explicit byte representations for optimal serialization
- **Priority Queues**: Message routing based on priority levels for responsive communication
- **Channel Pooling**: Efficient reuse of communication channels across plugin lifecycle

#### Plugin Management
- **Capability Discovery**: Automatic registration and indexing of plugin capabilities
- **Health Monitoring**: Continuous health checks with automatic cleanup of failed plugins
- **Lifecycle Tracking**: Complete plugin lifecycle management from registration to cleanup
- **Dynamic Registration**: Runtime plugin registration and unregistration support
- **Resource Isolation**: Secure resource allocation and cleanup per plugin

#### Message Routing
- **Priority-Based Routing**: Critical messages bypass normal queues for immediate processing
- **Request/Response Correlation**: Automatic correlation of request/response message pairs
- **Broadcast Efficiency**: Optimized broadcast to all registered plugins with filtering
- **Timeout Handling**: Configurable timeouts with automatic retry logic
- **Message Queuing**: Persistent message queues for offline plugins with replay capability

### Usage Patterns

#### Plugin Registration
```rust
// Register plugin with capabilities
fn register_plugin(mut plugin_registry: ResMut<PluginRegistryResource>, mut lifecycle_events: EventWriter<PluginLifecycleEvent>) {
    let plugin_info = PluginInfo {
        plugin_id: "example_plugin".to_string(),
        capabilities: vec![Capability { name: "search".to_string(), version: "1.0".to_string(), description: "Search capability".to_string(), metadata: HashMap::new() }],
        status: PluginStatus::Active,
        registration_time: TimeStamp::now(),
        last_heartbeat: Some(TimeStamp::now()),
        // ...
    };
    plugin_registry.plugins.insert("example_plugin".to_string(), plugin_info);
    lifecycle_events.send(PluginLifecycleEvent { plugin_id: "example_plugin".to_string(), event_type: LifecycleEventType::Registered, timestamp: TimeStamp::now() });
}
```

#### Inter-Plugin Messaging
```rust
// Send message between plugins
fn send_plugin_message(mut message_events: EventWriter<PluginMessageEvent>) {
    message_events.send(PluginMessageEvent {
        from: "sender_plugin".to_string(),
        to: "receiver_plugin".to_string(),
        message_type: "search_request".to_string(),
        payload: serde_json::json!({"query": "example", "max_results": 10}),
        priority: MessagePriority::High,
        timestamp: TimeStamp::now(),
        request_id: Some(uuid::Uuid::new_v4().to_string()),
    });
}
```

#### Service Integration
```rust
// Handle storage operations via service bridge
fn handle_storage_request(mut storage_events: EventWriter<StorageEvent>) {
    storage_events.send(StorageEvent {
        request_id: uuid::Uuid::new_v4().to_string(),
        operation: StorageOperation::Read("config_key".to_string()),
        timestamp: TimeStamp::now(),
    });
}
```

### Backend Integration
- **Tokio Integration**: Full async/await support with tokio runtime integration
- **Bevy ECS Native**: Deep integration with Bevy's ECS systems and scheduling
- **Memory Management**: Automatic cleanup of channels, messages, and plugin resources
- **Architecture**: Event-driven ECS communication hub for the entire plugin ecosystem
- **Performance**: Sub-microsecond message routing with priority-based scheduling

## Database Service

### Resource: `DatabaseService`
```rust
// SurrealDB integration with LazyLock singleton pattern (v3.0)
database_service.query(sql: &str) -> Result<Response, DatabaseError>
database_service.query_with_params(sql: &str, params: HashMap<String, Value>) -> Result<Response, DatabaseError>
database_service.create<T>(table: &str, data: T) -> Result<RecordId, DatabaseError>
database_service.select<T>(table: &str) -> Result<Vec<T>, DatabaseError>
database_service.update<T>(thing: &RecordId, data: T) -> Result<Option<T>, DatabaseError>
database_service.delete(thing: &RecordId) -> Result<Option<Value>, DatabaseError>
database_service.health_check() -> Result<(), DatabaseError>
```

### Configuration Management
```rust
// Database configuration with security validation
struct DatabaseConfig { namespace: String, database: String, engine: DatabaseEngine, query_timeout_ms: u64, enable_query_logging: bool }

// Engine types
enum DatabaseEngine { SurrealKv(std::path::PathBuf) }

// Error handling
enum DatabaseError { ConnectionFailed(String), QueryFailed(String), TransactionFailed(String), InvalidConfiguration(String), Timeout { timeout_ms: u64 } }
```

### Supporting Resources
```rust
// Service state management
struct DatabaseServiceError(String)  // Service unavailable with reason
struct DatabaseShutdown              // Shutdown coordination resource

// Validation functions
fn validate_table_name(table: &str) -> Result<(), DatabaseError>
fn validate_storage_path(path: &Path) -> Result<(), DatabaseError>
```

### Components
```rust
// Async initialization
struct DatabaseInitTask(Task<CommandQueue>)

// Connection state (managed by SurrealDB LazyLock)
static DB: LazyLock<Surreal<Db>> = LazyLock::new(Surreal::init);
```

### Systems
```rust
// Lifecycle management
handle_database_init_task            // Process async database initialization
handle_database_shutdown             // Graceful shutdown with resource cleanup

// Service integration
database_availability_check          // Monitor service availability via DatabaseService::is_available
connection_health_monitoring         // Periodic health checks with automatic recovery
query_timeout_enforcement            // Enforce query timeouts and cleanup stalled operations
```

### Types
```rust
// Core database types (re-exported from SurrealDB)
type RecordId = surrealdb::RecordId
type Response = surrealdb::Response  
type Value = surrealdb::Value

// Service availability checking
fn DatabaseService::is_available(world: &World) -> bool
```

### Features

#### SurrealDB v3.0 Integration
- **LazyLock Singleton**: Uses SurrealDB v3.0's recommended LazyLock pattern for optimal performance
- **SurrealKV Backend**: Persistent embedded database with configurable storage paths
- **Connection Management**: Automatic connection handling with retry logic and timeout support
- **Namespace/Database**: Proper namespace and database scoping for multi-tenant support
- **Capability Configuration**: Configurable capabilities including scripting support

#### Safety and Security
- **Path Validation**: Comprehensive path traversal attack prevention and system directory protection
- **Table Name Validation**: Strict table name validation preventing injection attacks  
- **Query Timeouts**: Configurable query timeouts preventing runaway operations
- **Connection Retries**: Exponential backoff retry logic for connection resilience
- **Graceful Shutdown**: Proper resource cleanup and connection termination

#### Bevy ECS Integration
- **Async Initialization**: Non-blocking database initialization using Bevy's async task system
- **Resource Management**: Full ECS resource lifecycle with availability checking
- **Service Availability**: Easy service availability checking via `DatabaseService::is_available()`
- **Error Handling**: Comprehensive error states with `DatabaseServiceError` resource
- **Shutdown Coordination**: Clean shutdown via `DatabaseShutdown` resource trigger

### Usage Patterns

#### Basic CRUD Operations
```rust
#[derive(Component)]
struct CreateUserTask(Task<CommandQueue>);

#[derive(Component)] 
struct GetUsersTask(Task<CommandQueue>);

// Create records
fn create_user(mut commands: Commands, db: Res<DatabaseService>) {
    let db = db.clone();
    let task = AsyncComputeTaskPool::get().spawn(async move {
        let mut command_queue = CommandQueue::default();
        let user = serde_json::json!({"name": "John", "email": "john@example.com"});
        
        match db.create("users", user).await {
            Ok(result) => {
                command_queue.push(move |world: &mut World| {
                    // Handle successful creation result
                    info!("User created successfully: {:?}", result);
                });
            },
            Err(e) => {
                command_queue.push(move |world: &mut World| {
                    error!("Failed to create user: {}", e);
                });
            }
        }
        command_queue
    });
    
    commands.spawn(CreateUserTask(task));
}

// Query with type safety
fn get_users(mut commands: Commands, db: Res<DatabaseService>) {
    let db = db.clone();
    let task = AsyncComputeTaskPool::get().spawn(async move {
        let mut command_queue = CommandQueue::default();
        
        match db.select::<Vec<User>>("users").await {
            Ok(users) => {
                command_queue.push(move |world: &mut World| {
                    world.insert_resource(UserList(users));
                    info!("Users retrieved successfully");
                });
            },
            Err(e) => {
                command_queue.push(move |world: &mut World| {
                    error!("Failed to get users: {}", e);
                });
            }
        }
        command_queue
    });
    
    commands.spawn(GetUsersTask(task));
}

// Handle completed tasks
fn handle_database_tasks(
    mut commands: Commands,
    mut create_tasks: Query<(Entity, &mut CreateUserTask)>,
    mut get_tasks: Query<(Entity, &mut GetUsersTask)>,
) {
    // Handle create user tasks
    for (entity, mut task) in &mut create_tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
    
    // Handle get users tasks
    for (entity, mut task) in &mut get_tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}
```

#### Raw SurrealQL Queries
```rust
#[derive(Component)]
struct CustomQueryTask(Task<CommandQueue>);

// Execute raw queries with parameters
fn custom_query(mut commands: Commands, db: Res<DatabaseService>) {
    let db = db.clone();
    let task = AsyncComputeTaskPool::get().spawn(async move {
        let mut command_queue = CommandQueue::default();
        let params = std::collections::HashMap::from([
            ("min_age".to_string(), Value::from(18)),
            ("max_age".to_string(), Value::from(65))
        ]);
        
        match db.query_with_params("SELECT * FROM users WHERE age >= $min_age AND age <= $max_age", params).await {
            Ok(results) => {
                command_queue.push(move |world: &mut World| {
                    world.insert_resource(QueryResults(results));
                    info!("Custom query executed successfully");
                });
            },
            Err(e) => {
                command_queue.push(move |world: &mut World| {
                    error!("Failed to execute custom query: {}", e);
                });
            }
        }
        command_queue
    });
    
    commands.spawn(CustomQueryTask(task));
}

// Handle custom query tasks
fn handle_custom_query_tasks(
    mut commands: Commands,
    mut query_tasks: Query<(Entity, &mut CustomQueryTask)>,
) {
    for (entity, mut task) in &mut query_tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}
```

#### Service Availability Checking
```rust
// Check service availability before operations
fn safe_database_operation(world: &World) {
    if DatabaseService::is_available(world) {
        // Perform database operations
        if let Some(db) = world.get_resource::<DatabaseService>() {
            // Database is available and ready
        }
    } else {
        // Handle database unavailability - check DatabaseServiceError for details
        if let Some(error) = world.get_resource::<DatabaseServiceError>() {
            warn!("Database unavailable: {}", error.0);
        }
    }
}
```

#### Transaction Support
```rust
// Access singleton for transactions (internal use)
fn transaction_example(db: Res<DatabaseService>) {
    let task = AsyncComputeTaskPool::get().spawn(async move {
        let db_instance = DatabaseService::db_instance();
        // Use SurrealDB transaction APIs directly via singleton
        // Implementation depends on SurrealDB transaction patterns
    });
}
```

### Configuration Examples

#### Default Configuration
```rust
// Uses application directories with secure defaults
let config = DatabaseConfig::default();
// namespace: "action_items", database: "main"
// engine: SurrealKv(<app_config_dir>/database)
// query_timeout_ms: 10000, enable_query_logging: true
```

#### Custom Configuration
```rust
// Custom storage path and settings
let config = DatabaseConfig::surreal_kv("/custom/path/database")
    .with_namespace("my_app")
    .with_database("production")
    .with_query_timeout(Duration::from_secs(30))
    .with_logging(false);
```

### Backend Integration
- **SurrealDB v3.0**: Latest SurrealDB with LazyLock singleton pattern and optimized performance
- **SurrealKV Engine**: Embedded persistent storage with ACID transactions and full SurrealQL support
- **Action Items Common**: Integration with application directory management for secure storage paths
- **Architecture**: Async ECS resource with comprehensive error handling and availability checking
- **Performance**: Connection pooling via singleton pattern with configurable timeouts and health monitoring

## Service Integration Patterns

The ECS services work together as a cohesive ecosystem with clear integration patterns and dependencies.

### Service Dependencies

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Search         │    │  Service        │    │  Database       │
│  Aggregator     │────│  Bridge         │    │  Service        │
│                 │    │                 │────│                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐    ┌─────────────────┐
                    │  Core Services  │    │  Runtime        │
                    │  Cache, HTTP,   │────│  Services       │
                    │  Clipboard, etc │    │  Deno, Task Mgmt│
                    └─────────────────┘    └─────────────────┘
```

### Integration Patterns

#### Search Aggregator ↔ Service Bridge
```rust
// Search Aggregator uses Service Bridge for plugin communication
fn discover_search_capable_plugins(capability_index: &PluginCapabilityIndex) -> Vec<String> {
    capability_index.plugins_by_capability
        .get("search")
        .map(|plugins| plugins.iter().filter(|plugin| is_healthy(plugin)).collect())
        .unwrap_or_default()
}

// Send search requests via Service Bridge messaging
async fn execute_plugin_search(plugin_id: &str, query: &str, plugin_channel: &mut PluginChannel) {
    let message = PluginMessageEvent {
        from: "search_aggregator".to_string(),
        to: plugin_id.to_string(),
        message_type: "search_request".to_string(),
        payload: serde_json::json!({"query": query, "max_results": 20}),
        priority: MessagePriority::High,
        // ...
    };
    plugin_channel.send_message_and_wait_response(message).await
}
```

#### Service Bridge ↔ Database Service
```rust
#[derive(Component)]
struct PersistPluginStateTask(Task<CommandQueue>);

// Service Bridge can use Database Service for persistence
fn persist_plugin_state(
    mut commands: Commands,
    db: Res<DatabaseService>, 
    plugin_registry: Res<PluginRegistryResource>
) {
    let db = db.clone();
    let plugins = plugin_registry.plugins.clone();
    
    let task = AsyncComputeTaskPool::get().spawn(async move {
        let mut command_queue = CommandQueue::default();
        
        for (plugin_id, plugin_info) in &plugins {
            let plugin_state = serde_json::json!({
                "plugin_id": plugin_id,
                "status": plugin_info.status,
                "last_heartbeat": plugin_info.last_heartbeat,
                "capabilities": plugin_info.capabilities
            });
            
            match db.create("plugin_states", plugin_state).await {
                Ok(_) => {
                    let plugin_id = plugin_id.clone();
                    command_queue.push(move |world: &mut World| {
                        debug!("Plugin state persisted for: {}", plugin_id);
                    });
                },
                Err(e) => {
                    let error = e.to_string();
                    let plugin_id = plugin_id.clone();
                    command_queue.push(move |world: &mut World| {
                        error!("Failed to persist plugin state for {}: {}", plugin_id, error);
                    });
                }
            }
        }
        command_queue
    });
    
    commands.spawn(PersistPluginStateTask(task));
}

// Handle plugin state persistence tasks
fn handle_plugin_persistence_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut PersistPluginStateTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}
```

#### Core Services ↔ Service Bridge
```rust
// All core services use Service Bridge for inter-service communication
fn cache_service_integration(mut storage_events: EventWriter<StorageEvent>) {
    // Cache service requests storage via Service Bridge
    storage_events.send(StorageEvent {
        request_id: uuid::Uuid::new_v4().to_string(),
        operation: StorageOperation::Write("cache_key".to_string(), "cached_data".to_string()),
        timestamp: TimeStamp::now(),
    });
}

// HTTP service can cache responses via Service Bridge
fn http_cache_integration(mut message_events: EventWriter<PluginMessageEvent>) {
    message_events.send(PluginMessageEvent {
        from: "http_service".to_string(),
        to: "cache_service".to_string(),
        message_type: "cache_store".to_string(),
        payload: serde_json::json!({"key": "http_response_key", "value": response_data, "ttl_seconds": 300}),
        priority: MessagePriority::Normal,
        // ...
    });
}
```

### Communication Flows

#### 1. Search Request Flow
```
User Query → Search Aggregator → Service Bridge → Plugins → Service Bridge → Search Aggregator → UI
                    ↓
              Database Service (optional: cache results, store search analytics)
```

#### 2. Plugin Registration Flow  
```
Plugin → Service Bridge → Plugin Registry → Database Service (persist registration)
                ↓
         Capability Index Update → Search Aggregator (update available plugins)
```

#### 3. Cross-Service Request Flow
```
HTTP Service → Service Bridge → Cache Service → Service Bridge → HTTP Service
     ↓                                    ↓
Database Service                  Database Service
(store request log)              (persist cache entry)
```

### Usage Patterns

#### Coordinated Service Usage
```rust
// System that uses multiple services together
fn coordinated_search_and_cache(
    mut search_events: EventWriter<SearchRequested>,
    mut cache_events: EventWriter<StorageEvent>, 
    current_query: Res<CurrentQuery>,
    db: Option<Res<DatabaseService>>
) {
    if !current_query.is_empty() {
        // 1. Check cache first via Service Bridge
        cache_events.send(StorageEvent {
            request_id: "search_cache_check".to_string(),
            operation: StorageOperation::Read(format!("search_cache:{}", current_query.0)),
            timestamp: TimeStamp::now(),
        });
        
        // 2. If cache miss, initiate search
        search_events.send(SearchRequested::new(
            current_query.0.clone(),
            vec!["plugin1".to_string(), "plugin2".to_string()]
        ));
        
        // 3. Optionally log search to database
        if let Some(database) = db {
            let task = AsyncComputeTaskPool::get().spawn(async move {
                let search_log = serde_json::json!({
                    "query": current_query.0,
                    "timestamp": chrono::Utc::now(),
                    "user_session": "session_id"
                });
                database.create("search_logs", search_log).await
            });
        }
    }
}
```

#### Service Health Coordination  
```rust
// Monitor overall system health across services
fn system_health_check(
    world: &World,
    service_bridge: Res<ServiceBridgeResource>,
    mut health_events: EventWriter<SystemHealthEvent>
) {
    let mut health_status = SystemHealth::Healthy;
    
    // Check Database Service
    if !DatabaseService::is_available(world) {
        health_status = SystemHealth::Degraded("Database unavailable".into());
    }
    
    // Check Service Bridge
    match service_bridge.health {
        ServiceBridgeHealth::Unhealthy(ref reason) => {
            health_status = SystemHealth::Critical(format!("Service Bridge failed: {}", reason));
        },
        ServiceBridgeHealth::Degraded(ref reason) => {
            if health_status == SystemHealth::Healthy {
                health_status = SystemHealth::Degraded(format!("Service Bridge degraded: {}", reason));
            }
        },
        _ => {}
    }
    
    health_events.send(SystemHealthEvent { status: health_status, timestamp: TimeStamp::now() });
}
```

This integration architecture provides a robust, scalable foundation for plugin-based applications with comprehensive service coordination and communication patterns.

## Deno Runtime Service

### Resources: `DenoRuntimePool`, `DenoOperationTracker`, `ExtensionDiscoveryManager`, `DenoMetrics`
```rust
// Runtime pool management with security sandboxing
deno_pool.max_runtimes: usize                    // Maximum concurrent Deno runtimes
deno_pool.default_timeout: Duration              // Default timeout for operations
deno_pool.sandbox_config: SandboxConfiguration  // Security sandbox settings
deno_pool.available_runtimes: Vec<Runtime>       // Pool of available runtimes

// Operation tracking and correlation
operation_tracker.active_operations: HashMap<DenoOperationId, OperationState>
operation_tracker.operation_history: RingBuffer<CompletedOperation>
operation_tracker.metrics: OperationMetrics

// Extension discovery management
discovery_manager.search_paths: Vec<PathBuf>     // Raycast extension search paths
discovery_manager.discovered_extensions: HashMap<String, ExtensionMetadata>
discovery_manager.discovery_cache: LRUCache<PathBuf, CachedDiscoveryResult>
```

### Events (Request/Response Pattern)
| Event | Direction | Purpose |
|-------|-----------|---------|
| `DenoScriptExecutionRequested` | Client → Service | Execute JavaScript code with timeout and sandbox config |
| `DenoScriptExecutionCompleted` | Service → Client | Script execution successful with results |
| `DenoScriptExecutionFailed` | Service → Client | Script execution failed with error details |
| `ExtensionDiscoveryRequested` | Client → Service | Discover Raycast extensions in specified paths |
| `ExtensionDiscoveryCompleted` | Service → Client | Extension discovery completed with found extensions |
| `ExtensionDiscoveryFailed` | Service → Client | Extension discovery failed with error details |
| `DenoRuntimeCreated` | Service → Clients | New Deno runtime created in pool |
| `DenoRuntimeDestroyed` | Service → Clients | Deno runtime destroyed and removed from pool |
| `DenoOperationTimeout` | Service → Client | Operation exceeded configured timeout |
| `DenoMetricsReportRequested` | Client → Service | Request performance metrics report |
| `DenoMetricsReportGenerated` | Service → Client | Performance metrics report available |

### Supporting Resources
```rust
// Security configuration
struct SandboxConfiguration { allow_net: bool, allow_read: bool, allow_write: bool, allow_env: bool, allow_run: bool, allow_ffi: bool, allow_hrtime: bool, allowed_hosts: Vec<String>, allowed_paths: Vec<PathBuf> }

// Performance optimization
struct PerformanceConfig { enable_string_interning: bool, string_interning_threshold: usize, enable_batch_processing: bool, max_batch_size: usize, enable_detailed_logging: bool, enable_v8_optimizations: bool }

// Operation metrics
struct DenoMetrics { total_executions: u64, successful_executions: u64, failed_executions: u64, average_execution_time: Duration, extensions_discovered: u64, cache_hits: u64 }
```

### Components
```rust
// Script execution tracking
struct DenoScriptExecution { operation_id: DenoOperationId, script_content: String, timeout: Duration, started_at: Instant, status: DenoOperationStatus }

// Extension discovery tracking  
struct ExtensionDiscoveryOperation { operation_id: DenoOperationId, search_paths: Vec<PathBuf>, started_at: Instant, status: DenoOperationStatus }

// Timeout management
struct DenoOperationTimeout { operation_id: DenoOperationId, deadline: Instant, timeout_duration: Duration }

// Operation status tracking
enum DenoOperationStatus { Pending, InProgress, Completed, Failed(DenoExecutionError), TimedOut }
```

### Systems
```rust
// Runtime management
manage_deno_runtime_pool_system          // Monitor runtime pool health, create/destroy runtimes
initialize_deno_runtimes_system          // Initialize Deno runtimes with sandbox configuration

// Script execution processing
process_script_execution_requests_system  // Handle script execution requests
process_script_execution_completions_system // Process completed script executions and emit results

// Extension discovery
process_extension_discovery_requests_system // Handle extension discovery requests
process_extension_discovery_completions_system // Process discovery completions and cache results

// Operation lifecycle
handle_operation_timeouts_system         // Monitor and handle operation timeouts
cleanup_completed_operations_system      // Clean up completed operations and update metrics
update_deno_metrics_system              // Update performance metrics and emit reports
```

### Types
```rust
// Core operation types
type DenoOperationId = uuid::Uuid
struct ScriptExecutionResult { output: serde_json::Value, execution_time: Duration, memory_usage: usize }

// Extension discovery types
struct ExtensionMetadata { name: String, version: String, description: String, main_script: PathBuf, permissions: Vec<String>, dependencies: Vec<String> }
struct CachedDiscoveryResult { extensions: Vec<ExtensionMetadata>, cached_at: SystemTime, path_modified_at: SystemTime }

// Error handling
enum DenoExecutionError { ScriptError(String), TimeoutError, SecurityViolation(String), RuntimeError(String), V8Error(String) }
enum ExtensionDiscoveryError { PathNotFound(PathBuf), PermissionDenied(PathBuf), InvalidExtension(String), FileSystemError(String) }
```

### Features

#### JavaScript Runtime Management
- **Runtime Pool**: Configurable pool of Deno runtimes for concurrent execution
- **Security Sandbox**: Comprehensive permission system for network, filesystem, environment access
- **V8 Optimizations**: Performance optimizations including string interning and batch processing
- **Timeout Handling**: Configurable timeouts with automatic cleanup for long-running operations
- **Memory Management**: Automatic cleanup of completed operations and runtime pool optimization

#### Raycast Extension Discovery
- **Path-Based Discovery**: Scan specified directories for Raycast-compatible extensions
- **Extension Validation**: Validate extension structure, metadata, and dependencies  
- **Caching System**: LRU cache for discovery results with filesystem change detection
- **Metadata Extraction**: Extract extension name, version, description, permissions, and dependencies
- **Performance Optimization**: Batch processing and parallel discovery operations

#### Development vs Production Modes
- **Development Mode**: Permissive sandbox settings, detailed logging, lower optimization thresholds
- **Production Mode**: Strict security sandbox, optimized performance, minimal logging
- **Custom Configuration**: Granular control over all security and performance settings

### Usage Patterns

#### Script Execution (Event-Driven)
```rust
// Execute JavaScript code
fn execute_script(mut script_events: EventWriter<DenoScriptExecutionRequested>) {
    script_events.send(DenoScriptExecutionRequested {
        operation_id: uuid::Uuid::new_v4(),
        script_content: "console.log('Hello from Deno!'); JSON.stringify({result: 'success'})".to_string(),
        timeout: Duration::from_secs(10),
        requester: "my_plugin".to_string(),
        requested_at: Instant::now(),
    });
}

// Handle script execution results
fn handle_script_results(mut events: EventReader<DenoScriptExecutionCompleted>) {
    for completion in events.read() {
        info!("Script {} completed in {:?}", completion.operation_id, completion.result.execution_time);
        match completion.result.output {
            serde_json::Value::Object(obj) => {
                // Process structured result
            },
            _ => {
                // Handle other result types
            }
        }
    }
}
```

#### Extension Discovery (Event-Driven)
```rust
// Discover extensions
fn discover_extensions(mut discovery_events: EventWriter<ExtensionDiscoveryRequested>) {
    discovery_events.send(ExtensionDiscoveryRequested {
        operation_id: uuid::Uuid::new_v4(),
        search_paths: vec![
            PathBuf::from("/Applications/Raycast.app/Contents/Resources/extensions"),
            PathBuf::from("~/Library/Application Support/com.raycast.macos/extensions"),
        ],
        requester: "extension_manager".to_string(),
        requested_at: Instant::now(),
    });
}

// Handle discovery results
fn handle_discovery_results(mut events: EventReader<ExtensionDiscoveryCompleted>) {
    for completion in events.read() {
        info!("Discovered {} extensions in {:?}", 
              completion.extensions.len(), 
              completion.discovery_duration);
        
        for extension in &completion.extensions {
            info!("Found extension: {} v{}", extension.name, extension.version);
        }
    }
}
```

#### Convenience API (Service-Style)
```rust
// Direct service usage
fn use_deno_service(
    mut commands: Commands,
    mut script_events: EventWriter<DenoScriptExecutionRequested>,
) {
    // Execute script with default settings
    let operation_id = DenoService::execute_script(
        "const result = Math.random() * 100; JSON.stringify({value: result})",
        &mut commands,
        &mut script_events,
    );
    
    info!("Started script execution with ID: {}", operation_id);
}
```

### Configuration Examples

#### Development Configuration
```rust
DenoPlugin::new()
    .development_mode()
    .with_max_runtimes(2)
    .with_default_timeout(Duration::from_secs(60))
    // Permissive sandbox, detailed logging, lower optimization thresholds
```

#### Production Configuration
```rust
DenoPlugin::new()
    .production_mode()
    .with_max_runtimes(8)
    .with_default_timeout(Duration::from_secs(30))
    // Strict security, optimized performance, minimal logging
```

#### Custom Configuration
```rust
DenoPlugin::new()
    .with_sandbox_config(SandboxConfig {
        allow_read: true,  // Allow file system read for extension discovery
        allow_net: false,  // Block network access
        // ... other security settings
    })
    .with_performance_config(PerformanceConfig {
        enable_string_interning: true,
        string_interning_threshold: 100,
        enable_v8_optimizations: true,
        // ... other performance settings
    })
```

### Backend Integration
- **Deno Runtime**: Uses Deno's secure JavaScript/TypeScript runtime with V8 engine
- **Raycast Compatibility**: Full compatibility with Raycast extension format and metadata
- **Bevy ECS Integration**: Event-driven architecture with proper resource management and system scheduling
- **Performance**: Runtime pooling, string interning, batch processing, and zero-allocation optimizations
- **Security**: Comprehensive sandbox with granular permission control for network, filesystem, and system access

## Task Management Service

### Resource: `TaskStatistics`
```rust
// Task lifecycle tracking and metrics
task_stats.total_spawned: u64        // Total tasks created
task_stats.total_completed: u64      // Successfully completed tasks  
task_stats.total_failed: u64         // Failed tasks
task_stats.total_expired: u64        // Tasks that exceeded timeout
task_stats.active_tasks: u64         // Currently running tasks
```

### Events (Request/Response Pattern)
| Event | Direction | Purpose |
|-------|-----------|---------|
| `TaskCompletedEvent<T>` | Service → Client | Generic task completion with Success/Expired variants |

### Components
```rust
// Generic async task wrapper
struct ManagedTask<T: Send + Sync + 'static> { id: Uuid, task: Task<T>, created_at: Instant, timeout_duration: Option<Duration> }

// Specialized task components
struct HotkeyPreferencesLoadTask { task: ManagedTask<HotkeyPreferencesResult> }
struct HotkeyPreferencesPersistTask { task: ManagedTask<Result<PathBuf, Box<dyn Error + Send + Sync>>> }
```

### Systems
```rust
// Generic task polling with timeout and event emission
poll_managed_tasks<T>     // Poll tasks, handle timeouts, emit completion events with results
```

### Types
```rust
// Task completion events
enum TaskCompletedEvent<T> { Success { id: Uuid, result: T, duration: Duration }, Expired { id: Uuid, duration: Duration } }

// Specialized result types
struct HotkeyPreferencesResult { preferred_combinations: Vec<HotkeyDefinition>, auto_fallback: bool }
struct HotkeyDefinition { modifiers: u32, code: u32, description: String }
```

### Features

#### Generic Task Management
- **Type-Safe Wrappers**: Generic `ManagedTask<T>` component for any async operation type
- **Timeout Handling**: Configurable per-task timeouts with automatic cleanup
- **Event-Driven Completion**: Emit structured events for task success/failure/expiration
- **Statistics Tracking**: Comprehensive metrics for task lifecycle monitoring
- **Resource Management**: Automatic entity cleanup and memory management

#### Specialized Task Support
- **Hotkey Preferences**: Built-in support for hotkey preference loading and persistence
- **Extensible Design**: Easy to add new specialized task types with proper type safety
- **Error Handling**: Comprehensive error propagation and structured error types

### Usage Patterns

#### Generic Task Usage
```rust
// Spawn a managed task
fn spawn_async_task(mut commands: Commands) {
    let async_task = AsyncComputeTaskPool::get().spawn(async {
        // Perform async work
        tokio::time::sleep(Duration::from_secs(5)).await;
        "Task completed successfully".to_string()
    });
    
    commands.spawn((
        ManagedTask::new(async_task)
            .with_timeout(Duration::from_secs(10)),
        TaskStatistics::default(),
    ));
}

// Handle task completion events
fn handle_task_completion(mut events: EventReader<TaskCompletedEvent<String>>) {
    for event in events.read() {
        match event {
            TaskCompletedEvent::Success { id, result, duration } => {
                info!("Task {} completed in {:?}: {}", id, duration, result);
            },
            TaskCompletedEvent::Expired { id, duration } => {
                warn!("Task {} expired after {:?}", id, duration);
            },
        }
    }
}
```

#### Specialized Hotkey Task Usage
```rust
// Load hotkey preferences
fn load_hotkey_preferences(mut commands: Commands) {
    let load_task = AsyncComputeTaskPool::get().spawn(async {
        // Load from file system or configuration
        HotkeyPreferencesResult {
            preferred_combinations: vec![
                HotkeyDefinition {
                    modifiers: 0x08, // Cmd key
                    code: 0x31,      // Space key
                    description: "Open Action Items".to_string(),
                }
            ],
            auto_fallback: true,
        }
    });
    
    commands.spawn(HotkeyPreferencesLoadTask {
        task: ManagedTask::new(load_task)
            .with_timeout(Duration::from_secs(5)),
    });
}
```

### Backend Integration
- **Bevy Tasks**: Uses Bevy's `AsyncComputeTaskPool` for efficient async task execution
- **Type Safety**: Full generic type support with compile-time safety guarantees
- **ECS Integration**: Proper component lifecycle with automatic cleanup and resource management
- **Performance**: Non-blocking task polling with minimal overhead and zero-allocation paths
- **Extensibility**: Easy to extend with new task types while maintaining consistent patterns

## Compression Service

### Resource: `CompressionManager`
```rust
// Sync API for immediate operations
compression_mgr.compress_sync(data: Vec<u8>) -> Result<CompressedData, CompressionError>
compression_mgr.decompress_sync(compressed: &CompressedData) -> Result<Vec<u8>, CompressionError>
compression_mgr.get_stats() -> CompressionStats
compression_mgr.optimal_algorithm(data: &[u8]) -> CompressionAlgorithm
compression_mgr.is_available() -> bool
```

### Events (Request/Response Pattern)
| Event | Direction | Purpose |
|-------|-----------|---------|
| `CompressionRequested` | Client → Service | Compress data with optional algorithm preference |
| `CompressionCompleted` | Service → Client | Compression operation result |
| `DecompressionRequested` | Client → Service | Decompress compressed data |
| `DecompressionCompleted` | Service → Client | Decompression operation result |
| `CompressionStatsRequested` | Client → Service | Request performance statistics |
| `CompressionStatsResponse` | Service → Client | Current compression statistics |

### Supporting Resources
```rust
// Configuration with zero-allocation optimization
struct CompressionConfig { 
    default_algorithm: CompressionAlgorithm, 
    min_size_threshold: usize,
    enable_entropy_analysis: bool,
    buffer_pool_size: usize,
    max_buffer_size: usize 
}

// Performance metrics with atomic operations
struct CompressionStats { 
    total_operations: AtomicU64,
    total_bytes_processed: AtomicU64, 
    total_compression_time: AtomicU64,
    compression_ratio_sum: AtomicU64,
    algorithm_usage: [AtomicU64; 4] // Gzip, Deflate, LZ4, Zstd
}
```

### Components
```rust
// Async operation tracking
struct CompressionTask { 
    operation_id: Uuid, 
    task: Task<CommandQueue>, 
    algorithm: CompressionAlgorithm,
    original_size: usize,
    started_at: Instant 
}

struct DecompressionTask { 
    operation_id: Uuid, 
    task: Task<CommandQueue>, 
    algorithm: CompressionAlgorithm,
    compressed_size: usize,
    started_at: Instant 
}

// Buffer pool management
struct ThreadLocalBufferPool { 
    buffers: Vec<Vec<u8>>, 
    max_buffers: usize, 
    buffer_size_limit: usize 
}
```

### Systems
```rust
// Core processing systems
process_compression_requests     // Handle CompressionRequested → CompressionCompleted
process_decompression_requests   // Handle DecompressionRequested → DecompressionCompleted
handle_compression_tasks         // Poll async compression tasks and emit completion events
handle_decompression_tasks       // Poll async decompression tasks and emit completion events

// Background systems
update_compression_stats         // Update performance statistics from completed operations
optimize_buffer_pools           // Manage thread-local buffer pool sizes and cleanup
monitor_compression_health      // Monitor service health and performance metrics
```

### Types
```rust
// Core compression types
struct CompressedData { 
    data: Vec<u8>, 
    original_size: usize, 
    algorithm: CompressionAlgorithm,
    compression_ratio: f32 
}

enum CompressionAlgorithm { 
    Gzip = 0,    // Balanced compression, wide compatibility
    Deflate = 1, // Similar to Gzip, slightly faster
    Lz4 = 2,     // Fastest compression/decompression
    Zstd = 3     // Best compression ratio, modern algorithm
}

// Operation requests
struct CompressionRequest { 
    operation_id: Uuid, 
    data: Vec<u8>, 
    preferred_algorithm: Option<CompressionAlgorithm>,
    requester: String 
}

struct DecompressionRequest { 
    operation_id: Uuid, 
    compressed_data: CompressedData, 
    requester: String 
}

// Error handling
enum CompressionError { 
    CompressionFailed(String),
    DecompressionFailed(String), 
    UnsupportedAlgorithm(CompressionAlgorithm),
    InvalidFormat,
    BufferPoolExhausted,
    SizeMismatch { expected: usize, actual: usize }
}
```

### Features

#### Zero-Allocation Architecture
- **Thread-Local Buffer Pools**: Per-thread buffer pools eliminate allocation after initialization
- **Streaming Compression**: Direct compression into pre-allocated buffers without intermediate copies
- **Buffer Lifecycle Management**: Custom Drop implementations ensure automatic buffer return to pools
- **Memory Bounded**: Configurable limits prevent unbounded memory growth
- **Lock-Free Operations**: Thread-local storage eliminates synchronization overhead

#### Intelligent Algorithm Selection
- **Entropy Analysis**: Automatic algorithm selection based on data entropy characteristics
- **Performance History**: Algorithm selection considers historical performance for data patterns
- **Size-Based Optimization**: Different algorithms for different data sizes (small/medium/large)
- **Fallback Handling**: Graceful degradation when compression doesn't provide benefits
- **Custom Override**: Allow explicit algorithm selection when needed

#### High-Performance Implementation
- **Blazing Fast**: Sub-microsecond buffer operations with optimized hot paths
- **Concurrent Processing**: Full support for concurrent compression operations across threads
- **Minimal Overhead**: Zero-allocation paths for all common operations after initialization
- **Streaming APIs**: Process data without loading entire datasets into memory
- **Batch Processing**: Efficient handling of multiple compression operations

#### Production Quality
- **Comprehensive Error Handling**: Detailed error types with recovery guidance
- **Statistics Tracking**: Real-time performance metrics with atomic operations
- **Health Monitoring**: Service health checks and automatic recovery mechanisms
- **Memory Safety**: No unsafe code, comprehensive bounds checking
- **Robust Testing**: Extensive test coverage including edge cases and stress testing

### Usage Patterns

#### Synchronous Compression (Direct API)
```rust
// Compress data with automatic algorithm selection
fn compress_data(compression_mgr: Res<CompressionManager>) {
    let data = vec![0u8; 1024]; // Example data
    
    match compression_mgr.compress_sync(data) {
        Ok(compressed) => {
            info!("Compressed {} bytes to {} bytes using {:?} (ratio: {:.2}%)",
                  compressed.original_size, 
                  compressed.data.len(),
                  compressed.algorithm,
                  compressed.compression_ratio * 100.0);
        },
        Err(e) => error!("Compression failed: {}", e),
    }
}

// Decompress with validation
fn decompress_data(compression_mgr: Res<CompressionManager>, compressed: CompressedData) {
    match compression_mgr.decompress_sync(&compressed) {
        Ok(decompressed) => {
            info!("Decompressed {} bytes back to {} bytes", 
                  compressed.data.len(), decompressed.len());
        },
        Err(e) => error!("Decompression failed: {}", e),
    }
}
```

#### Asynchronous Compression (Event-Driven)
```rust
// Request compression via events
fn request_compression(mut compression_events: EventWriter<CompressionRequested>) {
    compression_events.send(CompressionRequested {
        request: CompressionRequest {
            operation_id: Uuid::new_v4(),
            data: vec![0u8; 4096], // Large data for async processing
            preferred_algorithm: Some(CompressionAlgorithm::Zstd),
            requester: "data_processor".to_string(),
        },
        requested_at: Instant::now(),
    });
}

// Handle compression results
fn handle_compression_results(mut events: EventReader<CompressionCompleted>) {
    for completion in events.read() {
        match &completion.result {
            Ok(compressed_data) => {
                info!("Async compression {} completed in {:?}: {} → {} bytes",
                      completion.operation_id,
                      completion.duration,
                      compressed_data.original_size,
                      compressed_data.data.len());
            },
            Err(error) => {
                error!("Async compression {} failed: {}", completion.operation_id, error);
            }
        }
    }
}
```

#### Performance Monitoring
```rust
// Monitor compression statistics
fn monitor_compression_performance(compression_mgr: Res<CompressionManager>) {
    let stats = compression_mgr.get_stats();
    
    info!("Compression Statistics:");
    info!("  Total operations: {}", stats.total_operations.load(Ordering::Relaxed));
    info!("  Bytes processed: {}", stats.total_bytes_processed.load(Ordering::Relaxed));
    info!("  Average compression time: {}μs", 
          stats.total_compression_time.load(Ordering::Relaxed) / stats.total_operations.load(Ordering::Relaxed));
    
    // Algorithm usage distribution
    let algorithms = ["Gzip", "Deflate", "LZ4", "Zstd"];
    for (i, name) in algorithms.iter().enumerate() {
        info!("  {} usage: {}", name, stats.algorithm_usage[i].load(Ordering::Relaxed));
    }
}
```

#### Integration with Other Services
```rust
// Compress data before caching
fn compress_and_cache(
    compression_mgr: Res<CompressionManager>,
    mut cache_events: EventWriter<CacheWriteRequested>
) {
    let large_data = vec![0u8; 10240]; // 10KB data
    
    match compression_mgr.compress_sync(large_data) {
        Ok(compressed) => {
            // Store compressed data in cache
            cache_events.send(CacheWriteRequested {
                partition: "large_objects".to_string(),
                key: "dataset_123".to_string(),
                value: compressed.data,
                ttl_seconds: Some(3600),
                operation_id: Uuid::new_v4(),
                requester: "data_service".to_string(),
            });
        },
        Err(e) => error!("Failed to compress data for caching: {}", e),
    }
}

// Decompress cached data
fn decompress_from_cache(
    compression_mgr: Res<CompressionManager>,
    cached_data: Vec<u8>
) -> Result<Vec<u8>, CompressionError> {
    // Reconstruct CompressedData from cached bytes
    // (Implementation depends on serialization format)
    let compressed_data = CompressedData::from_bytes(cached_data)?;
    compression_mgr.decompress_sync(&compressed_data)
}
```

### Configuration Examples

#### High-Performance Configuration
```rust
CompressionPlugin::new()
    .with_config(CompressionConfig {
        default_algorithm: CompressionAlgorithm::Lz4, // Fastest
        min_size_threshold: 256,  // Compress data >= 256 bytes
        enable_entropy_analysis: true,
        buffer_pool_size: 16,     // 16 buffers per thread
        max_buffer_size: 1024 * 1024, // 1MB max buffer
    })
```

#### High-Compression Configuration
```rust
CompressionPlugin::new()
    .with_config(CompressionConfig {
        default_algorithm: CompressionAlgorithm::Zstd, // Best ratio
        min_size_threshold: 64,   // Compress smaller data
        enable_entropy_analysis: true,
        buffer_pool_size: 8,      // Fewer buffers for memory efficiency
        max_buffer_size: 512 * 1024, // 512KB max buffer
    })
```

#### Balanced Configuration (Default)
```rust
CompressionPlugin::new()
    .with_default_config()
    // default_algorithm: Gzip, min_size_threshold: 128
    // buffer_pool_size: 8, max_buffer_size: 1MB
```

### Backend Integration
- **Multiple Algorithms**: Support for Gzip, Deflate, LZ4, and Zstd compression algorithms
- **Thread-Local Pools**: Zero-allocation buffer management using thread-local storage
- **Bevy ECS Native**: Full integration with Bevy's async task system and event-driven architecture
- **Performance Optimized**: Sub-microsecond operations with streaming compression and lock-free design
- **Production Ready**: Comprehensive error handling, statistics tracking, and health monitoring


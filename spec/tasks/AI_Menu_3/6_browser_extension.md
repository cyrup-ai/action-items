# AI Menu 3 - Browser Extension Integration System

## Implementation Task: Cross-Browser Context Extraction and Real-time Communication

### Architecture Overview
Implement comprehensive browser extension integration system that extracts context from browser tabs, manages multi-browser connections, and provides real-time status monitoring with privacy controls.

### Core Components

#### Browser Extension Manager
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct BrowserExtensionManager {
    pub active_connections: HashMap<String, BrowserConnection>,
    pub connection_status: ConnectionStatus,
    pub context_extractor: ContextExtractor,
    pub communication_server: ExtensionServer,
    pub privacy_controls: PrivacyControls,
}

#[derive(Reflect)]
pub struct ConnectionStatus {
    pub is_connected: bool,
    pub last_successful_connection: Option<SystemTime>,
    pub connection_count: u64,
    pub supported_browsers: Vec<BrowserInfo>,
    pub active_browser_count: u32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct BrowserConnection {
    pub browser_id: String,
    pub browser_type: BrowserType,
    pub version: String,
    pub connection_established: SystemTime,
    pub last_activity: SystemTime,
    pub active_tabs: HashMap<String, TabInfo>,
    pub extension_version: String,
}

#[derive(Reflect)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Arc,
    Brave,
    Opera,
    Other(String),
}
```

#### Context Extraction System
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ContextExtractor {
    pub extraction_policies: ExtractionPolicies,
    pub content_filters: ContentFilters,
    pub privacy_processor: PrivacyProcessor,
    pub context_cache: ContextCache,
}

#[derive(Reflect)]
pub struct TabInfo {
    pub tab_id: String,
    pub url: String,
    pub title: String,
    pub content_type: ContentType,
    pub last_updated: SystemTime,
    pub context_data: Option<ExtractedContext>,
    pub privacy_level: PrivacyLevel,
}

#[derive(Reflect)]
pub struct ExtractedContext {
    pub visible_text: String,
    pub metadata: TabMetadata,
    pub structured_data: Option<StructuredData>,
    pub extraction_timestamp: SystemTime,
    pub content_hash: String,
    pub size_bytes: usize,
}

#[derive(Reflect)]
pub enum ContentType {
    Webpage,
    Document,
    SocialMedia,
    CodeRepository,
    Email,
    Shopping,
    News,
    Unknown,
}
```

### Bevy Implementation References

#### WebSocket Server for Browser Communication
- **Network Server**: `docs/bevy/examples/async_tasks/async_compute.rs`
  - WebSocket server for real-time browser communication
  - Connection handling for multiple browser instances
  - Message routing between browsers and application

#### Real-time Status Display
- **UI Text Updates**: `docs/bevy/examples/ui/text.rs`
  - Connection status display with timestamps
  - Real-time updates of connection information
  - Formatted timestamp display ("8/6/2025, 5:30 PM")

#### Cross-Platform Communication
- **System Integration**: `docs/bevy/examples/app/plugin.rs`
  - Cross-platform browser detection and communication
  - System-level integration with browser APIs
  - Platform-specific browser extension protocols

#### Privacy and Content Filtering
- **Event Processing**: `docs/bevy/examples/ecs/send_and_receive_events.rs`
  - Content filtering and privacy processing events
  - Real-time context extraction and validation
  - User consent and privacy enforcement events

### Browser Communication Protocol

#### Extension Communication Server
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ExtensionServer {
    pub server_port: u16,               // Local WebSocket server port
    pub server_status: ServerStatus,
    pub active_sessions: HashMap<String, ExtensionSession>,
    pub message_handlers: MessageHandlers,
    pub security_config: ServerSecurityConfig,
}

#[derive(Reflect)]
pub struct ExtensionSession {
    pub session_id: String,
    pub browser_connection: BrowserConnection,
    pub authentication_status: AuthStatus,
    pub message_queue: VecDeque<ExtensionMessage>,
    pub heartbeat_interval: Duration,
    pub last_heartbeat: SystemTime,
}

#[derive(Reflect)]
pub enum ExtensionMessage {
    ContextUpdate {
        tab_id: String,
        context: ExtractedContext,
    },
    TabChanged {
        old_tab: Option<String>,
        new_tab: String,
    },
    BrowserConnected {
        browser_info: BrowserInfo,
    },
    BrowserDisconnected {
        browser_id: String,
    },
    Heartbeat {
        timestamp: SystemTime,
    },
    Error {
        error_type: String,
        message: String,
    },
}
```

#### Multi-Browser Support
- **Chrome**: Native Messaging API integration
- **Firefox**: WebExtension Native Messaging
- **Safari**: Safari App Extension communication
- **Edge**: Chromium-based Native Messaging
- **Arc**: Custom protocol integration
- **Cross-platform**: Unified API abstraction layer

### Connection Status Display

#### Status Formatting System
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ConnectionStatusDisplay {
    pub status_text: Entity,            // "Last successful connection..."
    pub connection_indicator: Entity,   // Visual status indicator
    pub timestamp_formatter: TimestampFormatter,
    pub update_frequency: Duration,     // How often to refresh display
}

#[derive(Reflect)]
pub struct TimestampFormatter {
    pub format_style: TimestampStyle,
    pub timezone: TimeZone,
    pub locale: String,
}

#[derive(Reflect)]
pub enum TimestampStyle {
    DateTime,                           // "8/6/2025, 5:30 PM"
    RelativeTime,                       // "2 hours ago"
    Detailed,                           // "August 6, 2025 at 5:30:12 PM"
    ISO8601,                           // "2025-08-06T17:30:00Z"
}
```

#### Real-time Status Updates
- **Connection Events**: Update display immediately on connection changes
- **Timestamp Refresh**: Periodic refresh of "last connection" timestamps
- **Visual Indicators**: Color-coded connection health indicators
- **Error States**: Clear indication of connection problems

### Privacy and Security Framework

#### Privacy Controls Implementation
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PrivacyControls {
    pub data_collection_consent: bool,
    pub sensitive_content_filtering: bool,
    pub domain_whitelist: Vec<String>,
    pub content_type_filters: ContentTypeFilters,
    pub retention_policy: DataRetentionPolicy,
    pub anonymization_settings: AnonymizationSettings,
}

#[derive(Reflect)]
pub struct ContentTypeFilters {
    pub allow_social_media: bool,
    pub allow_shopping_sites: bool,
    pub allow_personal_email: bool,
    pub allow_banking_sites: bool,
    pub allow_private_documents: bool,
    pub custom_domain_rules: HashMap<String, FilterRule>,
}

#[derive(Reflect)]
pub enum FilterRule {
    Allow,
    Block,
    AllowWithRedaction,
    RequireExplicitConsent,
}

#[derive(Reflect)]
pub struct AnonymizationSettings {
    pub strip_personal_info: bool,
    pub redact_email_addresses: bool,
    pub redact_phone_numbers: bool,
    pub redact_credit_cards: bool,
    pub hash_sensitive_data: bool,
}
```

#### Content Security
- **Input Sanitization**: Sanitize all content received from browsers
- **XSS Prevention**: Prevent cross-site scripting attacks through content
- **Data Validation**: Validate all browser-provided data before processing
- **Secure Storage**: Encrypt stored context data and connection information

### Context Processing Pipeline

#### Smart Content Extraction
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ContextProcessor {
    pub extraction_engine: ExtractionEngine,
    pub content_analyzer: ContentAnalyzer,
    pub relevance_scorer: RelevanceScorer,
    pub summarization_engine: SummarizationEngine,
}

#[derive(Reflect)]
pub struct ExtractionEngine {
    pub text_extractors: HashMap<ContentType, TextExtractor>,
    pub metadata_extractors: HashMap<ContentType, MetadataExtractor>,
    pub structured_data_parsers: Vec<StructuredDataParser>,
}

#[derive(Reflect)]
pub enum TextExtractor {
    PlainText,
    ReadabilityBased,
    CustomSelector(String),
    AIPowered,
}

#[derive(Reflect)]
pub struct TabMetadata {
    pub favicon_url: Option<String>,
    pub language: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub canonical_url: Option<String>,
    pub open_graph: HashMap<String, String>,
}
```

#### Intelligent Context Selection
- **Relevance Filtering**: Filter context based on relevance to current AI tasks
- **Content Summarization**: Automatically summarize long content
- **Duplicate Detection**: Avoid extracting duplicate content from tabs
- **Quality Assessment**: Assess content quality and filter low-value content

### Performance Optimization

#### Efficient Data Transfer
- **Compression**: Compress context data before transmission
- **Incremental Updates**: Send only changed content, not full context
- **Selective Extraction**: Extract only necessary content based on user preferences
- **Background Processing**: Process context extraction without blocking browsers

#### Resource Management
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ResourceManager {
    pub memory_limits: MemoryLimits,
    pub processing_quotas: ProcessingQuotas,
    pub network_throttling: NetworkThrottling,
    pub cache_management: CacheManagement,
}

#[derive(Reflect)]
pub struct MemoryLimits {
    pub max_context_cache_size: usize,
    pub max_concurrent_connections: usize,
    pub max_tab_contexts: usize,
    pub context_size_limit: usize,
}
```

### Error Handling and Recovery

#### Connection Error Management
```rust
#[derive(Event)]
pub struct BrowserConnectionError {
    pub browser_id: String,
    pub error_type: ConnectionErrorType,
    pub error_message: String,
    pub recovery_action: RecoveryAction,
    pub timestamp: SystemTime,
}

#[derive(Reflect)]
pub enum ConnectionErrorType {
    ExtensionNotInstalled,
    ExtensionOutdated,
    PermissionDenied,
    NetworkError,
    ProtocolError,
    BrowserCrashed,
    SecurityViolation,
}

#[derive(Reflect)]
pub enum RecoveryAction {
    RetryConnection,
    UpdateExtension,
    RequestPermissions,
    ReinstallExtension,
    ContactSupport,
    DisableIntegration,
}
```

#### Graceful Degradation
- **Offline Mode**: Function without browser extension when unavailable
- **Partial Context**: Use available context when some browsers fail
- **Manual Context**: Allow manual context input when extension fails
- **Error Communication**: Clear user communication about extension issues

### Browser Extension Installation

#### Extension Management
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ExtensionInstallationManager {
    pub supported_browsers: Vec<BrowserSupport>,
    pub installation_status: HashMap<String, InstallationStatus>,
    pub version_management: VersionManager,
    pub update_notifications: UpdateNotificationSystem,
}

#[derive(Reflect)]
pub struct BrowserSupport {
    pub browser_type: BrowserType,
    pub minimum_version: String,
    pub extension_store_url: String,
    pub installation_instructions: Vec<String>,
    pub verification_method: VerificationMethod,
}
```

#### Installation Guidance
- **Browser Detection**: Automatically detect installed browsers
- **Installation Links**: Direct links to browser extension stores
- **Setup Instructions**: Step-by-step installation guidance
- **Verification**: Automatic verification of successful installation

### Testing and Validation

#### Cross-Browser Testing
- **Browser Compatibility**: Test with all major browser types and versions
- **Extension Protocol**: Test communication protocol reliability
- **Performance Impact**: Measure impact on browser performance
- **Security Testing**: Validate security of cross-browser communication

#### Context Extraction Testing
- **Content Accuracy**: Verify accurate extraction of various content types
- **Privacy Protection**: Test privacy filtering and data protection
- **Performance Testing**: Measure context extraction performance
- **Error Recovery**: Test recovery from various error scenarios

### Implementation Files
- `ai_menu_3/browser_extension.rs` - Core browser extension management system
- `ai_menu_3/extension_server.rs` - WebSocket server for browser communication
- `ai_menu_3/context_extractor.rs` - Content extraction and processing pipeline
- `ai_menu_3/privacy_processor.rs` - Privacy controls and content filtering
- `ai_menu_3/browser_protocols.rs` - Browser-specific communication protocols
- `ai_menu_3/connection_monitor.rs` - Connection status monitoring and display

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all context processing loops
- **Blazing-fast performance** - efficient cross-browser communication
- **Production quality** - secure, privacy-focused browser integration
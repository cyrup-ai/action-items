# ECS Notifications System - Architecture Design

## Overview

This document defines the sophisticated ECS (Entity-Component-System) architecture for an enterprise-grade notification system. The design incorporates patterns from Slack's distributed tracing, Discord's real-time pub/sub architecture, VS Code's process isolation, Teams' client data layer, and comprehensive platform integration (macOS UserNotifications, Windows Toast, Linux D-Bus, Web Push).

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          ECS Notification System                    │
├─────────────────────────────────────────────────────────────────────┤
│  Bevy Application Layer                                             │
│  ├─── Notification Builder API (Type-Safe)                         │
│  ├─── Event Systems (Creation, Interaction, Lifecycle)             │
│  └─── Query Systems (Analytics, State Management)                  │
├─────────────────────────────────────────────────────────────────────┤
│  ECS Core Components                                                │
│  ├─── NotificationEntity (Unique Identity + Correlation ID)        │
│  ├─── ContentComponent (Rich Media + Interaction Actions)          │
│  ├─── LifecycleComponent (State + Timing + Tracing)                │
│  ├─── PlatformComponent (Capabilities + Native Integration)        │
│  └─── AnalyticsComponent (Metrics + User Behavior)                 │
├─────────────────────────────────────────────────────────────────────┤
│  Platform Abstraction Layer                                        │
│  ├─── Capability Negotiation System                                │
│  ├─── Platform Feature Detection                                   │
│  ├─── Cross-Platform Event Translation                             │
│  └─── Graceful Degradation Management                              │
├─────────────────────────────────────────────────────────────────────┤
│  Native Platform Backends                                          │
│  ├─── macOS: UserNotifications + NSUserNotificationCenter          │
│  ├─── Windows: Toast XML + WinRT Integration                       │
│  ├─── Linux: D-Bus org.freedesktop.Notifications                   │
│  └─── Web: Service Worker + Push API + RFC 8030                    │
├─────────────────────────────────────────────────────────────────────┤
│  Enterprise Infrastructure                                         │
│  ├─── Distributed Tracing (OpenTelemetry + Correlation IDs)        │
│  ├─── Real-Time Event Bus (Pub/Sub + WebSocket)                    │
│  ├─── Attention Management Engine                                  │
│  ├─── Multi-Device Coordination                                    │
│  └─── Performance Monitoring + Analytics                           │
└─────────────────────────────────────────────────────────────────────┘
```

## 1. ECS Entity-Component Design

### 1.1 NotificationEntity Architecture

Each notification is represented as a unique Bevy entity with comprehensive tracking:

```rust
// Unique notification identity with enterprise tracing
#[derive(Component, Debug, Clone)]
pub struct NotificationIdentity {
    pub id: NotificationId,              // Unique notification identifier
    pub correlation_id: CorrelationId,   // For distributed tracing (Slack pattern)
    pub session_id: SessionId,           // User session tracking
    pub created_at: Instant,             // High-precision creation timestamp
    pub trace_span: TraceSpan,           // OpenTelemetry integration
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NotificationId(pub Uuid);

#[derive(Debug, Clone)]
pub struct CorrelationId(pub String);  // For cross-system tracing
```

### 1.2 ContentComponent - Rich Media and Interactions

Supports sophisticated content patterns discovered in enterprise systems:

```rust
#[derive(Component, Debug, Clone)]
pub struct NotificationContent {
    pub title: String,
    pub subtitle: Option<String>,        // macOS-specific
    pub body: RichText,                  // Support for markup/formatting
    pub media: Vec<MediaAttachment>,     // Images, sounds, videos
    pub interactions: InteractionSet,    // Buttons, inputs, quick replies
    pub category: NotificationCategory,  // For grouping and templates
    pub priority: Priority,              // Enterprise attention management
}

#[derive(Debug, Clone)]
pub enum MediaAttachment {
    Image {
        data: ImageData,
        placement: ImagePlacement,  // Icon, Hero, Inline, App Logo
        alt_text: Option<String>,   // Accessibility
    },
    Audio {
        sound: SoundSource,         // System, Custom File, URL
        loop_audio: bool,
        volume: f32,
    },
    Video {
        data: VideoData,
        thumbnail: Option<ImageData>,
    },
}

#[derive(Debug, Clone)]
pub struct InteractionSet {
    pub actions: Vec<NotificationAction>,
    pub inputs: Vec<NotificationInput>,
    pub quick_replies: Vec<QuickReply>,
    pub context_menu: Vec<ContextAction>,
}

// Rich interaction patterns from Slack/Discord/Teams
#[derive(Debug, Clone)]
pub enum NotificationAction {
    Button {
        id: ActionId,
        label: String,
        icon: Option<IconData>,
        style: ButtonStyle,         // Default, Destructive, Prominent
        activation: ActivationType,  // Foreground, Background, Protocol
    },
    Menu {
        id: ActionId,
        label: String,
        options: Vec<MenuOption>,
    },
    QuickAction {
        id: ActionId,
        icon: IconData,
        tooltip: String,
    },
}

#[derive(Debug, Clone)]
pub enum NotificationInput {
    Text {
        id: InputId,
        placeholder: String,
        validation: Option<InputValidation>,
        max_length: Option<usize>,
    },
    Selection {
        id: InputId,
        options: Vec<SelectionOption>,
        multi_select: bool,
    },
}
```

### 1.3 LifecycleComponent - State and Process Management

Comprehensive state tracking inspired by enterprise systems:

```rust
#[derive(Component, Debug, Clone)]
pub struct NotificationLifecycle {
    pub state: NotificationState,
    pub platform_state: HashMap<Platform, PlatformState>,
    pub timing: NotificationTiming,
    pub retry_policy: RetryPolicy,
    pub expiration: ExpirationPolicy,
    pub delivery_receipt: Option<DeliveryReceipt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationState {
    Created,              // Initial state
    Validating,          // Content validation and sanitization
    PlatformRouting,     // Capability negotiation
    Queued,              // Waiting for delivery
    Delivering,          // Active platform delivery
    Delivered,           // Successfully delivered
    InteractionPending,  // Waiting for user response
    InteractionReceived, // User interacted
    Expired,             // TTL exceeded
    Failed(ErrorDetails), // Delivery failed
    Cancelled,           // Cancelled before delivery
}

#[derive(Debug, Clone)]
pub struct NotificationTiming {
    pub created_at: Instant,
    pub scheduled_for: Option<Instant>,      // Scheduled notifications
    pub delivered_at: Option<Instant>,
    pub expires_at: Option<Instant>,         // TTL handling
    pub last_interaction: Option<Instant>,
    pub processing_duration: Option<Duration>,
}

// Enterprise retry patterns (exponential backoff, circuit breaker)
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub current_attempt: u32,
    pub backoff_strategy: BackoffStrategy,
    pub circuit_breaker_state: CircuitBreakerState,
}
```

### 1.4 PlatformComponent - Cross-Platform Integration

Handles platform-specific capabilities and native integration:

```rust
#[derive(Component, Debug, Clone)]
pub struct PlatformIntegration {
    pub target_platforms: Vec<Platform>,
    pub capabilities: PlatformCapabilities,
    pub native_handles: HashMap<Platform, NativeHandle>,
    pub feature_support: FeatureMatrix,
    pub degradation_strategy: DegradationStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    MacOS,
    Windows, 
    Linux,
    Web,
}

// Dynamic capability detection (from Linux D-Bus study)
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    pub supports_actions: bool,
    pub supports_rich_media: bool,
    pub supports_markup: bool,
    pub supports_sound: bool,
    pub supports_scheduling: bool,
    pub supports_progress: bool,
    pub max_actions: Option<u32>,
    pub max_content_length: Option<usize>,
}

// Native platform integration handles
#[derive(Debug)]
pub enum NativeHandle {
    MacOS {
        notification_center: Arc<UNUserNotificationCenter>,
        notification_request: Option<UNNotificationRequest>,
        delegate: Arc<NotificationDelegate>,
    },
    Windows {
        toast_manager: Arc<ToastNotificationManager>, 
        toast_xml: Option<XmlDocument>,
        notification: Option<ToastNotification>,
    },
    Linux {
        dbus_connection: Arc<Connection>,
        notification_id: Option<u32>,
        server_capabilities: Vec<String>,
    },
    Web {
        service_worker: ServiceWorker,
        push_subscription: Option<PushSubscription>,
        registration: ServiceWorkerRegistration,
    },
}
```

### 1.5 AnalyticsComponent - Enterprise Observability 

Comprehensive metrics and behavioral analytics:

```rust
#[derive(Component, Debug, Clone)]
pub struct NotificationAnalytics {
    pub metrics: NotificationMetrics,
    pub user_behavior: UserBehavior,
    pub performance: PerformanceMetrics,
    pub a_b_test: Option<ABTestData>,
    pub trace_data: DistributedTraceData,
}

#[derive(Debug, Clone)]
pub struct NotificationMetrics {
    pub delivery_attempts: u32,
    pub delivery_success: bool,
    pub delivery_latency: Duration,
    pub user_engagement: EngagementMetrics,
    pub platform_performance: HashMap<Platform, PlatformMetrics>,
}

#[derive(Debug, Clone)]
pub struct UserBehavior {
    pub interaction_type: Option<InteractionType>,
    pub response_time: Option<Duration>,        // Time to interact
    pub attention_context: AttentionContext,    // Device state, focus
    pub preference_signals: PreferenceSignals,  // Learning user patterns
}

// Distributed tracing integration (Slack's approach)
#[derive(Debug, Clone)]
pub struct DistributedTraceData {
    pub span_id: SpanId,
    pub trace_id: TraceId,
    pub parent_span: Option<SpanId>,
    pub service_chain: Vec<ServiceHop>,         // Cross-service tracking
    pub custom_attributes: HashMap<String, String>,
}
```

## 2. System Architecture - ECS Systems Design

### 2.1 Core Notification Systems

```rust
// Notification Creation and Validation System
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum NotificationSystemSet {
    Creation,           // Builder pattern and validation
    PlatformRouting,    // Capability negotiation
    Delivery,           // Platform-specific delivery
    InteractionHandling, // User response processing
    StateManagement,    // Lifecycle transitions
    Analytics,          // Metrics and tracing
    Cleanup,            // Resource cleanup
}

// Primary notification processing pipeline
pub fn notification_creation_system(
    mut commands: Commands,
    mut creation_events: EventReader<CreateNotificationEvent>,
    platform_manager: Res<PlatformManager>,
    trace_manager: Res<DistributedTraceManager>,
) {
    for event in creation_events.read() {
        // Create notification entity with full tracing
        let correlation_id = trace_manager.create_correlation_id();
        let trace_span = trace_manager.start_span("notification.create", &correlation_id);
        
        let notification_id = NotificationId::new();
        
        commands.spawn((
            NotificationIdentity {
                id: notification_id,
                correlation_id,
                session_id: event.session_id.clone(),
                created_at: Instant::now(),
                trace_span,
            },
            event.content.clone(),
            NotificationLifecycle::new(),
            PlatformIntegration::from_targets(&event.target_platforms, &platform_manager),
            NotificationAnalytics::new(),
        ));
        
        // Emit creation event for downstream processing
        trace_manager.record_event("notification.created", &notification_id);
    }
}

// Platform capability negotiation (Linux D-Bus inspired)
pub fn platform_capability_system(
    mut notifications: Query<(&NotificationIdentity, &NotificationContent, &mut PlatformIntegration)>,
    platform_manager: Res<PlatformManager>,
) {
    for (identity, content, mut platform) in notifications.iter_mut() {
        // Negotiate capabilities for each target platform
        for target_platform in &platform.target_platforms {
            let capabilities = platform_manager.get_capabilities(*target_platform);
            
            // Check feature support and plan degradation
            let feature_support = calculate_feature_support(content, &capabilities);
            platform.feature_support.insert(*target_platform, feature_support);
            
            if !capabilities.supports_actions && !content.interactions.actions.is_empty() {
                // Graceful degradation: convert actions to simple notification
                platform.degradation_strategy.apply_action_fallback();
            }
        }
    }
}
```

### 2.2 Platform-Specific Delivery Systems

```rust
// macOS UserNotifications delivery system
pub fn macos_delivery_system(
    mut notifications: Query<(
        &NotificationIdentity,
        &NotificationContent, 
        &mut NotificationLifecycle,
        &mut PlatformIntegration,
    ), With<MacOSDeliveryMarker>>,
    macos_backend: Res<MacOSNotificationBackend>,
    trace_manager: Res<DistributedTraceManager>,
) {
    for (identity, content, mut lifecycle, mut platform) in notifications.iter_mut() {
        if lifecycle.state == NotificationState::PlatformRouting {
            lifecycle.state = NotificationState::Delivering;
            
            // Create UNNotificationContent with rich media
            let notification_content = create_un_notification_content(content);
            
            // Handle interactive actions with UNNotificationCategory
            if !content.interactions.actions.is_empty() {
                let category = create_notification_category(&content.interactions);
                macos_backend.register_category(category);
                notification_content.set_category_identifier(&content.category.identifier);
            }
            
            // Create and schedule notification request
            let request = UNNotificationRequest::new(
                identity.id.to_string(),
                notification_content,
                None, // Immediate delivery, could be UNTimeIntervalNotificationTrigger
            );
            
            // Async delivery with callback handling
            let correlation_id = identity.correlation_id.clone();
            macos_backend.add_notification_request(request, move |error| {
                if let Some(error) = error {
                    trace_manager.record_error("notification.delivery.failed", &correlation_id, error);
                } else {
                    trace_manager.record_event("notification.delivered", &correlation_id);
                }
            });
        }
    }
}

// Windows Toast delivery system with XML generation
pub fn windows_delivery_system(
    mut notifications: Query<(
        &NotificationIdentity,
        &NotificationContent,
        &mut NotificationLifecycle, 
        &mut PlatformIntegration,
    ), With<WindowsDeliveryMarker>>,
    windows_backend: Res<WindowsNotificationBackend>,
) {
    for (identity, content, mut lifecycle, mut platform) in notifications.iter_mut() {
        if lifecycle.state == NotificationState::PlatformRouting {
            lifecycle.state = NotificationState::Delivering;
            
            // Generate adaptive Toast XML
            let toast_xml = create_adaptive_toast_xml(content);
            
            // Handle progress notifications
            if let Some(progress) = &content.progress {
                add_progress_elements(&mut toast_xml, progress);
            }
            
            // Create ToastNotification with proper activation handling
            let toast = windows_backend.create_toast_notification(toast_xml);
            
            // Set up background activation for enterprise apps
            if content.supports_background_activation() {
                toast.set_activation_type(ToastActivationType::Background);
            }
            
            windows_backend.show_toast_notification(toast);
            lifecycle.state = NotificationState::Delivered;
        }
    }
}
```

### 2.3 Interaction Response Systems

Complex user interaction handling inspired by Discord/Slack patterns:

```rust
// Universal interaction event processing
#[derive(Event, Debug, Clone)]
pub enum NotificationInteractionEvent {
    ActionPressed {
        notification_id: NotificationId,
        action_id: ActionId,
        activation_type: ActivationType,
    },
    InputSubmitted {
        notification_id: NotificationId,
        input_id: InputId,
        value: String,
    },
    MenuSelected {
        notification_id: NotificationId,
        menu_id: ActionId,
        selected_option: String,
    },
    NotificationClicked {
        notification_id: NotificationId,
        click_area: ClickArea,
    },
    NotificationClosed {
        notification_id: NotificationId,
        close_reason: CloseReason,
    },
}

// Sophisticated response routing system (Slack-inspired)
pub fn interaction_response_system(
    mut interaction_events: EventReader<NotificationInteractionEvent>,
    mut notifications: Query<(
        &NotificationIdentity,
        &NotificationContent,
        &mut NotificationLifecycle,
        &mut NotificationAnalytics,
    )>,
    mut response_senders: ResMut<ResponseSenderRegistry>,
    trace_manager: Res<DistributedTraceManager>,
) {
    for event in interaction_events.read() {
        if let Ok((identity, content, mut lifecycle, mut analytics)) = 
            notifications.get_mut(find_entity_by_id(event.notification_id())) {
            
            // Record interaction timing and type
            let interaction_time = Instant::now();
            let response_latency = interaction_time.duration_since(lifecycle.timing.delivered_at.unwrap_or(interaction_time));
            
            analytics.user_behavior.response_time = Some(response_latency);
            analytics.user_behavior.interaction_type = Some(event.interaction_type());
            
            // Process interaction based on type
            match event {
                NotificationInteractionEvent::ActionPressed { action_id, activation_type, .. } => {
                    // Find the specific action configuration
                    if let Some(action) = content.interactions.find_action(&action_id) {
                        match activation_type {
                            ActivationType::Foreground => {
                                // Bring app to foreground and route action
                                handle_foreground_activation(action);
                            }
                            ActivationType::Background => {
                                // Process action without UI disruption
                                handle_background_activation(action);
                            }
                            ActivationType::Protocol => {
                                // Launch external application
                                handle_protocol_activation(action);
                            }
                        }
                    }
                }
                NotificationInteractionEvent::InputSubmitted { input_id, value, .. } => {
                    // Validate input and route response
                    if let Some(input) = content.interactions.find_input(&input_id) {
                        if let Err(validation_error) = input.validate(&value) {
                            // Handle validation failure
                            send_validation_error_response(validation_error);
                        } else {
                            // Process valid input
                            process_input_response(input, value);
                        }
                    }
                }
                // ... handle other interaction types
            }
            
            // Send response to registered handlers using async channels
            if let Some(response_sender) = response_senders.get(&identity.id) {
                let response = NotificationResponse::new(event.clone(), interaction_time);
                if let Err(_) = response_sender.send(response).await {
                    trace_manager.record_warning(
                        "interaction.response.channel_closed", 
                        &identity.correlation_id
                    );
                }
            }
            
            lifecycle.state = NotificationState::InteractionReceived;
            lifecycle.timing.last_interaction = Some(interaction_time);
        }
    }
}
```

## 3. Enterprise Infrastructure Integration

### 3.1 Distributed Tracing Architecture

Implementing Slack's distributed tracing patterns:

```rust
#[derive(Resource)]
pub struct DistributedTraceManager {
    tracer: Arc<opentelemetry::Tracer>,
    correlation_registry: Arc<DashMap<CorrelationId, TraceContext>>,
    performance_collector: Arc<PerformanceCollector>,
}

impl DistributedTraceManager {
    pub fn start_notification_trace(&self, notification_id: NotificationId) -> TraceSpan {
        let trace_id = TraceId::new();
        let span = self.tracer.start("notification.lifecycle");
        
        span.set_attribute("notification.id", notification_id.to_string());
        span.set_attribute("service.name", "ecs-notifications");
        span.set_attribute("service.version", env!("CARGO_PKG_VERSION"));
        
        TraceSpan::new(span, trace_id)
    }
    
    pub fn record_platform_delivery(&self, correlation_id: &CorrelationId, platform: Platform, result: DeliveryResult) {
        let mut span = self.get_span(correlation_id);
        span.add_event(format!("platform.{}.delivery", platform.name().to_lowercase()));
        span.set_attribute("delivery.success", result.is_success());
        span.set_attribute("delivery.latency_ms", result.latency().as_millis() as i64);
        
        if let Err(error) = result {
            span.record_exception(&error);
            span.set_status(opentelemetry::trace::Status::error(error.to_string()));
        }
    }
}

// Performance monitoring system (Teams-inspired)
pub fn performance_monitoring_system(
    notifications: Query<(&NotificationIdentity, &NotificationLifecycle, &NotificationAnalytics)>,
    mut performance_metrics: ResMut<PerformanceMetrics>,
    time: Res<Time>,
) {
    for (identity, lifecycle, analytics) in notifications.iter() {
        // Collect real-time performance data
        if let Some(delivered_at) = lifecycle.timing.delivered_at {
            let delivery_latency = delivered_at.duration_since(lifecycle.timing.created_at);
            
            performance_metrics.record_delivery_latency(delivery_latency);
            performance_metrics.record_platform_performance(
                &analytics.metrics.platform_performance
            );
        }
        
        // Detect performance anomalies
        if let Some(response_time) = analytics.user_behavior.response_time {
            if response_time > Duration::from_secs(30) {
                performance_metrics.record_slow_interaction(identity.id, response_time);
            }
        }
    }
    
    // Periodic performance reporting
    if time.elapsed_seconds() % 60.0 < 0.1 { // Every minute
        performance_metrics.emit_performance_report();
    }
}
```

### 3.2 Attention Management Engine

Implementing Slack's sophisticated attention management:

```rust
#[derive(Resource)]
pub struct AttentionManager {
    user_context: Arc<RwLock<UserContext>>,
    device_coordinator: Arc<DeviceCoordinator>,
    notification_rules: Arc<NotificationRuleEngine>,
    learning_model: Arc<UserBehaviorModel>,
}

#[derive(Debug, Clone)]
pub struct UserContext {
    pub active_device: Option<DeviceType>,
    pub focus_state: FocusState,
    pub do_not_disturb: DnDSettings,
    pub current_activity: Option<UserActivity>,
    pub notification_fatigue: FatigueLevel,
}

pub fn attention_management_system(
    mut notifications: Query<(
        &NotificationIdentity, 
        &NotificationContent,
        &mut NotificationLifecycle,
    ), With<PendingDelivery>>,
    attention_manager: Res<AttentionManager>,
    time: Res<Time>,
) {
    for (identity, content, mut lifecycle) in notifications.iter_mut() {
        let user_context = attention_manager.user_context.read().unwrap();
        
        // Smart routing decision based on context
        let routing_decision = attention_manager.should_deliver_notification(
            &content.priority,
            &content.category,
            &user_context,
        );
        
        match routing_decision {
            RoutingDecision::DeliverImmediately => {
                lifecycle.state = NotificationState::PlatformRouting;
            }
            RoutingDecision::DelayUntil(delay_until) => {
                lifecycle.timing.scheduled_for = Some(delay_until);
                lifecycle.state = NotificationState::Queued;
            }
            RoutingDecision::BatchWithSimilar => {
                // Group with similar notifications for batch delivery
                lifecycle.state = NotificationState::Queued;
                attention_manager.add_to_batch(identity.id, &content.category);
            }
            RoutingDecision::SuppressDuplicate => {
                // Mark as delivered but don't actually show
                lifecycle.state = NotificationState::Delivered;
            }
        }
    }
}
```

## 4. Platform Integration Architecture

### 4.1 Unified Platform Backend System

```rust
#[async_trait]
pub trait NotificationBackend: Send + Sync {
    async fn negotiate_capabilities(&self) -> PlatformCapabilities;
    async fn deliver_notification(&self, request: NotificationRequest) -> DeliveryResult;
    async fn update_notification(&self, id: NotificationId, update: NotificationUpdate) -> Result<()>;
    async fn cancel_notification(&self, id: NotificationId) -> Result<()>;
    fn register_interaction_handler(&self, handler: Box<dyn InteractionHandler>);
}

// Platform-specific implementations with sophisticated patterns
#[derive(Resource)]
pub struct MacOSNotificationBackend {
    notification_center: Arc<UNUserNotificationCenter>,
    delegate: Arc<NotificationDelegate>,
    category_registry: Arc<CategoryRegistry>,
}

impl NotificationBackend for MacOSNotificationBackend {
    async fn negotiate_capabilities(&self) -> PlatformCapabilities {
        // Query system for actual capabilities
        let auth_status = self.notification_center.get_notification_settings().await;
        
        PlatformCapabilities {
            supports_actions: true,
            supports_rich_media: true,
            supports_markup: false, // macOS doesn't support HTML markup
            supports_sound: true,
            supports_scheduling: true,
            supports_progress: false, // Not directly supported
            max_actions: Some(4), // UNNotificationCategory limit
            max_content_length: Some(256), // Conservative estimate
            authorization_status: auth_status.into(),
        }
    }
    
    async fn deliver_notification(&self, request: NotificationRequest) -> DeliveryResult {
        let correlation_id = request.correlation_id.clone();
        
        // Create UNNotificationContent
        let content = UNMutableNotificationContent::new();
        content.set_title(&request.content.title);
        content.set_body(&request.content.body.to_plain_text());
        
        if let Some(subtitle) = &request.content.subtitle {
            content.set_subtitle(subtitle);
        }
        
        // Handle rich media attachments
        for media in &request.content.media {
            match media {
                MediaAttachment::Image { data, placement, .. } => {
                    match placement {
                        ImagePlacement::Icon => {
                            // Use app icon or custom icon
                            if let Some(icon_url) = data.as_url() {
                                content.set_icon_url(icon_url);
                            }
                        }
                        ImagePlacement::Hero => {
                            // Add as attachment for hero display
                            let attachment = UNNotificationAttachment::create(
                                "hero-image",
                                data.as_url()?,
                                None
                            )?;
                            content.set_attachments(&[attachment]);
                        }
                        _ => {}
                    }
                }
                MediaAttachment::Audio { sound, .. } => {
                    match sound {
                        SoundSource::System(system_sound) => {
                            content.set_sound(UNNotificationSound::from_system(*system_sound));
                        }
                        SoundSource::Custom(file_path) => {
                            content.set_sound(UNNotificationSound::from_file(file_path));
                        }
                        SoundSource::Default => {
                            content.set_sound(UNNotificationSound::default());
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Handle interactive actions
        if !request.content.interactions.actions.is_empty() {
            let category_id = format!("category_{}", request.notification_id);
            let category = self.create_notification_category(&category_id, &request.content.interactions);
            
            self.category_registry.register_category(category).await?;
            content.set_category_identifier(&category_id);
        }
        
        // Create notification request
        let notification_request = UNNotificationRequest::new(
            request.notification_id.to_string(),
            content,
            None, // Immediate delivery
        );
        
        // Deliver with completion tracking
        let (tx, rx) = oneshot::channel();
        
        self.notification_center.add_notification_request(notification_request, move |error| {
            let result = if let Some(error) = error {
                DeliveryResult::Failed(error.into())
            } else {
                DeliveryResult::Success(DeliveryMetrics::new())
            };
            let _ = tx.send(result);
        });
        
        rx.await.map_err(|_| DeliveryError::ResponseChannelClosed)?
    }
}
```

This architecture provides the enterprise-grade sophistication and complexity that matches production notification systems. It embraces the necessary complexity rather than oversimplifying, incorporating patterns from all the major systems studied (Slack, Discord, VS Code, Teams) and comprehensive platform integration.

The design supports:
- Rich interactive notifications with complex user flows  
- Cross-platform capability negotiation and graceful degradation
- Enterprise-scale distributed tracing and observability
- Sophisticated attention management and user behavior learning
- Real-time pub/sub patterns for high-frequency scenarios
- Comprehensive error handling and retry mechanisms
- Multi-device coordination and smart routing
- Security and privacy protection throughout

This represents the level of architectural sophistication that enterprise notification systems require.
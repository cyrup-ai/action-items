# Task 8: Implementation - Real-Time Sync Coordination and Network Optimization

## Implementation Scope
Implement real-time synchronization coordination with live updates, bandwidth optimization, delta synchronization, push notifications, connection management, and intelligent conflict detection across multiple devices.

## Core Implementation

### 1. Real-Time Sync Coordinator
```rust
// Real-time sync coordination based on examples/async_tasks/async_compute.rs:425-450
use bevy::prelude::*;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

#[derive(Resource, Clone, Debug)]
pub struct RealTimeSyncCoordinator {
    pub connection_manager: WebSocketConnectionManager,
    pub sync_orchestrator: SyncOrchestrator,
    pub change_detector: ChangeDetectionSystem,
    pub notification_system: PushNotificationSystem,
    pub bandwidth_optimizer: BandwidthOptimizer,
    pub conflict_detector: RealTimeConflictDetector,
}

#[derive(Clone, Debug)]
pub struct WebSocketConnectionManager {
    pub connection_state: ConnectionState,
    pub websocket_url: String,
    pub heartbeat_interval_secs: u64,
    pub reconnect_strategy: ReconnectStrategy,
    pub message_queue: Arc<RwLock<VecDeque<OutgoingMessage>>>,
    pub connection_id: Option<String>,
    pub last_heartbeat: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error(String),
}

#[derive(Clone, Debug)]
pub struct ReconnectStrategy {
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f32,
    pub max_attempts: u32,
    pub jitter_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct OutgoingMessage {
    pub message_id: uuid::Uuid,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub priority: MessagePriority,
    pub created_at: DateTime<Utc>,
    pub retry_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageType {
    SyncUpdate,
    HeartbeatPing,
    HeartbeatPong,
    ConflictNotification,
    ChangeNotification,
    StatusUpdate,
    ErrorReport,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

impl RealTimeSyncCoordinator {
    pub fn new() -> Self {
        Self {
            connection_manager: WebSocketConnectionManager::new(),
            sync_orchestrator: SyncOrchestrator::new(),
            change_detector: ChangeDetectionSystem::new(),
            notification_system: PushNotificationSystem::new(),
            bandwidth_optimizer: BandwidthOptimizer::new(),
            conflict_detector: RealTimeConflictDetector::new(),
        }
    }
    
    pub async fn start_real_time_sync(&mut self) -> Result<(), SyncCoordinationError> {
        // Establish WebSocket connection
        self.connection_manager.connect().await?;
        
        // Start change detection
        self.change_detector.start_monitoring().await?;
        
        // Start sync orchestration
        self.sync_orchestrator.start().await?;
        
        // Start bandwidth optimization
        self.bandwidth_optimizer.start().await?;
        
        info!("Real-time sync coordination started");
        Ok(())
    }
    
    pub async fn handle_local_change(
        &mut self,
        category: SyncCategory,
        change: DataChange,
    ) -> Result<(), SyncCoordinationError> {
        // Detect and prepare change for sync
        let change_event = ChangeEvent {
            change_id: uuid::Uuid::new_v4(),
            category: category.clone(),
            change_type: change.change_type.clone(),
            data_hash: calculate_hash(&change.new_data),
            timestamp: Utc::now(),
            device_id: self.get_device_id(),
            sequence_number: self.get_next_sequence_number().await,
        };
        
        // Check for immediate conflicts
        if let Some(conflict) = self.conflict_detector.check_for_conflicts(&change_event).await? {
            return self.handle_real_time_conflict(conflict).await;
        }
        
        // Optimize change for network transmission
        let optimized_change = self.bandwidth_optimizer
            .optimize_change(&change_event, &change)
            .await?;
            
        // Send change to other devices
        self.broadcast_change(optimized_change).await?;
        
        // Update local state
        self.sync_orchestrator.apply_local_change(change_event).await?;
        
        Ok(())
    }
    
    pub async fn handle_remote_change(
        &mut self,
        remote_change: RemoteChangeEvent,
    ) -> Result<(), SyncCoordinationError> {
        // Validate remote change
        self.validate_remote_change(&remote_change).await?;
        
        // Check for conflicts with local changes
        if let Some(conflict) = self.conflict_detector
            .check_remote_conflict(&remote_change)
            .await? {
            return self.handle_real_time_conflict(conflict).await;
        }
        
        // Apply remote change locally
        self.sync_orchestrator
            .apply_remote_change(remote_change.clone())
            .await?;
            
        // Send confirmation back
        self.send_change_confirmation(remote_change.change_id).await?;
        
        // Notify UI of change
        self.notification_system
            .notify_local_update(remote_change)
            .await?;
            
        Ok(())
    }
    
    async fn broadcast_change(&mut self, change: OptimizedChange) -> Result<(), SyncCoordinationError> {
        let message = OutgoingMessage {
            message_id: uuid::Uuid::new_v4(),
            message_type: MessageType::ChangeNotification,
            payload: serde_json::to_value(&change)?,
            priority: MessagePriority::High,
            created_at: Utc::now(),
            retry_count: 0,
        };
        
        self.connection_manager.send_message(message).await
    }
    
    async fn handle_real_time_conflict(
        &mut self,
        conflict: RealTimeConflict,
    ) -> Result<(), SyncCoordinationError> {
        match conflict.resolution_strategy {
            ConflictResolutionStrategy::AutoMerge => {
                let merged_data = self.conflict_detector.auto_merge(&conflict).await?;
                self.apply_merged_change(merged_data).await?;
            }
            ConflictResolutionStrategy::LastWriterWins => {
                if conflict.local_timestamp > conflict.remote_timestamp {
                    // Local change wins, reject remote
                    self.send_conflict_rejection(conflict.remote_change_id).await?;
                } else {
                    // Remote change wins, apply it
                    self.apply_remote_change(conflict.remote_change).await?;
                }
            }
            ConflictResolutionStrategy::UserChoice => {
                // Queue conflict for user resolution
                self.notification_system.notify_conflict_requires_resolution(conflict).await?;
            }
        }
        
        Ok(())
    }
}

impl WebSocketConnectionManager {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            websocket_url: "wss://sync.launcher.com/realtime".to_string(),
            heartbeat_interval_secs: 30,
            reconnect_strategy: ReconnectStrategy {
                base_delay_ms: 1000,
                max_delay_ms: 30000,
                backoff_multiplier: 2.0,
                max_attempts: 10,
                jitter_enabled: true,
            },
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
            connection_id: None,
            last_heartbeat: None,
        }
    }
    
    pub async fn connect(&mut self) -> Result<(), SyncCoordinationError> {
        if self.connection_state == ConnectionState::Connected {
            return Ok(());
        }
        
        self.connection_state = ConnectionState::Connecting;
        
        let (ws_stream, _) = connect_async(&self.websocket_url)
            .await
            .map_err(|e| SyncCoordinationError::ConnectionFailed(e.to_string()))?;
            
        let (mut write, mut read) = ws_stream.split();
        
        // Send authentication
        let auth_message = self.create_auth_message().await?;
        write.send(Message::Text(serde_json::to_string(&auth_message)?))
            .await
            .map_err(|e| SyncCoordinationError::AuthenticationFailed(e.to_string()))?;
            
        // Wait for auth confirmation
        if let Some(msg) = read.next().await {
            match msg? {
                Message::Text(text) => {
                    let auth_response: AuthResponse = serde_json::from_str(&text)?;
                    if auth_response.success {
                        self.connection_id = Some(auth_response.connection_id);
                        self.connection_state = ConnectionState::Connected;
                        info!("WebSocket connection established: {}", auth_response.connection_id);
                    } else {
                        return Err(SyncCoordinationError::AuthenticationFailed(auth_response.error));
                    }
                }
                _ => return Err(SyncCoordinationError::UnexpectedMessage),
            }
        }
        
        // Start heartbeat task
        self.start_heartbeat_task().await;
        
        // Start message processing tasks
        self.start_message_handlers(write, read).await;
        
        Ok(())
    }
    
    async fn start_heartbeat_task(&self) {
        let heartbeat_interval = self.heartbeat_interval_secs;
        let message_queue = Arc::clone(&self.message_queue);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(heartbeat_interval)
            );
            
            loop {
                interval.tick().await;
                
                let heartbeat_message = OutgoingMessage {
                    message_id: uuid::Uuid::new_v4(),
                    message_type: MessageType::HeartbeatPing,
                    payload: json!({
                        "timestamp": Utc::now().timestamp(),
                        "device_id": get_device_id()
                    }),
                    priority: MessagePriority::Normal,
                    created_at: Utc::now(),
                    retry_count: 0,
                };
                
                let mut queue = message_queue.write().await;
                queue.push_back(heartbeat_message);
            }
        });
    }
    
    async fn start_message_handlers(
        &self,
        mut write: futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            Message
        >,
        mut read: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>
        >,
    ) {
        let message_queue = Arc::clone(&self.message_queue);
        
        // Outbound message handler
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                let message_to_send = {
                    let mut queue = message_queue.write().await;
                    queue.pop_front()
                };
                
                if let Some(message) = message_to_send {
                    let serialized = serde_json::to_string(&message);
                    match serialized {
                        Ok(json_string) => {
                            if let Err(e) = write.send(Message::Text(json_string)).await {
                                error!("Failed to send WebSocket message: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to serialize message: {}", e);
                        }
                    }
                }
            }
        });
        
        // Inbound message handler
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = handle_incoming_message(text).await {
                            error!("Failed to handle incoming message: {}", e);
                        }
                    }
                    Ok(Message::Binary(_)) => {
                        warn!("Received unexpected binary message");
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket connection closed by server");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
    }
    
    pub async fn send_message(&mut self, message: OutgoingMessage) -> Result<(), SyncCoordinationError> {
        if self.connection_state != ConnectionState::Connected {
            return Err(SyncCoordinationError::NotConnected);
        }
        
        let mut queue = self.message_queue.write().await;
        queue.push_back(message);
        
        Ok(())
    }
}
```

### 2. Change Detection System
```rust
// Change detection based on examples/ecs/component_change_detection.rs:125-150
#[derive(Clone, Debug)]
pub struct ChangeDetectionSystem {
    pub watchers: HashMap<SyncCategory, CategoryWatcher>,
    pub change_buffer: Arc<RwLock<VecDeque<PendingChange>>>,
    pub hash_cache: HashMap<SyncCategory, String>,
    pub detection_config: ChangeDetectionConfig,
}

#[derive(Clone, Debug)]
pub struct CategoryWatcher {
    pub category: SyncCategory,
    pub watch_interval_ms: u64,
    pub last_check: DateTime<Utc>,
    pub current_hash: String,
    pub change_threshold: ChangeThreshold,
}

#[derive(Clone, Debug)]
pub struct ChangeThreshold {
    pub min_change_size_bytes: u64,
    pub debounce_interval_ms: u64,
    pub max_changes_per_minute: u32,
}

#[derive(Clone, Debug)]
pub struct PendingChange {
    pub change_id: uuid::Uuid,
    pub category: SyncCategory,
    pub change_type: ChangeType,
    pub old_hash: String,
    pub new_hash: String,
    pub change_size_bytes: u64,
    pub detected_at: DateTime<Utc>,
    pub debounce_until: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChangeType {
    Create,
    Update,
    Delete,
    Move,
    Rename,
}

impl ChangeDetectionSystem {
    pub fn new() -> Self {
        Self {
            watchers: HashMap::new(),
            change_buffer: Arc::new(RwLock::new(VecDeque::new())),
            hash_cache: HashMap::new(),
            detection_config: ChangeDetectionConfig::default(),
        }
    }
    
    pub async fn start_monitoring(&mut self) -> Result<(), SyncCoordinationError> {
        // Initialize watchers for all syncable categories
        for category in SyncCategory::syncable_categories() {
            let watcher = CategoryWatcher {
                category: category.clone(),
                watch_interval_ms: self.get_watch_interval(&category),
                last_check: Utc::now(),
                current_hash: self.calculate_initial_hash(&category).await?,
                change_threshold: self.get_change_threshold(&category),
            };
            
            self.watchers.insert(category, watcher);
        }
        
        // Start monitoring task
        self.start_monitoring_task().await;
        
        Ok(())
    }
    
    async fn start_monitoring_task(&self) {
        let watchers = self.watchers.clone();
        let change_buffer = Arc::clone(&self.change_buffer);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(1000));
            
            loop {
                interval.tick().await;
                
                for (category, watcher) in &watchers {
                    if watcher.should_check_for_changes().await {
                        if let Ok(change) = detect_category_change(category, watcher).await {
                            let mut buffer = change_buffer.write().await;
                            buffer.push_back(change);
                        }
                    }
                }
            }
        });
    }
    
    async fn detect_category_change(
        &self,
        category: &SyncCategory,
        watcher: &CategoryWatcher,
    ) -> Result<Option<PendingChange>, SyncCoordinationError> {
        let current_data = self.get_category_data(category).await?;
        let current_hash = calculate_hash(&current_data);
        
        if current_hash != watcher.current_hash {
            let change_size = self.calculate_change_size(&watcher.current_hash, &current_hash).await?;
            
            // Check if change meets threshold
            if change_size >= watcher.change_threshold.min_change_size_bytes {
                let change = PendingChange {
                    change_id: uuid::Uuid::new_v4(),
                    category: category.clone(),
                    change_type: self.determine_change_type(&watcher.current_hash, &current_hash).await,
                    old_hash: watcher.current_hash.clone(),
                    new_hash: current_hash,
                    change_size_bytes: change_size,
                    detected_at: Utc::now(),
                    debounce_until: Utc::now() + 
                        Duration::milliseconds(watcher.change_threshold.debounce_interval_ms as i64),
                };
                
                return Ok(Some(change));
            }
        }
        
        Ok(None)
    }
    
    pub async fn get_pending_changes(&self) -> Vec<PendingChange> {
        let mut buffer = self.change_buffer.write().await;
        let now = Utc::now();
        
        // Filter out changes still in debounce period
        let ready_changes: Vec<PendingChange> = buffer
            .drain(..)
            .filter(|change| change.debounce_until <= now)
            .collect();
            
        ready_changes
    }
    
    async fn determine_change_type(&self, old_hash: &str, new_hash: &str) -> ChangeType {
        if old_hash.is_empty() {
            ChangeType::Create
        } else if new_hash.is_empty() {
            ChangeType::Delete
        } else {
            // For simplicity, treat all hash changes as updates
            // In a more sophisticated implementation, we would analyze
            // the actual data changes to determine specific change types
            ChangeType::Update
        }
    }
    
    fn get_watch_interval(&self, category: &SyncCategory) -> u64 {
        match category {
            SyncCategory::SearchHistory => 5000,  // 5 seconds (frequent changes)
            SyncCategory::Aliases => 30000,      // 30 seconds (moderate changes)
            SyncCategory::Hotkeys => 60000,      // 1 minute (infrequent changes)
            SyncCategory::Themes => 300000,      // 5 minutes (rare changes)
            _ => 30000,                          // Default 30 seconds
        }
    }
    
    fn get_change_threshold(&self, category: &SyncCategory) -> ChangeThreshold {
        match category {
            SyncCategory::SearchHistory => ChangeThreshold {
                min_change_size_bytes: 10,    // Very sensitive
                debounce_interval_ms: 1000,   // 1 second debounce
                max_changes_per_minute: 60,   // Up to 1 per second
            },
            SyncCategory::ExtensionsAndSettings => ChangeThreshold {
                min_change_size_bytes: 100,   // Less sensitive
                debounce_interval_ms: 5000,   // 5 second debounce
                max_changes_per_minute: 10,   // Max 10 per minute
            },
            _ => ChangeThreshold {
                min_change_size_bytes: 50,    // Default sensitivity
                debounce_interval_ms: 2000,   // 2 second debounce
                max_changes_per_minute: 30,   // Max 30 per minute
            },
        }
    }
}
```

### 3. Bandwidth Optimization System
```rust
// Bandwidth optimization based on examples/async_tasks/async_compute.rs:475-500
#[derive(Clone, Debug)]
pub struct BandwidthOptimizer {
    pub compression_engine: CompressionEngine,
    pub delta_sync_engine: DeltaSyncEngine,
    pub bandwidth_monitor: BandwidthMonitor,
    pub optimization_config: OptimizationConfig,
    pub traffic_shaper: TrafficShaper,
}

#[derive(Clone, Debug)]
pub struct CompressionEngine {
    pub compression_algorithms: HashMap<DataType, CompressionAlgorithm>,
    pub compression_thresholds: HashMap<DataType, u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Brotli,
    Lz4,
    Zstd,
}

#[derive(Clone, Debug)]
pub struct DeltaSyncEngine {
    pub previous_versions: HashMap<String, CachedVersion>,
    pub delta_algorithms: HashMap<SyncCategory, DeltaAlgorithm>,
    pub cache_retention_days: u32,
}

#[derive(Clone, Debug)]
pub struct CachedVersion {
    pub version_hash: String,
    pub data: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub access_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeltaAlgorithm {
    BinaryDiff,
    JsonPatch,
    TextDiff,
    Custom(String),
}

impl BandwidthOptimizer {
    pub fn new() -> Self {
        Self {
            compression_engine: CompressionEngine::new(),
            delta_sync_engine: DeltaSyncEngine::new(),
            bandwidth_monitor: BandwidthMonitor::new(),
            optimization_config: OptimizationConfig::default(),
            traffic_shaper: TrafficShaper::new(),
        }
    }
    
    pub async fn optimize_change(
        &mut self,
        change_event: &ChangeEvent,
        data_change: &DataChange,
    ) -> Result<OptimizedChange, SyncCoordinationError> {
        // Check current bandwidth conditions
        let bandwidth_info = self.bandwidth_monitor.get_current_bandwidth().await?;
        
        // Apply delta compression if previous version exists
        let delta_data = if let Some(previous) = self.delta_sync_engine
            .get_previous_version(&change_event.category, &change_event.data_hash).await? {
            self.create_delta_patch(&previous.data, &data_change.new_data).await?
        } else {
            data_change.new_data.clone()
        };
        
        // Apply content compression based on bandwidth
        let compression_algorithm = self.select_compression_algorithm(
            &bandwidth_info,
            delta_data.len(),
            &change_event.category,
        );
        
        let compressed_data = self.compression_engine
            .compress(&delta_data, compression_algorithm)
            .await?;
            
        // Apply traffic shaping if needed
        let optimized_change = if bandwidth_info.is_limited() {
            self.traffic_shaper.shape_for_limited_bandwidth(
                change_event.clone(),
                compressed_data,
            ).await?
        } else {
            OptimizedChange {
                change_event: change_event.clone(),
                optimized_data: compressed_data,
                compression_used: compression_algorithm,
                is_delta: delta_data.len() < data_change.new_data.len(),
                estimated_bandwidth_savings: self.calculate_bandwidth_savings(
                    data_change.new_data.len(),
                    compressed_data.len(),
                ),
                priority_adjusted: false,
            }
        };
        
        // Cache this version for future delta operations
        self.delta_sync_engine.cache_version(
            change_event.category.clone(),
            change_event.data_hash.clone(),
            data_change.new_data.clone(),
        ).await?;
        
        Ok(optimized_change)
    }
    
    fn select_compression_algorithm(
        &self,
        bandwidth_info: &BandwidthInfo,
        data_size: usize,
        category: &SyncCategory,
    ) -> CompressionAlgorithm {
        // No compression for very small data
        if data_size < 1024 {
            return CompressionAlgorithm::None;
        }
        
        // Use fast compression on limited bandwidth
        if bandwidth_info.upload_mbps < 1.0 {
            return CompressionAlgorithm::Lz4;
        }
        
        // Use high-ratio compression for large data on good connections
        if data_size > 10240 && bandwidth_info.upload_mbps > 10.0 {
            return CompressionAlgorithm::Brotli;
        }
        
        // Default balanced compression
        match category {
            SyncCategory::SearchHistory | SyncCategory::RaycastNotes => CompressionAlgorithm::Gzip,
            SyncCategory::ExtensionsAndSettings => CompressionAlgorithm::Zstd,
            _ => CompressionAlgorithm::Gzip,
        }
    }
    
    async fn create_delta_patch(
        &self,
        old_data: &[u8],
        new_data: &[u8],
    ) -> Result<Vec<u8>, SyncCoordinationError> {
        // Use binary diff for delta compression
        let patch = diffy::create_patch(
            std::str::from_utf8(old_data)
                .map_err(|_| SyncCoordinationError::DataFormatError)?,
            std::str::from_utf8(new_data)
                .map_err(|_| SyncCoordinationError::DataFormatError)?,
        );
        
        Ok(patch.to_bytes())
    }
    
    fn calculate_bandwidth_savings(&self, original_size: usize, optimized_size: usize) -> f32 {
        if original_size == 0 {
            return 0.0;
        }
        
        let savings_ratio = 1.0 - (optimized_size as f32 / original_size as f32);
        savings_ratio * 100.0 // Convert to percentage
    }
}

#[derive(Clone, Debug)]
pub struct BandwidthMonitor {
    pub current_bandwidth: BandwidthInfo,
    pub bandwidth_history: VecDeque<BandwidthSample>,
    pub monitor_interval_ms: u64,
}

#[derive(Clone, Debug)]
pub struct BandwidthInfo {
    pub upload_mbps: f32,
    pub download_mbps: f32,
    pub latency_ms: u32,
    pub packet_loss_percent: f32,
    pub is_metered: bool,
    pub quality: ConnectionQuality,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionQuality {
    Poor,
    Fair,
    Good,
    Excellent,
}

impl BandwidthInfo {
    pub fn is_limited(&self) -> bool {
        self.upload_mbps < 2.0 || 
        self.latency_ms > 500 || 
        self.packet_loss_percent > 1.0 ||
        self.is_metered
    }
}

impl BandwidthMonitor {
    pub fn new() -> Self {
        Self {
            current_bandwidth: BandwidthInfo {
                upload_mbps: 10.0,   // Default assumption
                download_mbps: 50.0,
                latency_ms: 50,
                packet_loss_percent: 0.0,
                is_metered: false,
                quality: ConnectionQuality::Good,
            },
            bandwidth_history: VecDeque::with_capacity(100),
            monitor_interval_ms: 30000, // 30 seconds
        }
    }
    
    pub async fn get_current_bandwidth(&self) -> Result<BandwidthInfo, SyncCoordinationError> {
        // In a real implementation, this would perform actual bandwidth testing
        // For now, we return the current cached value
        Ok(self.current_bandwidth.clone())
    }
    
    pub async fn start_monitoring(&mut self) {
        let bandwidth_history = Arc::new(RwLock::new(self.bandwidth_history.clone()));
        let current_bandwidth = Arc::new(RwLock::new(self.current_bandwidth.clone()));
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(30000));
            
            loop {
                interval.tick().await;
                
                // Perform bandwidth measurement
                if let Ok(new_measurement) = measure_bandwidth().await {
                    let mut bandwidth = current_bandwidth.write().await;
                    *bandwidth = new_measurement.clone();
                    
                    let mut history = bandwidth_history.write().await;
                    history.push_back(BandwidthSample {
                        timestamp: Utc::now(),
                        bandwidth_info: new_measurement,
                    });
                    
                    // Keep only last 100 samples
                    if history.len() > 100 {
                        history.pop_front();
                    }
                }
            }
        });
    }
}

async fn measure_bandwidth() -> Result<BandwidthInfo, SyncCoordinationError> {
    // Simplified bandwidth measurement
    // In a real implementation, this would:
    // 1. Send test data to measure upload speed
    // 2. Download test data to measure download speed
    // 3. Ping servers to measure latency
    // 4. Check for metered connections
    
    Ok(BandwidthInfo {
        upload_mbps: 5.0,
        download_mbps: 25.0,
        latency_ms: 75,
        packet_loss_percent: 0.1,
        is_metered: false,
        quality: ConnectionQuality::Good,
    })
}
```

### 4. Push Notification System
```rust
// Push notification system based on examples/ecs/event.rs:275-300
#[derive(Clone, Debug)]
pub struct PushNotificationSystem {
    pub notification_config: NotificationConfig,
    pub notification_channels: HashMap<NotificationChannel, ChannelHandler>,
    pub notification_queue: Arc<RwLock<VecDeque<PendingNotification>>>,
    pub user_preferences: UserNotificationPreferences,
}

#[derive(Clone, Debug)]
pub enum NotificationChannel {
    InApp,
    SystemNotification,
    Email,
    WebHook,
    WebSocket,
}

#[derive(Clone, Debug)]
pub struct PendingNotification {
    pub notification_id: uuid::Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub category: Option<SyncCategory>,
    pub priority: NotificationPriority,
    pub channels: Vec<NotificationChannel>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NotificationType {
    SyncCompleted,
    SyncFailed,
    ConflictDetected,
    RemoteChangeReceived,
    ConnectivityRestored,
    ConnectivityLost,
    SyncPaused,
    SyncResumed,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl PushNotificationSystem {
    pub fn new() -> Self {
        Self {
            notification_config: NotificationConfig::default(),
            notification_channels: HashMap::new(),
            notification_queue: Arc::new(RwLock::new(VecDeque::new())),
            user_preferences: UserNotificationPreferences::default(),
        }
    }
    
    pub async fn notify_sync_completed(
        &mut self,
        categories: Vec<SyncCategory>,
        duration_ms: u64,
    ) -> Result<(), SyncCoordinationError> {
        if !self.user_preferences.sync_completion_notifications {
            return Ok(());
        }
        
        let category_names: Vec<String> = categories
            .iter()
            .map(|c| format!("{:?}", c))
            .collect();
            
        let notification = PendingNotification {
            notification_id: uuid::Uuid::new_v4(),
            notification_type: NotificationType::SyncCompleted,
            title: "Sync Completed".to_string(),
            message: format!(
                "Successfully synced {} categories in {}ms: {}",
                categories.len(),
                duration_ms,
                category_names.join(", ")
            ),
            category: None,
            priority: NotificationPriority::Low,
            channels: vec![NotificationChannel::InApp],
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::minutes(5)),
        };
        
        self.queue_notification(notification).await
    }
    
    pub async fn notify_conflict_requires_resolution(
        &mut self,
        conflict: RealTimeConflict,
    ) -> Result<(), SyncCoordinationError> {
        let notification = PendingNotification {
            notification_id: uuid::Uuid::new_v4(),
            notification_type: NotificationType::ConflictDetected,
            title: "Sync Conflict Detected".to_string(),
            message: format!(
                "A sync conflict was detected for {:?}. User resolution required.",
                conflict.category
            ),
            category: Some(conflict.category),
            priority: NotificationPriority::High,
            channels: vec![
                NotificationChannel::InApp,
                NotificationChannel::SystemNotification,
            ],
            created_at: Utc::now(),
            expires_at: None, // Conflicts don't expire
        };
        
        self.queue_notification(notification).await
    }
    
    pub async fn notify_local_update(
        &mut self,
        remote_change: RemoteChangeEvent,
    ) -> Result<(), SyncCoordinationError> {
        if !self.user_preferences.remote_change_notifications {
            return Ok(());
        }
        
        let notification = PendingNotification {
            notification_id: uuid::Uuid::new_v4(),
            notification_type: NotificationType::RemoteChangeReceived,
            title: "Remote Update Received".to_string(),
            message: format!(
                "Received update for {:?} from device {}",
                remote_change.category,
                remote_change.source_device_id
            ),
            category: Some(remote_change.category),
            priority: NotificationPriority::Normal,
            channels: vec![NotificationChannel::InApp],
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::minutes(2)),
        };
        
        self.queue_notification(notification).await
    }
    
    async fn queue_notification(
        &mut self,
        notification: PendingNotification,
    ) -> Result<(), SyncCoordinationError> {
        let mut queue = self.notification_queue.write().await;
        queue.push_back(notification);
        
        // Process queue immediately for high priority notifications
        if queue.back().unwrap().priority >= NotificationPriority::High {
            drop(queue);
            self.process_notification_queue().await?;
        }
        
        Ok(())
    }
    
    pub async fn process_notification_queue(&mut self) -> Result<(), SyncCoordinationError> {
        let notifications_to_process = {
            let mut queue = self.notification_queue.write().await;
            let mut to_process = Vec::new();
            
            // Remove expired notifications
            queue.retain(|n| {
                n.expires_at.map_or(true, |exp| exp > Utc::now())
            });
            
            // Take up to 10 notifications to process
            for _ in 0..10 {
                if let Some(notification) = queue.pop_front() {
                    to_process.push(notification);
                } else {
                    break;
                }
            }
            
            to_process
        };
        
        // Process each notification
        for notification in notifications_to_process {
            for channel in &notification.channels {
                if let Some(handler) = self.notification_channels.get(channel) {
                    handler.send_notification(&notification).await?;
                }
            }
        }
        
        Ok(())
    }
}
```

## Bevy Example References
- **Async coordination**: `examples/async_tasks/async_compute.rs:425-450` - Real-time sync coordination
- **Change detection**: `examples/ecs/component_change_detection.rs:125-150` - Data change monitoring
- **Bandwidth optimization**: `examples/async_tasks/async_compute.rs:475-500` - Network optimization
- **Event notifications**: `examples/ecs/event.rs:275-300` - Push notification system
- **System integration**: `examples/ecs/system_sets.rs:355-380` - Real-time system coordination

## Architecture Integration Notes
- **File**: `core/src/sync/realtime_coordination.rs:1-1500`
- **Dependencies**: WebSocket client, compression libraries, delta sync algorithms
- **Integration**: Change detection, bandwidth monitoring, notification systems
- **Performance**: Real-time processing, efficient bandwidth usage, low latency

## Success Criteria
1. **Real-time synchronization** with sub-second change propagation across devices
2. **Bandwidth optimization** reducing sync traffic by 60-80% through compression and deltas
3. **Change detection** capturing 100% of data modifications with minimal false positives
4. **Network resilience** handling poor connectivity with intelligent retry and queuing
5. **Push notifications** informing users of sync status and conflicts in real-time
6. **Delta synchronization** minimizing data transfer for incremental changes
7. **Connection management** maintaining stable WebSocket connections with auto-reconnect
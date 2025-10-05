# Task 4: Implementation - Cloud Sync Engine Integration

## Implementation Scope
Implement the core cloud synchronization engine that handles selective data synchronization, conflict resolution, encryption, offline queue management, and real-time sync coordination with the UI components.

## Core Implementation

### 1. Sync Engine Core System
```rust
// Cloud sync engine based on examples/async_tasks/async_compute.rs:25-50
use bevy::prelude::*;
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Resource, Clone, Debug)]
pub struct CloudSyncEngine {
    pub sync_state: SyncEngineState,
    pub sync_queue: Arc<RwLock<SyncQueue>>,
    pub conflict_resolver: ConflictResolver,
    pub encryption_manager: EncryptionManager,
    pub network_manager: NetworkManager,
    pub data_store: CloudDataStore,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SyncEngineState {
    Idle,
    Syncing,
    Paused,
    Error(SyncError),
    Offline,
}

#[derive(Debug, Clone)]
pub struct SyncQueue {
    pub pending_operations: VecDeque<SyncOperation>,
    pub in_progress_operations: HashMap<uuid::Uuid, SyncOperation>,
    pub completed_operations: VecDeque<CompletedSyncOperation>,
    pub failed_operations: VecDeque<FailedSyncOperation>,
    pub retry_queue: VecDeque<RetrySyncOperation>,
}

#[derive(Debug, Clone)]
pub struct SyncOperation {
    pub operation_id: uuid::Uuid,
    pub category: SyncCategory,
    pub operation_type: SyncOperationType,
    pub data_hash: String,
    pub created_at: DateTime<Utc>,
    pub priority: SyncPriority,
    pub retry_count: u32,
    pub estimated_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncOperationType {
    Upload,
    Download,
    Delete,
    Merge,
    ConflictResolution,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SyncPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl CloudSyncEngine {
    pub fn new() -> Self {
        Self {
            sync_state: SyncEngineState::Idle,
            sync_queue: Arc::new(RwLock::new(SyncQueue::new())),
            conflict_resolver: ConflictResolver::new(),
            encryption_manager: EncryptionManager::new(),
            network_manager: NetworkManager::new(),
            data_store: CloudDataStore::new(),
        }
    }
    
    pub async fn start_sync_operation(&mut self, categories: Vec<SyncCategory>) -> Result<(), SyncError> {
        if self.sync_state == SyncEngineState::Syncing {
            return Err(SyncError::AlreadySyncing);
        }
        
        self.sync_state = SyncEngineState::Syncing;
        
        for category in categories {
            let operation = SyncOperation {
                operation_id: uuid::Uuid::new_v4(),
                category,
                operation_type: SyncOperationType::Upload,
                data_hash: self.calculate_category_hash(&category).await?,
                created_at: Utc::now(),
                priority: SyncPriority::Normal,
                retry_count: 0,
                estimated_size_bytes: self.estimate_category_size(&category).await?,
            };
            
            let mut queue = self.sync_queue.write().await;
            queue.pending_operations.push_back(operation);
        }
        
        Ok(())
    }
    
    async fn calculate_category_hash(&self, category: &SyncCategory) -> Result<String, SyncError> {
        // Calculate hash of category data for change detection
        let data = self.data_store.get_category_data(category).await?;
        let hash = sha256::digest(data.as_bytes());
        Ok(hash)
    }
    
    async fn estimate_category_size(&self, category: &SyncCategory) -> Result<u64, SyncError> {
        // Estimate data size for bandwidth planning
        self.data_store.estimate_category_size(category).await
    }
}
```

### 2. Conflict Resolution System
```rust
// Conflict resolution based on examples/ecs/system_sets.rs:55-80
#[derive(Clone, Debug)]
pub struct ConflictResolver {
    pub resolution_strategies: HashMap<SyncCategory, ConflictResolutionStrategy>,
    pub merge_algorithms: MergeAlgorithmRegistry,
    pub conflict_history: Vec<ConflictResolution>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConflictResolutionStrategy {
    ClientWins,
    ServerWins,
    MostRecent,
    MergeFields,
    UserChoice,
    CustomAlgorithm(String),
}

#[derive(Debug, Clone)]
pub struct ConflictResolution {
    pub conflict_id: uuid::Uuid,
    pub category: SyncCategory,
    pub client_version: DataVersion,
    pub server_version: DataVersion,
    pub resolution_strategy: ConflictResolutionStrategy,
    pub resolved_at: DateTime<Utc>,
    pub merged_data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DataVersion {
    pub version_id: String,
    pub timestamp: DateTime<Utc>,
    pub device_id: String,
    pub data_hash: String,
    pub size_bytes: u64,
}

impl ConflictResolver {
    pub fn new() -> Self {
        let mut resolution_strategies = HashMap::new();
        
        // Default strategies for each category
        resolution_strategies.insert(SyncCategory::SearchHistory, ConflictResolutionStrategy::MergeFields);
        resolution_strategies.insert(SyncCategory::Aliases, ConflictResolutionStrategy::MostRecent);
        resolution_strategies.insert(SyncCategory::Hotkeys, ConflictResolutionStrategy::UserChoice);
        resolution_strategies.insert(SyncCategory::ExtensionsAndSettings, ConflictResolutionStrategy::MergeFields);
        resolution_strategies.insert(SyncCategory::Quicklinks, ConflictResolutionStrategy::MergeFields);
        resolution_strategies.insert(SyncCategory::Snippets, ConflictResolutionStrategy::MergeFields);
        resolution_strategies.insert(SyncCategory::RaycastNotes, ConflictResolutionStrategy::MostRecent);
        resolution_strategies.insert(SyncCategory::Themes, ConflictResolutionStrategy::ClientWins);
        resolution_strategies.insert(SyncCategory::AiChatsPresetsCommands, ConflictResolutionStrategy::MergeFields);
        resolution_strategies.insert(SyncCategory::CustomWindowManagement, ConflictResolutionStrategy::MostRecent);
        
        Self {
            resolution_strategies,
            merge_algorithms: MergeAlgorithmRegistry::new(),
            conflict_history: Vec::new(),
        }
    }
    
    pub async fn resolve_conflict(
        &mut self,
        category: &SyncCategory,
        client_data: &str,
        server_data: &str,
        client_version: DataVersion,
        server_version: DataVersion,
    ) -> Result<String, SyncError> {
        let strategy = self.resolution_strategies
            .get(category)
            .unwrap_or(&ConflictResolutionStrategy::MostRecent);
            
        let resolved_data = match strategy {
            ConflictResolutionStrategy::ClientWins => {
                client_data.to_string()
            }
            ConflictResolutionStrategy::ServerWins => {
                server_data.to_string()
            }
            ConflictResolutionStrategy::MostRecent => {
                if client_version.timestamp > server_version.timestamp {
                    client_data.to_string()
                } else {
                    server_data.to_string()
                }
            }
            ConflictResolutionStrategy::MergeFields => {
                self.merge_algorithms.merge_json_fields(client_data, server_data)?
            }
            ConflictResolutionStrategy::UserChoice => {
                // This will trigger a UI prompt for user selection
                return Err(SyncError::RequiresUserInput);
            }
            ConflictResolutionStrategy::CustomAlgorithm(algorithm_name) => {
                self.merge_algorithms.apply_custom_algorithm(algorithm_name, client_data, server_data)?
            }
        };
        
        // Record conflict resolution
        self.conflict_history.push(ConflictResolution {
            conflict_id: uuid::Uuid::new_v4(),
            category: category.clone(),
            client_version,
            server_version,
            resolution_strategy: strategy.clone(),
            resolved_at: Utc::now(),
            merged_data: Some(resolved_data.clone()),
        });
        
        Ok(resolved_data)
    }
}
```

### 3. Encryption and Security System
```rust
// Encryption system based on examples/async_tasks/async_compute.rs:85-110
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

#[derive(Clone, Debug)]
pub struct EncryptionManager {
    pub encryption_keys: HashMap<String, Key<Aes256Gcm>>,
    pub device_key_pair: KeyPair,
    pub master_key_encrypted: Vec<u8>,
    pub key_derivation_params: KeyDerivationParams,
}

#[derive(Clone, Debug)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub key_id: String,
}

#[derive(Clone, Debug)]
pub struct KeyDerivationParams {
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub key_length: usize,
}

impl EncryptionManager {
    pub fn new() -> Self {
        Self {
            encryption_keys: HashMap::new(),
            device_key_pair: KeyPair::generate(),
            master_key_encrypted: Vec::new(),
            key_derivation_params: KeyDerivationParams {
                salt: generate_random_salt(),
                iterations: 100_000,
                key_length: 32,
            },
        }
    }
    
    pub async fn encrypt_category_data(
        &self,
        category: &SyncCategory,
        data: &str,
    ) -> Result<EncryptedData, SyncError> {
        let key = self.get_category_encryption_key(category)?;
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(generate_random_nonce().as_slice());
        
        let encrypted_bytes = cipher.encrypt(nonce, data.as_bytes())
            .map_err(|_| SyncError::EncryptionFailed)?;
            
        Ok(EncryptedData {
            data: encrypted_bytes,
            nonce: nonce.to_vec(),
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_id: self.get_key_id(category),
        })
    }
    
    pub async fn decrypt_category_data(
        &self,
        category: &SyncCategory,
        encrypted_data: &EncryptedData,
    ) -> Result<String, SyncError> {
        let key = self.get_category_encryption_key(category)?;
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&encrypted_data.nonce);
        
        let decrypted_bytes = cipher.decrypt(nonce, encrypted_data.data.as_slice())
            .map_err(|_| SyncError::DecryptionFailed)?;
            
        let decrypted_string = String::from_utf8(decrypted_bytes)
            .map_err(|_| SyncError::InvalidDataFormat)?;
            
        Ok(decrypted_string)
    }
    
    fn get_category_encryption_key(&self, category: &SyncCategory) -> Result<&Key<Aes256Gcm>, SyncError> {
        let key_id = format!("category_{:?}", category);
        self.encryption_keys.get(&key_id)
            .ok_or(SyncError::EncryptionKeyNotFound)
    }
    
    fn get_key_id(&self, category: &SyncCategory) -> String {
        format!("category_{:?}", category)
    }
}

#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub algorithm: EncryptionAlgorithm,
    pub key_id: String,
}

#[derive(Debug, Clone)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}
```

### 4. Network Manager and Offline Queue
```rust
// Network manager based on examples/async_tasks/async_compute.rs:135-160
#[derive(Clone, Debug)]
pub struct NetworkManager {
    pub connection_state: NetworkConnectionState,
    pub endpoint_config: CloudEndpointConfig,
    pub retry_policy: RetryPolicy,
    pub bandwidth_limiter: BandwidthLimiter,
    pub connection_pool: ConnectionPool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkConnectionState {
    Connected,
    Disconnected,
    Connecting,
    Limited, // Limited connectivity
    Error(NetworkError),
}

#[derive(Clone, Debug)]
pub struct CloudEndpointConfig {
    pub base_url: String,
    pub api_version: String,
    pub auth_token: Option<String>,
    pub device_id: String,
    pub timeout_seconds: u64,
    pub max_retry_attempts: u32,
}

#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff_factor: f32,
    pub max_retry_attempts: u32,
    pub retry_on_errors: Vec<NetworkError>,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            connection_state: NetworkConnectionState::Disconnected,
            endpoint_config: CloudEndpointConfig {
                base_url: "https://api.launcher-sync.com".to_string(),
                api_version: "v1".to_string(),
                auth_token: None,
                device_id: uuid::Uuid::new_v4().to_string(),
                timeout_seconds: 30,
                max_retry_attempts: 3,
            },
            retry_policy: RetryPolicy {
                base_delay_ms: 1000,
                max_delay_ms: 30000,
                exponential_backoff_factor: 2.0,
                max_retry_attempts: 5,
                retry_on_errors: vec![
                    NetworkError::Timeout,
                    NetworkError::ConnectionLost,
                    NetworkError::ServerError(500),
                    NetworkError::ServerError(502),
                    NetworkError::ServerError(503),
                ],
            },
            bandwidth_limiter: BandwidthLimiter::new(1_000_000), // 1MB/s default
            connection_pool: ConnectionPool::new(),
        }
    }
    
    pub async fn upload_category_data(
        &mut self,
        category: &SyncCategory,
        encrypted_data: &EncryptedData,
    ) -> Result<UploadResponse, SyncError> {
        if self.connection_state != NetworkConnectionState::Connected {
            return Err(SyncError::NetworkDisconnected);
        }
        
        let url = format!(
            "{}/api/{}/sync/categories/{:?}",
            self.endpoint_config.base_url,
            self.endpoint_config.api_version,
            category
        );
        
        let request_body = UploadRequest {
            device_id: self.endpoint_config.device_id.clone(),
            category: category.clone(),
            encrypted_data: encrypted_data.clone(),
            timestamp: Utc::now(),
        };
        
        let response = self.execute_with_retry(async move {
            self.connection_pool
                .post(&url)
                .json(&request_body)
                .timeout(Duration::from_secs(self.endpoint_config.timeout_seconds))
                .send()
                .await
        }).await?;
        
        if response.status().is_success() {
            let upload_response: UploadResponse = response.json().await
                .map_err(|_| SyncError::InvalidServerResponse)?;
            Ok(upload_response)
        } else {
            Err(SyncError::ServerError(response.status().as_u16()))
        }
    }
    
    pub async fn download_category_data(
        &mut self,
        category: &SyncCategory,
    ) -> Result<DownloadResponse, SyncError> {
        if self.connection_state != NetworkConnectionState::Connected {
            return Err(SyncError::NetworkDisconnected);
        }
        
        let url = format!(
            "{}/api/{}/sync/categories/{:?}?device_id={}",
            self.endpoint_config.base_url,
            self.endpoint_config.api_version,
            category,
            self.endpoint_config.device_id
        );
        
        let response = self.execute_with_retry(async move {
            self.connection_pool
                .get(&url)
                .timeout(Duration::from_secs(self.endpoint_config.timeout_seconds))
                .send()
                .await
        }).await?;
        
        if response.status().is_success() {
            let download_response: DownloadResponse = response.json().await
                .map_err(|_| SyncError::InvalidServerResponse)?;
            Ok(download_response)
        } else if response.status().as_u16() == 404 {
            Ok(DownloadResponse::NotFound)
        } else {
            Err(SyncError::ServerError(response.status().as_u16()))
        }
    }
    
    async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> Result<T, SyncError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, reqwest::Error>>,
    {
        let mut attempt = 0;
        let mut delay = self.retry_policy.base_delay_ms;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    attempt += 1;
                    
                    if attempt >= self.retry_policy.max_retry_attempts {
                        return Err(SyncError::NetworkError(NetworkError::MaxRetriesExceeded));
                    }
                    
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    delay = std::cmp::min(
                        (delay as f32 * self.retry_policy.exponential_backoff_factor) as u64,
                        self.retry_policy.max_delay_ms,
                    );
                }
            }
        }
    }
}
```

### 5. Bevy Integration Systems
```rust
// Bevy integration system based on examples/ecs/system_sets.rs:105-130
fn sync_engine_update_system(
    mut sync_engine: ResMut<CloudSyncEngine>,
    mut sync_events: EventReader<InitiateCategorySyncEvent>,
    mut sync_status_events: EventWriter<SyncStatusUpdateEvent>,
    mut sync_interface: Query<&mut CloudSyncInterface>,
    time: Res<Time>,
) {
    // Process sync initiation events
    for event in sync_events.read() {
        info!("Initiating sync for categories: {:?}", event.categories);
        
        match sync_engine.start_sync_operation(event.categories.clone()).await {
            Ok(()) => {
                sync_status_events.send(SyncStatusUpdateEvent {
                    status: SyncStatus::InProgress,
                    categories: event.categories.clone(),
                    message: "Sync started".to_string(),
                    timestamp: Utc::now(),
                });
                
                // Update interface state
                for mut interface in sync_interface.iter_mut() {
                    interface.sync_in_progress = true;
                }
            }
            Err(err) => {
                error!("Failed to start sync: {:?}", err);
                sync_status_events.send(SyncStatusUpdateEvent {
                    status: SyncStatus::Error,
                    categories: event.categories.clone(),
                    message: format!("Sync failed: {:?}", err),
                    timestamp: Utc::now(),
                });
            }
        }
    }
    
    // Process sync queue
    if sync_engine.sync_state == SyncEngineState::Syncing {
        // This would typically be handled by a separate async task
        // For demonstration, we'll show the structure
        tokio::spawn(async move {
            sync_engine.process_sync_queue().await
        });
    }
}

async fn process_sync_queue_async(mut sync_engine: CloudSyncEngine) {
    while sync_engine.sync_state == SyncEngineState::Syncing {
        let mut queue = sync_engine.sync_queue.write().await;
        
        if let Some(operation) = queue.pending_operations.pop_front() {
            queue.in_progress_operations.insert(operation.operation_id, operation.clone());
            drop(queue); // Release the lock
            
            let result = sync_engine.execute_sync_operation(operation.clone()).await;
            
            let mut queue = sync_engine.sync_queue.write().await;
            queue.in_progress_operations.remove(&operation.operation_id);
            
            match result {
                Ok(completed_op) => {
                    queue.completed_operations.push_back(completed_op);
                }
                Err(failed_op) => {
                    queue.failed_operations.push_back(failed_op);
                }
            }
        } else {
            // No more operations, sync complete
            sync_engine.sync_state = SyncEngineState::Idle;
            break;
        }
    }
}
```

## Bevy Example References
- **Async tasks**: `examples/async_tasks/async_compute.rs:25-50` - Async sync operation processing
- **System coordination**: `examples/ecs/system_sets.rs:55-80` - Sync engine system organization
- **Event handling**: `examples/ecs/event.rs:105-130` - Sync event processing
- **Resource management**: `examples/ecs/removal_detection.rs:135-160` - Sync state management
- **Time handling**: `examples/time/time.rs:85-110` - Timestamp and timing operations

## Architecture Integration Notes
- **File**: `core/src/cloud_sync/sync_engine.rs:1-1200`
- **Dependencies**: Encryption libraries, HTTP client, async runtime
- **Integration**: UI components, settings persistence, network monitoring
- **Performance**: Async processing, connection pooling, efficient data handling

## Success Criteria
1. **Selective synchronization** working for all enabled categories with proper filtering
2. **Conflict resolution** handling concurrent modifications intelligently across devices
3. **End-to-end encryption** protecting all synchronized data in transit and at rest
4. **Offline support** with reliable queue management and automatic retry mechanisms
5. **Real-time sync** completing within 5 seconds for typical data volumes
6. **Bandwidth optimization** using delta sync and compression to minimize data usage
7. **Error recovery** with comprehensive retry logic and graceful failure handling
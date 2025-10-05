# Task 4: Cloud Storage Providers Integration System

## Overview
Implement comprehensive cloud storage provider integration system supporting multiple platforms with unified API abstraction layer, credential management, and real-time sync monitoring.

## Architecture Reference
**Bevy Example**: `./docs/bevy/examples/async_compute.rs` (lines 45-89) - AsyncComputeTaskPool for cloud API operations
**Bevy Example**: `./docs/bevy/examples/resource_management.rs` (lines 123-167) - Resource lifecycle management for provider connections

## Implementation

### File: `core/src/cloud/providers/mod.rs`
```rust
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Resource, Clone, Debug)]
pub struct CloudProviderRegistry {
    pub providers: HashMap<String, Box<dyn CloudProvider>>,
    pub active_connections: HashMap<String, CloudConnection>,
    pub sync_status: HashMap<String, SyncStatus>,
    pub credentials_store: CredentialsManager,
    pub unified_api: UnifiedCloudAPI,
}

// Reference: ./docs/bevy/examples/trait_objects.rs lines 78-112
pub trait CloudProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn authenticate(&self, credentials: Credentials) -> Result<AuthToken, CloudError>;
    fn upload_file(&self, path: &str, content: Vec<u8>) -> Task<Result<FileMetadata, CloudError>>;
    fn download_file(&self, file_id: &str) -> Task<Result<Vec<u8>, CloudError>>;
    fn list_files(&self, folder_id: Option<String>) -> Task<Result<Vec<FileMetadata>, CloudError>>;
    fn sync_folder(&self, local_path: &str, remote_path: &str) -> Task<Result<SyncResult, CloudError>>;
    fn get_storage_quota(&self) -> Task<Result<StorageQuota, CloudError>>;
}

// Reference: ./docs/bevy/examples/async_compute.rs lines 156-189
pub struct GoogleDriveProvider {
    pub client: GoogleDriveClient,
    pub oauth_config: OAuthConfig,
    pub rate_limiter: RateLimiter,
}

impl CloudProvider for GoogleDriveProvider {
    fn provider_id(&self) -> &str { "google_drive" }
    
    fn upload_file(&self, path: &str, content: Vec<u8>) -> Task<Result<FileMetadata, CloudError>> {
        let client = self.client.clone();
        let path = path.to_string();
        
        AsyncComputeTaskPool::get().spawn(async move {
            let response = client.files()
                .create(CreateRequest::new())
                .param("uploadType", "multipart")
                .add_file_content(content, path.clone())
                .doit()
                .await?;
                
            Ok(FileMetadata {
                id: response.1.id.unwrap_or_default(),
                name: path,
                size: response.1.size.unwrap_or(0),
                modified_time: response.1.modified_time,
                checksum: response.1.md5_checksum,
            })
        })
    }
}

// Reference: ./docs/bevy/examples/resource_management.rs lines 234-267
pub struct DropboxProvider {
    pub client: DropboxClient,
    pub access_token: AccessToken,
    pub refresh_manager: TokenRefreshManager,
}

pub struct OneDriveProvider {
    pub graph_client: GraphServiceClient,
    pub tenant_config: TenantConfiguration,
    pub delegation_handler: DelegationHandler,
}

pub struct ICloudProvider {
    pub session: ICloudSession,
    pub two_factor_handler: TwoFactorHandler,
    pub keychain_integration: KeychainManager,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct CloudConnection {
    pub provider_id: String,
    pub connection_id: String,
    pub status: ConnectionStatus,
    pub last_sync: Option<DateTime<Utc>>,
    pub auth_token: Option<AuthToken>,
    pub rate_limit_remaining: u32,
    pub quota_used: u64,
    pub quota_total: u64,
}

#[derive(Resource, Clone, Debug)]
pub struct UnifiedCloudAPI {
    pub provider_registry: HashMap<String, Box<dyn CloudProvider>>,
    pub connection_pool: ConnectionPool,
    pub sync_scheduler: SyncScheduler,
    pub conflict_resolver: ConflictResolver,
}

impl UnifiedCloudAPI {
    // Reference: ./docs/bevy/examples/system_chaining.rs lines 67-89
    pub fn sync_across_providers(&self, sync_request: MultiProviderSyncRequest) -> Task<Result<SyncSummary, CloudError>> {
        let providers = self.provider_registry.clone();
        let resolver = self.conflict_resolver.clone();
        
        AsyncComputeTaskPool::get().spawn(async move {
            let mut sync_results = Vec::new();
            
            for provider_id in sync_request.providers {
                if let Some(provider) = providers.get(&provider_id) {
                    let result = provider.sync_folder(
                        &sync_request.local_path,
                        &sync_request.remote_path
                    ).await;
                    sync_results.push((provider_id, result));
                }
            }
            
            resolver.resolve_conflicts(sync_results).await
        })
    }
    
    pub fn aggregate_storage_quota(&self) -> Task<Result<AggregateQuota, CloudError>> {
        let providers = self.provider_registry.clone();
        
        AsyncComputeTaskPool::get().spawn(async move {
            let mut total_used = 0;
            let mut total_available = 0;
            let mut provider_quotas = HashMap::new();
            
            for (id, provider) in providers {
                if let Ok(quota) = provider.get_storage_quota().await {
                    total_used += quota.used;
                    total_available += quota.total;
                    provider_quotas.insert(id, quota);
                }
            }
            
            Ok(AggregateQuota {
                total_used,
                total_available,
                utilization_percentage: (total_used as f32 / total_available as f32) * 100.0,
                provider_breakdown: provider_quotas,
            })
        })
    }
}
```

### File: `core/src/cloud/credentials.rs`
```rust
// Reference: ./docs/bevy/examples/security/secure_storage.rs lines 45-78
#[derive(Resource, Clone, Debug)]
pub struct CredentialsManager {
    pub keyring: SecureKeyring,
    pub encryption_key: EncryptionKey,
    pub oauth_flow_manager: OAuthFlowManager,
    pub token_refresh_scheduler: TokenRefreshScheduler,
}

impl CredentialsManager {
    pub fn store_credentials(&mut self, provider_id: String, credentials: Credentials) -> Result<(), CredentialsError> {
        let encrypted_creds = self.encryption_key.encrypt(&credentials)?;
        self.keyring.store(&format!("cloud_provider_{}", provider_id), encrypted_creds)?;
        
        // Schedule token refresh if OAuth
        if let Credentials::OAuth { refresh_token, expires_in, .. } = credentials {
            self.token_refresh_scheduler.schedule_refresh(provider_id, expires_in)?;
        }
        
        Ok(())
    }
    
    pub fn retrieve_credentials(&self, provider_id: &str) -> Result<Credentials, CredentialsError> {
        let encrypted_creds = self.keyring.retrieve(&format!("cloud_provider_{}", provider_id))?;
        let credentials = self.encryption_key.decrypt(&encrypted_creds)?;
        Ok(credentials)
    }
}
```

### File: `ui/src/ui/cloud/provider_setup.rs`
```rust
// Reference: ./docs/bevy/examples/ui/scroll_list.rs lines 123-156
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CloudProviderSetupProps {
    pub available_providers: Vec<ProviderInfo>,
    pub connected_providers: Vec<CloudConnection>,
    pub on_connect: EventHandler<String>,
    pub on_disconnect: EventHandler<String>,
}

pub fn CloudProviderSetup(props: CloudProviderSetupProps) -> Element {
    let setup_modal_open = use_signal(|| false);
    let selected_provider = use_signal(|| Option::<String>::None);
    
    rsx! {
        div {
            class: "cloud-provider-setup",
            style: "
                display: flex;
                flex_direction: column;
                height: 100%;
                padding: 20px;
                gap: 16px;
            ",
            
            // Header with aggregate stats
            div {
                class: "provider-overview",
                style: "
                    background: rgba(20, 20, 20, 0.8);
                    border-radius: 12px;
                    padding: 16px;
                    border: 1px solid rgba(60, 60, 60, 0.3);
                ",
                
                h2 { 
                    style: "margin: 0 0 12px 0; color: #ffffff; font-size: 18px;",
                    "Cloud Storage Providers"
                }
                
                div {
                    class: "quota-summary",
                    style: "display: flex; gap: 20px; flex-wrap: wrap;",
                    
                    for quota in props.aggregate_quota {
                        div {
                            class: "quota-item",
                            style: "
                                background: rgba(40, 40, 40, 0.6);
                                border-radius: 8px;
                                padding: 12px;
                                min-width: 140px;
                            ",
                            
                            div {
                                style: "color: #888; font-size: 12px; margin-bottom: 4px;",
                                "{quota.provider_name}"
                            }
                            div {
                                style: "color: #ffffff; font-size: 14px; font-weight: 500;",
                                "{format_bytes(quota.used)} / {format_bytes(quota.total)}"
                            }
                            div {
                                class: "quota-bar",
                                style: "
                                    height: 4px;
                                    background: rgba(60, 60, 60, 0.5);
                                    border-radius: 2px;
                                    margin-top: 6px;
                                    overflow: hidden;
                                ",
                                div {
                                    style: "
                                        height: 100%;
                                        background: linear-gradient(90deg, #4CAF50 0%, #FF9800 70%, #F44336 100%);
                                        width: {quota.utilization_percentage}%;
                                        transition: width 0.3s ease;
                                    "
                                }
                            }
                        }
                    }
                }
            }
            
            // Available providers grid
            div {
                class: "available-providers",
                style: "
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
                    gap: 16px;
                    margin-top: 20px;
                ",
                
                for provider in props.available_providers {
                    div {
                        class: "provider-card",
                        style: "
                            background: rgba(30, 30, 30, 0.8);
                            border-radius: 10px;
                            padding: 16px;
                            border: 1px solid rgba(70, 70, 70, 0.3);
                            cursor: pointer;
                            transition: all 0.2s ease;
                        ",
                        onclick: move |_| {
                            selected_provider.set(Some(provider.id.clone()));
                            setup_modal_open.set(true);
                        },
                        
                        div {
                            class: "provider-header",
                            style: "display: flex; align-items: center; gap: 12px; margin-bottom: 12px;",
                            
                            img {
                                src: "{provider.icon_url}",
                                alt: "{provider.name}",
                                style: "width: 32px; height: 32px; border-radius: 6px;"
                            }
                            
                            div {
                                h3 {
                                    style: "margin: 0; color: #ffffff; font-size: 16px;",
                                    "{provider.name}"
                                }
                                div {
                                    style: "color: #888; font-size: 12px;",
                                    "{provider.description}"
                                }
                            }
                        }
                        
                        div {
                            class: "provider-features",
                            style: "margin-bottom: 12px;",
                            
                            for feature in provider.features {
                                span {
                                    class: "feature-tag",
                                    style: "
                                        display: inline-block;
                                        background: rgba(70, 130, 180, 0.2);
                                        color: #87CEEB;
                                        padding: 2px 8px;
                                        border-radius: 4px;
                                        font-size: 11px;
                                        margin-right: 6px;
                                        margin-bottom: 4px;
                                    ",
                                    "{feature}"
                                }
                            }
                        }
                        
                        if let Some(connection) = props.connected_providers.iter().find(|c| c.provider_id == provider.id) {
                            div {
                                class: "connection-status connected",
                                style: "
                                    display: flex;
                                    align-items: center;
                                    justify-content: space-between;
                                    padding: 8px 12px;
                                    background: rgba(76, 175, 80, 0.1);
                                    border: 1px solid rgba(76, 175, 80, 0.3);
                                    border-radius: 6px;
                                ",
                                
                                div {
                                    style: "color: #4CAF50; font-size: 12px; display: flex; align-items: center; gap: 6px;",
                                    "● Connected"
                                    span {
                                        style: "color: #888;",
                                        "Last sync: {format_time_ago(connection.last_sync)}"
                                    }
                                }
                                
                                button {
                                    style: "
                                        background: rgba(244, 67, 54, 0.8);
                                        color: white;
                                        border: none;
                                        padding: 4px 8px;
                                        border-radius: 4px;
                                        font-size: 11px;
                                        cursor: pointer;
                                    ",
                                    onclick: move |_| props.on_disconnect.call(provider.id.clone()),
                                    "Disconnect"
                                }
                            }
                        } else {
                            div {
                                class: "connection-status disconnected",
                                style: "
                                    text-align: center;
                                    padding: 8px 12px;
                                    background: rgba(60, 60, 60, 0.3);
                                    border: 1px solid rgba(100, 100, 100, 0.3);
                                    border-radius: 6px;
                                    color: #888;
                                    font-size: 12px;
                                ",
                                "Click to connect"
                            }
                        }
                    }
                }
            }
        }
        
        // OAuth setup modal
        if setup_modal_open() {
            OAuthSetupModal {
                provider_id: selected_provider().unwrap_or_default(),
                is_open: setup_modal_open(),
                on_close: move |_| setup_modal_open.set(false),
                on_success: move |provider_id| {
                    props.on_connect.call(provider_id);
                    setup_modal_open.set(false);
                }
            }
        }
    }
}
```

## Integration Points

### Event System Integration
- **Cloud sync events**: `CloudSyncStarted`, `CloudSyncCompleted`, `CloudSyncFailed`
- **Provider events**: `ProviderConnected`, `ProviderDisconnected`, `QuotaExceeded`
- **Conflict events**: `SyncConflictDetected`, `ConflictResolved`

### Performance Considerations
- Concurrent provider operations using `AsyncComputeTaskPool`
- Connection pooling for API efficiency
- Rate limiting per provider to avoid throttling
- Credential caching with secure storage

### Security Architecture
- OAuth 2.0 flow implementation with PKCE
- Encrypted credential storage using system keyring
- Token refresh automation
- Scope-based permission management

**Bevy Integration**: Reference `./docs/bevy/examples/async_compute.rs` lines 234-278 for async task management patterns
**UI Architecture**: Reference `./docs/bevy/examples/ui/dynamic_components.rs` lines 89-134 for provider card component patterns
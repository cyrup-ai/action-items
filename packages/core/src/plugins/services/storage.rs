use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use serde_json::Value;
use tokio::fs;
use tokio::sync::RwLock;

/// Service for managing storage operations in plugins
#[derive(Resource, Clone)]
pub struct StorageService {
    inner: Arc<RwLock<StorageServiceInner>>,
    base_path: PathBuf,
    plugin_id: String,
}

struct StorageServiceInner {
    data: HashMap<String, Value>,
    storage_file: PathBuf,
}

impl StorageService {
    pub async fn new(
        base_path: PathBuf,
        plugin_id: String,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let plugin_path = base_path.join(&plugin_id);
        std::fs::create_dir_all(&plugin_path)?;

        let storage_file = plugin_path.join("storage.json");

        // Load existing data from file if it exists
        let data = if storage_file.exists() {
            let content = fs::read_to_string(&storage_file).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            inner: Arc::new(RwLock::new(StorageServiceInner {
                data,
                storage_file: storage_file.clone(),
            })),
            base_path: plugin_path,
            plugin_id,
        })
    }

    async fn persist_data(&self) -> Result<(), String> {
        let inner = self.inner.read().await;
        let json_content = serde_json::to_string_pretty(&inner.data)
            .map_err(|e| format!("Failed to serialize data: {}", e))?;

        fs::write(&inner.storage_file, json_content)
            .await
            .map_err(|e| format!("Failed to write storage file: {}", e))?;

        Ok(())
    }

    /// Store a value with the given key
    pub async fn set(&self, key: &str, value: Value) -> Result<(), String> {
        {
            let mut inner = self.inner.write().await;
            inner.data.insert(key.to_string(), value);
        }

        self.persist_data().await?;
        log::debug!("Storage set for plugin {}: {} = value", self.plugin_id, key);
        Ok(())
    }

    /// Retrieve a value by key
    pub async fn get(&self, key: &str) -> Result<Option<Value>, String> {
        let inner = self.inner.read().await;
        Ok(inner.data.get(key).cloned())
    }

    /// Delete a value by key
    pub async fn delete(&self, key: &str) -> Result<bool, String> {
        let existed = {
            let mut inner = self.inner.write().await;
            inner.data.remove(key).is_some()
        };

        if existed {
            self.persist_data().await?;
        }

        log::debug!(
            "Storage delete for plugin {}: {} (existed: {})",
            self.plugin_id,
            key,
            existed
        );
        Ok(existed)
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool, String> {
        let inner = self.inner.read().await;
        Ok(inner.data.contains_key(key))
    }

    /// Get all keys
    pub async fn keys(&self) -> Result<Vec<String>, String> {
        let inner = self.inner.read().await;
        Ok(inner.data.keys().cloned().collect())
    }

    /// Clear all data for this plugin
    pub async fn clear(&self) -> Result<(), String> {
        let count = {
            let mut inner = self.inner.write().await;
            let count = inner.data.len();
            inner.data.clear();
            count
        };

        self.persist_data().await?;
        log::debug!(
            "Storage cleared for plugin {}: {} items removed",
            self.plugin_id,
            count
        );
        Ok(())
    }

    /// Get the storage path for this plugin
    pub fn storage_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Get the plugin ID
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }
}

/// Initialize global storage service using shared Tokio runtime
///
/// This system runs at startup and creates the StorageService resource
/// using the shared TokioTasksRuntime, avoiding the need to create
/// a new runtime just for storage initialization.
pub fn initialize_storage_system(
    tokio_runtime: Res<TokioTasksRuntime>,
) {
    log::info!("Initializing global storage service using shared Tokio runtime");

    // Use temp directory for global storage
    let base_path = std::env::temp_dir().join("action_items_global_storage");
    let plugin_id = "global".to_string();

    // Spawn background task in shared Tokio runtime
    tokio_runtime.spawn_background_task(|mut ctx| async move {
        log::info!("Creating global storage service at path: {:?}", base_path);

        match StorageService::new(base_path.clone(), plugin_id.clone()).await {
            Ok(service) => {
                log::info!("Successfully initialized global storage service");
                
                // Insert StorageService as a Bevy resource on the main thread
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.insert_resource(service);
                    log::info!("StorageService resource registered in Bevy world");
                }).await;
            }
            Err(e) => {
                log::error!("Failed to initialize global storage service: {:?}", e);
                // Note: App continues without storage service
                // Individual plugins will need to handle missing storage gracefully
            }
        }
    });
}

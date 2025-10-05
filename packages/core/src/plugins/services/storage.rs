use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use bevy::prelude::*;
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

    /// Get global storage service instance with thread-safe singleton pattern
    pub fn global_instance() -> Result<Arc<RwLock<Self>>, Box<dyn std::error::Error + Send + Sync>>
    {
        use std::sync::OnceLock;

        use tokio::runtime::Handle;

        static INSTANCE: OnceLock<Arc<RwLock<StorageService>>> = OnceLock::new();

        match INSTANCE.get() {
            Some(instance) => Ok(instance.clone()),
            None => {
                let result = {
                    let base_path = std::env::temp_dir().join("action_items_global_storage");

                    // Try to use current tokio runtime, fallback to creating one
                    let instance_result = if let Ok(handle) = Handle::try_current() {
                        // Use existing runtime
                        handle.block_on(async { Self::new(base_path, "global".to_string()).await })
                    } else {
                        // Create new runtime for initialization
                        match tokio::runtime::Runtime::new() {
                            Ok(rt) => rt.block_on(async {
                                Self::new(base_path, "global".to_string()).await
                            }),
                            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
                        }
                    };

                    match instance_result {
                        Ok(service) => Arc::new(RwLock::new(service)),
                        Err(e) => return Err(e),
                    }
                };

                INSTANCE
                    .set(result.clone())
                    .map_err(|_| "Failed to set global instance")?;
                Ok(result)
            },
        }
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

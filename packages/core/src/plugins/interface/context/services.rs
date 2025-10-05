//! Service implementation structs for plugin context

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use moka::sync::Cache as MokaCache;

#[derive(Clone)]
pub struct ClipboardAccess;

impl Default for ClipboardAccess {
    fn default() -> Self {
        Self
    }
}

impl ClipboardAccess {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct NotificationService {
    #[allow(dead_code)]
    app_name: String,
}

impl NotificationService {
    pub fn new(app_name: String) -> Self {
        Self { app_name }
    }
}

#[derive(Clone)]
pub struct StorageService {
    #[allow(dead_code)]
    base_path: PathBuf,
}

impl StorageService {
    pub fn new(base_path: PathBuf, plugin_id: String) -> Result<Self, std::io::Error> {
        let plugin_path = base_path.join(&plugin_id);
        std::fs::create_dir_all(&plugin_path)?;
        Ok(Self {
            base_path: plugin_path,
        })
    }
}

#[derive(Clone)]
pub struct HttpClient {
    #[allow(dead_code)]
    timeout: std::time::Duration,
    #[allow(dead_code)]
    max_retries: u32,
}

impl HttpClient {
    pub fn new(timeout: std::time::Duration, max_retries: u32) -> Self {
        Self {
            timeout,
            max_retries,
        }
    }
}

#[derive(Clone)]
pub struct CacheService {
    cache: Arc<MokaCache<String, String>>,
}

impl CacheService {
    pub fn new(max_capacity: u64) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(5 * 60))
            .time_to_idle(Duration::from_secs(60))
            .build();
        Self {
            cache: Arc::new(cache),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key)
    }

    pub fn set(&self, key: String, value: String) {
        self.cache.insert(key, value);
    }

    pub fn remove(&self, key: &str) {
        self.cache.invalidate(key);
    }
}

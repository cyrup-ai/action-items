// Clipboard now handled by ECS service
pub mod database;
pub mod notification;
pub mod storage;

// Legacy resources that were in services.rs
use std::path::PathBuf;
use std::time::Duration;

// ClipboardService now handled by ECS service
// pub use database::{DatabaseConfig, DatabaseError, DatabaseService};  // Disabled due to
// missing dependency
pub use notification::NotificationService;
pub use storage::StorageService;

#[derive(bevy::prelude::Resource)]
/// Resource for managing plugin storage directories
pub struct StorageDirectory {
    base_path: PathBuf,
}

impl StorageDirectory {
    pub fn new(app_directories: &crate::config::AppDirectories) -> Result<Self, std::io::Error> {
        let base_path = app_directories.plugin_cache();
        std::fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    /// Create a disabled storage directory for fallback when regular initialization fails
    pub fn disabled(app_directories: &crate::config::AppDirectories) -> Self {
        Self {
            base_path: app_directories.plugin_cache().join("disabled"),
        }
    }

    pub fn path_for_plugin(&self, plugin_id: &str) -> PathBuf {
        let path = self.base_path.join(sanitize_filename(plugin_id));
        std::fs::create_dir_all(&path).ok();
        path
    }
}

/// Resource for plugin cache using moka
#[derive(bevy::prelude::Resource)]
pub struct PluginCache {
    cache: moka::sync::Cache<String, String>,
}

impl Default for PluginCache {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl PluginCache {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            cache: moka::sync::Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(Duration::from_secs(300)) // 5 minute default TTL
                .build(),
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

    pub fn clear(&self) {
        self.cache.invalidate_all();
    }
}

// Helper functions
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// Components to track plugin and request IDs (used by other modules)
#[derive(bevy::prelude::Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PluginId(pub String);

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PluginId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for PluginId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl From<PluginId> for String {
    fn from(val: PluginId) -> Self {
        val.0
    }
}

impl AsRef<str> for PluginId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(bevy::prelude::Component)]
pub struct RequestId(pub String);

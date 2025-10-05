use std::path::{Path, PathBuf};
use std::time::SystemTime;

use action_items_common::plugin_interface::Icon;
use serde::{Deserialize, Serialize};

use crate::discovery::core::types::MetadataProvider;
use crate::plugins::interface::{ActionItem as InterfaceActionItem, PluginManifest};

/// Lightweight plugin metadata for discovery and indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub manifest: PluginManifest,
    pub is_loaded: bool,
    pub last_accessed: Option<SystemTime>,
    pub load_count: u32,
}

impl MetadataProvider for PluginMetadata {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.manifest.description
    }

    fn get_version(&self) -> &str {
        &self.manifest.version
    }

    fn get_path(&self) -> Option<&Path> {
        Some(&self.path)
    }
}

/// Re-export the search result from plugin_interface for backwards compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionItem {
    pub title: String,
    pub description: String,
    pub action: String,
    pub icon: Option<String>,
    pub score: f32,
}

impl From<InterfaceActionItem> for ActionItem {
    fn from(result: InterfaceActionItem) -> Self {
        Self {
            title: result.title,
            description: result.subtitle.unwrap_or_default(),
            action: result.id,
            icon: result.icon.map(|icon| match icon {
                Icon::Emoji(e) => format!("emoji:{e}"),
                Icon::BuiltIn(name) => format!("builtin:{name}"),
                Icon::File(path_buf) => format!("file:{}", path_buf.display()),
                Icon::Url(url_str) => format!("url:{url_str}"),
                Icon::Base64(data) => format!("base64:{data}"),
            }),
            score: result.score,
        }
    }
}

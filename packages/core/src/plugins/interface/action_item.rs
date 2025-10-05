use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use super::commands::ActionType;

/// Enhanced search result with Raycast-like features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<Icon>,
    pub actions: Vec<ItemAction>,
    pub item_badges: Vec<ItemBadge>,
    pub metadata: Option<serde_json::Value>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Icon {
    BuiltIn(String),
    File(PathBuf),
    Url(String),
    Base64(String),
    Emoji(String),
}

impl Icon {
    pub fn as_str(&self) -> &str {
        match self {
            Icon::BuiltIn(s) => s,
            Icon::File(path) => path.to_str().unwrap_or(""),
            Icon::Url(s) => s,
            Icon::Base64(s) => s,
            Icon::Emoji(s) => s,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAction {
    pub id: String,
    pub title: String,
    pub icon: Option<Icon>,
    pub shortcut: Option<Shortcut>,
    pub action_type: ActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub modifiers: Vec<String>,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemBadge {
    pub text: Option<String>,
    pub icon: Option<Icon>,
    pub tooltip: Option<String>,
}

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub icon: Option<Icon>,
    pub actions: Vec<ItemAction>,
    pub item_badges: Vec<ItemBadge>,
    pub tags: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub score: f32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Icon {
    BuiltIn(String),
    File(PathBuf),
    Url(String),
    Base64(String),
    Emoji(String),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    OpenUrl(String),
    OpenFile(PathBuf),
    RunCommand(String),
    CopyToClipboard(String),
    ShowHud,
    CloseWindow,
    RefreshCommand,
    Custom(String),
}

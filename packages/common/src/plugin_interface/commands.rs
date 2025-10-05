use serde::{Deserialize, Serialize};

use super::action_item::ActionType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: String,
    pub icon: Option<String>,
    pub mode: CommandMode,
    pub keywords: Vec<String>,
    pub arguments: Vec<ArgumentDefinition>,
    pub hotkey: Option<String>,
    pub interval: Option<u64>, // For background commands
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandMode {
    NoView, // Runs in background
    List,   // Shows list of results
    Detail, // Shows detailed view
    Form,   // Shows input form
    View,   // Shows view-only interface
    Custom, // Custom UI
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentDefinition {
    pub name: String,
    pub placeholder: String,
    pub arg_type: ArgumentType,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgumentType {
    Text,
    Number,
    Boolean,
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub action_type: ActionType,
}

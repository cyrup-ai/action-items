use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::capabilities::{PluginCapabilities, PluginPermissions};
use super::commands::{ActionDefinition, CommandDefinition};
use super::config::{ConfigurationField, PreferenceField};

/// Comprehensive plugin metadata matching Raycast's capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    // Basic metadata
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,

    // UI and presentation
    pub icon: Option<String>,
    pub categories: Vec<PluginCategory>,
    pub keywords: Vec<String>,

    // Capabilities and permissions
    pub capabilities: PluginCapabilities,
    pub permissions: PluginPermissions,

    // Configuration
    pub configuration: Vec<ConfigurationField>,
    pub preferences: Vec<PreferenceField>,

    // Commands and actions
    pub commands: Vec<CommandDefinition>,
    pub actions: Vec<ActionDefinition>,

    // Dependencies
    pub dependencies: HashMap<String, String>,
    pub environment: HashMap<String, String>,

    // Update and lifecycle
    pub min_launcher_version: String,
    pub max_launcher_version: Option<String>,
    pub update_url: Option<String>,
    pub changelog_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCategory {
    Productivity,
    Developer,
    System,
    Web,
    Media,
    Communication,
    Finance,
    Education,
    Entertainment,
    Utilities,
    Custom(String),
}

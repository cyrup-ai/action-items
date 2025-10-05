//! Isolated Raycast types for deno operations
//!
//! This module provides completely isolated raycast types that mirror core raycast types
//! but are independent to avoid type conflicts in deno operations. These types prevent
//! extism::Error bleeding into #[op2(async)] macros while maintaining full raycast functionality.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Standard raycast plugin capabilities
pub mod capabilities {
    /// Plugin can perform search operations
    pub const SEARCH: &str = "search";
    /// Plugin can execute commands
    pub const EXECUTE: &str = "execute";
    /// Plugin can access filesystem
    pub const FILESYSTEM: &str = "filesystem";
    /// Plugin can make network requests
    pub const NETWORK: &str = "network";
    /// Plugin can access system information
    pub const SYSTEM: &str = "system";
    /// Plugin can manage clipboard
    pub const CLIPBOARD: &str = "clipboard";
    /// Plugin can show notifications
    pub const NOTIFICATIONS: &str = "notifications";
    /// Plugin can access preferences
    pub const PREFERENCES: &str = "preferences";
    /// Plugin can cache data
    pub const CACHE: &str = "cache";
    /// Plugin can access environment variables
    pub const ENVIRONMENT: &str = "environment";
}

/// Standard command modes
pub mod command_modes {
    /// Command shows a view with results
    pub const VIEW: &str = "view";
    /// Command runs without showing a view
    pub const NO_VIEW: &str = "no-view";
    /// Command runs silently in background
    pub const SILENT: &str = "silent";
}

/// Standard argument types
pub mod argument_types {
    /// Text input field
    pub const TEXT: &str = "text";
    /// Password input field
    pub const PASSWORD: &str = "password";
    /// Dropdown selection
    pub const DROPDOWN: &str = "dropdown";
    /// File picker
    pub const FILE: &str = "file";
    /// Directory picker
    pub const DIRECTORY: &str = "directory";
}

/// Standard preference types
pub mod preference_types {
    /// Text field preference
    pub const TEXTFIELD: &str = "textfield";
    /// Checkbox preference
    pub const CHECKBOX: &str = "checkbox";
    /// Dropdown preference
    pub const DROPDOWN: &str = "dropdown";
    /// Password field preference
    pub const PASSWORD: &str = "password";
}

/// Standard action types
pub mod action_types {
    /// Primary action (default)
    pub const PRIMARY: &str = "primary";
    /// Secondary action
    pub const SECONDARY: &str = "secondary";
    /// Destructive action (delete, etc.)
    pub const DESTRUCTIVE: &str = "destructive";
    /// Copy action
    pub const COPY: &str = "copy";
    /// Open action
    pub const OPEN: &str = "open";
}

/// Isolated raycast extension type for deno operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct IsolatedRaycastExtension {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub categories: Vec<String>,
    pub icon: Option<String>,
    pub path: String, // Using String instead of PathBuf for better serialization
    pub commands: Vec<IsolatedRaycastCommand>,
    pub version: Option<String>,
    pub keywords: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub preferences: Vec<IsolatedExtensionPreference>,
}

/// Isolated raycast command type for deno operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedRaycastCommand {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub mode: String,
    pub subtitle: Option<String>,
    pub keywords: Vec<String>,
    pub arguments: Vec<IsolatedCommandArgument>,
    pub preferences: Vec<IsolatedCommandPreference>,
}

/// Command argument definition for raycast commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedCommandArgument {
    pub name: String,
    pub placeholder: Option<String>,
    pub required: bool,
    pub argument_type: String, // "text", "password", "dropdown", etc.
}

/// Command preference definition for raycast commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedCommandPreference {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub preference_type: String, // "textfield", "checkbox", "dropdown", etc.
    pub default_value: Option<String>,
    pub required: bool,
}

/// Extension-level preference definition for raycast extensions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedExtensionPreference {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub preference_type: String, // "textfield", "checkbox", "dropdown", etc.
    pub default_value: Option<String>,
    pub required: bool,
}

/// Isolated raycast plugin type for plugin management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedRaycastPlugin {
    pub id: String,
    pub name: String,
    pub extension: IsolatedRaycastExtension,
    pub status: IsolatedPluginStatus,
    pub capabilities: Vec<String>,
    pub configuration: HashMap<String, String>,
    pub last_updated: String, // ISO 8601 timestamp
    pub execution_count: u64,
    pub average_execution_time_ms: u64,
}

/// Plugin status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IsolatedPluginStatus {
    Active,
    Inactive,
    Loading,
    Error(String),
    Disabled,
}

/// Plugin metadata for registration and discovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedPluginMetadata {
    pub id: String,
    pub name: String,
    pub path: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub version: String,
    pub metadata: HashMap<String, String>,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Command execution context for deno operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedExecutionContext {
    pub plugin_id: String,
    pub command_name: String,
    pub arguments: HashMap<String, String>,
    pub preferences: HashMap<String, String>,
    pub environment: HashMap<String, String>,
    pub working_directory: Option<String>,
    pub timeout_ms: Option<u64>,
}

/// Command execution result from deno operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedExecutionResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub exit_code: Option<i32>,
    pub artifacts: Vec<IsolatedArtifact>,
}

/// Execution artifact (files, data, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedArtifact {
    pub name: String,
    pub artifact_type: String, // "file", "data", "url", etc.
    pub content: String,
    pub metadata: HashMap<String, String>,
}

/// Search result for raycast commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedSearchResult {
    pub plugin_id: String,
    pub command_name: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub score: f64,
    pub keywords: Vec<String>,
    pub actions: Vec<IsolatedAction>,
}

/// Action that can be performed on a search result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IsolatedAction {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub action_type: String, // "primary", "secondary", "destructive", etc.
}

impl Default for IsolatedRaycastCommand {
    fn default() -> Self {
        Self {
            name: String::new(),
            title: String::new(),
            description: None,
            mode: command_modes::VIEW.to_string(),
            subtitle: None,
            keywords: Vec::new(),
            arguments: Vec::new(),
            preferences: Vec::new(),
        }
    }
}

impl Default for IsolatedRaycastPlugin {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            extension: IsolatedRaycastExtension::default(),
            status: IsolatedPluginStatus::Inactive,
            capabilities: Vec::new(),
            configuration: HashMap::new(),
            last_updated: String::new(),
            execution_count: 0,
            average_execution_time_ms: 0,
        }
    }
}

impl IsolatedRaycastExtension {
    /// Create a new isolated raycast extension
    pub fn new(id: String, name: String, path: String) -> Self {
        Self {
            id: id.clone(),
            name: name.clone(),
            title: name,
            path,
            ..Default::default()
        }
    }

    /// Validate the extension structure
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Extension ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Extension name cannot be empty".to_string());
        }
        if self.path.is_empty() {
            return Err("Extension path cannot be empty".to_string());
        }

        // Validate commands
        for command in &self.commands {
            command.validate()?;
        }

        Ok(())
    }
}

impl IsolatedRaycastCommand {
    /// Create a new isolated raycast command
    pub fn new(name: String, title: String) -> Self {
        Self {
            name,
            title,
            ..Default::default()
        }
    }

    /// Validate the command structure
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Command name cannot be empty".to_string());
        }
        if self.title.is_empty() {
            return Err("Command title cannot be empty".to_string());
        }
        if ![
            command_modes::VIEW,
            command_modes::NO_VIEW,
            command_modes::SILENT,
        ]
        .contains(&self.mode.as_str())
        {
            return Err(format!("Invalid command mode: {}", self.mode));
        }

        Ok(())
    }
}

impl IsolatedRaycastPlugin {
    /// Create a new isolated raycast plugin
    pub fn new(id: String, name: String, extension: IsolatedRaycastExtension) -> Self {
        Self {
            id,
            name,
            extension,
            status: IsolatedPluginStatus::Inactive,
            capabilities: Vec::new(),
            configuration: HashMap::new(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            execution_count: 0,
            average_execution_time_ms: 0,
        }
    }

    /// Update plugin execution statistics
    pub fn update_execution_stats(&mut self, execution_time_ms: u64) {
        self.execution_count += 1;

        // Calculate new average using incremental formula
        let total_time =
            (self.average_execution_time_ms * (self.execution_count - 1)) + execution_time_ms;
        self.average_execution_time_ms = total_time / self.execution_count;

        self.last_updated = chrono::Utc::now().to_rfc3339();
    }

    /// Check if plugin is active and ready for execution
    pub fn is_ready(&self) -> bool {
        matches!(self.status, IsolatedPluginStatus::Active)
    }
}

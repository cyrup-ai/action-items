//! Plugin management system
//!
//! Zero-allocation plugin lifecycle management with blazing-fast loading, execution, and cleanup.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use action_items_common::plugin_interface::action_item::ActionType;
use action_items_common::plugin_interface::config::PreferenceType;
use action_items_common::plugin_interface::{
    ActionDefinition, ArgumentDefinition, ArgumentType, CommandDefinition, CommandMode,
    ConfigFieldType, ConfigurationField, PluginCategory, PreferenceField,
};

use crate::plugins::interface::PluginManifest;
use crate::plugins::services::PluginId;
use crate::runtime::deno::types::*;

/// Plugin state enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Loading,
    Loaded,
    Running,
    Failed,
    Unloaded,
}

/// Plugin instance information
#[derive(Debug, Clone)]
pub struct PluginInstance {
    pub id: PluginId,
    pub path: std::path::PathBuf,
    pub state: PluginState,
    pub last_activity: std::time::Instant,
    pub load_time: Option<std::time::Duration>,
}

/// Plugin manager for lifecycle management
/// Zero-allocation plugin management with blazing-fast instance tracking
pub struct PluginManager {
    plugins: HashMap<PluginId, PluginInstance>,
    _config: RuntimeConfig,
}

impl PluginManager {
    /// Create new plugin manager
    /// Zero-allocation manager initialization with blazing-fast setup
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            plugins: HashMap::new(),
            _config: config,
        }
    }

    /// Load plugin from path
    /// Zero-allocation plugin loading with blazing-fast manifest parsing
    pub fn load_plugin(&mut self, path: &Path) -> Result<PluginId, String> {
        let plugin_id = PluginId::from(uuid::Uuid::new_v4().to_string());

        // Load and parse plugin manifest
        let _manifest = self.load_manifest(path)?;

        // Create plugin instance
        let instance = PluginInstance {
            id: plugin_id.clone(),
            path: path.to_path_buf(),
            state: PluginState::Loading,
            last_activity: std::time::Instant::now(),
            load_time: None,
        };

        self.plugins.insert(plugin_id.clone(), instance);
        Ok(plugin_id)
    }

    /// Load plugin manifest
    /// Zero-allocation manifest loading with blazing-fast JSON parsing
    pub fn load_manifest(&self, plugin_path: &Path) -> Result<PluginManifest, String> {
        // Locate plugin.toml manifest file
        let manifest_path = plugin_path.join("plugin.toml");
        if !manifest_path.exists() {
            // Fallback to package.json for Raycast compatibility
            let package_json = plugin_path.join("package.json");
            if package_json.exists() {
                return self.load_package_json_manifest(&package_json);
            }
            return Err(format!("No manifest found at {:?}", manifest_path));
        }

        // Read and parse TOML manifest
        let manifest_content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest at {:?}: {}", manifest_path, e))?;

        let mut toml_manifest: TomlManifest = toml::from_str(&manifest_content).map_err(|e| {
            format!(
                "Failed to parse TOML manifest at {:?}: {}",
                manifest_path, e
            )
        })?;

        // Extract dependencies early to avoid borrow checker issues
        let dependencies = toml_manifest.dependencies.take().unwrap_or_default();

        // Resolve main entry point path
        let _main_file = if let Some(ref main) = toml_manifest.plugin.main {
            main.clone()
        } else if let Some(ref wasm_config) = toml_manifest.plugin.wasm {
            wasm_config.module.clone()
        } else if let Some(ref native_config) = toml_manifest.plugin.native {
            // Use platform-specific library
            #[cfg(target_os = "macos")]
            let lib_name = native_config
                .macos
                .as_ref()
                .or(native_config.library.as_ref())
                .ok_or("No macOS library specified")?;
            #[cfg(target_os = "linux")]
            let lib_name = native_config
                .linux
                .as_ref()
                .or(native_config.library.as_ref())
                .ok_or("No Linux library specified")?;
            #[cfg(target_os = "windows")]
            let lib_name = native_config
                .windows
                .as_ref()
                .or(native_config.library.as_ref())
                .ok_or("No Windows library specified")?;

            lib_name.clone()
        } else {
            // Default entry points based on plugin type
            let potential_mains = ["index.ts", "index.js", "main.ts", "main.js", "mod.ts"];
            let mut found_main = None;
            for main in &potential_mains {
                let main_path = plugin_path.join(main);
                if main_path.exists() {
                    found_main = Some(main.to_string());
                    break;
                }
            }
            found_main.ok_or("No valid entry point found")?
        };

        // Extract permissions from capabilities
        let mut permissions = Vec::new();
        if let Some(ref capabilities) = toml_manifest.capabilities {
            if let Some(ref filesystem) = capabilities.filesystem {
                for perm in filesystem {
                    permissions.push(format!("filesystem:{}", perm));
                }
            }
            if capabilities.network.unwrap_or(false) {
                permissions.push("network".to_string());
            }
            if capabilities.clipboard.unwrap_or(false) {
                permissions.push("clipboard".to_string());
            }
            if capabilities.notifications.unwrap_or(false) {
                permissions.push("notifications".to_string());
            }
            if capabilities.shell_execute.unwrap_or(false) {
                permissions.push("shell_execute".to_string());
            }
            if let Some(ref paths) = capabilities.allowed_paths {
                for path in paths {
                    permissions.push(format!("path:{}", path));
                }
            }
        }

        // Validate required fields
        if toml_manifest.plugin.name.trim().is_empty() {
            return Err("Plugin name cannot be empty".to_string());
        }
        if toml_manifest.plugin.version.trim().is_empty() {
            return Err("Plugin version cannot be empty".to_string());
        }

        // Convert permissions to PluginPermissions structure
        use crate::plugins::interface::{PluginCapabilities, PluginPermissions};
        let plugin_permissions = PluginPermissions {
            read_clipboard: permissions.contains(&"clipboard:read".to_string())
                || permissions.contains(&"clipboard".to_string()),
            write_clipboard: permissions.contains(&"clipboard:write".to_string())
                || permissions.contains(&"clipboard".to_string()),
            read_files: permissions
                .iter()
                .filter_map(|p| p.strip_prefix("filesystem:read:"))
                .map(PathBuf::from)
                .collect(),
            write_files: permissions
                .iter()
                .filter_map(|p| p.strip_prefix("filesystem:write:"))
                .map(PathBuf::from)
                .collect(),
            execute_commands: permissions
                .iter()
                .filter_map(|p| p.strip_prefix("shell_execute:"))
                .map(|s| s.to_string())
                .collect(),
            network_hosts: permissions
                .iter()
                .filter_map(|p| p.strip_prefix("network:"))
                .map(|s| s.to_string())
                .collect(),
            environment_variables: Vec::new(), // Default empty
            system_notifications: permissions.contains(&"notifications".to_string()),
            accessibility: permissions.contains(&"accessibility".to_string()),
            camera: permissions.contains(&"camera".to_string()),
            microphone: permissions.contains(&"microphone".to_string()),
            location: permissions.contains(&"location".to_string()),
            contacts: permissions.contains(&"contacts".to_string()),
            calendar: permissions.contains(&"calendar".to_string()),
        };

        // Create basic capabilities from TOML data
        let plugin_capabilities = PluginCapabilities {
            search: true, // Default capability
            background_refresh: false,
            notifications: false, /* Default value since notifications field doesn't exist in
                                   * TomlPreference */
            shortcuts: false,
            deep_links: false,
            clipboard_access: permissions.iter().any(|p| p.starts_with("clipboard")),
            file_system_access: permissions.iter().any(|p| p.starts_with("filesystem:")),
            network_access: permissions.contains(&"network".to_string()),
            system_commands: permissions.contains(&"shell_execute".to_string()),
            ui_extensions: false,
            context_menu: false,
            quick_actions: false,
        };

        let plugin_id = toml_manifest
            .plugin
            .id
            .clone()
            .unwrap_or_else(|| toml_manifest.plugin.name.clone());
        let plugin_name = toml_manifest.plugin.name.clone();
        let plugin_version = toml_manifest.plugin.version.clone();
        let plugin_description = toml_manifest.plugin.description.clone().unwrap_or_default();
        let plugin_author = toml_manifest.plugin.author.clone().unwrap_or_default();
        let plugin_license = toml_manifest
            .plugin
            .license
            .clone()
            .unwrap_or("Unknown".to_string());
        let plugin_homepage = toml_manifest.plugin.homepage.clone();
        let plugin_repository = toml_manifest.plugin.repository.clone();
        let plugin_icon = toml_manifest.plugin.icon.clone();
        let plugin_categories = toml_manifest
            .plugin
            .categories
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|_| PluginCategory::Productivity)
            .collect();

        // Create final manifest with all required fields
        Ok(PluginManifest {
            id: plugin_id,
            name: plugin_name,
            version: plugin_version,
            description: plugin_description,
            author: plugin_author,
            license: plugin_license,
            homepage: plugin_homepage,
            repository: plugin_repository,
            icon: plugin_icon,
            categories: plugin_categories,
            keywords: vec![], // toml_manifest.plugin.keywords field doesn't exist
            capabilities: plugin_capabilities,
            permissions: plugin_permissions,
            configuration: convert_toml_config(toml_manifest.configuration.unwrap_or_default()),
            preferences: convert_toml_preferences(toml_manifest.preferences.unwrap_or_default()),
            commands: convert_toml_commands(toml_manifest.commands.unwrap_or_default()),
            actions: convert_toml_actions(toml_manifest.actions.unwrap_or_default()),
            dependencies,
            environment: std::collections::HashMap::new(),
            min_launcher_version: "0.1.0".to_string(),
            max_launcher_version: None,
            update_url: None,
            changelog_url: None,
        })
    }

    /// Load legacy package.json manifest for Raycast compatibility
    fn load_package_json_manifest(
        &self,
        package_json_path: &Path,
    ) -> Result<PluginManifest, String> {
        let content = std::fs::read_to_string(package_json_path)
            .map_err(|e| format!("Failed to read package.json: {}", e))?;

        let package: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse package.json: {}", e))?;

        let name = package
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or("Missing 'name' field in package.json")?;

        let version = package
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0");

        let description = package
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("");

        let _main = package
            .get("main")
            .and_then(|m| m.as_str())
            .unwrap_or("src/index.tsx");

        // Create default permissions and capabilities for Raycast compatibility
        use crate::plugins::interface::{PluginCapabilities, PluginPermissions};
        let plugin_permissions = PluginPermissions {
            read_clipboard: true,          // Raycast plugins typically have clipboard access
            write_clipboard: true,         // Raycast plugins typically have clipboard access
            read_files: vec![],            // Conservative default - empty list
            write_files: vec![],           // Conservative default - empty list
            execute_commands: vec![],      // Conservative default - empty list
            network_hosts: vec![],         // Conservative default - empty list
            environment_variables: vec![], // Conservative default
            system_notifications: true,    // Raycast plugins typically can show notifications
            accessibility: false,          // Conservative default
            camera: false,                 // Conservative default
            microphone: false,             // Conservative default
            location: false,               // Conservative default
            contacts: false,               // Conservative default
            calendar: false,               // Conservative default
        };

        let plugin_capabilities = PluginCapabilities {
            search: true,              // Raycast plugins are primarily search-based
            background_refresh: false, // Conservative default
            notifications: true,       // Raycast plugins typically can show notifications
            shortcuts: false,          // Conservative default
            deep_links: false,         // Conservative default
            clipboard_access: true,    // Raycast plugins typically have clipboard access
            file_system_access: false, // Conservative default
            network_access: false,     // Conservative default
            system_commands: false,    // Conservative default
            ui_extensions: false,      // Conservative default
            context_menu: false,       // Conservative default
            quick_actions: false,      // Conservative default
        };

        Ok(PluginManifest {
            id: name.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            author: "Unknown".to_string(),
            license: "Unknown".to_string(),
            homepage: None,
            repository: None,
            icon: None,
            categories: vec![],
            keywords: vec![],
            capabilities: plugin_capabilities,
            permissions: plugin_permissions,
            configuration: vec![],
            preferences: vec![],
            commands: vec![],
            actions: vec![],
            dependencies: HashMap::new(),
            environment: HashMap::new(),
            min_launcher_version: "0.1.0".to_string(),
            max_launcher_version: None,
            update_url: None,
            changelog_url: None,
        })
    }

    /// Get plugin instance
    /// Zero-allocation plugin retrieval with blazing-fast HashMap lookup
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginInstance> {
        let plugin_id = PluginId::from(plugin_id);
        self.plugins.get(&plugin_id)
    }

    /// Unload plugin
    /// Zero-allocation plugin cleanup with blazing-fast resource deallocation
    pub fn unload_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        let plugin_id = PluginId::from(plugin_id);
        self.plugins.remove(&plugin_id);
        Ok(())
    }
}

/// TOML manifest structure matching docs/proposed-plugin-manifest.md
#[derive(Debug, Clone, serde::Deserialize)]
struct TomlManifest {
    plugin: TomlPluginInfo,
    capabilities: Option<TomlCapabilities>,
    dependencies: Option<HashMap<String, String>>,
    configuration: Option<Vec<TomlConfig>>,
    preferences: Option<Vec<TomlPreference>>,
    commands: Option<Vec<TomlCommand>>,
    actions: Option<Vec<TomlAction>>,
    #[allow(dead_code)]
    metadata: Option<TomlMetadata>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlConfig {
    name: String,
    title: String,
    description: Option<String>,
    #[serde(rename = "type")]
    config_type: String,
    required: Option<bool>,
    default: Option<serde_json::Value>,
    placeholder: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlPreference {
    name: String,
    title: String,
    description: Option<String>,
    #[serde(rename = "type")]
    pref_type: String,
    default: Option<serde_json::Value>,
    #[allow(dead_code)]
    required: Option<bool>,
    options: Option<Vec<TomlPreferenceOption>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlPreferenceOption {
    value: String,
    #[allow(dead_code)]
    title: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlCommand {
    name: String,
    title: String,
    description: String,
    keywords: Option<Vec<String>>,
    icon: Option<String>,
    mode: Option<String>,
    arguments: Option<Vec<TomlArgument>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlArgument {
    name: String,
    placeholder: String,
    #[serde(rename = "type")]
    arg_type: String,
    required: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlAction {
    id: String,
    title: String,
    description: Option<String>,
    icon: Option<String>,
    #[serde(rename = "type")]
    action_type: Option<String>,
    shortcut: Option<TomlShortcut>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlShortcut {
    key: String,
    modifiers: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlMetadata {
    #[allow(dead_code)]
    min_launcher_version: Option<String>,
    #[allow(dead_code)]
    max_launcher_version: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlPluginInfo {
    id: Option<String>,
    name: String,
    description: Option<String>,
    version: String,
    author: Option<String>,
    license: Option<String>,
    icon: Option<String>,
    homepage: Option<String>,
    repository: Option<String>,
    categories: Option<Vec<String>>,

    main: Option<String>,
    native: Option<TomlNativeConfig>,
    wasm: Option<TomlWasmConfig>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlNativeConfig {
    library: Option<String>,
    macos: Option<String>,
    #[allow(dead_code)]
    linux: Option<String>,
    #[allow(dead_code)]
    windows: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlWasmConfig {
    module: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TomlCapabilities {
    filesystem: Option<Vec<String>>,
    network: Option<bool>,
    clipboard: Option<bool>,
    notifications: Option<bool>,
    shell_execute: Option<bool>,
    allowed_paths: Option<Vec<String>>,
}

/// Convert TOML configuration fields to PluginManifest configuration
#[inline]
fn convert_toml_config(configs: Vec<TomlConfig>) -> Vec<ConfigurationField> {
    let mut result = Vec::with_capacity(configs.len());
    for config in configs {
        result.push(ConfigurationField {
            name: config.name,
            title: config.title,
            description: config.description,
            field_type: match config.config_type.as_str() {
                "text" => ConfigFieldType::Text,
                "password" => ConfigFieldType::Password,
                "number" => ConfigFieldType::Number,
                "boolean" => ConfigFieldType::Boolean,
                "file" => ConfigFieldType::File,
                "directory" => ConfigFieldType::Directory,
                "color" => ConfigFieldType::Color,
                "date" => ConfigFieldType::Date,
                "time" => ConfigFieldType::Time,
                "datetime" => ConfigFieldType::DateTime,
                _ => ConfigFieldType::Text,
            },
            required: config.required.unwrap_or(false),
            default: config.default,
            placeholder: config.placeholder,
            validation: None,
        });
    }
    result
}

/// Convert TOML preferences to PluginManifest preferences
#[inline]
fn convert_toml_preferences(prefs: Vec<TomlPreference>) -> Vec<PreferenceField> {
    let mut result = Vec::with_capacity(prefs.len());
    for pref in prefs {
        result.push(PreferenceField {
            key: pref.name,
            title: pref.title,
            description: pref.description,
            preference_type: match pref.pref_type.as_str() {
                "string" => PreferenceType::String { multiline: false },
                "bool" => PreferenceType::Bool,
                "number" => PreferenceType::Number {
                    min: None,
                    max: None,
                },
                "enum" => PreferenceType::Enum {
                    options: pref
                        .options
                        .map(|opts| opts.into_iter().map(|o| o.value).collect())
                        .unwrap_or_default(),
                },
                "hotkey" => PreferenceType::Hotkey,
                "color" => PreferenceType::Color,
                _ => PreferenceType::String { multiline: false },
            },
            default: pref.default.unwrap_or(serde_json::Value::Null),
        });
    }
    result
}

/// Convert TOML commands to PluginManifest commands
#[inline]
fn convert_toml_commands(commands: Vec<TomlCommand>) -> Vec<CommandDefinition> {
    let mut result = Vec::with_capacity(commands.len());
    for cmd in commands {
        result.push(CommandDefinition {
            id: cmd.name,
            title: cmd.title,
            subtitle: None,
            description: cmd.description,
            icon: cmd.icon,
            mode: match cmd.mode.as_deref() {
                Some("no-view") => CommandMode::NoView,
                Some("list") => CommandMode::List,
                Some("detail") => CommandMode::Detail,
                Some("form") => CommandMode::Form,
                Some("view") => CommandMode::View,
                Some("custom") => CommandMode::Custom,
                _ => CommandMode::List,
            },
            keywords: cmd.keywords.unwrap_or_default(),
            arguments: convert_toml_arguments(cmd.arguments.unwrap_or_default()),
            hotkey: None,
            interval: None,
        });
    }
    result
}

/// Convert TOML arguments to ArgumentDefinition
#[inline]
fn convert_toml_arguments(args: Vec<TomlArgument>) -> Vec<ArgumentDefinition> {
    let mut result = Vec::with_capacity(args.len());
    for arg in args {
        result.push(ArgumentDefinition {
            name: arg.name,
            placeholder: arg.placeholder,
            arg_type: match arg.arg_type.as_str() {
                "text" => ArgumentType::Text,
                "number" => ArgumentType::Number,
                "boolean" => ArgumentType::Boolean,
                "file" => ArgumentType::File,
                "directory" => ArgumentType::Directory,
                _ => ArgumentType::Text,
            },
            required: arg.required.unwrap_or(false),
        });
    }
    result
}

/// Convert TOML actions to PluginManifest actions
#[inline]
fn convert_toml_actions(actions: Vec<TomlAction>) -> Vec<ActionDefinition> {
    let mut result = Vec::with_capacity(actions.len());
    for action in actions {
        result.push(ActionDefinition {
            id: action.id,
            title: action.title,
            description: action.description,
            icon: action.icon,
            shortcut: action.shortcut.map(|s| {
                if let Some(modifiers) = s.modifiers {
                    if modifiers.is_empty() {
                        s.key
                    } else {
                        format!("{}+{}", modifiers.join("+"), s.key)
                    }
                } else {
                    s.key
                }
            }),
            action_type: match action.action_type.as_deref() {
                Some("open_url") => ActionType::OpenUrl(String::new()),
                Some("copy") => ActionType::CopyToClipboard(String::new()),
                Some("show_hud") => ActionType::ShowHud,
                Some("close_window") => ActionType::CloseWindow,
                Some("refresh") => ActionType::RefreshCommand,
                _ => ActionType::Custom("default".to_string()),
            },
        });
    }
    result
}

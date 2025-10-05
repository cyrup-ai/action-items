//! Launcher service events
//!
//! Event definitions for the ECS launcher service, providing loose coupling
//! between launcher components and external systems.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::resources::ActionDefinition;

/// Action execution request event
#[derive(Event, Debug, Clone)]
pub struct ActionExecuteRequested {
    pub action_id: String,
    pub requester: String,
    pub parameters: Value,
    pub execution_context: ExecutionContext,
}

/// Action execution completion event
#[derive(Event, Debug, Clone)]
pub struct ActionExecuteCompleted {
    pub action_id: String,
    pub requester: String,
    pub success: bool,
    pub result: Option<Value>,
    pub error_message: Option<String>,
    pub execution_time: std::time::Duration,
}

/// Search request event
#[derive(Event, Debug, Clone)]
pub struct SearchRequested {
    pub query: String,
    pub requester: String,
    pub search_type: SearchType,
    pub filters: SearchFilters,
}

impl SearchRequested {
    pub fn new(query: String, requester: String) -> Self {
        Self {
            query,
            requester,
            search_type: SearchType::Combined,
            filters: SearchFilters::default(),
        }
    }
}

/// Search completion event
#[derive(Event, Debug, Clone)]
pub struct SearchCompleted {
    pub query: String,
    pub requester: String,
    pub results: Vec<SearchResult>,
    pub result_count: usize,
    pub search_duration: std::time::Duration,
}

/// UI state change event
#[derive(Event, Debug, Clone)]
pub struct UIStateChanged {
    pub previous_state: UIState,
    pub new_state: UIState,
    pub trigger: UITrigger,
    pub requester: String,
}

/// Action registration request event
#[derive(Event, Debug, Clone)]
pub struct ActionRegisterRequested {
    pub action: ActionDefinition,
    pub requester: String,
    pub source_plugin: Option<String>,
}

/// Search query change event for real-time search updates
#[derive(Event, Debug, Clone)]
pub struct SearchQueryChanged {
    pub query: String,
    pub requester: String,
    pub search_type: SearchType,
    pub max_results: Option<usize>,
}

/// Plugin discovery request event
#[derive(Event, Debug, Clone)]
pub struct PluginDiscoveryRequested {
    pub discovery_paths: Vec<String>,
    pub requester: String,
    pub recursive: bool,
}

/// Plugin discovery event
#[derive(Event, Debug, Clone)]
pub struct PluginDiscovered {
    pub plugin_info: DiscoveredPlugin,
    pub discovery_method: DiscoveryMethod,
}

/// Window trigger source enumeration for comprehensive trigger tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WindowTrigger {
    /// Triggered by global hotkey activation
    Hotkey,
    /// Triggered by UI interaction (button click, menu selection, etc.)
    UI,
    /// Triggered by external API call
    API,
    /// Triggered by plugin execution
    Plugin,
    /// Triggered by system event or automation
    System,
}

impl Default for WindowTrigger {
    fn default() -> Self {
        Self::UI
    }
}

/// Launcher window toggle event
#[derive(Event, Debug, Clone)]
pub struct LauncherWindowToggled {
    pub visible: bool,
    pub trigger: WindowTrigger,
    pub requester: String,
}

/// Launcher preferences updated event
#[derive(Event, Debug, Clone)]
pub struct LauncherPreferencesUpdated {
    pub preferences: LauncherPreferences,
    pub requester: String,
}

// ============================================================================
// SUPPORTING TYPES
// ============================================================================

/// Execution context for action requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub source: ExecutionSource,
    pub priority: ExecutionPriority,
    pub timeout: Option<std::time::Duration>,
    pub environment: std::collections::HashMap<String, String>,
    pub requester: String,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            source: ExecutionSource::UI,
            priority: ExecutionPriority::Normal,
            timeout: Some(std::time::Duration::from_secs(30)),
            environment: std::collections::HashMap::new(),
            requester: "default".to_string(),
        }
    }
}

/// Source of action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionSource {
    UI,
    Hotkey,
    API,
    Plugin,
    Automation,
}

/// Execution priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Search type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchType {
    Actions,
    Files,
    Applications,
    Web,
    Combined,
}

/// Search filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub date_range: Option<DateRange>,
    pub size_range: Option<SizeRange>,
    pub exclude_patterns: Vec<String>,
}

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: Option<std::time::SystemTime>,
    pub end: Option<std::time::SystemTime>,
}

/// Size range filter (for files)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeRange {
    pub min_bytes: Option<u64>,
    pub max_bytes: Option<u64>,
}

/// Search result definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub icon_path: Option<String>,
    pub score: f32,
    pub result_type: SearchResultType,
    pub metadata: std::collections::HashMap<String, Value>,
}

/// Search result type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchResultType {
    Action,
    File,
    Application,
    WebResult,
    Plugin,
}

/// UI state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UIState {
    #[default]
    Hidden,
    Visible,
    Searching,
    Executing,
    Preferences,
    Error,
}

/// UI state change trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UITrigger {
    Hotkey,
    Click,
    Focus,
    Timer,
    SystemEvent,
}

/// Discovered plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPlugin {
    pub name: String,
    pub version: String,
    pub path: std::path::PathBuf,
    pub capabilities: Vec<String>,
    pub metadata: std::collections::HashMap<String, Value>,
}

/// Plugin discovery method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    FileSystem,
    Registry,
    Network,
    Manual,
}

/// Comprehensive launcher preferences configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherPreferences {
    // UI Appearance Settings
    pub theme: String,
    pub font_size: u16,
    pub window_opacity: f32,
    pub high_contrast_mode: bool,

    // Search Behavior Configuration
    pub max_results: usize,
    pub search_delay: std::time::Duration,
    pub auto_execute_single_result: bool,
    pub fuzzy_search_enabled: bool,
    pub search_history_enabled: bool,

    // Performance Settings
    pub enable_animations: bool,
    pub ui_update_frequency: std::time::Duration,
    pub background_indexing: bool,
    pub cache_search_results: bool,

    // Plugin Management
    pub plugins_enabled: bool,
    pub auto_discover_plugins: bool,
    pub plugin_security_level: PluginSecurityLevel,
    pub enabled_plugin_categories: Vec<String>,

    // Hotkey Configuration
    pub global_hotkeys: std::collections::HashMap<String, String>,
    pub hotkey_modifiers_required: bool,
    pub hotkey_feedback_enabled: bool,

    // Privacy and Security
    pub telemetry_enabled: bool,
    pub auto_update_enabled: bool,
    pub secure_input_mode: bool,
    pub usage_analytics: bool,

    // Accessibility Options
    pub screen_reader_support: bool,
    pub keyboard_navigation_only: bool,
    pub reduce_motion: bool,
    pub voice_feedback: bool,

    // Advanced Configuration
    pub startup_behavior: StartupBehavior,
    pub window_positioning: WindowPositioning,
    pub resource_usage_limits: ResourceLimits,
}

/// Plugin security level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginSecurityLevel {
    Disabled,
    Restricted,
    Standard,
    Elevated,
    Administrative,
}

/// Launcher startup behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StartupBehavior {
    Hidden,
    Minimized,
    Visible,
    LastState,
}

/// Window positioning preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPositioning {
    pub remember_position: bool,
    pub center_on_screen: bool,
    pub follow_mouse_cursor: bool,
    pub multi_monitor_behavior: MultiMonitorBehavior,
}

/// Multi-monitor behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MultiMonitorBehavior {
    PrimaryMonitor,
    ActiveMonitor,
    MouseMonitor,
    LastUsedMonitor,
}

/// Resource usage limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_usage_mb: u64,
    pub max_cpu_percentage: f32,
    pub max_background_threads: usize,
    pub network_timeout_seconds: u64,
}

impl Default for LauncherPreferences {
    fn default() -> Self {
        Self {
            // UI Appearance Settings - Modern, accessible defaults
            theme: "system".to_string(),
            font_size: 14,
            window_opacity: 0.95,
            high_contrast_mode: false,

            // Search Behavior - Fast, efficient defaults
            max_results: 50,
            search_delay: std::time::Duration::from_millis(150),
            auto_execute_single_result: false,
            fuzzy_search_enabled: true,
            search_history_enabled: true,

            // Performance Settings - Balanced performance
            enable_animations: true,
            ui_update_frequency: std::time::Duration::from_millis(16), // 60 FPS
            background_indexing: true,
            cache_search_results: true,

            // Plugin Management - Secure by default
            plugins_enabled: true,
            auto_discover_plugins: false, // Security: require explicit enabling
            plugin_security_level: PluginSecurityLevel::Standard,
            enabled_plugin_categories: vec![
                "productivity".to_string(),
                "utilities".to_string(),
                "files".to_string(),
            ],

            // Hotkey Configuration - Standard launcher hotkey
            global_hotkeys: {
                let mut hotkeys = std::collections::HashMap::new();
                hotkeys.insert("toggle_launcher".to_string(), "cmd+space".to_string());
                hotkeys.insert("preferences".to_string(), "cmd+comma".to_string());
                hotkeys.insert("quit".to_string(), "cmd+q".to_string());
                hotkeys
            },
            hotkey_modifiers_required: true,
            hotkey_feedback_enabled: true,

            // Privacy and Security - Privacy-first defaults
            telemetry_enabled: false,  // Privacy: opt-in only
            auto_update_enabled: true, // Security: auto-updates enabled
            secure_input_mode: false,
            usage_analytics: false, // Privacy: opt-in only

            // Accessibility Options - Inclusive defaults
            screen_reader_support: false, // Auto-detected by system
            keyboard_navigation_only: false,
            reduce_motion: false, // Respect system preferences
            voice_feedback: false,

            // Advanced Configuration
            startup_behavior: StartupBehavior::Hidden,
            window_positioning: WindowPositioning {
                remember_position: true,
                center_on_screen: true,
                follow_mouse_cursor: false,
                multi_monitor_behavior: MultiMonitorBehavior::ActiveMonitor,
            },
            resource_usage_limits: ResourceLimits {
                max_memory_usage_mb: 256,
                max_cpu_percentage: 10.0,
                max_background_threads: 4,
                network_timeout_seconds: 10,
            },
        }
    }
}

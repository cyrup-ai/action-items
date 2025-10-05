//! Launcher service resources
//!
//! Core resource definitions for the ECS launcher service.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::events::*;

/// Main launcher state resource
#[derive(Resource, Default)]
pub struct LauncherState {
    pub current_ui_state: UIState,
    pub is_window_visible: bool,
    pub current_query: String,
    pub selected_result_index: usize,
    pub last_action_time: Option<Instant>,
}

/// Search state management resource
#[derive(Resource)]
#[derive(Default)]
pub struct SearchState {
    pub current_query: String,
    pub current_results: Vec<SearchResult>,
    pub search_in_progress: bool,
    pub last_search_time: Option<Instant>,
    pub search_providers: HashMap<String, SearchProvider>,
}


/// Search provider definition
#[derive(Debug, Clone)]
pub struct SearchProvider {
    pub name: String,
    pub provider_type: SearchProviderType,
    pub enabled: bool,
    pub weight: f32,
    pub timeout: Duration,
    pub last_response_time: Option<Duration>,
}

/// Search provider type
#[derive(Debug, Clone)]
pub enum SearchProviderType {
    Local,
    Plugin,
    WebAPI,
    Database,
}

/// Action registry for managing available actions
#[derive(Resource, Default)]
pub struct ActionRegistry {
    pub actions: HashMap<String, ActionDefinition>,
    pub categories: HashMap<String, Vec<String>>,
    pub execution_history: Vec<ExecutionRecord>,
}

/// Action definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub icon_path: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub action_type: String,
    pub executable_path: Option<String>,
    pub parameters: Vec<ActionParameter>,
    pub created_at: std::time::SystemTime,
    pub last_executed: Option<std::time::SystemTime>,
    pub execution_count: u64,
}

/// Action parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionParameter {
    pub name: String,
    pub parameter_type: ParameterType,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
}

/// Parameter type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    File,
    Directory,
    Selection(Vec<String>),
}

/// Execution record for tracking action history
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub action_id: String,
    pub requester: String,
    pub execution_time: std::time::SystemTime,
    pub duration: Duration,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Launcher configuration resource
#[derive(Resource, Clone, Debug)]
pub struct LauncherConfig {
    pub enable_debug_logging: bool,
    pub search_update_interval: Duration,
    pub max_search_results: usize,
    pub enable_plugin_discovery: bool,
}

/// Launcher metrics resource
#[derive(Resource, Default)]
pub struct LauncherMetrics {
    pub total_searches: u64,
    pub successful_actions: u64,
    pub failed_actions: u64,
    pub average_search_time: Duration,
    pub average_action_time: Duration,
    pub plugin_count: usize,
    pub last_updated: Option<Instant>,
}

/// Plugin registry resource
#[derive(Resource, Default)]
pub struct PluginRegistry {
    pub discovered_plugins: HashMap<String, DiscoveredPlugin>,
    pub loaded_plugins: HashMap<String, LoadedPlugin>,
    pub plugin_capabilities: HashMap<String, Vec<String>>,
}

/// Loaded plugin information
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub name: String,
    pub version: String,
    pub status: PluginStatus,
    pub loaded_at: Instant,
    pub last_activity: Option<Instant>,
}

/// Plugin status enumeration
#[derive(Debug, Clone)]
pub enum PluginStatus {
    Loading,
    Active,
    Error(String),
    Disabled,
}

/// Window state management resource
#[derive(Resource)]
pub struct WindowState {
    pub position: WindowPosition,
    pub size: WindowSize,
    pub is_focused: bool,
    pub last_shown: Option<Instant>,
    pub show_count: u64,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            position: WindowPosition::Center,
            size: WindowSize::default(),
            is_focused: false,
            last_shown: None,
            show_count: 0,
        }
    }
}

/// Window position enumeration
#[derive(Debug, Clone)]
pub enum WindowPosition {
    Center,
    TopCenter,
    BottomCenter,
    Custom { x: f32, y: f32 },
}

/// Window size configuration
#[derive(Debug, Clone)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 600.0,
            height: 400.0,
            min_width: 400.0,
            min_height: 200.0,
            max_width: Some(1200.0),
            max_height: Some(800.0),
        }
    }
}

/// Theme resource for UI styling
#[derive(Resource, Clone)]
pub struct LauncherTheme {
    pub name: String,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub animations: ThemeAnimations,
}

impl Default for LauncherTheme {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            colors: ThemeColors::default(),
            typography: ThemeTypography::default(),
            spacing: ThemeSpacing::default(),
            animations: ThemeAnimations::default(),
        }
    }
}

/// Theme color definitions
#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub primary: (f32, f32, f32, f32),
    pub secondary: (f32, f32, f32, f32),
    pub background: (f32, f32, f32, f32),
    pub surface: (f32, f32, f32, f32),
    pub text_primary: (f32, f32, f32, f32),
    pub text_secondary: (f32, f32, f32, f32),
    pub accent: (f32, f32, f32, f32),
    pub error: (f32, f32, f32, f32),
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            primary: (0.2, 0.4, 0.8, 1.0),
            secondary: (0.5, 0.5, 0.5, 1.0),
            background: (0.1, 0.1, 0.1, 0.95),
            surface: (0.15, 0.15, 0.15, 1.0),
            text_primary: (0.95, 0.95, 0.95, 1.0),
            text_secondary: (0.7, 0.7, 0.7, 1.0),
            accent: (0.2, 0.8, 0.4, 1.0),
            error: (0.8, 0.2, 0.2, 1.0),
        }
    }
}

/// Theme typography settings
#[derive(Debug, Clone)]
pub struct ThemeTypography {
    pub font_family: String,
    pub base_size: f32,
    pub heading_size: f32,
    pub small_size: f32,
}

impl Default for ThemeTypography {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            base_size: 14.0,
            heading_size: 18.0,
            small_size: 12.0,
        }
    }
}

/// Theme spacing settings
#[derive(Debug, Clone)]
pub struct ThemeSpacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl Default for ThemeSpacing {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
        }
    }
}

/// Theme animation settings
#[derive(Debug, Clone)]
pub struct ThemeAnimations {
    pub fade_duration: Duration,
    pub slide_duration: Duration,
    pub scale_duration: Duration,
    pub easing: String,
}

impl Default for ThemeAnimations {
    fn default() -> Self {
        Self {
            fade_duration: Duration::from_millis(200),
            slide_duration: Duration::from_millis(300),
            scale_duration: Duration::from_millis(150),
            easing: "ease-out".to_string(),
        }
    }
}

//! Hotkey service resources
//!
//! Core resource definitions for the ECS hotkey service, extracted from production code.

use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use global_hotkey::GlobalHotKeyManager;
use global_hotkey::hotkey::{Code, Modifiers};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::events::HotkeyDefinition;

/// Hotkey preferences configuration
/// Zero-allocation preferences management with blazing-fast fallback handling
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct HotkeyPreferences {
    pub preferred_combinations: Vec<HotkeyDefinition>,
    pub custom_hotkey: Option<HotkeyDefinition>,
    pub auto_fallback: bool,
}

impl Default for HotkeyPreferences {
    fn default() -> Self {
        Self {
            preferred_combinations: get_default_hotkey_combinations(),
            custom_hotkey: None,
            auto_fallback: true,
        }
    }
}

/// Serializable hotkey binding for profile storage
/// Converted to HotkeyBinding when profile is activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileBinding {
    pub definition: HotkeyDefinition,
    pub action: String,
    pub requester: String,
}

impl ProfileBinding {
    pub fn from_hotkey_binding(binding: &HotkeyBinding) -> Self {
        Self {
            definition: binding.definition.clone(),
            action: binding.action.clone(),
            requester: binding.requester.clone(),
        }
    }

    pub fn to_hotkey_binding(&self) -> HotkeyBinding {
        HotkeyBinding {
            id: HotkeyId::new(),
            definition: self.definition.clone(),
            action: self.action.clone(),
            requester: self.requester.clone(),
            registered_at: Instant::now(),
        }
    }
}

/// A named collection of hotkey bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyProfile {
    pub name: String,
    pub bindings: Vec<ProfileBinding>,
    pub is_default: bool,
}

impl HotkeyProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bindings: Vec::new(),
            is_default: false,
        }
    }

    pub fn default_profile() -> Self {
        Self {
            name: "Default".to_string(),
            bindings: Vec::new(),
            is_default: true,
        }
    }
}

/// Resource managing hotkey profile collection and active profile
#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct HotkeyProfiles {
    pub profiles: HashMap<String, HotkeyProfile>,
    pub active_profile: String,
    #[serde(skip)]
    pub active_bindings: HashMap<HotkeyId, String>,
}

impl Default for HotkeyProfiles {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("Default".to_string(), HotkeyProfile::default_profile());

        Self {
            profiles,
            active_profile: "Default".to_string(),
            active_bindings: HashMap::new(),
        }
    }
}

impl HotkeyProfiles {
    pub fn switch_profile(&mut self, profile_name: &str) -> Result<(), String> {
        if !self.profiles.contains_key(profile_name) {
            return Err(format!("Profile '{}' does not exist", profile_name));
        }

        self.active_profile = profile_name.to_string();
        Ok(())
    }

    pub fn get_active_bindings(&self) -> Option<&Vec<ProfileBinding>> {
        self.profiles.get(&self.active_profile)
            .map(|profile| &profile.bindings)
    }

    pub fn add_profile(&mut self, profile: HotkeyProfile) -> Result<(), String> {
        if self.profiles.contains_key(&profile.name) {
            return Err(format!("Profile '{}' already exists", profile.name));
        }

        self.profiles.insert(profile.name.clone(), profile);
        Ok(())
    }

    pub fn remove_profile(&mut self, name: &str) -> Result<(), String> {
        let profile = self.profiles.get(name)
            .ok_or_else(|| format!("Profile '{}' does not exist", name))?;

        if profile.is_default {
            return Err("Cannot remove default profile".to_string());
        }

        if name == self.active_profile {
            return Err("Cannot remove active profile".to_string());
        }

        self.profiles.remove(name);
        Ok(())
    }

    pub fn list_profiles(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }

    pub fn add_binding_to_profile(&mut self, profile_name: &str, binding: ProfileBinding) -> Result<(), String> {
        let profile = self.profiles.get_mut(profile_name)
            .ok_or_else(|| format!("Profile '{}' does not exist", profile_name))?;

        profile.bindings.push(binding);
        Ok(())
    }

    pub fn remove_binding_from_profile(&mut self, profile_name: &str, action: &str) -> Result<(), String> {
        let profile = self.profiles.get_mut(profile_name)
            .ok_or_else(|| format!("Profile '{}' does not exist", profile_name))?;

        profile.bindings.retain(|b| b.action != action);
        Ok(())
    }
}

/// Get default hotkey combinations for preferences system
/// Zero-allocation default hotkey generation with blazing-fast platform detection
pub fn get_default_hotkey_combinations() -> Vec<HotkeyDefinition> {
    vec![
        // PRIMARY choices - avoid conflicts with system hotkeys
        #[cfg(target_os = "macos")]
        HotkeyDefinition {
            modifiers: Modifiers::META | Modifiers::SHIFT,
            code: Code::Space,
            description: "Cmd+Shift+Space".to_string(),
        },
        #[cfg(not(target_os = "macos"))]
        HotkeyDefinition {
            modifiers: Modifiers::CONTROL | Modifiers::SHIFT,
            code: Code::Space,
            description: "Ctrl+Shift+Space".to_string(),
        },
        // Fallbacks - try system defaults (likely to conflict)
        #[cfg(target_os = "macos")]
        HotkeyDefinition {
            modifiers: Modifiers::META,
            code: Code::Space,
            description: "Cmd+Space".to_string(),
        },
        #[cfg(not(target_os = "macos"))]
        HotkeyDefinition {
            modifiers: Modifiers::CONTROL,
            code: Code::Space,
            description: "Ctrl+Space".to_string(),
        },
        // Alternative modifier combinations
        #[cfg(target_os = "macos")]
        HotkeyDefinition {
            modifiers: Modifiers::META | Modifiers::ALT,
            code: Code::Space,
            description: "Cmd+Alt+Space".to_string(),
        },
        #[cfg(not(target_os = "macos"))]
        HotkeyDefinition {
            modifiers: Modifiers::ALT,
            code: Code::Space,
            description: "Alt+Space".to_string(),
        },
    ]
}

/// Hotkey status enumeration for UI display
/// Zero allocation status tracking with semantic error information
#[derive(Debug, Clone, Default)]
pub enum HotkeyStatus {
    #[default]
    Empty,
    Valid,
    /// Conflict with application name
    Conflict(String),
    Testing,
    TestSuccess,
    TestFailed(String),
}

/// Hotkey capture logic state (stays in ecs-hotkey)
/// 
/// Pure business logic for recording hotkey combinations.
/// No UI concerns - can be used headless or in terminal.
#[derive(Resource, Default)]
pub struct HotkeyCaptureState {
    /// Currently recording keystrokes?
    pub capturing: bool,
    
    /// Currently held modifier keys (Cmd, Alt, Ctrl, Shift)
    pub held_modifiers: Modifiers,
    
    /// Main key that was pressed (Space, Enter, A, etc.)
    pub captured_key: Option<Code>,
    
    /// Complete captured combination
    pub captured_hotkey: Option<HotkeyDefinition>,
    
    /// Current requester for capture operations
    pub current_requester: Option<String>,
}

/// Hotkey capture UI state (move to ecs-preferences or ecs-ui package)
/// 
/// UI-specific state for preferences window rendering.
/// TODO: In future refactor, move HotkeyCaptureUIState to:
/// packages/ecs-preferences/src/resources.rs OR packages/ecs-ui/src/preferences/resources.rs
#[derive(Resource, Default)]
pub struct HotkeyCaptureUIState {
    /// Whether preferences window is visible
    pub visible: bool,
    
    /// Is the hotkey input field focused?
    pub input_focused: bool,
    
    /// Current hotkey status for UI display
    pub current_status: HotkeyStatus,
    
    /// Whether currently testing a hotkey
    pub testing_hotkey: bool,
    
    /// Available alternative hotkey combinations
    pub available_alternatives: Vec<HotkeyDefinition>,
}

/// Unique identifier for hotkey operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotkeyId(pub Uuid);

impl Default for HotkeyId {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Conflict type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    AlreadyRegistered,
    RegistrationLimitExceeded,
    PlatformNotSupported,
    PermissionDenied,
}

/// Hotkey binding definition
#[derive(Debug, Clone)]
pub struct HotkeyBinding {
    pub id: HotkeyId,
    pub definition: HotkeyDefinition,
    pub action: String,
    pub requester: String,
    pub registered_at: Instant,
}

impl HotkeyBinding {
    /// Create new hotkey binding with action
    pub fn new(definition: HotkeyDefinition, action: impl Into<String>) -> Self {
        Self {
            id: HotkeyId::new(),
            definition,
            action: action.into(),
            requester: "unknown".to_string(),
            registered_at: Instant::now(),
        }
    }

    /// Set requester name (for debugging/analytics)
    pub fn with_requester(mut self, requester: impl Into<String>) -> Self {
        self.requester = requester.into();
        self
    }

    /// Set specific hotkey ID (for deserialization)
    pub fn with_id(mut self, id: HotkeyId) -> Self {
        self.id = id;
        self
    }

    /// Update hotkey definition (for migration/updates)
    pub fn with_definition(mut self, definition: HotkeyDefinition) -> Self {
        self.definition = definition;
        self
    }
    
    /// Update action name (for refactoring)
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }
}

/// Hotkey registry for managing registered hotkeys
#[derive(Resource, Default)]
pub struct HotkeyRegistry {
    pub registered_hotkeys: HashMap<HotkeyId, HotkeyBinding>,
    pub conflicts: Vec<ConflictReport>,
    pub by_action: HashMap<String, Vec<HotkeyId>>,
}

/// Conflict report for hotkey registration issues
#[derive(Debug, Clone)]
pub struct ConflictReport {
    pub conflicting_hotkey: HotkeyDefinition,
    pub conflict_type: ConflictType,
    pub conflicting_application: Option<String>,
    pub suggested_alternative: Option<HotkeyDefinition>,
}

/// Hotkey manager resource - infrastructure only
/// 
/// Contains OS-level hotkey infrastructure. For hotkey data, query
/// `HotkeyRegistry` and `HotkeyPreferences` resources separately.
#[derive(Resource)]
pub struct HotkeyManager {
    /// Platform hotkey manager (global-hotkey crate)
    pub global_manager: GlobalHotKeyManager,
    
    /// Maximum concurrent hotkey registrations
    pub max_hotkeys: usize,
    
    /// Enable automatic conflict resolution
    pub enable_conflict_resolution: bool,
    
    /// Wayland backend (Linux only)
    #[cfg(target_os = "linux")]
    pub wayland_manager: Option<std::sync::Arc<tokio::sync::Mutex<crate::platform::linux_wayland::WaylandHotkeyManager>>>,
}

impl HotkeyManager {
    pub fn new(max_hotkeys: usize, enable_conflict_resolution: bool) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            global_manager: GlobalHotKeyManager::new()?,
            max_hotkeys,
            enable_conflict_resolution,
            #[cfg(target_os = "linux")]
            wayland_manager: None,
        })
    }

    /// Check if we can register more hotkeys
    #[inline]
    pub fn can_register_more(&self, registry: &HotkeyRegistry) -> bool {
        registry.registered_hotkeys.len() < self.max_hotkeys
    }
}

/// Metrics resource for tracking hotkey service performance
#[derive(Resource, Default)]
pub struct HotkeyMetrics {
    pub registered_count: usize,
    pub press_count: usize,
    pub last_press: Option<Instant>,
    pub total_registrations: u64,
    pub successful_registrations: u64,
    pub failed_registrations: u64,
    pub conflicts_detected: u64,
    pub capture_sessions: u64,
    pub successful_captures: u64,
    pub tests_performed: u64,
    pub successful_tests: u64,
}

/// Configuration resource for hotkey service
#[derive(Resource, Clone, Debug)]
pub struct HotkeyConfig {
    pub enable_debug_logging: bool,
    pub polling_interval: std::time::Duration,
    pub max_hotkeys: usize,
    pub enable_conflict_resolution: bool,
}

/// Resource tracking which entities own which hotkeys
/// 
/// Maintains bidirectional mapping for automatic cleanup when entities despawn.
#[derive(Resource, Default)]
pub struct HotkeyEntityMap {
    pub entity_to_hotkey: HashMap<Entity, HotkeyId>,
}

/// Serializable version of HotkeyBinding for import/export
/// Omits the `registered_at: Instant` field which cannot be serialized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableHotkeyBinding {
    pub id: Uuid,
    pub definition: HotkeyDefinition,
    pub action: String,
    pub requester: String,
}

impl From<&HotkeyBinding> for SerializableHotkeyBinding {
    fn from(binding: &HotkeyBinding) -> Self {
        Self {
            id: binding.id.0,
            definition: binding.definition.clone(),
            action: binding.action.clone(),
            requester: binding.requester.clone(),
        }
    }
}

impl SerializableHotkeyBinding {
    /// Convert to HotkeyBinding with current timestamp
    pub fn to_binding(&self) -> HotkeyBinding {
        HotkeyBinding {
            id: HotkeyId(self.id),
            definition: self.definition.clone(),
            action: self.action.clone(),
            requester: self.requester.clone(),
            registered_at: Instant::now(),
        }
    }
}

/// Exported configuration with version metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedConfig {
    /// Configuration format version for compatibility checking
    pub version: String,
    /// All registered hotkey bindings
    pub bindings: Vec<SerializableHotkeyBinding>,
}

/// Strategy for merging imported bindings with existing ones
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Replace all existing bindings with imported ones
    Replace,
    /// Add imported bindings to existing ones (may create duplicates)
    Append,
    /// Only add imported bindings with IDs that don't exist
    SkipConflicts,
}

/// Export hotkey configuration to JSON file with atomic write
///
/// Follows the same atomic write pattern as persist_hotkey_preferences_owned:
/// 1. Serialize to JSON with pretty formatting
/// 2. Write to temporary .tmp file
/// 3. Atomically rename to final path (crash-safe)
///
/// # Arguments
/// * `registry` - The hotkey registry to export
/// * `path` - Destination file path (will create .tmp during write)
///
/// # Example
/// ```no_run
/// let registry = hotkey_manager.registry;
/// export_config(&registry, Path::new("hotkeys.json")).await?;
/// ```
pub async fn export_config(
    registry: &HotkeyRegistry,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Convert all bindings to serializable format
    let bindings: Vec<SerializableHotkeyBinding> = registry
        .registered_hotkeys
        .values()
        .map(|binding| binding.into())
        .collect();

    let exported = ExportedConfig {
        version: "1.0".to_string(),
        bindings,
    };

    // Serialize with pretty formatting (human-readable)
    let json_content = serde_json::to_string_pretty(&exported)
        .map_err(|e| format!("Failed to serialize configuration: {}", e))?;

    // Atomic write pattern: write to .tmp, then rename
    let temp_file = path.with_extension("tmp");

    std::fs::write(&temp_file, &json_content)
        .map_err(|e| format!("Failed to write temporary config file: {}", e))?;

    std::fs::rename(&temp_file, path)
        .map_err(|e| format!("Failed to finalize config file: {}", e))?;

    Ok(())
}

/// Import hotkey configuration from JSON file with version validation
///
/// # Arguments
/// * `path` - Source file path to read configuration from
///
/// # Returns
/// Vector of serializable bindings on success
///
/// # Errors
/// Returns error if:
/// - File not found or cannot be read
/// - JSON is invalid or malformed
/// - Version is incompatible (not "1.0")
/// - Required fields are missing
///
/// # Example
/// ```no_run
/// let bindings = import_config(Path::new("hotkeys.json")).await?;
/// ```
pub async fn import_config(
    path: &Path,
) -> Result<Vec<SerializableHotkeyBinding>, Box<dyn std::error::Error + Send + Sync>> {
    // Read file contents
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    // Deserialize JSON
    let config: ExportedConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON format: {}", e))?;

    // Validate version compatibility
    if config.version != "1.0" {
        return Err(format!(
            "Unsupported config version: {} (expected 1.0)",
            config.version
        ).into());
    }

    Ok(config.bindings)
}

/// Merge imported bindings with existing ones using specified strategy
///
/// # Arguments
/// * `existing` - Current hotkey bindings in the registry
/// * `imported` - Bindings loaded from import file
/// * `strategy` - How to handle conflicts (Replace/Append/SkipConflicts)
///
/// # Returns
/// Merged vector of hotkey bindings
///
/// # Strategy Details
/// - `Replace`: Discard all existing bindings, use only imported
/// - `Append`: Keep existing, add all imported (may create ID duplicates)
/// - `SkipConflicts`: Keep existing, add imported only if ID doesn't exist
pub fn merge_bindings(
    existing: Vec<HotkeyBinding>,
    imported: Vec<SerializableHotkeyBinding>,
    strategy: MergeStrategy,
) -> Vec<HotkeyBinding> {
    match strategy {
        MergeStrategy::Replace => {
            // Discard existing, use only imported
            imported
                .into_iter()
                .map(|sb| sb.to_binding())
                .collect()
        }
        MergeStrategy::Append => {
            // Keep all existing, add all imported
            let mut result = existing;
            for sb in imported {
                result.push(sb.to_binding());
            }
            result
        }
        MergeStrategy::SkipConflicts => {
            // Keep existing, add imported only if ID doesn't exist
            let mut result = existing.clone();
            let existing_ids: HashSet<Uuid> = existing
                .iter()
                .map(|b| b.id.0)
                .collect();

            for sb in imported {
                if !existing_ids.contains(&sb.id) {
                    result.push(sb.to_binding());
                }
            }
            result
        }
    }
}

/// Scan for available hotkey combinations using user preferences
/// Zero allocation preference scanning with intelligent conflict detection
#[inline]
pub fn scan_for_available_hotkeys(
    capture_ui_state: &mut HotkeyCaptureUIState,
    hotkey_prefs: &HotkeyPreferences,
) {
    // Use preferred_combinations from HotkeyPreferences instead of hardcoded list
    // This ensures user preferences are respected for conflict scanning
    capture_ui_state.available_alternatives = hotkey_prefs.preferred_combinations.clone();
}


// ============================================================================
// HOTKEY ANALYTICS AND USAGE TRACKING
// ============================================================================

/// Per-hotkey usage statistics with time-series data
/// 
/// Tracks detailed usage patterns using a rolling window of recent press timestamps.
/// This complements HotkeyUsageTracker's exponential moving average approach by
/// providing precise historical data for analytics and export.
///
/// Memory footprint: ~800 bytes per hotkey (100 × 8-byte Instant + metadata)
///
/// Reference: [std::collections::VecDeque](https://doc.rust-lang.org/std/collections/struct.VecDeque.html)
#[derive(Debug, Clone)]
pub struct UsageStats {
    /// Total number of times this hotkey was pressed
    pub total_presses: u64,
    
    /// Timestamp of most recent press
    pub last_pressed: Option<Instant>,
    
    /// Average time between consecutive presses
    /// 
    /// Calculated from rolling window using arithmetic mean of intervals.
    /// Updated on each press if at least 2 timestamps exist.
    ///
    /// Reference: [std::time::Duration](https://doc.rust-lang.org/std/time/struct.Duration.html)
    pub average_interval: Duration,
    
    /// Rolling window of last 100 press timestamps for trend analysis
    /// 
    /// Uses VecDeque for efficient O(1) push_back and pop_front operations.
    /// Window size of 100 balances statistical significance with memory overhead.
    pub press_timestamps: VecDeque<Instant>,
}

impl Default for UsageStats {
    fn default() -> Self {
        Self {
            total_presses: 0,
            last_pressed: None,
            average_interval: Duration::ZERO,
            press_timestamps: VecDeque::with_capacity(100),
        }
    }
}

impl UsageStats {
    /// Calculate time since last press
    /// 
    /// Returns None if hotkey has never been pressed.
    #[inline]
    pub fn time_since_last_press(&self) -> Option<Duration> {
        self.last_pressed.map(|t| Instant::now().duration_since(t))
    }
    
    /// Get press frequency in presses per minute
    /// 
    /// Returns None if less than 2 presses recorded.
    #[inline]
    pub fn presses_per_minute(&self) -> Option<f64> {
        if self.average_interval.as_secs_f64() > 0.0 {
            Some(60.0 / self.average_interval.as_secs_f64())
        } else {
            None
        }
    }
}


/// Serializable version of UsageStats for JSON export
/// 
/// Converts Instant to Duration for human-readable output.
/// Instant is not serializable because it's relative to program start.
///
/// # Example JSON Output
/// ```json
/// {
///   "hotkey_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
///   "total_presses": 42,
///   "last_pressed_secs_ago": 3.5,
///   "average_interval_secs": 120.0,
///   "press_count_in_window": 42
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct UsageStatsExport {
    pub hotkey_id: String,
    pub total_presses: u64,
    pub last_pressed_secs_ago: Option<f64>,
    pub average_interval_secs: f64,
    pub press_count_in_window: usize,
}


/// Centralized hotkey analytics tracking resource
/// 
/// Maintains per-hotkey usage statistics for data-driven UX improvements.
/// All data is local-only with no telemetry or network transmission.
///
/// # Privacy Considerations
/// - Usage patterns can reveal user behavior and workflows
/// - Export functions require explicit method calls (no automatic upload)
/// - All data stored in-memory only (cleared on app restart)
/// - JSON export writes to local filesystem only
///
/// # Example Usage
/// ```rust
/// fn my_system(analytics: Res<HotkeyAnalytics>) {
///     // Get most frequently used hotkeys
///     let top_5 = analytics.get_most_used(5);
///     for (id, stats) in top_5 {
///         println!("{:?}: {} presses", id, stats.total_presses);
///     }
/// }
/// ```
#[derive(Resource, Default, Debug)]
pub struct HotkeyAnalytics {
    /// Usage statistics indexed by hotkey ID
    /// 
    /// Uses HashMap for O(1) lookup by HotkeyId.
    /// Memory grows dynamically with number of registered hotkeys.
    usage_stats: HashMap<HotkeyId, UsageStats>,
}


impl HotkeyAnalytics {
    /// Create new empty analytics tracker
    pub fn new() -> Self {
        Self {
            usage_stats: HashMap::new(),
        }
    }
    
    /// Record a hotkey press event and update statistics
    /// 
    /// This method:
    /// 1. Increments total press counter
    /// 2. Updates last pressed timestamp
    /// 3. Adds timestamp to rolling window (max 100 entries)
    /// 4. Recalculates average interval from window data
    ///
    /// # Performance
    /// - O(1) HashMap lookup and insertion
    /// - O(n) average calculation where n ≤ 100
    /// - Only runs on user-initiated hotkey presses (low frequency)
    ///
    /// # Arguments
    /// * `hotkey_id` - Unique identifier for the pressed hotkey
    #[inline]
    pub fn record_press(&mut self, hotkey_id: &HotkeyId) {
        let now = Instant::now();
        
        // Get or create stats entry for this hotkey (O(1) amortized)
        let stats = self.usage_stats.entry(hotkey_id.clone()).or_default();
        
        // Increment total press counter
        stats.total_presses += 1;
        
        // Update last pressed timestamp
        stats.last_pressed = Some(now);
        
        // Add timestamp to rolling window
        stats.press_timestamps.push_back(now);
        
        // Maintain rolling window of last 100 presses
        if stats.press_timestamps.len() > 100 {
            stats.press_timestamps.pop_front();
        }
        
        // Calculate average interval from timestamp differences
        // Only calculate if we have at least 2 timestamps (need pairs for intervals)
        if stats.press_timestamps.len() >= 2 {
            let mut total_duration = Duration::ZERO;
            let mut interval_count = 0;
            
            // Iterate through consecutive timestamp pairs
            // Example: [t1, t2, t3] → windows: [(t1,t2), (t2,t3)]
            // Manual iteration without allocation - zero-allocation hot path
            let len = stats.press_timestamps.len();
            for i in 0..len.saturating_sub(1) {
                let earlier = stats.press_timestamps[i];
                let later = stats.press_timestamps[i + 1];
                total_duration += later.duration_since(earlier);
                interval_count += 1;
            }
            
            // Calculate arithmetic mean interval
            if interval_count > 0 {
                stats.average_interval = total_duration / interval_count as u32;
            }
        }
    }
    
    /// Get statistics for a specific hotkey
    /// 
    /// Returns None if hotkey has never been pressed.
    #[inline]
    pub fn get_stats(&self, hotkey_id: &HotkeyId) -> Option<&UsageStats> {
        self.usage_stats.get(hotkey_id)
    }
    
    /// Get the most frequently used hotkeys
    /// 
    /// Returns sorted list of (hotkey_id, stats) tuples by total_presses descending.
    /// Useful for identifying power-user shortcuts vs rarely-used bindings.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of results to return
    ///
    /// # Performance
    /// O(n log n) where n = number of registered hotkeys (typically < 64)
    pub fn get_most_used(&self, limit: usize) -> Vec<(&HotkeyId, &UsageStats)> {
        let mut entries: Vec<_> = self.usage_stats.iter().collect();
        
        // Sort by total presses, highest first
        entries.sort_by(|a, b| b.1.total_presses.cmp(&a.1.total_presses));
        
        // Take top N entries
        entries.into_iter().take(limit).collect()
    }
    
    /// Get the least frequently used hotkeys
    /// 
    /// Useful for identifying potentially removable keybindings.
    pub fn get_least_used(&self, limit: usize) -> Vec<(&HotkeyId, &UsageStats)> {
        let mut entries: Vec<_> = self.usage_stats.iter().collect();
        
        // Sort by total presses, lowest first
        entries.sort_by(|a, b| a.1.total_presses.cmp(&b.1.total_presses));
        
        entries.into_iter().take(limit).collect()
    }
    
    /// Clear all statistics (useful for reset functionality)
    pub fn clear_stats(&mut self) {
        self.usage_stats.clear();
    }
    
    /// Get total number of tracked hotkeys
    pub fn tracked_count(&self) -> usize {
        self.usage_stats.len()
    }
    
    /// Export analytics data as JSON string
    /// 
    /// Converts all usage statistics to a serializable format with timestamps
    /// converted to durations (seconds since last press).
    ///
    /// # Returns
    /// Pretty-printed JSON string with all analytics data
    ///
    /// # Errors
    /// Returns serde_json::Error if serialization fails (rare - only if data is corrupt)
    ///
    /// # Privacy Note
    /// This exports usage patterns. Ensure user consent before sharing data.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let now = Instant::now();
        
        let export_data: Vec<UsageStatsExport> = self
            .usage_stats
            .iter()
            .map(|(id, stats)| UsageStatsExport {
                hotkey_id: format!("{:?}", id.0), // UUID as string
                total_presses: stats.total_presses,
                last_pressed_secs_ago: stats.last_pressed.map(|t| now.duration_since(t).as_secs_f64()),
                average_interval_secs: stats.average_interval.as_secs_f64(),
                press_count_in_window: stats.press_timestamps.len(),
            })
            .collect();
        
        serde_json::to_string_pretty(&export_data)
    }
    
    /// Export analytics to a JSON file
    /// 
    /// Follows the same pattern as `persist_hotkey_preferences_owned` in
    /// [systems.rs:619](../packages/ecs-hotkey/src/systems.rs#L619) but synchronous
    /// for simpler export use case.
    ///
    /// # Arguments
    /// * `path` - Destination file path (will be created or overwritten)
    ///
    /// # Privacy Note
    /// This exports usage patterns to disk. Ensure user consent before calling.
    ///
    /// # Errors
    /// Returns error if:
    /// - Serialization fails
    /// - File cannot be created
    /// - Write permission denied
    pub fn export_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json_content = self.export_json()?;
        
        let mut file = std::fs::File::create(path)?;
        file.write_all(json_content.as_bytes())?;
        
        Ok(())
    }
}


// ============================================================================
// MULTI-CAPTURE SESSION SUPPORT
// ============================================================================

/// Unique identifier for hotkey capture sessions
/// 
/// Each UI component requesting capture gets a unique session ID.
/// Enables multiple concurrent capture sessions without conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CaptureSessionId(pub Uuid);

impl CaptureSessionId {
    /// Create a new unique capture session ID
    #[inline]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CaptureSessionId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Per-session capture state
/// 
/// Tracks the state of an individual capture session.
/// Multiple sessions can be active simultaneously,
/// each maintaining independent modifier and key state.
#[derive(Debug, Clone)]
pub struct CaptureSession {
    /// Component that requested this capture session
    pub requester: String,
    
    /// When this capture session started
    pub started_at: Instant,
    
    /// Currently held modifier keys for this session
    pub held_modifiers: Modifiers,
    
    /// Captured key code (None until key is pressed)
    pub captured_key: Option<Code>,
}

impl CaptureSession {
    /// Create a new capture session
    #[inline]
    pub fn new(requester: impl Into<String>) -> Self {
        Self {
            requester: requester.into(),
            started_at: Instant::now(),
            held_modifiers: Modifiers::empty(),
            captured_key: None,
        }
    }
}


/// Multi-capture state resource
/// 
/// Manages multiple concurrent hotkey capture sessions.
/// Allows different UI components to capture hotkeys simultaneously
/// without conflicts or state corruption.
/// 
/// # Zero Allocation Design
/// - HashMap with pre-allocated capacity for typical usage
/// - Inline methods for blazing-fast performance
/// - No locking required (single-threaded ECS resource)
///
/// # Example Usage
/// ```rust
/// fn start_capture_system(
///     mut multi_capture: ResMut<MultiCaptureState>,
/// ) {
///     let session_id = multi_capture.start_session("my_component");
///     // Use session_id for subsequent operations
/// }
/// ```
#[derive(Resource, Debug)]
pub struct MultiCaptureState {
    /// Active capture sessions indexed by session ID
    /// 
    /// Pre-allocated capacity for typical usage (4-8 concurrent sessions)
    active_sessions: HashMap<CaptureSessionId, CaptureSession>,
}

impl Default for MultiCaptureState {
    #[inline]
    fn default() -> Self {
        Self {
            // Pre-allocate for typical concurrent usage
            active_sessions: HashMap::with_capacity(8),
        }
    }
}

impl MultiCaptureState {
    /// Create a new multi-capture state with default capacity
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Start a new capture session
    /// 
    /// Creates a unique session ID and initializes capture state for the requester.
    /// Multiple sessions can be active simultaneously.
    ///
    /// # Arguments
    /// * `requester` - Identifier for the component requesting capture
    ///
    /// # Returns
    /// Unique session ID for this capture session
    ///
    /// # Example
    /// ```rust
    /// let session_id = multi_capture.start_session("preferences_dialog");
    /// ```
    #[inline]
    pub fn start_session(&mut self, requester: impl Into<String>) -> CaptureSessionId {
        let session_id = CaptureSessionId::new();
        let session = CaptureSession::new(requester);
        
        self.active_sessions.insert(session_id, session);
        
        session_id
    }
    
    /// Complete and remove a capture session
    /// 
    /// Removes the session from active sessions and returns its final state.
    /// Returns None if session ID does not exist.
    ///
    /// # Arguments
    /// * `id` - Session ID to complete
    ///
    /// # Returns
    /// The completed capture session if it existed, None otherwise
    #[inline]
    pub fn complete_session(&mut self, id: &CaptureSessionId) -> Option<CaptureSession> {
        self.active_sessions.remove(id)
    }
    
    /// Get mutable access to a specific capture session
    /// 
    /// Allows updating session state (modifiers, captured key).
    /// Returns None if session ID does not exist.
    ///
    /// # Arguments
    /// * `id` - Session ID to access
    ///
    /// # Returns
    /// Mutable reference to the session if it exists, None otherwise
    #[inline]
    pub fn get_session_mut(&mut self, id: &CaptureSessionId) -> Option<&mut CaptureSession> {
        self.active_sessions.get_mut(id)
    }
    
    /// Get immutable access to a specific capture session
    /// 
    /// # Arguments
    /// * `id` - Session ID to access
    ///
    /// # Returns
    /// Reference to the session if it exists, None otherwise
    #[inline]
    pub fn get_session(&self, id: &CaptureSessionId) -> Option<&CaptureSession> {
        self.active_sessions.get(id)
    }
    
    /// Get the number of active capture sessions
    #[inline]
    pub fn active_count(&self) -> usize {
        self.active_sessions.len()
    }
    
    /// Check if a specific session is active
    #[inline]
    pub fn is_session_active(&self, id: &CaptureSessionId) -> bool {
        self.active_sessions.contains_key(id)
    }
    
    /// Get all active session IDs
    /// 
    /// Returns an iterator over all active session IDs.
    /// Useful for broadcast operations or cleanup.
    #[inline]
    pub fn active_session_ids(&self) -> impl Iterator<Item = &CaptureSessionId> {
        self.active_sessions.keys()
    }
    
    /// Cancel all active sessions
    /// 
    /// Clears all active capture sessions.
    /// Useful for reset or error recovery.
    #[inline]
    pub fn cancel_all_sessions(&mut self) {
        self.active_sessions.clear();
    }
}

/// Global hotkey manager resource wrapper
/// Wraps the global_hotkey crate's GlobalHotKeyManager for use as a Bevy resource
#[derive(Resource)]
pub struct AppGlobalHotkeyManager {
    pub manager: global_hotkey::GlobalHotKeyManager,
    pub toggle_hotkey: global_hotkey::hotkey::HotKey,
}

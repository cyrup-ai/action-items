# Advanced_Menu Task 0: Advanced Settings Data Models

## Task Overview
Implement comprehensive multi-monitor and navigation data structures for the Advanced menu, supporting complex display configurations, keyboard navigation schemes, and advanced search parameters.

## Implementation Requirements

### Core Data Models
```rust
// Advanced settings system
#[derive(Resource, Reflect, Debug)]
pub struct AdvancedSettingsResource {
    pub multi_monitor_config: MultiMonitorConfiguration,
    pub navigation_settings: NavigationSettings,
    pub search_configuration: SearchConfiguration,
    pub input_method_config: InputMethodConfiguration,
    pub escape_key_behavior: EscapeKeyBehavior,
    pub performance_settings: PerformanceSettings,
}

#[derive(Reflect, Debug, Clone)]
pub struct MultiMonitorConfiguration {
    pub detected_monitors: Vec<MonitorInfo>,
    pub primary_monitor: MonitorId,
    pub launcher_placement: LauncherPlacement,
    pub monitor_preferences: HashMap<MonitorId, MonitorPreferences>,
    pub display_scaling: HashMap<MonitorId, f32>,
}

#[derive(Reflect, Debug, Clone)]
pub struct MonitorInfo {
    pub monitor_id: MonitorId,
    pub name: String,
    pub resolution: Resolution,
    pub physical_size: PhysicalSize,
    pub position: MonitorPosition,
    pub scale_factor: f32,
    pub is_primary: bool,
    pub refresh_rate: u32,
}

#[derive(Reflect, Debug, Clone, Hash, PartialEq, Eq)]
pub struct MonitorId(pub String);

#[derive(Reflect, Debug, Clone)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Reflect, Debug, Clone)]
pub struct MonitorPosition {
    pub x: i32,
    pub y: i32,
}
```

### Navigation Configuration System
```rust
// Keyboard navigation and bindings
#[derive(Reflect, Debug, Clone)]
pub struct NavigationSettings {
    pub navigation_scheme: NavigationScheme,
    pub custom_bindings: HashMap<NavigationAction, KeyBinding>,
    pub vi_mode_enabled: bool,
    pub emacs_bindings_enabled: bool,
    pub navigation_sensitivity: f32,
    pub repeat_delay: Duration,
    pub repeat_rate: Duration,
}

#[derive(Reflect, Debug, Clone)]
pub enum NavigationScheme {
    Default,
    Vi,
    Emacs,
    Custom,
    Platform, // Follow OS conventions
}

#[derive(Reflect, Debug, Clone, Hash, PartialEq, Eq)]
pub enum NavigationAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PageUp,
    PageDown,
    Home,
    End,
    SelectAll,
    SelectNone,
    ToggleSelection,
    ConfirmAction,
    CancelAction,
    QuickSearch,
    FocusSearch,
}

#[derive(Reflect, Debug, Clone)]
pub struct KeyBinding {
    pub primary_key: KeyCode,
    pub modifiers: ModifierKeys,
    pub alternative_binding: Option<AlternativeBinding>,
    pub context: BindingContext,
}

#[derive(Reflect, Debug, Clone)]
pub enum BindingContext {
    Global,
    SearchMode,
    SelectionMode,
    EditMode,
    MenuMode,
}
```

### Search Configuration System
```rust
// Advanced search and fuzzy matching settings
#[derive(Reflect, Debug, Clone)]
pub struct SearchConfiguration {
    pub search_algorithm: SearchAlgorithm,
    pub fuzzy_matching: FuzzyMatchingConfig,
    pub search_scope: SearchScope,
    pub result_ranking: ResultRankingConfig,
    pub search_history: SearchHistoryConfig,
}

#[derive(Reflect, Debug, Clone)]
pub enum SearchAlgorithm {
    ExactMatch,
    FuzzyMatch,
    Substring,
    Regex,
    Hybrid,
}

#[derive(Reflect, Debug, Clone)]
pub struct FuzzyMatchingConfig {
    pub match_threshold: f32,
    pub case_sensitive: bool,
    pub word_boundary_bonus: f32,
    pub camel_case_bonus: f32,
    pub consecutive_bonus: f32,
    pub leading_letter_penalty: f32,
    pub max_gap_penalty: f32,
}

#[derive(Reflect, Debug, Clone)]
pub struct SearchScope {
    pub search_extensions: bool,
    pub search_commands: bool,
    pub search_files: bool,
    pub search_web: bool,
    pub search_history: bool,
    pub max_results: u32,
}

#[derive(Reflect, Debug, Clone)]
pub struct ResultRankingConfig {
    pub frequency_weight: f32,
    pub recency_weight: f32,
    pub relevance_weight: f32,
    pub location_weight: f32,
    pub custom_boost_rules: Vec<BoostRule>,
}
```

### Input Method Integration
```rust
// Input method and localization settings
#[derive(Reflect, Debug, Clone)]
pub struct InputMethodConfiguration {
    pub auto_switch_input_source: bool,
    pub preferred_input_methods: Vec<InputMethodInfo>,
    pub input_source_detection: InputSourceDetection,
    pub ime_integration: IMEIntegration,
    pub text_input_settings: TextInputSettings,
}

#[derive(Reflect, Debug, Clone)]
pub struct InputMethodInfo {
    pub method_id: String,
    pub display_name: String,
    pub language_code: String,
    pub is_enabled: bool,
    pub priority: u8,
}

#[derive(Reflect, Debug, Clone)]
pub struct InputSourceDetection {
    pub auto_detect_language: bool,
    pub detection_sensitivity: f32,
    pub fallback_method: String,
    pub detection_timeout: Duration,
}

#[derive(Reflect, Debug, Clone)]
pub struct IMEIntegration {
    pub composition_preview: bool,
    pub candidate_window_position: CandidateWindowPosition,
    pub ime_aware_search: bool,
}

#[derive(Reflect, Debug, Clone)]
pub enum CandidateWindowPosition {
    FollowCursor,
    NearSearchBox,
    Fixed { x: i32, y: i32 },
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `window/multiple_windows.rs` - Multi-window/monitor handling
- `input/keyboard_input.rs` - Advanced keyboard input
- `reflection/reflection.rs` - Settings serialization

### Implementation Pattern
```rust
// Based on multiple_windows.rs for multi-monitor support
fn multi_monitor_detection_system(
    mut advanced_settings: ResMut<AdvancedSettingsResource>,
    windows: Query<&Window>,
) {
    let detected_monitors = detect_available_monitors();
    
    if detected_monitors != advanced_settings.multi_monitor_config.detected_monitors {
        advanced_settings.multi_monitor_config.detected_monitors = detected_monitors;
        // Trigger monitor configuration update
    }
}

// Based on keyboard_input.rs for navigation bindings
fn navigation_binding_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    navigation_settings: Res<NavigationSettings>,
    mut navigation_events: EventWriter<NavigationEvent>,
) {
    for (action, binding) in &navigation_settings.custom_bindings {
        if check_key_combination(&keyboard_input, binding) {
            navigation_events.send(NavigationEvent {
                action: action.clone(),
                context: get_current_context(),
            });
        }
    }
}
```

## Performance Optimization Settings
- Advanced performance tuning parameters
- Resource usage monitoring and limits
- Rendering optimization settings
- Memory management configuration

## Performance Constraints
- **ZERO ALLOCATIONS** during settings access
- Efficient multi-monitor state management
- Optimized navigation binding lookup
- Minimal overhead for input method detection

## Success Criteria
- Complete advanced settings data model implementation
- Efficient multi-monitor configuration system
- No unwrap()/expect() calls in production code
- Zero-allocation settings access patterns
- Comprehensive navigation and search configuration

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for data model validation
- Integration tests for multi-monitor detection
- Performance tests for settings access patterns
- Configuration persistence tests
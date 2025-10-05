# AI Menu 3 - Experimental Features Toggle System

## Implementation Task: Dynamic Feature Flag Management with Visual Toggle Interface

### Architecture Overview
Implement comprehensive experimental features system with visual toggle controls, feature flag management, rollout configuration, and telemetry collection for the 5 specified experimental features.

### Core Components

#### Experimental Features Manager
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ExperimentalFeaturesManager {
    pub feature_toggles: HashMap<String, FeatureToggle>,
    pub toggle_components: HashMap<String, Entity>,
    pub rollout_controller: RolloutController,
    pub telemetry_collector: FeatureTelemetry,
    pub user_preferences: ExperimentalPreferences,
}

#[derive(Reflect, Clone)]
pub struct FeatureToggle {
    pub feature_id: String,
    pub display_name: String,
    pub description: String,
    pub enabled: bool,
    pub toggle_style: ToggleStyle,
    pub info_button: Option<Entity>,
    pub rollout_config: RolloutConfig,
    pub telemetry_enabled: bool,
}

// Specific experimental features from specification
#[derive(Reflect)]
pub struct SpecificExperimentalFeatures {
    pub auto_models: FeatureToggle,         // ON (blue toggle)
    pub chat_branching: FeatureToggle,      // ON (blue toggle) 
    pub custom_providers: FeatureToggle,    // OFF (gray toggle)
    pub mcp_http_servers: FeatureToggle,    // ON (blue toggle)
    pub ai_extensions_ollama: FeatureToggle, // ON (blue toggle)
}
```

#### Visual Toggle System
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ToggleSwitch {
    pub feature_id: String,
    pub current_state: bool,
    pub visual_state: ToggleVisualState,
    pub animation_progress: f32,        // 0.0 to 1.0 for smooth transitions
    pub interaction_state: InteractionState,
    pub accessibility_label: String,
}

#[derive(Reflect)]
pub struct ToggleVisualState {
    pub background_color: Color,        // Blue (#007AFF) or Gray (#8E8E93)
    pub toggle_position: f32,           // 0.0 (left) to 1.0 (right)
    pub animation_duration: Duration,   // Smooth slide transition
    pub hover_state: HoverState,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ToggleGroup {
    pub group_title: String,            // "Experiments"
    pub group_description: String,      // "New AI features in development..."
    pub toggle_entities: Vec<Entity>,
    pub layout: GroupLayout,
}

#[derive(Reflect)]
pub struct GroupLayout {
    pub spacing: f32,
    pub alignment: Alignment,
    pub two_column_layout: bool,        // Label left, toggle right, info far right
}
```

### Bevy Implementation References

#### Toggle Switch UI Components
- **Interactive UI**: `docs/bevy/examples/ui/button.rs`
  - Toggle switch interactions and state management
  - Smooth animation between on/off states
  - Hover and click state handling

#### Animation System
- **UI Animations**: `docs/bevy/examples/animation/animated_fox.rs`
  - Smooth slide animations for toggle transitions
  - Color transitions between blue and gray states
  - Easing functions for natural animation feel

#### Layout Management
- **UI Layout**: `docs/bevy/examples/ui/flex_layout.rs`
  - Two-column layout with labels, toggles, and info buttons
  - Consistent spacing and alignment across toggle group
  - Responsive layout for different screen sizes

#### State Management
- **State Systems**: `docs/bevy/examples/state/states.rs`
  - Feature flag state persistence and synchronization
  - State-dependent behavior activation/deactivation
  - Cross-system state coordination

### Feature Flag Implementation

#### Feature Flag Data Structure
```rust
#[derive(Reflect, Clone)]
pub struct FeatureConfig {
    pub feature_id: String,
    pub name: String,
    pub description: String,
    pub default_enabled: bool,
    pub rollout_percentage: f32,        // 0.0 to 100.0
    pub prerequisites: Vec<String>,     // Dependent feature IDs
    pub risk_level: FeatureRiskLevel,
    pub experimental_phase: ExperimentalPhase,
}

#[derive(Reflect)]
pub enum FeatureRiskLevel {
    Low,        // Cosmetic or minor features
    Medium,     // Features that may affect performance
    High,       // Features that may affect stability
    Critical,   // Features that may affect data integrity
}

#[derive(Reflect)]
pub enum ExperimentalPhase {
    Development,     // Early development, internal only
    Alpha,           // Limited alpha testing
    Beta,            // Public beta testing
    ReleaseCandidate, // Nearly ready for production
    Deprecated,      // Being phased out
}
```

#### Specific Feature Definitions
Based on specification requirements:

```rust
impl Default for SpecificExperimentalFeatures {
    fn default() -> Self {
        Self {
            auto_models: FeatureToggle {
                feature_id: "auto_models".to_string(),
                display_name: "Auto Models".to_string(),
                description: "Automatically select optimal models for tasks".to_string(),
                enabled: true,  // ON in specification
                toggle_style: ToggleStyle::active(),
                info_button: Some(/* Entity for info button */),
                rollout_config: RolloutConfig::full_rollout(),
                telemetry_enabled: true,
            },
            chat_branching: FeatureToggle {
                feature_id: "chat_branching".to_string(),
                display_name: "Chat Branching".to_string(),
                description: "Branch conversations for exploring different paths".to_string(),
                enabled: true,  // ON in specification
                toggle_style: ToggleStyle::active(),
                info_button: Some(/* Entity */),
                rollout_config: RolloutConfig::full_rollout(),
                telemetry_enabled: true,
            },
            custom_providers: FeatureToggle {
                feature_id: "custom_providers".to_string(),
                display_name: "Custom Providers".to_string(), 
                description: "Add custom AI providers beyond built-in options".to_string(),
                enabled: false, // OFF in specification
                toggle_style: ToggleStyle::inactive(),
                info_button: Some(/* Entity */),
                rollout_config: RolloutConfig::limited_rollout(25.0),
                telemetry_enabled: true,
            },
            mcp_http_servers: FeatureToggle {
                feature_id: "mcp_http_servers".to_string(),
                display_name: "MCP HTTP Servers".to_string(),
                description: "Support for Model Context Protocol over HTTP".to_string(),
                enabled: true,  // ON in specification
                toggle_style: ToggleStyle::active(),
                info_button: Some(/* Entity */),
                rollout_config: RolloutConfig::full_rollout(),
                telemetry_enabled: true,
            },
            ai_extensions_ollama: FeatureToggle {
                feature_id: "ai_extensions_ollama".to_string(),
                display_name: "AI Extensions for Ollama Models".to_string(),
                description: "Enhanced AI capabilities for locally hosted Ollama models".to_string(),
                enabled: true,  // ON in specification
                toggle_style: ToggleStyle::active(),
                info_button: Some(/* Entity */),
                rollout_config: RolloutConfig::full_rollout(), 
                telemetry_enabled: true,
            },
        }
    }
}
```

### Visual Toggle Design System

#### Toggle Style Implementation
```rust
#[derive(Reflect, Clone)]
pub struct ToggleStyle {
    pub size: Vec2,                     // Toggle dimensions
    pub active_color: Color,            // Blue (#007AFF) for ON
    pub inactive_color: Color,          // Gray (#8E8E93) for OFF
    pub toggle_circle_color: Color,     // White circle
    pub border_radius: f32,             // Rounded rectangle background
    pub animation_curve: AnimationCurve,
}

impl ToggleStyle {
    pub fn active() -> Self {
        Self {
            size: Vec2::new(51.0, 31.0),  // iOS-style toggle size
            active_color: Color::rgb(0.0, 0.48, 1.0),    // #007AFF
            inactive_color: Color::rgb(0.557, 0.557, 0.576), // #8E8E93
            toggle_circle_color: Color::WHITE,
            border_radius: 15.5,          // Half height for perfect circle
            animation_curve: AnimationCurve::ease_in_out(),
        }
    }
    
    pub fn inactive() -> Self {
        let mut style = Self::active();
        style.active_color = style.inactive_color;
        style
    }
}
```

#### Toggle Animation System
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ToggleAnimation {
    pub start_time: SystemTime,
    pub duration: Duration,             // 0.2 seconds for smooth transition
    pub start_state: bool,
    pub target_state: bool,
    pub current_progress: f32,          // 0.0 to 1.0
    pub easing_function: EasingFunction,
}

#[derive(Reflect)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}
```

### Feature Flag Management

#### Rollout Control System
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RolloutController {
    pub rollout_policies: HashMap<String, RolloutPolicy>,
    pub user_segments: UserSegmentation,
    pub a_b_testing: ABTestingConfig,
    pub gradual_rollout: GradualRolloutManager,
}

#[derive(Reflect)]
pub struct RolloutPolicy {
    pub feature_id: String,
    pub rollout_percentage: f32,        // 0.0 to 100.0
    pub user_criteria: Vec<UserCriteria>,
    pub geographic_restrictions: Vec<String>,
    pub device_restrictions: Vec<DeviceType>,
    pub rollback_triggers: Vec<RollbackTrigger>,
}

#[derive(Reflect)]
pub enum UserCriteria {
    BetaUser,
    PremiumUser,
    DeveloperUser,
    EarlyAdopter,
    GeographicRegion(String),
    AppVersion(String),
    Random(f32),                        // Random percentage
}
```

#### Feature Dependency Management
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DependencyManager {
    pub feature_dependencies: HashMap<String, Vec<String>>,
    pub circular_dependency_detection: bool,
    pub automatic_dependency_resolution: bool,
    pub dependency_validation: DependencyValidation,
}

impl DependencyManager {
    pub fn validate_toggle_change(&self, feature_id: &str, new_state: bool) -> Result<(), DependencyError> {
        // Validate that toggling this feature won't break dependencies
        if !new_state {
            // Check if any enabled features depend on this one
            for (other_feature, deps) in &self.feature_dependencies {
                if deps.contains(&feature_id.to_string()) {
                    return Err(DependencyError::RequiredByOtherFeatures(vec![other_feature.clone()]));
                }
            }
        }
        Ok(())
    }
}
```

### Telemetry and Analytics

#### Feature Usage Tracking
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct FeatureTelemetry {
    pub usage_metrics: HashMap<String, FeatureUsageMetrics>,
    pub interaction_events: VecDeque<InteractionEvent>,
    pub performance_metrics: HashMap<String, PerformanceMetrics>,
    pub error_tracking: HashMap<String, ErrorMetrics>,
}

#[derive(Reflect)]
pub struct FeatureUsageMetrics {
    pub feature_id: String,
    pub toggle_count: u64,
    pub time_enabled: Duration,
    pub time_disabled: Duration,
    pub user_satisfaction_score: Option<f32>,
    pub crash_correlation: CrashCorrelation,
}

#[derive(Reflect)]
pub struct InteractionEvent {
    pub timestamp: SystemTime,
    pub feature_id: String,
    pub event_type: InteractionEventType,
    pub user_context: UserContext,
}

#[derive(Reflect)]
pub enum InteractionEventType {
    ToggleEnabled,
    ToggleDisabled,
    InfoButtonClicked,
    FeatureUsed,
    ErrorEncountered,
    FeedbackProvided,
}
```

### User Interface Integration

#### Info Button System
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct InfoButton {
    pub feature_id: String,
    pub button_entity: Entity,
    pub tooltip_content: String,
    pub expanded_info: Option<ExpandedInfo>,
    pub interaction_state: ButtonInteractionState,
}

#[derive(Reflect)]
pub struct ExpandedInfo {
    pub detailed_description: String,
    pub benefits: Vec<String>,
    pub risks: Vec<String>,
    pub prerequisites: Vec<String>,
    pub documentation_links: Vec<String>,
}
```

#### Layout and Spacing
- **Two-Column Layout**: Feature name left, toggle center-right, info icon far right
- **Consistent Spacing**: Uniform vertical spacing between toggle items
- **Visual Hierarchy**: Clear separation between toggle groups and individual items
- **Responsive Design**: Adapt to different screen sizes and orientations

### Error Handling and Safety

#### Feature Safety System
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FeatureSafety {
    pub safety_checks: HashMap<String, Vec<SafetyCheck>>,
    pub automatic_rollback: AutomaticRollback,
    pub user_consent: ConsentManagement,
    pub impact_monitoring: ImpactMonitoring,
}

#[derive(Reflect)]
pub enum SafetyCheck {
    PerformanceImpact(f32),            // Max acceptable performance degradation
    StabilityCheck,                     // Crash rate monitoring
    DataIntegrityCheck,                 // Data corruption prevention
    SecurityValidation,                 // Security implications
    UserExperienceMetrics,              // UX impact measurement
}

#[derive(Event)]
pub struct FeatureSafetyViolation {
    pub feature_id: String,
    pub violation_type: SafetyViolationType,
    pub severity: ViolationSeverity,
    pub automatic_action: Option<SafetyAction>,
}
```

### Performance Optimization

#### Efficient Toggle Rendering
- **Minimal Re-renders**: Only re-render toggles that change state
- **Animation Optimization**: Smooth 60fps animations for toggle transitions
- **Memory Management**: Efficient storage of toggle state and visual data
- **Event Batching**: Batch toggle events for better performance

#### Feature Flag Evaluation
- **Fast Lookups**: Use HashMap for O(1) feature flag lookups
- **Caching**: Cache feature flag evaluation results
- **Lazy Loading**: Load feature configurations only when needed
- **Background Processing**: Update rollout configurations in background

### Integration Points

#### System Feature Integration
- **AI System**: Coordinate experimental AI features with core AI functionality
- **Extension System**: Enable/disable experimental extension capabilities
- **UI System**: Control experimental UI features and behaviors
- **Performance System**: Monitor performance impact of experimental features

#### Settings Persistence
- **User Preferences**: Save user toggle preferences across sessions
- **Cloud Sync**: Optionally sync experimental feature preferences
- **Migration**: Handle feature graduation from experimental to stable
- **Reset Options**: Allow users to reset experimental features to defaults

### Implementation Files
- `ai_menu_3/experimental_features.rs` - Core experimental features management
- `ai_menu_3/toggle_components.rs` - Visual toggle switch components and animations
- `ai_menu_3/feature_flags.rs` - Feature flag system and rollout control
- `ai_menu_3/feature_telemetry.rs` - Usage tracking and analytics
- `ai_menu_3/feature_safety.rs` - Safety monitoring and automatic rollback
- `ai_menu_3/feature_ui.rs` - UI integration and layout management

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all toggle animation loops
- **Blazing-fast performance** - 60fps toggle animations, instant state updates
- **Production quality** - robust feature flag system with comprehensive safety measures
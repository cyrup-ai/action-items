//! Launcher service components
//!
//! Component definitions for tracking launcher operations and state.

use std::time::Instant;

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::Task;
use serde_json::Value;

use crate::events::*;

/// Component to track action execution operations
#[derive(Component)]
pub struct ActionExecution {
    pub action_id: String,
    pub requester: String,
    pub parameters: Value,
    pub status: ExecutionStatus,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub result: Option<Value>,
    pub error_message: Option<String>,
}

/// Execution status enumeration
#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Component to track search operations
#[derive(Component)]
pub struct SearchOperation {
    pub query: String,
    pub requester: String,
    pub search_type: SearchType,
    pub status: SearchStatus,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub result_count: usize,
}

/// Search status enumeration
#[derive(Debug, Clone)]
pub enum SearchStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Component for async search tasks
#[derive(Component)]
pub struct SearchTask {
    pub task: Task<CommandQueue>,
    pub query: String,
    pub started_at: Instant,
}

/// Component for async action execution tasks
#[derive(Component)]
pub struct ActionExecutionTask {
    pub task: Task<CommandQueue>,
    pub action_id: String,
    pub started_at: Instant,
}

/// Component to track UI interactions
#[derive(Component)]
pub struct UIInteraction {
    pub interaction_type: InteractionType,
    pub target: String,
    pub timestamp: Instant,
    pub requester: String,
}

/// UI interaction type
#[derive(Debug, Clone)]
pub enum InteractionType {
    Click,
    Hover,
    KeyPress,
    Scroll,
    Focus,
    Blur,
}

/// Component for window management
#[derive(Component)]
#[derive(Default)]
pub struct LauncherWindow {
    pub is_visible: bool,
    pub is_focused: bool,
    pub last_shown: Option<Instant>,
    pub show_count: u64,
}


/// Component for plugin integration
#[derive(Component)]
pub struct PluginIntegration {
    pub plugin_name: String,
    pub capabilities: Vec<String>,
    pub last_activity: Option<Instant>,
    pub status: crate::resources::PluginStatus,
}

/// Component for result scoring and ranking
#[derive(Component)]
pub struct ResultRanking {
    pub base_score: f32,
    pub context_score: f32,
    pub usage_score: f32,
    pub final_score: f32,
    pub rank_position: usize,
}

impl ResultRanking {
    pub fn calculate_final_score(&mut self) {
        self.final_score =
            (self.base_score * 0.4) + (self.context_score * 0.3) + (self.usage_score * 0.3);
    }
}

/// Component for caching search results
#[derive(Component)]
pub struct SearchResultCache {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub cached_at: Instant,
    pub ttl: std::time::Duration,
    pub hit_count: u32,
}

impl SearchResultCache {
    pub fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    pub fn record_hit(&mut self) {
        self.hit_count += 1;
    }
}

/// Component for tracking user preferences
#[derive(Component)]
pub struct UserPreference {
    pub preference_key: String,
    pub preference_value: Value,
    pub updated_at: Instant,
    pub requester: String,
}

/// Component for analytics and metrics collection
#[derive(Component)]
pub struct MetricsCollector {
    pub metric_type: MetricType,
    pub value: f64,
    pub timestamp: Instant,
    pub tags: std::collections::HashMap<String, String>,
}

/// Metric type enumeration
#[derive(Debug, Clone)]
pub enum MetricType {
    SearchLatency,
    ActionExecutionTime,
    ResultClickRate,
    PluginResponseTime,
    UIInteractionCount,
    ErrorRate,
}

/// Component for keyboard shortcut handling
#[derive(Component)]
pub struct KeyboardShortcut {
    pub key_combination: String,
    pub action: String,
    pub enabled: bool,
    pub last_triggered: Option<Instant>,
    pub trigger_count: u32,
}

/// Component for theme management
#[derive(Component)]
pub struct ThemeComponent {
    pub theme_name: String,
    pub applied_at: Instant,
    pub custom_overrides: std::collections::HashMap<String, Value>,
}

/// Marker component for the main launcher entity
#[derive(Component)]
pub struct MainLauncher;

/// Marker component for search result entities
#[derive(Component)]
pub struct SearchResultEntity {
    pub result_id: String,
    pub position: usize,
}

/// Component for animation state with comprehensive easing support
#[derive(Component)]
pub struct AnimationState {
    pub animation_type: AnimationType,
    pub progress: f32,
    pub duration: std::time::Duration,
    pub started_at: Instant,
    pub start_value: f32,
    pub target_value: f32,
    pub current_value: f32,
    pub easing_type: EasingType,
}

/// Animation type enumeration
#[derive(Debug, Clone)]
pub enum AnimationType {
    FadeIn,
    FadeOut,
    SlideUp,
    SlideDown,
    Scale,
    Rotate,
}

impl AnimationState {
    pub fn is_complete(&self) -> bool {
        self.started_at.elapsed() >= self.duration
    }

    pub fn update_progress(&mut self) {
        let elapsed = self.started_at.elapsed();
        self.progress = (elapsed.as_millis() as f32 / self.duration.as_millis() as f32).min(1.0);

        // Production-quality easing functions for professional animations
        let eased_progress = apply_easing_function(self.progress, self.easing_type);
        self.current_value =
            self.start_value + (self.target_value - self.start_value) * eased_progress;
    }
}

/// Comprehensive easing function types for production-quality animations
#[derive(Debug, Clone, Copy)]
pub enum EasingType {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

/// Apply sophisticated easing functions for professional animation quality
/// Zero-allocation implementation using mathematical formulas
pub fn apply_easing_function(t: f32, easing_type: EasingType) -> f32 {
    let t = t.clamp(0.0, 1.0);

    match easing_type {
        EasingType::Linear => t,

        // Quadratic
        EasingType::EaseInQuad => t * t,
        EasingType::EaseOutQuad => 1.0 - (1.0 - t).powi(2),
        EasingType::EaseInOutQuad => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        },

        // Cubic
        EasingType::EaseInCubic => t.powi(3),
        EasingType::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
        EasingType::EaseInOutCubic => {
            if t < 0.5 {
                4.0 * t.powi(3)
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        },

        // Quartic
        EasingType::EaseInQuart => t.powi(4),
        EasingType::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
        EasingType::EaseInOutQuart => {
            if t < 0.5 {
                8.0 * t.powi(4)
            } else {
                1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
            }
        },

        // Sine
        EasingType::EaseInSine => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
        EasingType::EaseOutSine => (t * std::f32::consts::FRAC_PI_2).sin(),
        EasingType::EaseInOutSine => -((t * std::f32::consts::PI).cos() - 1.0) / 2.0,

        // Exponential
        EasingType::EaseInExpo => {
            if t == 0.0 {
                0.0
            } else {
                2.0_f32.powf(10.0 * (t - 1.0))
            }
        },
        EasingType::EaseOutExpo => {
            if t == 1.0 {
                1.0
            } else {
                1.0 - 2.0_f32.powf(-10.0 * t)
            }
        },
        EasingType::EaseInOutExpo => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                2.0_f32.powf(20.0 * t - 10.0) / 2.0
            } else {
                (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
            }
        },

        // Circular
        EasingType::EaseInCirc => 1.0 - (1.0 - t.powi(2)).sqrt(),
        EasingType::EaseOutCirc => (1.0 - (t - 1.0).powi(2)).sqrt(),
        EasingType::EaseInOutCirc => {
            if t < 0.5 {
                (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
            } else {
                ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
            }
        },

        // Back (overshoot)
        EasingType::EaseInBack => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            c3 * t.powi(3) - c1 * t.powi(2)
        },
        EasingType::EaseOutBack => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
        },
        EasingType::EaseInOutBack => {
            let c1 = 1.70158;
            let c2 = c1 * 1.525;
            if t < 0.5 {
                ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
            } else {
                ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
            }
        },

        // Elastic
        EasingType::EaseInElastic => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                -2.0_f32.powf(10.0 * t - 10.0)
                    * ((t * 10.0 - 10.75) * 2.0 * std::f32::consts::PI / 3.0).sin()
            }
        },
        EasingType::EaseOutElastic => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else {
                2.0_f32.powf(-10.0 * t)
                    * ((t * 10.0 - 0.75) * 2.0 * std::f32::consts::PI / 3.0).sin()
                    + 1.0
            }
        },
        EasingType::EaseInOutElastic => {
            if t == 0.0 {
                0.0
            } else if t == 1.0 {
                1.0
            } else if t < 0.5 {
                -(2.0_f32.powf(20.0 * t - 10.0)
                    * ((20.0 * t - 11.125) * 2.0 * std::f32::consts::PI / 4.5).sin())
                    / 2.0
            } else {
                (2.0_f32.powf(-20.0 * t + 10.0)
                    * ((20.0 * t - 11.125) * 2.0 * std::f32::consts::PI / 4.5).sin())
                    / 2.0
                    + 1.0
            }
        },

        // Bounce
        EasingType::EaseInBounce => 1.0 - bounce_out(1.0 - t),
        EasingType::EaseOutBounce => bounce_out(t),
        EasingType::EaseInOutBounce => {
            if t < 0.5 {
                (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
            } else {
                (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
            }
        },
    }
}

/// Helper function for bounce easing calculations
fn bounce_out(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;

    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t_adj = t - 1.5 / d1;
        n1 * t_adj * t_adj + 0.75
    } else if t < 2.5 / d1 {
        let t_adj = t - 2.25 / d1;
        n1 * t_adj * t_adj + 0.9375
    } else {
        let t_adj = t - 2.625 / d1;
        n1 * t_adj * t_adj + 0.984375
    }
}

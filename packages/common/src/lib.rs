//! Common types and traits shared across the action-items workspace

pub mod directories;
pub mod metrics;
pub mod plugin_interface;

// Re-export commonly used types
// Re-export directories types
pub use directories::AppDirectories;
// Re-export metrics types
pub use metrics::{
    BatchCollectionResult, CollectionResult, DashboardData, HistoricalDataPoint, MetricCollector,
    MetricsConfig, MetricsError, MetricsSystem, SystemSnapshot, ThresholdType, ViolationDetector,
    ViolationSeverity, ViolationThreshold,
};
pub use plugin_interface::action_item::{
    ActionItem, ActionType, Icon, ItemAction, ItemBadge, Shortcut,
};
pub use plugin_interface::manifest::{PluginCategory, PluginManifest};

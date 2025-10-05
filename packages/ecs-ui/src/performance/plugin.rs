//! Performance monitoring plugin

use bevy::prelude::*;
use super::resources::{PerformanceMetrics, PerformanceManager};
use super::events::PerformanceWarning;
use super::systems::*;

/// Plugin that registers performance monitoring systems
///
/// Provides zero-allocation frame time tracking, entity virtualization,
/// and performance warnings.
pub struct PerformancePlugin;

impl Plugin for PerformancePlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize resources
            .init_resource::<PerformanceMetrics>()
            .init_resource::<PerformanceManager>()
            // Register events
            .add_event::<PerformanceWarning>()
            // Add performance monitoring systems
            .add_systems(Update, (
                update_performance_metrics,
                update_virtualization,
                manage_virtualized_entities,
                apply_performance_optimizations,
                optimize_memory_usage,
                emit_performance_warnings,
                log_performance_debug,
            ));
    }
}

//! Performance monitoring and optimization systems

use bevy::prelude::*;
use tracing::debug;
use super::components::VirtualizedEntity;
use super::resources::{PerformanceMetrics, PerformanceManager};
use super::events::{PerformanceWarning, WarningLevel};

/// Update performance metrics with zero allocation
#[inline(always)]
pub fn update_performance_metrics(
    time: Res<Time>,
    mut metrics: ResMut<PerformanceMetrics>,
    entities: Query<Entity>,
) {
    metrics.add_frame_time(time.delta_secs());
    metrics.entity_count = entities.iter().count() as u32;
}

/// Apply performance optimizations with zero allocation
#[inline(always)]
pub fn apply_performance_optimizations(
    manager: Res<PerformanceManager>,
    mut query: Query<&mut VirtualizedEntity>,
) {
    if !manager.virtualization_enabled {
        return;
    }

    // High-performance virtualization logic
    for mut virtualized in query.iter_mut() {
        // Optimize visibility based on priority
        if virtualized.priority < 5 {
            virtualized.visible = false;
        }
    }
}

/// Optimize memory usage with zero allocation
#[inline(always)]
pub fn optimize_memory_usage(
    manager: Res<PerformanceManager>,
    mut metrics: ResMut<PerformanceMetrics>,
) {
    if !manager.memory_optimization_enabled {
        return;
    }

    // Trim frame time buffer if needed
    let max_samples = metrics.max_samples;
    if metrics.frame_times.len() > max_samples {
        metrics.frame_times.truncate(max_samples);
    }
}

/// Manage virtualized entities with zero allocation
#[inline(always)]
pub fn manage_virtualized_entities(mut query: Query<(&mut VirtualizedEntity, &mut Visibility)>) {
    for (virtualized, mut visibility) in query.iter_mut() {
        *visibility = if virtualized.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Update virtualization with zero allocation
#[inline(always)]
pub fn update_virtualization(mut query: Query<&mut VirtualizedEntity>) {
    // High-performance virtualization updates
    for mut virtualized in query.iter_mut() {
        // Update visibility based on performance criteria
        virtualized.visible = virtualized.priority >= 5;
    }
}

/// Log performance debug information
#[inline(always)]
pub fn log_performance_debug(metrics: Res<PerformanceMetrics>, manager: Res<PerformanceManager>) {
    if !manager.debug_enabled {
        return;
    }

    if metrics.entity_count.is_multiple_of(100) {
        let fps = metrics.fps();
        let entity_count = metrics.entity_count;
        let memory_kb = metrics.memory_usage / 1024;

        debug!(
            "Performance: FPS={:.1}, Entities={}, Memory={}KB",
            fps, entity_count, memory_kb
        );
    }
}

/// Emit performance warnings based on metrics
///
/// This system monitors performance and sends events when thresholds are exceeded.
pub fn emit_performance_warnings(
    metrics: Res<PerformanceMetrics>,
    mut warnings: EventWriter<PerformanceWarning>,
) {
    let fps = metrics.fps();
    
    // FPS warnings
    if fps < 30.0 && fps > 0.0 {
        warnings.write(PerformanceWarning {
            level: WarningLevel::Critical,
            message: format!("FPS critically low: {:.1}", fps),
        });
    } else if fps < 50.0 && fps > 0.0 {
        warnings.write(PerformanceWarning {
            level: WarningLevel::Warning,
            message: format!("FPS below target: {:.1}", fps),
        });
    }
    
    // Memory warnings (if > 1GB)
    if metrics.memory_usage > 1_000_000_000 {
        warnings.write(PerformanceWarning {
            level: WarningLevel::Warning,
            message: format!("Memory usage high: {} MB", metrics.memory_usage / 1_000_000),
        });
    }
    
    // Entity count warnings (informational)
    if metrics.entity_count > 10_000 {
        warnings.write(PerformanceWarning {
            level: WarningLevel::Info,
            message: format!("High entity count: {}", metrics.entity_count),
        });
    }
}

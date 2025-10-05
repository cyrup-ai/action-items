//! Performance monitoring resources

use std::collections::VecDeque;
use bevy::prelude::*;

/// Performance metrics resource for zero-allocation tracking
#[derive(Resource, Debug)]
pub struct PerformanceMetrics {
    pub(crate) frame_times: VecDeque<f32>,
    pub memory_usage: u64,
    pub entity_count: u32,
    pub(crate) max_samples: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(60),
            memory_usage: 0,
            entity_count: 0,
            max_samples: 60,
        }
    }
}

impl PerformanceMetrics {
    /// Add frame time sample with zero allocation
    #[inline(always)]
    pub fn add_frame_time(&mut self, frame_time: f32) {
        if self.frame_times.len() >= self.max_samples {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(frame_time);
    }

    /// Get average frame time with zero allocation
    #[inline(always)]
    pub fn average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    /// Get current FPS with zero allocation
    #[inline(always)]
    pub fn fps(&self) -> f32 {
        let avg_time = self.average_frame_time();
        if avg_time > 0.0 { 1.0 / avg_time } else { 0.0 }
    }
}

/// Performance manager for coordinating optimization systems
#[derive(Resource, Debug)]
pub struct PerformanceManager {
    pub virtualization_enabled: bool,
    pub memory_optimization_enabled: bool,
    pub debug_enabled: bool,
}

impl Default for PerformanceManager {
    fn default() -> Self {
        Self {
            virtualization_enabled: true,
            memory_optimization_enabled: true,
            debug_enabled: cfg!(debug_assertions),
        }
    }
}

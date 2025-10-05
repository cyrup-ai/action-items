//! Search UI-related types and structures
//!
//! NOTE: Core search types (SearchResult, SearchScore, etc.) have been migrated to ecs-search package.
//! This file only contains UI-specific types for the search interface.

use std::time::Duration;
use bevy::prelude::*;

/// Real-time search UI state for instant visual feedback
/// Zero-allocation search UI state management with blazing-fast visual updates
#[derive(Component, Debug, Default)]
pub struct RealTimeSearchUI {
    /// Search input loading state
    pub search_loading: bool,
    /// Search debounce timer for visual feedback
    pub debounce_timer: Option<Duration>,
    /// Last query change timestamp for animation timing
    pub last_query_change: Option<std::time::Instant>,
    /// Search progress indicator
    pub search_progress: f32,
    /// Number of results being animated in
    pub animating_results: usize,
}

impl RealTimeSearchUI {
    /// Start search loading state
    #[inline]
    pub fn start_search_loading(&mut self) {
        self.search_loading = true;
        self.search_progress = 0.0;
        self.last_query_change = Some(std::time::Instant::now());
    }

    /// Complete search loading state
    #[inline]
    pub fn complete_search_loading(&mut self) {
        self.search_loading = false;
        self.search_progress = 1.0;
    }

    /// Update search progress animation
    #[inline]
    pub fn update_search_progress(&mut self, delta_time: Duration) {
        if self.search_loading {
            // Simulate progress animation during search
            let progress_speed = 3.0; // Progress per second
            self.search_progress += delta_time.as_secs_f32() * progress_speed;
            self.search_progress = self.search_progress.min(0.9); // Don't reach 100% until complete
        }
    }
}

/// Search performance overlay marker component
#[cfg(debug_assertions)]
#[derive(Component)]
pub struct SearchPerformanceOverlay;

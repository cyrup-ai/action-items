//! Core search filtering logic
//!
//! Provides filter management and result filtering without UI dependencies.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use serde::{Deserialize, Serialize};

/// Global filter statistics with atomic counters
static FILTER_OPERATIONS: AtomicU64 = AtomicU64::new(0);
static FILTERS_APPLIED: AtomicU64 = AtomicU64::new(0);
static RESULTS_FILTERED: AtomicU64 = AtomicU64::new(0);
static ACTIVE_FILTER_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Available filter categories for search results
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterCategory {
    /// Filter by application results
    Applications,
    /// Filter by document/file results
    Documents,
    /// Filter by web/bookmark results
    Web,
    /// Filter by system/settings results
    System,
    /// Filter by plugin-specific results
    Plugin(String),
    /// Filter by custom user-defined categories
    Custom(String),
}

impl FilterCategory {
    /// Get display name for the filter category
    pub fn display_name(&self) -> &str {
        match self {
            Self::Applications => "Applications",
            Self::Documents => "Documents",
            Self::Web => "Web",
            Self::System => "System",
            Self::Plugin(name) => name,
            Self::Custom(name) => name,
        }
    }
}

/// Core filter state without UI components
#[derive(Debug, Default)]
pub struct FilterState {
    /// Active filter categories
    pub active_filters: HashSet<FilterCategory>,
    /// Available filter categories
    pub available_filters: Vec<FilterCategory>,
    /// Results match counts per filter
    pub match_counts: HashMap<FilterCategory, usize>,
}

impl FilterState {
    /// Create new filter state with default categories
    pub fn new() -> Self {
        Self {
            active_filters: HashSet::new(),
            available_filters: vec![
                FilterCategory::Applications,
                FilterCategory::Documents,
                FilterCategory::Web,
                FilterCategory::System,
            ],
            match_counts: HashMap::new(),
        }
    }

    /// Create new filter state with custom categories
    pub fn with_categories(categories: Vec<FilterCategory>) -> Self {
        Self {
            active_filters: HashSet::new(),
            available_filters: categories,
            match_counts: HashMap::new(),
        }
    }

    /// Toggle a specific filter category
    pub fn toggle_filter(&mut self, category: FilterCategory) {
        if self.active_filters.contains(&category) {
            self.active_filters.remove(&category);
            ACTIVE_FILTER_COUNT.fetch_sub(1, Ordering::Relaxed);
        } else {
            self.active_filters.insert(category);
            ACTIVE_FILTER_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        FILTERS_APPLIED.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if a filter category is active
    pub fn is_filter_active(&self, category: &FilterCategory) -> bool {
        self.active_filters.contains(category)
    }

    /// Clear all active filters
    pub fn clear_all_filters(&mut self) {
        let count = self.active_filters.len();
        self.active_filters.clear();
        if count > 0 {
            ACTIVE_FILTER_COUNT.store(0, Ordering::Relaxed);
            FILTERS_APPLIED.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Check if any filters are currently active
    pub fn has_active_filters(&self) -> bool {
        !self.active_filters.is_empty()
    }

    /// Update match count for a filter category
    pub fn update_match_count(&mut self, category: FilterCategory, count: usize) {
        self.match_counts.insert(category, count);
    }

    /// Get match count for a filter category
    pub fn get_match_count(&self, category: &FilterCategory) -> usize {
        self.match_counts.get(category).copied().unwrap_or(0)
    }

    /// Check if result categories match active filters
    pub fn matches_filters(&self, result_categories: &[FilterCategory]) -> bool {
        FILTER_OPERATIONS.fetch_add(1, Ordering::Relaxed);

        // If no filters active, all results match
        if self.active_filters.is_empty() {
            return true;
        }

        // Check if any result category matches any active filter
        let matches = result_categories
            .iter()
            .any(|cat| self.active_filters.contains(cat));

        if !matches {
            RESULTS_FILTERED.fetch_add(1, Ordering::Relaxed);
        }

        matches
    }
}

/// Get filter performance statistics
pub fn get_filter_stats() -> FilterStatistics {
    FilterStatistics {
        filter_operations: FILTER_OPERATIONS.load(Ordering::Relaxed),
        filters_applied: FILTERS_APPLIED.load(Ordering::Relaxed),
        results_filtered: RESULTS_FILTERED.load(Ordering::Relaxed),
        active_filter_count: ACTIVE_FILTER_COUNT.load(Ordering::Relaxed),
    }
}

/// Filter system performance statistics
#[derive(Debug, Clone, Copy)]
pub struct FilterStatistics {
    pub filter_operations: u64,
    pub filters_applied: u64,
    pub results_filtered: u64,
    pub active_filter_count: usize,
}

impl FilterStatistics {
    /// Calculate filter efficiency (operations per filter applied)
    pub fn filter_efficiency(&self) -> f64 {
        if self.filters_applied == 0 {
            0.0
        } else {
            self.filter_operations as f64 / self.filters_applied as f64
        }
    }

    /// Calculate filter rate (results filtered per operation)
    pub fn filter_rate(&self) -> f64 {
        if self.filter_operations == 0 {
            0.0
        } else {
            self.results_filtered as f64 / self.filter_operations as f64
        }
    }
}

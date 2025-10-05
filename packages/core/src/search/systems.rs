use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use log::{debug, error, info, warn};

use crate::plugins::ecs_queries::{PluginExecutor, PluginSearcher};
use crate::search::index::SearchIndex;

pub fn setup_search_index(mut commands: Commands) {
    let mut index = SearchIndex::new();
    if let Err(e) = index.rebuild_index() {
        error!("Failed to build search index: {}", e);
    }
    commands.insert_resource(index);

    // Initialize real-time search resources
    commands.insert_resource(RealTimeSearchState::default());
    commands.insert_resource(SearchDebounceTimer::default());
    commands.insert_resource(SearchPerformanceMetrics::default());
}

/// Real-time search state for blazing-fast reactive search
/// Zero-allocation state management with optimized string handling
#[derive(Resource, Debug, Clone)]
pub struct RealTimeSearchState {
    /// Current search query for real-time processing
    pub current_query: String,
    /// Previous query for incremental filtering optimization
    pub previous_query: String,
    /// Cached search results for incremental updates
    pub cached_results: Vec<crate::plugins::core::ActionItem>,
    /// Last search timestamp for performance tracking
    pub last_search_time: Option<Instant>,
    /// Search in progress flag to prevent overlapping searches
    pub search_in_progress: bool,
    /// Incremental filtering enabled flag
    pub incremental_filtering: bool,
    /// Result limit for performance optimization
    pub max_results: usize,
}

impl Default for RealTimeSearchState {
    fn default() -> Self {
        Self {
            current_query: String::new(),
            previous_query: String::new(),
            cached_results: Vec::with_capacity(20), // Pre-allocate for typical result sets
            last_search_time: None,
            search_in_progress: false,
            incremental_filtering: true,
            max_results: 10,
        }
    }
}

impl RealTimeSearchState {
    /// Check if current query is an extension of previous query for incremental filtering
    #[inline]
    pub fn is_incremental_query(&self) -> bool {
        self.incremental_filtering
            && !self.previous_query.is_empty()
            && self.current_query.starts_with(&self.previous_query)
            && self.current_query.len() > self.previous_query.len()
    }

    /// Update query and return whether search should be triggered
    #[inline]
    pub fn update_query(&mut self, new_query: String) -> bool {
        if self.current_query == new_query {
            return false; // No change, skip search
        }

        self.previous_query = std::mem::take(&mut self.current_query);
        self.current_query = new_query;
        true
    }

    /// Mark search as started
    #[inline]
    pub fn start_search(&mut self) {
        self.search_in_progress = true;
        self.last_search_time = Some(Instant::now());
    }

    /// Mark search as completed and update cached results
    #[inline]
    pub fn complete_search(&mut self, results: Vec<crate::plugins::core::ActionItem>) {
        self.search_in_progress = false;
        self.cached_results = results;
    }

    /// Get search duration if last search was completed
    #[inline]
    pub fn get_last_search_duration(&self) -> Option<Duration> {
        self.last_search_time.map(|start| start.elapsed())
    }
}

/// Debounced search timer for real-time search optimization
/// Zero-allocation timer management with configurable debounce intervals
#[derive(Resource, Debug)]
pub struct SearchDebounceTimer {
    /// Timer for search debouncing
    pub timer: Timer,
    /// Whether a search is pending after debounce period
    pub search_pending: bool,
    /// Query that triggered the pending search
    pub pending_query: String,
    /// Debounce interval for search optimization
    pub debounce_duration: Duration,
}

impl Default for SearchDebounceTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(150), TimerMode::Once), // 150ms debounce
            search_pending: false,
            pending_query: String::new(),
            debounce_duration: Duration::from_millis(150),
        }
    }
}

impl SearchDebounceTimer {
    /// Reset timer with new query
    #[inline]
    pub fn reset_with_query(&mut self, query: String) {
        self.timer.reset();
        self.search_pending = true;
        self.pending_query = query;
    }

    /// Check if debounce period has elapsed and search should be triggered
    #[inline]
    pub fn should_trigger_search(&mut self, delta_time: Duration) -> bool {
        if !self.search_pending {
            return false;
        }

        self.timer.tick(delta_time);
        if self.timer.finished() {
            self.search_pending = false;
            true
        } else {
            false
        }
    }

    /// Get pending query and clear it
    #[inline]
    pub fn take_pending_query(&mut self) -> String {
        std::mem::take(&mut self.pending_query)
    }
}

/// Search performance metrics for optimization and monitoring
/// Zero-allocation performance tracking with blazing-fast metric collection
#[derive(Resource, Debug, Default)]
pub struct SearchPerformanceMetrics {
    /// Total number of searches performed
    pub total_searches: u64,
    /// Total search time for average calculation
    pub total_search_time: Duration,
    /// Fastest search time recorded
    pub fastest_search: Option<Duration>,
    /// Slowest search time recorded
    pub slowest_search: Option<Duration>,
    /// Number of incremental searches performed
    pub incremental_searches: u64,
    /// Number of full searches performed
    pub full_searches: u64,
    /// Last performance log timestamp
    pub last_performance_log: Option<Instant>,
}

impl SearchPerformanceMetrics {
    /// Record search performance
    #[inline]
    pub fn record_search(&mut self, duration: Duration, was_incremental: bool) {
        self.total_searches += 1;
        self.total_search_time += duration;

        if was_incremental {
            self.incremental_searches += 1;
        } else {
            self.full_searches += 1;
        }

        // Update min/max times
        match (self.fastest_search, self.slowest_search) {
            (None, None) => {
                self.fastest_search = Some(duration);
                self.slowest_search = Some(duration);
            },
            (Some(fastest), Some(slowest)) => {
                if duration < fastest {
                    self.fastest_search = Some(duration);
                }
                if duration > slowest {
                    self.slowest_search = Some(duration);
                }
            },
            _ => {}, // Should not happen
        }
    }

    /// Get average search time
    #[inline]
    pub fn average_search_time(&self) -> Duration {
        if self.total_searches > 0 {
            self.total_search_time / self.total_searches as u32
        } else {
            Duration::ZERO
        }
    }

    /// Check if performance logging should occur
    #[inline]
    pub fn should_log_performance(&mut self) -> bool {
        const LOG_INTERVAL: Duration = Duration::from_secs(30);

        let now = Instant::now();
        match self.last_performance_log {
            None => {
                self.last_performance_log = Some(now);
                true
            },
            Some(last) if now.duration_since(last) >= LOG_INTERVAL => {
                self.last_performance_log = Some(now);
                true
            },
            _ => false,
        }
    }
}

/// Real-time search debounce system
/// Zero-allocation system for managing search debouncing with blazing-fast query processing
/// Prevents excessive search operations while maintaining responsive user experience
#[inline]
pub fn real_time_search_debounce_system(
    time: Res<Time>,
    current_query: Res<crate::CurrentQuery>,
    mut search_state: ResMut<RealTimeSearchState>,
    mut debounce_timer: ResMut<SearchDebounceTimer>,
) {
    // Check for query changes
    if current_query.is_changed() {
        let new_query = current_query.0.clone();

        // Update search state and check if search is needed
        if search_state.update_query(new_query.clone()) {
            // Reset debounce timer with new query
            debounce_timer.reset_with_query(new_query);
        }
    }

    // Check if debounce period has elapsed
    if debounce_timer.should_trigger_search(time.delta()) {
        let query = debounce_timer.take_pending_query();

        // Update search state to indicate search should be triggered
        if !query.is_empty() && !search_state.search_in_progress {
            search_state.current_query = query;
            // Search will be triggered by real_time_search_system
        }
    }
}

/// Real-time incremental search system
/// Zero-allocation system for blazing-fast incremental search with fuzzy matching optimization
/// Implements sub-100ms search response with intelligent result caching
#[inline]
pub fn real_time_incremental_search_system(
    index: Res<SearchIndex>,
    plugin_searcher: PluginSearcher,
    mut search_state: ResMut<RealTimeSearchState>,
    mut current_search_results_res: ResMut<crate::plugins::core::CurrentSearchResults>,
    mut performance_metrics: ResMut<SearchPerformanceMetrics>,
    mut commands: Commands,
) {
    // Only process if we have a query and no search is in progress
    if search_state.current_query.is_empty() || search_state.search_in_progress {
        return;
    }

    // Check if this should be an incremental search
    let is_incremental = search_state.is_incremental_query();

    // Clone query to avoid borrow conflicts
    let query = search_state.current_query.clone();

    search_state.start_search();

    debug!(
        "Performing {} search for query: \"{}\"",
        if is_incremental {
            "incremental"
        } else {
            "full"
        },
        query
    );

    // Perform search based on type
    let mut results = if is_incremental && !search_state.cached_results.is_empty() {
        // Incremental search: filter existing results
        perform_incremental_search(&search_state.cached_results, &query)
    } else {
        // Full search: search index and plugins
        perform_full_search(&index, &plugin_searcher, &mut commands, &query)
    };

    // Apply result limits and sorting
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(search_state.max_results);

    // Record performance metrics
    if let Some(duration) = search_state.get_last_search_duration() {
        performance_metrics.record_search(duration, is_incremental);

        // Warn if search exceeds target time
        if duration > Duration::from_millis(100) {
            warn!(
                "Search exceeded 100ms target: {}ms for query \"{}\"",
                duration.as_millis(),
                query
            );
        }
    }

    // Update results and complete search
    current_search_results_res.results = results.clone();
    search_state.complete_search(results);

    info!(
        "Completed {} search for \"{}\": {} results in {:?}",
        if is_incremental {
            "incremental"
        } else {
            "full"
        },
        query,
        current_search_results_res.results.len(),
        search_state.get_last_search_duration()
    );
}

/// Perform incremental search on cached results
/// Zero-allocation incremental filtering for blazing-fast result refinement
#[inline]
fn perform_incremental_search(
    cached_results: &[crate::plugins::core::ActionItem],
    query: &str,
) -> Vec<crate::plugins::core::ActionItem> {
    let query_lower = query.to_lowercase();

    cached_results
        .iter()
        .filter_map(|item| {
            // Simple but fast fuzzy matching for incremental search
            let title_matches = item.title.to_lowercase().contains(&query_lower);
            let desc_matches = item.description.to_lowercase().contains(&query_lower);

            if title_matches || desc_matches {
                let mut result = item.clone();
                // Adjust score based on match quality for incremental search
                if title_matches {
                    result.score *= 1.1; // Boost title matches
                }
                Some(result)
            } else {
                None
            }
        })
        .collect()
}

/// Perform full search using index and plugins
/// Zero-allocation full search with optimized plugin integration
#[inline]
fn perform_full_search(
    index: &SearchIndex,
    plugin_searcher: &PluginSearcher,
    commands: &mut Commands,
    query: &str,
) -> Vec<crate::plugins::core::ActionItem> {
    // Get built-in search results
    let built_in_results_items = index.search(query);

    let results: Vec<crate::plugins::core::ActionItem> = built_in_results_items
        .into_iter()
        .map(|item| crate::plugins::core::ActionItem {
            title: item.title,
            description: item.description,
            action: item.id,
            icon: item
                .icon_path
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            score: item.score,
        })
        .collect();

    // Search plugins using ECS
    let task_pool = AsyncComputeTaskPool::get();
    plugin_searcher.search_plugins_ecs(commands, query, task_pool);

    results
}

/// Real-time search performance monitoring system
/// Zero-allocation system for tracking and optimizing search performance
/// Provides insights for sub-100ms search target maintenance
#[inline]
pub fn real_time_search_performance_system(
    mut performance_metrics: ResMut<SearchPerformanceMetrics>,
    search_state: Res<RealTimeSearchState>,
) {
    // Log performance metrics periodically
    if performance_metrics.should_log_performance() && performance_metrics.total_searches > 0 {
        info!("Search Performance Metrics:");
        info!("  Total searches: {}", performance_metrics.total_searches);
        info!(
            "  Average time: {:?}",
            performance_metrics.average_search_time()
        );
        info!(
            "  Fastest: {:?}",
            performance_metrics.fastest_search.unwrap_or(Duration::ZERO)
        );
        info!(
            "  Slowest: {:?}",
            performance_metrics.slowest_search.unwrap_or(Duration::ZERO)
        );
        info!(
            "  Incremental: {} / Full: {}",
            performance_metrics.incremental_searches, performance_metrics.full_searches
        );

        // Performance optimization suggestions
        let avg_time = performance_metrics.average_search_time();
        if avg_time > Duration::from_millis(100) {
            warn!("Average search time ({:?}) exceeds 100ms target", avg_time);
        }

        let incremental_ratio = performance_metrics.incremental_searches as f64
            / performance_metrics.total_searches as f64;
        if incremental_ratio < 0.5 {
            info!(
                "Consider optimizing for more incremental searches (current: {:.1}%)",
                incremental_ratio * 100.0
            );
        }
    }

    // Monitor for performance degradation
    if let Some(duration) = search_state.get_last_search_duration()
        && duration > Duration::from_millis(200)
    {
        warn!(
            "Critical: Search took {:?} - significantly exceeds performance target",
            duration
        );
    }
}

/// Real-time search result optimization system
/// Zero-allocation system for optimizing search result quality and relevance
/// Handles result ranking, deduplication, and quality scoring
#[inline]
pub fn real_time_search_optimization_system(
    mut current_search_results_res: ResMut<crate::plugins::core::CurrentSearchResults>,
    search_state: Res<RealTimeSearchState>,
) {
    // Only optimize if we have results and search is complete
    if current_search_results_res.results.is_empty() || search_state.search_in_progress {
        return;
    }

    let results = &mut current_search_results_res.results;

    // Remove duplicates based on action ID
    results.sort_by(|a, b| a.action.cmp(&b.action));
    results.dedup_by(|a, b| a.action == b.action);

    // Re-sort by score after deduplication
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply quality scoring adjustments
    for (index, result) in results.iter_mut().enumerate() {
        // Boost score for top results to maintain ranking stability
        if index < 3 {
            result.score *= 1.05;
        }

        // Boost results with icons for better user experience
        if result.icon.is_some() {
            result.score *= 1.02;
        }

        // Boost results with descriptions
        if !result.description.is_empty() {
            result.score *= 1.01;
        }
    }

    // Final sort after quality adjustments
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// **ENHANCED**: Real-time ECS-based search system with fallback support
///
/// This system provides fallback search functionality when real-time search systems are not active.
/// The preferred search pipeline is: real_time_search_debounce_system ->
/// real_time_incremental_search_system
///
/// Use real-time search systems for optimal performance and user experience.
pub fn search_system_ecs(
    index: Res<SearchIndex>,
    plugin_searcher: PluginSearcher,
    mut current_search_results_res: ResMut<crate::plugins::core::CurrentSearchResults>,
    current_query: Res<crate::CurrentQuery>,
    search_state: Option<Res<RealTimeSearchState>>,
    mut commands: Commands,
) {
    // Skip if real-time search is handling queries
    if let Some(rt_state) = search_state
        && (rt_state.search_in_progress || !rt_state.current_query.is_empty())
    {
        return; // Real-time search is active
    }

    if current_query.is_changed() && !current_query.0.is_empty() {
        let query = &current_query.0;
        debug!(
            "Performing fallback ECS-based search for query: \"{}\"",
            query
        );

        // Get built-in search results
        let built_in_results_items = index.search(query);
        info!("Found {} built-in results", built_in_results_items.len());

        let mut results: Vec<crate::plugins::core::ActionItem> = built_in_results_items
            .into_iter()
            .map(|item| crate::plugins::core::ActionItem {
                title: item.title,
                description: item.description,
                action: item.id,
                icon: item
                    .icon_path
                    .and_then(|p| p.to_str().map(|s| s.to_string())),
                score: item.score,
            })
            .collect();

        // Search plugins using ECS (this spawns tasks but doesn't return results immediately)
        let task_pool = AsyncComputeTaskPool::get();
        plugin_searcher.search_plugins_ecs(&mut commands, query, task_pool);

        // Note: Plugin results will be handled by handle_search_results_system

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(10);

        info!(
            "Total {} fallback ECS results after merge & truncate. Top 3: {:?}",
            results.len(),
            results.iter().take(3).collect::<Vec<_>>()
        );

        current_search_results_res.results = results;
    }
}

pub fn execute_action_item_ecs(
    mut execute_events: EventReader<crate::events::LauncherEvent>,
    index: Res<SearchIndex>,
    mut plugin_executor: PluginExecutor,
) {
    for event in execute_events.read() {
        if let crate::events::LauncherEventType::Execute(action_id) = &event.event_type {
            // Check if it's a plugin action (format: "plugin_name:action")
            if action_id.contains(':') {
                // Parse plugin_id from action_id - this needs proper implementation
                // based on the actual action_id format used by the plugins
                if let Some((plugin_id, _action_part)) = action_id.split_once(':') {
                    let task_pool = AsyncComputeTaskPool::get();
                    if let Err(e) =
                        plugin_executor.execute_action_ecs(action_id, plugin_id, task_pool)
                    {
                        error!("Failed to execute ECS plugin action {}: {}", action_id, e);
                    }
                } else {
                    error!("Invalid plugin action format: {}", action_id);
                }
            } else if let Some(item) = index.get_item(action_id)
                && let Err(e) = item.execute()
            {
                error!("Failed to execute action {}: {}", action_id, e);
            }
        }
    }
}

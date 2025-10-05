//! Core search components and types
//!
//! Provides component types for search functionality in the ECS.

use bevy::prelude::*;
use bevy::tasks::Task;
use serde::{Deserialize, Serialize};

use crate::scoring::SearchScore;
use crate::systems::filtering::FilterCategory;

/// Search result component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Unique identifier for this result
    pub id: String,
    /// Display title
    pub title: String,
    /// Subtitle or description
    pub subtitle: String,
    /// Icon identifier or path
    pub icon: String,
    /// Search score
    pub score: f32,
    /// Full score information
    #[serde(skip)]
    pub score_details: SearchScore,
    /// Filter categories this result belongs to
    pub categories: Vec<FilterCategory>,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(
        id: String,
        title: String,
        subtitle: String,
        icon: String,
        score: f32,
    ) -> Self {
        Self {
            id,
            title,
            subtitle,
            icon,
            score,
            score_details: SearchScore::default(),
            categories: Vec::new(),
        }
    }

    /// Create a new search result with categories
    pub fn with_categories(mut self, categories: Vec<FilterCategory>) -> Self {
        self.categories = categories;
        self
    }

    /// Create a new search result with full score details
    pub fn with_score_details(mut self, score_details: SearchScore) -> Self {
        self.score_details = score_details;
        self.score = score_details.final_score;
        self
    }
}

/// Component for async search tasks
#[derive(Component)]
pub struct SearchTask {
    /// The search query being processed
    pub query: String,
    /// Async task handle
    pub task_handle: Task<Vec<SearchResult>>,
    /// When the search started
    pub started_at: std::time::Instant,
}

impl SearchTask {
    /// Create a new search task
    pub fn new(query: String, task_handle: Task<Vec<SearchResult>>) -> Self {
        Self {
            query,
            task_handle,
            started_at: std::time::Instant::now(),
        }
    }

    /// Get elapsed time since search started
    pub fn elapsed(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }
}

/// Marker component for search-related entities
#[derive(Component, Debug)]
pub struct SearchEntity;

/// Component marking an entity as having search capabilities
#[derive(Component, Debug)]
pub struct Searchable {
    /// Searchable text content
    pub content: String,
    /// Keywords for improved matching
    pub keywords: Vec<String>,
    /// Priority weight for ranking (higher = more important)
    pub weight: f32,
}

impl Searchable {
    /// Create a new searchable component
    pub fn new(content: String) -> Self {
        Self {
            content,
            keywords: Vec::new(),
            weight: 1.0,
        }
    }

    /// Add keywords for improved matching
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Set priority weight
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }
}

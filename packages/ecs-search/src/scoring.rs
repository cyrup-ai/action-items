//! Core search scoring algorithms
//!
//! Provides scoring calculations for search results with confidence indicators.
//! This module contains the core scoring logic without UI components.

use std::sync::atomic::{AtomicU64, Ordering};

/// Global scoring statistics with atomic counters for zero-allocation monitoring
static SCORE_CALCULATIONS: AtomicU64 = AtomicU64::new(0);
static HIGH_CONFIDENCE_RESULTS: AtomicU64 = AtomicU64::new(0);
static LOW_CONFIDENCE_RESULTS: AtomicU64 = AtomicU64::new(0);

/// Comprehensive score information for search results
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SearchScore {
    /// Overall confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Text matching score (0.0 to 1.0)
    pub text_match: f32,
    /// Frequency/usage score (0.0 to 1.0)
    pub frequency: f32,
    /// Recency score (0.0 to 1.0)
    pub recency: f32,
    /// Context relevance score (0.0 to 1.0)
    pub relevance: f32,
    /// Final composite score (0.0 to 1.0)
    pub final_score: f32,
}

impl Default for SearchScore {
    fn default() -> Self {
        Self {
            confidence: 0.5,
            text_match: 0.0,
            frequency: 0.0,
            recency: 0.0,
            relevance: 0.0,
            final_score: 0.0,
        }
    }
}

impl SearchScore {
    /// Create new search score with individual components
    pub fn new(text_match: f32, frequency: f32, recency: f32, relevance: f32) -> Self {
        let components = [text_match, frequency, recency, relevance];
        let final_score = Self::calculate_composite_score(&components);
        let confidence = Self::calculate_confidence(&components, final_score);

        SCORE_CALCULATIONS.fetch_add(1, Ordering::Relaxed);

        Self {
            confidence,
            text_match,
            frequency,
            recency,
            relevance,
            final_score,
        }
    }

    /// Calculate composite score using weighted average
    fn calculate_composite_score(components: &[f32]) -> f32 {
        // Weights: text match is most important, then relevance, frequency, recency
        let weights = [0.4, 0.25, 0.2, 0.15];
        let weighted_sum: f32 = components
            .iter()
            .zip(weights.iter())
            .map(|(score, weight)| score * weight)
            .sum();

        weighted_sum.clamp(0.0, 1.0)
    }

    /// Calculate confidence based on score consistency
    fn calculate_confidence(components: &[f32], final_score: f32) -> f32 {
        let mean = components.iter().sum::<f32>() / components.len() as f32;
        let variance =
            components.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / components.len() as f32;

        // High variance = low confidence, low variance = high confidence
        let consistency = (1.0 - variance).max(0.0);

        // Combine consistency with final score strength
        (consistency * 0.6 + final_score * 0.4).clamp(0.0, 1.0)
    }

    /// Get score tier for classification
    pub fn score_tier(&self) -> ScoreTier {
        match self.final_score {
            x if x >= 0.8 => ScoreTier::Excellent,
            x if x >= 0.6 => ScoreTier::Good,
            x if x >= 0.4 => ScoreTier::Average,
            x if x >= 0.2 => ScoreTier::Poor,
            _ => ScoreTier::VeryPoor,
        }
    }

    /// Get confidence level for classification
    pub fn confidence_level(&self) -> ConfidenceLevel {
        match self.confidence {
            x if x >= 0.8 => ConfidenceLevel::VeryHigh,
            x if x >= 0.6 => ConfidenceLevel::High,
            x if x >= 0.4 => ConfidenceLevel::Medium,
            x if x >= 0.2 => ConfidenceLevel::Low,
            _ => ConfidenceLevel::VeryLow,
        }
    }

    /// Check if this is a high confidence result
    pub fn is_high_confidence(&self) -> bool {
        let is_high = self.final_score >= 0.7 && self.confidence >= 0.6;
        if is_high {
            HIGH_CONFIDENCE_RESULTS.fetch_add(1, Ordering::Relaxed);
        } else {
            LOW_CONFIDENCE_RESULTS.fetch_add(1, Ordering::Relaxed);
        }
        is_high
    }
}

/// Score tier classifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreTier {
    Excellent,
    Good,
    Average,
    Poor,
    VeryPoor,
}

impl ScoreTier {
    /// Get display name for this tier
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Excellent => "Excellent Match",
            Self::Good => "Good Match",
            Self::Average => "Average Match",
            Self::Poor => "Poor Match",
            Self::VeryPoor => "Very Poor Match",
        }
    }
}

/// Confidence level classifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    VeryHigh,
    High,
    Medium,
    Low,
    VeryLow,
}

/// Get scoring performance statistics
pub fn get_scoring_stats() -> ScoringStatistics {
    ScoringStatistics {
        score_calculations: SCORE_CALCULATIONS.load(Ordering::Relaxed),
        high_confidence_results: HIGH_CONFIDENCE_RESULTS.load(Ordering::Relaxed),
        low_confidence_results: LOW_CONFIDENCE_RESULTS.load(Ordering::Relaxed),
    }
}

/// Scoring system performance statistics
#[derive(Debug, Clone, Copy)]
pub struct ScoringStatistics {
    pub score_calculations: u64,
    pub high_confidence_results: u64,
    pub low_confidence_results: u64,
}

impl ScoringStatistics {
    /// Calculate average confidence ratio
    pub fn confidence_ratio(&self) -> f64 {
        let total_results = self.high_confidence_results + self.low_confidence_results;
        if total_results == 0 {
            0.0
        } else {
            self.high_confidence_results as f64 / total_results as f64
        }
    }
}

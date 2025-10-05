//! ECS Launcher Service Systems
//!
//! Production-grade modular systems for ECS launcher service providing action execution,
//! search coordination, plugin discovery, and UI management with comprehensive
//! error handling, security validation, and performance optimization.
//!
//! This module coordinates focused subsystems with clear separation of concerns:
//! - Actions: Zero-allocation action execution with timeout handling
//! - Search: High-performance search with aggregator integration
//! - Plugins: Blazing-fast plugin discovery and capability indexing
//! - UI: Zero-allocation UI state management and window control
//! - Core: Cleanup operations, metrics tracking, and helper functions

// Focused subsystem modules in systems directory
pub mod actions;
pub mod core;
pub mod plugins;
pub mod search;
pub mod ui;

// Re-export all subsystem functionality for clean public API
pub use core::*;
use std::sync::atomic::{AtomicU64, Ordering};

pub use actions::*;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::Task;
pub use plugins::*;
pub use search::*;
pub use ui::*;

// Global performance counters for zero-allocation metrics tracking
static TOTAL_ACTIONS_PROCESSED: AtomicU64 = AtomicU64::new(0);
static TOTAL_SEARCHES_EXECUTED: AtomicU64 = AtomicU64::new(0);
static TOTAL_PLUGINS_DISCOVERED: AtomicU64 = AtomicU64::new(0);

/// Search constraints for advanced aggregator integration
#[derive(Debug, Clone)]
pub struct SearchConstraints {
    pub max_results_per_plugin: usize,
    pub timeout_per_plugin: std::time::Duration,
    pub parallel_execution: bool,
    pub result_deduplication: bool,
}

impl Default for SearchConstraints {
    fn default() -> Self {
        Self {
            max_results_per_plugin: 50,
            timeout_per_plugin: std::time::Duration::from_secs(2),
            parallel_execution: true,
            result_deduplication: true,
        }
    }
}

/// Associated task tracking component for linking entities to their tasks
#[derive(Component, Debug, Clone)]
pub struct AssociatedTask(pub Entity);

// Task Components for ECS integration
// ActionExecutionTask is defined in components.rs - removed duplicate definition

// SearchTask is defined in components.rs - removed duplicate definition

#[derive(Component)]
pub struct PluginDiscoveryTask(pub Task<CommandQueue>);

/// Get total actions processed (atomic read)
pub fn get_total_actions_processed() -> u64 {
    TOTAL_ACTIONS_PROCESSED.load(Ordering::Relaxed)
}

/// Get total searches executed (atomic read)
pub fn get_total_searches_executed() -> u64 {
    TOTAL_SEARCHES_EXECUTED.load(Ordering::Relaxed)
}

/// Get total plugins discovered (atomic read)
pub fn get_total_plugins_discovered() -> u64 {
    TOTAL_PLUGINS_DISCOVERED.load(Ordering::Relaxed)
}

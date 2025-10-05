//! ECS Deno Runtime Service
//!
//! A comprehensive Bevy ECS service for Deno JavaScript runtime operations including:
//! - Raycast extension discovery with performance optimizations
//! - JavaScript execution with security sandboxing
//! - Async operation tracking with proper ECS patterns
//! - Zero-allocation optimizations for performance-critical paths
//!
//! This service converts the original deno-ops functionality into a proper
//! Bevy ECS service following the established patterns of other ecs-* packages.

pub mod plugin;
pub mod events;
pub mod resources;
pub mod components;
pub mod systems;
pub mod performance;

// Keep original deno-ops modules for internal use
pub mod discovery;
pub mod discovery_ops;
pub mod raycast_errors;
pub mod raycast_types;

// Re-export main ECS API
pub use plugin::{DenoPlugin, DenoService, SandboxConfig, PerformanceConfig};
pub use events::{
    DenoOperationId, DenoScriptExecutionRequested, DenoScriptExecutionCompleted,
    DenoScriptExecutionFailed, ExtensionDiscoveryRequested, ExtensionDiscoveryCompleted,
    ExtensionDiscoveryFailed, ScriptExecutionResult,
    DenoExecutionError, ExtensionDiscoveryError, DenoMetricsReport,
};
pub use resources::{
    DenoRuntimePool, DenoOperationTracker, ExtensionDiscoveryManager, DenoMetrics,
};
pub use components::{
    DenoScriptExecution, ExtensionDiscoveryOperation, DenoOperationTimeout,
    DenoOperationStatus, ScriptExecutionBundle, ExtensionDiscoveryBundle,
};
pub use performance::{
    MemorySnapshot, MemoryDelta, TimingContext, TimingResult, 
    PerformanceMonitor, OperationPerformanceData, PerformanceError,
    PerformanceStatistics, TimingStatistics, PerformanceTrend,
};

// Legacy deno-ops functions are now internal-only - use ECS events for external API:
// - ExtensionDiscoveryRequested -> ExtensionDiscoveryCompleted/Failed
// - DenoScriptExecutionRequested -> DenoScriptExecutionCompleted/Failed

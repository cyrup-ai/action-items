//! Professional focused window detection for multi-monitor positioning
//!
//! Implements platform-specific APIs for detecting the currently focused application window
//! with zero allocation, blazing-fast performance, and comprehensive error handling.
//!
//! The module has been decomposed into logical submodules:
//! - `types` - Core data structures and error types
//! - `geometry` - Window bounds intersection and overlap calculations
//! - `platform` - Cross-platform entry point for focused window detection
//! - `macos` - macOS-specific implementation using NSWorkspace + Core Graphics
//! - `monitor_calculations` - Best monitor selection based on window overlap
//! - `system_integration` - Bevy system for updating active monitor

pub mod geometry;
pub mod linux;
pub mod macos;
pub mod monitor_calculations;
pub mod platform;
pub mod system_integration;
pub mod types;
pub mod windows;

// Re-export all public types and functions

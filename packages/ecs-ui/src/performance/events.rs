//! Performance monitoring events

use bevy::prelude::*;

/// Event fired when performance issues are detected
#[derive(Event, Debug, Clone)]
pub struct PerformanceWarning {
    pub level: WarningLevel,
    pub message: String,
}

/// Severity level for performance warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningLevel {
    /// Informational message (e.g., high entity count)
    Info,
    /// Performance degradation detected (e.g., FPS below 50)
    Warning,
    /// Critical performance issue (e.g., FPS below 30)
    Critical,
}

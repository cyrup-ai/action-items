use bevy::prelude::*;

use crate::prelude::*;

/// Resource to control progress debug logging
///
/// When enabled, this will log progress updates, completions, and other
/// debug information to help troubleshoot loading issues.
#[derive(Resource, Debug, Clone)]
pub struct ProgressLogger {
    /// Whether debug logging is enabled
    pub enabled: bool,
    /// Log level threshold for progress updates
    pub log_level: LogLevel,
    /// Whether to log individual progress updates (can be verbose)
    pub log_updates: bool,
    /// Whether to log completion events
    pub log_completions: bool,
    /// Whether to log state transitions
    pub log_transitions: bool,
}

/// Log level for progress debugging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Only log errors
    Error,
    /// Log warnings and errors
    Warn,
    /// Log info, warnings, and errors
    Info,
    /// Log debug info and above
    Debug,
    /// Log everything including trace
    Trace,
}

impl Default for ProgressLogger {
    fn default() -> Self {
        Self {
            enabled: false,
            log_level: LogLevel::Debug,
            log_updates: true,
            log_completions: true,
            log_transitions: true,
        }
    }
}

impl ProgressLogger {
    /// Create a new logger with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable debug logging
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the log level
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    /// Configure what to log
    pub fn with_logging_options(
        mut self,
        updates: bool,
        completions: bool,
        transitions: bool,
    ) -> Self {
        self.log_updates = updates;
        self.log_completions = completions;
        self.log_transitions = transitions;
        self
    }

    /// Enable all logging options with debug level
    pub fn debug_all() -> Self {
        Self {
            enabled: true,
            log_level: LogLevel::Debug,
            log_updates: true,
            log_completions: true,
            log_transitions: true,
        }
    }

    /// Enable only completion and transition logging with info level
    pub fn info_milestones() -> Self {
        Self {
            enabled: true,
            log_level: LogLevel::Info,
            log_updates: false,
            log_completions: true,
            log_transitions: true,
        }
    }

    /// Check if logging is enabled for the current configuration
    pub fn should_log(&self) -> bool {
        self.enabled
    }

    /// Check if updates should be logged
    pub fn should_log_updates(&self) -> bool {
        self.enabled && self.log_updates
    }

    /// Check if completions should be logged
    pub fn should_log_completions(&self) -> bool {
        self.enabled && self.log_completions
    }

    /// Check if transitions should be logged
    pub fn should_log_transitions(&self) -> bool {
        self.enabled && self.log_transitions
    }
}

/// System that logs progress updates when debug logging is enabled
pub fn debug_progress<S: States>(
    logger: Option<Res<ProgressLogger>>,
    monitor: Res<ProgressMonitor<S>>,
    state: Res<State<S>>,
    mut progress_reader: EventReader<Progress>,
    mut hidden_reader: EventReader<HiddenProgress>,
) {
    let Some(logger) = logger else { return };
    if !logger.should_log_updates() {
        return;
    }

    let mut has_updates = false;

    // Log individual progress events
    for progress in progress_reader.read() {
        has_updates = true;
        match logger.log_level {
            LogLevel::Trace => trace!(
                "Progress update: {}/{} ({:.1}%)",
                progress.done,
                progress.total,
                progress.fraction() * 100.0
            ),
            LogLevel::Debug => debug!(
                "Progress update: {}/{} ({:.1}%)",
                progress.done,
                progress.total,
                progress.fraction() * 100.0
            ),
            LogLevel::Info => info!(
                "Progress update: {}/{} ({:.1}%)",
                progress.done,
                progress.total,
                progress.fraction() * 100.0
            ),
            _ => {}
        }
    }

    for hidden in hidden_reader.read() {
        has_updates = true;
        match logger.log_level {
            LogLevel::Trace => trace!(
                "Hidden progress update: {}/{} ({:.1}%)",
                hidden.done,
                hidden.total,
                hidden.fraction() * 100.0
            ),
            LogLevel::Debug => debug!(
                "Hidden progress update: {}/{} ({:.1}%)",
                hidden.done,
                hidden.total,
                hidden.fraction() * 100.0
            ),
            _ => {} /* Hidden progress is typically only logged at
                     * debug/trace level */
        }
    }

    // Log overall progress summary if there were updates
    if has_updates {
        let visible = monitor.get_total_visible();
        let hidden = monitor.get_total_hidden();
        let is_complete = monitor.is_complete();

        match logger.log_level {
            LogLevel::Trace | LogLevel::Debug => {
                debug!(
                    "Total progress in {:?}: Visible {}/{} ({:.1}%), Hidden {}/{} ({:.1}%), Complete: {}",
                    state.get(),
                    visible.done,
                    visible.total,
                    visible.fraction() * 100.0,
                    hidden.done,
                    hidden.total,
                    hidden.fraction() * 100.0,
                    is_complete
                );
            }
            LogLevel::Info => {
                if is_complete {
                    info!("Progress complete in state {:?}", state.get());
                }
            }
            _ => {}
        }
    }
}

/// System that logs progress completion events
pub fn debug_progress_completion<S: States>(
    logger: Option<Res<ProgressLogger>>,
    mut completion_reader: EventReader<ProgressComplete<S>>,
) {
    let Some(logger) = logger else { return };
    if !logger.should_log_completions() {
        return;
    }

    for completion in completion_reader.read() {
        match logger.log_level {
            LogLevel::Error => {
                error!("Progress completed for state: {:?}", completion.state)
            }
            LogLevel::Warn => {
                warn!("Progress completed for state: {:?}", completion.state)
            }
            LogLevel::Info => {
                info!("Progress completed for state: {:?}", completion.state)
            }
            LogLevel::Debug => {
                debug!("Progress completed for state: {:?}", completion.state)
            }
            LogLevel::Trace => {
                trace!("Progress completed for state: {:?}", completion.state)
            }
        }
    }
}

/// System that logs state transitions
pub fn debug_state_transitions<S: States>(
    logger: Option<Res<ProgressLogger>>,
    mut state_events: EventReader<StateTransitionEvent<S>>,
) {
    let Some(logger) = logger else { return };
    if !logger.should_log_transitions() {
        return;
    }

    for transition in state_events.read() {
        if let Some(exited) = &transition.exited {
            if let Some(entered) = &transition.entered {
                match logger.log_level {
                    LogLevel::Error => error!(
                        "State transition: {:?} -> {:?}",
                        exited, entered
                    ),
                    LogLevel::Warn => {
                        warn!("State transition: {:?} -> {:?}", exited, entered)
                    }
                    LogLevel::Info => {
                        info!("State transition: {:?} -> {:?}", exited, entered)
                    }
                    LogLevel::Debug => debug!(
                        "State transition: {:?} -> {:?}",
                        exited, entered
                    ),
                    LogLevel::Trace => trace!(
                        "State transition: {:?} -> {:?}",
                        exited, entered
                    ),
                }
            }
        }
    }
}

/// Run condition for debug systems - only run when logging is enabled
pub fn debug_logging_enabled(logger: Option<Res<ProgressLogger>>) -> bool {
    logger.map(|l| l.enabled).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_logger_defaults() {
        let logger = ProgressLogger::default();
        assert!(!logger.enabled);
        assert_eq!(logger.log_level, LogLevel::Debug);
        assert!(logger.log_updates);
        assert!(logger.log_completions);
        assert!(logger.log_transitions);
    }

    #[test]
    fn test_progress_logger_builders() {
        let logger = ProgressLogger::debug_all();
        assert!(logger.enabled);
        assert_eq!(logger.log_level, LogLevel::Debug);
        assert!(logger.should_log_updates());
        assert!(logger.should_log_completions());
        assert!(logger.should_log_transitions());

        let logger = ProgressLogger::info_milestones();
        assert!(logger.enabled);
        assert_eq!(logger.log_level, LogLevel::Info);
        assert!(!logger.should_log_updates());
        assert!(logger.should_log_completions());
        assert!(logger.should_log_transitions());
    }

    #[test]
    fn test_should_log_conditions() {
        let mut logger = ProgressLogger::new().with_enabled(true);
        assert!(logger.should_log());

        logger = logger.with_logging_options(false, false, false);
        assert!(!logger.should_log_updates());
        assert!(!logger.should_log_completions());
        assert!(!logger.should_log_transitions());

        logger.enabled = false;
        assert!(!logger.should_log());
    }
}

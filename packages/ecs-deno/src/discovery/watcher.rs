//! File system watching for real-time changes
//!
//! This module provides file system monitoring capabilities for detecting
//! changes to plugin extensions in real-time using the notify crate.

use std::path::{Path, PathBuf};

use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tracing;

use super::types::*;

/// Discovery events for file system changes
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    Added(PathBuf),
    Modified(PathBuf),
    Removed(PathBuf),
}

/// Plugin watcher for file system monitoring
pub struct PluginWatcher {
    watcher: Option<notify::RecommendedWatcher>,
    event_receiver: mpsc::UnboundedReceiver<DiscoveryEvent>,
    _event_sender: mpsc::UnboundedSender<DiscoveryEvent>, // Keep sender alive
}

impl PluginWatcher {
    /// Create a new plugin watcher
    pub fn new() -> Result<Self, RaycastDiscoveryError> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let sender_clone = event_sender.clone();

        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let Some(discovery_event) = Self::convert_notify_event(event)
                        && sender_clone.send(discovery_event).is_err() {
                            // Channel closed, watcher should stop
                        }
                },
                Err(e) => {
                    // Log error but continue watching
                    tracing::error!("File watcher error: {}", e);
                },
            }
        })
        .map_err(|e| RaycastDiscoveryError::Filesystem {
            path: "watcher_init".to_string(),
            operation: "create_watcher".to_string(),
            source: e.to_string(),
        })?;

        Ok(Self {
            watcher: Some(watcher),
            event_receiver,
            _event_sender: event_sender,
        })
    }

    /// Convert notify events to discovery events
    fn convert_notify_event(event: Event) -> Option<DiscoveryEvent> {
        // Only process events for package.json files or extension directories
        let paths: Vec<PathBuf> = event
            .paths
            .into_iter()
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name == "package.json")
                    .unwrap_or(false)
                    || path.is_dir()
            })
            .collect();

        if paths.is_empty() {
            return None;
        }

        // Use the first relevant path
        let path = paths.into_iter().next()?;

        match event.kind {
            EventKind::Create(_) => Some(DiscoveryEvent::Added(path)),
            EventKind::Modify(_) => Some(DiscoveryEvent::Modified(path)),
            EventKind::Remove(_) => Some(DiscoveryEvent::Removed(path)),
            _ => None,
        }
    }

    /// Start watching a directory for changes
    pub fn watch_directory(&mut self, path: &Path) -> Result<(), RaycastDiscoveryError> {
        if let Some(ref mut watcher) = self.watcher {
            watcher.watch(path, RecursiveMode::Recursive).map_err(|e| {
                RaycastDiscoveryError::Filesystem {
                    path: path.to_string_lossy().into_owned(),
                    operation: "watch_directory".to_string(),
                    source: e.to_string(),
                }
            })?;
        }
        Ok(())
    }

    /// Stop watching a specific directory
    pub fn unwatch_directory(&mut self, path: &Path) -> Result<(), RaycastDiscoveryError> {
        if let Some(ref mut watcher) = self.watcher {
            watcher
                .unwatch(path)
                .map_err(|e| RaycastDiscoveryError::Filesystem {
                    path: path.to_string_lossy().into_owned(),
                    operation: "unwatch_directory".to_string(),
                    source: e.to_string(),
                })?;
        }
        Ok(())
    }

    /// Get the next discovery event
    pub async fn next_event(&mut self) -> Option<DiscoveryEvent> {
        self.event_receiver.recv().await
    }

    /// Stop watching and cleanup resources
    pub fn stop_watching(&mut self) {
        self.event_receiver.close();
        self.watcher = None;
    }

    /// Check if the watcher is active
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }

    /// Create a fallback watcher that doesn't actually watch files (for initialization safety)
    fn new_fallback() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            watcher: None, // No actual file watcher
            event_receiver,
            _event_sender: event_sender,
        }
    }
}

impl Default for PluginWatcher {
    fn default() -> Self {
        match Self::new() {
            Ok(watcher) => watcher,
            Err(error) => {
                // Log the error but create a minimal fallback watcher
                tracing::warn!("Failed to create file system watcher: {}", error);
                Self::new_fallback()
            },
        }
    }
}

impl Drop for PluginWatcher {
    fn drop(&mut self) {
        self.stop_watching();
    }
}

use bevy::prelude::*;

/// Resource to track plugin loading progress
#[derive(Resource, Default)]
pub struct PluginLoadingProgress {
    pub total_plugins: usize,
    pub loaded_plugins: usize,
    pub failed_plugins: usize,
    pub loading_complete: bool,
}

impl PluginLoadingProgress {
    pub fn new(total: usize) -> Self {
        Self {
            total_plugins: total,
            loaded_plugins: 0,
            failed_plugins: 0,
            loading_complete: false,
        }
    }

    pub fn mark_loaded(&mut self) {
        self.loaded_plugins += 1;
        self.check_completion();
    }

    pub fn mark_failed(&mut self) {
        self.failed_plugins += 1;
        self.check_completion();
    }

    fn check_completion(&mut self) {
        if self.loaded_plugins + self.failed_plugins >= self.total_plugins {
            self.loading_complete = true;
        }
    }

    pub fn progress_percentage(&self) -> f32 {
        if self.total_plugins == 0 {
            100.0
        } else {
            ((self.loaded_plugins + self.failed_plugins) as f32 / self.total_plugins as f32) * 100.0
        }
    }
}

/// System to check loading completion and send completion event
pub fn check_plugin_loading_completion(
    mut loading_complete_events: EventWriter<super::events::PluginLoadingComplete>,
    progress: Res<PluginLoadingProgress>,
    mut completion_sent: Local<bool>,
) {
    // Only send completion event once
    if progress.loading_complete && progress.total_plugins > 0 && !*completion_sent {
        loading_complete_events.write(super::events::PluginLoadingComplete {
            loaded_count: progress.loaded_plugins,
            failed_count: progress.failed_plugins,
        });

        *completion_sent = true;

        log::info!(
            "Plugin loading complete: {} loaded, {} failed",
            progress.loaded_plugins,
            progress.failed_plugins
        );
    }
}

/// System to log plugin loading progress
pub fn log_plugin_loading_progress(
    progress: Res<PluginLoadingProgress>,
    mut last_progress: Local<f32>,
) {
    let current_progress = progress.progress_percentage();

    // Log progress every 10%
    if (current_progress - *last_progress).abs() >= 10.0 {
        log::info!(
            "Plugin loading progress: {:.1}% ({}/{} loaded, {} failed)",
            current_progress,
            progress.loaded_plugins,
            progress.total_plugins,
            progress.failed_plugins
        );
        *last_progress = current_progress;
    }
}

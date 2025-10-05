use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::asset::{LoadState, RecursiveDependencyLoadState, UntypedAssetId};
use std::collections::HashSet;

use crate::prelude::*;

/// System set for asset progress tracking
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetProgressSet;

/// Resource for tracking asset loading progress
///
/// This resource automatically tracks assets you register with it and emits
/// progress events as they load. It integrates seamlessly with the main
/// progress tracking system.
///
/// # Example
///
/// ```rust
/// # use bevy::prelude::*;
/// # use action_items_ecs_progress::prelude::*;
/// #
/// # #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// # enum GameState { #[default] Loading }
/// #
/// fn load_assets(
///     mut tracker: ResMut<AssetsTracker<GameState>>,
///     asset_server: Res<AssetServer>,
/// ) {
///     // Register assets to track
///     let texture_handle = asset_server.load("textures/player.png");
///     tracker.track(texture_handle.id());
///     
///     let sound_handle = asset_server.load("sounds/shoot.ogg");
///     tracker.track(sound_handle.id());
/// }
/// ```
#[derive(Resource)]
pub struct AssetsTracker<S: States> {
    /// Assets currently being loaded
    pending: HashSet<UntypedAssetId>,
    /// Assets that have finished loading (successfully or with errors if
    /// ignored)
    completed: HashSet<UntypedAssetId>,
    /// Assets that failed to load and are not ignored
    failed: HashSet<UntypedAssetId>,
    /// Whether to ignore loading errors and count failed assets as complete
    pub ignore_errors: bool,
    /// Whether to include recursive dependencies when checking load state
    pub include_dependencies: bool,
    _phantom: PhantomData<S>,
}

impl<S: States> Default for AssetsTracker<S> {
    fn default() -> Self {
        Self {
            pending: HashSet::default(),
            completed: HashSet::default(),
            failed: HashSet::default(),
            ignore_errors: true,
            include_dependencies: true,
            _phantom: PhantomData,
        }
    }
}

impl<S: States> AssetsTracker<S> {
    /// Create a new asset tracker with default settings
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure error handling
    #[allow(dead_code)]
    pub fn with_error_handling(mut self, ignore_errors: bool) -> Self {
        self.ignore_errors = ignore_errors;
        self
    }

    /// Configure dependency tracking
    #[allow(dead_code)]
    pub fn with_dependencies(mut self, include_dependencies: bool) -> Self {
        self.include_dependencies = include_dependencies;
        self
    }

    /// Register an asset to be tracked
    #[allow(dead_code)]
    pub fn track<A: Asset>(&mut self, handle: &Handle<A>) {
        self.track_untyped(handle.id().into());
    }

    /// Register an untyped asset to be tracked
    #[allow(dead_code)]
    pub fn track_untyped(&mut self, asset_id: UntypedAssetId) {
        if !self.completed.contains(&asset_id)
            && !self.failed.contains(&asset_id)
        {
            self.pending.insert(asset_id);
        }
    }

    /// Track multiple assets at once
    #[allow(dead_code)]
    pub fn track_many<A: Asset, I: IntoIterator<Item = Handle<A>>>(
        &mut self,
        handles: I,
    ) {
        for handle in handles {
            self.track(&handle);
        }
    }

    /// Get current progress as a Progress struct
    pub fn get_progress(&self) -> Progress {
        Progress {
            done: self.completed.len() as u32,
            total: (self.pending.len()
                + self.completed.len()
                + self.failed.len()) as u32,
        }
    }

    /// Check if all tracked assets are loaded
    pub fn is_complete(&self) -> bool {
        self.pending.is_empty()
            && (self.ignore_errors || self.failed.is_empty())
    }

    /// Get the number of assets still loading
    #[allow(dead_code)]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get the number of successfully loaded assets
    #[allow(dead_code)]
    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    /// Get the number of failed assets
    #[allow(dead_code)]
    pub fn failed_count(&self) -> usize {
        self.failed.len()
    }

    /// Clear all tracking data
    pub fn clear(&mut self) {
        self.pending.clear();
        self.completed.clear();
        self.failed.clear();
    }

    /// Remove a specific asset from tracking
    #[allow(dead_code)]
    pub fn untrack(&mut self, asset_id: UntypedAssetId) {
        self.pending.remove(&asset_id);
        self.completed.remove(&asset_id);
        self.failed.remove(&asset_id);
    }
}

/// System that tracks asset loading progress and emits progress events
///
/// This system runs automatically when asset tracking is enabled. It checks
/// the loading state of all registered assets and updates progress accordingly.
pub fn track_assets_progress<S: States>(
    mut tracker: ResMut<AssetsTracker<S>>,
    asset_server: Res<AssetServer>,
    mut progress_writer: EventWriter<Progress>,
    mut completion_writer: EventWriter<ProgressComplete<S>>,
    state: Res<State<S>>,
) {
    let mut any_changed = false;

    // Check each pending asset
    let pending_assets: Vec<_> = tracker.pending.iter().cloned().collect();
    for asset_id in pending_assets {
        let finished = if tracker.include_dependencies {
            let state = asset_server.recursive_dependency_load_state(asset_id);
            match state {
                RecursiveDependencyLoadState::Loaded => {
                    tracker.pending.remove(&asset_id);
                    tracker.completed.insert(asset_id);
                    any_changed = true;
                    true
                }
                RecursiveDependencyLoadState::Failed(_) => {
                    tracker.pending.remove(&asset_id);
                    if tracker.ignore_errors {
                        tracker.completed.insert(asset_id);
                    } else {
                        tracker.failed.insert(asset_id);
                    }
                    any_changed = true;
                    true
                }
                RecursiveDependencyLoadState::Loading => false,
                RecursiveDependencyLoadState::NotLoaded => false,
            }
        } else {
            let state = asset_server.load_state(asset_id);
            match state {
                LoadState::Loaded => {
                    tracker.pending.remove(&asset_id);
                    tracker.completed.insert(asset_id);
                    any_changed = true;
                    true
                }
                LoadState::Failed(ref _error) => {
                    tracker.pending.remove(&asset_id);
                    if tracker.ignore_errors {
                        tracker.completed.insert(asset_id);
                    } else {
                        tracker.failed.insert(asset_id);
                    }
                    any_changed = true;
                    true
                }
                LoadState::Loading => false,
                LoadState::NotLoaded => false,
            }
        };

        if finished {
            #[cfg(feature = "debug")]
            {
                let status = if tracker.completed.contains(&asset_id) {
                    "completed"
                } else {
                    "failed"
                };
                debug!("Asset {:?} loading {}", asset_id, status);
            }
        }
    }

    if any_changed {
        let progress = tracker.get_progress();
        progress_writer.write(progress);

        #[cfg(feature = "debug")]
        debug!(
            "Asset progress: {}/{} ({:.1}%)",
            progress.done,
            progress.total,
            progress.fraction() * 100.0
        );

        // Check for completion
        if tracker.is_complete() {
            completion_writer.write(ProgressComplete {
                state: state.get().clone(),
            });

            #[cfg(feature = "debug")]
            info!("All assets loaded for state: {:?}", state.get());
        }
    }
}

/// System to clear asset tracking data
pub fn clear_assets_tracker<S: States>(mut tracker: ResMut<AssetsTracker<S>>) {
    tracker.clear();

    #[cfg(feature = "debug")]
    debug!("Asset tracker data cleared");
}

/// Extension trait for easy asset tracking
#[allow(dead_code)]
pub trait TrackAssets<S: States> {
    /// Track this asset for the given state
    fn track_for_state(self, tracker: &mut AssetsTracker<S>) -> Self;
}

impl<S: States, A: Asset> TrackAssets<S> for Handle<A> {
    fn track_for_state(self, tracker: &mut AssetsTracker<S>) -> Self {
        tracker.track(&self);
        self
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;

    #[derive(Asset, TypePath)]
    struct TestAsset;

    #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestState {
        #[default]
        Loading,
    }

    #[test]
    fn test_asset_tracker_creation() {
        let tracker = AssetsTracker::<TestState>::new()
            .with_error_handling(false)
            .with_dependencies(false);

        assert!(!tracker.ignore_errors);
        assert!(!tracker.include_dependencies);
        assert_eq!(tracker.pending_count(), 0);
        assert_eq!(tracker.completed_count(), 0);
        assert!(tracker.is_complete());
    }

    #[test]
    fn test_asset_tracking() {
        let mut tracker = AssetsTracker::<TestState>::new();
        let asset_id = AssetId::<TestAsset>::default().untyped();

        tracker.track_untyped(asset_id);
        assert_eq!(tracker.pending_count(), 1);
        assert!(!tracker.is_complete());

        let progress = tracker.get_progress();
        assert_eq!(progress.done, 0);
        assert_eq!(progress.total, 1);
    }

    #[test]
    fn test_progress_calculation() {
        let mut tracker = AssetsTracker::<TestState>::new();

        // Track 3 assets
        for _i in 0..3 {
            let asset_id =
                AssetId::<TestAsset>::invalid()
                    .untyped();
            tracker.track_untyped(asset_id);
        }

        // Complete 1 asset
        let first_asset =
            AssetId::<TestAsset>::invalid()
                .untyped();
        tracker.pending.remove(&first_asset);
        tracker.completed.insert(first_asset);

        let progress = tracker.get_progress();
        assert_eq!(progress.done, 1);
        assert_eq!(progress.total, 3);
        assert_eq!(progress.fraction(), 1.0 / 3.0);
    }
}

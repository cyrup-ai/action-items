use std::marker::PhantomData;

use bevy::prelude::*;

use crate::prelude::*;

/// Component for storing progress data on entities
///
/// This allows you to track progress by attaching components to entities rather
/// than using system parameters or return values. The plugin will automatically
/// sum up all entity progress and include it in the total progress calculation.
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
/// fn spawn_loading_tasks(mut commands: Commands) {
///     // Spawn entities with progress components
///     commands.spawn((
///         ProgressEntity::<GameState>::new()
///             .with_progress(0, 100)  // 0/100 visible progress
///             .with_hidden_progress(0, 3), // 0/3 hidden progress
///         Name::new("Asset Loading Task"),
///     ));
///     
///     commands.spawn((
///         ProgressEntity::<GameState>::new()
///             .with_progress(0, 50),  // 0/50 visible progress
///         Name::new("World Generation Task"),
///     ));
/// }
///
/// fn update_loading_task(
///     mut query: Query<&mut ProgressEntity<GameState>, With<Name>>,
/// ) {
///     for mut progress in query.iter_mut() {
///         // Update progress as tasks complete
///         progress.visible.done += 1;
///         progress.hidden.done += 1;
///     }
/// }
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct ProgressEntity<S: States> {
    /// Visible progress (shown in UI)
    pub visible: Progress,
    /// Hidden progress (required for completion but not displayed)
    pub hidden: HiddenProgress,
    _phantom: PhantomData<S>,
}

impl<S: States> Default for ProgressEntity<S> {
    fn default() -> Self {
        Self {
            visible: Progress::default(),
            hidden: HiddenProgress::default(),
            _phantom: PhantomData,
        }
    }
}

impl<S: States> ProgressEntity<S> {
    /// Create a new progress entity with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the visible progress using builder pattern
    pub fn with_progress(mut self, done: u32, total: u32) -> Self {
        self.visible = Progress { done, total };
        self
    }

    /// Set the hidden progress using builder pattern  
    pub fn with_hidden_progress(mut self, done: u32, total: u32) -> Self {
        self.hidden = HiddenProgress(Progress { done, total });
        self
    }

    /// Set both visible and hidden progress
    pub fn with_both_progress(
        mut self,
        visible: (u32, u32),
        hidden: (u32, u32),
    ) -> Self {
        self.visible = Progress {
            done: visible.0,
            total: visible.1,
        };
        self.hidden = HiddenProgress(Progress {
            done: hidden.0,
            total: hidden.1,
        });
        self
    }

    /// Check if this entity's progress is complete (both visible and hidden)
    pub fn is_complete(&self) -> bool {
        self.visible.is_complete() && self.hidden.is_complete()
    }

    /// Get the combined progress fraction (0.0 to 1.0)
    pub fn combined_fraction(&self) -> f32 {
        let total_done = self.visible.done + self.hidden.done;
        let total_expected = self.visible.total + self.hidden.total;

        if total_expected == 0 {
            1.0
        } else {
            (total_done as f32 / total_expected as f32).min(1.0)
        }
    }

    /// Update visible progress
    pub fn update_visible(&mut self, done: u32, total: u32) {
        self.visible = Progress { done, total };
    }

    /// Update hidden progress
    pub fn update_hidden(&mut self, done: u32, total: u32) {
        self.hidden = HiddenProgress(Progress { done, total });
    }

    /// Add to visible progress
    pub fn add_visible_progress(&mut self, done: u32, total: u32) {
        self.visible.done += done;
        self.visible.total += total;
    }

    /// Add to hidden progress
    pub fn add_hidden_progress(&mut self, done: u32, total: u32) {
        self.hidden.done += done;
        self.hidden.total += total;
    }

    /// Mark visible progress as complete
    pub fn complete_visible(&mut self) {
        self.visible.done = self.visible.total;
    }

    /// Mark hidden progress as complete
    pub fn complete_hidden(&mut self) {
        self.hidden.done = self.hidden.total;
    }

    /// Mark all progress as complete
    pub fn complete_all(&mut self) {
        self.complete_visible();
        self.complete_hidden();
    }
}

/// System that sums up all entity progress and updates the monitor
///
/// This system runs automatically when the plugin is added and entities with
/// `ProgressEntity<S>` components exist. It computes the sum of all entity
/// progress and stores it in the `ProgressMonitor` resource.
pub fn sum_entity_progress<S: States>(
    query: Query<&ProgressEntity<S>>,
    mut monitor: ResMut<ProgressMonitor<S>>,
) {
    let (visible_sum, hidden_sum) = query.iter().fold(
        (Progress::default(), HiddenProgress::default()),
        |(vis_acc, hid_acc), entity| {
            (vis_acc + entity.visible, hid_acc + entity.hidden)
        },
    );

    monitor.set_entity_sum(visible_sum, hidden_sum);
}

/// Query helper for finding incomplete progress entities
pub type IncompleteProgressQuery<'w, 's, S> = Query<
    'w,
    's,
    (Entity, &'static mut ProgressEntity<S>),
    Without<CompleteProgress>,
>;

/// Query helper for finding complete progress entities  
pub type CompleteProgressQuery<'w, 's, S> =
    Query<'w, 's, (Entity, &'static ProgressEntity<S>), With<CompleteProgress>>;

/// Marker component for entities that have completed their progress
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompleteProgress;

/// System to add CompleteProgress marker to finished entities
pub fn mark_complete_entities<S: States>(
    mut commands: Commands,
    query: Query<(Entity, &ProgressEntity<S>), Without<CompleteProgress>>,
) {
    for (entity, progress) in query.iter() {
        if progress.is_complete() {
            commands.entity(entity).insert(CompleteProgress);
        }
    }
}

/// System to remove CompleteProgress marker from entities that are no longer
/// complete
pub fn unmark_incomplete_entities<S: States>(
    mut commands: Commands,
    query: Query<(Entity, &ProgressEntity<S>), With<CompleteProgress>>,
) {
    for (entity, progress) in query.iter() {
        if !progress.is_complete() {
            commands.entity(entity).remove::<CompleteProgress>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestState {
        #[default]
        Loading,
    }

    #[test]
    fn test_progress_entity_creation() {
        let entity = ProgressEntity::<TestState>::new()
            .with_progress(5, 10)
            .with_hidden_progress(2, 3);

        assert_eq!(entity.visible.done, 5);
        assert_eq!(entity.visible.total, 10);
        assert_eq!(entity.hidden.done, 2);
        assert_eq!(entity.hidden.total, 3);
        assert!(!entity.is_complete());
    }

    #[test]
    fn test_progress_entity_completion() {
        let mut entity = ProgressEntity::<TestState>::new()
            .with_progress(10, 10)
            .with_hidden_progress(3, 3);

        assert!(entity.is_complete());

        entity.visible.done = 5;
        assert!(!entity.is_complete());

        entity.complete_all();
        assert!(entity.is_complete());
    }

    #[test]
    fn test_combined_fraction() {
        let entity = ProgressEntity::<TestState>::new()
            .with_progress(5, 10) // 50% visible
            .with_hidden_progress(1, 2); // 50% hidden

        // (5 + 1) / (10 + 2) = 6/12 = 0.5
        assert_eq!(entity.combined_fraction(), 0.5);
    }
}

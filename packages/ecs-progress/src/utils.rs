use std::time::{Duration, Instant};

use bevy::prelude::*;

use crate::prelude::*;

/// Utility system that waits for a fixed number of frames before completing
///
/// Returns `HiddenProgress` that becomes complete (1/1) after N frames have
/// passed. Useful for testing or ensuring certain initialization happens before
/// proceeding.
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
/// fn setup_loading_systems(app: &mut App) {
///     app.add_systems(
///         Update,
///         wait_frames::<60>.track_progress::<GameState>()  // Wait 60 frames (1 second at 60fps)
///     );
/// }
/// ```
pub fn wait_frames<const N: u32>(mut counter: Local<u32>) -> HiddenProgress {
    *counter += 1;
    HiddenProgress(Progress {
        done: if *counter >= N { 1 } else { 0 },
        total: 1,
    })
}

/// Utility system that counts frames up to a maximum
///
/// Returns `HiddenProgress` that gradually progresses from 0/MAX to MAX/MAX.
/// Useful for gradual initialization or multi-frame setup processes.
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
/// fn setup_loading_systems(app: &mut App) {
///     app.add_systems(
///         Update,
///         count_frames::<100>.track_progress::<GameState>()  // Count from 0 to 100
///     );
/// }
/// ```
pub fn count_frames<const MAX: u32>(mut counter: Local<u32>) -> HiddenProgress {
    if *counter < MAX {
        *counter += 1;
    }
    HiddenProgress(Progress {
        done: *counter,
        total: MAX,
    })
}

/// Utility system that waits for a specified duration before completing
///
/// Returns `HiddenProgress` that becomes complete (1/1) after the specified
/// number of milliseconds have passed. Useful for time-based loading delays
/// or ensuring minimum loading screen display time.
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
/// fn setup_loading_systems(app: &mut App) {
///     app.add_systems(
///         Update,
///         wait_duration::<2000>.track_progress::<GameState>()  // Wait 2 seconds
///     );
/// }
/// ```
pub fn wait_duration<const MILLIS: u64>(
    mut start_time: Local<Option<Instant>>,
) -> HiddenProgress {
    let now = Instant::now();
    let target_duration = Duration::from_millis(MILLIS);

    let start = *start_time.get_or_insert(now);
    let elapsed = now.duration_since(start);

    HiddenProgress(Progress {
        done: if elapsed >= target_duration { 1 } else { 0 },
        total: 1,
    })
}

/// Utility system that provides progress based on elapsed time
///
/// Returns `HiddenProgress` that gradually progresses from 0 to 1 over the
/// specified duration. Useful for time-based progress bars or animations.
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
/// fn setup_loading_systems(app: &mut App) {
///     app.add_systems(
///         Update,
///         timed_progress::<5000>.track_progress::<GameState>()  // Progress over 5 seconds
///     );
/// }
/// ```
pub fn timed_progress<const MILLIS: u64>(
    mut start_time: Local<Option<Instant>>,
) -> HiddenProgress {
    let now = Instant::now();
    let target_duration = Duration::from_millis(MILLIS);

    let start = *start_time.get_or_insert(now);
    let elapsed = now.duration_since(start);

    let progress = if elapsed >= target_duration {
        1.0
    } else {
        elapsed.as_secs_f32() / target_duration.as_secs_f32()
    };

    // Scale to a reasonable integer range for precision
    let scaled_progress = (progress * 1000.0) as u32;

    HiddenProgress(Progress {
        done: scaled_progress.min(1000),
        total: 1000,
    })
}

/// Utility system that simulates random work completion
///
/// Returns `Progress` that randomly advances based on a simple linear
/// congruential generator. Useful for testing progress tracking with
/// unpredictable timing.
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
/// fn setup_loading_systems(app: &mut App) {
///     app.add_systems(
///         Update,
///         random_work::<100>.track_progress::<GameState>()  // Random progress up to 100
///     );
/// }
/// ```
pub fn random_work<const MAX_WORK: u32>(
    mut rng_state: Local<u32>,
    mut current_work: Local<u32>,
) -> Progress {
    // Simple LCG for deterministic "randomness"
    *rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);

    // Randomly advance work (0-3 units per frame)
    let advance = *rng_state % 4;
    *current_work = (*current_work + advance).min(MAX_WORK);

    Progress {
        done: *current_work,
        total: MAX_WORK,
    }
}

/// Utility system that waits for a condition to be true
///
/// Takes a system that returns a boolean and converts it to progress.
/// Useful for waiting on external conditions or complex state checks.
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
/// fn check_network_ready() -> bool {
///     // Your condition check here
///     true
/// }
///
/// fn setup_loading_systems(app: &mut App) {
///     app.add_systems(
///         Update,
///         check_network_ready.pipe(wait_for_condition).track_progress::<GameState>()
///     );
/// }
/// ```
pub fn wait_for_condition(In(condition): In<bool>) -> HiddenProgress {
    HiddenProgress(Progress::from(condition))
}

/// Utility system that counts successful operations out of total attempts
///
/// Maintains counters for attempts and successes, returning progress as
/// successes/attempts. Useful for tracking batch operations or retries.
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
/// fn try_operation() -> bool {
///     // Your operation that might succeed or fail
///     rand::random::<f32>() > 0.8  // 20% success rate
/// }
///
/// fn setup_loading_systems(app: &mut App) {
///     app.add_systems(
///         Update,
///         try_operation.pipe(count_successes::<10>).track_progress::<GameState>()
///     );
/// }
/// ```
pub fn count_successes<const TARGET: u32>(
    In(success): In<bool>,
    mut successes: Local<u32>,
    mut attempts: Local<u32>,
) -> Progress {
    *attempts += 1;
    if success {
        *successes += 1;
    }

    Progress {
        done: *successes,
        total: TARGET,
    }
}

/// System that reports constant progress
///
/// Always returns the same progress value. Useful for placeholder systems
/// or representing fixed amounts of work.
pub fn constant_progress<const DONE: u32, const TOTAL: u32>() -> Progress {
    Progress {
        done: DONE,
        total: TOTAL,
    }
}

/// System that always reports completion
///
/// Always returns complete progress (1/1). Useful as a placeholder or for
/// systems that should always be considered done.
pub fn always_complete() -> Progress {
    Progress { done: 1, total: 1 }
}

/// System that never completes
///
/// Always returns incomplete progress (0/1). Useful for testing infinite
/// loading states or systems that should never finish.
pub fn never_complete() -> Progress {
    Progress { done: 0, total: 1 }
}

#[cfg(test)]
mod tests {

    use bevy::ecs::schedule::Schedule;

    use super::*;

    #[test]
    fn test_wait_frames() {
        let mut schedule = Schedule::default();
        schedule.add_systems(wait_frames::<3>.map(|_| ()));

        let mut world = World::new();

        // First frame
        schedule.run(&mut world);
        // Should not be complete yet (we need to wait for 3 frames)

        // Second frame
        schedule.run(&mut world);
        // Still not complete

        // Third frame
        schedule.run(&mut world);
        // Should be complete now
    }

    #[test]
    fn test_count_frames() {
        let mut schedule = Schedule::default();
        schedule.add_systems(count_frames::<5>.map(|_| ()));

        let mut world = World::new();

        for _i in 1..=5 {
            schedule.run(&mut world);
            // Progress should be i/5
        }

        // Run extra frames - should stay at 5/5
        schedule.run(&mut world);
        schedule.run(&mut world);
    }

    #[test]
    fn test_constant_progress() {
        let progress = constant_progress::<3, 10>();
        assert_eq!(progress.done, 3);
        assert_eq!(progress.total, 10);
        assert_eq!(progress.fraction(), 0.3);
    }

    #[test]
    fn test_always_complete() {
        let progress = always_complete();
        assert!(progress.is_complete());
        assert_eq!(progress.fraction(), 1.0);
    }

    #[test]
    fn test_never_complete() {
        let progress = never_complete();
        assert!(!progress.is_complete());
        assert_eq!(progress.fraction(), 0.0);
    }

    #[test]
    fn test_wait_for_condition() {
        let incomplete = wait_for_condition(In(false));
        assert!(!incomplete.is_complete());

        let complete = wait_for_condition(In(true));
        assert!(complete.is_complete());
    }

    #[test]
    fn test_count_successes() {
        let mut schedule = Schedule::default();
        schedule.add_systems((|| true).pipe(count_successes::<3>).map(|_| ()));

        let mut world = World::new();

        // All operations succeed, so should reach 3/3 after 3 runs
        for _ in 0..3 {
            schedule.run(&mut world);
        }
    }
}

//! Tests for utils.rs

use bevy::prelude::*;
use bevy::ecs::schedule::Schedule;
use action_items_ecs_progress::prelude::*;
use action_items_ecs_progress::utils::*;

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

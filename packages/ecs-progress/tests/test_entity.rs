//! Tests for entity.rs

use bevy::prelude::*;
use action_items_ecs_progress::entity::ProgressEntity;

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

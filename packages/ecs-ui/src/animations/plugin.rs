//! Animation plugin with system registration for UI transitions

use bevy::prelude::*;
use super::state::{AnimationState, AnimationSequence};
use super::window::{WindowAnimation, ListAnimation, ItemAnimation};

/// Plugin providing complete animation system for UI transitions
#[derive(Debug, Default, Clone)]
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Update systems run every frame
            .add_systems(
                Update,
                (
                    update_animation_states,
                    update_animation_sequences,
                    update_window_animations,
                    update_list_animations,
                    update_item_animations,
                ),
            )
            // Cleanup systems run after updates
            .add_systems(
                PostUpdate,
                (
                    cleanup_completed_animations,
                    cleanup_completed_sequences,
                    cleanup_completed_window_animations,
                    cleanup_completed_list_animations,
                    cleanup_completed_item_animations,
                ),
            );
    }
}

// === UPDATE SYSTEMS ===

/// Update all animation states
#[inline]
fn update_animation_states(
    time: Res<Time>,
    mut query: Query<&mut AnimationState>,
) {
    let delta_time = time.delta_secs();
    for mut anim in &mut query {
        anim.update(delta_time);
    }
}

/// Update all animation sequences
#[inline]
fn update_animation_sequences(
    time: Res<Time>,
    mut query: Query<&mut AnimationSequence>,
) {
    let delta_time = time.delta_secs();
    for mut sequence in &mut query {
        sequence.update(delta_time);
    }
}

/// Update window animations (opacity + scale)
#[inline]
fn update_window_animations(
    time: Res<Time>,
    mut query: Query<(
        &mut WindowAnimation,
        &mut Transform,
        &mut BackgroundColor,
    )>,
) {
    let delta_time = time.delta_secs();
    
    for (mut anim, mut transform, mut bg_color) in &mut query {
        let (opacity, scale) = anim.update(delta_time);
        
        transform.scale = scale;
        
        let mut color = bg_color.0;
        color = color.with_alpha(opacity);
        *bg_color = BackgroundColor(color);
    }
}

/// Update list animations and initialize item animations
#[inline]
fn update_list_animations(
    time: Res<Time>,
    mut commands: Commands,
    mut list_query: Query<(Entity, &mut ListAnimation, &Children), Changed<ListAnimation>>,
    children_query: Query<Entity, With<ChildOf>>,
) {
    let delta_time = time.delta_secs();
    
    for (_list_entity, mut list_anim, children) in &mut list_query {
        // Initialize item animations on first update
        if !list_anim.initialized {
            for (index, child_entity) in children.iter().enumerate() {
                if children_query.contains(child_entity) {
                    let delay = list_anim.delay_for_item(index);
                    let item_anim = ItemAnimation::new(
                        index,
                        delay,
                        list_anim.base_animation.duration,
                        list_anim.base_animation.easing,
                    );
                    commands.entity(child_entity).insert(item_anim);
                }
            }
            list_anim.initialized = true;
        }
        
        // Update base animation time
        list_anim.base_animation.update(delta_time);
    }
}

/// Update individual item animations
#[inline]
fn update_item_animations(
    time: Res<Time>,
    list_query: Query<&ListAnimation>,
    mut item_query: Query<(
        &ChildOf,
        &mut ItemAnimation,
        &mut Transform,
        &mut BackgroundColor,
    )>,
) {
    let delta_time = time.delta_secs();
    
    for (parent, mut item_anim, mut transform, mut bg_color) in &mut item_query {
        // Get the list animation to know elapsed time
        if let Ok(list_anim) = list_query.get(parent.parent()) {
            let elapsed_time = list_anim.base_animation.current_time;
            
            let (opacity, scale, translation) = item_anim.update(
                delta_time,
                elapsed_time,
            );
            
            transform.scale = scale;
            transform.translation = translation;
            
            let mut color = bg_color.0;
            color = color.with_alpha(opacity);
            *bg_color = BackgroundColor(color);
        }
    }
}

// === CLEANUP SYSTEMS ===

/// Remove completed standalone animation states
fn cleanup_completed_animations(
    mut commands: Commands,
    query: Query<(Entity, &AnimationState), Without<AnimationSequence>>,
) {
    for (entity, anim) in &query {
        if anim.is_complete() {
            commands.entity(entity).remove::<AnimationState>();
        }
    }
}

/// Remove completed animation sequences
fn cleanup_completed_sequences(
    mut commands: Commands,
    query: Query<(Entity, &AnimationSequence)>,
) {
    for (entity, sequence) in &query {
        if sequence.is_complete() {
            commands.entity(entity).remove::<AnimationSequence>();
        }
    }
}

/// Remove completed window animations
fn cleanup_completed_window_animations(
    mut commands: Commands,
    query: Query<(Entity, &WindowAnimation)>,
) {
    for (entity, anim) in &query {
        if anim.is_complete() {
            commands.entity(entity).remove::<WindowAnimation>();
        }
    }
}

/// Remove completed list animations
fn cleanup_completed_list_animations(
    mut commands: Commands,
    query: Query<(Entity, &ListAnimation)>,
) {
    for (entity, list_anim) in &query {
        let elapsed_time = list_anim.base_animation.current_time;
        if list_anim.is_complete(elapsed_time) {
            commands.entity(entity).remove::<ListAnimation>();
        }
    }
}

/// Remove completed item animations
fn cleanup_completed_item_animations(
    mut commands: Commands,
    list_query: Query<&ListAnimation>,
    item_query: Query<(Entity, &ChildOf, &ItemAnimation)>,
) {
    for (entity, parent, item_anim) in &item_query {
        if let Ok(list_anim) = list_query.get(parent.parent()) {
            let elapsed_time = list_anim.base_animation.current_time;
            if item_anim.is_complete(elapsed_time) {
                commands.entity(entity).remove::<ItemAnimation>();
            }
        }
    }
}

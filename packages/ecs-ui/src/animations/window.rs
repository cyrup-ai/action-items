//! Window and list animation components for UI transitions

use bevy::prelude::*;
use super::state::AnimationState;
use super::easing::EasingFunction;

/// Window animation component for smooth UI transitions
#[derive(Component, Debug, Clone)]
pub struct WindowAnimation {
    pub opacity_animation: AnimationState,
    pub scale_animation: AnimationState,
    pub initial_opacity: f32,
    pub target_opacity: f32,
    pub initial_scale: Vec3,
    pub target_scale: Vec3,
}

impl WindowAnimation {
    /// Create window animation with professional timing
    pub fn new(
        opacity_duration: f32,
        scale_duration: f32,
        target_opacity: f32,
        target_scale: Vec3,
    ) -> Self {
        Self {
            opacity_animation: AnimationState::new(opacity_duration, EasingFunction::EaseOut),
            scale_animation: AnimationState::new(scale_duration, EasingFunction::EaseOutBack),
            initial_opacity: 0.0,
            target_opacity,
            initial_scale: Vec3::splat(0.8),
            target_scale,
        }
    }

    /// Create window animation with custom initial values
    pub fn with_initial_values(
        opacity_duration: f32,
        scale_duration: f32,
        initial_opacity: f32,
        target_opacity: f32,
        initial_scale: Vec3,
        target_scale: Vec3,
    ) -> Self {
        Self {
            opacity_animation: AnimationState::new(opacity_duration, EasingFunction::EaseOut),
            scale_animation: AnimationState::new(scale_duration, EasingFunction::EaseOutBack),
            initial_opacity,
            target_opacity,
            initial_scale,
            target_scale,
        }
    }

    /// Update animations and return current values
    #[inline]
    pub fn update(&mut self, delta_time: f32) -> (f32, Vec3) {
        let _opacity_complete = self.opacity_animation.update(delta_time);
        let _scale_complete = self.scale_animation.update(delta_time);

        let opacity = lerp(
            self.initial_opacity,
            self.target_opacity,
            self.opacity_animation.progress(),
        );

        let scale = self
            .initial_scale
            .lerp(self.target_scale, self.scale_animation.progress());

        (opacity, scale)
    }

    /// Check if all animations are complete
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.opacity_animation.is_complete() && self.scale_animation.is_complete()
    }

    /// Create a fade-in animation
    pub fn fade_in(duration: f32) -> Self {
        Self::with_initial_values(
            duration,
            duration,
            0.0,
            1.0,
            Vec3::splat(0.8),
            Vec3::ONE,
        )
    }

    /// Create a fade-out animation
    pub fn fade_out(duration: f32) -> Self {
        Self::with_initial_values(
            duration,
            duration,
            1.0,
            0.0,
            Vec3::ONE,
            Vec3::splat(0.8),
        )
    }
}

/// List animation component for staggered reveal effects
#[derive(Component, Debug, Clone)]
pub struct ListAnimation {
    /// Base animation state for all items
    pub base_animation: AnimationState,
    /// Delay between each item's animation start
    pub stagger_delay: f32,
    /// Total number of items in the list
    pub item_count: usize,
    /// Whether animation has been initialized
    pub initialized: bool,
}

impl ListAnimation {
    /// Create a new list animation with staggered timing
    pub fn new(duration: f32, stagger_delay: f32, item_count: usize) -> Self {
        Self {
            base_animation: AnimationState::new(duration, EasingFunction::EaseOut),
            stagger_delay,
            item_count,
            initialized: false,
        }
    }

    /// Calculate the delay offset for an item at given index
    #[inline]
    pub fn delay_for_item(&self, index: usize) -> f32 {
        index as f32 * self.stagger_delay
    }

    /// Check if all items have completed animation
    pub fn is_complete(&self, current_time: f32) -> bool {
        let last_item_delay = self.delay_for_item(self.item_count.saturating_sub(1));
        current_time >= (last_item_delay + self.base_animation.duration)
    }
}

/// Individual item animation within a list
#[derive(Component, Debug, Clone)]
pub struct ItemAnimation {
    /// Index of this item in the list
    pub index: usize,
    /// Delay before this item's animation starts
    pub delay_offset: f32,
    /// Animation state for this item
    pub animation: AnimationState,
    /// Initial transform values
    pub initial_opacity: f32,
    pub initial_scale: Vec3,
    pub initial_translation: Vec3,
    /// Target transform values
    pub target_opacity: f32,
    pub target_scale: Vec3,
    pub target_translation: Vec3,
}

impl ItemAnimation {
    /// Create a new item animation with delay
    pub fn new(
        index: usize,
        delay_offset: f32,
        duration: f32,
        easing: EasingFunction,
    ) -> Self {
        Self {
            index,
            delay_offset,
            animation: AnimationState::new(duration, easing),
            initial_opacity: 0.0,
            initial_scale: Vec3::splat(0.95),
            initial_translation: Vec3::new(0.0, -10.0, 0.0),
            target_opacity: 1.0,
            target_scale: Vec3::ONE,
            target_translation: Vec3::ZERO,
        }
    }

    /// Update animation with delay consideration
    #[inline]
    pub fn update(&mut self, _delta_time: f32, elapsed_time: f32) -> (f32, Vec3, Vec3) {
        // Only update if past the delay
        if elapsed_time < self.delay_offset {
            return (self.initial_opacity, self.initial_scale, self.initial_translation);
        }

        // Update the animation state
        let local_time = elapsed_time - self.delay_offset;
        self.animation.current_time = local_time;
        
        // Calculate progress
        let progress = self.animation.progress();

        // Interpolate values
        let opacity = lerp(self.initial_opacity, self.target_opacity, progress);
        let scale = self.initial_scale.lerp(self.target_scale, progress);
        let translation = self.initial_translation.lerp(self.target_translation, progress);

        (opacity, scale, translation)
    }

    /// Check if this item's animation is complete
    #[inline]
    pub fn is_complete(&self, elapsed_time: f32) -> bool {
        elapsed_time >= (self.delay_offset + self.animation.duration)
    }
}

/// High-performance linear interpolation
#[inline]
pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

//! Core animation state components for timing and sequencing

use bevy::prelude::*;
use std::collections::VecDeque;
use super::easing::EasingFunction;

/// Core animation state component with blazing-fast easing functions
#[derive(Component, Debug, Clone)]
pub struct AnimationState {
    pub current_time: f32,
    pub duration: f32,
    pub easing: EasingFunction,
    pub state: AnimationPlayState,
    pub loop_mode: AnimationLoop,
}

impl AnimationState {
    /// Create a new animation state with optimal defaults
    #[inline]
    pub fn new(duration: f32, easing: EasingFunction) -> Self {
        Self {
            current_time: 0.0,
            duration,
            easing,
            state: AnimationPlayState::Playing,
            loop_mode: AnimationLoop::Once,
        }
    }

    /// Get normalized progress (0.0 to 1.0) with easing applied
    #[inline]
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            return 1.0;
        }

        let linear_progress = (self.current_time / self.duration).clamp(0.0, 1.0);
        self.easing.apply(linear_progress)
    }

    /// Update animation time and return whether it's complete
    #[inline]
    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.state != AnimationPlayState::Playing {
            return self.is_complete();
        }

        self.current_time += delta_time;

        match self.loop_mode {
            AnimationLoop::Once => {
                if self.current_time >= self.duration {
                    self.current_time = self.duration;
                    self.state = AnimationPlayState::Finished;
                    true
                } else {
                    false
                }
            },
            AnimationLoop::Repeat => {
                if self.current_time >= self.duration {
                    self.current_time %= self.duration;
                }
                false
            },
            AnimationLoop::PingPong => {
                if self.current_time >= self.duration * 2.0 {
                    self.current_time %= self.duration * 2.0;
                }
                false
            },
        }
    }

    /// Check if animation is complete
    #[inline]
    pub fn is_complete(&self) -> bool {
        matches!(self.state, AnimationPlayState::Finished)
    }

    /// Reset animation to beginning
    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.state = AnimationPlayState::Playing;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        self.state = AnimationPlayState::Paused;
    }

    /// Resume the animation
    pub fn resume(&mut self) {
        if self.state == AnimationPlayState::Paused {
            self.state = AnimationPlayState::Playing;
        }
    }

    /// Set animation progress directly (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        self.current_time = progress.clamp(0.0, 1.0) * self.duration;
    }
}

/// Animation playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationPlayState {
    Playing,
    Paused,
    Finished,
}

/// Animation loop behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationLoop {
    Once,
    Repeat,
    PingPong,
}

/// Animation sequence for chaining multiple animations
#[derive(Component, Debug)]
pub struct AnimationSequence {
    pub animations: VecDeque<AnimationState>,
    pub current_index: usize,
}

impl AnimationSequence {
    /// Create a new animation sequence
    pub fn new() -> Self {
        Self {
            animations: VecDeque::new(),
            current_index: 0,
        }
    }

    /// Add an animation to the sequence
    pub fn add(&mut self, animation: AnimationState) {
        self.animations.push_back(animation);
    }

    /// Get the current animation
    pub fn current(&self) -> Option<&AnimationState> {
        self.animations.get(self.current_index)
    }

    /// Get the current animation mutably
    pub fn current_mut(&mut self) -> Option<&mut AnimationState> {
        self.animations.get_mut(self.current_index)
    }

    /// Update the sequence and return whether it's complete
    pub fn update(&mut self, delta_time: f32) -> bool {
        if let Some(current) = self.current_mut()
            && current.update(delta_time)
        {
            self.current_index += 1;
            if self.current_index >= self.animations.len() {
                return true; // Sequence complete
            }
        }
        false
    }

    /// Check if sequence is complete
    pub fn is_complete(&self) -> bool {
        self.current_index >= self.animations.len()
    }

    /// Reset sequence to beginning
    pub fn reset(&mut self) {
        self.current_index = 0;
        for animation in &mut self.animations {
            animation.reset();
        }
    }
}

impl Default for AnimationSequence {
    fn default() -> Self {
        Self::new()
    }
}

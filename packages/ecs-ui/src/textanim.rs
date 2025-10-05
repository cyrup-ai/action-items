#[cfg(feature = "animations")]
use std::hash::{DefaultHasher, Hash, Hasher};

#[cfg(feature = "animations")]
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};

use crate::*;

#[derive(Component, Reflect, Clone, PartialEq, Debug)]
enum DurationMode {
    CharSpeed(f32),
    AnimDuration(f32),
}

/// This component modifies attached [`Text2d`] with a modified string outputted from a time
/// dependant function.
#[derive(Component, Reflect, Clone, Debug)]
pub struct TextAnimator {
    string: String,
    function: fn(t: f32, text: &str, buffer: &mut String),
    counter: f32,
    mode: DurationMode,
    buffer: String,
    last_counter: f32,
}
impl Default for TextAnimator {
    fn default() -> Self {
        Self {
            string: String::new(),
            function: typing_animation_underscore,
            counter: 0.0,
            mode: DurationMode::AnimDuration(5.0),
            buffer: String::with_capacity(128),
            last_counter: 0.0,
        }
    }
}

/// Custom PartialEq implementation that excludes function pointer comparison
impl PartialEq for TextAnimator {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
            && self.counter == other.counter
            && self.mode == other.mode
            && self.buffer == other.buffer
            && self.last_counter == other.last_counter
        // Note: function pointer intentionally excluded from comparison
    }
}

impl TextAnimator {
    /// Creates new instance
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            string: text.into(),
            ..Default::default()
        }
    }
    /// Replace the text with a new one and resets the animation.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.string = text.into();
        self.counter = 0.0;
        self.last_counter = 0.0;
        self.buffer.clear();
    }
    /// Replace the text with a new one and skip to the end of the animation.
    /// This shows the text in its final animated state immediately.
    pub fn set_text_completed(&mut self, text: impl Into<String>) {
        self.string = text.into();
        self.buffer.clear();
        match self.mode {
            DurationMode::AnimDuration(duration) => {
                self.counter = duration;
                self.last_counter = duration;
            },
            DurationMode::CharSpeed(_) => {
                self.counter = 1.0 + self.string.trim().chars().count() as f32;
                self.last_counter = self.counter;
            },
        }
    }
    /// Replace the default function with a new one. The function provided takes time as input,
    /// original string, and a mutable buffer for output.
    pub fn function(mut self, function: fn(t: f32, text: &str, buffer: &mut String)) -> Self {
        self.function = function;
        self
    }
    /// Replace the default speed in seconds with a new one.
    pub fn speed(mut self, speed: f32) -> Self {
        self.mode = DurationMode::CharSpeed(speed);
        self
    }
    /// Replace the default duration in seconds with a new one.
    pub fn duration(mut self, duration: f32) -> Self {
        self.mode = DurationMode::AnimDuration(duration);
        self
    }
    /// This system takes care of updating the TextAnimator in time.
    pub(crate) fn system_2d(
        mut query: Query<(&mut Text2d, &mut TextAnimator)>,
        time: Res<Time>,
        mut commands: Commands,
    ) {
        for (mut text, mut animator) in &mut query {
            match animator.mode {
                DurationMode::CharSpeed(speed) => {
                    let chars = 1.0 + animator.string.trim().chars().count() as f32;
                    let modified = if animator.counter < chars {
                        animator.counter += time.delta_secs() * speed;
                        true
                    } else {
                        false
                    };
                    animator.counter = animator.counter.min(chars);

                    // Skip if no change and counter hasn't changed significantly
                    if !modified && animator.counter == animator.last_counter {
                        continue;
                    }

                    animator.buffer.clear();
                    let func = animator.function;
                    let counter = animator.counter;
                    let string = animator.string.clone();
                    func(counter / chars, &string, &mut animator.buffer);
                    if animator.buffer != text.as_str() {
                        text.0 = animator.buffer.clone();
                        commands.trigger(RecomputeUiLayout);
                    }
                    animator.last_counter = animator.counter;
                },
                DurationMode::AnimDuration(duration) => {
                    let modified = if animator.counter < duration {
                        animator.counter += time.delta_secs();
                        true
                    } else {
                        false
                    };
                    animator.counter = animator.counter.min(duration);

                    // Skip if no change and counter hasn't changed significantly
                    if !modified && animator.counter == animator.last_counter {
                        continue;
                    }

                    animator.buffer.clear();
                    let func = animator.function;
                    let counter = animator.counter;
                    let string = animator.string.clone();
                    func(counter / duration, &string, &mut animator.buffer);
                    if animator.buffer != text.as_str() {
                        text.0 = animator.buffer.clone();
                        commands.trigger(RecomputeUiLayout);
                    }
                    animator.last_counter = animator.counter;
                },
            }
        }
    }
    /// Updates Text3d content for animation using Conservative Single-Segment approach.
    ///
    /// This function safely animates Text3d with single String segments while preserving
    /// rich text styling for multi-segment text. Based on research-driven architecture
    /// that prioritizes data integrity and aligns with Bevy animation patterns.
    ///
    /// # Behavior
    /// - **Single String segment**: Animates text content directly (fast path)
    /// - **Multi-segment text**: Skips animation to preserve rich text styling
    /// - **Extract segments**: Skips animation to avoid conflicts with dynamic content
    ///
    /// # Returns
    /// - `true` if text content was updated (triggers UI layout recomputation)
    /// - `false` if no changes were made
    #[cfg(feature = "text3d")]
    fn update_text3d_content(text: &mut Text3d, new_content: &str) -> bool {
        // Single-segment fast path: animate simple text directly
        if let Some(current_text) = text.get_single() {
            if current_text != new_content {
                // Safe unwrap: if get_single() succeeds, get_single_mut() will too
                // This follows the bevy_rich_text3d API contract
                match text.get_single_mut() {
                    Some(text_value) => *text_value = new_content.to_string(),
                    None => {
                        warn!("Failed to get mutable text reference after successful immutable access");
                        return false;
                    }
                }
                return true;
            }
            return false;
        }

        // Multi-segment case: preserve rich text styling by skipping animation
        // This maintains colors, fonts, and Extract segments without data loss
        // Future enhancement: could implement style-preserving animation here
        false
    }

    /// This system takes care of updating the TextAnimator in time.
    #[cfg(feature = "text3d")]
    pub(crate) fn system_3d(
        mut query: Query<(&mut Text3d, &mut TextAnimator)>,
        time: Res<Time>,
        mut commands: Commands,
    ) {
        for (mut text, mut animator) in &mut query {
            match animator.mode {
                DurationMode::CharSpeed(speed) => {
                    let chars = 1.0 + animator.string.trim().chars().count() as f32;
                    let modified = if animator.counter < chars {
                        animator.counter += time.delta_secs() * speed;
                        true
                    } else {
                        false
                    };
                    animator.counter = animator.counter.min(chars);

                    if !modified && animator.counter == animator.last_counter {
                        continue;
                    }

                    animator.buffer.clear();
                    let func = animator.function;
                    let counter = animator.counter;
                    let string = animator.string.clone();
                    func(counter / chars, &string, &mut animator.buffer);

                    // Update Text3d using Conservative Single-Segment approach
                    if Self::update_text3d_content(&mut text, &animator.buffer) {
                        commands.trigger(RecomputeUiLayout);
                    }
                    animator.last_counter = animator.counter;
                },
                DurationMode::AnimDuration(duration) => {
                    let modified = if animator.counter < duration {
                        animator.counter += time.delta_secs();
                        true
                    } else {
                        false
                    };
                    animator.counter = animator.counter.min(duration);

                    if !modified && animator.counter == animator.last_counter {
                        continue;
                    }

                    animator.buffer.clear();
                    let func = animator.function;
                    let counter = animator.counter;
                    let string = animator.string.clone();
                    func(counter / duration, &string, &mut animator.buffer);

                    // Update Text3d using Conservative Single-Segment approach
                    if Self::update_text3d_content(&mut text, &animator.buffer) {
                        commands.trigger(RecomputeUiLayout);
                    }
                    animator.last_counter = animator.counter;
                },
            }
        }
    }
}

/// Simulates typing animation with an underscore cursor
pub fn typing_animation_underscore(t: f32, text: &str, buffer: &mut String) {
    let char_count = text.chars().count();
    if char_count == 0 {
        return; // Handle empty string gracefully
    }

    let visible_chars = (t * char_count as f32).floor() as usize;
    let visible_chars = visible_chars.min(char_count);

    // Use character-based slicing to handle Unicode properly
    let visible_text: String = text.chars().take(visible_chars).collect();
    buffer.push_str(&visible_text);

    if visible_chars < char_count {
        buffer.push('_');
    }
}

/// Simulates typing animation with an vertical line cursor
pub fn typing_animation_cursor(t: f32, text: &str, buffer: &mut String) {
    let char_count = text.chars().count();
    if char_count == 0 {
        return; // Handle empty string gracefully
    }

    let visible_chars = (t * char_count as f32).floor() as usize;
    let visible_chars = visible_chars.min(char_count);

    // Use character-based slicing to handle Unicode properly
    let visible_text: String = text.chars().take(visible_chars).collect();
    buffer.push_str(&visible_text);

    if visible_chars < char_count {
        buffer.push('|');
    }
}

/// Creates a decryption effect where random symbols gradually become the actual text
#[cfg(feature = "animations")]
pub fn decryption_animation(t: f32, text: &str, buffer: &mut String) {
    let char_count = text.chars().count();
    if char_count == 0 {
        return; // Handle empty string gracefully
    }

    // Hash input data into unique seed
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let seed: u64 = hasher.finish();

    // Create unique reproducible RNG from time
    let mut rng = StdRng::seed_from_u64(seed + (t * 60.0).round() as u64);

    // Define symbols used
    let symbols = "!@#$%^&*()_+-=[]{}|;:'\",.<>/?`~";
    buffer.reserve(char_count);

    for (i, c) in text.chars().enumerate() {
        // Use character count for consistent timing with character-based indexing
        let char_progress = (t * char_count as f32) - i as f32;

        if char_progress < 0.0 {
            // Not yet started decrypting this character
            let symbol_idx = rng.random_range(0..symbols.len());
            if let Some(symbol) = symbols.chars().nth(symbol_idx) {
                buffer.push(symbol);
            } else {
                // Fallback to first symbol if index is somehow invalid
                buffer.push('?');
            }
        } else if char_progress >= 1.0 {
            // This character is fully decrypted
            buffer.push(c);
        } else {
            // This character is in the process of being decrypted
            // 80% chance of showing the real character as we get closer to 1.0
            if rng.random::<f32>() < char_progress {
                buffer.push(c);
            } else {
                let symbol_idx = rng.random_range(0..symbols.len());
                if let Some(symbol) = symbols.chars().nth(symbol_idx) {
                    buffer.push(symbol);
                } else {
                    // Fallback to first symbol if index is somehow invalid
                    buffer.push('?');
                }
            }
        }
    }
}

/// Creates a slide-in effect where characters come in from the sides
pub fn slide_in_animation(t: f32, text: &str, buffer: &mut String) {
    let char_count = text.chars().count();
    if char_count == 0 {
        return; // Handle empty string gracefully
    }

    let center = char_count / 2;
    buffer.reserve(char_count);

    for (i, c) in text.chars().enumerate() {
        let distance_from_center = center.abs_diff(i);
        let char_progress = t * 2.0 - (distance_from_center as f32 / center as f32);

        if char_progress >= 1.0 {
            // Character is fully visible
            buffer.push(c);
        } else if char_progress > 0.0 {
            // Character is sliding in
            buffer.push('_');
        } else {
            // Character hasn't started appearing yet
            buffer.push(' ');
        }
    }
}

/// Reveals characters in a scrambled order
#[cfg(feature = "animations")]
pub fn scrambled_reveal_animation(t: f32, text: &str, buffer: &mut String) {
    // Create a seeded RNG for consistent scrambling
    let text_chars: Vec<char> = text.chars().collect();
    let char_count = text_chars.len();

    if char_count == 0 {
        buffer.clear();
        return;
    }

    let mut indices: Vec<usize> = (0..char_count).collect(); // Use character count, not byte count
    let seed = 42; // Fixed seed for consistent scrambling
    let mut rng = StdRng::seed_from_u64(seed);

    // Shuffle indices to determine reveal order
    indices.shuffle(&mut rng);

    let chars_to_reveal = (t * char_count as f32).floor() as usize;
    let mut result = vec![' '; char_count];

    // Reveal characters in scrambled order
    for i in indices.iter().take(chars_to_reveal.min(char_count)) {
        if let Some(ch) = text_chars.get(*i) {
            result[*i] = *ch;
        }
    }

    buffer.clear();
    buffer.extend(result.iter());
}

/// Fallback scrambled reveal animation when animations feature is disabled
#[cfg(not(feature = "animations"))]
pub fn scrambled_reveal_animation(t: f32, text: &str, buffer: &mut String) {
    // Simple linear reveal fallback
    let char_count = text.chars().count();
    let chars_to_reveal = (t * char_count as f32).floor() as usize;
    buffer.clear();
    buffer.push_str(&text.chars().take(chars_to_reveal).collect::<String>());
    buffer.push_str(&" ".repeat(char_count.saturating_sub(chars_to_reveal)));
}

/// This plugin is used for the main logic.
#[derive(Debug, Default, Clone)]
pub struct UiLunexAnimPlugin;
impl Plugin for UiLunexAnimPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, TextAnimator::system_2d);

        // Add text 3d support
        #[cfg(feature = "text3d")]
        {
            app.add_systems(Update, TextAnimator::system_3d);
        }
    }
}

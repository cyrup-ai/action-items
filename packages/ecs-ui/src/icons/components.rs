use bevy::prelude::*;
use super::types::{IconType, IconSize};

/// Interaction states for icon animations
///
/// Follows GradientInteractionState pattern for consistency across ecs-ui.
/// Used by IconComponent to determine visual appearance based on user interaction.
///
/// # States
/// - **Default**: No interaction, normal appearance
/// - **Hover**: Mouse cursor is over the icon
/// - **Pressed**: Icon is being actively clicked/pressed
/// - **Selected**: Icon represents a selected item or active state
/// - **Disabled**: Icon is non-interactive and visually muted
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconInteractionState {
    /// Default state - no interaction
    Default,
    /// Hover state - cursor over element
    Hover,
    /// Pressed state - element is being clicked
    Pressed,
    /// Selected state - element is selected/active
    Selected,
    /// Disabled state - element is non-interactive
    Disabled,
}

impl Default for IconInteractionState {
    fn default() -> Self {
        Self::Default
    }
}

/// Icon component for managing icon state and animations
///
/// Analogous to GradientComponent - tracks current icon type, size,
/// interaction state, and animation parameters.
///
/// Attach this to entities with Text or Image bundles to enable
/// icon interaction and animation features.
///
/// # Fields
/// - `icon_type`: What icon to display (folder, file, code, etc.)
/// - `size`: Icon dimensions (Small=16px, Medium=32px, Large=64px, XLarge=128px)
/// - `interaction_state`: Current interaction mode (default, hover, pressed, etc.)
/// - `custom_color`: Optional color override (None = use theme colors)
/// - `transition_speed`: Animation duration in seconds for state changes
/// - `elapsed_transition_time`: Internal animation timer
/// - `previous_state`: For detecting state changes
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use action_items_ecs_ui::icons::{IconComponent, IconType, IconSize, IconInteractionState};
///
/// fn spawn_folder_icon(mut commands: Commands) {
///     let icon = IconComponent::new(IconType::Folder, IconSize::Medium)
///         .with_transition_speed(0.3);
///
///     commands.spawn((
///         TextBundle::default(),
///         icon,
///         Interaction::default(), // from bevy::prelude
///     ));
/// }
/// ```
#[derive(Component, Debug, Clone)]
pub struct IconComponent {
    /// Current icon type
    pub icon_type: IconType,
    /// Current size
    pub size: IconSize,
    /// Interaction state for color/animation selection
    pub interaction_state: IconInteractionState,
    /// Custom color override (None = use theme colors)
    pub custom_color: Option<Color>,
    /// Animation speed for state transitions (seconds)
    pub transition_speed: f32,
    /// Accumulated time since transition started
    pub elapsed_transition_time: f32,
    /// Previous state for detecting state changes
    pub previous_state: Option<IconInteractionState>,
}

impl Default for IconComponent {
    fn default() -> Self {
        Self {
            icon_type: IconType::Unknown,
            size: IconSize::Medium,
            interaction_state: IconInteractionState::Default,
            custom_color: None,
            transition_speed: 0.2, // 200ms default
            elapsed_transition_time: 0.0,
            previous_state: None,
        }
    }
}

impl IconComponent {
    /// Create icon component with specified type and size
    ///
    /// # Arguments
    /// * `icon_type` - The icon to display
    /// * `size` - Icon dimensions
    ///
    /// # Example
    /// ```rust
    /// let icon = IconComponent::new(IconType::Code, IconSize::Large);
    /// ```
    #[inline]
    pub fn new(icon_type: IconType, size: IconSize) -> Self {
        Self {
            icon_type,
            size,
            ..Default::default()
        }
    }

    /// Set custom color override
    ///
    /// When set, this color will be used instead of theme-based colors.
    /// Useful for brand colors or special highlighting.
    ///
    /// # Example
    /// ```rust
    /// let icon = IconComponent::folder(IconSize::Medium)
    ///     .with_color(Color::srgb(1.0, 0.5, 0.0)); // Orange
    /// ```
    #[inline]
    pub fn with_color(mut self, color: Color) -> Self {
        self.custom_color = Some(color);
        self
    }

    /// Set transition speed for animations
    ///
    /// Clamped to 0.05-2.0 seconds for reasonable animation speeds.
    /// Lower = faster, higher = slower.
    ///
    /// # Example
    /// ```rust
    /// let icon = IconComponent::code(IconSize::Medium)
    ///     .with_transition_speed(0.15); // Fast 150ms transition
    /// ```
    #[inline]
    pub fn with_transition_speed(mut self, speed: f32) -> Self {
        self.transition_speed = speed.clamp(0.05, 2.0);
        self
    }

    /// Create icon for application
    ///
    /// Helper constructor for common application icon use case.
    #[inline]
    pub fn application(size: IconSize) -> Self {
        Self::new(IconType::Application, size)
    }

    /// Create icon for folder
    ///
    /// Helper constructor for common folder icon use case.
    #[inline]
    pub fn folder(size: IconSize) -> Self {
        Self::new(IconType::Folder, size)
    }

    /// Create icon for code file
    ///
    /// Helper constructor for common code file icon use case.
    #[inline]
    pub fn code(size: IconSize) -> Self {
        Self::new(IconType::Code, size)
    }
}

/// Icon animation component for smooth transitions
///
/// Similar to WindowAnimation - handles icon color and scale changes
/// with eased transitions.
///
/// This component is typically added/removed by animation systems.
/// When animation completes, systems should remove this component.
///
/// # Animation Features
/// - Color interpolation (RGBA linear blend)
/// - Scale interpolation for hover/press effects
/// - Duration-based timing
/// - Automatic completion detection
///
/// # Lifecycle
/// 1. System detects state change on IconComponent
/// 2. System adds IconAnimation with current and target values
/// 3. Each frame, system calls update() and applies results
/// 4. When is_complete() returns true, system removes component
///
/// # Example
/// ```rust
/// // In a system that detects icon state changes:
/// let animation = IconAnimation::new(
///     0.2, // 200ms duration
///     Color::srgb(0.8, 0.8, 1.0), // Light blue target
///     1.1, // Slightly larger scale
/// ).with_initial_values(
///     current_color,
///     current_scale,
/// );
/// commands.entity(icon_entity).insert(animation);
/// ```
#[derive(Component, Debug, Clone)]
pub struct IconAnimation {
    /// Current animation time (incremented each frame)
    pub current_time: f32,
    /// Total animation duration
    pub duration: f32,
    /// Starting color
    pub initial_color: Color,
    /// Target color
    pub target_color: Color,
    /// Starting scale
    pub initial_scale: f32,
    /// Target scale
    pub target_scale: f32,
}

impl IconAnimation {
    /// Create new animation with target values
    ///
    /// Initial values default to Color::WHITE and scale 1.0.
    /// Use with_initial_values() to set proper starting point.
    ///
    /// # Arguments
    /// * `duration` - Animation length in seconds
    /// * `target_color` - Final color value
    /// * `target_scale` - Final scale multiplier
    pub fn new(duration: f32, target_color: Color, target_scale: f32) -> Self {
        Self {
            current_time: 0.0,
            duration,
            initial_color: Color::WHITE,
            target_color,
            initial_scale: 1.0,
            target_scale,
        }
    }

    /// Create color-only animation (scale remains 1.0)
    ///
    /// Useful for simple color transitions without size changes.
    pub fn color(duration: f32, target_color: Color) -> Self {
        Self::new(duration, target_color, 1.0)
    }

    /// Create scale-only animation (color remains white)
    ///
    /// Useful for hover/press effects without color changes.
    pub fn scale(duration: f32, target_scale: f32) -> Self {
        Self::new(duration, Color::WHITE, target_scale)
    }

    /// Update animation and return current (color, scale)
    ///
    /// Uses linear interpolation for smooth transitions.
    ///
    /// # Arguments
    /// * `delta_time` - Time since last frame (typically from Time resource)
    ///
    /// # Returns
    /// Tuple of (current_color, current_scale)
    ///
    /// # Example
    /// ```rust
    /// fn update_icon_animations(
    ///     time: Res<Time>,
    ///     mut query: Query<(&mut IconAnimation, &mut Text, &mut Transform)>,
    /// ) {
    ///     for (mut anim, mut text, mut transform) in &mut query {
    ///         let (color, scale) = anim.update(time.delta_seconds());
    ///         text.sections[0].style.color = color;
    ///         transform.scale = Vec3::splat(scale);
    ///     }
    /// }
    /// ```
    pub fn update(&mut self, delta_time: f32) -> (Color, f32) {
        self.current_time += delta_time;
        let t = (self.current_time / self.duration).clamp(0.0, 1.0);

        // Linear interpolation for color components
        let initial_srgba = self.initial_color.to_srgba();
        let target_srgba = self.target_color.to_srgba();

        let color = Color::srgba(
            initial_srgba.red + (target_srgba.red - initial_srgba.red) * t,
            initial_srgba.green + (target_srgba.green - initial_srgba.green) * t,
            initial_srgba.blue + (target_srgba.blue - initial_srgba.blue) * t,
            initial_srgba.alpha + (target_srgba.alpha - initial_srgba.alpha) * t,
        );

        // Linear interpolation for scale
        let scale = self.initial_scale + (self.target_scale - self.initial_scale) * t;

        (color, scale)
    }

    /// Check if animation is complete
    ///
    /// Returns true when current_time >= duration.
    /// Systems should remove this component when complete.
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.current_time >= self.duration
    }

    /// Set initial values from current state
    ///
    /// Call this immediately after creating animation to capture
    /// the starting point for interpolation.
    ///
    /// # Example
    /// ```rust
    /// let animation = IconAnimation::color(0.2, target_color)
    ///     .with_initial_values(current_color, 1.0);
    /// ```
    pub fn with_initial_values(
        mut self,
        initial_color: Color,
        initial_scale: f32,
    ) -> Self {
        self.initial_color = initial_color;
        self.initial_scale = initial_scale;
        self
    }
}

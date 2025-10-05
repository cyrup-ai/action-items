# ICON_2: Icon Component System

## OBJECTIVE
Create the icon component infrastructure including IconInteractionState enum, IconComponent for state tracking, and IconAnimation for smooth transitions. These components enable interactive icons with hover effects and animations.

## RATIONALE
Following the GradientComponent pattern, icon components need to track interaction states (hover, press, selection), manage icon properties (type, size, color), and support smooth animations. This enables rich interactive UIs with professional polish.

## CODEBASE VERIFICATION

**Prerequisites Confirmed**:
- ✅ `IconSize` exists at [`/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/types.rs:8-32`](../packages/ecs-ui/src/icons/types.rs)
- ✅ `IconType` exists at [`/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/types.rs:50-91`](../packages/ecs-ui/src/icons/types.rs)
- ✅ Icon module structure exists with types, theme, cache modules
- ❌ `components.rs` does NOT exist yet - **MUST CREATE**

**Workspace Context**:
- Package: `action-items_ecs-ui` (Rust edition 2024, version 0.1.0)
- Bevy version: 0.15+ (based on Color API usage)
- Dependencies: All required types available via `bevy::prelude::*`

## REFERENCE PATTERNS

### Pattern 1: Interaction State Enum
See [`packages/ecs-ui/src/gradients/states.rs:8-19`](../packages/ecs-ui/src/gradients/states.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientInteractionState {
    Default,
    Hover,
    Selected,
    Pressed,
    Disabled,
}

impl Default for GradientInteractionState {
    fn default() -> Self {
        Self::Default
    }
}
```

### Pattern 2: Component with State Tracking
See [`packages/ecs-ui/src/gradients/components.rs:23-104`](../packages/ecs-ui/src/gradients/components.rs)

Key elements:
- Derives: `Component, Debug, Clone`
- Fields: type identifier, interaction state, custom overrides, timing fields
- Default implementation
- Builder methods marked `#[inline]`
- Helper constructors for common use cases

### Pattern 3: Animation Component
See [`packages/ecs-ui/src/animations/window.rs:7-103`](../packages/ecs-ui/src/animations/window.rs)

Key elements:
- Derives: `Component, Debug, Clone`
- Fields: initial/target values, timing
- `update(delta_time) -> (interpolated_values)` method
- `is_complete() -> bool` method
- Linear interpolation for smooth transitions

### Pattern 4: Module Export Structure
See [`packages/ecs-ui/src/gradients/mod.rs:36-50`](../packages/ecs-ui/src/gradients/mod.rs)

```rust
pub mod components;
pub mod states;
// ... other submodules

pub use components::GradientComponent;
pub use states::{GradientComponentType, GradientInteractionState};
```

## IMPLEMENTATION ORDER

### Step 1: Create components.rs (New File)
**File**: `/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/components.rs`

This file does NOT exist. Create it from scratch.

### Step 2: Add IconInteractionState Enum
At the top of components.rs, after imports.

### Step 3: Add IconComponent Struct
Following IconInteractionState in the same file.

### Step 4: Add IconAnimation Struct
Following IconComponent in the same file.

### Step 5: Update Module Exports
**File**: `/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/mod.rs`

Add components module declaration and re-export types.

### Step 6: Verify Compilation
Run `cargo check -p ecs-ui` from workspace root.

## SUBTASK 1: Create IconInteractionState Enum

**File**: `/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/components.rs` (NEW FILE)

**Create File With**:
```rust
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
```

**Pattern Reference**: [`packages/ecs-ui/src/gradients/states.rs:8-19`](../packages/ecs-ui/src/gradients/states.rs)

**What This Does**:
- Defines 5 interaction states matching UX patterns across the UI system
- Provides Default trait for initialization
- Enables state-based color/animation selection in future systems

## SUBTASK 2: Create IconComponent

**File**: `/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/components.rs` (APPEND)

**Add After IconInteractionState**:
```rust
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
```

**Pattern Reference**: [`packages/ecs-ui/src/gradients/components.rs:23-104`](../packages/ecs-ui/src/gradients/components.rs)

**What This Does**:
- Tracks all icon state in a single Component
- Provides builder pattern API for ergonomic construction
- Integrates with Bevy's `Interaction` component for hover/click detection
- Enables theme-based or custom coloring

## SUBTASK 3: Create IconAnimation Component

**File**: `/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/components.rs` (APPEND)

**Add After IconComponent**:
```rust
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
```

**Pattern Reference**: [`packages/ecs-ui/src/animations/window.rs:7-103`](../packages/ecs-ui/src/animations/window.rs)

**What This Does**:
- Provides smooth color/scale transitions for icon state changes
- Uses linear interpolation (lerp) for natural animation feel
- Self-contained timing logic - just call update() each frame
- Integrates with Bevy's Time resource via delta_time

## SUBTASK 4: Update Module Exports

**File**: `/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/mod.rs`

**Current Content** (lines 36-44):
```rust
pub mod types;
pub mod theme;
pub mod cache;

// Re-export public types
pub use types::{IconSize, IconType};
pub use theme::{ThemeColors, IconTheme};
pub use cache::IconCache;
```

**Modify To**:
```rust
pub mod types;
pub mod theme;
pub mod cache;
pub mod components;

// Re-export public types
pub use types::{IconSize, IconType};
pub use theme::{ThemeColors, IconTheme};
pub use cache::IconCache;
pub use components::{IconInteractionState, IconComponent, IconAnimation};
```

**Changes**:
1. Add `pub mod components;` after cache module declaration
2. Add `pub use components::{IconInteractionState, IconComponent, IconAnimation};` to re-exports

**Note**: The main `lib.rs` already exports icon types at line 173. It will automatically pick up these new types through the module re-export chain. No changes to lib.rs are needed.

## DEFINITION OF DONE

### Compilation Checks
- [ ] `cargo check -p ecs-ui` passes without errors
- [ ] All component types properly exported from icons module
- [ ] No clippy warnings for new code
- [ ] File created: `/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/components.rs`
- [ ] File modified: `/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/mod.rs`

### Functionality Verification
- [ ] IconInteractionState has all 5 states (Default, Hover, Pressed, Selected, Disabled)
- [ ] IconInteractionState::default() returns IconInteractionState::Default
- [ ] IconComponent can be created with new() constructor
- [ ] IconComponent builder methods work: with_color(), with_transition_speed()
- [ ] IconComponent::with_transition_speed() clamps values to 0.05-2.0
- [ ] IconComponent helper constructors exist: application(), folder(), code()
- [ ] IconAnimation::new() creates animation with specified parameters
- [ ] IconAnimation::update() returns interpolated color and scale values
- [ ] IconAnimation::is_complete() correctly identifies finished animations
- [ ] IconAnimation helper constructors exist: color(), scale()

### Code Quality
- [ ] IconInteractionState derives Debug, Clone, Copy, PartialEq, Eq
- [ ] IconComponent derives Component, Debug, Clone
- [ ] IconAnimation derives Component, Debug, Clone
- [ ] Builder pattern methods are marked #[inline]
- [ ] Helper constructors provided (application(), folder(), code())
- [ ] Documentation explains relationship to GradientComponent pattern
- [ ] All public items have doc comments
- [ ] Code follows existing ecs-ui formatting and conventions

## CONSTRAINTS

- **NO TESTS**: Do not write unit tests, integration tests, or test modules
- **NO BENCHMARKS**: Do not write benchmark code  
- **NO EXTENSIVE DOCS**: Code comments are sufficient, no separate documentation files needed
- **SINGLE SESSION**: This task must be completable in one focused session
- **NO SYSTEMS**: Components only - animation and interaction systems come in ICON_6
- **NO PLUGINS**: Plugin registration comes in later tasks
- **SCOPE UNCHANGED**: Create exactly 3 types (enum, 2 structs), modify 1 file

## WHAT NEEDS TO CHANGE

### Create New File
**Path**: `/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/components.rs`

**Contents**:
1. Imports: `bevy::prelude::*` and `super::types::{IconType, IconSize}`
2. IconInteractionState enum (5 variants + Default impl)
3. IconComponent struct (7 fields + Default impl + 6 methods)
4. IconAnimation struct (6 fields + 5 methods)

**Total**: ~245 lines of code

### Modify Existing File  
**Path**: `/Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/mod.rs`

**Changes**:
1. Line 38: Add `pub mod components;`
2. Line 44: Add `pub use components::{IconInteractionState, IconComponent, IconAnimation};`

**Total**: 2 new lines

## REFERENCE FILES

**Pattern Files** (Read These):
- [`packages/ecs-ui/src/gradients/components.rs:23-104`](../packages/ecs-ui/src/gradients/components.rs) - GradientComponent pattern
- [`packages/ecs-ui/src/gradients/states.rs:8-19`](../packages/ecs-ui/src/gradients/states.rs) - State enum pattern  
- [`packages/ecs-ui/src/animations/window.rs:7-103`](../packages/ecs-ui/src/animations/window.rs) - WindowAnimation pattern
- [`packages/ecs-ui/src/gradients/mod.rs:36-50`](../packages/ecs-ui/src/gradients/mod.rs) - Module export pattern

**Dependencies** (Already Exist):
- `IconSize` - [`packages/ecs-ui/src/icons/types.rs:8-32`](../packages/ecs-ui/src/icons/types.rs)
- `IconType` - [`packages/ecs-ui/src/icons/types.rs:50-91`](../packages/ecs-ui/src/icons/types.rs)
- `Color` - from `bevy::prelude::*` (Bevy 0.15+)
- `Component` - from `bevy::prelude::*`

**Module Context**:
- Main export: [`packages/ecs-ui/src/lib.rs:172-173`](../packages/ecs-ui/src/lib.rs)
- Current icon mod: [`packages/ecs-ui/src/icons/mod.rs`](../packages/ecs-ui/src/icons/mod.rs)

## VERIFICATION COMMAND

```bash
# From workspace root
cargo check -p ecs-ui

# Expected output:
# Checking action-items_ecs-ui v0.1.0 (/Volumes/samsung_t9/action-items/packages/ecs-ui)
# Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs
```

## NOTES

- **Bevy Version**: This uses Bevy 0.15+ Color API (`.to_srgba()`, `Color::srgba()`)
- **No External Dependencies**: All types come from Bevy or existing ecs-ui modules
- **Systems Integration**: These components will be consumed by systems in ICON_6 task
- **Theme Integration**: IconComponent.custom_color is optional - None means "use theme"
- **Animation Lifecycle**: IconAnimation is intended to be added/removed by systems

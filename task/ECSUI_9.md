# ECSUI_9: Move InteractiveGradient Systems to ecs-ui Package

## Status
Not Started

## Priority
High - Architecture cleanup

## Description
The `InteractiveGradient` component is defined in `packages/ecs-ui/src/gradients/interactive.rs` but its systems are implemented in `packages/ui/src/ui/systems/animations.rs`. Systems for ecs-ui components should live in ecs-ui, not in application code.

## Current State

### InteractiveGradient Component Location
- **File**: `packages/ecs-ui/src/gradients/interactive.rs`
- **Exports**: Component with `default_color`, `hover_color`, `selected_color`, `transition_speed`
- **Purpose**: Simple color-based gradient for interactive UI elements

### Systems Currently in packages/ui
**File**: `packages/ui/src/ui/systems/animations.rs`

1. **interactive_gradient_system** (lines 105-136)
   - Handles Interaction changes for InteractiveGradient components
   - Adds/updates GradientTransition component for smooth color transitions
   - Uses Time resource for animation timing

2. **update_gradient_transitions_system** (lines 140-165)
   - Animates GradientTransition components
   - Interpolates colors using easing functions
   - Updates BackgroundColor during animation
   - Removes GradientTransition when animation completes

3. **GradientTransition component** (lines 12-17)
   - Tracks from_color, to_color, and animation state
   - Used by the above systems for smooth transitions

## Required Changes

### 1. Move Systems to ecs-ui
**Target**: `packages/ecs-ui/src/gradients/systems.rs`

Add these systems to the existing gradients systems file:
```rust
/// Interactive gradient system for InteractiveGradient component
/// Updates gradient states based on UI interactions with smooth transitions
#[inline]
pub fn interactive_gradient_interactive_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Interaction,
        &InteractiveGradient,
        &BackgroundColor,
        Option<&mut InteractiveGradientTransition>,
    ), Changed<Interaction>>,
    time: Res<Time>,
) {
    // Implementation from ui/systems/animations.rs lines 105-136
}

/// Animate InteractiveGradient color transitions
/// Handles smooth color interpolation for InteractiveGradient components
#[inline]
pub fn animate_interactive_gradient_transitions_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BackgroundColor, &mut InteractiveGradientTransition)>,
    time: Res<Time>,
) {
    // Implementation from ui/systems/animations.rs lines 140-165
}
```

### 2. Move GradientTransition Component
**Target**: `packages/ecs-ui/src/gradients/interactive.rs`

Rename to `InteractiveGradientTransition` and add to interactive.rs:
```rust
/// Component to track gradient transition animations for InteractiveGradient
#[derive(Component, Debug)]
pub struct InteractiveGradientTransition {
    pub from_color: Color,
    pub to_color: Color,
    pub animation: crate::animations::AnimationState,
}
```

### 3. Update GradientPlugin
**File**: `packages/ecs-ui/src/gradients/plugin.rs`

Add the new systems to the plugin:
```rust
.add_systems(
    Update,
    (
        apply_gradient_system,
        interactive_gradient_system, // Existing for GradientComponent
        animate_gradient_transitions_system, // Existing for GradientComponent
        interactive_gradient_interactive_system, // NEW for InteractiveGradient
        animate_interactive_gradient_transitions_system, // NEW for InteractiveGradient
    ),
)
```

### 4. Update ecs-ui Exports
**File**: `packages/ecs-ui/src/gradients/mod.rs`

Export the new types:
```rust
pub use interactive::{InteractiveGradient, InteractiveGradientTransition};
```

**File**: `packages/ecs-ui/src/lib.rs`

Add to public exports:
```rust
pub use crate::gradients::{
    // ... existing exports
    InteractiveGradientTransition,
    interactive_gradient_interactive_system,
    animate_interactive_gradient_transitions_system,
};
```

### 5. Remove from packages/ui
**File**: `packages/ui/src/ui/systems/animations.rs`

- Remove `GradientTransition` component (lines 12-17)
- Remove `interactive_gradient_system` function (lines 105-136)
- Remove `update_gradient_transitions_system` function (lines 140-165)
- Remove InteractiveGradient from imports (line 7)

**Keep** the following in animations.rs (they're app-specific):
- `animate_result_items_system` - Uses ActionResultItem
- `animate_hover_effects_system` - Simple hover animations

### 6. Update ui Plugin
**File**: `packages/ui/src/lib.rs`

Remove the deleted systems from LauncherUiPlugin if they're registered there.

## Verification Checklist

- [ ] InteractiveGradient systems moved to ecs-ui/gradients/systems.rs
- [ ] InteractiveGradientTransition component in ecs-ui/gradients/interactive.rs
- [ ] Systems registered in GradientPlugin
- [ ] Exports updated in ecs-ui/lib.rs
- [ ] Systems removed from ui/systems/animations.rs
- [ ] ui package still compiles
- [ ] ecs-ui package compiles
- [ ] No duplicate system implementations
- [ ] GradientPlugin automatically provides InteractiveGradient functionality

## Files Modified

- `packages/ecs-ui/src/gradients/systems.rs` - Add 2 new systems
- `packages/ecs-ui/src/gradients/interactive.rs` - Add InteractiveGradientTransition
- `packages/ecs-ui/src/gradients/plugin.rs` - Register new systems
- `packages/ecs-ui/src/gradients/mod.rs` - Export new types
- `packages/ecs-ui/src/lib.rs` - Public exports
- `packages/ui/src/ui/systems/animations.rs` - Remove 2 systems + component
- `packages/ui/src/lib.rs` - Remove system registrations (if any)

## Dependencies

None - this is a pure refactoring task

## Notes

- This aligns with the architecture principle: "ecs-ui provides components AND their systems"
- After this change, apps using InteractiveGradient just add GradientPlugin and get full functionality
- The naming distinction helps: `interactive_gradient_system` (for GradientComponent) vs `interactive_gradient_interactive_system` (for InteractiveGradient)

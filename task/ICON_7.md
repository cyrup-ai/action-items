# ICON_7: IconPlugin and ecs-ui Integration

## OBJECTIVE
Create the IconPlugin that registers all icon resources, events, and systems, then integrate it into ecs-ui's public API. This completes the icon infrastructure and makes it available for use in applications.

## RATIONALE
The plugin ties together all icon components (types, components, events, systems, FontAwesome, extraction) into a single, cohesive unit. Following the GradientPlugin and VisibilityPlugin patterns, it provides complete functionality out of the box.

## PREREQUISITES
- ICON_1 through ICON_6 complete (all icon infrastructure exists)

## SUBTASK 1: Create IconPlugin

**File**: `packages/ecs-ui/src/icons/plugin.rs`

**Create**:
```rust
use bevy::prelude::*;

use super::{
    // Resources
    IconCache,
    IconTheme,
    FontAwesome,
    // Events
    IconExtractionRequest,
    IconExtractionResult,
    IconColorChangeEvent,
    IconSizeChangeEvent,
    IconStateChangeEvent,
    IconAnimationCompleteEvent,
    // Extraction systems
    process_icon_extraction_requests,
    poll_icon_extraction_tasks,
    process_icon_extraction_results,
    // Interactive systems
    apply_icon_system,
    interactive_icon_system,
    animate_icon_transitions_system,
    handle_icon_color_change_events,
    handle_icon_size_change_events,
    optimize_icon_performance_system,
    validate_icon_cache_system,
};

/// Complete interactive icon system plugin
///
/// Provides full icon functionality:
/// - Icon types and theming (IconSize, IconType, IconTheme)
/// - FontAwesome icon rendering system
/// - Interactive icon components with state tracking
/// - Smooth animations and transitions
/// - Async icon extraction from files (macOS/Windows/Linux)
/// - Event-driven color/size changes
/// - Performance optimization and caching
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use action_items_ecs_ui::icons::IconPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(IconPlugin)
///         .run();
/// }
/// ```
#[derive(Debug, Default, Clone)]
pub struct IconPlugin;

impl Plugin for IconPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources - icon cache, theme, and FontAwesome
            .init_resource::<IconCache>()
            .init_resource::<IconTheme>()
            .init_resource::<FontAwesome>()

            // Events - extraction, interactive changes, animations
            .add_event::<IconExtractionRequest>()
            .add_event::<IconExtractionResult>()
            .add_event::<IconColorChangeEvent>()
            .add_event::<IconSizeChangeEvent>()
            .add_event::<IconStateChangeEvent>()
            .add_event::<IconAnimationCompleteEvent>()

            // Core icon systems (run every frame)
            .add_systems(
                Update,
                (
                    apply_icon_system,
                    interactive_icon_system,
                    animate_icon_transitions_system,
                ),
            )

            // Icon extraction systems (async)
            .add_systems(
                Update,
                (
                    process_icon_extraction_requests,
                    poll_icon_extraction_tasks,
                    process_icon_extraction_results,
                ),
            )

            // Event handler systems
            .add_systems(
                Update,
                (
                    handle_icon_color_change_events,
                    handle_icon_size_change_events,
                ),
            )

            // Performance systems (run when cache changes)
            .add_systems(
                Update,
                (
                    optimize_icon_performance_system,
                    validate_icon_cache_system,
                )
                    .run_if(resource_changed::<IconCache>),
            );
    }
}
```

**Pattern**: Follows `packages/ecs-ui/src/gradients/plugin.rs:30-60` and `packages/ecs-ui/src/visibility/plugin.rs:20-35`

## SUBTASK 2: Update Icons Module

**File**: `packages/ecs-ui/src/icons/mod.rs`

**Modify** (add plugin module and re-export):
```rust
//! Complete interactive icon system for Bevy applications
//!
//! Provides:
//! - Icon types and theming (IconSize, IconType, IconTheme)
//! - FontAwesome icon rendering with Unicode characters
//! - Interactive icon components with hover/press/selection states
//! - Async icon extraction from files (platform-specific)
//! - Smooth animations and transitions
//! - Event-driven dynamic icon changes
//! - Performance optimization and caching
//!
//! # Example
//! ```rust
//! use bevy::prelude::*;
//! use action_items_ecs_ui::icons::{IconPlugin, IconComponent, IconType, IconSize};
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn((
//!         TextBundle::default(),
//!         IconComponent::new(IconType::Folder, IconSize::Medium),
//!         Interaction::default(),
//!     ));
//! }
//!
//! App::new()
//!     .add_plugins(IconPlugin)
//!     .add_systems(Startup, setup)
//!     .run();
//! ```

pub mod types;
pub mod theme;
pub mod cache;
pub mod components;
pub mod events;
pub mod fontawesome;
pub mod extraction;
pub mod systems;
pub mod plugin;

// Re-export public types
pub use types::{IconSize, IconType};
pub use theme::{ThemeColors, IconTheme};
pub use cache::IconCache;
pub use components::{IconInteractionState, IconComponent, IconAnimation};
pub use events::{
    IconExtractionRequest,
    IconExtractionResult,
    IconColorChangeEvent,
    IconSizeChangeEvent,
    IconStateChangeEvent,
    IconAnimationCompleteEvent,
    IconAnimationType,
};
pub use fontawesome::{FontAwesome, IconDetection, IconFallback};
pub use extraction::{
    IconExtractionInProgress,
    process_icon_extraction_requests,
    poll_icon_extraction_tasks,
    process_icon_extraction_results,
};
pub use systems::{
    apply_icon_system,
    interactive_icon_system,
    animate_icon_transitions_system,
    handle_icon_color_change_events,
    handle_icon_size_change_events,
    optimize_icon_performance_system,
    validate_icon_cache_system,
};
pub use plugin::IconPlugin;
```

## SUBTASK 3: Update ecs-ui lib.rs

**File**: `packages/ecs-ui/src/lib.rs`

**Locate** the visibility module export (around line 120), **then add after it**:

```rust
// Icon system infrastructure
pub mod icons;
pub use icons::{
    // Plugin
    IconPlugin,
    // Core types
    IconSize,
    IconType,
    IconCache,
    IconTheme,
    ThemeColors,
    // Components
    IconComponent,
    IconInteractionState,
    IconAnimation,
    // Events
    IconExtractionRequest,
    IconExtractionResult,
    IconColorChangeEvent,
    IconSizeChangeEvent,
    IconStateChangeEvent,
    IconAnimationCompleteEvent,
    IconAnimationType,
    // FontAwesome
    FontAwesome,
    IconDetection,
    IconFallback,
};
```

**Note**: The exact line number may vary - insert after the visibility module exports.

## DEFINITION OF DONE

### Compilation
- [ ] `cargo check -p ecs-ui` passes without errors
- [ ] `cargo build -p ecs-ui` succeeds
- [ ] IconPlugin properly exported from ecs-ui
- [ ] No missing dependencies or import errors

### Functionality
- [ ] IconPlugin registers 3 resources (IconCache, IconTheme, FontAwesome)
- [ ] IconPlugin registers 6 event types
- [ ] IconPlugin registers 12 systems across 4 system sets
- [ ] IconPlugin implements Plugin trait with build() method
- [ ] IconPlugin derives Debug, Default, Clone

### Code Quality
- [ ] Plugin documentation explains full feature set
- [ ] Example code shows basic usage
- [ ] System registration organized by category (core, extraction, events, performance)
- [ ] run_if condition used for performance systems

## CONSTRAINTS

- **NO TESTS**: Do not write unit tests, integration tests, or test modules
- **NO BENCHMARKS**: Do not write benchmark code
- **SINGLE SESSION**: This task must be completable in one focused session
- **COMPLETE REGISTRATION**: All components, events, and systems must be registered

## REFERENCE FILES

**Pattern Files**:
- `packages/ecs-ui/src/gradients/plugin.rs:30-60` - Plugin registration pattern
- `packages/ecs-ui/src/visibility/plugin.rs:20-35` - Simple plugin example
- `packages/ecs-ui/src/lib.rs:100-120` - Module export pattern

**System Counts**:
- **Core systems** (3): apply_icon, interactive_icon, animate_transitions
- **Extraction systems** (3): process_requests, poll_tasks, process_results
- **Event handlers** (2): handle_color_change, handle_size_change
- **Performance systems** (2): optimize_performance, validate_cache
- **Total**: 10 systems + 6 events + 3 resources

**Dependencies**:
- Requires all of ICON_1 through ICON_6 complete

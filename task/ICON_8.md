# ICON_8: UI Package Integration and Cleanup

## OBJECTIVE
Update the ui package to use ecs-ui's IconPlugin, migrate application-specific helper functions, update privacy icons, and delete migrated files. This completes the icon system migration.

## RATIONALE
The ui package should consume ecs-ui's icon infrastructure and only contain application-specific code: ActionItem/SearchResult → IconType mappings and launcher-specific UI components.

## PREREQUISITES
- ICON_1 through ICON_7 complete (full icon system in ecs-ui)
- ecs-ui exports IconPlugin and all icon types

## SUBTASK 1: Update ui Package Icons Module

**File**: `packages/ui/src/ui/icons/mod.rs`

**Replace entire file with**:
```rust
//! Icon system - re-exports ecs-ui infrastructure with app-specific helpers

// Re-export all ecs-ui icon types
pub use action_items_ecs_ui::icons::*;

// Keep only application-specific modules
pub mod utils;
pub mod privacy_icons;

// Re-export application-specific helpers
pub use utils::{get_icon_for_result, get_icon_for_search_result};
pub use privacy_icons::*;
```

**Key Changes**:
- Remove types, extraction, fontawesome, generation modules (now in ecs-ui)
- Keep only utils and privacy_icons (app-specific)

## SUBTASK 2: Update Icon Utils

**File**: `packages/ui/src/ui/icons/utils.rs`

**Modify imports**:
```rust
use action_items_core::SearchResult;
use bevy::prelude::*;

// Import from ecs-ui instead of local
use action_items_ecs_ui::icons::{IconCache, IconType};

/// Get icon for ActionItem result
///
/// Application-specific mapping: ActionItem → IconType.
/// Uses generic IconCache from ecs-ui.
pub fn get_icon_for_result(
    result: &action_items_core::plugins::ActionItem,
    icon_cache: &IconCache,
) -> Handle<Image> {
    // (keep existing implementation unchanged)
}

/// Get icon for search result
///
/// Application-specific mapping: SearchResult → IconType.
/// Uses generic IconCache from ecs-ui.
pub fn get_icon_for_search_result(
    result: &SearchResult,
    icon_cache: &IconCache,
) -> Handle<Image> {
    // (keep existing implementation unchanged)
}
```

**Changes**: Only update imports - implementation stays the same

## SUBTASK 3: Update Privacy Icons

**File**: `packages/ui/src/ui/icons/privacy_icons.rs`

**Modify imports** (add at top):
```rust
use bevy::prelude::*;

// These components are defined locally (launcher-specific UI)
use super::super::ai_menu::privacy_events::IndicatorType;
use super::super::ai_menu::{PrivacyIconButton, PrivacyIndicators};
use crate::ui::components::PrivacyIndicatorPanel;
```

**No other changes** - privacy icons are launcher-specific and stay as-is.

## SUBTASK 4: Update ui Package lib.rs

**File**: `packages/ui/src/lib.rs`

**Find** the LauncherUiPlugin implementation (around line 50-100).

**Replace** icon-related code:
```rust
// OLD (remove these lines):
// app.init_resource::<IconCache>()
//    .init_resource::<IconTheme>()
//    .add_event::<IconExtractionRequest>()
//    .add_event::<IconExtractionResult>()
//    .add_systems(Update, (
//        process_icon_extraction_requests,
//        poll_icon_extraction_tasks,
//        process_icon_extraction_results,
//    ))

// NEW (add single line):
app.add_plugins(action_items_ecs_ui::icons::IconPlugin)
```

**Note**: Exact location depends on current plugin structure - look for icon resource/event registration.

## SUBTASK 5: Delete Migrated Files

**Delete these files**:
```bash
# Core types migrated to ecs-ui
rm /Volumes/samsung_t9/action-items/packages/ui/src/ui/icons/types.rs

# Extraction systems migrated to ecs-ui
rm /Volumes/samsung_t9/action-items/packages/ui/src/ui/icons/extraction.rs

# Generation no longer needed (use FontAwesome)
rm /Volumes/samsung_t9/action-items/packages/ui/src/ui/icons/generation.rs

# FontAwesome migrated to ecs-ui
rm -rf /Volumes/samsung_t9/action-items/packages/ui/src/ui/icons/fontawesome
```

**Verify deletion**:
```bash
ls -la /Volumes/samsung_t9/action-items/packages/ui/src/ui/icons/
# Should only see: mod.rs, utils.rs, privacy_icons.rs
```

## SUBTASK 6: Verify Compilation

**Run checks**:
```bash
# Check ecs-ui compiles
cargo check -p ecs-ui

# Check ui compiles with new imports
cargo check -p ui

# Build entire workspace
cargo build --workspace
```

## DEFINITION OF DONE

### Compilation
- [ ] `cargo check -p ecs-ui` passes
- [ ] `cargo check -p ui` passes
- [ ] `cargo build --workspace` succeeds
- [ ] No import errors or missing symbols

### File Structure
- [ ] `packages/ui/src/ui/icons/` contains only: mod.rs, utils.rs, privacy_icons.rs
- [ ] Deleted files: types.rs, extraction.rs, generation.rs, fontawesome/
- [ ] All imports use `action_items_ecs_ui::icons::*`

### Functionality
- [ ] ui package re-exports all ecs-ui icon types
- [ ] get_icon_for_result works (uses ecs-ui IconCache)
- [ ] get_icon_for_search_result works
- [ ] Privacy icons still render correctly
- [ ] LauncherUiPlugin uses IconPlugin

### Code Organization
- [ ] No duplicate code between ecs-ui and ui packages
- [ ] Application-specific helpers clearly documented
- [ ] Generic infrastructure consumed from ecs-ui

## CONSTRAINTS

- **NO TESTS**: Do not write unit tests, integration tests, or test modules
- **NO BENCHMARKS**: Do not write benchmark code
- **SINGLE SESSION**: This task must be completable in one focused session
- **MINIMAL CHANGES**: Only update imports and delete files - no logic changes

## REFERENCE FILES

**Files to Modify**:
- `packages/ui/src/ui/icons/mod.rs` - Re-export ecs-ui + app helpers
- `packages/ui/src/ui/icons/utils.rs` - Update imports only
- `packages/ui/src/ui/icons/privacy_icons.rs` - Update imports only
- `packages/ui/src/lib.rs` - Replace icon registration with IconPlugin

**Files to Delete**:
- `packages/ui/src/ui/icons/types.rs`
- `packages/ui/src/ui/icons/extraction.rs`
- `packages/ui/src/ui/icons/generation.rs`
- `packages/ui/src/ui/icons/fontawesome/` (directory)

**Verification**:
- After changes, only 3 files remain in ui/icons: mod.rs, utils.rs, privacy_icons.rs
- All icon infrastructure comes from ecs-ui
- ui package has no icon types, systems, or resources defined locally

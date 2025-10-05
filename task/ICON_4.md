# ICON_4: FontAwesome System Migration

## OBJECTIVE
Migrate the complete FontAwesome icon system from ui package to ecs-ui, including Unicode character mappings, semantic color system, size configurations, file type detection, and fallback handling. Make it usable with the generic IconType enum.

## RATIONALE
FontAwesome provides a comprehensive icon rendering system using Unicode characters. This is 100% generic - any Bevy app needs file type icons with semantic colors. The system maps IconType variants to Unicode characters and theme-appropriate colors.

## PREREQUISITES
- ICON_1 complete (IconType enum exists)
- ecs-ui theme module exists (Theme resource with colors)

## SUBTASK 1: Create FontAwesome Directory

**Command**:
```bash
mkdir -p /Volumes/samsung_t9/action-items/packages/ecs-ui/src/icons/fontawesome
```

## SUBTASK 2: Migrate Icon Mappings

**File**: `packages/ecs-ui/src/icons/fontawesome/mappings.rs`

**Source**: Migrate from `packages/ui/src/ui/icons/fontawesome/mappings.rs`

**Changes Required**:
1. Update imports: `use crate::ui::icons::IconType` → `use crate::icons::types::IconType`
2. Update imports: `use action_items_ecs_ui::theme::Theme` → `use crate::theme::Theme`
3. Update imports: `use crate::ui::icons::types::IconSize` → `use crate::icons::types::IconSize`
4. Keep all HashMap implementations unchanged
5. Keep all Unicode character mappings unchanged
6. Keep all color mapping logic unchanged

**Key Points**:
- IconMappings maps IconType → Unicode char
- ColorMappings maps IconType → Color (using Theme)
- SizeConfigs maps IconSize → f32 font size

## SUBTASK 3: Migrate Icon Detection

**File**: `packages/ecs-ui/src/icons/fontawesome/detection.rs`

**Source**: Migrate from `packages/ui/src/ui/icons/fontawesome/detection.rs`

**Changes Required**:
1. Update imports: `use crate::ui::icons::IconType` → `use crate::icons::types::IconType`
2. Keep all detection logic unchanged (detect_from_extension, detect_from_path, etc.)

**Key Points**:
- IconDetection provides file extension → IconType mapping
- Comprehensive extension support (code, documents, media, archives, etc.)
- Path-based detection for folders and files

## SUBTASK 4: Migrate Icon Fallback

**File**: `packages/ecs-ui/src/icons/fontawesome/fallback.rs`

**Source**: Migrate from `packages/ui/src/ui/icons/fontawesome/fallback.rs`

**Changes Required**:
1. Update imports: `use crate::ui::icons::IconType` → `use crate::icons::types::IconType`
2. Keep all fallback logic unchanged

**Key Points**:
- IconFallback provides ASCII fallback characters
- Used when FontAwesome font fails to load

## SUBTASK 5: Migrate FontAwesome Main

**File**: `packages/ecs-ui/src/icons/fontawesome/main.rs`

**Source**: Migrate from `packages/ui/src/ui/icons/fontawesome/main.rs`

**Changes Required**:
1. Update imports:
   - `use action_items_ecs_ui::theme::Theme` → `use crate::theme::Theme`
   - `use crate::ui::icons::{IconSize, IconType}` → `use crate::icons::types::{IconSize, IconType}`
   - Remove `use crate::ui::typography::TypographyScale` (will be parameterized)
2. Modify `create_icon_text` signature to accept font handle directly:
```rust
pub fn create_icon_text(
    &self,
    icon_type: IconType,
    size: IconSize,
    theme: &Theme,
    font_handle: Handle<Font>,
) -> (Text, TextFont, TextColor, TextShadow) {
    let character = self.get_icon_char(icon_type);
    let color = self.get_icon_color(icon_type, theme);
    let font_size = self.get_icon_size(size);

    (
        Text::new(character.to_string()),
        TextFont {
            font: font_handle,
            font_size,
            ..default()
        },
        TextColor(color),
        TextShadow {
            color: theme.colors.text_secondary.with_alpha(0.5),
            offset: Vec2::new(0.0, 1.0),
        },
    )
}
```

**Key Points**:
- FontAwesome is a Resource
- Provides get_icon_char, get_icon_color, get_icon_size methods
- create_icon_text helper creates complete Text bundle

## SUBTASK 6: Create FontAwesome Module File

**File**: `packages/ecs-ui/src/icons/fontawesome/mod.rs`

**Source**: Copy from `packages/ui/src/ui/icons/fontawesome/mod.rs`

**Content** (unchanged):
```rust
//! FontAwesome icon system module
//!
//! Provides a comprehensive FontAwesome icon system with Unicode character mappings
//! for zero-allocation icon rendering, intelligent type detection, and semantic coloring.
//!
//! The module has been decomposed into logical submodules:
//! - `mappings` - Icon mappings, color configurations, and size settings
//! - `detection` - Icon type detection utilities for intelligent inference
//! - `fallback` - Fallback icon system for graceful degradation
//! - `main` - Main FontAwesome struct and public API

pub mod detection;
pub mod fallback;
pub mod main;
pub mod mappings;

// Re-export all public types and functions
pub use detection::IconDetection;
pub use fallback::IconFallback;
pub use main::FontAwesome;
pub use mappings::{ColorMappings, IconMappings, SizeConfigs};
```

## SUBTASK 7: Update Icons Module

**File**: `packages/ecs-ui/src/icons/mod.rs`

**Modify** (add fontawesome module):
```rust
//! Core icon type system for Bevy applications

pub mod types;
pub mod theme;
pub mod cache;
pub mod components;
pub mod events;
pub mod fontawesome;

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
```

## DEFINITION OF DONE

### Compilation
- [ ] `cargo check -p ecs-ui` passes without errors
- [ ] FontAwesome exports correctly from icons module
- [ ] No import errors or missing dependencies

### Functionality
- [ ] FontAwesome::get_icon_char() returns correct Unicode for all 22 IconType variants
- [ ] FontAwesome::get_icon_color() returns theme-appropriate colors
- [ ] FontAwesome::get_icon_size() returns correct font sizes for IconSize variants
- [ ] IconDetection::detect_from_extension() works for common file types
- [ ] IconFallback provides ASCII fallbacks

### Code Quality
- [ ] All imports updated to use ecs-ui paths
- [ ] No references to ui package remain
- [ ] Documentation preserved
- [ ] derive(Resource) on FontAwesome

## CONSTRAINTS

- **NO TESTS**: Do not write unit tests, integration tests, or test modules
- **NO BENCHMARKS**: Do not write benchmark code
- **SINGLE SESSION**: This task must be completable in one focused session
- **EXACT MIGRATION**: Copy existing logic, only update imports

## REFERENCE FILES

**Source Files**:
- `packages/ui/src/ui/icons/fontawesome/mod.rs` - Module structure
- `packages/ui/src/ui/icons/fontawesome/main.rs` - FontAwesome resource (modify font param)
- `packages/ui/src/ui/icons/fontawesome/mappings.rs` - Mappings (update imports)
- `packages/ui/src/ui/icons/fontawesome/detection.rs` - Detection (update imports)
- `packages/ui/src/ui/icons/fontawesome/fallback.rs` - Fallback (update imports)

**Dependencies**:
- Requires ecs-ui theme module (Theme resource with colors)
- Requires ICON_1 (IconType, IconSize)

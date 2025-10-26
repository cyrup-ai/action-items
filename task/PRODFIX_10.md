# PRODFIX_10: Remove Redundant HotkeyCaptureUIState Resource

## OBJECTIVE
Remove `HotkeyCaptureUIState` from the ecs-hotkey package as it is completely redundant with `PreferencesResource` in ecs-preferences. This fixes architectural duplication and prevents circular dependency risk.

**CRITICAL DISCOVERY**: This task is NOT about moving code - it's about removing duplication. `PreferencesResource` in `packages/ecs-preferences/src/resources.rs` already contains all the fields from `HotkeyCaptureUIState` plus additional functionality. The solution is deprecation and cleanup, not code migration.

## PRIORITY
**P2 - MEDIUM (Architecture Issue - Redundant Code)**

## CURRENT STATE

### HotkeyCaptureUIState Definition
**Location**: `packages/ecs-hotkey/src/resources.rs:260-278`

```rust
/// Hotkey capture UI state (move to ecs-preferences or ecs-ui package)
/// 
/// UI-specific state for preferences window rendering.
/// TODO: In future refactor, move HotkeyCaptureUIState to:
/// packages/ecs-preferences/src/resources.rs OR packages/ecs-ui/src/preferences/resources.rs
#[derive(Resource, Default)]
pub struct HotkeyCaptureUIState {
    /// Whether preferences window is visible
    pub visible: bool,
    
    /// Is the hotkey input field focused?
    pub input_focused: bool,
    
    /// Current hotkey status for UI display
    pub current_status: HotkeyStatus,
    
    /// Whether currently testing a hotkey
    pub testing_hotkey: bool,
    
    /// Available alternative hotkey combinations
    pub available_alternatives: Vec<HotkeyDefinition>,
}
```

### PreferencesResource Definition
**Location**: `packages/ecs-preferences/src/resources.rs:20-75`

```rust
/// Preferences UI state resource
#[derive(Resource)]
pub struct PreferencesResource {
    /// Whether preferences window is visible
    pub is_visible: bool,  // EQUIVALENT to HotkeyCaptureUIState.visible
    /// Whether preferences are currently loading from disk
    pub loading: bool,
    /// Whether preferences are currently being saved to disk
    pub saving: bool,

    // Hotkey capture state
    /// Is the hotkey input field focused?
    pub input_focused: bool,  // EXACT MATCH
    /// Currently recording keystrokes?
    pub capturing: bool,
    /// Currently held modifier keys - updated in real-time
    pub held_modifiers: Modifiers,
    /// Main key that was pressed
    pub captured_key: Option<Code>,
    /// Complete captured combination
    pub captured_hotkey: Option<HotkeyDefinition>,

    // Status and testing
    /// Current hotkey status for UI display
    pub current_status: HotkeyStatus,  // EXACT MATCH
    /// Whether currently testing a hotkey
    pub testing_hotkey: bool,  // EXACT MATCH
    /// Available alternative hotkey combinations
    pub available_alternatives: Vec<HotkeyDefinition>,  // EXACT MATCH
    /// Last error message from file operations
    pub last_error: Option<String>,
    /// Timestamp of last successful save
    pub last_save_success: Option<std::time::SystemTime>,
    /// Currently loaded preferences from disk
    pub loaded_preferences: Option<HotkeyPreferences>,
}
```

### Field Mapping Analysis

| HotkeyCaptureUIState Field | PreferencesResource Equivalent | Notes |
|----------------------------|-------------------------------|-------|
| `visible: bool` | `is_visible: bool` | Semantically identical, different name |
| `input_focused: bool` | `input_focused: bool` | Exact match |
| `current_status: HotkeyStatus` | `current_status: HotkeyStatus` | Exact match |
| `testing_hotkey: bool` | `testing_hotkey: bool` | Exact match |
| `available_alternatives: Vec<HotkeyDefinition>` | `available_alternatives: Vec<HotkeyDefinition>` | Exact match |
| N/A | `capturing: bool` | Additional field (enhanced functionality) |
| N/A | `held_modifiers: Modifiers` | Additional field (enhanced functionality) |
| N/A | `captured_key: Option<Code>` | Additional field (enhanced functionality) |
| N/A | `captured_hotkey: Option<HotkeyDefinition>` | Additional field (enhanced functionality) |

**Conclusion**: `PreferencesResource` is a complete superset of `HotkeyCaptureUIState` with additional functionality.

### Current Usage Analysis

Total references found: **4 locations** (minimal usage)

1. **Definition**: `packages/ecs-hotkey/src/resources.rs:260-278` - Struct definition with TODO comment
2. **Plugin Registration**: `packages/ecs-hotkey/src/lib.rs:221` - `.insert_resource(HotkeyCaptureUIState::default())`
3. **Utility Function**: `packages/ecs-hotkey/src/resources.rs:639-645` - `scan_for_available_hotkeys()` function

No actual usage in business logic - only registration and one utility function.

## IMPLEMENTATION PLAN

### STEP 1: Deprecate HotkeyCaptureUIState
**File**: `packages/ecs-hotkey/src/resources.rs`
**Line**: 257-278

**Current code**:
```rust
/// Hotkey capture UI state (move to ecs-preferences or ecs-ui package)
/// 
/// UI-specific state for preferences window rendering.
/// TODO: In future refactor, move HotkeyCaptureUIState to:
/// packages/ecs-preferences/src/resources.rs OR packages/ecs-ui/src/preferences/resources.rs
#[derive(Resource, Default)]
pub struct HotkeyCaptureUIState {
    pub visible: bool,
    pub input_focused: bool,
    pub current_status: HotkeyStatus,
    pub testing_hotkey: bool,
    pub available_alternatives: Vec<HotkeyDefinition>,
}
```

**Replace with**:
```rust
/// DEPRECATED: Use PreferencesResource from ecs-preferences package instead
/// 
/// This resource is completely redundant with `PreferencesResource` in the
/// `ecs-preferences` package, which contains all these fields plus additional
/// functionality. This struct is kept for backward compatibility only and will
/// be removed in a future version.
/// 
/// Migration path:
/// - Replace `HotkeyCaptureUIState` → `PreferencesResource` from `ecs_preferences`
/// - Replace `visible` → `is_visible`
/// - All other fields have identical names and types
#[deprecated(
    since = "0.2.0",
    note = "Use PreferencesResource from ecs-preferences package instead. \
            This resource is redundant and will be removed in the next major version."
)]
#[derive(Resource, Default)]
pub struct HotkeyCaptureUIState {
    pub visible: bool,
    pub input_focused: bool,
    pub current_status: HotkeyStatus,
    pub testing_hotkey: bool,
    pub available_alternatives: Vec<HotkeyDefinition>,
}
```

### STEP 2: Remove Plugin Registration
**File**: `packages/ecs-hotkey/src/lib.rs`
**Line**: 221

**Current code**:
```rust
app
    .insert_resource(HotkeyRegistry::default())
    .insert_resource(HotkeyCaptureState::default())
    .insert_resource(HotkeyCaptureUIState::default())  // REMOVE THIS LINE
    .insert_resource(MultiCaptureState::default())
```

**Replace with**:
```rust
app
    .insert_resource(HotkeyRegistry::default())
    .insert_resource(HotkeyCaptureState::default())
    // HotkeyCaptureUIState removed - use PreferencesResource from ecs-preferences instead
    .insert_resource(MultiCaptureState::default())
```

### STEP 3: Remove Redundant Utility Function
**File**: `packages/ecs-hotkey/src/resources.rs`
**Line**: 636-645

**Current code**:
```rust
/// Scan for available hotkey combinations using user preferences
/// Zero allocation preference scanning with intelligent conflict detection
#[inline]
pub fn scan_for_available_hotkeys(
    capture_ui_state: &mut HotkeyCaptureUIState,
    hotkey_prefs: &HotkeyPreferences,
) {
    // Use preferred_combinations from HotkeyPreferences instead of hardcoded list
    // This ensures user preferences are respected for conflict scanning
    capture_ui_state.available_alternatives = hotkey_prefs.preferred_combinations.clone();
}
```

**Replace with**:
```rust
/// DEPRECATED: Use load_preferred_alternatives from ecs-preferences instead
/// 
/// This function is redundant with `load_preferred_alternatives` in
/// `packages/ecs-preferences/src/resources.rs` which performs the same operation
/// on `PreferencesResource`.
#[deprecated(
    since = "0.2.0",
    note = "Use load_preferred_alternatives from ecs-preferences package instead"
)]
#[inline]
pub fn scan_for_available_hotkeys(
    capture_ui_state: &mut HotkeyCaptureUIState,
    hotkey_prefs: &HotkeyPreferences,
) {
    capture_ui_state.available_alternatives = hotkey_prefs.preferred_combinations.clone();
}
```

**Alternative**: Delete the function entirely if no external code depends on it.

## EXISTING ARCHITECTURE (NO CHANGES NEEDED)

### PreferencesPlugin Structure
**File**: `packages/ecs-preferences/src/plugin.rs`

Already correctly registers `PreferencesResource`:

```rust
impl Plugin for PreferencesPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.init_resource::<PreferencesResource>();  // ✓ Already correct
        
        // Events
        app.add_event::<PreferencesShowRequested>()
            .add_event::<PreferencesHideRequested>()
            .add_event::<PreferencesVisibilityChanged>()
            .add_event::<PreferencesSaveRequested>()
            .add_event::<PreferencesSaved>()
            .add_event::<HotkeyRecordingStarted>()
            .add_event::<HotkeyRecorded>()
            .add_event::<HotkeyRecordingCancelled>();
        
        // Systems
        app.add_systems(Update, (
            process_save_requests,
            process_recording_started,
            process_recording_cancelled,
            process_hotkey_recorded,
        ));
    }
}
```

### PreferencesUIPlugin Structure
**File**: `packages/ecs-preferences/src/ui/plugin.rs`

Already correctly implements UI systems:

```rust
impl Plugin for PreferencesUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            manage_preferences_visibility,
            handle_close_button,
            handle_recorder_button,
            handle_save_button,
            handle_cancel_button,
            update_hotkey_display,
            manage_recording_overlay,
        ));
    }
}
```

### Event-Driven Architecture
**File**: `packages/ecs-preferences/src/events.rs`

Already has complete event system for preferences UI:

```rust
// Visibility events
pub struct PreferencesShowRequested;
pub struct PreferencesHideRequested;
pub struct PreferencesVisibilityChanged { pub is_visible: bool }

// Save events
pub struct PreferencesSaveRequested { pub hotkey: HotkeyDefinition }
pub struct PreferencesSaved;

// Recording events
pub struct HotkeyRecordingStarted;
pub struct HotkeyRecorded { pub hotkey: HotkeyDefinition, pub has_conflict: bool, ... }
pub struct HotkeyRecordingCancelled;
```

## BONUS DISCOVERY: HotkeyStatus Duplication

**Also duplicated** (lower priority, out of scope for this task):
- `HotkeyStatus` enum defined in both:
  - `packages/ecs-hotkey/src/resources.rs:222-235`
  - `packages/ecs-preferences/src/resources.rs:7-17`
- Both have identical variants: `Empty`, `Valid`, `Conflict(String)`, `Testing`, `TestSuccess`, `TestFailed(String)`
- Recommendation: Consolidate to one location (suggest ecs-preferences) in a future refactor

## DEFINITION OF DONE

- [ ] `HotkeyCaptureUIState` struct marked with `#[deprecated]` attribute in `packages/ecs-hotkey/src/resources.rs`
- [ ] Clear deprecation documentation added explaining migration to `PreferencesResource`
- [ ] `.insert_resource(HotkeyCaptureUIState::default())` removed from `packages/ecs-hotkey/src/lib.rs:221`
- [ ] `scan_for_available_hotkeys` function either deprecated or removed from `packages/ecs-hotkey/src/resources.rs`
- [ ] Code compiles without errors
- [ ] No new warnings introduced (deprecation warnings expected in consuming code)
- [ ] `PreferencesResource` continues to function correctly (no changes needed to ecs-preferences)

## MIGRATION GUIDE (For Future Consumers)

If any code in the future references `HotkeyCaptureUIState`, use this migration guide:

### Import Changes
```rust
// OLD (deprecated)
use ecs_hotkey::HotkeyCaptureUIState;

// NEW (correct)
use ecs_preferences::PreferencesResource;
```

### Field Name Changes
```rust
// OLD
ui_state.visible

// NEW
prefs.is_visible

// All other fields have identical names:
// - input_focused
// - current_status
// - testing_hotkey
// - available_alternatives
```

### Function Changes
```rust
// OLD (deprecated)
use ecs_hotkey::scan_for_available_hotkeys;
scan_for_available_hotkeys(&mut ui_state, &hotkey_prefs);

// NEW (correct)
use ecs_preferences::load_preferred_alternatives;
load_preferred_alternatives(&mut prefs, &hotkey_prefs);
```

## REFERENCE FILES

### Files Modified
1. [`/Volumes/samsung_t9/action-items/packages/ecs-hotkey/src/resources.rs`](../packages/ecs-hotkey/src/resources.rs) - Lines 257-278, 636-645
2. [`/Volumes/samsung_t9/action-items/packages/ecs-hotkey/src/lib.rs`](../packages/ecs-hotkey/src/lib.rs) - Line 221

### Reference Files (No Changes)
1. [`/Volumes/samsung_t9/action-items/packages/ecs-preferences/src/resources.rs`](../packages/ecs-preferences/src/resources.rs) - PreferencesResource definition
2. [`/Volumes/samsung_t9/action-items/packages/ecs-preferences/src/events.rs`](../packages/ecs-preferences/src/events.rs) - Event definitions
3. [`/Volumes/samsung_t9/action-items/packages/ecs-preferences/src/plugin.rs`](../packages/ecs-preferences/src/plugin.rs) - Plugin registration
4. [`/Volumes/samsung_t9/action-items/packages/ecs-preferences/src/ui/plugin.rs`](../packages/ecs-preferences/src/ui/plugin.rs) - UI plugin registration

## CONSTRAINTS

- **NO unit tests** - another team handles testing
- **NO benchmarks** - another team handles performance
- **NO extensive documentation** - keep changes minimal
- Focus on deprecation and cleanup in `./src`
- Maintain backward compatibility through deprecation (don't break existing code)

## ARCHITECTURAL NOTES

### Why PreferencesResource is the Correct Location
1. **Separation of Concerns**: UI state belongs in the preferences package, not the backend hotkey package
2. **Prevents Circular Dependencies**: Hotkey backend should not depend on UI concepts
3. **Already Implemented**: The correct architecture already exists in ecs-preferences
4. **Event-Driven Communication**: ecs-preferences properly uses events for inter-package communication

### Package Responsibilities
- **ecs-hotkey**: Backend logic for hotkey registration, capture logic, conflict detection
- **ecs-preferences**: UI state, preferences management, user interaction
- **ecs-preferences/ui**: UI rendering, window management, user input handling

This refactor maintains clean architectural boundaries.
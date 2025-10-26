# Task: Implement Proper Wizard Navigation Logic

## OBJECTIVE

Fix 3 temporary "for now" implementations in permission wizard navigation that bypass proper logic checks. These placeholders prevent the wizard from properly validating user progress and handling skip/cancel actions correctly.

**Core Goal**: Replace temporary placeholder logic with actual state validation using existing infrastructure in the wizard system.

## PRIORITY

**P2 - HIGH** - Affects wizard UX and completion detection. Users can advance through wizard steps without meeting requirements, and skip/cancel behave identically (both just go back).

## CODEBASE ANALYSIS

### Existing Infrastructure

The wizard system has robust infrastructure already in place:

1. **WizardPermissionManager** ([../packages/ecs-permissions/src/wizard/systems/permissions.rs](../packages/ecs-permissions/src/wizard/systems/permissions.rs))
   - `all_required_permissions_granted()` - checks if required permissions are authorized
   - `calculate_progress()` - returns (completed, total) permission counts
   - `get_cached_status(permission_type)` - retrieves current permission status
   - `required_permissions` - vec of PermissionType that must be granted
   - `optional_permissions` - vec of PermissionType that are nice-to-have

2. **HotkeyRegistry** ([../packages/ecs-hotkey/src/resources.rs](../packages/ecs-hotkey/src/resources.rs))
   - `registered_hotkeys: HashMap<String, RegisteredHotkey>` - tracks all registered hotkeys
   - Can check if `.len() > 0` to see if any hotkeys configured

3. **HotkeyWizardManager** ([../packages/ecs-permissions/src/wizard/systems/permissions.rs](../packages/ecs-permissions/src/wizard/systems/permissions.rs#L706))
   - Local resource tracking hotkey configuration during wizard
   - `configured_hotkeys: HashMap<String, HotkeyConfiguration>` - hotkeys user has set up
   - `registration_results: HashMap<String, HotkeyRegistrationResult>` - registration outcomes
   - `setup_skipped: bool` - whether user skipped hotkey setup

4. **WizardCancelRequest** ([../packages/ecs-permissions/src/wizard/events.rs](../packages/ecs-permissions/src/wizard/events.rs))
   - Already has `WizardCancelReason::UserSkipped` and `WizardCancelReason::UserCanceled`
   - `save_progress: bool` field to control partial progress saving
   - Handler exists in navigation.rs at `handle_wizard_cancellation()`

### Current "For Now" Implementations

#### 1. Hotkey Setup Auto-Advance
**Location**: [../packages/ecs-permissions/src/wizard/systems/navigation.rs:307](../packages/ecs-permissions/src/wizard/systems/navigation.rs#L307)

```rust
WizardState::SettingUpHotkeys => {
    // For now, always allow advance from hotkey setup
    // In the future, this could check if hotkeys are properly configured
    true
},
```

**Problem**: Always returns `true`, allowing users to skip hotkey setup without configuring anything.

#### 2. Progress Auto-Complete
**Location**: [../packages/ecs-permissions/src/wizard/systems/progress.rs:245](../packages/ecs-permissions/src/wizard/systems/progress.rs#L245)

```rust
WizardState::SettingUpHotkeys => {
    // Complete wizard when hotkeys are configured
    // For now, auto-complete after a brief delay
    step_complete_events.write(WizardStepComplete::new(
        WizardState::SettingUpHotkeys,
        WizardState::Complete,
    ));
},
```

**Problem**: Immediately triggers completion without checking if permissions are granted OR hotkeys configured.

#### 3. Skip/Cancel Actions
**Location**: [../packages/ecs-permissions/src/wizard/events.rs:333-334](../packages/ecs-permissions/src/wizard/events.rs#L333)

```rust
WizardAction::Skip => Self::back(), // For now, treat skip as back
WizardAction::Cancel => Self::back(), // For now, treat cancel as back
```

**Problem**: Both actions do the same thing (go back), when they should have distinct behaviors.

## IMPLEMENTATION DETAILS

### FIX 1: Hotkey Setup Auto-Advance Check

**File**: `packages/ecs-permissions/src/wizard/systems/navigation.rs`

**Current Code** (line ~307 in `validate_current_step`):
```rust
WizardState::SettingUpHotkeys => {
    // For now, always allow advance from hotkey setup
    // In the future, this could check if hotkeys are properly configured
    true
},
```

**Replace With**:
```rust
WizardState::SettingUpHotkeys => {
    // Hotkey setup is optional - allow advance even if skipped
    // This allows users to configure hotkeys later if desired
    true
}
```

**Rationale**: After examining the codebase, hotkey setup is intentionally optional. The system has a skip button for this step (line 450 in navigation.rs shows Skip button is visible for SettingUpHotkeys state). Users should be able to proceed without configuring hotkeys, as they can set them up later in preferences.

**Change Required**: Update the comment to accurately reflect the intentional design, remove "for now" language.

### FIX 2: Progress Auto-Complete Logic

**File**: `packages/ecs-permissions/src/wizard/systems/progress.rs`

**Current Code** (line ~245 in `update_wizard_progress`):
```rust
WizardState::SettingUpHotkeys => {
    // Complete wizard when hotkeys are configured
    // For now, auto-complete after a brief delay
    step_complete_events.write(WizardStepComplete::new(
        WizardState::SettingUpHotkeys,
        WizardState::Complete,
    ));
},
```

**Replace With**:
```rust
WizardState::SettingUpHotkeys => {
    // Auto-advance to complete only if all required permissions granted
    // Hotkeys are optional - wizard can complete without them
    if tracker.all_required_permissions_granted() {
        step_complete_events.write(WizardStepComplete::new(
            WizardState::SettingUpHotkeys,
            WizardState::Complete,
        ));
    }
},
```

**Explanation**: 
- Use existing `tracker.all_required_permissions_granted()` method
- Don't block on hotkey configuration (it's optional per the design)
- Only auto-advance when required permissions are actually granted
- This prevents wizard from completing prematurely if user skips back and permissions get revoked

### FIX 3: Implement Proper Skip vs Cancel

**File**: `packages/ecs-permissions/src/wizard/events.rs`

**Current Code** (line ~333 in `WizardNavigationRequest::new()`):
```rust
WizardAction::Skip => Self::back(), // For now, treat skip as back
WizardAction::Cancel => Self::back(), // For now, treat cancel as back
```

**Replace With**:
```rust
WizardAction::Skip => {
    // Skip current step and continue forward
    if let Some(next_state) = current_state.next_state() {
        Self::skip_to(next_state)
    } else {
        Self::next() // Already at end, just advance
    }
}
WizardAction::Cancel => {
    // This is handled separately by WizardCancelRequest event
    // Don't create navigation request, let cancel handler deal with it
    Self::back() // Temporary fallback, prefer using WizardCancelRequest
}
```

**Additional Changes Required**:

1. **Update Skip button handler** in `packages/ecs-permissions/src/wizard/ui/observers.rs` or wherever Skip button click is handled:

```rust
// When Skip button is clicked (in SettingUpHotkeys state)
fn handle_skip_button_click(
    mut navigation_events: EventWriter<WizardNavigationRequest>,
    wizard_state: Res<State<WizardState>>,
) {
    if *wizard_state.get() == WizardState::SettingUpHotkeys {
        // Send skip-to-complete request
        navigation_events.write(WizardNavigationRequest::skip_to(WizardState::Complete));
    }
}
```

2. **Update Cancel button handler** to use WizardCancelRequest:

```rust
// When Cancel button is clicked
fn handle_cancel_button_click(
    mut cancel_events: EventWriter<WizardCancelRequest>,
) {
    cancel_events.write(WizardCancelRequest::user_canceled());
}
```

**Note**: The `handle_wizard_cancellation()` system in navigation.rs already exists and properly handles:
- Saving partial progress when `save_progress: true`
- Transitioning to `WizardState::NotStarted` for UserCanceled
- Transitioning to `WizardState::Complete` for UserSkipped

## FILE STRUCTURE REFERENCE

```
packages/ecs-permissions/src/wizard/
├── events.rs                    # WizardAction, WizardNavigationRequest, WizardCancelRequest
├── states.rs                    # WizardState enum with next_state(), previous_state()
├── components.rs                # PermissionCard, WizardRoot, navigation button components
├── systems/
│   ├── navigation.rs           # FIX 1: validate_current_step() - line ~307
│   │                           # Already has: handle_wizard_cancellation() - line ~77
│   ├── progress.rs             # FIX 2: update_wizard_progress() - line ~245
│   └── permissions.rs          # WizardPermissionManager, HotkeyWizardManager
└── ui/
    └── observers.rs            # FIX 3: Button click handlers (may need updating)

packages/ecs-hotkey/src/
├── lib.rs                      # HotkeyPlugin, system sets
├── resources.rs                # HotkeyRegistry with registered_hotkeys HashMap
└── events.rs                   # HotkeyRegisterCompleted, etc.
```

## IMPLEMENTATION PATTERNS FROM CODEBASE

### Pattern 1: Accessing WizardPermissionManager

The manager is passed as `Option<Res<WizardPermissionManager>>` because it may not be initialized:

```rust
pub fn validate_current_step(
    current_state: WizardState,
    permission_manager: Option<&WizardPermissionManager>,
) -> bool {
    // ...
    if let Some(manager) = permission_manager {
        manager.all_required_permissions_granted()
    } else {
        true // Allow advance if no manager available
    }
}
```

### Pattern 2: Using WizardProgressTracker

In progress.rs, use the tracker resource:

```rust
pub fn update_wizard_progress(
    mut tracker: ResMut<WizardProgressTracker>,
    // ... other params
) {
    if tracker.all_required_permissions_granted() {
        // Advance wizard
    }
}
```

### Pattern 3: Event-Driven State Changes

Navigation uses events, not direct state manipulation:

```rust
// DON'T DO THIS:
next_wizard_state.set(WizardState::Complete);

// DO THIS:
step_complete_events.write(WizardStepComplete::new(
    WizardState::SettingUpHotkeys,
    WizardState::Complete,
));
```

### Pattern 4: Handling Optional Resources

Local resources may not persist across system calls:

```rust
pub fn some_system(
    mut local_resource: Local<HotkeyWizardManager>,
) {
    // Local resources reset between calls in some contexts
    // Don't rely on them for cross-system state
}
```

## DEFINITION OF DONE

### Behavior Changes

1. **Hotkey Setup Validation**
   - Comment accurately describes that hotkey setup is optional
   - "For now" language removed
   - Users can still advance without configuring hotkeys (intentional design)

2. **Wizard Auto-Completion**
   - Wizard only auto-completes when `all_required_permissions_granted()` returns true
   - Hotkey configuration is not required for completion (optional feature)
   - If user reverts permissions, wizard does not auto-complete

3. **Skip vs Cancel Behavior**
   - **Skip Button**: Advances to next step/completion, saves progress
   - **Cancel Button**: Exits wizard to NotStarted, saves partial progress
   - Both actions have distinct, clear behaviors
   - All "for now" comments removed

### Code Quality

- All three "for now" comments replaced with accurate descriptions
- No new compiler warnings introduced
- Existing event handlers (cancel, navigation) work correctly
- Code follows existing patterns in wizard codebase

### User Experience

- Wizard doesn't complete prematurely if permissions aren't granted
- Skip and Cancel buttons behave as users expect
- Hotkey setup remains optional (users can configure later)
- Partial progress saved on both skip and cancel

## CONSTRAINTS

### What NOT to Do

- ❌ DO NOT add new unit tests or functional tests
- ❌ DO NOT write benchmarks for this feature
- ❌ DO NOT create extensive documentation files
- ❌ DO NOT change the scope (hotkeys remain optional)
- ❌ DO NOT make breaking changes to existing event types

### What TO Do

- ✅ Use existing `WizardPermissionManager.all_required_permissions_granted()`
- ✅ Use existing `WizardProgressTracker.all_required_permissions_granted()`
- ✅ Use existing `WizardCancelRequest` event infrastructure
- ✅ Keep hotkey setup optional (don't require hotkeys for completion)
- ✅ Ensure both skip and cancel save partial progress
- ✅ Follow existing code patterns shown in this document

## IMPLEMENTATION CHECKLIST

- [ ] Update comment in `navigation.rs:~307` to remove "for now" language
- [ ] Replace auto-complete logic in `progress.rs:~245` with permission check
- [ ] Update Skip action in `events.rs:~333` to use skip_to()
- [ ] Update Cancel action in `events.rs:~334` with comment about WizardCancelRequest
- [ ] Verify Skip button emits correct navigation request
- [ ] Verify Cancel button emits WizardCancelRequest event
- [ ] Run `cargo check` to ensure no compilation errors
- [ ] Manual verification: wizard doesn't complete without permissions
- [ ] Manual verification: skip and cancel behave differently

## NOTES

### Skip vs Cancel Philosophy

- **Skip** = "I'll do this later, keep going" - User wants to continue wizard journey
- **Cancel** = "Stop the wizard entirely" - User wants to exit wizard completely  
- Both save progress so user can resume later
- Cancel returns to NotStarted, Skip continues to next step/Complete

### Hotkey Setup Design

- Hotkeys are **OPTIONAL** by design - not required for wizard completion
- Users can configure hotkeys later via preferences
- Permission setup is the critical requirement
- Wizard can complete successfully with just permissions granted

### Existing Code Handles Most Scenarios

- `handle_wizard_cancellation()` already implements save-and-exit logic
- `WizardPermissionManager` already tracks required vs optional permissions
- `WizardProgressTracker` already calculates completion status
- `WizardNavigationRequest::skip_to()` already exists for state skipping

**The infrastructure is complete - this task just connects it properly.**

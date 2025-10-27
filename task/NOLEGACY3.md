# Task: Fix Permission Screen "Legacy" Description

## OBJECTIVE

Remove misleading "legacy" reference from address book permission description - the address book API is standard, not deprecated.

---

## PRIORITY

**P2 - HIGH** - User-facing text that incorrectly implies the feature is deprecated.

---

## SUBTASK 1: Update Address Book Permission Description

**File**: `packages/ecs-permissions/src/wizard/ui/permission_screens.rs`  
**Line**: 213 (approximately - in address book permission builder)

**What to Change**: Permission description text

**Before**:
```rust
PermissionScreenContent::builder(PermissionType::AddressBook)
    .title("Address Book Access Required")
    .description("This application needs access to your address book for legacy contact management features")
    .platform_instructions(instructions)
```

**After**:
```rust
PermissionScreenContent::builder(PermissionType::AddressBook)
    .title("Address Book Access Required")
    .description("This application needs access to your address book to manage and sync contacts")
    .platform_instructions(instructions)
```

**Why**: 
- The address book API is the standard way to access contacts - not legacy
- Removes user-facing text that implies the feature is deprecated
- More accurate and professional description

---

## SUBTASK 2: Verify Changes

**Command**:
```bash
rg -i "legacy" --type rust packages/ecs-permissions/src/wizard/ui/permission_screens.rs
```

**Expected Result**: Zero matches for "legacy" in address book permission context

---

## SUBTASK 3: Verify Compilation

**Command**:
```bash
cargo check --workspace
```

**Expected Result**: Clean compilation with no errors

---

## DEFINITION OF DONE

- [ ] "legacy contact management features" changed to "to manage and sync contacts"
- [ ] No "legacy" references in address book permission description
- [ ] User-facing text is accurate and professional
- [ ] Code compiles successfully

---

## CONSTRAINTS

- DO NOT write unit tests
- DO NOT write benchmarks
- DO NOT create documentation files
- DO NOT change any functional code - only description text
- DO verify compilation after changes

---

## RESEARCH NOTES

### Address Book API Status

The address book/contacts API:
- **Standard platform API** on macOS, iOS, and other platforms
- **Not deprecated** - actively maintained by platform vendors
- **Not legacy** - the modern way to access contact information

### Why This Matters

User-facing permission text that says "legacy":
- Confuses users about whether the feature is deprecated
- Makes the application seem outdated
- Implies the feature might be removed soon
- Reduces user confidence in granting permission

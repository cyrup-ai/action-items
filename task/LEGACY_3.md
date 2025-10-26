# Task: Remove Misleading "Legacy" Description from AddressBook Permission

## OBJECTIVE
Remove the word "legacy" from the AddressBook permission description that misleadingly suggests the feature is deprecated when it's actually a legitimate, current permission type.

## PRIORITY
P3 - MEDIUM - Misleading user-facing text that doesn't affect functionality

## PROBLEM ANALYSIS

### What This Task Is Really About
The original task description was misleading. This is **NOT** about renaming a struct field called `.legacy`. 

**The actual issue:** Line 213 in `permission_screens.rs` contains a description string that reads:
```rust
.description("This application needs access to your address book for legacy contact management features")
```

The word **"legacy"** in this user-facing description text makes it sound like:
- The AddressBook permission is deprecated
- This is an old/outdated feature
- Users shouldn't rely on it

This is **FALSE** and **MISLEADING**.

### Technical Context: AddressBook vs Contacts

Both `PermissionType::AddressBook` and `PermissionType::Contacts` are **valid, current permission types** in the system. They are NOT the same thing:

#### On macOS:
- **Contacts** → Uses modern `CNContactStore` from Contacts.framework
  - Implementation: [`packages/ecs-permissions/src/platforms/macos/contacts_permissions.rs`](../packages/ecs-permissions/src/platforms/macos/contacts_permissions.rs)
  - Direct framework integration
  
- **AddressBook** → Uses TCC (Transparency, Consent, and Control) permission system
  - Implementation: Handled through [`packages/ecs-permissions/src/platforms/macos/tcc_permissions.rs`](../packages/ecs-permissions/src/platforms/macos/tcc_permissions.rs)
  - TCC database-level permission checking
  - See handler routing: [`packages/ecs-permissions/src/platforms/macos/handler.rs`](../packages/ecs-permissions/src/platforms/macos/handler.rs) lines 64-67

#### On Windows:
- Both map to the same underlying permission: `app_capabilities::check_contacts()`
- See: [`packages/ecs-permissions/src/platforms/windows/mod.rs`](../packages/ecs-permissions/src/platforms/windows/mod.rs) lines 49-50

#### On Linux:
- Both map to the same underlying permission: `dbus_services::check_contacts()`
- See: [`packages/ecs-permissions/src/platforms/linux/mod.rs`](../packages/ecs-permissions/src/platforms/linux/mod.rs) lines 33-34

### Why "Legacy" Is Wrong

AddressBook is NOT a legacy/deprecated feature:
1. It's a distinct permission type with its own implementation path
2. On macOS, it provides TCC-level permission checking (different from Contacts framework)
3. It's actively used and maintained in the codebase
4. Other descriptions in the codebase describe it correctly without "legacy":
   - [`components.rs:196`](../packages/ecs-permissions/src/wizard/components.rs#L196): "Address book access for contact management"
   - [`components.rs:242`](../packages/ecs-permissions/src/wizard/components.rs#L242): "Required for address book integration"

## FILE TO FIX

**Primary file:**
`packages/ecs-permissions/src/wizard/ui/permission_screens.rs` - Line 213

**Function:** `create_address_book_permission_screen()`

## THE FIX

### Current Code (Line 211-218):
```rust
PermissionScreenContent::builder(PermissionType::AddressBook)
    .title("Address Book Access Required")
    .description("This application needs access to your address book for legacy contact management features")
    .platform_instructions(instructions)
    .icon('📇')
    .button_text("Grant Address Book Access")
    .requires_elevation(false)
    .build()
```

### Recommended Fix Option 1 (Simplest - Remove "legacy"):
```rust
PermissionScreenContent::builder(PermissionType::AddressBook)
    .title("Address Book Access Required")
    .description("This application needs access to your address book for contact management features")
    .platform_instructions(instructions)
    .icon('📇')
    .button_text("Grant Address Book Access")
    .requires_elevation(false)
    .build()
```

### Alternative Fix Option 2 (Clarify TCC distinction):
```rust
PermissionScreenContent::builder(PermissionType::AddressBook)
    .title("Address Book Access Required")
    .description("This application needs access to your address book for system-level contact management")
    .platform_instructions(instructions)
    .icon('📇')
    .button_text("Grant Address Book Access")
    .requires_elevation(false)
    .build()
```

### Alternative Fix Option 3 (Match existing patterns):
Following the pattern from Contacts permission (line 194):
```rust
PermissionScreenContent::builder(PermissionType::AddressBook)
    .title("Address Book Access Required")
    .description("This application needs access to your address book for contact organization and management")
    .platform_instructions(instructions)
    .icon('📇')
    .button_text("Grant Address Book Access")
    .requires_elevation(false)
    .build()
```

## IMPLEMENTATION STEPS

### Step 1: Open the file
```bash
# File location
packages/ecs-permissions/src/wizard/ui/permission_screens.rs
```

### Step 2: Locate the function
Navigate to function `create_address_book_permission_screen()` around line 202-219

### Step 3: Find the exact line
Line 213 contains:
```rust
.description("This application needs access to your address book for legacy contact management features")
```

### Step 4: Apply the fix
Replace "legacy contact management features" with one of:
- **Recommended:** "contact management features" (simplest, matches other permissions)
- **Alternative:** "system-level contact management" (clarifies TCC distinction)
- **Alternative:** "contact organization and management" (matches Contacts permission pattern)

### Step 5: Verify compilation
```bash
cd packages/ecs-permissions
cargo check
```

### Step 6: Verify no other occurrences
```bash
# Search for "legacy" in permission descriptions
rg -i "legacy" packages/ecs-permissions/src/wizard/ui/permission_screens.rs
```

Expected: No matches after the fix

## SCOPE VERIFICATION

### Files Checked for "legacy" Usage:
- ✅ `permission_screens.rs` - **ONLY location with the problematic "legacy" text**
- ✅ `components.rs` - Uses correct descriptions ("Address book access for contact management")
- ✅ `types.rs` - Enum definition only, no descriptions
- ✅ Platform handlers - No user-facing text

### Conclusion:
This is a **ONE LINE CHANGE** - only the description string on line 213 needs modification.

## DEFINITION OF DONE

- [ ] Line 213 updated with new description (without "legacy")
- [ ] File compiles without errors (`cargo check`)
- [ ] No instances of "legacy" remain in permission screen descriptions
- [ ] Description accurately reflects that AddressBook is a valid, current permission type

## REFERENCE LINKS

### Source Code References:
- **Main fix location:** [`packages/ecs-permissions/src/wizard/ui/permission_screens.rs:213`](../packages/ecs-permissions/src/wizard/ui/permission_screens.rs#L213)
- **Correct description examples:** [`packages/ecs-permissions/src/wizard/components.rs`](../packages/ecs-permissions/src/wizard/components.rs) (lines 196, 242)
- **Permission type enum:** [`packages/ecs-permissions/src/types.rs`](../packages/ecs-permissions/src/types.rs) (line 28)
- **macOS implementation routing:** [`packages/ecs-permissions/src/platforms/macos/handler.rs`](../packages/ecs-permissions/src/platforms/macos/handler.rs) (lines 64-67)
- **Contacts permission comparison:** [`packages/ecs-permissions/src/wizard/ui/permission_screens.rs:194`](../packages/ecs-permissions/src/wizard/ui/permission_screens.rs#L194)

### Architecture Understanding:
```
PermissionType::AddressBook (enum variant)
    ↓
Platform Handler Routes:
    ├─ macOS    → tcc_permissions::check_permission() [TCC database]
    ├─ Windows  → app_capabilities::check_contacts()  [Windows Contacts API]
    └─ Linux    → dbus_services::check_contacts()     [D-Bus Evolution Data Server]
```

## CONSTRAINTS

- ✅ DO make the change to line 213 only
- ✅ DO use one of the recommended description replacements
- ✅ DO verify with `cargo check`
- ❌ DO NOT change functionality
- ❌ DO NOT rename the enum variant
- ❌ DO NOT modify other files
- ❌ DO NOT write unit tests for this change
- ❌ DO NOT write documentation files

## NOTES

### Why This Matters:
User-facing permission request screens should not suggest that features are "legacy" or deprecated when they are actually current, supported functionality. This creates user confusion and diminishes trust in the permission request.

### Related Permissions:
For reference, the Contacts permission description (which is separate but related) says:
> "This application needs access to your contacts for address book and contact management features"

The AddressBook description should be similarly clear and non-deprecating.
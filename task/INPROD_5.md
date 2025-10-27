# Task: Fix Field Name Mismatch in Camera Permission Example

## CRITICAL BUG - MUST FIX IMMEDIATELY

The camera permission example has **incorrect field names** in the `handle_permission_changes()` function that will cause runtime errors.

## LOCATION
**File**: `examples/permissions/camera_permission.rs:45-54`

## THE PROBLEM

The function uses field names that don't exist on the `PermissionChanged` struct:
- Uses `event.permission_type` → should be `event.typ`
- Uses `event.new_status` → should be `event.status`

## THE FIX

### Current Broken Code (Lines 45-54):
```rust
if event.permission_type == PermissionType::Camera {
    match event.new_status {
        PermissionStatus::Authorized => {
            info!("Camera access granted! You can now use the camera.");
        }
        PermissionStatus::Denied => {
            warn!("Camera access denied. Some features may not work.");
        }
        _ => {
            info!("Camera permission changed to: {}", event.new_status);
        }
    }
}
```

### Corrected Code:
```rust
if event.typ == PermissionType::Camera {
    match event.status {
        PermissionStatus::Authorized => {
            info!("Camera access granted! You can now use the camera.");
        }
        PermissionStatus::Denied => {
            warn!("Camera access denied. Some features may not work.");
        }
        _ => {
            info!("Camera permission changed to: {}", event.status);
        }
    }
}
```

## VERIFICATION

The `PermissionChanged` struct is defined in `packages/ecs-permissions/src/plugin.rs:16-19`:
```rust
#[derive(Event)]
pub struct PermissionChanged {
    pub typ: PermissionType,
    pub status: PermissionStatus,
}
```

## IMPLEMENTATION STEPS

1. Edit line 45: change `event.permission_type` → `event.typ`
2. Edit line 46: change `event.new_status` → `event.status`  
3. Edit line 54: change `event.new_status` → `event.status`
4. Test: `cargo run --example camera_permission`
5. Verify permission status changes are logged correctly

## WHY THIS MATTERS

- Example code must be production-quality and actually work
- Developers learn from examples - broken code damages trust
- Field name mismatches cause confusing runtime errors
- This blocks the main task completion (INPROD_5) which is otherwise done

## DEFINITION OF DONE

- [ ] Line 45: `event.typ` used instead of `event.permission_type`
- [ ] Line 46: `event.status` used instead of `event.new_status`
- [ ] Line 54: `event.status` used instead of `event.new_status`
- [ ] Example compiles without warnings
- [ ] Example runs and correctly logs permission changes
- [ ] No field access errors at runtime

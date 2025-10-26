# PRODFIX_5: Fix macOS Notification Authorization Initialization

## OBJECTIVE
Fix the missing authorization initialization in the Deno runtime's macOS notification backend. The current implementation has complete notification display logic but fails to initialize authorization status, causing all notifications to fail with a 3-second timeout.

## PRIORITY
**P1 - HIGH (Broken Functionality)**

## LOCATION
`packages/core/src/runtime/deno/notifications/macos.rs`

## ROOT CAUSE ANALYSIS

### Current State
The file contains **FULLY IMPLEMENTED** methods at lines 44-60 and 145-241:
- `ensure_authorized()` - Complete synchronous authorization checker using parking_lot Mutex/Condvar
- `show_notification()` - Complete UserNotifications implementation with RcBlock completion handlers

### The Actual Bug
The `auth_status` field (Arc<(Mutex<Option<UNAuthorizationStatus>>, Condvar)>) is **NEVER POPULATED**. The `ensure_authorized()` method waits for this field to be set, but no code exists to:
1. Check the current authorization status via `getNotificationSettingsWithCompletionHandler()`
2. Request authorization if needed via `requestAuthorizationWithOptions_completionHandler()`
3. Update the `auth_status` field with the result

This causes `ensure_authorized()` to always timeout after 3 seconds (line 50) and return `false`, which makes `show_notification()` fail with `NotificationError::PermissionDenied` (line 151-154).

## COMPARISON WITH WORKING ECS IMPLEMENTATION

The ECS notifications backend at [`packages/ecs-notifications/src/backends/macos.rs`](../../packages/ecs-notifications/src/backends/macos.rs) shows the correct async pattern for checking authorization (but not requesting it):

### ECS Authorization Check (Lines 44-103)
```rust
pub async fn check_authorization(&self) -> NotificationResult<bool> {
    let (tx, rx) = oneshot::channel();
    // ... spawns async task ...
    let center = UNUserNotificationCenter::currentNotificationCenter();
    
    let block = block2::StackBlock::new(
        move |settings: std::ptr::NonNull<UNNotificationSettings>| {
            let auth_status = unsafe { settings.as_ref() }.authorizationStatus();
            let is_authorized = matches!(
                auth_status,
                UNAuthorizationStatus::Authorized | UNAuthorizationStatus::Provisional
            );
            // Send result through channel
        }
    );
    
    center.getNotificationSettingsWithCompletionHandler(&block);
}
```

### Key Architectural Differences
- **ECS version**: Uses async/await with Bevy AsyncComputeTaskPool and `StackBlock` for async callbacks
- **Deno version**: Uses synchronous blocking with parking_lot Mutex/Condvar and `RcBlock` for sync callbacks
- **ECS version**: Only checks authorization status, doesn't request it
- **Deno version**: Needs to both check AND request authorization on-demand

**Important**: The ECS version uses `StackBlock` because it's in an async context. The Deno version correctly uses `RcBlock` because it needs reference counting for the synchronous blocking pattern with Mutex/Condvar.

## CODEBASE PATTERNS FOR OBJC2 COMPLETION HANDLERS

### RcBlock Pattern in Permissions (Already Working)

The codebase has multiple examples of RcBlock with completion handlers in [`packages/ecs-permissions/src/platforms/macos/`](../../packages/ecs-permissions/src/platforms/macos/):

#### AV Permissions Pattern (av_permissions.rs:62-70)
```rust
let handler = RcBlock::new(move |granted: Bool| {
    let status = if granted.as_bool() {
        PermissionStatus::Authorized
    } else {
        PermissionStatus::Denied
    };
    let _ = tx.send(Ok(status));
});

unsafe {
    AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &handler);
}
```

This shows the exact pattern for authorization requests with a Bool parameter.

#### Existing RcBlock Pattern in Deno Notifications (macos.rs:183-195)
```rust
let completion_block = RcBlock::new(move |error: *mut NSError| {
    let (lock, cvar) = &*completion_clone;
    {
        let mut sent = lock.lock();
        *sent = true;
    }
    cvar.notify_all();
    
    if !error.is_null() {
        let err = unsafe { &*error };
        error!("Failed: {:?}", err);
    }
});

center.addNotificationRequest_withCompletionHandler(&request, Some(&completion_block));
```

This proves the RcBlock + Mutex/Condvar pattern already works correctly in the Deno notification code.

## REQUIRED CHANGES

### Step 1: Add Missing Imports

**Location:** Lines 10-13 in [`packages/core/src/runtime/deno/notifications/macos.rs`](../../packages/core/src/runtime/deno/notifications/macos.rs)

**Current imports:**
```rust
use objc2_user_notifications::{
    UNAuthorizationStatus, UNMutableNotificationContent, UNNotificationRequest,
    UNNotificationSound, UNTimeIntervalNotificationTrigger, UNUserNotificationCenter,
};
```

**Add to existing imports:**
```rust
use objc2::runtime::Bool;
use objc2_user_notifications::{
    UNAuthorizationOptions, UNAuthorizationStatus, UNMutableNotificationContent,
    UNNotificationRequest, UNNotificationSound, UNTimeIntervalNotificationTrigger,
    UNUserNotificationCenter, UNNotificationSettings,
};
```

**Why these imports:**
- `UNAuthorizationOptions` - Bitflags for requestAuthorizationWithOptions (Alert, Sound, Badge)
- `UNNotificationSettings` - Type for getNotificationSettingsWithCompletionHandler callback parameter
- `objc2::runtime::Bool` - Objective-C BOOL type used in requestAuthorizationWithOptions callback

**API References:**
- [UNAuthorizationOptions](https://docs.rs/objc2-user-notifications/0.3.1/objc2_user_notifications/struct.UNAuthorizationOptions.html)
- [objc2::runtime::Bool](https://docs.rs/objc2/0.6.2/objc2/runtime/struct.Bool.html)

### Step 2: Replace `ensure_authorized()` with Active Authorization Logic

**Location:** Lines 44-60 in [`packages/core/src/runtime/deno/notifications/macos.rs`](../../packages/core/src/runtime/deno/notifications/macos.rs)

**Current implementation** (passive waiting - BROKEN):
```rust
fn ensure_authorized(&self) -> bool {
    let (lock, cvar) = &*self.auth_status;
    let mut auth = lock.lock();

    // Wait with timeout for authorization resolution
    let timeout = Duration::from_secs(3);
    let timeout_result = cvar.wait_while_for(&mut auth, |status| status.is_none(), timeout);
    let timed_out = timeout_result.timed_out();

    if timed_out {
        warn!("Authorization request timed out");
        return false;
    }

    matches!(*auth, Some(UNAuthorizationStatus::Authorized))
}
```

**Replace with** (active authorization - FIXED):
```rust
fn ensure_authorized(&self) -> bool {
    let (lock, cvar) = &*self.auth_status;
    
    // Fast path: Check if we already have cached authorization status
    {
        let auth = lock.lock();
        if let Some(status) = *auth {
            return matches!(
                status,
                UNAuthorizationStatus::Authorized | UNAuthorizationStatus::Provisional
            );
        }
    }
    
    // No cached status - actively check and request authorization
    debug!("No cached authorization status, checking with UserNotifications");
    
    autoreleasepool(|_| {
        // Phase 1: Check current authorization status
        let check_complete = Arc::new((Mutex::new(false), Condvar::new()));
        let check_complete_clone = Arc::clone(&check_complete);
        let auth_status_clone = Arc::clone(&self.auth_status);
        
        let settings_block = RcBlock::new(move |settings: std::ptr::NonNull<UNNotificationSettings>| {
            let status = unsafe { settings.as_ref() }.authorizationStatus();
            
            debug!("Current authorization status: {:?}", status);
            
            // Cache the status
            {
                let (lock, cvar) = &*auth_status_clone;
                let mut auth = lock.lock();
                *auth = Some(status);
                cvar.notify_all();
            }
            
            // Signal check complete
            {
                let (lock, cvar) = &*check_complete_clone;
                let mut done = lock.lock();
                *done = true;
                cvar.notify_all();
            }
        });
        
        self.notification_center.getNotificationSettingsWithCompletionHandler(&settings_block);
        
        // Wait for settings check to complete
        let (lock, cvar) = &*check_complete;
        let mut done = lock.lock();
        let timeout_result = cvar.wait_while_for(&mut done, |d| !*d, Duration::from_secs(5));
        
        if timeout_result.timed_out() {
            warn!("Authorization status check timed out");
            return false;
        }
    });
    
    // Check if we need to request authorization
    let needs_request = {
        let auth = lock.lock();
        matches!(*auth, Some(UNAuthorizationStatus::NotDetermined))
    };
    
    if needs_request {
        debug!("Authorization not determined, requesting user permission");
        
        autoreleasepool(|_| {
            let request_complete = Arc::new((Mutex::new(false), Condvar::new()));
            let request_complete_clone = Arc::clone(&request_complete);
            let auth_status_clone = Arc::clone(&self.auth_status);
            
            let request_block = RcBlock::new(move |granted: Bool, error: *mut NSError| {
                if !error.is_null() {
                    let err = unsafe { &*error };
                    error!("Authorization request failed: {:?}", err);
                }
                
                // Update status based on grant result
                let new_status = if granted.as_bool() {
                    UNAuthorizationStatus::Authorized
                } else {
                    UNAuthorizationStatus::Denied
                };
                
                debug!("Authorization request completed: granted={}", granted.as_bool());
                
                // Cache the new status
                {
                    let (lock, cvar) = &*auth_status_clone;
                    let mut auth = lock.lock();
                    *auth = Some(new_status);
                    cvar.notify_all();
                }
                
                // Signal request complete
                {
                    let (lock, cvar) = &*request_complete_clone;
                    let mut done = lock.lock();
                    *done = true;
                    cvar.notify_all();
                }
            });
            
            // Request authorization with standard options
            let options = UNAuthorizationOptions::Alert 
                | UNAuthorizationOptions::Sound 
                | UNAuthorizationOptions::Badge;
            
            self.notification_center.requestAuthorizationWithOptions_completionHandler(
                options,
                &request_block,
            );
            
            // Wait for request to complete
            let (lock, cvar) = &*request_complete;
            let mut done = lock.lock();
            let timeout_result = cvar.wait_while_for(&mut done, |d| !*d, Duration::from_secs(10));
            
            if timeout_result.timed_out() {
                warn!("Authorization request timed out");
                return false;
            }
        });
    }
    
    // Return final authorization status
    let auth = lock.lock();
    matches!(
        *auth,
        Some(UNAuthorizationStatus::Authorized | UNAuthorizationStatus::Provisional)
    )
}
```

### Implementation Details Explained

#### 1. Fast Path Caching (Lines 1-11 of new code)
First checks if `auth_status` is already populated. If yes, returns immediately without any API calls. This makes subsequent notification calls instant.

#### 2. Phase 1: Check Current Status (Lines 19-56)
Uses `getNotificationSettingsWithCompletionHandler()` to query the current authorization state. The RcBlock callback receives `std::ptr::NonNull<UNNotificationSettings>` (not a direct reference) which must be dereferenced with `unsafe { settings.as_ref() }`.

**Pattern matches ECS version** at [`packages/ecs-notifications/src/backends/macos.rs:60-70`](../../packages/ecs-notifications/src/backends/macos.rs#L60-L70)

#### 3. Phase 2: Request If Needed (Lines 58-129)
If status is `NotDetermined`, calls `requestAuthorizationWithOptions_completionHandler()` to show the macOS permission dialog. The callback receives `(granted: Bool, error: *mut NSError)` parameters.

**Pattern matches permissions code** at [`packages/ecs-permissions/src/platforms/macos/av_permissions.rs:62-70`](../../packages/ecs-permissions/src/platforms/macos/av_permissions.rs#L62-L70)

#### 4. RcBlock + Mutex/Condvar Synchronization
Both completion handlers use the same proven pattern from the existing `show_notification()` code (lines 183-195):
- Create Arc-wrapped `(Mutex<bool>, Condvar)` for completion signaling
- Clone Arc into RcBlock closure for thread-safe communication
- Block main thread with `wait_while_for()` until callback completes
- Use `notify_all()` in callback to wake waiting thread

#### 5. No Constructor Changes Required
All work happens lazily in `ensure_authorized()` when first called. No modifications needed to `new_without_db()` or field initialization.

## TECHNICAL CONTEXT

### objc2 Block Completion Handlers

#### RcBlock vs StackBlock
- **RcBlock**: Reference-counted block for synchronous contexts. Used when the block needs to outlive the current scope and be moved into Mutex/Condvar synchronization. Perfect for the Deno runtime's synchronous blocking pattern.
- **StackBlock**: Stack-allocated block for async contexts. Used in ECS version with Bevy's AsyncComputeTaskPool where the block lives only during the async operation.

**Source**: [block2 documentation](https://docs.rs/block2/0.6.1/block2/index.html)

#### UNNotificationSettings Callback Parameter
The callback to `getNotificationSettingsWithCompletionHandler` receives `std::ptr::NonNull<UNNotificationSettings>`, not a direct reference. This is why the ECS version uses:
```rust
move |settings: std::ptr::NonNull<UNNotificationSettings>| {
    let auth_status = unsafe { settings.as_ref() }.authorizationStatus();
}
```

#### Bool Type for Authorization Callback
The `requestAuthorizationWithOptions_completionHandler` callback receives `objc2::runtime::Bool`, Objective-C's BOOL type (not Rust's bool). Convert with `.as_bool()`:
```rust
move |granted: Bool, error: *mut NSError| {
    let new_status = if granted.as_bool() {  // Convert objc Bool to Rust bool
        UNAuthorizationStatus::Authorized
    } else {
        UNAuthorizationStatus::Denied
    };
}
```

**Example**: [`packages/ecs-permissions/src/platforms/macos/av_permissions.rs:62-67`](../../packages/ecs-permissions/src/platforms/macos/av_permissions.rs#L62-L67)

### UNAuthorizationStatus Enum

From objc2-user-notifications:
- `NotDetermined` - User hasn't been asked yet → Trigger permission dialog
- `Denied` - User explicitly denied permission → Cannot show notifications
- `Authorized` - Full authorization granted → Can show notifications
- `Provisional` - Limited authorization (iOS quiet notifications, also valid on macOS) → Can show notifications

**Source**: [UNAuthorizationStatus](https://docs.rs/objc2-user-notifications/0.3.1/objc2_user_notifications/enum.UNAuthorizationStatus.html)

### Authorization Options Bitflags

Combine with `|` operator:
- `UNAuthorizationOptions::Alert` - Show notification alerts (required)
- `UNAuthorizationOptions::Sound` - Play notification sounds
- `UNAuthorizationOptions::Badge` - Update app badge count
- `UNAuthorizationOptions::CriticalAlert` - Bypass Do Not Disturb (requires entitlement)
- `UNAuthorizationOptions::TimeSensitive` - Mark as time-sensitive (macOS 12+)

**Standard request** (what we use):
```rust
let options = UNAuthorizationOptions::Alert 
    | UNAuthorizationOptions::Sound 
    | UNAuthorizationOptions::Badge;
```

**Source**: [UNAuthorizationOptions](https://docs.rs/objc2-user-notifications/0.3.1/objc2_user_notifications/struct.UNAuthorizationOptions.html)

## FILES REQUIRING CHANGES

### Primary Implementation File
- **Path:** [`packages/core/src/runtime/deno/notifications/macos.rs`](../../packages/core/src/runtime/deno/notifications/macos.rs)
- **Lines to modify:** 
  - 10-13: Update imports (add UNAuthorizationOptions, UNNotificationSettings, Bool)
  - 44-60: Replace `ensure_authorized` method body with active authorization logic
- **Existing code to preserve:** Lines 62-296 (`create_content`, `show_notification`, `dismiss`)

### No Changes Needed
- [`packages/ecs-notifications/src/backends/macos.rs`](../../packages/ecs-notifications/src/backends/macos.rs) - ECS version works correctly with async pattern
- [`packages/core/src/runtime/deno/notifications/mod.rs`](../../packages/core/src/runtime/deno/notifications/mod.rs) - Public interface unchanged
- [`packages/core/Cargo.toml`](../../packages/core/Cargo.toml) - All dependencies already present:
  - `objc2 = "0.6.2"` (line 121)
  - `objc2-foundation = "0.3.1"` (line 122)
  - `objc2-user-notifications = "0.3.1"` (line 123)
  - `block2 = "0.6.1"` (line 125)

## DEFINITION OF DONE

- [ ] `objc2::runtime::Bool` added to imports at top of file
- [ ] `UNAuthorizationOptions` and `UNNotificationSettings` added to objc2_user_notifications imports
- [ ] `ensure_authorized()` method replaced with active authorization logic
- [ ] First call checks authorization status with `getNotificationSettingsWithCompletionHandler()`
- [ ] Settings callback uses `std::ptr::NonNull<UNNotificationSettings>` parameter
- [ ] If NotDetermined, requests authorization with `requestAuthorizationWithOptions_completionHandler()`
- [ ] Request callback uses `Bool` type and converts with `.as_bool()`
- [ ] Both completion handlers use RcBlock pattern and update `auth_status` field
- [ ] Authorization status cached in `auth_status` for fast subsequent calls
- [ ] Code compiles without warnings or errors on macOS
- [ ] First notification attempt triggers macOS permission dialog (if status is NotDetermined)
- [ ] After permission granted, notifications display in macOS Notification Center
- [ ] Subsequent notification calls are instant (no 3-second timeout)

## VERIFICATION APPROACH

Run the Deno runtime and attempt to show a notification:

```typescript
// Deno script to verify fix
const result = await Deno.notifications.show({
    title: "Test Notification",
    message: "Authorization test - first run"
});
console.log("Notification shown:", result);

// Wait a bit
await new Promise(resolve => setTimeout(resolve, 2000));

// Second notification should be instant (cached authorization)
const result2 = await Deno.notifications.show({
    title: "Test Notification",
    message: "Second notification - should be instant"
});
console.log("Second notification shown:", result2);
```

**Expected behavior:**
1. **First run**: macOS permission dialog appears ("Allow notifications?")
2. **User grants permission**: Dialog dismissed
3. **First notification**: Appears in Notification Center (no 3-second delay)
4. **Second notification**: Appears instantly (authorization cached, no dialog)
5. **Subsequent runs**: All notifications instant (authorization cached from first run)

**If permission denied:**
- `ensure_authorized()` returns false
- `show_notification()` returns `NotificationError::PermissionDenied`
- Error logged, no crash

**Before the fix:**
- Always times out after 3 seconds
- Always returns `NotificationError::PermissionDenied`
- Permission dialog never shows

## REFERENCES

### Apple Documentation
- [UNUserNotificationCenter](https://developer.apple.com/documentation/usernotifications/unusernotificationcenter)
- [Requesting Authorization for User Notifications](https://developer.apple.com/documentation/usernotifications/asking-permission-to-use-notifications)
- [UNAuthorizationOptions](https://developer.apple.com/documentation/usernotifications/unauthorizationoptions)

### objc2 Crate Documentation
- [objc2-user-notifications 0.3.1](https://docs.rs/objc2-user-notifications/0.3.1/)
- [UNAuthorizationOptions](https://docs.rs/objc2-user-notifications/0.3.1/objc2_user_notifications/struct.UNAuthorizationOptions.html)
- [UNUserNotificationCenter methods](https://docs.rs/objc2-user-notifications/0.3.1/objc2_user_notifications/struct.UNUserNotificationCenter.html)
- [block2 RcBlock](https://docs.rs/block2/0.6.1/block2/struct.RcBlock.html)
- [block2 StackBlock](https://docs.rs/block2/0.6.1/block2/struct.StackBlock.html)
- [objc2::runtime::Bool](https://docs.rs/objc2/0.6.2/objc2/runtime/struct.Bool.html)

### Project Code References
- [Working ECS implementation](../../packages/ecs-notifications/src/backends/macos.rs) - Lines 44-103 (async authorization check with StackBlock)
- [Existing RcBlock pattern](../../packages/core/src/runtime/deno/notifications/macos.rs) - Lines 183-195 (completion handler in show_notification)
- [RcBlock permission examples](../../packages/ecs-permissions/src/platforms/macos/) - AV, EventKit, Contacts permissions
- [AV permissions authorization](../../packages/ecs-permissions/src/platforms/macos/av_permissions.rs) - Lines 62-70 (RcBlock with Bool parameter)
- [Dependencies](../../packages/core/Cargo.toml) - Lines 119-126 (macOS objc2 crates)

## ARCHITECTURAL NOTES

### Why Two Implementations Exist

1. **Deno Runtime Version** ([`packages/core/src/runtime/deno/notifications/macos.rs`](../../packages/core/src/runtime/deno/notifications/macos.rs))
   - For Deno JavaScript runtime plugins
   - Uses **synchronous blocking** with parking_lot Mutex/Condvar
   - Uses **RcBlock** for reference-counted closures
   - Simpler, standalone (no Bevy dependency)
   - **On-demand authorization** - checks when needed, not at construction

2. **ECS Service Version** ([`packages/ecs-notifications/src/backends/macos.rs`](../../packages/ecs-notifications/src/backends/macos.rs))
   - For native Bevy ECS plugins
   - Uses **async/await** with Bevy's AsyncComputeTaskPool
   - Uses **StackBlock** for stack-allocated async closures
   - Integrates with Bevy event system
   - Follows ECS request/response pattern
   - **Only checks authorization**, doesn't request it

Both implementations use the same underlying objc2-user-notifications APIs but with different concurrency models appropriate to their runtime environments.

### The Fix Strategy

**Before (BROKEN):** `ensure_authorized()` passively waited for `auth_status` to be populated by something else (that never existed)

**After (FIXED):** `ensure_authorized()` **actively checks and requests** authorization itself:
1. Check if `auth_status` is cached → return immediately (fast path, zero API calls)
2. If None, call `getNotificationSettingsWithCompletionHandler()` to check current status
3. If NotDetermined, call `requestAuthorizationWithOptions_completionHandler()` to request permission
4. Cache result in `auth_status` for future calls (makes subsequent calls instant)
5. Use RcBlock + Mutex/Condvar pattern (already proven in `show_notification()` lines 183-195)

This makes `ensure_authorized()` **self-sufficient** - no constructor modifications, no background threads, just active authorization on first use with caching for performance.
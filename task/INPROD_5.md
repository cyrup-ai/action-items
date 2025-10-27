# Task: Fix "In a Real App" Comment Stub in Camera Permission Example

## OBJECTIVE
Replace the placeholder comment "In a real app, you would trigger a permission request here" in `examples/permissions/camera_permission.rs` with actual working code that demonstrates the proper permission request pattern.

## PRIORITY
P2 - HIGH - Example code quality issue that directly affects developer onboarding and understanding of the permission system.

## CONTEXT

### File Structure
```
examples/
├── clipboard/
│   ├── basic_usage.rs
│   └── sync_operations.rs
└── permissions/
    ├── all_permissions.rs          # Comprehensive permission status checker
    ├── basic_usage.rs              # Interactive keyboard-driven permission demo
    └── camera_permission.rs        # TARGET FILE - Simple camera permission example
```

### The Problem
The `camera_permission.rs` example is likely one of the first files developers look at when learning the permission system. Line 29 contains a stub comment instead of working code:

```rust
Ok(PermissionStatus::NotDetermined) => {
    info!("Camera permission not determined, requesting access...");
    // In a real app, you would trigger a permission request here  ← REMOVE THIS
}
```

This creates confusion about:
1. How to actually request permissions
2. Whether the API is complete
3. What the proper event-driven pattern looks like

## ARCHITECTURE

### Permission System Overview

The permission system uses Bevy's event-driven architecture. Here's how it works:

**File**: [`packages/ecs-permissions/src/plugin.rs`](../packages/ecs-permissions/src/plugin.rs)

```rust
// 1. Simple Event Definition (lines 21-24)
#[derive(Event)]
pub struct PermissionRequest {
    pub typ: PermissionType,
}

// 2. Event Processing System (lines 72-84)
fn initiate_permission_requests(
    mut events: EventReader<PermissionRequest>,
    mut res: ResMut<PermissionResource>,
) {
    for event in events.read() {
        if !res.pending_requests.contains_key(&event.typ) {
            let rx = res.manager.request_permission(event.typ);
            res.pending_requests.insert(event.typ, Arc::new(Mutex::new(rx)));
        }
    }
}

// 3. Result Polling System (lines 86-112)
fn poll_permission_results(
    mut res: ResMut<PermissionResource>,
    mut changes: EventWriter<PermissionChanged>,
    mut errors: EventWriter<PermissionRequestError>,
) {
    let mut completed = Vec::new();
    for (&typ, rx_arc) in res.pending_requests.iter() {
        if let Ok(rx) = rx_arc.try_lock()
            && let Ok(result) = rx.try_recv()
        {
            match result {
                Ok(status) => {
                    changes.send(PermissionChanged { typ, status });
                },
                Err(error) => {
                    errors.send(PermissionRequestError { typ, error });
                },
            }
            completed.push(typ);
        }
    }
    for typ in completed {
        res.pending_requests.remove(&typ);
    }
}
```

### Event Flow Pattern

```
┌─────────────────────┐
│ Your System         │
│ (camera_permission) │
└──────────┬──────────┘
           │ 1. Send PermissionRequest event
           ▼
┌─────────────────────┐
│ initiate_permission │
│ _requests system    │  
└──────────┬──────────┘
           │ 2. Call manager.request_permission()
           ▼
┌─────────────────────┐
│ Platform-Specific   │
│ Permission Handler  │  (macOS/Windows/Linux)
└──────────┬──────────┘
           │ 3. Ask OS for permission (async)
           ▼
┌─────────────────────┐
│ poll_permission     │
│ _results system     │
└──────────┬──────────┘
           │ 4. Send PermissionChanged event
           ▼
┌─────────────────────┐
│ Your Listener       │
│ (handle_permission  │
│  _changes)          │
└─────────────────────┘
```

### Working Example Pattern

**File**: [`examples/permissions/basic_usage.rs`](../examples/permissions/basic_usage.rs:33-52)

```rust
fn handle_permission_responses(
    input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<PermissionDemo>,
    permission_res: Res<action_items_ecs_permissions::PermissionResource>,
    mut permission_events: EventWriter<action_items_ecs_permissions::PermissionRequest>,
) {
    if input.just_pressed(KeyCode::KeyC) && !demo.camera_requested {
        demo.camera_requested = true;
        
        match permission_res.check_permission(PermissionType::Camera) {
            Ok(status) => {
                info!("Camera permission status: {}", status);
                
                if status == PermissionStatus::NotDetermined {
                    info!("Requesting camera permission...");
                    let (sender, _receiver) = oneshot::channel();
                    permission_events.send(action_items_ecs_permissions::PermissionRequest {
                        permission_type: PermissionType::Camera,
                        response_sender: sender,
                    });
                }
            }
            Err(e) => error!("Failed to check camera permission: {}", e),
        }
    }
}
```

## SOLUTION

### Current Code (camera_permission.rs:16-36)

```rust
fn request_camera_permission(
    permission_res: Res<action_items_ecs_permissions::PermissionResource>,
) {
    match permission_res.check_permission(PermissionType::Camera) {
        Ok(PermissionStatus::Authorized) => {
            info!("Camera access already granted!");
        }
        Ok(PermissionStatus::Denied) => {
            warn!("Camera access was denied. Please enable in System Preferences.");
        }
        Ok(PermissionStatus::NotDetermined) => {
            info!("Camera permission not determined, requesting access...");
            // In a real app, you would trigger a permission request here  ← FIX THIS
        }
        Ok(status) => {
            info!("Camera permission status: {}", status);
        }
        Err(e) => {
            error!("Failed to check camera permission: {}", e);
        }
    }
}
```

### Required Changes

**1. Update Imports (line 5)**

Add `PermissionRequest` to the import list:

```rust
use action_items_ecs_permissions::{
    PermissionType, 
    PermissionStatus, 
    PermissionPlugin,
    PermissionRequest,  // ← ADD THIS
};
```

**2. Add EventWriter Parameter (line 16-18)**

```rust
fn request_camera_permission(
    permission_res: Res<action_items_ecs_permissions::PermissionResource>,
    mut permission_requests: EventWriter<PermissionRequest>,  // ← ADD THIS
) {
```

**3. Replace Comment with Working Code (line 27-29)**

**OLD**:
```rust
        Ok(PermissionStatus::NotDetermined) => {
            info!("Camera permission not determined, requesting access...");
            // In a real app, you would trigger a permission request here
        }
```

**NEW**:
```rust
        Ok(PermissionStatus::NotDetermined) => {
            info!("Camera permission not determined, requesting access...");
            permission_requests.send(PermissionRequest { 
                typ: PermissionType::Camera 
            });
        }
```

### Complete Fixed Function

```rust
fn request_camera_permission(
    permission_res: Res<action_items_ecs_permissions::PermissionResource>,
    mut permission_requests: EventWriter<PermissionRequest>,
) {
    match permission_res.check_permission(PermissionType::Camera) {
        Ok(PermissionStatus::Authorized) => {
            info!("Camera access already granted!");
        }
        Ok(PermissionStatus::Denied) => {
            warn!("Camera access was denied. Please enable in System Preferences.");
        }
        Ok(PermissionStatus::NotDetermined) => {
            info!("Camera permission not determined, requesting access...");
            permission_requests.send(PermissionRequest { 
                typ: PermissionType::Camera 
            });
        }
        Ok(status) => {
            info!("Camera permission status: {}", status);
        }
        Err(e) => {
            error!("Failed to check camera permission: {}", e);
        }
    }
}
```

## IMPLEMENTATION STEPS

### Step 1: Edit camera_permission.rs

**File**: `/Volumes/samsung_t9/action-items/examples/permissions/camera_permission.rs`

Make these three changes:

1. **Line 5**: Add `PermissionRequest` to imports
2. **Line 17**: Add `mut permission_requests: EventWriter<PermissionRequest>` parameter  
3. **Line 28**: Replace comment with: `permission_requests.send(PermissionRequest { typ: PermissionType::Camera });`

### Step 2: Verify Compilation

```bash
cd /Volumes/samsung_t9/action-items
cargo build --example camera_permission
```

Expected: Clean compilation with no errors.

### Step 3: Test the Example

```bash
cargo run --example camera_permission
```

Expected behavior:
- On first run with "NotDetermined" status, it will request camera permission
- macOS will show system permission dialog
- After user grants/denies, the `handle_permission_changes` system will log the result
- The `PermissionChanged` event fires automatically when OS responds

## KEY TECHNICAL DETAILS

### Why EventWriter<T>.send() Not .write()

Bevy's event API uses `.send()` for EventWriter:

```rust
permission_requests.send(PermissionRequest { typ: PermissionType::Camera });
```

NOT `.write()` (that's for a different pattern).

### PermissionRequest Struct Fields

From [`plugin.rs:21-24`](../packages/ecs-permissions/src/plugin.rs:21-24):

```rust
#[derive(Event)]
pub struct PermissionRequest {
    pub typ: PermissionType,  // ← Field name is "typ" not "permission_type"
}
```

### Available Permission Types

From [`types.rs`](../packages/ecs-permissions/src/types.rs):
- `Camera` - Camera hardware access
- `Microphone` - Microphone hardware access  
- `ScreenCapture` - Screen recording permission
- `Accessibility` - macOS accessibility API access
- `InputMonitoring` - Keyboard/mouse monitoring
- `FullDiskAccess` - Full disk access (macOS)
- `Bluetooth` - Bluetooth hardware access
- `Location` - GPS/location services
- `Contacts`, `Calendar`, `Reminders`, `Photos` - Data access permissions
- And more...

### Permission Status States

From [`types.rs`](../packages/ecs-permissions/src/types.rs):
- `NotDetermined` - User hasn't been asked yet
- `Authorized` - Permission granted
- `Denied` - Permission denied
- `Restricted` - Restricted by system policy
- `Limited` - Limited/partial access granted

## ADVANCED API (For Reference Only - Not Used in This Task)

The codebase also has an advanced permission API with wizard integration:

**File**: [`packages/ecs-permissions/src/events.rs:15-25`](../packages/ecs-permissions/src/events.rs)

```rust
fn request_camera_with_wizard(
    mut events: EventWriter<PermissionSetRequest>
) {
    events.send(
        PermissionSetRequest::new("camera_service")
            .with_required(PermissionType::Camera)
            .with_reason("Camera access is needed for profile pictures")
            .with_priority(RequestPriority::High)
    );
}
```

This is demonstrated in other examples (like `basic_usage.rs` and `all_permissions.rs`) but is NOT needed for the simple `camera_permission.rs` example.

## DEFINITION OF DONE

- [ ] Line 5: `PermissionRequest` added to imports
- [ ] Line 17: `EventWriter<PermissionRequest>` parameter added to function
- [ ] Line 28: Stub comment replaced with actual permission request code
- [ ] Example compiles without errors (`cargo build --example camera_permission`)
- [ ] Example runs and requests camera permission (`cargo run --example camera_permission`)
- [ ] When permission status is `NotDetermined`, the system now sends a `PermissionRequest` event
- [ ] The event is processed by `initiate_permission_requests` system
- [ ] The OS permission dialog appears (on macOS/Windows/Linux)
- [ ] `handle_permission_changes` system logs the result when OS responds

## CONSTRAINTS

- DO NOT change the example's simple structure - keep it beginner-friendly
- DO NOT add wizard integration to this example (that's in other examples)
- DO NOT modify other files in the codebase
- DO change ONLY the three specific lines indicated above
- DO keep the example focused on the simple permission request pattern

## WHY THIS MATTERS

The `camera_permission.rs` file serves as the "Hello World" for the permission system. Having placeholder comments instead of working code:

1. **Breaks trust** - Developers question if the API actually works
2. **Wastes time** - Forces developers to search other files for the answer
3. **Creates confusion** - Unclear which API to use (simple vs advanced)
4. **Hurts onboarding** - First impression is that code is incomplete

Fixing this makes the codebase more professional and the permission system easier to learn.

## CODE REFERENCES

### Files Modified
- [`examples/permissions/camera_permission.rs`](../examples/permissions/camera_permission.rs) - Lines 5, 17, 28

### Files Referenced
- [`packages/ecs-permissions/src/plugin.rs`](../packages/ecs-permissions/src/plugin.rs) - Event definitions and processing systems
- [`packages/ecs-permissions/src/events.rs`](../packages/ecs-permissions/src/events.rs) - Advanced API with PermissionSetRequest
- [`packages/ecs-permissions/src/types.rs`](../packages/ecs-permissions/src/types.rs) - PermissionType and PermissionStatus enums
- [`examples/permissions/basic_usage.rs`](../examples/permissions/basic_usage.rs) - Working example of permission requests
- [`examples/permissions/all_permissions.rs`](../examples/permissions/all_permissions.rs) - Comprehensive permission status checker

### Platform-Specific Implementations
- [`packages/ecs-permissions/src/platforms/macos/`](../packages/ecs-permissions/src/platforms/macos/) - macOS permission handlers
- [`packages/ecs-permissions/src/platforms/windows/`](../packages/ecs-permissions/src/platforms/windows/) - Windows permission handlers  
- [`packages/ecs-permissions/src/platforms/linux/`](../packages/ecs-permissions/src/platforms/linux/) - Linux permission handlers via D-Bus/Portals

## NOTES

### What Changed During Research

The original task description incorrectly identified `packages/ecs-permissions/tests/integration_tests.rs:180` as the target. Research revealed:

1. That line contains legitimate test documentation: `"// Note: In a real test environment..."` which is appropriate
2. The ACTUAL target is `examples/permissions/camera_permission.rs:29`
3. The task file has been corrected to focus on the right file

### Other "In a Real" Comments Found

Additional stub comments exist in the codebase but are lower priority:
- `packages/ecs-search-aggregator/src/plugin.rs:432` - Search cleanup stub
- `packages/core/src/plugins/bridge/handlers/processor.rs` - WASM function call stubs
- `packages/ecs-permissions/src/wizard/ui/theme.rs` - macOS theme detection stubs

These can be addressed in separate tasks but don't block developer onboarding like the camera_permission.rs stub does.

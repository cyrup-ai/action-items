# macOS Framework Integration Patterns

## Executive Summary

This document provides a systematic approach to integrating macOS framework APIs with Rust using objc2, based on analysis of working reference implementations. The key insight is that macOS frameworks use two distinct callback architectures that require different implementation patterns:

- **Block Pattern**: For APIs that accept closure/block parameters (e.g., `completionHandler:`)
- **Delegate Pattern**: For APIs that require setting delegate objects (e.g., `setDelegate:`)

## Decision Framework

### Step 1: API Analysis
Examine the framework method signature to determine callback architecture:

```rust
// Block-based API - accepts closure parameter
AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &handler)
                                                             ^^^^^^^^^^^^

// Delegate-based API - requires delegate object
manager.setDelegate(Some(ProtocolObject::from_ref(&*delegate)))
        ^^^^^^^^^^^
```

### Step 2: Pattern Selection
- If method accepts block/closure parameter → **Use Block Pattern**
- If method requires setting delegate object → **Use Delegate Pattern**

### Step 3: Implementation
Apply the appropriate pattern with verified templates (see sections below).

## Block Pattern Implementation

### When to Use
- APIs with `completionHandler:` parameters
- One-time callback scenarios
- Examples: AVFoundation, EventKit, Contacts

### Template
```rust
use std::sync::mpsc::Sender;
use block2::RcBlock;
use objc2::runtime::Bool;

pub fn request_permission(
    tx: Sender<Result<PermissionStatus, PermissionError>>,
) {
    let handler = RcBlock::new(move |granted: Bool| {
        let status = if granted.as_bool() {
            PermissionStatus::Authorized
        } else {
            PermissionStatus::Denied
        };
        let _ = tx.send(Ok(status));
    });

    unsafe {
        FrameworkAPI::requestAccess_completionHandler(param, &handler);
    }
}
```

### Key Characteristics
- ✅ Uses `RcBlock::new(move |params| { ... })`
- ✅ Moves `Sender<T>` into closure (never clone)
- ✅ Direct channel communication
- ✅ Closure captures by move, not borrow

## Delegate Pattern Implementation

### When to Use
- APIs requiring `setDelegate:` calls
- Persistent callback scenarios
- Multiple potential callbacks
- Examples: CoreLocation, CoreBluetooth

### Template
```rust
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, OnceLock};
use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol, ProtocolObject};
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send};

static FRAMEWORK_TX: OnceLock<Arc<Mutex<Sender<Result<PermissionStatus, PermissionError>>>>> =
    OnceLock::new();

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct FrameworkDelegate;

    unsafe impl NSObjectProtocol for FrameworkDelegate {}

    unsafe impl FrameworkDelegateProtocol for FrameworkDelegate {
        #[unsafe(method(frameworkDidUpdate:))]
        fn frameworkDidUpdate(&self, status: FrameworkStatus) {
            if let Some(tx_arc) = FRAMEWORK_TX.get() {
                if let Ok(tx) = tx_arc.lock() {
                    let _ = tx.send(Ok(status.into()));
                }
            }
        }
    }
);

pub fn request_permission(tx: Sender<Result<PermissionStatus, PermissionError>>) {
    if let Some(mtm) = MainThreadMarker::new() {
        let _ = FRAMEWORK_TX.set(Arc::new(Mutex::new(tx)));
        let delegate = FrameworkDelegate::alloc(mtm);
        let delegate: Retained<FrameworkDelegate> = unsafe { msg_send![delegate, init] };
        let manager = unsafe { FrameworkManager::new() };
        unsafe {
            manager.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        }
    }
}
```

### Key Characteristics
- ✅ Uses global `OnceLock<Arc<Mutex<Sender<...>>>>`
- ✅ Delegate class defined with `define_class!` macro
- ✅ Methods marked with `#[unsafe(method(...))]`
- ✅ `MainThreadMarker` for delegate allocation
- ✅ `ProtocolObject::from_ref()` for delegate setting

## Framework Categorization

### Block-Based Frameworks
| Framework | API Example | Pattern |
|-----------|-------------|---------|
| AVFoundation | `requestAccessForMediaType_completionHandler` | Block |
| EventKit | `requestAccessToEntityType_completion` | Block |
| Contacts | `requestAccessForEntityType_completionHandler` | Block |

### Delegate-Based Frameworks  
| Framework | API Example | Pattern |
|-----------|-------------|---------|
| CoreLocation | `setDelegate:` + `authorizationStatus` | Delegate |
| CoreBluetooth | `setDelegate:` + `centralManagerDidUpdateState` | Delegate |
| HealthKit | `setDelegate:` + health callbacks | Delegate |

## Technical Validation Checklist

### Block Pattern Validation
- [ ] `Sender<T>` moved into closure (not cloned)
- [ ] `RcBlock::new()` used (not StackBlock)
- [ ] Captured variables moved, not borrowed
- [ ] Closure signature matches framework expectation
- [ ] No `.copy()` calls on blocks

### Delegate Pattern Validation  
- [ ] Global static `OnceLock<Arc<Mutex<Sender<...>>>>` used
- [ ] Delegate class defined with `define_class!` macro
- [ ] Delegate methods marked `#[unsafe(method(...))]`
- [ ] `MainThreadMarker` used for delegate allocation
- [ ] `ProtocolObject::from_ref()` used for delegate setting
- [ ] No attempt to access non-existent ivars

## Troubleshooting Guide

### Common Errors

#### "Sender<T> doesn't implement Clone"
**Cause**: Attempting to clone channel sender
**Solution**: Use move semantics in blocks, shared ownership in delegates

#### "StackBlock has no method copy()"  
**Cause**: Calling `.copy()` on StackBlock
**Solution**: Use `RcBlock::new()` directly

#### "Cannot access ivars"
**Cause**: Trying to access ivars that don't exist in delegate
**Solution**: Use global static for state sharing

#### "Method not found"
**Cause**: Incorrect method signature in delegate protocol
**Solution**: Verify against framework documentation

### Debug Strategy
1. Verify API signature against documentation
2. Confirm pattern selection (block vs delegate)
3. Check technical validation checklist
4. Compare against reference implementations
5. Test compilation before assuming correctness

## Reference Implementations

Working examples can be found in:
- `packages/ecs-permissions/src/platforms/macos/av_permissions.rs` (Block Pattern)
- `packages/ecs-permissions/src/platforms/macos/location_permissions.rs` (Delegate Pattern)  
- `packages/ecs-permissions/src/platforms/macos/bluetooth_permissions.rs` (Delegate Pattern)

These implementations have been verified to compile and work correctly with objc2 0.6 and block2 0.6.
# PRODFIX_11: Optional - Implement Custom High-Performance Hotkey Systems for Windows/Linux

## ⚠️ CRITICAL CLARIFICATION - READ FIRST

**THIS IS NOT A BUG OR MISSING FEATURE. ALL PLATFORMS HAVE WORKING HOTKEY FUNCTIONALITY.**

The original task description is MISLEADING. After deep code analysis, here's the truth:

### What Actually Works Today (All Platforms):
1. ✅ **Global Hotkey Detection** (app minimized/not focused)
   - Windows/Linux: Uses `global-hotkey` crate (RegisterHotKey/XGrabKey APIs)
   - macOS: Custom CGEventTap implementation (617 lines, high-performance)
2. ✅ **Hotkey Capture UI** ("Press your hotkey..." recording)
   - ALL platforms: Uses Bevy's `KeyboardInput` events (cross-platform)
   - Implementation: [`packages/app/src/events/handlers/key_capture.rs`](../packages/app/src/events/handlers/key_capture.rs)

### What This Task Actually Is:
An **optional optimization** to implement Windows/Linux equivalents to the macOS CGEventTap system for:
- Performance consistency across platforms
- Architectural uniformity (all platforms use custom implementations)
- Potential performance improvements over `global-hotkey` crate

**RECOMMENDATION:** Reclassify from P2-MEDIUM to **P3-LOW** or mark as **WONTFIX** since current implementation works well.

---

## PRIORITY
**P3 - LOW (Optional Optimization)** *(Originally P2-MEDIUM due to misunderstanding)*

## LOCATION
- Primary: `packages/ecs-hotkey/src/platform/windows.rs` (already exists, uses global-hotkey)
- Primary: `packages/ecs-hotkey/src/platform/linux.rs` (already exists, uses global-hotkey)
- Reference: `packages/ecs-hotkey/src/platform/macos.rs` (617-line CGEventTap implementation)

## ARCHITECTURE OVERVIEW

The hotkey system has TWO complementary subsystems:

### System 1: Global Hotkey Detection (App Not Focused)
Detects when user presses a registered hotkey while app is minimized/unfocused.

**Current Implementation:**
```
macOS:      Custom CGEventTap (617 lines) → HotkeyPressed event
Windows:    global-hotkey crate (RegisterHotKey Win32 API) → HotkeyPressed event  
Linux:      global-hotkey crate (XGrabKey X11 API) → HotkeyPressed event
```

**Code References:**
- macOS: [`packages/ecs-hotkey/src/platform/macos.rs:452-471`](../packages/ecs-hotkey/src/platform/macos.rs#L452-L471) (`process_macos_hotkey_events_system`)
- Win/Linux: [`packages/ecs-hotkey/src/systems.rs:135`](../packages/ecs-hotkey/src/systems.rs#L135) (`process_hotkey_registration_requests_system`)

### System 2: Hotkey Capture (Recording New Hotkeys)
When user clicks "Record hotkey..." in preferences, captures the key combination they press.

**Current Implementation:**
```
ALL PLATFORMS: Bevy KeyboardInput → PreferencesEvent::KeyCaptured
```

**Code Reference:**
- [`packages/app/src/events/handlers/key_capture.rs:26-95`](../packages/app/src/events/handlers/key_capture.rs#L26-L95) (`real_hotkey_capture_system`)

**This works perfectly on all platforms.** The Bevy input system abstracts platform differences.

---

## CURRENT STATE DETAILED ANALYSIS

### macOS Implementation (Custom, High-Performance)

**File:** [`packages/ecs-hotkey/src/platform/macos.rs`](../packages/ecs-hotkey/src/platform/macos.rs) (617 lines)

**Architecture:**
```rust
// Lock-free, zero-allocation design
static HOTKEY_REGISTRY: Lazy<DashMap<(u32, u32), u64>> = ...;
static EVENT_RING: LockFreeEventRing = ...;

// CGEventTap callback (runs on EVERY keystroke system-wide)
unsafe extern "C" fn hotkey_event_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    cg_event: NonNull<CGEvent>,
    _user_info: *mut c_void,
) -> *mut CGEvent {
    // O(1) hash lookup in DashMap
    // Push to lock-free ring buffer
    // Zero allocations for performance
}

// Bevy system polls events from ring buffer
pub fn process_macos_hotkey_events_system(
    mut hotkey_pressed_events: EventWriter<HotkeyPressed>,
    hotkey_registry: Res<crate::resources::HotkeyRegistry>,
) {
    while let Some(action_hash) = pop_hotkey_event() {
        // Convert to HotkeyPressed event
    }
}
```

**Key Features:**
- Lock-free atomics and ring buffers
- Zero-allocation event processing
- Direct system-wide keyboard monitoring via CGEventTap
- Requires Accessibility permissions

**Why Custom Implementation?**
- Performance: Eliminates global-hotkey crate overhead
- Control: Direct access to macOS APIs
- Features: Can intercept ALL keystrokes (not just registered ones)

### Windows Implementation (global-hotkey Crate)

**File:** [`packages/ecs-hotkey/src/platform/windows.rs`](../packages/ecs-hotkey/src/platform/windows.rs) (81 lines)

**Architecture:**
```rust
use global_hotkey::{GlobalHotKeyManager, hotkey::HotKey};

// Registration handled by global-hotkey crate
// Uses Win32 RegisterHotKey() API internally
pub fn check_windows_permissions() -> Result<(), String> {
    // Validates Win32 event loop is running
}
```

**How It Works:**
1. `global-hotkey` creates hidden window to receive `WM_HOTKEY` messages
2. Calls `RegisterHotKey()` for each hotkey combination
3. Win32 message loop dispatches to cross-platform event channel
4. Bevy system polls `GlobalHotKeyEvent::receiver()`

**Reference:** [`tmp/global-hotkey/src/platform_impl/windows/`](../tmp/global-hotkey/src/platform_impl/windows/)

### Linux Implementation (global-hotkey Crate)

**File:** [`packages/ecs-hotkey/src/platform/linux.rs`](../packages/ecs-hotkey/src/platform/linux.rs) (214 lines)

**Architecture:**
```rust
use global_hotkey::{GlobalHotKeyManager, hotkey::HotKey};

// X11 implementation via x11rb
pub fn is_wayland() -> bool { /* Check WAYLAND_DISPLAY */ }
pub fn detect_compositor() -> LinuxCompositor { /* KDE/GNOME/Hyprland */ }

// Wayland support for KDE/Hyprland via DBus
#[cfg(target_os = "linux")]
pub mod linux_wayland;  // Separate implementation
```

**How It Works:**
1. `global-hotkey` connects to X11 display server
2. Calls `XGrabKey()` on root window for each hotkey
3. Spawns thread to poll X11 events
4. Processes `KeyPress` events → cross-platform event channel
5. Bevy system polls `GlobalHotKeyEvent::receiver()`

**Wayland Support:**
- KDE Plasma: DBus global shortcuts API ([`packages/ecs-hotkey/src/platform/linux_wayland_kde.rs`](../packages/ecs-hotkey/src/platform/linux_wayland_kde.rs))
- Hyprland: XDG Desktop Portal ([`packages/ecs-hotkey/src/platform/linux_wayland_portal.rs`](../packages/ecs-hotkey/src/platform/linux_wayland_portal.rs))
- Others: Fallback to XWayland

**Reference:** [`tmp/global-hotkey/src/platform_impl/x11/`](../tmp/global-hotkey/src/platform_impl/x11/)

---

## WHY CONSIDER CUSTOM IMPLEMENTATIONS?

### Potential Benefits:
1. **Performance**: Direct API access, zero-allocation designs
2. **Consistency**: All platforms use similar architecture
3. **Features**: Ability to intercept unregistered keystrokes (future)
4. **Control**: No dependency on third-party crate maintenance

### Downsides:
1. **Complexity**: 600+ lines per platform vs 80 lines with crate
2. **Maintenance**: Must track OS API changes (Win32, X11, Wayland)
3. **Testing**: Platform-specific bugs harder to reproduce
4. **Permissions**: May require elevated privileges (Linux especially)

### Current global-hotkey Limitations:
- Linux: X11 only (no native Wayland, requires XWayland)
  - **Note:** Action Items already has custom Wayland support via DBus
- Windows: Some hotkeys reserved by OS (Win+L, Win+D)
- macOS: Uses Carbon APIs (deprecated but still functional)

**Verdict:** Custom implementations provide marginal benefits but significant maintenance burden.

---

## IF YOU CHOOSE TO IMPLEMENT THIS

### Windows: Low-Level Keyboard Hook

**Required Changes:**

**1. Create Windows Hook System**

File: `packages/ecs-hotkey/src/platform/windows_hooks.rs` (new)

```rust
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx,
    WH_KEYBOARD_LL, KBDLLHOOKSTRUCT, LLKHF_UP, HHOOK,
    WPARAM, LPARAM, LRESULT,
};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use dashmap::DashMap;
use once_cell::sync::Lazy;

// Similar to macOS design
static HOTKEY_REGISTRY: Lazy<DashMap<(u32, u32), u64>> = Lazy::new(DashMap::new);
static EVENT_RING: LockFreeEventRing = LockFreeEventRing::new();
static HOOK_HANDLE: AtomicU64 = AtomicU64::new(0);

// Low-level keyboard hook callback
unsafe extern "system" fn keyboard_hook_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
        let vk_code = kbd_struct.vkCode;
        let flags = kbd_struct.flags;
        
        // Check if key down (not key up)
        if (flags & LLKHF_UP) == 0 {
            // Extract modifiers from GetAsyncKeyState
            let modifiers = get_current_modifiers();
            
            // O(1) lookup in registry
            if let Some(action_hash) = HOTKEY_REGISTRY.get(&(vk_code, modifiers)) {
                EVENT_RING.try_push(*action_hash);
            }
        }
    }
    
    CallNextHookEx(None, n_code, w_param, l_param)
}

pub fn install_keyboard_hook() -> Result<(), String> {
    unsafe {
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_hook_proc),
            None,
            0
        )?;
        
        HOOK_HANDLE.store(hook.0 as u64, Ordering::Release);
    }
    Ok(())
}

pub fn uninstall_keyboard_hook() {
    let handle = HOOK_HANDLE.swap(0, Ordering::Acquire);
    if handle != 0 {
        unsafe {
            UnhookWindowsHookEx(HHOOK(handle as isize));
        }
    }
}

// Bevy system to process events (similar to macOS)
pub fn process_windows_hotkey_events_system(
    mut hotkey_pressed_events: EventWriter<HotkeyPressed>,
    hotkey_registry: Res<crate::resources::HotkeyRegistry>,
) {
    while let Some(action_hash) = EVENT_RING.pop() {
        // Find binding by action hash
        for (hotkey_id, binding) in &hotkey_registry.registered_hotkeys {
            if hash_action(&binding.action) == action_hash {
                hotkey_pressed_events.write(HotkeyPressed {
                    hotkey_id: hotkey_id.clone(),
                    binding: binding.clone(),
                });
                break;
            }
        }
    }
}
```

**2. Add Windows-Specific Dependencies**

File: `packages/ecs-hotkey/Cargo.toml`

```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_Foundation"
] }
dashmap = "6.1"
once_cell = "1.20"
```

**3. Integrate into Plugin**

File: `packages/ecs-hotkey/src/lib.rs` (modify)

```rust
#[cfg(target_os = "windows")]
app.add_systems(Startup, crate::platform::windows_hooks::setup_windows_hook_system);

#[cfg(target_os = "windows")]
app.add_systems(Update, 
    (crate::platform::windows_hooks::process_windows_hotkey_events_system,)
    .in_set(HotkeySystemSet::Detection)
);
```

**References:**
- Win32 Hooks: https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks
- KBDLLHOOKSTRUCT: https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct
- SetWindowsHookExW: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw
- Existing Windows code: [`packages/ecs-hotkey/src/platform/windows.rs`](../packages/ecs-hotkey/src/platform/windows.rs)

### Linux: XRecord Extension

**Required Changes:**

**1. Create X11 XRecord System**

File: `packages/ecs-hotkey/src/platform/linux_xrecord.rs` (new)

```rust
use x11rb::connection::Connection;
use x11rb::protocol::record::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{self, KeyPressEvent};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use dashmap::DashMap;
use once_cell::sync::Lazy;

// Similar to macOS/Windows design
static HOTKEY_REGISTRY: Lazy<DashMap<(u8, u16), u64>> = Lazy::new(DashMap::new);
static EVENT_RING: LockFreeEventRing = LockFreeEventRing::new();
static RECORDING: AtomicBool = AtomicBool::new(false);

pub struct X11RecordCapture {
    connection: x11rb::rust_connection::RustConnection,
    context: record::Context,
}

impl X11RecordCapture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, screen_num) = x11rb::connect(None)?;
        
        // Create recording context for all keyboard events
        let context = conn.generate_id()?;
        let client_spec = record::CS::ALL_CLIENTS;
        let ranges = record::Range {
            device_events: Some(record::Range8 {
                first: xproto::KEY_PRESS_EVENT,
                last: xproto::KEY_RELEASE_EVENT,
            }),
            ..Default::default()
        };
        
        conn.record_create_context(context, 0, &[client_spec], &[ranges])?;
        
        Ok(Self { connection: conn, context })
    }
    
    pub fn start_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        RECORDING.store(true, Ordering::Release);
        
        // Start recording in separate thread
        std::thread::spawn(move || {
            while RECORDING.load(Ordering::Acquire) {
                // Process X11 events
                // Extract keycode and modifiers
                // Match against HOTKEY_REGISTRY
                // Push to EVENT_RING
            }
        });
        
        Ok(())
    }
}

// Bevy system to process events
pub fn process_linux_xrecord_events_system(
    mut hotkey_pressed_events: EventWriter<HotkeyPressed>,
    hotkey_registry: Res<crate::resources::HotkeyRegistry>,
) {
    while let Some(action_hash) = EVENT_RING.pop() {
        // Same as macOS/Windows
    }
}
```

**2. Add X11 Dependencies**

File: `packages/ecs-hotkey/Cargo.toml`

```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11rb = { version = "0.13", features = ["record", "xtest"] }
dashmap = "6.1"
once_cell = "1.20"
```

**3. Handle Wayland**

Wayland has NO global keyboard interception API. Options:
1. Use existing DBus integrations (KDE/Hyprland) - **already implemented**
2. Require XWayland fallback
3. Compositor-specific protocols (limited support)

**Current Wayland support is ALREADY IMPLEMENTED:**
- [`packages/ecs-hotkey/src/platform/linux_wayland_kde.rs`](../packages/ecs-hotkey/src/platform/linux_wayland_kde.rs)
- [`packages/ecs-hotkey/src/platform/linux_wayland_portal.rs`](../packages/ecs-hotkey/src/platform/linux_wayland_portal.rs)

**References:**
- X11 XRecord: https://www.x.org/releases/X11R7.7/doc/libXtst/recordlib.html
- x11rb docs: https://docs.rs/x11rb/latest/x11rb/
- Existing Linux code: [`packages/ecs-hotkey/src/platform/linux.rs`](../packages/ecs-hotkey/src/platform/linux.rs)

---

## CODE EXAMPLES AND PATTERNS

### Lock-Free Ring Buffer (from macOS implementation)

```rust
// From packages/ecs-hotkey/src/platform/macos.rs:54-88

#[repr(align(64))] // Cache line aligned
struct LockFreeEventRing {
    events: [AtomicU64; EVENT_RING_SIZE],
    write_idx: AtomicUsize,
    read_idx: AtomicUsize,
}

impl LockFreeEventRing {
    const fn new() -> Self {
        Self {
            events: [const { AtomicU64::new(0) }; EVENT_RING_SIZE],
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn try_push(&self, action_hash: u64) -> bool {
        if action_hash == 0 { return false; }
        
        let write_idx = self.write_idx.load(Ordering::Relaxed);
        let read_idx = self.read_idx.load(Ordering::Acquire);
        
        if (write_idx + 1) % EVENT_RING_SIZE == read_idx {
            return false; // Ring buffer full
        }
        
        if self.events[write_idx].compare_exchange_weak(
            0, action_hash, Ordering::Release, Ordering::Relaxed
        ).is_ok() {
            self.write_idx.store((write_idx + 1) % EVENT_RING_SIZE, Ordering::Release);
            true
        } else {
            false
        }
    }
}
```

### DashMap Registry Pattern

```rust
// From packages/ecs-hotkey/src/platform/macos.rs:23-24

use dashmap::DashMap;
use once_cell::sync::Lazy;

static HOTKEY_REGISTRY: Lazy<DashMap<(u32, u32), u64>> = Lazy::new(DashMap::new);

// O(1) concurrent hash map operations
pub fn register_hotkey_atomic(key_code: u32, modifiers: u32, action: &str) -> Result<u64, Error> {
    let action_hash = hash_action(action);
    let key = (key_code, modifiers);
    HOTKEY_REGISTRY.insert(key, action_hash);
    Ok(action_hash)
}
```

### Action Hash Function

```rust
// From packages/ecs-hotkey/src/platform/macos.rs:314-321

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[inline]
fn hash_action(action: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    action.hash(&mut hasher);
    hasher.finish()
}
```

---

## ALTERNATIVE APPROACH: AUGMENT global-hotkey

Instead of reimplementing everything, consider contributing improvements to the `global-hotkey` crate:

**Fork and Enhance:**
1. Add performance optimizations (lock-free designs)
2. Improve Wayland support
3. Add more granular error handling
4. Submit upstream PRs

**Repository:** https://github.com/tauri-apps/global-hotkey
**Local Clone:** [`tmp/global-hotkey/`](../tmp/global-hotkey/)

This approach provides benefits to the entire Rust ecosystem while reducing maintenance burden.

---

## DEPENDENCIES REQUIRED

### Windows Custom Implementation:
```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_Foundation"
] }
dashmap = "6.1"
once_cell = "1.20"
```

### Linux Custom Implementation:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11rb = { version = "0.13", features = ["record", "xtest"] }
dashmap = "6.1"
once_cell = "1.20"
```

---

## CONSTRAINTS

- **NO changes to capture UI** - Bevy KeyboardInput works perfectly
- **NO changes to Wayland support** - Already implemented via DBus
- **NO removal of global-hotkey** - Keep as fallback option
- **Must match macOS performance** - Zero-allocation, lock-free designs
- **Must compile on all platforms** - Use feature flags appropriately

---

## DEFINITION OF DONE

**IF you decide to implement this (not recommended):**

- [ ] Windows WH_KEYBOARD_LL hook implementation complete
- [ ] Linux X11 XRecord implementation complete  
- [ ] Lock-free ring buffers implemented for both platforms
- [ ] DashMap-based hotkey registries for both platforms
- [ ] Bevy systems for polling events from ring buffers
- [ ] Feature flags to toggle between custom/global-hotkey implementations
- [ ] Platform-specific startup systems registered in plugin
- [ ] Code compiles without warnings on Windows, Linux, macOS
- [ ] Hotkeys work in manual testing on all platforms
- [ ] Performance benchmarks show improvement over global-hotkey (if not, revert)

**Definition of Success:**
- All platforms use consistent architecture
- Performance matches or exceeds macOS implementation
- No regressions in functionality
- Maintenance burden is acceptable

---

## RECOMMENDATION: DO NOT IMPLEMENT

**Reasons to Skip This Task:**

1. ✅ **Current implementation works perfectly** - Zero user-facing issues
2. ✅ **global-hotkey is well-maintained** - Active development, cross-platform
3. ❌ **High maintenance burden** - 600+ lines per platform to maintain
4. ❌ **Marginal performance gains** - Hotkey registration is not a bottleneck
5. ❌ **Platform-specific bugs** - More surface area for OS-specific issues
6. ❌ **Permissions complexity** - Especially on Linux (may require root)

**Better Use of Time:**
- Focus on application features users actually request
- Improve error messages and user feedback
- Enhance Wayland support (already in progress)
- Contribute to `global-hotkey` crate upstream

**If You Must Implement:**
- Start with Windows (simpler API than X11)
- Add feature flag to toggle custom vs global-hotkey
- Benchmark thoroughly to justify the complexity
- Prepare for increased bug reports and maintenance

---

## RESEARCH NOTES AND REFERENCES

### Documentation:
- **macOS CGEventTap**: https://developer.apple.com/documentation/coregraphics/1454426-cgeventtapcreate
- **Windows Hooks**: https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks
- **X11 XRecord**: https://www.x.org/releases/X11R7.7/doc/libXtst/recordlib.html
- **Wayland Global Shortcuts**: No standard protocol (compositor-specific)

### Code References:
- **Current macOS Implementation**: [`packages/ecs-hotkey/src/platform/macos.rs`](../packages/ecs-hotkey/src/platform/macos.rs) (617 lines)
- **Current Windows Implementation**: [`packages/ecs-hotkey/src/platform/windows.rs`](../packages/ecs-hotkey/src/platform/windows.rs) (81 lines)
- **Current Linux Implementation**: [`packages/ecs-hotkey/src/platform/linux.rs`](../packages/ecs-hotkey/src/platform/linux.rs) (214 lines)
- **Capture UI (All Platforms)**: [`packages/app/src/events/handlers/key_capture.rs`](../packages/app/src/events/handlers/key_capture.rs) (98 lines)
- **global-hotkey Source**: [`tmp/global-hotkey/`](../tmp/global-hotkey/)
- **Architecture Doc**: [`packages/ecs-hotkey/ARCHITECTURE.md`](../packages/ecs-hotkey/ARCHITECTURE.md)

### Key Files in global-hotkey:
- Windows: [`tmp/global-hotkey/src/platform_impl/windows/mod.rs`](../tmp/global-hotkey/src/platform_impl/windows/mod.rs)
- X11: [`tmp/global-hotkey/src/platform_impl/x11/mod.rs`](../tmp/global-hotkey/src/platform_impl/x11/mod.rs)
- macOS: [`tmp/global-hotkey/src/platform_impl/macos/mod.rs`](../tmp/global-hotkey/src/platform_impl/macos/mod.rs)

### Performance Considerations:
- **Lock-free design**: Critical for system-wide keyboard hooks (runs on EVERY keystroke)
- **Ring buffers**: Prevent blocking between callback and Bevy systems
- **DashMap**: Concurrent hash map for thread-safe hotkey registry
- **Zero allocation**: Hotkey detection must not allocate memory

### Platform Limitations:
- **Windows**: Some system hotkeys cannot be intercepted (Win+L, Win+Tab, etc.)
- **Linux X11**: Global grabs can be overridden by window manager
- **Linux Wayland**: No standard API (compositor-specific DBus protocols)
- **macOS**: Requires Accessibility permissions (user must approve)

---

## CONCLUSION

This task is **NOT RECOMMENDED** for implementation. The current system using `global-hotkey` crate for Windows/Linux works perfectly and is well-maintained. Implementing custom platform-specific systems would add 1000+ lines of complex, platform-specific code with minimal benefit.

**The TODO comments on lines 18-19 of `packages/ecs-hotkey/src/lib.rs` should be considered aspirational, not critical.**

If performance or architectural consistency become actual problems (with evidence from user feedback or profiling), reconsider this task. Until then, focus on features that provide clear user value.

**Task Status Recommendation:** Mark as **WONTFIX** or **FUTURE** and remove from active backlog.

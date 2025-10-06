# ecs-hotkey Architecture

## Design Philosophy

This package provides comprehensive ECS-based global hotkey management with two complementary systems:

### 1. Hotkey Registration & Firing (All Platforms)

**Purpose**: Register predefined hotkeys and fire events when pressed.

**Implementation**: 
- Uses `global-hotkey` crate (cross-platform Win32/X11/macOS)
- Systems: `process_hotkey_pressed_events_system`
- Handles: Windows (RegisterHotKey), Linux (XGrabKey), macOS (Carbon/fallback)

**Flow**:
```
App → HotkeyRegisterRequested → global_manager.register() → OS
User presses hotkey → OS → global_manager.poll() → HotkeyPressed event → App
```

**Files**:
- [systems.rs:process_hotkey_registration_requests_system](src/systems.rs#L135)
- [systems.rs:process_hotkey_pressed_events_system](src/systems.rs#L215)

### 2. Hotkey Capture (Platform-Specific)

**Purpose**: Allow users to record custom hotkey combinations ("Press your hotkey...").

**Why Needed**: `global-hotkey` only monitors *registered* hotkeys. To implement
"Press your desired hotkey..." UI (like Raycast/Alfred), we need to intercept
ALL keypresses temporarily during recording.

**Implementation**:
- **macOS**: CGEventTap (system-wide event interception) - [platform/macos.rs](src/platform/macos.rs)
  - 617 lines of lock-free atomic data structures
  - Ring buffer for event queueing
  - Accessibility permission required
- **Windows**: TODO - Need low-level keyboard hooks (WH_KEYBOARD_LL)
  - Reference: `SetWindowsHookExW` Win32 API
  - Similar to global-hotkey's internal implementation
- **Linux**: TODO - Need X11/Wayland event listeners
  - X11: `XRecordExtension` for passive grab
  - Wayland: compositor-specific (KDE/Hyprland via DBus)

**Flow**:
```
User clicks "Record Hotkey" → HotkeyCaptureRequested event
Capture system intercepts ALL keypresses (modifier tracking)
User presses desired combination → HotkeyCaptureCompleted event
App registers new hotkey via system #1
```

**Files**:
- [capture.rs](src/capture.rs)
- [systems.rs:real_hotkey_capture_system](src/systems.rs#L245)

### Platform Support Matrix

| Platform | Registration | Capture | Status |
|----------|-------------|---------|--------|
| macOS | global-hotkey | CGEventTap | ✅ Complete |
| Windows | global-hotkey | TODO | 🟡 Partial |
| Linux X11 | global-hotkey | TODO | 🟡 Partial |
| Linux Wayland (KDE) | global-hotkey + DBus | TODO | 🟡 Partial |

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     HotkeyPlugin                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────┐           ┌────────────────────┐     │
│  │ Registration    │           │ Capture System     │     │
│  │ (global-hotkey) │           │ (platform-specific)│     │
│  │ - All platforms │           │ - macOS: 617 lines │     │
│  │ - Registered    │           │ - Win/Linux: TODO  │     │
│  │   hotkeys only  │           │ - ALL keypresses   │     │
│  └────────┬────────┘           └─────────┬──────────┘     │
│           │                               │                 │
│           ▼                               ▼                 │
│  ┌─────────────────────────────────────────────────┐      │
│  │         Shared State Resources                   │      │
│  │  - HotkeyRegistry (registered combos)            │      │
│  │  - HotkeyPreferences (user config)               │      │
│  │  - HotkeyCaptureState (recording session)        │      │
│  └─────────────────────────────────────────────────┘      │
│                                                             │
│  Events: HotkeyRegisterRequested → HotkeyPressed          │
│          HotkeyCaptureRequested → HotkeyCaptureCompleted  │
└─────────────────────────────────────────────────────────────┘
```

## Future Work

1. Windows capture: Implement using `SetWindowsHookExW(WH_KEYBOARD_LL)`
2. Linux X11 capture: Implement using `XRecordExtension`
3. Linux Wayland: Compositor-specific DBus integration
4. Unified capture API across all platforms

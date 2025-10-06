# HOTKEY_4: Windows & Linux Platform Enhancement - Integration Required

## Core Objective

Integrate platform-specific helper functions that are already implemented but not yet connected to the hotkey registration and error handling flow.

**What Already Exists** (✅ COMPLETE):
- Permission checking functions: `check_windows_permissions()`, `check_linux_permissions()`
- Error formatting functions: `format_windows_error()`, `format_linux_error()`
- Info display functions: `display_windows_hotkey_info()`, `display_linux_hotkey_info()`
- Full Wayland support (653 lines across 3 files) with KDE and XDG Portal backends

**What Needs Integration** (🔧 REQUIRED):
- Wire up error formatting in systems.rs registration/unregistration error handlers
- Call permission checks during plugin initialization
- Display platform-specific info messages at startup

---

## Research: Existing Implementation

### Platform Files (Already Complete)

**Windows Platform**: [`packages/ecs-hotkey/src/platform/windows.rs`](../packages/ecs-hotkey/src/platform/windows.rs) (83 lines)
```rust
// Lines 41-55: Permission checking
pub fn check_windows_permissions() -> Result<(), String> {
    use std::env;
    if env::var("TERM").is_ok() && env::var("DISPLAY").is_err() {
        return Err("Windows hotkeys require a GUI event loop...".to_string());
    }
    Ok(())
}

// Lines 61-72: Error formatting with actionable guidance
pub fn format_windows_error(error: &str) -> String {
    if error.contains("AlreadyRegistered") {
        "Hotkey already registered by another application.\n\
         Common conflicts on Windows:\n\
         • Win+Shift+Space: Often used by Windows input methods\n\
         • Ctrl+Shift+Space: May conflict with office applications\n\
         Try a different hotkey combination in Settings.".to_string()
    } else {
        format!("Windows hotkey error: {}", error)
    }
}

// Lines 75-80: Startup info display  
pub fn display_windows_hotkey_info() {
    info!("🚀 Action Items Launcher is ready!");
    info!("📋 Press Ctrl+Shift+Space to activate the launcher from anywhere");
    info!("ℹ️  Windows: Hotkeys use RegisterHotKey Win32 API (no special permissions needed)");
}
```

**Linux Platform**: [`packages/ecs-hotkey/src/platform/linux.rs`](../packages/ecs-hotkey/src/platform/linux.rs) (214 lines)
```rust
// Lines 49-62: Environment detection
pub fn is_wayland() -> bool { /* ... */ }
pub fn is_x11_available() -> bool { /* ... */ }

// Lines 64-88: Compositor detection
pub enum LinuxCompositor { Kde, Gnome, Hyprland, Sway, Unknown }
pub fn detect_compositor() -> LinuxCompositor { /* ... */ }

// Lines 95-133: Intelligent permission checking
pub fn check_linux_permissions() -> Result<(), String> {
    if is_wayland() {
        let compositor = detect_compositor();
        match compositor {
            LinuxCompositor::Kde => {
                info!("✅ Wayland detected with KDE Plasma - native support available");
                Ok(())
            }
            LinuxCompositor::Hyprland => {
                info!("✅ Wayland detected with Hyprland - XDG Portal support available");
                Ok(())
            }
            // ... more compositor-specific handling
        }
    } else if is_x11_available() {
        info!("✅ X11 display server detected, hotkeys will work");
        Ok(())
    } else {
        Err("No X11 display server detected...".to_string())
    }
}

// Lines 137-201: Compositor-specific error formatting
pub fn format_linux_error(error: &str) -> String {
    if error.contains("Wayland") || error.contains("compositor") {
        let compositor = detect_compositor();
        match compositor {
            LinuxCompositor::Sway => {
                format!("Global hotkeys not supported on Sway compositor.\n\
                         Sway does not implement XDG Portal GlobalShortcuts.\n\
                         Workarounds:\n\
                         • Run under XWayland: GDK_BACKEND=x11 ./action_items\n\
                         • Switch to X11 session\n\
                         • Use Sway's built-in keybinding system...")
            }
            // ... more compositor-specific guidance
        }
    }
    // ... more error types
}
```

**Module Exports**: [`packages/ecs-hotkey/src/platform/mod.rs`](../packages/ecs-hotkey/src/platform/mod.rs)
```rust
// Line 54: All platform functions exported
pub use linux::*;
pub use windows::*;
```

**Library Exports**: [`packages/ecs-hotkey/src/lib.rs`](../packages/ecs-hotkey/src/lib.rs)
```rust
// Line 54: All platform types re-exported
pub use platform::*;
```

---

## Integration Points

### 1. Error Formatting in Registration System

**File**: [`packages/ecs-hotkey/src/systems.rs`](../packages/ecs-hotkey/src/systems.rs)

**Current Code** (lines 136-145):
```rust
Err(e) => {
    warn!(
        "Failed to register hotkey {}: {}",
        request.binding.definition.description, e
    );
    
    // ... conflict handling ...
    
    registration_completed.write(HotkeyRegisterCompleted {
        binding: request.binding.clone(),
        requester: request.binding.requester.clone(),
        success: false,
        error_message: Some(e.to_string()),  // ← Plain error, no formatting
    });
}
```

**Required Change**:
```rust
Err(e) => {
    // Format error with platform-specific actionable guidance
    let formatted_error = {
        #[cfg(target_os = "windows")]
        { crate::platform::format_windows_error(&e.to_string()) }
        
        #[cfg(target_os = "linux")]
        { crate::platform::format_linux_error(&e.to_string()) }
        
        #[cfg(target_os = "macos")]
        { crate::platform::format_macos_error(&e.to_string()) }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        { e.to_string() }
    };
    
    error!("Failed to register hotkey: {}", formatted_error);
    
    // ... existing conflict handling ...
    
    registration_completed.write(HotkeyRegisterCompleted {
        binding: request.binding.clone(),
        requester: request.binding.requester.clone(),
        success: false,
        error_message: Some(formatted_error),  // ← Now with actionable guidance
    });
}
```

**Also apply to unregistration errors** (around line 235):
```rust
Err(e) => {
    let formatted_error = {
        #[cfg(target_os = "windows")]
        { crate::platform::format_windows_error(&e.to_string()) }
        
        #[cfg(target_os = "linux")]
        { crate::platform::format_linux_error(&e.to_string()) }
        
        #[cfg(target_os = "macos")]
        { crate::platform::format_macos_error(&e.to_string()) }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        { e.to_string() }
    };
    
    error!("Failed to unregister hotkey: {}", formatted_error);
    
    unregistration_completed.write(HotkeyUnregisterCompleted {
        hotkey_id: request.hotkey_id.clone(),
        success: false,
    });
}
```

**Test error formatting** (around line 427):
```rust
Err(e) => {
    let formatted_error = {
        #[cfg(target_os = "windows")]
        { crate::platform::format_windows_error(&e.to_string()) }
        
        #[cfg(target_os = "linux")]
        { crate::platform::format_linux_error(&e.to_string()) }
        
        #[cfg(target_os = "macos")]
        { crate::platform::format_macos_error(&e.to_string()) }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        { e.to_string() }
    };
    
    info!("Hotkey test failed: {}", formatted_error);
    test_results.write(HotkeyTestResult {
        hotkey_definition: request.definition.clone(),
        requester: request.requester.clone(),
        success: false,
        error_message: Some(formatted_error),
        test_timestamp: std::time::Instant::now(),
    });
}
```

---

### 2. Permission Checks at Plugin Initialization

**File**: [`packages/ecs-hotkey/src/lib.rs`](../packages/ecs-hotkey/src/lib.rs)

**Current Code** (lines 140-205): Plugin initialization creates HotkeyManager but doesn't validate permissions

**Required Change**: Add permission validation in `Plugin::build()` after line 142 (before creating HotkeyManager):

```rust
impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing ECS Hotkey Service Plugin");

        // ========== ADD PERMISSION CHECKS HERE ==========
        // Validate platform permissions before initializing
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = crate::platform::check_windows_permissions() {
                error!("❌ Windows hotkey permissions check failed: {}", e);
                error!("Hotkey functionality may not work correctly");
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Err(e) = crate::platform::check_linux_permissions() {
                error!("❌ Linux hotkey permissions check failed: {}", e);
                error!("Hotkey functionality may not work correctly");
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = crate::platform::check_macos_permissions() {
                error!("❌ macOS hotkey permissions check failed: {}", e);
                error!("Hotkey functionality may not work correctly");
            }
        }
        // ===============================================

        // Initialize resources with Wayland support on Linux
        let hotkey_manager_opt: Option<HotkeyManager> = {
            // ... existing Wayland initialization code ...
        };
        
        // ... rest of plugin initialization ...
    }
}
```

---

### 3. Display Platform Info at Startup

**File**: [`packages/ecs-hotkey/src/lib.rs`](../packages/ecs-hotkey/src/lib.rs)

**Add new startup system function** (after the Plugin impl, around line 360):

```rust
/// Display platform-specific hotkey information at startup
fn display_platform_hotkey_info_system() {
    #[cfg(target_os = "windows")]
    crate::platform::display_windows_hotkey_info();
    
    #[cfg(target_os = "linux")]
    crate::platform::display_linux_hotkey_info();
    
    #[cfg(target_os = "macos")]
    crate::platform::display_macos_hotkey_info();
}
```

**Register system in Plugin::build()** (around line 257, after other startup systems):

```rust
// Add platform-specific startup systems
#[cfg(target_os = "macos")]
app.add_systems(Startup, crate::platform::macos::setup_macos_hotkey_system);

// ADD THIS LINE:
app.add_systems(Startup, display_platform_hotkey_info_system);

// Add profile loading startup system
app.add_systems(Startup, load_hotkey_profiles_startup_system);
```

---

### 4. Fix Dead Code in mod.rs (Optional Cleanup)

**File**: [`packages/ecs-hotkey/src/platform/mod.rs`](../packages/ecs-hotkey/src/platform/mod.rs)

**Current Buggy Code** (lines 23-40):
```rust
pub fn init_platform_hotkeys() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        windows::init_windows_hotkeys().map_err(|e| e.to_string())?;  // ❌ Bug: returns (), not Result
    }
    
    #[cfg(target_os = "linux")]
    {
        linux::init_linux_hotkeys().map_err(|e| e.to_string())?;  // ❌ Bug: returns (), not Result
    }
    
    Ok(())
}
```

**Fix**: Either remove `.map_err()` or delete the function entirely (it's dead code):

**Option 1 - Fix the function**:
```rust
pub fn init_platform_hotkeys() {
    #[cfg(target_os = "windows")]
    windows::init_windows_hotkeys();
    
    #[cfg(target_os = "linux")]
    linux::init_linux_hotkeys();
    
    #[cfg(target_os = "macos")]
    macos::init_macos_hotkeys();
}
```

**Option 2 - Delete it** (recommended if never called):
Just remove lines 23-40 entirely.

---

## Definition of Done

✅ **Task is complete when:**

1. **Error formatting integrated** - systems.rs registration, unregistration, and test error handlers call `format_*_error()`
2. **Permission checks called** - lib.rs Plugin::build() validates permissions before HotkeyManager creation
3. **Info display wired up** - lib.rs adds startup system that calls `display_*_hotkey_info()`
4. **Dead code fixed** - mod.rs init_platform_hotkeys() either fixed or removed
5. **Package compiles** - `cargo check --package ecs-hotkey` succeeds with no errors or warnings
6. **No unwrap/expect** - Integration code uses proper error handling

---

## Implementation Notes

**What NOT to change:**
- Do NOT modify platform/*.rs files - they're correct
- Do NOT modify Wayland integration in lib.rs - it's working
- Do NOT change error types or event structures

**Where to make changes:**
- systems.rs: Error formatting in 3 places (registration, unregistration, testing)
- lib.rs: Permission checks before HotkeyManager creation
- lib.rs: Add display_platform_hotkey_info_system function and register it
- mod.rs: Fix or remove init_platform_hotkeys()

**Testing approach** (manual, no code required):
- Run on Windows and trigger a hotkey conflict → should see formatted error with actionable guidance
- Run on Linux Wayland/X11 → should see compositor-specific messages in logs
- Check startup logs → should see platform-specific "ready" messages

---

## Code Quality Requirements

- Use `#[cfg(target_os = "...")]` attributes consistently
- Handle all platforms: windows, linux, macos, and fallback for others
- No `unwrap()` or `expect()` in integration code
- Use existing error handling patterns (Result<>, warn!, error!)
- Follow existing code style in the files being modified

---

## References

**Existing Platform Implementations**:
- [windows.rs](../packages/ecs-hotkey/src/platform/windows.rs) - Lines 41-80
- [linux.rs](../packages/ecs-hotkey/src/platform/linux.rs) - Lines 49-214
- [mod.rs](../packages/ecs-hotkey/src/platform/mod.rs) - Export declarations

**Integration Target Files**:
- [systems.rs](../packages/ecs-hotkey/src/systems.rs) - Lines 136, 235, 427
- [lib.rs](../packages/ecs-hotkey/src/lib.rs) - Lines 142, 257, 360

**Upstream Library**:
- global-hotkey crate v0.7: https://crates.io/crates/global-hotkey
- Docs: https://docs.rs/global-hotkey/0.7.0/
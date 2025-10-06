//! System hotkey conflict detection
//!
//! Zero-allocation database of known OS-level keyboard shortcuts that may conflict
//! with user-defined hotkeys. Provides proactive warnings before registration.
//!
//! # Performance
//! - Static initialization using `once_cell::Lazy`
//! - O(1) HashMap lookup with zero allocations on hot path
//! - Returns static string references (no clones)
//! - Follows pattern from platform/macos.rs:189-192

use std::collections::HashMap;
use once_cell::sync::Lazy;
use global_hotkey::hotkey::{Code, Modifiers};
use crate::events::HotkeyDefinition;

/// Static registry of system hotkeys for zero-allocation conflict detection
/// 
/// Initialized once on first access, then cached for O(1) lookups.
/// Key: (Modifiers, Code) tuple for direct comparison.
/// Value: Static string slice describing the system shortcut.
///
/// # Platform Behavior
/// - macOS: 7 critical system shortcuts (Spotlight, Mission Control, etc.)
/// - Windows: 8 critical system shortcuts (Search, Task View, etc.)
/// - Linux: Empty (desktop environments too varied for reliable database)
static SYSTEM_HOTKEY_MAP: Lazy<HashMap<(Modifiers, Code), &'static str>> = Lazy::new(|| {
    // Platform-specific shortcut definitions
    let shortcuts: &[(Modifiers, Code, &'static str)] = {
        #[cfg(target_os = "macos")]
        {
            &[
                (Modifiers::META, Code::Space, "Cmd+Space (Spotlight Search)"),
                (Modifiers::CONTROL, Code::ArrowUp, "Ctrl+Up (Mission Control)"),
                (Modifiers::CONTROL, Code::ArrowDown, "Ctrl+Down (Application Windows)"),
                (Modifiers::META, Code::Tab, "Cmd+Tab (App Switcher)"),
                (Modifiers::META, Code::KeyH, "Cmd+H (Hide Window)"),
                (Modifiers::META, Code::KeyQ, "Cmd+Q (Quit Application)"),
                (Modifiers::META, Code::F3, "Cmd+F3 (Show Desktop)"),
            ]
        }
        
        #[cfg(target_os = "windows")]
        {
            &[
                (Modifiers::SUPER, Code::KeyS, "Win+S (Windows Search)"),
                (Modifiers::SUPER, Code::Tab, "Win+Tab (Task View)"),
                (Modifiers::SUPER, Code::KeyL, "Win+L (Lock PC)"),
                (Modifiers::SUPER, Code::KeyD, "Win+D (Show Desktop)"),
                (Modifiers::SUPER, Code::KeyE, "Win+E (File Explorer)"),
                (Modifiers::SUPER, Code::KeyI, "Win+I (Settings)"),
                (Modifiers::SUPER, Code::KeyA, "Win+A (Quick Settings)"),
                (Modifiers::ALT, Code::Tab, "Alt+Tab (App Switcher)"),
            ]
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux desktop environments (GNOME, KDE, XFCE, i3, etc.) have vastly
            // different and user-customizable keybindings. No reliable universal
            // database can be maintained. Users are expected to know their DE shortcuts.
            &[]
        }
    };
    
    // Convert array to HashMap for O(1) lookups
    shortcuts
        .iter()
        .map(|(mods, code, desc)| ((*mods, *code), *desc))
        .collect()
});

/// Check if a hotkey conflicts with a known system shortcut
///
/// Returns static string reference describing the conflicting shortcut if found.
/// 
/// # Performance Characteristics
/// - **Allocations**: Zero (reads from static HashMap)
/// - **Lookup Time**: O(1) hash table access
/// - **Return**: `&'static str` (zero-copy reference, no clone)
/// - **Memory**: One-time initialization cost, then cached forever
///
/// # Example
/// ```rust
/// use ecs_hotkey::system_hotkeys::is_system_hotkey;
/// use ecs_hotkey::events::HotkeyDefinition;
/// use global_hotkey::hotkey::{Code, Modifiers};
///
/// let cmd_space = HotkeyDefinition {
///     modifiers: Modifiers::META,
///     code: Code::Space,
///     description: "My Custom Hotkey".to_string(),
/// };
///
/// if let Some(conflict) = is_system_hotkey(&cmd_space) {
///     println!("Warning: Conflicts with {}", conflict);
///     // Output: "Warning: Conflicts with Cmd+Space (Spotlight Search)"
/// }
/// ```
#[inline]
pub fn is_system_hotkey(definition: &HotkeyDefinition) -> Option<&'static str> {
    SYSTEM_HOTKEY_MAP
        .get(&(definition.modifiers, definition.code))
        .copied()
}

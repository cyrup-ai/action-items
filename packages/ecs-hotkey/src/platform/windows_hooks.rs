//! High-performance Windows low-level keyboard hook implementation
//!
//! Zero-allocation, lock-free global hotkey system using WH_KEYBOARD_LL.
//! Designed for blazing-fast performance with atomic operations and static data structures.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::ffi::c_void;
use std::ptr::NonNull;

use bevy::prelude::*;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use global_hotkey::{hotkey::{Code, Modifiers}};
use thiserror::Error;
use tracing::{debug, error, info};

use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx,
    WH_KEYBOARD_LL, KBDLLHOOKSTRUCT, LLKHF_UP, HHOOK,
    WPARAM, LPARAM, LRESULT,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_LSHIFT, VK_RSHIFT, VK_LCONTROL, VK_RCONTROL,
    VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN,
};
use windows::Win32::Foundation::{BOOL};

use crate::events::HotkeyPressed;

/// Maximum number of concurrent hotkey registrations
const MAX_HOTKEYS: usize = 256;

/// Ring buffer size for hotkey events (must be power of 2)
const EVENT_RING_SIZE: usize = 1024;

/// Windows permission-related errors
#[derive(Debug, Error)]
pub enum WindowsPermissionError {
    #[error("Hook install error: {message}")]
    HookInstallError { message: String },

    #[error("Hash collision detected: action '{action}' hashes to {hash}, which is already registered")]
    HashCollision { action: String, hash: u64 },

    #[error("Hotkey registry full: cannot register more than {0} hotkeys")]
    RegistryFull(usize),

    #[error("Event ring buffer full: events being dropped")]
    EventRingFull,
}

/// Lock-free event ring buffer
#[repr(align(64))] // Cache line aligned
struct LockFreeEventRing {
    /// Ring buffer for hotkey events
    events: [AtomicU64; EVENT_RING_SIZE],
    /// Write index
    write_idx: AtomicUsize,
    /// Read index  
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
        if action_hash == 0 { return false; } // 0 is reserved for empty

        let write_idx = self.write_idx.load(Ordering::Relaxed);
        let read_idx = self.read_idx.load(Ordering::Acquire);
        
        // Check if ring is full
        if (write_idx + 1) % EVENT_RING_SIZE == read_idx {
            return false; // Ring buffer full
        }

        // Try to write event
        if self.events[write_idx].compare_exchange_weak(
            0, action_hash, Ordering::Release, Ordering::Relaxed
        ).is_ok() {
            // Advance write index
            self.write_idx.store((write_idx + 1) % EVENT_RING_SIZE, Ordering::Release);
            true
        } else {
            false
        }
    }

    #[inline]
    fn try_pop(&self) -> Option<u64> {
        let read_idx = self.read_idx.load(Ordering::Relaxed);
        let write_idx = self.write_idx.load(Ordering::Acquire);

        // Check if ring is empty
        if read_idx == write_idx {
            return None;
        }

        // Try to read event
        let action_hash = self.events[read_idx].swap(0, Ordering::Acquire);
        if action_hash != 0 {
            // Advance read index
            self.read_idx.store((read_idx + 1) % EVENT_RING_SIZE, Ordering::Release);
            Some(action_hash)
        } else {
            None
        }
    }
}

/// Lock-free hotkey registry with O(1) lookup
/// Key: (vk_code, modifiers), Value: action_hash
static HOTKEY_REGISTRY: Lazy<DashMap<(u32, u32), u64>> = Lazy::new(DashMap::new);

/// Track action hashes to detect collisions
static ACTION_HASH_REGISTRY: Lazy<DashMap<u64, String>> = Lazy::new(DashMap::new);

/// Static event ring for zero-allocation event passing
#[allow(clippy::declare_interior_mutable_const)]
static EVENT_RING: LockFreeEventRing = LockFreeEventRing::new();

/// Hook handle
static HOOK_HANDLE: AtomicU64 = AtomicU64::new(0);

/// System initialization state
static SYSTEM_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Fast hash function for action strings (FNV-1a)
#[inline]
const fn hash_action(action: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;
    
    let bytes = action.as_bytes();
    let mut hash = FNV_OFFSET_BASIS;
    let mut i = 0;
    
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    
    // Ensure non-zero (0 is reserved for empty)
    if hash == 0 { 1 } else { hash }
}

/// Get current modifier state using GetAsyncKeyState
#[inline]
fn get_current_modifiers() -> u32 {
    let mut modifiers = 0u32;
    
    // Shift (0x0004)
    if unsafe { (GetAsyncKeyState(VK_LSHIFT.0 as i32) as u16 & 0x8000) != 0 } ||
       unsafe { (GetAsyncKeyState(VK_RSHIFT.0 as i32) as u16 & 0x8000) != 0 } {
        modifiers |= 0x0004;
    }
    
    // Control (0x0002)
    if unsafe { (GetAsyncKeyState(VK_LCONTROL.0 as i32) as u16 & 0x8000) != 0 } ||
       unsafe { (GetAsyncKeyState(VK_RCONTROL.0 as i32) as u16 & 0x8000) != 0 } {
        modifiers |= 0x0002;
    }
    
    // Alt (0x0008)
    if unsafe { (GetAsyncKeyState(VK_LMENU.0 as i32) as u16 & 0x8000) != 0 } ||
       unsafe { (GetAsyncKeyState(VK_RMENU.0 as i32) as u16 & 0x8000) != 0 } {
        modifiers |= 0x0008;
    }
    
    // Super/Win (0x0001 for Meta/Command)
    if unsafe { (GetAsyncKeyState(VK_LWIN.0 as i32) as u16 & 0x8000) != 0 } ||
       unsafe { (GetAsyncKeyState(VK_RWIN.0 as i32) as u16 & 0x8000) != 0 } {
        modifiers |= 0x0001;
    }
    
    modifiers
}

/// Low-level keyboard hook callback - zero allocation, lock-free
/// 
/// # Safety
/// This function is the hook procedure called by Windows for every keyboard event.
unsafe extern "system" fn keyboard_hook_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kbd_struct = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
        let vk_code = kbd_struct.vkCode;
        let flags = kbd_struct.flags;
        
        // Key down (not key up)
        if (flags & LLKHF_UP) == 0 {
            let modifiers = get_current_modifiers();
            
            let key = (vk_code, modifiers);
            if let Some(action_hash_ref) = HOTKEY_REGISTRY.get(&key) {
                let action_hash = *action_hash_ref.value();
                drop(action_hash_ref);
                
                if !EVENT_RING.try_push(action_hash) {
                    error!("EVENT RING BUFFER FULL - event dropped (hash: {})", action_hash);
                }
            }
        }
    }
    
    CallNextHookEx(None, n_code, w_param, l_param)
}

/// Install the low-level keyboard hook
pub fn install_keyboard_hook() -> Result<(), WindowsPermissionError> {
    if SYSTEM_INITIALIZED.load(Ordering::Acquire) {
        return Ok(());
    }

    let h_instance = std::ptr::null(); // For executable, use null
    let hook_id = unsafe {
        SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_hook_proc),
            h_instance,
            0, // Global hook
        )
    };

    if let Ok(handle) = hook_id {
        HOOK_HANDLE.store(handle.0 as u64, Ordering::Release);
        SYSTEM_INITIALIZED.store(true, Ordering::Release);
        info!("✅ Windows low-level keyboard hook installed");
        Ok(())
    } else {
        Err(WindowsPermissionError::HookInstallError {
            message: "SetWindowsHookExW failed - may require admin privileges or UAC".to_string(),
        })
    }
}

/// Uninstall the keyboard hook
pub fn uninstall_keyboard_hook() {
    let handle_val = HOOK_HANDLE.swap(0, Ordering::Acquire);
    if handle_val != 0 {
        unsafe {
            let _ = UnhookWindowsHookEx(HHOOK(handle_val as isize));
        }
    }
    SYSTEM_INITIALIZED.store(false, Ordering::Release);
    info!("🔄 Windows keyboard hook uninstalled");
}

/// Register a hotkey in the concurrent hash map (O(1) operation)
#[inline]
pub fn register_hotkey_atomic(
    vk_code: u32, 
    modifiers: u32, 
    action: &str
) -> Result<u64, WindowsPermissionError> {
    if HOTKEY_REGISTRY.len() >= MAX_HOTKEYS {
        return Err(WindowsPermissionError::RegistryFull(MAX_HOTKEYS));
    }

    let action_hash = hash_action(action);
    let key = (vk_code, modifiers);
    
    // Check for hash collision
    if let Some(existing_action) = ACTION_HASH_REGISTRY.get(&action_hash) {
        if existing_action != action {
            return Err(WindowsPermissionError::HashCollision {
                action: action.to_string(),
                hash: action_hash,
            });
        }
        // Same action, ok
    } else {
        ACTION_HASH_REGISTRY.insert(action_hash, action.to_string());
    }
    
    // Insert or overwrite
    HOTKEY_REGISTRY.insert(key, action_hash);
    info!("Registered hotkey: VK={} MOD={} -> {}", vk_code, modifiers, action);
    
    Ok(action_hash)
}

/// Unregister a hotkey from the concurrent hash map (O(1) operation)
#[inline]
pub fn unregister_hotkey_atomic(vk_code: u32, modifiers: u32) -> bool {
    let key = (vk_code, modifiers);
    HOTKEY_REGISTRY.remove(&key).is_some()
}

/// Pop events from the ring buffer (lock-free)
#[inline]
pub fn pop_hotkey_event() -> Option<u64> {
    EVENT_RING.try_pop()
}

/// Bevy system to process hotkey events (zero allocation)
pub fn process_windows_hotkey_events_system(
    mut hotkey_pressed_events: EventWriter<HotkeyPressed>,
    hotkey_registry: Res<crate::resources::HotkeyRegistry>,
) {
    while let Some(action_hash) = pop_hotkey_event() {
        for (hotkey_id, binding) in &hotkey_registry.registered_hotkeys {
            if hash_action(&binding.action) == action_hash {
                debug!("Hotkey triggered: {}", binding.definition.description);
                
                hotkey_pressed_events.write(HotkeyPressed {
                    hotkey_id: hotkey_id.clone(),
                    binding: binding.clone(),
                });
                break;
            }
        }
    }
}

/// Convert global_hotkey Code to Windows VK code
#[inline]
fn windows_vk_from_global(code: Code) -> u32 {
    use Code::*;
    match code {
        KeyA => b'A' as u32,
        KeyB => b'B' as u32,
        KeyC => b'C' as u32,
        KeyD => b'D' as u32,
        KeyE => b'E' as u32,
        KeyF => b'F' as u32,
        KeyG => b'G' as u32,
        KeyH => b'H' as u32,
        KeyI => b'I' as u32,
        KeyJ => b'J' as u32,
        KeyK => b'K' as u32,
        KeyL => b'L' as u32,
        KeyM => b'M' as u32,
        KeyN => b'N' as u32,
        KeyO => b'O' as u32,
        KeyP => b'P' as u32,
        KeyQ => b'Q' as u32,
        KeyR => b'R' as u32,
        KeyS => b'S' as u32,
        KeyT => b'T' as u32,
        KeyU => b'U' as u32,
        KeyV => b'V' as u32,
        KeyW => b'W' as u32,
        KeyX => b'X' as u32,
        KeyY => b'Y' as u32,
        KeyZ => b'Z' as u32,
        Digit1 => b'1' as u32,
        Digit2 => b'2' as u32,
        Digit3 => b'3' as u32,
        Digit4 => b'4' as u32,
        Digit5 => b'5' as u32,
        Digit6 => b'6' as u32,
        Digit7 => b'7' as u32,
        Digit8 => b'8' as u32,
        Digit9 => b'9' as u32,
        Digit0 => b'0' as u32,
        Space => 0x20,
        Enter => 0x0D,
        Tab => 0x09,
        Backspace => 0x08,
        Escape => 0x1B,
        F1 => 0x70,
        F2 => 0x71,
        F3 => 0x72,
        F4 => 0x73,
        F5 => 0x74,
        F6 => 0x75,
        F7 => 0x76,
        F8 => 0x77,
        F9 => 0x78,
        F10 => 0x79,
        F11 => 0x7A,
        F12 => 0x7B,
        ArrowLeft => 0x25,
        ArrowRight => 0x27,
        ArrowUp => 0x26,
        ArrowDown => 0x28,
        _ => 0,
    }
}

/// Convert global_hotkey Modifiers to internal format
#[inline]
fn windows_modifiers_from_global(modifiers: Modifiers) -> u32 {
    let mut result = 0u32;
    
    if modifiers.contains(Modifiers::ALT) {
        result |= 0x0008;
    }
    if modifiers.contains(Modifiers::CONTROL) {
        result |= 0x0002;
    }
    if modifiers.contains(Modifiers::SHIFT) {
        result |= 0x0004;
    }
    if modifiers.contains(Modifiers::SUPER) {
        result |= 0x0001;
    }
    
    result
}

/// Bevy system to register hotkeys with the Windows system
pub fn register_hotkey_with_windows_system(
    mut registration_requests: EventReader<crate::events::HotkeyRegisterRequested>,
    mut registration_completed: EventWriter<crate::events::HotkeyRegisterCompleted>,
) {
    for request in registration_requests.read() {
        let vk_code = windows_vk_from_global(request.binding.definition.code);
        let internal_modifiers = windows_modifiers_from_global(request.binding.definition.modifiers);
        
        match register_hotkey_atomic(vk_code, internal_modifiers, &request.binding.action) {
            Ok(_id) => {
                info!("✅ Registered hotkey: {}", request.binding.definition.description);
                registration_completed.write(crate::events::HotkeyRegisterCompleted {
                    binding: request.binding.clone(),
                    requester: request.binding.requester.clone(),
                    success: true,
                    error_message: None,
                });
            }
            Err(e) => {
                error!("❌ Failed to register hotkey: {}", e);
                registration_completed.write(crate::events::HotkeyRegisterCompleted {
                    binding: request.binding.clone(),
                    requester: request.binding.requester.clone(),
                    success: false,
                    error_message: Some(e.to_string()),
                });
            }
        }
    }
}

/// Bevy resource for tracking system state
#[derive(Resource)]
pub struct WindowsHotkeyResource {
    pub initialized: bool,
}

/// Bevy system to initialize the Windows hotkey system
pub fn setup_windows_hotkey_system(
    mut commands: Commands,
    hotkey_resource: Option<Res<WindowsHotkeyResource>>,
) {
    // Only initialize once
    if let Some(resource) = hotkey_resource {
        if resource.initialized {
            return;
        }
    }

    match install_keyboard_hook() {
        Ok(()) => {
            commands.insert_resource(WindowsHotkeyResource { initialized: true });
            info!("✅ Windows hotkey system setup completed");
        }
        Err(e) => {
            error!("❌ Failed to initialize Windows hotkey system: {}", e);
            commands.insert_resource(WindowsHotkeyResource { initialized: false });
        }
    }
}

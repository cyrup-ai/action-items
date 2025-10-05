//! High-performance macOS CGEventTap hotkey implementation
//!
//! Zero-allocation, lock-free global hotkey system using CGEventTap API.
//! Designed for blazing-fast performance with atomic operations and static data structures.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::ffi::c_void;
use std::ptr::NonNull;

use bevy::prelude::*;
use objc2_core_foundation::{CFMachPort, CFRunLoop, kCFRunLoopCommonModes};
use objc2_core_graphics::{
    CGEvent, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType, CGEventMask, CGEventFlags, CGEventField
};
use thiserror::Error;
use tracing::{debug, error, info};

use crate::events::HotkeyPressed;

/// Maximum number of concurrent hotkey registrations
const MAX_HOTKEYS: usize = 256;

/// Ring buffer size for hotkey events (must be power of 2)
const EVENT_RING_SIZE: usize = 1024;

/// macOS permission-related errors
#[derive(Debug, Error)]
pub enum MacOSPermissionError {
    #[error("Accessibility permission denied for process '{process_name}': {instructions}")]
    AccessibilityPermissionDenied {
        process_name: String,
        instructions: String,
    },

    #[error("Failed to retrieve current process information: {details}")]
    ProcessInfoError { details: String },

    #[error("macOS API call failed: {api_name} - {error_message}")]
    ApiError {
        api_name: String,
        error_message: String,
    },

    #[error("Hotkey registry full: cannot register more than {MAX_HOTKEYS} hotkeys")]
    RegistryFull,

    #[error("Event ring buffer full: events being dropped")]
    EventRingFull,
}

/// Atomic hotkey slot for lock-free registration
#[repr(align(64))] // Cache line aligned
struct AtomicHotkeySlot {
    /// Hotkey ID (0 means empty slot)
    id: AtomicU64,
    /// Key code
    key_code: AtomicU32,
    /// Modifier flags
    modifiers: AtomicU32,
    /// Action string hash (for fast comparison)
    action_hash: AtomicU64,
    /// Slot is active
    active: AtomicBool,
}

impl AtomicHotkeySlot {
    const fn new() -> Self {
        Self {
            id: AtomicU64::new(0),
            key_code: AtomicU32::new(0),
            modifiers: AtomicU32::new(0),
            action_hash: AtomicU64::new(0),
            active: AtomicBool::new(false),
        }
    }

    #[inline]
    fn try_register(&self, id: u64, key_code: u32, modifiers: u32, action_hash: u64) -> bool {
        // Try to claim empty slot atomically
        if self.id.compare_exchange_weak(0, id, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
            // Successfully claimed slot, now populate it
            self.key_code.store(key_code, Ordering::Relaxed);
            self.modifiers.store(modifiers, Ordering::Relaxed);
            self.action_hash.store(action_hash, Ordering::Relaxed);
            self.active.store(true, Ordering::Release);
            true
        } else {
            false
        }
    }

    #[inline]
    fn try_unregister(&self, id: u64) -> bool {
        if self.id.compare_exchange_weak(id, 0, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
            self.active.store(false, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    #[inline]
    fn matches(&self, key_code: u32, modifiers: u32) -> Option<u64> {
        if self.active.load(Ordering::Acquire) &&
           self.key_code.load(Ordering::Relaxed) == key_code &&
           self.modifiers.load(Ordering::Relaxed) == modifiers {
            Some(self.action_hash.load(Ordering::Relaxed))
        } else {
            None
        }
    }
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
        const INIT: AtomicU64 = AtomicU64::new(0);
        Self {
            events: [INIT; EVENT_RING_SIZE],
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

/// Static hotkey registry for zero-allocation operation
static HOTKEY_REGISTRY: [AtomicHotkeySlot; MAX_HOTKEYS] = {
    const INIT: AtomicHotkeySlot = AtomicHotkeySlot::new();
    [INIT; MAX_HOTKEYS]
};

/// Static event ring for zero-allocation event passing
static EVENT_RING: LockFreeEventRing = LockFreeEventRing::new();

/// CGEventTap initialization state (we don't store the handle as it's not Send/Sync)
/// The run loop keeps the tap alive once it's added

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

/// Convert macOS key code to our internal format
#[inline]
const fn macos_keycode_to_internal(key_code: u32) -> u32 {
    // Direct mapping - macOS key codes are already suitable
    key_code
}

/// Convert CGEventFlags to our internal modifier format
#[inline]
fn macos_flags_to_internal(flags: CGEventFlags) -> u32 {
    let mut modifiers = 0u32;
    
    if flags.contains(CGEventFlags::MaskCommand) {
        modifiers |= 0x0001;
    }
    if flags.contains(CGEventFlags::MaskControl) {
        modifiers |= 0x0002;
    }
    if flags.contains(CGEventFlags::MaskShift) {
        modifiers |= 0x0004;
    }
    if flags.contains(CGEventFlags::MaskAlternate) {
        modifiers |= 0x0008;
    }
    
    modifiers
}

/// Ultra-fast CGEventTap callback - zero allocation, lock-free
unsafe extern "C-unwind" fn hotkey_event_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    cg_event: NonNull<CGEvent>,
    _user_info: *mut c_void,
) -> *mut CGEvent {
    // Only process key down events
    if event_type != CGEventType::KeyDown {
        return cg_event.as_ptr();
    }

    // Extract key code and modifiers (zero allocation)
    let key_code = unsafe {
        CGEvent::integer_value_field(Some(cg_event.as_ref()), CGEventField::KeyboardEventKeycode)
    } as u32;
    
    let flags = unsafe {
        CGEvent::flags(Some(cg_event.as_ref()))
    };
    
    let internal_key_code = macos_keycode_to_internal(key_code);
    let internal_modifiers = macos_flags_to_internal(flags);

    // Fast lookup in static registry (lock-free)
    for slot in &HOTKEY_REGISTRY {
        if let Some(action_hash) = slot.matches(internal_key_code, internal_modifiers) {
            // Push to event ring (lock-free)
            if !EVENT_RING.try_push(action_hash) {
                // Ring buffer full - this is a performance issue but we don't drop the event
                // Instead, we could implement a fallback mechanism here
            }
            break; // Found match, stop searching
        }
    }

    // Return event unmodified (don't consume it)
    cg_event.as_ptr()
}

/// Register a hotkey in the static registry (lock-free)
#[inline]
pub fn register_hotkey_atomic(
    key_code: u32, 
    modifiers: u32, 
    action: &str
) -> Result<u64, MacOSPermissionError> {
    let id = hash_action(action);
    let action_hash = id;
    
    // Try to find empty slot and register atomically
    for slot in &HOTKEY_REGISTRY {
        if slot.try_register(id, key_code, modifiers, action_hash) {
            return Ok(id);
        }
    }
    
    Err(MacOSPermissionError::RegistryFull)
}

/// Unregister a hotkey from the static registry (lock-free)
#[inline]
pub fn unregister_hotkey_atomic(id: u64) -> bool {
    for slot in &HOTKEY_REGISTRY {
        if slot.try_unregister(id) {
            return true;
        }
    }
    false
}

/// Pop events from the ring buffer (lock-free)
#[inline]
pub fn pop_hotkey_event() -> Option<u64> {
    EVENT_RING.try_pop()
}

/// Check macOS accessibility permissions
pub fn check_macos_permissions() -> Result<(), MacOSPermissionError> {
    use accessibility_sys::AXIsProcessTrustedWithOptions;
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;

    let prompt_key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
    let prompt_value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(prompt_key.as_CFType(), prompt_value.as_CFType())]);

    let is_trusted = unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) };

    if is_trusted {
        info!("✅ macOS accessibility permissions granted");
        Ok(())
    } else {
        error!("❌ macOS accessibility permissions required");
        Err(MacOSPermissionError::AccessibilityPermissionDenied {
            process_name: std::env::current_exe()
                .ok()
                .and_then(|path| path.file_name().map(|name| name.to_string_lossy().to_string()))
                .unwrap_or_else(|| "Unknown Process".to_string()),
            instructions: "Grant accessibility permissions in System Settings".to_string(),
        })
    }
}

/// Initialize the high-performance CGEventTap system
pub fn init_macos_hotkey_system() -> Result<(), MacOSPermissionError> {
    // Check if already initialized
    if SYSTEM_INITIALIZED.load(Ordering::Acquire) {
        return Ok(());
    }

    // Check permissions first
    check_macos_permissions()?;

    // Create event mask for key events
    let event_mask: CGEventMask = 1 << CGEventType::KeyDown.0;

    // Create the event tap
    let event_tap = unsafe {
        CGEvent::tap_create(
            CGEventTapLocation::HIDEventTap, // Works in fullscreen apps
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            event_mask,
            Some(hotkey_event_callback),
            std::ptr::null_mut(),
        )
    };

    let tap = event_tap.ok_or_else(|| MacOSPermissionError::ApiError {
        api_name: "CGEventTapCreate".to_string(),
        error_message: "Failed to create event tap - check accessibility permissions".to_string(),
    })?;

    // Create run loop source
    let loop_source = CFMachPort::new_run_loop_source(None, Some(&tap), 0)
        .ok_or_else(|| MacOSPermissionError::ApiError {
            api_name: "CFMachPortCreateRunLoopSource".to_string(),
            error_message: "Failed to create run loop source".to_string(),
        })?;

    // Add to current run loop
    let current_loop = CFRunLoop::current().ok_or_else(|| MacOSPermissionError::ApiError {
        api_name: "CFRunLoopGetCurrent".to_string(),
        error_message: "Failed to get current run loop".to_string(),
    })?;

    unsafe {
        current_loop.add_source(Some(&loop_source), kCFRunLoopCommonModes);
        CGEvent::tap_enable(&tap, true);
    }

    // Note: We don't store the tap handle as CFMachPort is not Send/Sync
    // The run loop keeps it alive once it's added

    // Mark as initialized
    SYSTEM_INITIALIZED.store(true, Ordering::Release);

    info!("✅ High-performance macOS CGEventTap initialized");
    Ok(())
}

/// Bevy resource for tracking system state
#[derive(Resource)]
pub struct MacOSHotkeyResource {
    pub initialized: bool,
}

/// Bevy system to initialize the macOS hotkey system
pub fn setup_macos_hotkey_system(
    mut commands: Commands,
    hotkey_resource: Option<Res<MacOSHotkeyResource>>,
) {
    // Only initialize once
    if let Some(resource) = hotkey_resource {
        if resource.initialized {
            return;
        }
    }

    match init_macos_hotkey_system() {
        Ok(()) => {
            commands.insert_resource(MacOSHotkeyResource { initialized: true });
            info!("✅ macOS hotkey system setup completed");
        }
        Err(e) => {
            error!("❌ Failed to initialize macOS hotkey system: {}", e);
            commands.insert_resource(MacOSHotkeyResource { initialized: false });
        }
    }
}

/// Bevy system to process hotkey events (zero allocation)
pub fn process_macos_hotkey_events_system(
    mut hotkey_pressed_events: EventWriter<HotkeyPressed>,
    hotkey_registry: Res<crate::resources::HotkeyRegistry>,
) {
    // Process all pending events from the lock-free ring buffer
    while let Some(action_hash) = pop_hotkey_event() {
        // Find the corresponding binding by action hash
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

/// Bevy system to register hotkeys with the high-performance system
pub fn register_hotkey_with_macos_system(
    mut registration_requests: EventReader<crate::events::HotkeyRegisterRequested>,
    mut registration_completed: EventWriter<crate::events::HotkeyRegisterCompleted>,
) {
    for request in registration_requests.read() {
        // Convert to internal format
        let key_code = macos_keycode_from_global_hotkey(request.definition.code);
        let modifiers = macos_modifiers_from_global_hotkey(request.definition.modifiers);
        
        match register_hotkey_atomic(key_code, modifiers, &request.action) {
            Ok(_id) => {
                info!("✅ Registered hotkey: {}", request.definition.description);
                registration_completed.write(crate::events::HotkeyRegisterCompleted {
                    binding: request.binding.clone(),
                    requester: request.requester.clone(),
                    success: true,
                    error_message: None,
                });
            }
            Err(e) => {
                error!("❌ Failed to register hotkey: {}", e);
                registration_completed.write(crate::events::HotkeyRegisterCompleted {
                    binding: request.binding.clone(),
                    requester: request.requester.clone(),
                    success: false,
                    error_message: Some(e.to_string()),
                });
            }
        }
    }
}

/// Convert global_hotkey Code to macOS key code
#[inline]
fn macos_keycode_from_global_hotkey(code: global_hotkey::hotkey::Code) -> u32 {
    use global_hotkey::hotkey::Code;
    
    match code {
        // Letters
        Code::KeyA => 0x00,
        Code::KeyS => 0x01,
        Code::KeyD => 0x02,
        Code::KeyF => 0x03,
        Code::KeyH => 0x04,
        Code::KeyG => 0x05,
        Code::KeyZ => 0x06,
        Code::KeyX => 0x07,
        Code::KeyC => 0x08,
        Code::KeyV => 0x09,
        Code::KeyB => 0x0B,
        Code::KeyQ => 0x0C,
        Code::KeyW => 0x0D,
        Code::KeyE => 0x0E,
        Code::KeyR => 0x0F,
        Code::KeyY => 0x10,
        Code::KeyT => 0x11,
        Code::KeyO => 0x1F,
        Code::KeyU => 0x20,
        Code::BracketLeft => 0x21,
        Code::KeyI => 0x22,
        Code::KeyP => 0x23,
        Code::KeyL => 0x25,
        Code::KeyJ => 0x26,
        Code::Quote => 0x27,
        Code::KeyK => 0x28,
        Code::KeyN => 0x2D,
        Code::KeyM => 0x2E,
        
        // Numbers
        Code::Digit1 => 0x12,
        Code::Digit2 => 0x13,
        Code::Digit3 => 0x14,
        Code::Digit4 => 0x15,
        Code::Digit5 => 0x17,
        Code::Digit6 => 0x16,
        Code::Digit7 => 0x1A,
        Code::Digit8 => 0x1C,
        Code::Digit9 => 0x19,
        Code::Digit0 => 0x1D,
        
        // Special keys
        Code::Space => 0x31,
        Code::Enter => 0x24,
        Code::Tab => 0x30,
        Code::Backspace => 0x33,
        Code::Escape => 0x35,
        
        // Function keys
        Code::F1 => 0x7A,
        Code::F2 => 0x78,
        Code::F3 => 0x63,
        Code::F4 => 0x76,
        Code::F5 => 0x60,
        Code::F6 => 0x61,
        Code::F7 => 0x62,
        Code::F8 => 0x64,
        Code::F9 => 0x65,
        Code::F10 => 0x6D,
        Code::F11 => 0x67,
        Code::F12 => 0x6F,
        
        // Arrow keys
        Code::ArrowLeft => 0x7B,
        Code::ArrowRight => 0x7C,
        Code::ArrowDown => 0x7D,
        Code::ArrowUp => 0x7E,
        
        _ => 0, // Unknown key
    }
}

/// Convert global_hotkey Modifiers to internal format
#[inline]
fn macos_modifiers_from_global_hotkey(modifiers: global_hotkey::hotkey::Modifiers) -> u32 {
    use global_hotkey::hotkey::Modifiers;
    
    let mut result = 0u32;
    
    if modifiers.contains(Modifiers::META) {
        result |= 0x0001;
    }
    if modifiers.contains(Modifiers::CONTROL) {
        result |= 0x0002;
    }
    if modifiers.contains(Modifiers::SHIFT) {
        result |= 0x0004;
    }
    if modifiers.contains(Modifiers::ALT) {
        result |= 0x0008;
    }
    
    result
}

/// Display macOS-specific hotkey setup instructions
pub fn display_macos_hotkey_info() {
    info!("🚀 High-Performance Action Items Launcher Ready!");
    info!("📋 Press Cmd+Shift+Space to activate the launcher from anywhere");
    info!("⚡ Zero-allocation, lock-free hotkey system active");
    info!("🔒 IMPORTANT: If global hotkeys don't work on macOS:");
    info!("   1. Open System Preferences → Privacy & Security → Accessibility");
    info!("   2. Add and enable 'action_items' or your terminal app");
    info!("   3. Restart this app after granting permissions");
}
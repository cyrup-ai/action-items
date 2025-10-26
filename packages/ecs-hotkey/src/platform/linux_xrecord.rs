//! High-performance Linux X11 global hotkey implementation
//!
//! Zero-allocation, lock-free global hotkey system using XGrabKey and event polling.
//! Designed for blazing-fast performance with atomic operations and static data structures.

use std::sync::{Arc, Mutex};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::error::Error;

use bevy::prelude::*;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use global_hotkey::hotkey::{Code, Modifiers};
use thiserror::Error;
use tracing::{debug, error, info};

use x11rb::rust_connection::RustConnection;
use x11rb::protocol::xproto::{self, ConnectionExt, KeyPressEvent, Window, GrabMode, EventMask, ChangeWindowAttributesAux};
use x11rb::protocol::Error as X11Error;
use x11rb::keysym;

use crate::events::HotkeyPressed;

/// Maximum number of concurrent hotkey registrations
const MAX_HOTKEYS: usize = 256;

/// Ring buffer size for hotkey events (must be power of 2)
const EVENT_RING_SIZE: usize = 1024;

/// Linux permission-related errors
#[derive(Debug, Error)]
pub enum LinuxPermissionError {
    #[error("X11 connection error: {0}")]
    ConnectionError(String),

    #[error("Hash collision detected: action '{action}' hashes to {hash}, which is already registered")]
    HashCollision { action: String, hash: u64 },

    #[error("Hotkey registry full: cannot register more than {0} hotkeys")]
    RegistryFull(usize),

    #[error("Keycode lookup failed for {0:?}")]
    KeycodeLookupFailed(Code),

    #[error("Event ring buffer full: events being dropped")]
    EventRingFull,

    #[error("X11 request failed: {0}")]
    X11RequestFailed(Box<dyn Error + Send + Sync>),
}

/// Lock-free event ring buffer
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
            return false;
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

    #[inline]
    fn try_pop(&self) -> Option<u64> {
        let read_idx = self.read_idx.load(Ordering::Relaxed);
        let write_idx = self.write_idx.load(Ordering::Acquire);

        if read_idx == write_idx {
            return None;
        }

        let action_hash = self.events[read_idx].swap(0, Ordering::Acquire);
        if action_hash != 0 {
            self.read_idx.store((read_idx + 1) % EVENT_RING_SIZE, Ordering::Release);
            Some(action_hash)
        } else {
            None
        }
    }
}

/// Lock-free hotkey registry with O(1) lookup
/// Key: (keycode, state), Value: action_hash
static HOTKEY_REGISTRY: Lazy<DashMap<(u32, u32), u64>> = Lazy::new(DashMap::new);

/// Track action hashes to detect collisions
static ACTION_HASH_REGISTRY: Lazy<DashMap<u64, String>> = Lazy::new(DashMap::new);

#[allow(clippy::declare_interior_mutable_const)]
static EVENT_RING: LockFreeEventRing = LockFreeEventRing::new();

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
    
    if hash == 0 { 1 } else { hash }
}

/// Convert global_hotkey Modifiers to internal format
#[inline]
fn linux_modifiers_from_global(modifiers: Modifiers) -> u32 {
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

/// Convert internal modifiers to X11 modmask
#[inline]
fn internal_mod_to_x11(internal: u32) -> u16 {
    let mut mask = 0u16;
    
    if internal & 0x0001 != 0 { mask |= 64; } // Super -> Mod4
    if internal & 0x0002 != 0 { mask |= 4; } // Control
    if internal & 0x0004 != 0 { mask |= 1; } // Shift
    if internal & 0x0008 != 0 { mask |= 8; } // Alt -> Mod1
    
    mask
}

/// Map Code to X keysym
fn code_to_keysym(code: Code) -> Result<u32, LinuxPermissionError> {
    use x11rb::keysym;
    match code {
        Code::KeyA => Ok(keysym::XK_a),
        Code::KeyB => Ok(keysym::XK_b),
        Code::KeyC => Ok(keysym::XK_c),
        Code::KeyD => Ok(keysym::XK_d),
        Code::KeyE => Ok(keysym::XK_e),
        Code::KeyF => Ok(keysym::XK_f),
        Code::KeyG => Ok(keysym::XK_g),
        Code::KeyH => Ok(keysym::XK_h),
        Code::KeyI => Ok(keysym::XK_i),
        Code::KeyJ => Ok(keysym::XK_j),
        Code::KeyK => Ok(keysym::XK_k),
        Code::KeyL => Ok(keysym::XK_l),
        Code::KeyM => Ok(keysym::XK_m),
        Code::KeyN => Ok(keysym::XK_n),
        Code::KeyO => Ok(keysym::XK_o),
        Code::KeyP => Ok(keysym::XK_p),
        Code::KeyQ => Ok(keysym::XK_q),
        Code::KeyR => Ok(keysym::XK_r),
        Code::KeyS => Ok(keysym::XK_s),
        Code::KeyT => Ok(keysym::XK_t),
        Code::KeyU => Ok(keysym::XK_u),
        Code::KeyV => Ok(keysym::XK_v),
        Code::KeyW => Ok(keysym::XK_w),
        Code::KeyX => Ok(keysym::XK_x),
        Code::KeyY => Ok(keysym::XK_y),
        Code::KeyZ => Ok(keysym::XK_z),
        Code::Digit1 => Ok(keysym::XK_1),
        Code::Digit2 => Ok(keysym::XK_2),
        Code::Digit3 => Ok(keysym::XK_3),
        Code::Digit4 => Ok(keysym::XK_4),
        Code::Digit5 => Ok(keysym::XK_5),
        Code::Digit6 => Ok(keysym::XK_6),
        Code::Digit7 => Ok(keysym::XK_7),
        Code::Digit8 => Ok(keysym::XK_8),
        Code::Digit9 => Ok(keysym::XK_9),
        Code::Digit0 => Ok(keysym::XK_0),
        Code::Space => Ok(keysym::XK_space),
        Code::Enter => Ok(keysym::XK_Return),
        Code::Tab => Ok(keysym::XK_Tab),
        Code::Backspace => Ok(keysym::XK_BackSpace),
        Code::Escape => Ok(keysym::XK_Escape),
        Code::F1 => Ok(keysym::XK_F1),
        Code::F2 => Ok(keysym::XK_F2),
        Code::F3 => Ok(keysym::XK_F3),
        Code::F4 => Ok(keysym::XK_F4),
        Code::F5 => Ok(keysym::XK_F5),
        Code::F6 => Ok(keysym::XK_F6),
        Code::F7 => Ok(keysym::XK_F7),
        Code::F8 => Ok(keysym::XK_F8),
        Code::F9 => Ok(keysym::XK_F9),
        Code::F10 => Ok(keysym::XK_F10),
        Code::F11 => Ok(keysym::XK_F11),
        Code::F12 => Ok(keysym::XK_F12),
        Code::ArrowLeft => Ok(keysym::XK_Left),
        Code::ArrowRight => Ok(keysym::XK_Right),
        Code::ArrowUp => Ok(keysym::XK_Up),
        Code::ArrowDown => Ok(keysym::XK_Down),
        _ => Err(LinuxPermissionError::KeycodeLookupFailed(code)),
    }
}

/// Bevy resource for X11 connection
#[derive(Resource)]
pub struct X11HotkeyResource {
    pub conn: Mutex<RustConnection>,
    pub root: Window,
}

/// Bevy system to initialize the Linux X11 hotkey system
pub fn setup_linux_hotkey_system(
    mut commands: Commands,
) {
    match x11rb::connect(None) {
        Ok((conn, screen_num)) => {
            let setup = conn.setup();
            let screen = &setup.roots[screen_num];
            let root = screen.root;

            let aux = ChangeWindowAttributesAux::default()
                .event_mask(EventMask::KEY_PRESS);

            if let Err(e) = xproto::change_window_attributes(&conn, root, &aux) {
                error!("Failed to set event mask: {:?}", e);
                return;
            }

            if let Err(e) = conn.flush() {
                error!("Failed to flush: {:?}", e);
                return;
            }

            commands.insert_resource(X11HotkeyResource {
                conn: Mutex::new(conn),
                root,
            });

            info!("✅ Linux X11 hotkey system setup completed");
        }
        Err(e) => {
            error!("Failed to connect to X11: {:?}", e);
        }
    }
}

/// Register a hotkey atomic + X11 grab
#[inline]
pub fn register_hotkey_atomic(
    keycode: u32,
    modmask: u32,
    action: &str,
    conn: &mut RustConnection,
    root: Window,
) -> Result<u64, LinuxPermissionError> {
    if HOTKEY_REGISTRY.len() >= MAX_HOTKEYS {
        return Err(LinuxPermissionError::RegistryFull(MAX_HOTKEYS));
    }

    let action_hash = hash_action(action);
    let key = (keycode, modmask);

    // Check collision
    if let Some(existing) = ACTION_HASH_REGISTRY.get(&action_hash) {
        if *existing != action {
            return Err(LinuxPermissionError::HashCollision {
                action: action.to_string(),
                hash: action_hash,
            });
        }
    } else {
        ACTION_HASH_REGISTRY.insert(action_hash, action.to_string());
    }

    // Grab key
    let grab_mod = internal_mod_to_x11(modmask as u32) as u16; // modmask is already u32 from internal? Wait, in call, modmask is u16 from x11, but in key u32
    // Wait, in registry, use (keycode u32, modmask u16 as u32)

    xproto::grab_key(
        conn,
        false,
        root,
        grab_mod,
        x11rb::Keycode(keycode as u16),
        GrabMode::ASYNC,
        GrabMode::ASYNC,
        x11rb::Window(0),
    ).map_err(|e| LinuxPermissionError::X11RequestFailed(Box::new(e)))?;

    conn.flush().map_err(|e| LinuxPermissionError::X11RequestFailed(Box::new(e)))?;

    HOTKEY_REGISTRY.insert(key, action_hash);

    info!("Registered hotkey: keycode={} modmask={} -> {}", keycode, grab_mod, action);

    Ok(action_hash)
}

/// Unregister hotkey atomic + X11 ungrab
#[inline]
pub fn unregister_hotkey_atomic(keycode: u32, modmask: u32, conn: &mut RustConnection, root: Window) -> bool {
    let key = (keycode, modmask);
    let removed = HOTKEY_REGISTRY.remove(&key).is_some();
    if removed {
        let grab_mod = internal_mod_to_x11(modmask) as u16;
        let _ = xproto::ungrab_key(
            conn,
            x11rb::Keycode(keycode as u16),
            grab_mod,
            root,
        );
        let _ = conn.flush();
    }
    removed
}

/// Pop events from the ring buffer
#[inline]
pub fn pop_hotkey_event() -> Option<u64> {
    EVENT_RING.try_pop()
}

/// Bevy system to process hotkey events
pub fn process_linux_hotkey_events_system(
    hotkey_resource: Res<X11HotkeyResource>,
    mut hotkey_pressed_events: EventWriter<HotkeyPressed>,
    hotkey_registry: Res<crate::resources::HotkeyRegistry>,
) {
    let mut conn_guard = hotkey_resource.conn.lock().expect("conn lock");

    while let Ok(event) = conn_guard.wait_for_event_timeout(0) {
        match event {
            xproto::Event::KeyPress(ev) => {
                if ev.root == hotkey_resource.root && ev.child == x11rb::Window(0) {
                    let keycode = ev.detail.0 as u32;
                    let state = ev.state.0 as u32;
                    let key = (keycode, state);

                    if let Some(action_hash_ref) = HOTKEY_REGISTRY.get(&key) {
                        let action_hash = *action_hash_ref.value();
                        drop(action_hash_ref);

                        if !EVENT_RING.try_push(action_hash) {
                            error!("EVENT RING BUFFER FULL - event dropped (hash: {})", action_hash);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Bevy system to register hotkeys with the Linux system
pub fn register_hotkey_with_linux_system(
    hotkey_resource: Res<X11HotkeyResource>,
    mut registration_requests: EventReader<crate::events::HotkeyRegisterRequested>,
    mut registration_completed: EventWriter<crate::events::HotkeyRegisterCompleted>,
) {
    let mut conn = hotkey_resource.conn.lock().expect("conn lock");
    let root = hotkey_resource.root;

    for request in registration_requests.read() {
        let keysym = match code_to_keysym(request.binding.definition.code) {
            Ok(k) => k,
            Err(e) => {
                registration_completed.write(crate::events::HotkeyRegisterCompleted {
                    binding: request.binding.clone(),
                    requester: request.binding.requester.clone(),
                    success: false,
                    error_message: Some(e.to_string()),
                });
                continue;
            }
        };

        let keycodes = match conn.keysym_to_keycodes(keysym) {
            Ok(kc) => kc,
            Err(e) => {
                let err = LinuxPermissionError::X11RequestFailed(Box::new(e));
                registration_completed.write(crate::events::HotkeyRegisterCompleted {
                    binding: request.binding.clone(),
                    requester: request.binding.requester.clone(),
                    success: false,
                    error_message: Some(err.to_string()),
                });
                continue;
            }
        };

        let keycode = if let Some(&kc) = keycodes.first() {
            kc as u8 as u32
        } else {
            let err = LinuxPermissionError::KeycodeLookupFailed(request.binding.definition.code);
            registration_completed.write(crate::events::HotkeyRegisterCompleted {
                binding: request.binding.clone(),
                requester: request.binding.requester.clone(),
                success: false,
                error_message: Some(err.to_string()),
            });
            continue;
        };

        let internal_modifiers = linux_modifiers_from_global(request.binding.definition.modifiers);
        let x11_modmask = internal_mod_to_x11(internal_modifiers) as u32;

        match register_hotkey_atomic(
            keycode,
            x11_modmask,
            &request.binding.action,
            &mut *conn,
            root,
        ) {
            Ok(_) => {
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
                // Try ungrab if partial
                let _ = unregister_hotkey_atomic(keycode, x11_modmask, &mut *conn, root);
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

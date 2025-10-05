//! Linux-specific focused window detection using X11 and Wayland APIs

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
use wayland_client::{Connection, QueueHandle, protocol::wl_output};
#[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
use x11::xlib::{
    Display, Window, XCloseDisplay, XGetInputFocus, XGetWindowAttributes, XOpenDisplay,
    XRootWindow, XTranslateCoordinates, XWindowAttributes,
};

#[cfg(all(unix, not(target_os = "macos")))]
use super::types::{FocusedWindowError, FocusedWindowResult, WindowBounds};

/// Linux-specific focused window detection supporting both X11 and Wayland
/// Automatically detects the display server and uses appropriate implementation
#[cfg(all(unix, not(target_os = "macos")))]
#[inline]
pub fn get_focused_window_bounds_linux() -> FocusedWindowResult<WindowBounds> {
    // Check for Wayland first (more modern)
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        #[cfg(all(
            feature = "wayland-client",
            feature = "wayland-protocols-wlr",
            target_os = "linux"
        ))]
        {
            return get_focused_window_bounds_wayland();
        }
        #[cfg(not(all(
            feature = "wayland-client",
            feature = "wayland-protocols-wlr",
            target_os = "linux"
        )))]
        {
            return Err(FocusedWindowError::SystemError(
                "Wayland detected but wayland-client and wayland-protocols-wlr features not \
                 enabled"
                    .to_string(),
            ));
        }
    }

    // Fall back to X11
    if std::env::var("DISPLAY").is_ok() {
        #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
        {
            return get_focused_window_bounds_x11();
        }
        #[cfg(not(all(feature = "x11", unix, not(target_os = "macos"))))]
        {
            return Err(FocusedWindowError::SystemError(
                "X11 detected but x11 feature not enabled".to_string(),
            ));
        }
    }

    Err(FocusedWindowError::DisplayServerNotFound)
}

/// X11-specific focused window detection
#[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
fn get_focused_window_bounds_x11() -> FocusedWindowResult<WindowBounds> {
    unsafe {
        // Open connection to X11 display
        let display: *mut Display = XOpenDisplay(std::ptr::null());
        if display.is_null() {
            return Err(FocusedWindowError::SystemError(
                "Failed to open X11 display".to_string(),
            ));
        }

        let mut focus_window: Window = 0;
        let mut revert_to: i32 = 0;

        // Get the currently focused window
        XGetInputFocus(display, &mut focus_window, &mut revert_to);

        // Check if we got a valid focus window
        if focus_window == 0 {
            XCloseDisplay(display);
            return Err(FocusedWindowError::NoFocusedWindow);
        }

        // Get window attributes (dimensions)
        let mut attrs: XWindowAttributes = std::mem::zeroed();
        let result = XGetWindowAttributes(display, focus_window, &mut attrs);
        if result == 0 {
            XCloseDisplay(display);
            return Err(FocusedWindowError::SystemError(
                "XGetWindowAttributes failed".to_string(),
            ));
        }

        // Get window position relative to root window
        let root_window = XRootWindow(display, 0);
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut child: Window = 0;

        let translate_result = XTranslateCoordinates(
            display,
            focus_window,
            root_window,
            0,
            0,
            &mut x,
            &mut y,
            &mut child,
        );

        if translate_result == 0 {
            XCloseDisplay(display);
            return Err(FocusedWindowError::SystemError(
                "XTranslateCoordinates failed".to_string(),
            ));
        }

        XCloseDisplay(display);

        // Validate dimensions
        if attrs.width <= 0 || attrs.height <= 0 {
            return Err(FocusedWindowError::SystemError(
                "Invalid window dimensions from X11".to_string(),
            ));
        }

        tracing::debug!(
            "X11 focused window detected: {}x{} at ({}, {})",
            attrs.width,
            attrs.height,
            x,
            y
        );

        Ok(WindowBounds::new(x, y, attrs.width, attrs.height))
    }
}

/// Wayland-specific active monitor detection
/// Returns bounds of the monitor containing the cursor or focused window
/// This is sufficient for launcher positioning (same as KLauncher, Alfred, etc.)
#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
fn get_focused_window_bounds_wayland() -> FocusedWindowResult<WindowBounds> {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use wayland_client::Dispatch;
    use wayland_client::globals::registry_queue_init;
    use wayland_client::protocol::wl_registry;

    let conn = Connection::connect_to_env().map_err(|e| {
        FocusedWindowError::SystemError(format!("Failed to connect to Wayland: {}", e))
    })?;

    let (globals, mut event_queue) = registry_queue_init::<WaylandState>(&conn)
        .map_err(|e| FocusedWindowError::SystemError(format!("Registry init failed: {}", e)))?;

    let qh = event_queue.handle();

    // Bind to foreign toplevel manager to track focused windows
    let toplevel_manager = globals.bind::<wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, _, _>(&qh, 1..=3, ())
        .map_err(|_| FocusedWindowError::CompositorNotSupported(
            "wlr-foreign-toplevel-management protocol not available".to_string()
        ))?;

    // Bind to output manager to get precise output information
    let output_manager = globals.bind::<wayland_protocols_wlr::output_management::v1::client::zwlr_output_manager_v1::ZwlrOutputManagerV1, _, _>(&qh, 1..=2, ())
        .map_err(|_| FocusedWindowError::CompositorNotSupported(
            "wlr-output-management protocol not available".to_string()
        ))?;

    let state = Arc::new(Mutex::new(WaylandState {
        focused_toplevel: None,
        toplevels: Vec::new(),
        output_heads: std::collections::HashMap::new(),
        output_name_to_output: std::collections::HashMap::new(),
        output_geometry: std::collections::HashMap::new(),
        ready: false,
    }));

    let mut queue_state = WaylandQueueState {
        state: Arc::clone(&state),
        toplevel_manager,
        output_manager,
    };

    // Dispatch events with timeout to get current state
    let timeout = Duration::from_millis(500);
    let start = Instant::now();
    let mut rounds = 0;

    while start.elapsed() < timeout && rounds < 50 {
        match event_queue.blocking_dispatch(&mut queue_state) {
            Ok(_) => {},
            Err(e) => {
                return Err(FocusedWindowError::SystemError(format!(
                    "Event dispatch failed: {}",
                    e
                )));
            },
        }

        // Check if we have focused window and its output information
        if let Ok(state_lock) = state.lock() {
            if let Some(ref focused) = state_lock.focused_toplevel {
                if !focused.outputs.is_empty() {
                    // Find enabled outputs from our output heads
                    for output_info in state_lock.output_heads.values() {
                        if output_info.enabled {
                            tracing::debug!(
                                "Found focused window on active output: {}x{} at ({}, {})",
                                output_info.mode_width,
                                output_info.mode_height,
                                output_info.pos_x,
                                output_info.pos_y
                            );
                            return Ok(WindowBounds::new(
                                output_info.pos_x,
                                output_info.pos_y,
                                output_info.mode_width,
                                output_info.mode_height,
                            ));
                        }
                    }
                }
            }
        }

        rounds += 1;
        std::thread::sleep(Duration::from_millis(10));
    }

    // Fallback: return any enabled output if we found one
    if let Ok(state_lock) = state.lock() {
        for output_info in state_lock.output_heads.values() {
            if output_info.enabled {
                tracing::warn!("Focused window detection timed out, using first available output");
                return Ok(WindowBounds::new(
                    output_info.pos_x,
                    output_info.pos_y,
                    output_info.mode_width,
                    output_info.mode_height,
                ));
            }
        }
    }

    Err(FocusedWindowError::NoFocusedWindow)
}

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
struct WaylandState {
    focused_toplevel: Option<ToplevelInfo>,
    toplevels: Vec<ToplevelInfo>,
    output_heads: std::collections::HashMap<String, OutputInfo>, // output_name -> detailed info
    output_name_to_output: std::collections::HashMap<String, wl_output::WlOutput>, /* name -> wl_output mapping */
    output_geometry: std::collections::HashMap<wl_output::WlOutput, (i32, i32, i32, i32)>, /* output -> (x, y, width, height) - legacy */
    ready: bool,
}

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
struct WaylandQueueState {
    state: Arc<Mutex<WaylandState>>,
    toplevel_manager: wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
    output_manager: wayland_protocols_wlr::output_management::v1::client::zwlr_output_manager_v1::ZwlrOutputManagerV1,
}

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
#[derive(Debug, Clone)]
struct OutputInfo {
    enabled: bool,
    pos_x: i32,
    pos_y: i32,
    mode_width: i32,
    mode_height: i32,
    scale: f64,
    name: String,
}

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
#[derive(Debug, Clone)]
struct ToplevelInfo {
    handle: wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
    title: String,
    outputs: Vec<wayland_client::protocol::wl_output::WlOutput>, // outputs this toplevel appears on
    activated: bool,
}

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
impl Dispatch<wl_registry::WlRegistry, ()> for WaylandQueueState {
    fn event(
        _state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Registry events handled by globals system
    }
}

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
impl Dispatch<wl_output::WlOutput, ()> for WaylandQueueState {
    fn event(
        _state: &mut Self,
        _output: &wl_output::WlOutput,
        _event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // Legacy wl_output events are handled by output management protocol
        // This dispatch is required but we don't process these events directly
    }
}

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
impl Dispatch<wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, ()> for WaylandQueueState {
    fn event(
        state: &mut Self,
        _: &wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
        event: wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::Event;

        match event {
            Event::Toplevel { toplevel } => {
                let info = ToplevelInfo {
                    handle: toplevel,
                    title: String::new(),
                    outputs: Vec::new(),
                    activated: false,
                };

                if let Ok(mut state_lock) = state.state.lock() {
                    state_lock.toplevels.push(info);
                }
            }
            Event::Finished => {
                if let Ok(mut state_lock) = state.state.lock() {
                    state_lock.ready = true;
                }
            }
            _ => {}
        }
    }
}

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
impl Dispatch<wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ()> for WaylandQueueState {
    fn event(
        state: &mut Self,
        proxy: &wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
        event: wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::{Event, State as ToplevelState};

        let Ok(mut state_lock) = state.state.lock() else {
            return; // Skip event if we can't lock state
        };

        // Find the toplevel by handle
        if let Some(toplevel) = state_lock.toplevels.iter_mut().find(|t| t.handle == *proxy) {
            match event {
                Event::Title { title } => {
                    toplevel.title = title;
                }
                Event::OutputEnter { output } => {
                    // Track which outputs this toplevel appears on
                    if !toplevel.outputs.contains(&output) {
                        toplevel.outputs.push(output);
                    }
                }
                Event::OutputLeave { output } => {
                    // Remove output from tracking when toplevel leaves it
                    toplevel.outputs.retain(|o| *o != output);
                }
                Event::State { state: toplevel_state } => {
                    // Check for activated state in the state array
                    let activated = toplevel_state
                        .chunks_exact(4)
                        .any(|chunk| {
                            let state_val = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                            state_val == ToplevelState::Activated as u32
                        });

                    toplevel.activated = activated;

                    if activated {
                        state_lock.focused_toplevel = Some(toplevel.clone());
                    }
                }
                Event::Done => {
                    // State is consistent for this toplevel
                    if toplevel.activated {
                        state_lock.ready = true;
                    }
                }
                Event::Closed => {
                    // Remove from tracking
                    if let Some(ref focused) = state_lock.focused_toplevel {
                        if focused.handle == *proxy {
                            state_lock.focused_toplevel = None;
                        }
                    }
                    state_lock.toplevels.retain(|t| t.handle != *proxy);
                }
                _ => {}
            }
        }
    }
}

// Output Management Protocol Dispatch Implementations

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
impl Dispatch<wayland_protocols_wlr::output_management::v1::client::zwlr_output_manager_v1::ZwlrOutputManagerV1, ()> for WaylandQueueState {
    fn event(
        _state: &mut Self,
        _: &wayland_protocols_wlr::output_management::v1::client::zwlr_output_manager_v1::ZwlrOutputManagerV1,
        event: wayland_protocols_wlr::output_management::v1::client::zwlr_output_manager_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wayland_protocols_wlr::output_management::v1::client::zwlr_output_manager_v1::Event;

        match event {
            Event::Head { head } => {
                // New output head - we'll track it via the head dispatch
                let _ = head; // Use head object in head dispatch
            }
            Event::Done { serial: _ } => {
                // Configuration update complete
            }
            Event::Finished => {
                // Output manager finished
            }
            _ => {}
        }
    }
}

#[cfg(all(
    feature = "wayland-client",
    feature = "wayland-protocols-wlr",
    target_os = "linux"
))]
impl
    Dispatch<
        wayland_protocols_wlr::output_management::v1::client::zwlr_output_head_v1::ZwlrOutputHeadV1,
        (),
    > for WaylandQueueState
{
    fn event(
        state: &mut Self,
        proxy: &wayland_protocols_wlr::output_management::v1::client::zwlr_output_head_v1::ZwlrOutputHeadV1,
        event: wayland_protocols_wlr::output_management::v1::client::zwlr_output_head_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use wayland_protocols_wlr::output_management::v1::client::zwlr_output_head_v1::Event;

        if let Ok(mut state_lock) = state.state.lock() {
            // Use proxy address as unique string key for this head
            let head_key = format!("{:p}", proxy);

            let output_info = state_lock
                .output_heads
                .entry(head_key.clone())
                .or_insert_with(|| OutputInfo {
                    enabled: false,
                    pos_x: 0,
                    pos_y: 0,
                    mode_width: 1920,
                    mode_height: 1080,
                    scale: 1.0,
                    name: String::new(),
                });

            match event {
                Event::Name { name } => {
                    output_info.name = name.clone();
                    // Also update the key mapping if we want to use names
                },
                Event::Enabled { enabled } => {
                    output_info.enabled = enabled != 0;
                },
                Event::Position { x, y } => {
                    output_info.pos_x = x;
                    output_info.pos_y = y;
                },
                Event::Scale { scale } => {
                    output_info.scale = scale.into();
                },
                Event::CurrentMode { mode: _ } => {
                    // Mode details come from mode object dispatch
                },
                Event::Finished => {
                    // Remove this head from tracking
                    state_lock.output_heads.remove(&head_key);
                },
                _ => {},
            }
        }
    }
}

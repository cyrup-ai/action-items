//! Unix/Linux platform configurations (Wayland, X11, XCB)

#[cfg(all(feature = "wayland-client", target_os = "linux"))]
use wayland_client::{Connection, Dispatch, QueueHandle};
#[cfg(all(feature = "wayland-protocols-wlr", target_os = "linux"))]
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1},
};
#[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
use x11::xlib;
#[cfg(all(feature = "xcb", unix, not(target_os = "macos")))]
use xcb;

use super::types::OverlayError;

#[cfg(all(unix, not(target_os = "macos")))]
#[allow(dead_code)] // Platform-specific function - only used on Unix systems
pub fn configure_unix_overlay(winit_window: &winit::window::Window) -> Result<(), OverlayError> {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

    let display_handle = winit_window
        .display_handle()
        .map_err(|_| OverlayError::HandleAccess)?;

    match display_handle.as_raw() {
        #[cfg(all(feature = "wayland-client", target_os = "linux"))]
        RawDisplayHandle::Wayland(wayland_display) => {
            configure_wayland_layer_shell(winit_window, wayland_display)
        },
        #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
        RawDisplayHandle::Xlib(xlib_display) => {
            configure_x11_override_redirect(winit_window, xlib_display)
        },
        #[cfg(all(feature = "xcb", unix, not(target_os = "macos")))]
        RawDisplayHandle::Xcb(xcb_display) => {
            configure_xcb_override_redirect(winit_window, xcb_display)
        },
        _ => {
            tracing::info!("Unsupported display server, using default window behavior");
            Ok(())
        },
    }
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
#[allow(dead_code)] // Platform-specific function - only used on Unix systems
pub fn configure_unix_overlay(_winit_window: &winit::window::Window) -> Result<(), OverlayError> {
    tracing::warn!("Unix overlay configuration not available on this platform");
    Err(OverlayError::PlatformMismatch)
}

#[cfg(all(feature = "wayland-client", target_os = "linux"))]
fn configure_wayland_layer_shell(
    winit_window: &winit::window::Window,
    wayland_display: &raw_window_handle::WaylandDisplayHandle,
) -> Result<(), OverlayError> {
    use std::os::fd::AsFd;

    use wayland_client::protocol::wl_output::WlOutput;
    use wayland_client::protocol::wl_surface::WlSurface;

    let window_handle = winit_window
        .window_handle()
        .map_err(|_| OverlayError::HandleAccess)?;

    if let RawWindowHandle::Wayland(wayland_window) = window_handle.as_raw() {
        // Connect to Wayland display
        let display_fd = wayland_display.display.as_fd();
        let connection = Connection::from_fd(display_fd.as_fd()).map_err(|e| {
            OverlayError::wayland_connection(format!("Failed to connect to Wayland display: {}", e))
        })?;

        let mut event_queue = connection.new_event_queue();
        let qh = event_queue.handle();

        // Get globals
        let display = connection.display();
        display.get_registry(&qh, ());

        // Roundtrip to get globals
        let mut state = LayerShellState::new();
        event_queue.roundtrip(&mut state).map_err(|e| {
            OverlayError::wayland_protocol(
                "wl_registry",
                format!("Registry roundtrip failed: {}", e),
            )
        })?;

        // Configure layer surface if layer shell is available
        if let Some(layer_shell) = &state.layer_shell {
            // Create ObjectId from raw surface pointer with proper error handling
            use wayland_client::backend::ObjectId;
            use wayland_client::protocol::wl_surface;

            let surface_ptr = wayland_window.surface.as_ptr() as *mut wayland_client::sys::wl_proxy;
            let surface_id = unsafe {
                ObjectId::from_ptr(&wl_surface::WlSurface::interface(), surface_ptr).map_err(
                    |e| {
                        OverlayError::wayland_connection(format!(
                            "Failed to create surface ObjectId: {}",
                            e
                        ))
                    },
                )?
            };

            let surface = WlSurface::from_id(&connection, surface_id).map_err(|e| {
                OverlayError::wayland_connection(format!(
                    "Failed to create WlSurface from ObjectId: {}",
                    e
                ))
            })?;

            let layer_surface = layer_shell.get_layer_surface(
                &surface,
                None, // Let compositor choose output
                Layer::Overlay,
                "action-items-launcher",
            );

            // Configure for launcher overlay behavior with error checking
            layer_surface.set_anchor(Anchor::Top);
            layer_surface.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
            layer_surface.set_exclusive_zone(0);
            layer_surface.set_margin(50, 0, 0, 0);
            layer_surface.set_size(600, 420);

            surface.commit();

            tracing::info!("✅ Configured Wayland layer shell overlay");
        } else {
            return Err(OverlayError::wayland_protocol(
                "zwlr_layer_shell_v1",
                "Layer shell protocol not available",
            ));
        }

        Ok(())
    } else {
        Err(OverlayError::PlatformMismatch)
    }
}

#[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
fn configure_x11_override_redirect(
    winit_window: &winit::window::Window,
    xlib_display: &raw_window_handle::XlibDisplayHandle,
) -> Result<(), OverlayError> {
    let window_handle = winit_window
        .window_handle()
        .map_err(|_| OverlayError::HandleAccess)?;

    if let RawWindowHandle::Xlib(xlib_window) = window_handle.as_raw() {
        unsafe {
            let display = xlib_display.display as *mut xlib::Display;
            let window = xlib_window.window as xlib::Window;

            // Set override_redirect to bypass window manager
            let mut attributes = xlib::XSetWindowAttributes {
                override_redirect: 1, // True
                ..std::mem::zeroed()
            };

            let result = xlib::XChangeWindowAttributes(
                display,
                window,
                xlib::CWOverrideRedirect,
                &mut attributes,
            );

            // Validate X11 operation success (Success = 0, errors are non-zero)
            if result != 0 {
                return Err(OverlayError::x11_configure_failed(format!(
                    "XChangeWindowAttributes failed with code: {}",
                    result
                )));
            }

            tracing::info!("✅ Configured X11 override_redirect overlay");
        }
        Ok(())
    } else {
        Err(OverlayError::PlatformMismatch)
    }
}

#[cfg(all(feature = "xcb", unix, not(target_os = "macos")))]
fn configure_xcb_override_redirect(
    winit_window: &winit::window::Window,
    xcb_display: &raw_window_handle::XcbDisplayHandle,
) -> Result<(), OverlayError> {
    let window_handle = winit_window
        .window_handle()
        .map_err(|_| OverlayError::HandleAccess)?;

    if let RawWindowHandle::Xcb(xcb_window) = window_handle.as_raw() {
        unsafe {
            let connection_ptr = xcb_display.connection as *mut xcb::ffi::xcb_connection_t;
            let connection = xcb::Connection::from_raw_conn(connection_ptr);
            let window = xcb_window.window.get() as xcb::Window;

            // Configure window attributes for override_redirect
            let values = [(xcb::CW_OVERRIDE_REDIRECT, 1u32)];

            // Use checked request to validate XCB operation success
            let cookie = xcb::change_window_attributes_checked(&connection, window, &values);
            if let Err(e) = connection.check_request(cookie) {
                return Err(OverlayError::xcb_configure_failed(format!(
                    "change_window_attributes failed: {:?}",
                    e
                )));
            }
            connection.flush();

            tracing::info!("✅ Configured XCB override_redirect overlay");
        }
        Ok(())
    } else {
        Err(OverlayError::PlatformMismatch)
    }
}

#[cfg(all(feature = "wayland-client", target_os = "linux"))]
pub struct LayerShellState {
    pub layer_shell: Option<ZwlrLayerShellV1>,
}

#[cfg(all(feature = "wayland-client", target_os = "linux"))]
impl LayerShellState {
    pub fn new() -> Self {
        Self { layer_shell: None }
    }
}

#[cfg(all(feature = "wayland-client", target_os = "linux"))]
impl Dispatch<wayland_client::protocol::wl_registry::WlRegistry, ()> for LayerShellState {
    fn event(
        state: &mut Self,
        registry: &wayland_client::protocol::wl_registry::WlRegistry,
        event: wayland_client::protocol::wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_registry::Event;

        if let Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "zwlr_layer_shell_v1" {
                let layer_shell = registry.bind::<ZwlrLayerShellV1, _, _>(name, version, qh, ());
                state.layer_shell = Some(layer_shell);
            }
        }
    }
}

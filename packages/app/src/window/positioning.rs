//! Professional multi-monitor positioning system for Action Items launcher
//!
//! Implements Bevy Monitor component pattern for blazing-fast monitor detection
//! and intelligent launcher positioning with zero allocation, production-quality code.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::window::Monitor;
use tracing::{info, warn};

use crate::window::errors::ScreenDimensionError;
use crate::window::state::ViewportState;

/// Monitor to camera mapping for multi-monitor support
#[derive(Resource, Default, Debug)]
pub struct MonitorCameraRegistry {
    /// Monitor entity -> Camera entity mapping
    pub monitor_cameras: HashMap<Entity, Entity>,
}

/// Screen dimensions resource for viewport-responsive UI calculations
/// Zero-allocation screen dimension tracking with blazing-fast access patterns
#[derive(Resource, Debug, Clone, Copy)]
pub struct ScreenDimensions {
    /// Physical width of active screen in pixels
    pub width: u32,
    /// Physical height of active screen in pixels  
    pub height: u32,
    /// Scale factor for high-DPI displays
    pub scale_factor: f64,
    /// Logical width accounting for DPI scaling
    pub logical_width: f32,
    /// Logical height accounting for DPI scaling
    pub logical_height: f32,
}

impl Default for ScreenDimensions {
    #[inline]
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
            logical_width: 1920.0,
            logical_height: 1080.0,
        }
    }
}

impl ScreenDimensions {
    /// Calculate viewport percentage as pixel value
    /// Optimized for zero allocation with inline calculations
    #[inline]
    pub fn vw_to_pixels(&self, vw_percent: f32) -> f32 {
        (self.logical_width * vw_percent) / 100.0
    }

    /// Calculate viewport height percentage as pixel value  
    /// Optimized for zero allocation with inline calculations
    #[inline]
    pub fn vh_to_pixels(&self, vh_percent: f32) -> f32 {
        (self.logical_height * vh_percent) / 100.0
    }

    /// Calculate aspect ratio of the screen
    /// Returns logical width divided by logical height
    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        if self.logical_height > 0.0 {
            self.logical_width / self.logical_height
        } else {
            1.0 // Default to square aspect ratio if height is invalid
        }
    }
}

/// Calculate responsive window size based on viewport percentages
/// Uses viewport-to-pixel conversion methods for precision sizing
/// Applies aspect ratio constraints for optimal layout
#[inline]
pub fn calculate_responsive_window_size(
    screen_dims: &ScreenDimensions,
    width_vw: f32,
    height_vh: f32,
) -> (f32, f32) {
    let width_pixels = screen_dims.vw_to_pixels(width_vw);
    let height_pixels = screen_dims.vh_to_pixels(height_vh);

    // Use aspect ratio to ensure reasonable window proportions
    let aspect_ratio = screen_dims.aspect_ratio();

    // For very wide screens (>2.0 aspect ratio), limit window width
    // For very tall screens (<0.8 aspect ratio), limit window height
    let (adjusted_width, adjusted_height) = if aspect_ratio > 2.0 {
        // Ultra-wide screen: constrain width to maintain usable proportions
        let max_reasonable_width = screen_dims.logical_height * 1.6; // 16:10 ratio
        (width_pixels.min(max_reasonable_width), height_pixels)
    } else if aspect_ratio < 0.8 {
        // Very tall screen: constrain height to maintain usable proportions
        let max_reasonable_height = screen_dims.logical_width * 1.25; // 4:5 ratio
        (width_pixels, height_pixels.min(max_reasonable_height))
    } else {
        // Normal aspect ratio: use calculated dimensions
        (width_pixels, height_pixels)
    };

    (adjusted_width, adjusted_height)
}

/// Get active screen dimensions with comprehensive error handling
/// Uses Bevy Monitor component queries for cross-platform screen detection
/// Returns current screen dimensions or safe fallback values
#[inline]
pub fn get_active_screen_dimensions(
    active_monitor: &ActiveMonitor,
    monitors_query: &Query<(Entity, &Monitor)>,
) -> Result<ScreenDimensions, ScreenDimensionError> {
    // Get target monitor or fallback to primary
    let target_entity = active_monitor
        .target
        .or(active_monitor.primary)
        .ok_or(ScreenDimensionError::ActiveMonitorNotFound)?;

    // Query monitor component for screen dimensions
    let (_, monitor) = monitors_query
        .get(target_entity)
        .map_err(|_| ScreenDimensionError::InvalidMonitorEntity)?;

    // Validate monitor dimensions
    if monitor.physical_width == 0 || monitor.physical_height == 0 {
        return Err(ScreenDimensionError::InvalidDimensions {
            width: monitor.physical_width,
            height: monitor.physical_height,
        });
    }

    // Calculate logical dimensions accounting for DPI scaling
    let logical_width = monitor.physical_width as f32 / monitor.scale_factor as f32;
    let logical_height = monitor.physical_height as f32 / monitor.scale_factor as f32;

    Ok(ScreenDimensions {
        width: monitor.physical_width,
        height: monitor.physical_height,
        scale_factor: monitor.scale_factor,
        logical_width,
        logical_height,
    })
}

/// Active monitor tracking resource following Bevy patterns
/// Stores current monitor entity for fastest launcher positioning
#[derive(Resource, Debug, Default)]
pub struct ActiveMonitor {
    /// Entity of the monitor where launcher should appear
    pub target: Option<Entity>,
    /// Cache for fastest primary monitor lookup
    pub primary: Option<Entity>,
}

impl ActiveMonitor {
    // Methods removed - positioning now handled by consolidated window management system
}

/// Zero-allocation monitor detection system following monitor_info.rs pattern
/// Uses direct Bevy Monitor component queries for blazing-fast performance
/// Automatically updates ScreenDimensions and ViewportState resources on monitor configuration
/// changes
#[inline]
pub fn detect_monitors_system(
    monitors_added: Query<(Entity, &Monitor), Added<Monitor>>,
    mut monitors_removed: RemovedComponents<Monitor>,
    mut active_monitor: ResMut<ActiveMonitor>,
    mut screen_dimensions: ResMut<ScreenDimensions>,
    mut viewport_state: ResMut<ViewportState>,
    monitors_query: Query<(Entity, &Monitor)>,
) {
    // Handle newly connected monitors using exact Bevy pattern
    for (entity, monitor) in monitors_added.iter() {
        // Set primary monitor if this is the first one detected
        if active_monitor.primary.is_none() {
            active_monitor.primary = Some(entity);
            active_monitor.target = Some(entity);

            let name = monitor.name.as_deref().unwrap_or("Unknown Display");
            info!(
                "Primary monitor detected: {} ({}x{} @{:.1}x scale)",
                name, monitor.physical_width, monitor.physical_height, monitor.scale_factor
            );
        }

        let monitor_name = monitor.name.as_deref().unwrap_or("Unknown");
        info!(
            "Monitor connected: {} at position ({}, {})",
            monitor_name, monitor.physical_position.x, monitor.physical_position.y
        );
    }

    // Handle disconnected monitors using exact Bevy RemovedComponents pattern
    for removed_entity in monitors_removed.read() {
        // Update primary monitor if it was removed
        if active_monitor.primary == Some(removed_entity) {
            active_monitor.primary = None;
            active_monitor.target = None;
            warn!("Primary monitor disconnected, will need re-detection");
        }

        // Update target monitor if it was removed
        if active_monitor.target == Some(removed_entity) {
            active_monitor.target = active_monitor.primary;
        }

        info!("Monitor disconnected: {:?}", removed_entity);
    }

    // Update ScreenDimensions and ViewportState resources based on current active monitor
    // Zero-allocation update with optimized error handling
    if let Ok(dimensions) = get_active_screen_dimensions(&active_monitor, &monitors_query) {
        *screen_dimensions = dimensions;
        viewport_state.update_from_screen_dimensions(&dimensions);

        info!(
            "Screen dimensions updated: {}x{} (logical: {:.0}x{:.0}) @{:.1}x scale",
            dimensions.width,
            dimensions.height,
            dimensions.logical_width,
            dimensions.logical_height,
            dimensions.scale_factor
        );

        info!(
            "Viewport conversion ratios: width={:.6}, height={:.6}",
            viewport_state.width_ratio, viewport_state.height_ratio
        );
    }
}

/// Calculate proportional launcher size based on monitor dimensions
/// Returns (width, height) as percentage of monitor: 45% width, 35% height
#[inline]
pub fn calculate_launcher_size(monitor: &Monitor) -> (f32, f32) {
    let monitor_width = monitor.physical_width as f32;
    let monitor_height = monitor.physical_height as f32;

    // Calculate as percentage of monitor size - no hardcoded values
    let width = monitor_width * 0.45; // 45% of screen width
    let height = monitor_height * 0.35; // 35% of screen height

    // Ensure minimum usable size while staying proportional
    let min_width = monitor_width * 0.25; // Minimum 25% width
    let max_width = monitor_width * 0.60; // Maximum 60% width
    let min_height = monitor_height * 0.20; // Minimum 20% height
    let max_height = monitor_height * 0.50; // Maximum 50% height

    (
        width.clamp(min_width, max_width),
        height.clamp(min_height, max_height),
    )
}

/// REMOVED: Professional launcher positioning system
/// This function was causing a focus loop by duplicating window management
/// with events/handlers.rs. LauncherEvent::Show is now handled exclusively
/// in events/handlers.rs to prevent window management conflicts.
///
/// The ActiveMonitor::get_launcher_position() method is still used by
/// the consolidated handler in events/handlers.rs for proper positioning.
/// Professional active application monitor detection system
/// Uses proper focused window detection via platform-specific APIs
#[inline]
pub fn track_active_application_monitor_system(
    active_monitor: ResMut<ActiveMonitor>,
    monitors_query: Query<(Entity, &Monitor)>,
) {
    // Delegate to the professional focused window detection system
    crate::window::focused_window::system_integration::detect_focused_window_monitor_system(
        active_monitor,
        monitors_query,
    );
}

/// Debug system for monitoring multi-monitor configuration  
/// Provides real-time monitor information for development
#[inline]
pub fn debug_monitor_info_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    active_monitor: Res<ActiveMonitor>,
    monitors_query: Query<(Entity, &Monitor)>,
) {
    if keyboard.just_pressed(KeyCode::F12) {
        info!("=== Multi-Monitor Debug Info (Bevy Pattern) ===");
        info!("Total monitors detected: {}", monitors_query.iter().count());

        for (entity, monitor) in monitors_query.iter() {
            let name = monitor.name.as_deref().unwrap_or("Unknown Display");
            let is_primary = active_monitor.primary == Some(entity);
            let is_target = active_monitor.target == Some(entity);

            info!("Monitor: {} (Entity: {:?})", name, entity);
            info!(
                "  Position: ({}, {})",
                monitor.physical_position.x, monitor.physical_position.y
            );
            info!(
                "  Size: {}x{}",
                monitor.physical_width, monitor.physical_height
            );
            info!("  Scale: {:.2}x", monitor.scale_factor);
            info!("  Primary: {} | Target: {}", is_primary, is_target);

            let (calc_width, calc_height) = calculate_launcher_size(monitor);
            info!(
                "  Launcher size: {:.0}x{:.0} ({:.1}% x {:.1}%)",
                calc_width,
                calc_height,
                (calc_width / monitor.physical_width as f32) * 100.0,
                (calc_height / monitor.physical_height as f32) * 100.0
            );
        }

        match (active_monitor.primary, active_monitor.target) {
            (Some(primary), Some(target)) => {
                info!(
                    "Active configuration: Primary={:?}, Target={:?}",
                    primary, target
                );
            },
            (Some(primary), None) => {
                info!("Active configuration: Primary={:?}, No target set", primary);
            },
            _ => {
                warn!("No active monitor configuration - awaiting detection");
            },
        }
    }
}

/// System to create cameras for each monitor
#[inline]
#[allow(dead_code)]
pub fn setup_monitor_cameras(
    mut commands: Commands,
    monitors: Query<Entity, With<Monitor>>,
    mut camera_registry: ResMut<MonitorCameraRegistry>,
) {
    for monitor_entity in monitors.iter() {
        // Create camera for each monitor if not exists
        camera_registry
            .monitor_cameras
            .entry(monitor_entity)
            .or_insert_with(|| {
                let camera = commands
                    .spawn((
                        Camera2d,
                        Name::new(format!("Monitor Camera {:?}", monitor_entity)),
                    ))
                    .id();

                info!(
                    "Created camera {:?} for monitor {:?}",
                    camera, monitor_entity
                );
                camera
            });
    }
}

/// Camera targeting system for multi-monitor UI rendering
/// Follows multiple_windows.rs:59 pattern exactly
#[inline]
pub fn setup_ui_target_camera(
    mut commands: Commands,
    monitors: Query<(Entity, &Monitor)>,
    active_monitor: Res<ActiveMonitor>,
    ui_root_query: Query<Entity, With<action_items_ui::prelude::UiRoot>>,
    camera_registry: Res<MonitorCameraRegistry>,
) {
    if let Ok(ui_root) = ui_root_query.single() {
        // Find camera for target monitor
        if let Some(target_monitor) = active_monitor.target.or(active_monitor.primary) {
            // Validate that the target monitor still exists
            if let Ok((_, monitor)) = monitors.get(target_monitor) {
                // Use per-monitor camera from registry
                if let Some(camera_entity) = camera_registry.monitor_cameras.get(&target_monitor) {
                    let monitor_name = monitor.name.as_deref().unwrap_or("Unknown");
                    info!(
                        "Setting UiTargetCamera for monitor: {} ({:?}) -> camera {:?}",
                        monitor_name, target_monitor, camera_entity
                    );

                    // Add UiTargetCamera component following multiple_windows.rs:59 pattern
                    commands
                        .entity(ui_root)
                        .insert(UiTargetCamera(*camera_entity));
                } else {
                    warn!(
                        "No camera found for target monitor {:?}, camera setup may be pending",
                        target_monitor
                    );
                }
            } else {
                warn!(
                    "Target monitor {:?} no longer exists, UI targeting may be incorrect",
                    target_monitor
                );
            }
        }
    }
}

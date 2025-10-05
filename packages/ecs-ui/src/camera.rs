//! Camera integration systems for UI layout dimensions in the Lunex UI system

use crate::{Dimension, UiFetchFromCamera, UiLayoutRoot, UiSourceCamera, *};

/// This system takes [`Camera`] viewport data and pipes them into querried [`Dimension`] +
/// [`UiLayoutRoot`] + [`UiFetchFromCamera`].
pub fn system_fetch_dimension_from_camera<const INDEX: usize>(
    src_query: Query<
        (&Camera, Option<&Projection>),
        (With<UiSourceCamera<INDEX>>, Changed<Camera>),
    >,
    mut dst_query: Query<&mut Dimension, (With<UiLayoutRoot>, With<UiFetchFromCamera<INDEX>>)>,
) {
    // Check if we have a camera dimension input
    if src_query.is_empty() {
        return;
    }
    let Ok((camera, projection_option)) = src_query.single() else {
        warn_once!(
            "Multiple UiSourceCamera<{INDEX}> exist at once! Ignoring all camera inputs to avoid \
             unexpected behavior!"
        );
        return;
    };

    // Pipe the camera viewport size
    if let Some(cam_size) = camera.logical_viewport_size() {
        for mut size in &mut dst_query {
            **size = Vec2::from((cam_size.x, cam_size.y))
                * if let Some(Projection::Orthographic(p)) = projection_option {
                    p.scale
                } else {
                    1.0
                };
        }
    }
}

/// This system listens for added [`UiFetchFromCamera`] components and if it finds one, mutable
/// accesses all [`Camera`]s to trigger fetching systems.
pub fn system_touch_camera_if_fetch_added<const INDEX: usize>(
    query: Query<Entity, Added<UiFetchFromCamera<INDEX>>>,
    mut cameras: Query<&mut Camera, With<UiSourceCamera<INDEX>>>,
) {
    if !query.is_empty() {
        for mut camera in &mut cameras {
            camera.as_mut();
        }
    }
}

use bevy::math::FloatExt;
use bevy::picking::PickSet;
use bevy::window::PrimaryWindow;
use thiserror::Error;

use crate::*;

/// UI interaction errors
#[derive(Error, Debug)]
pub enum UiError {
    #[error("Viewport not found for camera")]
    ViewportNotFound,
    #[error("Camera projection error: {0}")]
    CameraProjection(String),
    #[error("Transform calculation failed: {0}")]
    TransformError(String),
}

// Cache for picking system
#[derive(Resource, Default)]
struct PickingCache {
    sorted_nodes: Vec<(Entity, Dimension, GlobalTransform, Option<Pickable>)>,
    dirty: bool,
}

// #===============#
// #=== BACKEND ===#

/// Adds picking support for Lunex.
#[derive(Clone)]
pub struct UiLunexPickingPlugin;
impl Plugin for UiLunexPickingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PickingCache>().add_systems(
            PreUpdate,
            (
                system_update_picking_cache,
                system_mark_picking_cache_dirty,
                lunex_2d_picking,
            )
                .chain()
                .in_set(PickSet::Backend),
        );
    }
}

/// This component disables the Lunex picking backend for this entity.
/// Use this only if you want to use a different or custom picking
/// bakckend. To disable picking entirely, use [`Pickable::IGNORE`].
#[derive(Component)]
pub struct NoLunexPicking;

/// System to update the picking cache when dirty
fn system_update_picking_cache(
    mut cache: ResMut<PickingCache>,
    query: Query<
        (
            Entity,
            &Dimension,
            &GlobalTransform,
            Option<&Pickable>,
            &ViewVisibility,
        ),
        Without<NoLunexPicking>,
    >,
) {
    if cache.dirty {
        cache.sorted_nodes = query
            .iter()
            .filter_map(|(entity, dimension, transform, pickable, vis)| {
                if !transform.affine().is_nan() && vis.get() {
                    Some((entity, dimension.clone(), *transform, pickable.cloned()))
                } else {
                    None
                }
            })
            .collect();

        // radsort is a stable radix sort that performed better than `slice::sort_by_key`
        radsort::sort_by_key(&mut cache.sorted_nodes, |(_, _, transform, _)| {
            -transform.translation().z
        });

        cache.dirty = false;
    }
}

/// System to mark picking cache as dirty when relevant components change
fn system_mark_picking_cache_dirty(
    mut cache: ResMut<PickingCache>,
    query: Query<
        Entity,
        Or<(
            Changed<Dimension>,
            Changed<GlobalTransform>,
            Changed<ViewVisibility>,
        )>,
    >,
) {
    if !query.is_empty() {
        cache.dirty = true;
    }
}

/// Checks if any Dimension entities are under a pointer
fn lunex_2d_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &Projection)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    cache: Res<PickingCache>,
    mut output: EventWriter<PointerHits>,
) {
    let primary_window = primary_window.single().ok();

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let mut blocked = false;
        let Some((cam_entity, camera, cam_transform, Projection::Orthographic(cam_ortho))) =
            cameras
                .iter()
                .filter(|(_, camera, ..)| camera.is_active)
                .find(|(_, camera, ..)| {
                    camera
                        .target
                        .normalize(primary_window)
                        .is_some_and(|x| x == location.target)
                })
        else {
            continue;
        };

        let viewport_pos = camera
            .logical_viewport_rect()
            .map(|v| v.min)
            .unwrap_or_else(|| {
                tracing::warn!("Camera viewport not found, using default position");
                Vec2::ZERO
            });
        let pos_in_viewport = location.position - viewport_pos;

        let Ok(cursor_ray_world) = camera.viewport_to_world(cam_transform, pos_in_viewport) else {
            continue;
        };
        let cursor_ray_len = cam_ortho.far - cam_ortho.near;
        let cursor_ray_end = cursor_ray_world.origin + cursor_ray_world.direction * cursor_ray_len;

        let picks: Vec<(Entity, HitData)> = cache
            .sorted_nodes
            .iter()
            .filter_map(|(entity, dimension, node_transform, pickable)| {
                if blocked {
                    return None;
                }

                // Transform cursor line segment to node coordinate system
                let world_to_node = node_transform.affine().inverse();
                let cursor_start_node = world_to_node.transform_point3(cursor_ray_world.origin);
                let cursor_end_node = world_to_node.transform_point3(cursor_ray_end);

                // Find where the cursor segment intersects the plane Z=0 (which is the node's
                // plane in node-local space). It may not intersect if, for example, we're
                // viewing the node side-on
                if cursor_start_node.z == cursor_end_node.z {
                    // Cursor ray is parallel to the node and misses it
                    return None;
                }
                let lerp_factor = f32::inverse_lerp(cursor_start_node.z, cursor_end_node.z, 0.0);
                if !(0.0..=1.0).contains(&lerp_factor) {
                    // Lerp factor is out of range, meaning that while an infinite line cast by
                    // the cursor would intersect the node, the node is not between the
                    // camera's near and far planes
                    return None;
                }
                // Otherwise we can interpolate the xy of the start and end positions by the
                // lerp factor to get the cursor position in node space!
                let cursor_pos_sprite = cursor_start_node.lerp(cursor_end_node, lerp_factor).xy();

                let rect = Rect::from_center_size(Vec2::ZERO, dimension.0);
                let is_cursor_in_sprite = rect.contains(cursor_pos_sprite);

                blocked = is_cursor_in_sprite
                    && pickable
                        .as_ref()
                        .map(|p| p.should_block_lower)
                        .unwrap_or(true);

                is_cursor_in_sprite.then(|| {
                    let hit_pos_world =
                        node_transform.transform_point(cursor_pos_sprite.extend(0.0));
                    // Transform point from world to camera space to get the Z distance
                    let hit_pos_cam = cam_transform
                        .affine()
                        .inverse()
                        .transform_point3(hit_pos_world);
                    // HitData requires a depth as calculated from the camera's near clipping plane
                    let depth = -cam_ortho.near - hit_pos_cam.z;
                    (
                        *entity,
                        HitData::new(
                            cam_entity,
                            depth,
                            Some(hit_pos_world),
                            Some(*node_transform.back()),
                        ),
                    )
                })
            })
            .collect();

        let order = camera.order as f32;
        output.write(PointerHits::new(*pointer, picks, order));
    }
}

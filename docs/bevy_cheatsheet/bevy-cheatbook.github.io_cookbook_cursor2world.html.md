---
url: "https://bevy-cheatbook.github.io/cookbook/cursor2world.html"
title: "Convert cursor to world coordinates - Unofficial Bevy Cheat Book"
---

- Auto
- Light
- Rust
- Coal
- Navy
- Ayu

# Unofficial Bevy Cheat Book

[Print this book](https://bevy-cheatbook.github.io/print.html "Print this book")[Suggest an edit](https://github.com/bevy-cheatbook/bevy-cheatbook/issues/new "Suggest an edit")

| [Bevy Version:](https://bevy-cheatbook.github.io/introduction.html#maintenance-policy) | [0.13](https://bevyengine.org/news/bevy-0-13) | (outdated!) |
| --- | --- | --- |

As this page is outdated, please refer to Bevy's official migration guides while reading,
to cover the differences:
[0.13 to 0.14](https://bevyengine.org/learn/migration-guides/0-13-to-0-14/),
[0.14 to 0.15](https://bevyengine.org/learn/migration-guides/0-14-to-0-15/),
[0.15 to 0.16](https://bevyengine.org/learn/migration-guides/0-15-to-0-16/).

I apologize for the inconvenience. I will update the page as soon as I find the time.

* * *

If you only have one window (the primary window), as is the case for most apps
and games, you can do this:

`Code (simple version):`

```rust no_run noplayground hljs
use bevy::window::PrimaryWindow;

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    // Make sure to add the marker component when you set up your camera
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn my_cursor_system(
    mut mycoords: ResMut<MyWorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
        eprintln!("World coords: {}/{}", world_position.x, world_position.y);
    }
}
```

```rust no_run noplayground hljs
app.init_resource::<MyWorldCoords>();
app.add_systems(Startup, setup);
app.add_systems(Update, my_cursor_system);
```

If you have a more complex application with multiple windows, here is a more
complex version of the code that can handle that:

`Code (multi-window version):`

```rust no_run noplayground hljs
use bevy::render::camera::RenderTarget;
use bevy::window::WindowRef;

/// We will add this to each camera we want to compute cursor position for.
/// Add the component to the camera that renders to each window.
#[derive(Component, Default)]
struct WorldCursorCoords(Vec2);

fn setup_multiwindow(mut commands: Commands) {
    // TODO: set up multiple cameras for multiple windows.
    // See bevy's example code for how to do that.

    // Make sure we add our component to each camera
    commands.spawn((Camera2dBundle::default(), WorldCursorCoords::default()));
}

fn my_cursor_system_multiwindow(
    // query to get the primary window
    q_window_primary: Query<&Window, With<PrimaryWindow>>,
    // query to get other windows
    q_window: Query<&Window>,
    // query to get camera transform
    mut q_camera: Query<(&Camera, &GlobalTransform, &mut WorldCursorCoords)>,
) {
    for (camera, camera_transform, mut worldcursor) in &mut q_camera {
        // get the window the camera is rendering to
        let window = match camera.target {
            // the camera is rendering to the primary window
            RenderTarget::Window(WindowRef::Primary) => {
                q_window_primary.single()
            },
            // the camera is rendering to some other window
            RenderTarget::Window(WindowRef::Entity(e_window)) => {
                q_window.get(e_window).unwrap()
            },
            // the camera is rendering to something else (like a texture), not a window
            _ => {
                // skip this camera
                continue;
            }
        };

        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        if let Some(world_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            worldcursor.0 = world_position;
        }
    }
}
```

```rust no_run noplayground hljs
app.add_systems(Startup, setup_multiwindow);
app.add_systems(Update, my_cursor_system_multiwindow);
```

If you'd like to be able to detect what 3D object the cursor is pointing at, select
objects, etc., there is a good (unofficial) plugin:
[`bevy_mod_picking`](https://github.com/aevyrie/bevy_mod_picking).

For a simple top-down camera view game with a flat ground plane, it might be
sufficient to just compute the coordinates on the ground under the cursor.

Load Interactive Example

In the interactive example, there is a ground plane with a non-default position
and rotation. There is a red cube, which is positioned using the global
coordinates, and a blue cube, which is a [child entity](https://bevy-cheatbook.github.io/fundamentals/hierarchy.html) of the
ground plane and positioned using local coordinates. They should both follow the
cursor.

`Code and explanation:`

```rust no_run noplayground hljs
/// Here we will store the position of the mouse cursor on the 3D ground plane.
#[derive(Resource, Default)]
struct MyGroundCoords {
    // Global (world-space) coordinates
    global: Vec3,
    // Local (relative to the ground plane) coordinates
    local: Vec2,
}

/// Used to help identify our main camera
#[derive(Component)]
struct MyGameCamera;

/// Used to help identify our ground plane
#[derive(Component)]
struct MyGroundPlane;

fn setup_3d_scene(mut commands: Commands) {
    // Make sure to add the marker component when you set up your camera
    commands.spawn((
        MyGameCamera,
        Camera3dBundle {
            // ... your camera configuration ...
            ..default()
        },
    ));
    // Spawn the ground
    commands.spawn((
        MyGroundPlane,
        PbrBundle {
            // feel free to change this to rotate/tilt or reposition the ground
            transform: Transform::default(),
            // TODO: set up your mesh / visuals for rendering:
            // mesh: ...
            // material: ...
            ..default()
        },
    ));
}

fn cursor_to_ground_plane(
    mut mycoords: ResMut<MyGroundCoords>,
    // query to get the window (so we can read the current cursor position)
    // (we will only work with the primary window)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MyGameCamera>>,
    // query to get ground plane's transform
    q_plane: Query<&GlobalTransform, With<MyGroundPlane>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // Ditto for the ground plane's transform
    let ground_transform = q_plane.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    let Some(cursor_position) = window.cursor_position() else {
        // if the cursor is not inside the window, we can't do anything
        return;
    };

    // Mathematically, we can represent the ground as an infinite flat plane.
    // To do that, we need a point (to position the plane) and a normal vector
    // (the "up" direction, perpendicular to the ground plane).

    // We can get the correct values from the ground entity's GlobalTransform
    let plane_origin = ground_transform.translation();
    let plane = Plane3d::new(ground_transform.up());

    // Ask Bevy to give us a ray pointing from the viewport (screen) into the world
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        // if it was impossible to compute for whatever reason; we can't do anything
        return;
    };

    // do a ray-plane intersection test, giving us the distance to the ground
    let Some(distance) = ray.intersect_plane(plane_origin, plane) else {
        // If the ray does not intersect the ground
        // (the camera is not looking towards the ground), we can't do anything
        return;
    };

    // use the distance to compute the actual point on the ground in world-space
    let global_cursor = ray.get_point(distance);

    mycoords.global = global_cursor;
    eprintln!("Global cursor coords: {}/{}/{}",
        global_cursor.x, global_cursor.y, global_cursor.z
    );

    // to compute the local coordinates, we need the inverse of the plane's transform
    let inverse_transform_matrix = ground_transform.compute_matrix().inverse();
    let local_cursor = inverse_transform_matrix.transform_point3(global_cursor);

    // we can discard the Y coordinate, because it should always be zero
    // (our point is supposed to be on the plane)
    mycoords.local = local_cursor.xz();
    eprintln!("Local cursor coords: {}/{}", local_cursor.x, local_cursor.z);
}
```

```rust no_run noplayground hljs
app.init_resource::<MyGroundCoords>();
app.add_systems(Startup, setup_3d_scene);
app.add_systems(Update, cursor_to_ground_plane);
```

If the ground is tilted/rotated or moved, the global and local coordinates
will differ, and may be useful for different use cases, so we compute both.

For some examples:

- if you want to spawn a [child](https://bevy-cheatbook.github.io/fundamentals/hierarchy.html) entity, or to quantize
the coordinates to a grid (for a tile-based game, to detect the grid tile under the cursor),
the local coordinates will be more useful
- if you want to spawn some overlays, particle effects, other independent game entities,
at the position of the cursor, the global coordinates will be more useful

[GitHub Sponsors](https://github.com/sponsors/inodentry) [Patreon](https://patreon.com/iyesgames) [Bitcoin](bitcoin:bc1qaf32uqsg6mngw9g4aqc3l2jvuv46qx0zw2438p) If you like this book, please donate to support me!

I also offer professional tutoring / private lessons for Bevy and Rust. [Contact me](https://bevy-cheatbook.github.io/contact.html) if interested!
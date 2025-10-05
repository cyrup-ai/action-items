---
url: "https://bevy-cheatbook.github.io/cookbook/smooth-movement.html"
title: "Transform Interpolation/Extrapolation - Unofficial Bevy Cheat Book"
---

- Auto
- Light
- Rust
- Coal
- Navy
- Ayu

# Unofficial Bevy Cheat Book

[Print this book](https://bevy-cheatbook.github.io/print.html "Print this book")[Suggest an edit](https://github.com/bevy-cheatbook/bevy-cheatbook/issues/new "Suggest an edit")

| [Bevy Version:](https://bevy-cheatbook.github.io/introduction.html#maintenance-policy) | [0.14](https://bevyengine.org/news/bevy-0-14) | (outdated!) |
| --- | --- | --- |

As this page is outdated, please refer to Bevy's official migration guides while reading,
to cover the differences:
[0.14 to 0.15](https://bevyengine.org/learn/migration-guides/0-14-to-0-15/),
[0.15 to 0.16](https://bevyengine.org/learn/migration-guides/0-15-to-0-16/).

I apologize for the inconvenience. I will update the page as soon as I find the time.

* * *

Movement code for controlling the player (and other gameplay entities)
can pose a tricky problem.

You want it to be computed reliably as part of your gameplay/physics
simulation, which means doing it [on a fixed timestep](https://bevy-cheatbook.github.io/fundamentals/fixed-timestep.html). This
is to ensure consistent gameplay behavior regardless of the display
framerate. It is a must, to avoid glitchy behavior like clipping into walls.

However, you also want movement to look smooth on-screen. If you simply
mutate the [transforms](https://bevy-cheatbook.github.io/fundamentals/transforms.html) from within [`FixedUpdate`](https://docs.rs/bevy/0.14/bevy/app/struct.FixedUpdate.html), that
will look choppy, especially on modern high-refresh-rate gaming displays.

The solution is to not manipulate [`Transform`](https://docs.rs/bevy/0.14/bevy/transform/components/struct.Transform.html) directly, but to create your
own custom [component](https://bevy-cheatbook.github.io/programming/ec.html#components) types to use instead. Implement your
gameplay using your own types. Then, have a system in [`Update`](https://docs.rs/bevy/0.14/bevy/app/struct.Update.html), which uses
your custom components to compute the [`Transform`](https://docs.rs/bevy/0.14/bevy/transform/components/struct.Transform.html) that Bevy should use to
display the entity on every frame.

Interpolation means computing a [`Transform`](https://docs.rs/bevy/0.14/bevy/transform/components/struct.Transform.html) that is somewhere in-between the
current state of the entity, and the old state from the previous gameplay tick.

Extrapolation means computing a [`Transform`](https://docs.rs/bevy/0.14/bevy/transform/components/struct.Transform.html) that is somewhere in-between
the current state of the entity, and the predicted future state on the next
gameplay tick.

Interpolation creates movement that always looks both smooth and accurate,
but feels laggy / less responsive. The user will never see a truly up-to-date
representation of the gameplay state, as what you are rendering is always
delayed by one fixed timestep duration. Thus, interpolation is not suitable
for games where a responsive low-latency feel is important to gameplay.

Extrapolation creates movement that looks smooth and feels responsive, but
may be inaccurate. Since you are trying to predict the future, you might
guess wrong, and occasionally the rendered position of the entity on-screen
might jump slightly, to correct mispredictions.

First, create some custom [components](https://bevy-cheatbook.github.io/programming/ec.html#components) to store your movement
state.

If you'd like to do interpolation, you need to remember the old position from
the previous gameplay tick. We created a separate component for that purpose.

If you'd like to do extrapolation, it might not be necessary, depending on
how you go about predicting the future position.

```rust no_run noplayground hljs

#[derive(Component)]
struct MyMovementState {
    position: Vec3,
    velocity: Vec3,
}

#[derive(Component)]
struct OldMovementState {
    position: Vec3,
}
```

Now, you can create your [systems](https://bevy-cheatbook.github.io/programming/systems.html) to implement your movement
simulation. These systems should run in [`FixedUpdate`](https://docs.rs/bevy/0.14/bevy/app/struct.FixedUpdate.html). For this simple
example, we just apply our velocity value.

```rust no_run noplayground hljs

fn my_movement(
    time: Res<Time>,
    mut q_movement: Query<(&mut MyMovementState, &mut OldMovementState)>,
) {
    for (mut state, mut old_state) in &mut q_movement {
        // Reborrow `state` to mutably access both of its fields
        // See Cheatbook page on "Split Borrows"
        let state = &mut *state;

        // Store the old position.
        old_state.position = state.position;

        // Compute the new position.
        // (`delta_seconds` always returns the fixed timestep
        // duration, if this system is added to `FixedUpdate`)
        state.position += state.velocity * time.delta_seconds();
    }
}
```

```rust no_run noplayground hljs

app.add_systems(FixedUpdate, my_movement);
```

Now we need to create the [system](https://bevy-cheatbook.github.io/programming/systems.html) to run every frame in
[`Update`](https://docs.rs/bevy/0.14/bevy/app/struct.Update.html), which computes the actual [transform](https://bevy-cheatbook.github.io/fundamentals/transforms.html) that Bevy
will use to display our entity on-screen.

[`Time<Fixed>`](https://docs.rs/bevy/0.14/bevy/time/struct.Time.html) can give us the "overstep fraction", which is a
value between `0.0` and `1.0` indicating how much of a "partial timestep"
has accumulated since the last [fixed timestep](https://bevy-cheatbook.github.io/fundamentals/fixed-timestep.html) run.
This value is our lerp coefficient.

```rust no_run noplayground hljs

fn transform_movement_interpolate(
    fixed_time: Res<Time<Fixed>>,
    mut q_movement: Query<(
        &mut Transform, &MyMovementState, &OldMovementState
    )>,
) {
    for (mut xf, state, old_state) in &mut q_movement {
        let a = fixed_time.overstep_fraction();
        xf.translation = old_state.position.lerp(state.position, a);
    }
}
```

```rust no_run noplayground hljs

```

To do extrapolation, you need some sort of prediction about the future
position on the next gameplay tick.

In our example, we have our `velocity` value and we can reasonably assume
that on the next tick, it will simply be added to the position. So we can
use that as our prediction. As a general principle, if you have the necessary
information to make a good prediction about the future position, you should
use it.

```rust no_run noplayground hljs

fn transform_movement_extrapolate_velocity(
    fixed_time: Res<Time<Fixed>>,
    mut q_movement: Query<(
        &mut Transform, &MyMovementState,
    )>,
) {
    for (mut xf, state) in &mut q_movement {
        let a = fixed_time.overstep_fraction();
        let future_position = state.position
            + state.velocity * fixed_time.delta_seconds();
        xf.translation = state.position.lerp(future_position, a);
    }
}
```

```rust no_run noplayground hljs

```

If you'd like to make a general implementation of extrapolation, that does
not rely on knowing any information about how the movement works (such as
our `velocity` value in this example), you could try predicting the future
position based on the old position, assuming it will continue moving the
same way.

```rust no_run noplayground hljs

fn transform_movement_extrapolate_from_old(
    fixed_time: Res<Time<Fixed>>,
    mut q_movement: Query<(
        &mut Transform, &MyMovementState, &OldMovementState
    )>,
) {
    for (mut xf, state, old_state) in &mut q_movement {
        let a = fixed_time.overstep_fraction();
        let delta = state.position - old_state.position;
        let future_position = state.position + delta;
        xf.translation = state.position.lerp(future_position, a);
    }
}
```

```rust no_run noplayground hljs

```

However, such an implementation will always guess wrong if the velocity is
changing, leading to poor results (jumpy movement that needs to correct its
course often).

[GitHub Sponsors](https://github.com/sponsors/inodentry) [Patreon](https://patreon.com/iyesgames) [Bitcoin](bitcoin:bc1qaf32uqsg6mngw9g4aqc3l2jvuv46qx0zw2438p) If you like this book, please donate to support me!

I also offer professional tutoring / private lessons for Bevy and Rust. [Contact me](https://bevy-cheatbook.github.io/contact.html) if interested!
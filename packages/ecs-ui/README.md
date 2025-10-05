# ECS UI

A high-performance, ergonomic UI library for Bevy using Entity Component System architecture.

## Features

- Declarative layout system with Window, Boundary, and Solid layouts
- State-based animations and transitions
- Theme system for consistent styling
- Fluent builder API for intuitive UI construction
- Event-driven interactions for decoupled UI logic

## Usage

```rust
use bevy::prelude::*;
use ecs_ui::prelude::*;

fn setup(mut commands: Commands) {
    // Spawn UI root
    commands.spawn((
        UiLayoutRoot::new_2d(),
        UiFetchFromCamera::<0>,
    )).with_children(|ui| {
        // Spawn a button with auto-hover and click handling
        ui.spawn_ui_button("Click Me", UiLayout::window().centered().pack());
    });
}
```
# Bevy Game Engine Expertise Summary

## Core Architecture Patterns

### Entity Component System (ECS)
- **Entities**: Unique IDs that group components together
- **Components**: Plain Rust data types (`#[derive(Component)]`)
- **Systems**: Functions that operate on entities with specific component combinations
- **Resources**: Global state accessible across systems (`Res<T>`, `ResMut<T>`)

### App Builder Pattern
```rust
App::new()
    .add_plugins(DefaultPlugins)
    .init_resource::<GameState>()
    .add_systems(Startup, setup_system)
    .add_systems(Update, (system1, system2).chain())
    .run();
```

### System Scheduling
- **Startup**: Runs once at app initialization
- **Update**: Runs every frame
- **Last**: Runs at the end of each frame
- **Custom Sets**: Group related systems with ordering constraints
- **System Ordering**: `.before()`, `.after()`, `.chain()`

## Rendering Systems

### 3D Rendering
- **Mesh3d**: 3D geometry component
- **MeshMaterial3d**: Material/texture component
- **Transform**: Position, rotation, scale
- **Camera3d**: 3D camera with projection
- **PointLight**: Lighting with shadows support

### 2D Rendering
- **Sprite**: 2D image rendering
- **Camera2d**: 2D camera system
- **Mesh2d**: Custom 2D geometry
- **Transform**: Also used for 2D positioning

### Asset Management
- **AssetServer**: Load assets from files (`asset_server.load()`)
- **Assets<T>**: Resource for managing asset collections
- **Handle<T>**: Reference to loaded assets

## Animation Framework

### Core Components
- **AnimationClip**: Defines keyframe animations
- **AnimationPlayer**: Controls playback
- **AnimationTarget**: Links entities to animations
- **AnimationGraph**: Manages complex animation states

### Animation Curves
- Target specific transform properties (translation, rotation, scale)
- Support hierarchical targeting by entity names
- Keyframe-based with interpolation

## Input Handling

### Input Resources
- **ButtonInput<KeyCode>**: Keyboard state
- **ButtonInput<MouseButton>**: Mouse button state
- **ButtonInput<GamepadButton>**: Gamepad input
- **Events**: Input event streams for frame-specific detection

### Input Patterns
```rust
fn input_system(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.pressed(KeyCode::KeyA) { /* held */ }
    if keyboard.just_pressed(KeyCode::KeyA) { /* this frame */ }
    if keyboard.just_released(KeyCode::KeyA) { /* released this frame */ }
}
```

## UI System

### UI Components
- **Node**: Layout container with flexbox properties
- **Button**: Interactive button component
- **Text**: Text rendering with styling
- **BackgroundColor**: Component for background colors
- **BorderColor**: Component for borders

### UI Patterns
- Component-based styling (similar to CSS-in-JS)
- Flexbox layout system
- Interaction states (None, Hovered, Pressed)
- Hierarchical UI with parent-child relationships

## Query System

### Query Types
- **Query<&Component>**: Read-only access
- **Query<&mut Component>**: Mutable access
- **Query<(Component1, Component2)>**: Multiple components
- **Query<Entity>**: Access entity IDs
- **With<Component>**: Filter entities with component
- **Without<Component>**: Filter entities without component

### Query Patterns
```rust
fn system(mut query: Query<(&Player, &mut Score), With<Active>>) {
    for (player, mut score) in &mut query {
        // Process each entity
    }
}
```

## Commands System

### Entity Management
- **Commands**: Deferred world mutations
- **spawn()**: Create new entities
- **spawn_batch()**: Create multiple entities efficiently
- **despawn()**: Remove entities
- **insert()**: Add components to existing entities

## Plugin System

### Built-in Plugins
- **DefaultPlugins**: Core functionality (rendering, input, audio, etc.)
- **MinimalPlugins**: Minimal set for headless applications
- Custom plugins for modular functionality

## Performance Patterns

### System Optimization
- Parallel execution by default
- Change detection for efficient updates
- System sets for controlling execution order
- Local<T> for system-specific state

### Memory Management
- Component storage optimized for cache efficiency
- Asset sharing through handles
- Automatic cleanup of unused assets

## Advanced Concepts

### Events
- **EventWriter<T>**: Send events
- **EventReader<T>**: Receive events
- Frame-based event system

### Observers
- React to component changes
- Entity lifecycle events
- Custom event propagation

### States
- State machines for game flow
- State-scoped systems
- Transition handling

## Best Practices

1. **Component Design**: Keep components focused on single concerns
2. **System Organization**: Group related systems in sets
3. **Resource Usage**: Minimize global state, prefer components
4. **Performance**: Use change detection and efficient queries
5. **Architecture**: Leverage ECS patterns for clean, maintainable code

## Common Patterns

### Game Loop Structure
```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .add_systems(Update, (
        input_system,
        movement_system,
        collision_system,
        render_system,
    ).chain())
    .run();
```

### Entity Spawning
```rust
commands.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    Mesh3d(meshes.add(Sphere::default())),
    MeshMaterial3d(materials.add(Color::RED)),
));
```

This knowledge foundation enables building complex games and applications using Bevy's powerful, data-driven architecture.
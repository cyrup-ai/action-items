use std::collections::HashMap;

use bevy::input::ButtonState;
use bevy::input::gamepad::GamepadButtonChangedEvent;
use bevy::input::mouse::MouseButtonInput;
use bevy::picking::PickSet;
use bevy::picking::pointer::{Location, PointerAction, PointerId, PointerInput, PointerLocation};
use bevy::picking::prelude::Pickable;
use bevy::prelude::*;
use bevy::render::camera::{NormalizedRenderTarget, RenderTarget};
use bevy::window::{PrimaryWindow, SystemCursorIcon, WindowRef};
use bevy::winit::cursor::CursorIcon;

use crate::Dimension;

// Exported prelude
pub mod prelude {
    // All standard exports
    // Export stuff from other crates
    pub use bevy::window::SystemCursorIcon;

    pub use super::{
        ConfigError, GamepadCursor, GamepadCursorConfig, GamepadCursorMode, NavigationError,
        OnHoverSetCursor, SoftwareCursor,
    };
}

// #=========================#
// #=== CURSOR ICON QUEUE ===#

#[derive(Resource, Reflect, Clone, PartialEq, Debug, Default)]
pub struct CursorIconQueue {
    pointers: HashMap<PointerId, CursorQueueData>,
}
impl CursorIconQueue {
    /// A method to request a new cursor icon. Works only if priority is higher than already set
    /// priority this tick.
    pub fn request_cursor(
        &mut self,
        pointer: PointerId,
        window: Option<Entity>,
        requestee: Entity,
        request: SystemCursorIcon,
        priority: usize,
    ) {
        if let Some(data) = self.pointers.get_mut(&pointer) {
            data.window = window;
            data.queue.insert(requestee, (request, priority));
        } else {
            let mut queue = HashMap::new();
            queue.insert(requestee, (request, priority));
            self.pointers.insert(pointer, CursorQueueData {
                window,
                queue,
                top_priority: 0,
                top_request: Default::default(),
            });
        }
    }
    /// A method to cancel existing cursor in the queue stack
    pub fn cancel_cursor(&mut self, pointer: PointerId, requestee: &Entity) {
        if let Some(data) = self.pointers.get_mut(&pointer) {
            data.queue.remove(requestee);
        }
    }
}

#[derive(Reflect, Clone, PartialEq, Debug)]
struct CursorQueueData {
    window: Option<Entity>,
    top_priority: usize,
    top_request: SystemCursorIcon,
    queue: HashMap<Entity, (SystemCursorIcon, usize)>,
}

/// This system will apply cursor changes to the windows it has in the resource.
fn system_cursor_icon_queue_apply(
    mut queue: ResMut<CursorIconQueue>,
    mut windows: Query<Option<&mut CursorIcon>, With<Window>>,
    mut commands: Commands,
) {
    if !queue.is_changed() {
        return;
    }
    for data in queue.pointers.values_mut() {
        let mut top_priority = 0;
        let mut top_request = SystemCursorIcon::Default;

        // Look for highest priority to use
        for (icon, priority) in data.queue.values() {
            if *priority > top_priority {
                top_priority = *priority;
                top_request = *icon;
            }
        }

        data.top_priority = top_priority;
        data.top_request = top_request;

        if let Some(window) = data.window
            && let Ok(window_cursor_option) = windows.get_mut(window) {
                // Apply the cursor icon somehow
                if let Some(mut window_cursor) = window_cursor_option {
                    match window_cursor.as_mut() {
                        CursorIcon::System(previous) => {
                            if *previous != data.top_request {
                                *previous = data.top_request;
                            }
                        },
                    }
                } else {
                    commands
                        .entity(window)
                        .insert(CursorIcon::System(data.top_request));
                }
            }
    }
}

/// This system will cleanup the queue if any invalid data is found.
fn system_cursor_icon_queue_purge(
    mut queue: ResMut<CursorIconQueue>,
    mut windows: Query<&Window>,
    entities: Query<Entity>,
) {
    let mut to_remove = Vec::new();
    for (pointer, data) in &mut queue.pointers {
        // Remove invalid pointers
        if let Some(window) = data.window
            && windows.get_mut(window).is_err() {
                to_remove.push(*pointer);
            }

        // Remove despawned entities
        let mut entities_to_remove = Vec::new();
        for entity in data.queue.keys() {
            if entities.get(*entity).is_err() {
                entities_to_remove.push(*entity);
            }
        }

        // Cleanup
        for entity in entities_to_remove {
            data.queue.remove(&entity);
        }
    }

    // Cleanup
    for pointer in to_remove {
        queue.pointers.remove(&pointer);
    }
}

// #========================#
// #=== CURSOR ADDITIONS ===#

/// Requests cursor icon on hover
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
#[require(Pickable)]
pub struct OnHoverSetCursor {
    /// SoftwareCursor type to request on hover
    pub cursor: SystemCursorIcon,
}
impl OnHoverSetCursor {
    /// Creates new struct
    pub fn new(cursor: SystemCursorIcon) -> Self {
        OnHoverSetCursor { cursor }
    }
}

fn observer_cursor_request_cursor_icon(
    mut trigger: Trigger<Pointer<Over>>,
    mut pointers: Query<(&PointerId, &PointerLocation, Has<GamepadCursor>)>,
    query: Query<&OnHoverSetCursor>,
    mut queue: ResMut<CursorIconQueue>,
) {
    // Find the pointer location that triggered this observer
    let id = trigger.pointer_id;
    for (pointer, location, is_gamepad) in pointers.iter_mut().filter(|(p_id, ..)| id == **p_id) {
        // Check if the pointer is attached to a window
        if let Some(location) = &location.location
            && let NormalizedRenderTarget::Window(window) = location.target {
                // Request a cursor change
                if let Ok(requestee) = query.get(trigger.target) {
                    trigger.propagate(false);
                    queue.request_cursor(
                        *pointer,
                        if is_gamepad {
                            None
                        } else {
                            Some(window.entity())
                        },
                        trigger.target,
                        requestee.cursor,
                        1,
                    );
                }
            }
    }
}

fn observer_cursor_cancel_cursor_icon(
    mut trigger: Trigger<Pointer<Out>>,
    mut pointers: Query<(&PointerId, &PointerLocation)>,
    query: Query<&OnHoverSetCursor>,
    mut queue: ResMut<CursorIconQueue>,
) {
    // Find the pointer location that triggered this observer
    let id = trigger.pointer_id;
    for (pointer, location) in pointers.iter_mut().filter(|(p_id, _)| id == **p_id) {
        // Check if the pointer is attached to a window
        if let Some(location) = &location.location
            && matches!(location.target, NormalizedRenderTarget::Window(_)) {
                // Cancel existing cursor icon request if applicable
                if query.get(trigger.target).is_ok() {
                    trigger.propagate(false);
                    queue.cancel_cursor(*pointer, &trigger.target);
                }
            }
    }
}

// #=======================#
// #=== SOFTWARE CURSOR ===#

/// Component for creating software mouse.
#[derive(Component, Reflect, Clone, PartialEq, Debug, Default)]
#[require(PointerId, Pickable::IGNORE)]
pub struct SoftwareCursor {
    /// Indicates which cursor is being requested.
    cursor_request: SystemCursorIcon,
    /// Indicates the priority of the requested cursor.
    cursor_request_priority: f32,
    /// Map which cursor has which atlas index and offset
    cursor_atlas_map: HashMap<SystemCursorIcon, (usize, Vec2)>,
    /// Location of the cursor (same as [`Transform`] without sprite offset).
    pub location: Vec2,
}
impl SoftwareCursor {
    /// Creates new default SoftwareCursor.
    pub fn new() -> SoftwareCursor {
        SoftwareCursor {
            cursor_request: SystemCursorIcon::Default,
            cursor_request_priority: 0.0,
            cursor_atlas_map: HashMap::new(),
            location: Vec2::ZERO,
        }
    }
    /// A method to request a new cursor icon. Works only if priority is higher than already set
    /// priority this tick.
    pub fn request_cursor(&mut self, request: SystemCursorIcon, priority: f32) {
        if priority > self.cursor_request_priority {
            self.cursor_request = request;
            self.cursor_request_priority = priority;
        }
    }
    /// This function binds the specific cursor icon to an image index that is used if the entity
    /// has texture atlas attached to it.
    pub fn set_index(
        mut self,
        icon: SystemCursorIcon,
        index: usize,
        offset: impl Into<Vec2>,
    ) -> Self {
        self.cursor_atlas_map.insert(icon, (index, offset.into()));
        self
    }
}

/// This will make the [`SoftwareCursor`] controllable by a gamepad.
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
pub struct GamepadCursor {
    /// This struct defines how should the cursor movement behave.
    pub mode: GamepadCursorMode,
    /// SoftwareCursor speed scale
    pub speed: f32,
}
impl GamepadCursor {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self::default()
    }
}
impl Default for GamepadCursor {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            speed: 1.0,
        }
    }
}

/// This struct defines how should the cursor movement behave.
#[derive(Debug, Clone, Default, PartialEq, Reflect)]
pub enum GamepadCursorMode {
    /// SoftwareCursor will freely move on input.
    #[default]
    Free,
    /// Will try to snap to nearby nodes on input.
    /// Uses directional input to find the closest UI element in that direction
    /// and smoothly moves the cursor toward it.
    Snap,
}

/// Configuration for gamepad cursor behavior in snap mode
#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct GamepadCursorConfig {
    /// Threshold for input magnitude before attempting snap navigation (0.0-1.0)
    pub snap_threshold: f32,
    /// Movement speed multiplier when snapping to targets  
    pub snap_speed: f32,
    /// Maximum distance to search for snap targets
    pub max_snap_distance: f32,
    /// Bias factor for directional alignment when selecting targets
    pub directional_bias: f32,
    /// Speed multiplier for free movement fallback in snap mode
    pub fallback_speed: f32,
    /// Minimum alignment threshold for considering elements in snap direction (0.0-1.0)
    pub alignment_threshold: f32,
    /// Multiplier for alignment bonus in scoring algorithm
    pub alignment_bonus_multiplier: f32,
    /// Minimum distance threshold to consider elements for snapping
    pub minimum_distance: f32,
    /// Gamepad stick deadzone threshold (0.0-1.0)
    pub gamepad_deadzone: f32,
    /// Base movement speed multiplier for cursor movement
    pub movement_speed_multiplier: f32,
}

impl Default for GamepadCursorConfig {
    fn default() -> Self {
        Self {
            snap_threshold: 0.1,
            snap_speed: 800.0,
            max_snap_distance: 200.0,
            directional_bias: 2.0,
            fallback_speed: 0.5,
            alignment_threshold: 0.3,
            alignment_bonus_multiplier: 50.0,
            minimum_distance: 1.0,
            gamepad_deadzone: 0.1,
            movement_speed_multiplier: 500.0,
        }
    }
}

impl GamepadCursorConfig {
    /// Validates the configuration parameters and returns a Result indicating success or failure.
    /// This ensures all values are within reasonable ranges to prevent runtime issues.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate threshold values are in 0.0-1.0 range
        if !(0.0..=1.0).contains(&self.snap_threshold) {
            return Err(ConfigError::OutOfRange {
                param: "snap_threshold".to_string(),
                value: self.snap_threshold,
                min: 0.0,
                max: 1.0,
            });
        }

        if !(0.0..=1.0).contains(&self.alignment_threshold) {
            return Err(ConfigError::OutOfRange {
                param: "alignment_threshold".to_string(),
                value: self.alignment_threshold,
                min: 0.0,
                max: 1.0,
            });
        }

        if !(0.0..=1.0).contains(&self.gamepad_deadzone) {
            return Err(ConfigError::OutOfRange {
                param: "gamepad_deadzone".to_string(),
                value: self.gamepad_deadzone,
                min: 0.0,
                max: 1.0,
            });
        }

        // Validate positive values
        if self.snap_speed <= 0.0 {
            return Err(ConfigError::MustBePositive {
                param: "snap_speed".to_string(),
                value: self.snap_speed,
            });
        }

        if self.max_snap_distance <= 0.0 {
            return Err(ConfigError::MustBePositive {
                param: "max_snap_distance".to_string(),
                value: self.max_snap_distance,
            });
        }

        if self.minimum_distance < 0.0 {
            return Err(ConfigError::OutOfRange {
                param: "minimum_distance".to_string(),
                value: self.minimum_distance,
                min: 0.0,
                max: f32::INFINITY,
            });
        }

        if self.movement_speed_multiplier <= 0.0 {
            return Err(ConfigError::MustBePositive {
                param: "movement_speed_multiplier".to_string(),
                value: self.movement_speed_multiplier,
            });
        }

        if self.directional_bias <= 0.0 {
            return Err(ConfigError::MustBePositive {
                param: "directional_bias".to_string(),
                value: self.directional_bias,
            });
        }

        if self.fallback_speed <= 0.0 {
            return Err(ConfigError::MustBePositive {
                param: "fallback_speed".to_string(),
                value: self.fallback_speed,
            });
        }

        if self.alignment_bonus_multiplier <= 0.0 {
            return Err(ConfigError::MustBePositive {
                param: "alignment_bonus_multiplier".to_string(),
                value: self.alignment_bonus_multiplier,
            });
        }

        // Validate logical relationships
        if self.minimum_distance >= self.max_snap_distance {
            return Err(ConfigError::LogicalConstraint {
                constraint: format!(
                    "minimum_distance ({}) must be less than max_snap_distance ({})",
                    self.minimum_distance, self.max_snap_distance
                ),
            });
        }

        Ok(())
    }

    /// Creates a new validated configuration from another config, falling back to defaults for
    /// invalid values. This provides a safe way to create configurations that are guaranteed to
    /// be valid.
    pub fn new_validated(config: Self) -> Self {
        // If validation fails, return default config
        if config.validate().is_err() {
            Self::default()
        } else {
            config
        }
    }
}

/// Error types for spatial navigation failures
#[derive(Debug, Clone, thiserror::Error)]
pub enum NavigationError {
    #[error("No valid snap target found in the specified direction")]
    NoValidTarget,
    #[error("Input direction too weak (magnitude: {magnitude}, threshold: {threshold})")]
    WeakInput { magnitude: f32, threshold: f32 },
    #[error("All potential targets are too far away (closest: {distance}, max: {max})")]
    AllTargetsTooFar { distance: f32, max: f32 },
}

/// Error types for configuration validation failures
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error("Parameter {param} value {value} is out of valid range {min}..={max}")]
    OutOfRange {
        param: String,
        value: f32,
        min: f32,
        max: f32,
    },
    #[error("Parameter {param} must be positive, got {value}")]
    MustBePositive { param: String, value: f32 },
    #[error("Logical constraint violated: {constraint}")]
    LogicalConstraint { constraint: String },
}

/// This component is used for SoftwareCursor-Gamepad relation.
/// - It is added to a Gamepad if he has a virtual cursor assigned.
/// - It is added to a SoftwareCursor if he is assigned to an existing gamepad.
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
pub struct GamepadAttachedCursor(pub Entity);

// #========================#
// #=== CURSOR FUNCTIONS ===#

/// This system will hide the native cursor.
fn system_cursor_hide_native(
    mut windows: Query<&mut Window>,
    query: Query<(&PointerLocation, Has<GamepadCursor>), With<SoftwareCursor>>,
) {
    for (pointer_location, _is_gamepad) in &query {
        if let Some(location) = &pointer_location.location
            && let NormalizedRenderTarget::Window(window) = location.target
                && let Ok(mut window) = windows.get_mut(window.entity()) {
                    window.cursor_options.visible = false;
                }
    }
}

/// This system will hide the native cursor.
fn system_cursor_software_change_icon(
    icons: Res<CursorIconQueue>,
    mut query: Query<(&PointerId, &SoftwareCursor, &mut Sprite)>,
) {
    for (pointer_id, software_cursor, mut sprite) in &mut query {
        if let Some(atlas) = &mut sprite.texture_atlas
            && let Some(icon_data) = icons.pointers.get(pointer_id) {
                atlas.index = software_cursor
                    .cursor_atlas_map
                    .get(&icon_data.top_request)
                    .unwrap_or(&(0, Vec2::ZERO))
                    .0;
            }
    }
}

/// This system will attach free cursors to available gamepads using 1:1 pairing.
fn system_cursor_gamepad_assign(
    mut commands: Commands,
    cursors: Query<(Entity, &SoftwareCursor, &GamepadCursor), Without<GamepadAttachedCursor>>,
    gamepads: Query<(Entity, &Gamepad), Without<GamepadAttachedCursor>>,
) {
    // Pair cursors with gamepads 1:1
    for ((cursor_entity, ..), (gamepad_entity, _)) in cursors.iter().zip(gamepads.iter()) {
        commands
            .entity(cursor_entity)
            .insert(GamepadAttachedCursor(gamepad_entity));
        commands
            .entity(gamepad_entity)
            .insert(GamepadAttachedCursor(cursor_entity));
        info!("Gamepad {gamepad_entity} bound to cursor {cursor_entity}");
    }
}

/// Attempts to find the nearest UI element in the specified input direction using spatial
/// navigation. Returns the position to snap to, or an error if no suitable target is found.
fn snap_to_nearest_element(
    current_pos: &Vec2,
    input_direction: Vec2,
    ui_elements: &Query<(&GlobalTransform, &Dimension), (With<Pickable>, Without<SoftwareCursor>)>,
    config: &GamepadCursorConfig,
) -> Result<Vec2, NavigationError> {
    let input_magnitude = input_direction.length();

    if input_magnitude < config.snap_threshold {
        return Err(NavigationError::WeakInput {
            magnitude: input_magnitude,
            threshold: config.snap_threshold,
        });
    }

    // Prevent NaN from normalize() if input_magnitude is exactly 0
    if input_magnitude == 0.0 {
        return Err(NavigationError::WeakInput {
            magnitude: input_magnitude,
            threshold: config.snap_threshold,
        });
    }

    let normalized_input = input_direction.normalize();
    let mut best_candidate: Option<Vec2> = None;
    let mut best_score = f32::INFINITY;

    for (transform, _dimension) in ui_elements.iter() {
        let element_pos = transform.translation().truncate();
        let to_element = element_pos - *current_pos;
        let distance = to_element.length();

        // Skip elements at exactly the same position (prevents NaN from normalize())
        if distance == 0.0 {
            continue;
        }

        // Skip elements that are too far away
        if distance > config.max_snap_distance || distance < config.minimum_distance {
            continue;
        }

        // Calculate alignment with input direction
        let element_direction = to_element.normalize();
        let alignment = element_direction.dot(normalized_input);

        // Only consider elements in the general direction of input
        // Use alignment threshold to be more inclusive than exact directional alignment
        if alignment > config.alignment_threshold {
            // Scoring algorithm combines distance and directional alignment
            // Lower score is better - prioritizes closer elements with better alignment
            let distance_score = distance;
            let alignment_bonus =
                alignment * config.directional_bias * config.alignment_bonus_multiplier;
            let score = distance_score - alignment_bonus;

            if score < best_score {
                best_score = score;
                best_candidate = Some(element_pos);
            }
        }
    }

    if let Some(target) = best_candidate {
        Ok(target)
    } else {
        // Check if there were any elements at all within range
        let closest_distance = ui_elements
            .iter()
            .map(|(transform, _)| {
                let element_pos = transform.translation().truncate();
                (*current_pos - element_pos).length()
            })
            .fold(f32::INFINITY, f32::min);

        if closest_distance > config.max_snap_distance {
            Err(NavigationError::AllTargetsTooFar {
                distance: closest_distance,
                max: config.max_snap_distance,
            })
        } else {
            Err(NavigationError::NoValidTarget)
        }
    }
}

/// Applies free movement to the cursor for fallback scenarios in snap mode
fn apply_free_movement(
    cursor: &mut SoftwareCursor,
    input: Vec2,
    gamepad_settings: &GamepadCursor,
    config: &GamepadCursorConfig,
    time: &Time,
) {
    let speed_multiplier = config.fallback_speed;
    let x = input.x
        * gamepad_settings.speed
        * speed_multiplier
        * time.delta_secs()
        * config.movement_speed_multiplier;
    let y = input.y
        * gamepad_settings.speed
        * speed_multiplier
        * time.delta_secs()
        * config.movement_speed_multiplier;

    if x != 0.0 {
        cursor.location.x += x;
    }
    if y != 0.0 {
        cursor.location.y += y;
    }
}

/// This system will move the gamepad cursor.
fn system_cursor_gamepad_move(
    time: Res<Time>,
    config: Res<GamepadCursorConfig>,
    gamepads: Query<&Gamepad, With<GamepadAttachedCursor>>,
    mut cursors: Query<
        (&mut SoftwareCursor, &GamepadCursor, &GamepadAttachedCursor),
        Without<Gamepad>,
    >,
    ui_elements: Query<(&GlobalTransform, &Dimension), (With<Pickable>, Without<SoftwareCursor>)>,
) {
    for (mut cursor, gamepad_settings, attached_gamepad) in &mut cursors {
        if let Ok(gamepad) = gamepads.get(attached_gamepad.0) {
            // Get the gamepad input
            let mut input = Vec2::new(
                gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0),
                gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0),
            );

            // Clamp the deadzone as a vector
            if input.length_squared() < config.gamepad_deadzone.powi(2) {
                input *= 0.0;
            }

            match gamepad_settings.mode {
                GamepadCursorMode::Free => {
                    // Free movement mode - original behavior
                    let x = input.x
                        * gamepad_settings.speed
                        * time.delta_secs()
                        * config.movement_speed_multiplier;
                    let y = input.y
                        * gamepad_settings.speed
                        * time.delta_secs()
                        * config.movement_speed_multiplier;

                    // Move the cursor if it changed
                    if x != 0.0 {
                        cursor.location.x += x;
                    }
                    if y != 0.0 {
                        cursor.location.y += y;
                    }
                },
                GamepadCursorMode::Snap => {
                    // Production-quality snap mode using spatial navigation patterns
                    // Based on Bevy's DirectionalNavigationMap approach for optimal performance

                    if input.length() > config.snap_threshold {
                        match snap_to_nearest_element(
                            &cursor.location,
                            input,
                            &ui_elements,
                            &config,
                        ) {
                            Ok(target_pos) => {
                                // Move cursor towards target with configurable interpolation
                                let move_speed =
                                    gamepad_settings.speed * config.snap_speed * time.delta_secs();
                                let to_target = target_pos - cursor.location;

                                if to_target.length() <= move_speed {
                                    cursor.location = target_pos;
                                } else {
                                    cursor.location += to_target.normalize() * move_speed;
                                }
                            },
                            Err(e) => {
                                error!("Snap navigation failed: {}", e);
                                // Fallback to free movement on error
                                apply_free_movement(
                                    &mut cursor,
                                    input,
                                    gamepad_settings,
                                    &config,
                                    &time,
                                );
                            },
                        }
                    }
                },
            }
        }
    }
}

/// This system will move the mouse cursor.
fn system_cursor_mouse_move(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<&Projection>,
    mut query: Query<(&mut SoftwareCursor, Option<&ChildOf>), Without<GamepadCursor>>,
) {
    if let Ok(window) = windows.single() {
        for (mut cursor, parent_option) in &mut query {
            if let Some(position) = window.cursor_position() {
                // Get projection scale to account for zoomed cameras
                let scale = if let Some(parent) = parent_option {
                    if let Ok(Projection::Orthographic(projection)) = cameras.get(parent.parent()) {
                        projection.scale
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

                // Compute the cursor position
                let x = (position.x - window.width() * 0.5) * scale;
                let y = -((position.y - window.height() * 0.5) * scale);

                // Move the cursor if it changed
                if x != cursor.location.x {
                    cursor.location.x = x;
                }
                if y != cursor.location.y {
                    cursor.location.y = y;
                }
            }
        }
    }
}

/// This system will update the transform component to reflect the sprite offset.
fn system_cursor_update_transform(mut query: Query<(&SoftwareCursor, &mut Transform)>) {
    for (cursor, mut transform) in &mut query {
        let sprite_offset = cursor
            .cursor_atlas_map
            .get(&cursor.cursor_request)
            .unwrap_or(&(0, Vec2::ZERO))
            .1;
        transform.translation.x = cursor.location.x - sprite_offset.x * transform.scale.x;
        transform.translation.y = cursor.location.y + sprite_offset.y * transform.scale.y;
    }
}

/// This system will move the virtual pointer location.
fn system_cursor_move_pointer(
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut query: Query<(&mut PointerLocation, &SoftwareCursor)>,
) {
    if let Ok((win_entity, window)) = windows.single() {
        for (mut pointer, cursor) in query.iter_mut() {
            // Change the pointer location
            if let Some(normalized_target) =
                RenderTarget::Window(WindowRef::Primary).normalize(Some(win_entity))
            {
                pointer.location = Some(Location {
                    target: normalized_target,
                    position: Vec2 {
                        x: cursor.location.x + window.width() / 2.0,
                        y: -cursor.location.y + window.height() / 2.0,
                    }
                    .round(),
                });
            } else {
                error!("Failed to normalize render target for software cursor");
            }
        }
    }
}

/// This system will send out pointer move events if they changed position
fn system_cursor_send_move_events(
    mut cursor_last: Local<HashMap<PointerId, Vec2>>,
    pointers: Query<(&PointerId, &PointerLocation), With<SoftwareCursor>>,
    mut pointer_output: EventWriter<PointerInput>,
) {
    // Send mouse movement events
    for (pointer, location) in &pointers {
        if let Some(location) = &location.location {
            let last = cursor_last.get(pointer).unwrap_or(&Vec2::ZERO);
            if *last == location.position {
                continue;
            }

            pointer_output.write(PointerInput::new(
                *pointer,
                Location {
                    target: location.target.clone(),
                    position: location.position,
                },
                PointerAction::Move {
                    delta: location.position - *last,
                },
            ));
            cursor_last.insert(*pointer, location.position);
        }
    }
}

/// This system will send out mouse pick events
fn system_cursor_mouse_send_pick_events(
    pointers: Query<&PointerLocation, (With<SoftwareCursor>, Without<GamepadCursor>)>,
    mut mouse_inputs: EventReader<MouseButtonInput>,
    mut pointer_output: EventWriter<PointerInput>,
) {
    // Send mouse click events
    for location in &pointers {
        if let Some(location) = &location.location {
            // Send mouse click events
            for input in mouse_inputs.read() {
                // Which state to change
                match input.state {
                    ButtonState::Pressed => {
                        // Send out the event
                        pointer_output.write(PointerInput::new(
                            PointerId::Mouse,
                            Location {
                                target: location.target.clone(),
                                position: location.position,
                            },
                            PointerAction::Press(match input.button {
                                MouseButton::Left => PointerButton::Primary,
                                MouseButton::Right => PointerButton::Secondary,
                                MouseButton::Middle => PointerButton::Middle,
                                MouseButton::Other(_)
                                | MouseButton::Back
                                | MouseButton::Forward => continue,
                            }),
                        ));
                    },
                    ButtonState::Released => {
                        // Send out the event
                        pointer_output.write(PointerInput::new(
                            PointerId::Mouse,
                            Location {
                                target: location.target.clone(),
                                position: location.position,
                            },
                            PointerAction::Release(match input.button {
                                MouseButton::Left => PointerButton::Primary,
                                MouseButton::Right => PointerButton::Secondary,
                                MouseButton::Middle => PointerButton::Middle,
                                MouseButton::Other(_)
                                | MouseButton::Back
                                | MouseButton::Forward => continue,
                            }),
                        ));
                    },
                };
            }
        }
    }
}

/// This system will send out gamepad pick events
fn system_cursor_gamepad_send_pick_events(
    pointers: Query<
        (&PointerLocation, &PointerId, &GamepadAttachedCursor),
        (With<SoftwareCursor>, With<GamepadCursor>),
    >,
    mut gamepad_inputs: EventReader<GamepadButtonChangedEvent>,
    mut pointer_output: EventWriter<PointerInput>,
) {
    // Send gamepad click events
    for (location, pointer_id, attached_gamepad) in &pointers {
        if let Some(location) = &location.location {
            // Send gamepad click events filtered by specific gamepad
            for input in gamepad_inputs.read() {
                // Only process events from the gamepad attached to this cursor
                if input.entity != attached_gamepad.0 {
                    continue;
                }

                // Which state to change
                match input.state {
                    ButtonState::Pressed => {
                        // Send out the event
                        pointer_output.write(PointerInput::new(
                            *pointer_id,
                            Location {
                                target: location.target.clone(),
                                position: location.position,
                            },
                            PointerAction::Press(match input.button {
                                GamepadButton::South => PointerButton::Primary,
                                GamepadButton::East => PointerButton::Secondary,
                                GamepadButton::West => PointerButton::Middle,
                                _ => continue,
                            }),
                        ));
                    },
                    ButtonState::Released => {
                        // Send out the event
                        pointer_output.write(PointerInput::new(
                            *pointer_id,
                            Location {
                                target: location.target.clone(),
                                position: location.position,
                            },
                            PointerAction::Release(match input.button {
                                GamepadButton::South => PointerButton::Primary,
                                GamepadButton::East => PointerButton::Secondary,
                                GamepadButton::West => PointerButton::Middle,
                                _ => continue,
                            }),
                        ));
                    },
                };
            }
        }
    }
}

// #==============#
// #=== PLUGIN ===#

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add SoftwareCursor Icon Queue resource to the app
            .insert_resource(CursorIconQueue::default())
            // Add GamepadCursorConfig resource
            .init_resource::<GamepadCursorConfig>()
            .add_systems(
                PostUpdate,
                (
                    system_cursor_icon_queue_purge,
                    system_cursor_icon_queue_apply,
                ),
            )
            // OnHoverSetCursor observers
            .add_observer(observer_cursor_request_cursor_icon)
            .add_observer(observer_cursor_cancel_cursor_icon)
            // #=== SOFTWARE CURSOR ===#
            // Add systems for emulating picking events
            .add_systems(
                First,
                (
                    system_cursor_send_move_events,
                    system_cursor_mouse_send_pick_events,
                    system_cursor_gamepad_send_pick_events,
                    ApplyDeferred,
                )
                    .chain()
                    .in_set(PickSet::Input),
            )
            // Add core systems
            .add_systems(
                PreUpdate,
                (
                    system_cursor_gamepad_move,
                    system_cursor_mouse_move,
                    system_cursor_update_transform,
                    system_cursor_move_pointer,
                )
                    .chain(),
            )
            // Other stuff
            .add_systems(
                Update,
                (
                    system_cursor_hide_native,
                    system_cursor_software_change_icon,
                    system_cursor_gamepad_assign,
                ),
            );
    }
}

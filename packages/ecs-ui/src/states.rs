use crate::*;

// Exported prelude
pub mod prelude {
    // All standard exports
    pub use super::{
        UiClicked, UiHover, UiIntro, UiOutro, UiSelected, clicked_set, hover_set, selected_set,
    };
}

// #=======================#
// #=== THE HOVER STATE ===#

/// **Ui Hover** - A built in state that should be triggered manually when a pointer hovers over a
/// Ui-Node. This state first **needs to be enabled** for the entity by adding it as a component.
///
/// Then you can use the [`Self::id`] function to identify this state inside components
/// that allow you to specify per state properties like [`UiLayout`].
///
/// For more information check the documentation on [`UiState`].
///
/// ```
/// # use bevy_ecs::prelude::*; use bevy_asset::prelude::*; use bevy_picking::prelude::*; use bevy_color::prelude::*; use bevy_lunex::prelude::*; use bevy_text::prelude::*; use bevy_sprite::prelude::*; use bevy_color::palettes::basic::*; use bevy_math::prelude::*;
///      UiLayout::new(vec![
///          (UiBase::id(), UiLayout::window().full()),
///          (UiHover::id(), UiLayout::window().x(Rl(10.0)).full())
///      ]);
/// ```
///
/// To trigger the state we can either manually flip the [`UiHover::enable`] field or trigger the
/// [`UiHoverSet`] helper event. To do this easily, there is a convinient observer [`hover_set`]
/// provided for it.
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_asset::prelude::*; use bevy_picking::prelude::*; use bevy_color::prelude::*; use bevy_lunex::prelude::*; use bevy_text::prelude::*; use bevy_sprite::prelude::*; use bevy_color::palettes::basic::*; use bevy_math::prelude::*;
/// # fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
/// # commands.spawn((
/// #     UiLayoutRoot::new_2d(),
/// # )).with_children(|ui| {
///       ui.spawn((
///           // ... Layout, Color, etc.
///           UiHover::new().forward_speed(20.0).backward_speed(4.0),   // Enable the state
///
///       // Add the observers
///       )).observe(hover_set::<Pointer<Over>, true>)
///         .observe(hover_set::<Pointer<Out>, false>);
/// # });
/// # }
/// ```
#[derive(Component, Reflect, Clone, Debug)]
pub struct UiHover {
    value: f32,
    /// If the state is enabled
    pub enable: bool,
    /// The function to smooth the transition
    #[reflect(ignore, default = "default_linear_curve")]
    pub curve: fn(f32) -> f32,
    /// The speed of transition forwards
    pub forward_speed: f32,
    /// The speed of transition backwards
    pub backward_speed: f32,
    /// Enable to have instant state transition
    pub instant: bool,
}
/// Method implementations
impl UiHover {
    /// Create new instance
    pub fn new() -> Self {
        Self::default()
    }
    /// Replaces the curve function.
    pub fn curve(mut self, curve: fn(f32) -> f32) -> Self {
        self.curve = curve;
        self
    }
    /// Replaces the speed with a new value.
    pub fn forward_speed(mut self, forward_speed: f32) -> Self {
        self.forward_speed = forward_speed;
        self
    }
    /// Replaces the speed with a new value.
    pub fn backward_speed(mut self, backward_speed: f32) -> Self {
        self.backward_speed = backward_speed;
        self
    }
    /// Replaces the instant property with a new value.
    pub fn instant(mut self, instant: bool) -> Self {
        self.instant = instant;
        self
    }
}
/// Constructor
impl Default for UiHover {
    fn default() -> Self {
        Self {
            value: 0.0,
            enable: false,
            curve: |v| v,
            forward_speed: 1.0,
            backward_speed: 1.0,
            instant: false,
        }
    }
}
/// State implementation
impl UiStateTrait for UiHover {
    fn value(&self) -> f32 {
        (self.curve)(self.value)
    }
}

/// Custom PartialEq implementation that excludes function pointer comparison
impl PartialEq for UiHover {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.enable == other.enable
            && self.forward_speed == other.forward_speed
            && self.backward_speed == other.backward_speed
            && self.instant == other.instant
        // Note: curve function pointer intentionally excluded from comparison
    }
}

/// This system updates the hover transition value over time
pub fn system_state_hover_update(time: Res<Time>, mut query: Query<&mut UiHover>) {
    for mut hover in &mut query {
        if hover.enable && hover.value < 1.0 {
            if hover.instant {
                hover.value = 1.0;
                continue;
            }
            hover.value = (hover.value + hover.forward_speed * time.delta_secs()).min(1.0);
        }
        if !hover.enable && hover.value > 0.0 {
            if hover.instant {
                hover.value = 0.0;
                continue;
            }
            hover.value = (hover.value - hover.backward_speed * time.delta_secs()).max(0.0);
        }
    }
}

/// Event that enables the hover transition
#[derive(Event, Clone, Copy)]
pub struct UiHoverSet(pub bool);

/// This observer enables the hover transition on trigger
fn observer_state_hover_set(
    trigger: Trigger<UiHoverSet>,
    mut query: Query<&mut UiHover>,
    mut events: EventWriter<UiEvent>,
) {
    if let Ok(mut hover) = query.get_mut(trigger.target()) {
        hover.enable = trigger.event().0;
        if trigger.event().0 {
            events.write(UiEvent::Hover(trigger.target()));
        }
    }
}

/// Utility observer that triggers the [`UiHoverSet`] event on triggered event.
pub fn hover_set<E: Event, const BOOL: bool>(trigger: Trigger<E>, mut commands: Commands) {
    commands.trigger_targets(UiHoverSet(BOOL), trigger.target());
}

// #==========================#
// #=== THE SELECTED STATE ===#

/// **Ui Selected** - A built-in state for managing selection behavior.
/// Similar to UiHover but for selection state management.
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_lunex::prelude::*;
/// # fn spawn_ui(mut commands: Commands) {
/// commands.spawn((
///     UiSelected::new().forward_speed(10.0).backward_speed(5.0),
///     // ... other components
/// ));
/// # }
/// ```
#[derive(Component, Reflect, Clone, Debug)]
pub struct UiSelected {
    value: f32,
    /// If the state is enabled
    pub enable: bool,
    /// The function to smooth the transition
    #[reflect(ignore, default = "default_linear_curve")]
    pub curve: fn(f32) -> f32,
    /// The speed of transition forwards
    pub forward_speed: f32,
    /// The speed of transition backwards
    pub backward_speed: f32,
    /// Enable to have instant state transition
    pub instant: bool,
}

impl UiSelected {
    /// Create new instance
    pub fn new() -> Self {
        Self::default()
    }
    /// Replaces the curve function.
    pub fn curve(mut self, curve: fn(f32) -> f32) -> Self {
        self.curve = curve;
        self
    }
    /// Replaces the forward speed with a new value.
    pub fn forward_speed(mut self, forward_speed: f32) -> Self {
        self.forward_speed = forward_speed;
        self
    }
    /// Replaces the backward speed with a new value.
    pub fn backward_speed(mut self, backward_speed: f32) -> Self {
        self.backward_speed = backward_speed;
        self
    }
    /// Replaces the instant property with a new value.
    pub fn instant(mut self, instant: bool) -> Self {
        self.instant = instant;
        self
    }
}

impl Default for UiSelected {
    fn default() -> Self {
        Self {
            value: 0.0,
            enable: false,
            curve: |v| v,
            forward_speed: 1.0,
            backward_speed: 1.0,
            instant: false,
        }
    }
}

impl UiStateTrait for UiSelected {
    fn value(&self) -> f32 {
        (self.curve)(self.value)
    }
}

/// Custom PartialEq implementation that excludes function pointer comparison
impl PartialEq for UiSelected {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.enable == other.enable
            && self.forward_speed == other.forward_speed
            && self.backward_speed == other.backward_speed
            && self.instant == other.instant
        // Note: curve function pointer intentionally excluded from comparison
    }
}

// #=========================#
// #=== THE CLICKED STATE ===#

/// **Ui Clicked** - A built-in state for managing click behavior.
/// Activated when an element is clicked and held.
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_lunex::prelude::*;
/// # fn spawn_ui(mut commands: Commands) {
/// commands.spawn((
///     UiClicked::new().forward_speed(15.0).backward_speed(8.0),
///     // ... other components
/// ));
/// # }
/// ```
#[derive(Component, Reflect, Clone, Debug)]
pub struct UiClicked {
    pub value: f32,
    /// If the state is enabled
    pub enable: bool,
    /// The function to smooth the transition
    #[reflect(ignore, default = "default_linear_curve")]
    pub curve: fn(f32) -> f32,
    /// The speed of transition forwards
    pub forward_speed: f32,
    /// The speed of transition backwards
    pub backward_speed: f32,
    /// Enable to have instant state transition
    pub instant: bool,
}

impl UiClicked {
    /// Create new instance
    pub fn new() -> Self {
        Self::default()
    }
    /// Replaces the curve function.
    pub fn curve(mut self, curve: fn(f32) -> f32) -> Self {
        self.curve = curve;
        self
    }
    /// Replaces the forward speed with a new value.
    pub fn forward_speed(mut self, forward_speed: f32) -> Self {
        self.forward_speed = forward_speed;
        self
    }
    /// Replaces the backward speed with a new value.
    pub fn backward_speed(mut self, backward_speed: f32) -> Self {
        self.backward_speed = backward_speed;
        self
    }
    /// Replaces the instant property with a new value.
    pub fn instant(mut self, instant: bool) -> Self {
        self.instant = instant;
        self
    }
}

impl Default for UiClicked {
    fn default() -> Self {
        Self {
            value: 0.0,
            enable: false,
            curve: |v| v,
            forward_speed: 1.0,
            backward_speed: 1.0,
            instant: false,
        }
    }
}

impl UiStateTrait for UiClicked {
    fn value(&self) -> f32 {
        (self.curve)(self.value)
    }
}

/// Custom PartialEq implementation that excludes function pointer comparison
impl PartialEq for UiClicked {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.enable == other.enable
            && self.forward_speed == other.forward_speed
            && self.backward_speed == other.backward_speed
            && self.instant == other.instant
        // Note: curve function pointer intentionally excluded from comparison
    }
}

// #=======================#
// #=== THE INTRO STATE ===#

/// **Ui Intro** - A built-in state for intro animations.
/// Automatically progresses from 0 to 1 over a specified duration when spawned.
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_lunex::prelude::*;
/// # fn spawn_ui(mut commands: Commands) {
/// commands.spawn((
///     UiIntro::new().duration(2.0).curve(|t| t * t), /* 2 second quadratic ease
///                                                     * ... other components */
/// ));
/// # }
/// ```
#[derive(Component, Reflect, Clone, Debug)]
pub struct UiIntro {
    value: f32,
    /// The function to smooth the transition
    #[reflect(ignore, default = "default_linear_curve")]
    pub curve: fn(f32) -> f32,
    /// Duration of the intro animation in seconds
    pub duration: f32,
    /// Timer tracking the current progress
    timer: f32,
}

impl UiIntro {
    /// Create new instance
    pub fn new() -> Self {
        Self::default()
    }
    /// Replaces the curve function.
    pub fn curve(mut self, curve: fn(f32) -> f32) -> Self {
        self.curve = curve;
        self
    }
    /// Sets the duration of the intro animation.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Internal method to update the intro progress
    pub fn update(&mut self, delta_time: f32) {
        if self.timer < self.duration {
            self.timer += delta_time;
            self.value = (self.timer / self.duration).min(1.0);
        }
    }

    /// Check if the intro animation is complete
    pub fn is_complete(&self) -> bool {
        self.timer >= self.duration
    }
}

impl Default for UiIntro {
    fn default() -> Self {
        Self {
            value: 0.0,
            curve: |v| v,
            duration: 1.0,
            timer: 0.0,
        }
    }
}

impl UiStateTrait for UiIntro {
    fn value(&self) -> f32 {
        (self.curve)(self.value)
    }
}

/// Custom PartialEq implementation that excludes function pointer comparison
impl PartialEq for UiIntro {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.duration == other.duration && self.timer == other.timer
        // Note: curve function pointer intentionally excluded from comparison
    }
}

// #=======================#
// #=== THE OUTRO STATE ===#

/// **Ui Outro** - A built-in state for outro animations.
/// Progresses from 0 to 1 over a specified duration when triggered.
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_lunex::prelude::*;
/// # fn spawn_ui(mut commands: Commands) {
/// commands.spawn((
///     UiOutro::new()
///         .duration(1.5)
///         .curve(|t| 1.0 - (1.0 - t) * (1.0 - t)), /* Ease out
///                                                   * ... other components */
/// ));
/// # }
/// ```
#[derive(Component, Reflect, Clone, Debug)]
pub struct UiOutro {
    value: f32,
    /// Whether the outro animation is active
    pub enable: bool,
    /// The function to smooth the transition
    #[reflect(ignore, default = "default_linear_curve")]
    pub curve: fn(f32) -> f32,
    /// Duration of the outro animation in seconds
    pub duration: f32,
    /// Timer tracking the current progress
    timer: f32,
}

impl UiOutro {
    /// Create new instance
    pub fn new() -> Self {
        Self::default()
    }
    /// Replaces the curve function.
    pub fn curve(mut self, curve: fn(f32) -> f32) -> Self {
        self.curve = curve;
        self
    }
    /// Sets the duration of the outro animation.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }
    /// Trigger the outro animation
    pub fn trigger(&mut self) {
        self.enable = true;
        self.timer = 0.0;
        self.value = 0.0;
    }

    /// Internal method to update the outro progress
    pub fn update(&mut self, delta_time: f32) {
        if self.enable && self.timer < self.duration {
            self.timer += delta_time;
            self.value = (self.timer / self.duration).min(1.0);
        }
    }

    /// Check if the outro animation is complete
    pub fn is_complete(&self) -> bool {
        self.enable && self.timer >= self.duration
    }
}

impl Default for UiOutro {
    fn default() -> Self {
        Self {
            value: 0.0,
            enable: false,
            curve: |v| v,
            duration: 1.0,
            timer: 0.0,
        }
    }
}

impl UiStateTrait for UiOutro {
    fn value(&self) -> f32 {
        (self.curve)(self.value)
    }
}

/// Custom PartialEq implementation that excludes function pointer comparison
impl PartialEq for UiOutro {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.enable == other.enable
            && self.duration == other.duration
            && self.timer == other.timer
        // Note: curve function pointer intentionally excluded from comparison
    }
}

// #======================#
// #=== STATE SYSTEMS ===#

/// System to update UiSelected state transitions
pub fn system_state_selected_update(time: Res<Time>, mut query: Query<&mut UiSelected>) {
    for mut selected in &mut query {
        let target_value = if selected.enable { 1.0 } else { 0.0 };
        let speed = if selected.enable {
            selected.forward_speed
        } else {
            selected.backward_speed
        };

        if selected.instant {
            selected.value = target_value;
        } else if selected.value != target_value {
            let delta = (target_value - selected.value).signum() * speed * time.delta_secs();
            selected.value = (selected.value + delta).clamp(0.0, 1.0);
        }
    }
}

/// System to update UiClicked state transitions
pub fn system_state_clicked_update(time: Res<Time>, mut query: Query<&mut UiClicked>) {
    for mut clicked in &mut query {
        let target_value = if clicked.enable { 1.0 } else { 0.0 };
        let speed = if clicked.enable {
            clicked.forward_speed
        } else {
            clicked.backward_speed
        };

        if clicked.instant {
            clicked.value = target_value;
        } else if clicked.value != target_value {
            let delta = (target_value - clicked.value).signum() * speed * time.delta_secs();
            clicked.value = (clicked.value + delta).clamp(0.0, 1.0);
        }
    }
}

/// System to update UiIntro state progress
pub fn system_state_intro_update(
    time: Res<Time>,
    mut query: Query<&mut UiIntro>,
    mut commands: Commands,
) {
    for mut intro in &mut query {
        if !intro.is_complete() {
            intro.update(time.delta_secs());
            // Trigger layout recompute to reflect animation progress
            commands.trigger(RecomputeUiLayout);
        }
    }
}

/// System to update UiOutro state progress
pub fn system_state_outro_update(
    time: Res<Time>,
    mut query: Query<&mut UiOutro>,
    mut commands: Commands,
) {
    for mut outro in &mut query {
        if outro.enable && !outro.is_complete() {
            outro.update(time.delta_secs());
            // Trigger layout recompute to reflect animation progress
            commands.trigger(RecomputeUiLayout);
        }
    }
}

// #========================#
// #=== EVENT STRUCTURES ===#

/// Event structure for UiSelected state changes
#[derive(Event, Clone, Copy)]
pub struct UiSelectedSet(pub bool);

/// Event structure for UiClicked state changes
#[derive(Event, Clone, Copy)]
pub struct UiClickedSet(pub bool);

// #======================#
// #=== EVENT OBSERVERS ===#

/// Observer for selected state changes
fn observer_state_selected_set(
    trigger: Trigger<UiSelectedSet>,
    mut query: Query<&mut UiSelected>,
    mut events: EventWriter<UiEvent>,
) {
    if let Ok(mut selected) = query.get_mut(trigger.target()) {
        selected.enable = trigger.event().0;
        if trigger.event().0 {
            events.write(UiEvent::Select(trigger.target()));
        }
    }
}

/// Observer for clicked state changes
fn observer_state_clicked_set(
    trigger: Trigger<UiClickedSet>,
    mut query: Query<&mut UiClicked>,
    mut events: EventWriter<UiEvent>,
) {
    if let Ok(mut clicked) = query.get_mut(trigger.target()) {
        clicked.enable = trigger.event().0;
        if trigger.event().0 {
            events.write(UiEvent::Click(trigger.target()));
        }
    }
}

// #=======================#
// #=== HELPER FUNCTIONS ===#

/// Helper observer for setting selected state
pub fn selected_set<E: Event, const BOOL: bool>(trigger: Trigger<E>, mut commands: Commands) {
    commands.trigger_targets(UiSelectedSet(BOOL), trigger.target());
}

/// Helper observer for setting clicked state
pub fn clicked_set<E: Event, const BOOL: bool>(trigger: Trigger<E>, mut commands: Commands) {
    commands.trigger_targets(UiClickedSet(BOOL), trigger.target());
}

// #==================================#
// #=== AUTO-OBSERVER INTEGRATION ===#

/// System that automatically adds UiHover to entities with Pickable but without UiHover
pub fn system_auto_add_hover_to_pickable(
    query: Query<Entity, (With<Pickable>, Without<UiHover>, Added<Pickable>)>,
    mut commands: Commands,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(UiHover::new().forward_speed(6.0).backward_speed(4.0))
            .observe(hover_set::<Pointer<Over>, true>)
            .observe(hover_set::<Pointer<Out>, false>);
    }
}

/// System that automatically adds UiClicked to entities with Pickable and UiHover but without
/// UiClicked
pub fn system_auto_add_clicked_to_interactive(
    query: Query<
        Entity,
        (
            With<Pickable>,
            With<UiHover>,
            Without<UiClicked>,
            Added<UiHover>,
        ),
    >,
    mut commands: Commands,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(UiClicked::new().forward_speed(12.0).backward_speed(8.0))
            .observe(clicked_set::<Pointer<Click>, true>)
            .observe(clicked_set::<Pointer<Click>, false>);
    }
}

// #========================#
// #=== THE STATE PLUGIN ===#

/// Default linear curve used for reflection defaults
pub fn default_linear_curve() -> fn(f32) -> f32 {
    |v| v
}

/// This observer will listen for said event and duplicate it to it's children
fn observer_event_duplicator<E: Event + Copy>(
    trigger: Trigger<E>,
    mut commands: Commands,
    mut query: Query<&Children>,
) {
    if let Ok(children) = query.get_mut(trigger.target()) {
        let targets: Vec<Entity> = children.iter().collect();
        commands.trigger_targets(*trigger.event(), targets);
    }
}

/// This plugin is used for the main logic.
pub struct UiLunexStatePlugin;
impl Plugin for UiLunexStatePlugin {
    fn build(&self, app: &mut App) {
        // Add events
        app.add_event::<UiSelectedSet>().add_event::<UiClickedSet>();

        // Add observers
        app.add_observer(observer_state_hover_set)
            .add_observer(observer_state_selected_set)
            .add_observer(observer_state_clicked_set);

        // Add event child duplication
        app.add_observer(observer_event_duplicator::<UiHoverSet>)
            .add_observer(observer_event_duplicator::<UiSelectedSet>)
            .add_observer(observer_event_duplicator::<UiClickedSet>);

        // PRE-COMPUTE SYSTEMS
        app.add_systems(
            Update,
            (
                system_state_hover_update,
                system_state_selected_update,
                system_state_clicked_update,
                system_state_intro_update,
                system_state_outro_update,
                system_state_pipe_into_manager::<UiHover>,
                system_state_pipe_into_manager::<UiSelected>,
                system_state_pipe_into_manager::<UiClicked>,
                system_state_pipe_into_manager::<UiIntro>,
                system_state_pipe_into_manager::<UiOutro>,
                // Auto-observer systems
                system_auto_add_hover_to_pickable,
                system_auto_add_clicked_to_interactive,
            )
                .in_set(UiSystems::PreCompute),
        );
    }
}

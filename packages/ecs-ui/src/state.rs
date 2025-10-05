//! UI state management system for the Lunex UI system

use std::any::TypeId;
use std::collections::HashMap;

use crate::{RecomputeUiLayout, *};

/// **Ui State** - This component aggrages state transition values for later reference
/// by other components. You don't directly control or spawn this component, but use an abstraction
/// instead. You can use the prebuilt state components or create a custom ones with a completely
/// unique transition logic. You just have to provide transition value to this component later.
/// - [`UiBase`] _(Type only, not a component)_
/// - [`UiHover`]
/// - [`UiSelected`]
/// - [`UiClicked`]
/// - [`UiIntro`]
/// - [`UiOutro`]
///
/// Dependant components:
/// - [`UiLayout`]
/// - [`UiColor`]
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_asset::prelude::*; use bevy_picking::prelude::*; use bevy_color::prelude::*; use bevy_lunex::prelude::*; use bevy_text::prelude::*; use bevy_sprite::prelude::*; use bevy_color::palettes::basic::*; use bevy_math::prelude::*;
/// # fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
/// # commands.spawn((
/// #     UiLayoutRoot::new_2d(),
/// # )).with_children(|ui| {
///       ui.spawn((
///           // Like this you can enable a state
///           UiHover::new().forward_speed(20.0).backward_speed(4.0),
///           // You can define layouts per state
///           UiLayout::new(vec![
///               (UiBase::id(), UiLayout::window().full()),
///               (UiHover::id(), UiLayout::window().x(Rl(10.0)).full())
///           ]),
///           // You can define colors per state
///           UiColor::new(vec![
///               (UiBase::id(), Color::Srgba(RED).with_alpha(0.8)),
///               (UiHover::id(), Color::Srgba(YELLOW).with_alpha(1.2))
///           ]),
///           // ... Sprite, Text, etc.
///
///       // Add observers that enable/disable the hover state component
///       )).observe(hover_set::<Pointer<Over>, true>)
///         .observe(hover_set::<Pointer<Out>, false>);
/// # });
/// # }
/// ```
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
pub struct UiState {
    /// Stored transition per state
    pub states: HashMap<TypeId, f32>,
}
/// Default constructor
impl Default for UiState {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(UiBase::id(), 1.0);
        Self { states: map }
    }
}

/// **Ui Base** - The default state for a Ui-Node, used only for the [`UiBase::id`] key. It is not a
/// component that you can control.
#[derive(Clone, PartialEq, Debug)]
pub struct UiBase;
impl UiStateTrait for UiBase {
    fn value(&self) -> f32 {
        1.0
    }
}

/// Trait that all states must implement before being integrated into the state machine.
pub trait UiStateTrait: Send + Sync + 'static {
    /// This is used as a key to identify a Ui-Node state.
    fn id() -> TypeId {
        TypeId::of::<Self>()
    }
    /// This must return a value between `0.0 - 1.0`. It is used as transition value
    /// for a state, with `0.0` being off and `1.0` being on. Any smoothing should happen
    /// inside this function.
    fn value(&self) -> f32;
}

/// This system controls the [`UiBase`] state. This state is decreased based on total sum of all
/// other active states.
pub fn system_state_base_balancer(mut query: Query<&mut UiState, Changed<UiState>>) {
    for mut manager in &mut query {
        // Normalize the active nobase state weights
        let mut total_nonbase_weight = 0.0;
        for (state, value) in &manager.states {
            if *state == UiBase::id() {
                continue;
            }
            total_nonbase_weight += value;
        }

        // Decrease base transition based on other states
        if let Some(value) = manager.states.get_mut(&UiBase::id()) {
            *value = (1.0 - total_nonbase_weight).clamp(0.0, 1.0);
        }
    }
}
/// This system pipes the attached state component data to the [`UiState`] component.
pub fn system_state_pipe_into_manager<S: UiStateTrait + Component>(
    mut commands: Commands,
    mut query: Query<(&mut UiState, &S), Changed<S>>,
) {
    for (mut manager, state) in &mut query {
        // Send the value to the manager
        if let Some(value) = manager.states.get_mut(&S::id()) {
            *value = state.value();

        // Insert the value if it does not exist
        } else {
            manager.states.insert(S::id(), state.value());
        }
        // Recompute layout
        commands.trigger(RecomputeUiLayout);
    }
}

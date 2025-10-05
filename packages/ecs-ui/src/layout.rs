//! UI layout system components and computation for the Lunex UI system

use std::any::TypeId;
use std::collections::HashMap;

use crate::layouts::{UiLayoutType, UiLayoutTypeBoundary, UiLayoutTypeSolid, UiLayoutTypeWindow};
use crate::{
    Dimension, DirtyLayout, LayoutCache, RecomputeUiLayout, Rectangle2D, UiBase, UiDepth, UiState,
    *,
};

/// **Ui Layout Root** - This component marks the start of a worldspace Ui-Tree. Spawn this
/// standalone for worldspace 3D UI or spawn this as a child of camera for a HUD. For 2D UI, if your
/// camera does not move you can spawn it standalone too.
///
/// Important components:
/// - [`Transform`] - Set the position of the Ui-Tree
/// - [`Dimension`] - Set the size of the Ui-Tree
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_asset::prelude::*; use bevy_lunex::prelude::*;
/// # fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
/// commands
///     .spawn((
///         UiLayoutRoot::new_2d(),
///         UiFetchFromCamera::<0>, // Pipe the size from Camera
///     ))
///     .with_children(|ui| {
///         // ... spawn your Ui Here
///     });
/// # }
/// ```
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
#[require(Visibility, Transform, Dimension, VisibilityClass)]
#[component(on_add = view::add_visibility_class::<UiLayoutRoot>)]
pub struct UiLayoutRoot {
    abs_scale: f32,
}
impl UiLayoutRoot {
    pub fn new_2d() -> Self {
        Self { abs_scale: 1.0 }
    }
    pub fn new_3d() -> Self {
        Self { abs_scale: 0.001 }
    }
}

/// **Ui Root 3d** - This is a marker component for all entities which fall under a 3D UI. You can
/// check through this component if a specific node is 2D or 3D without looking for its root.
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
pub struct UiRoot3d;

/// **Ui Layout** - This component specifies the layout of a Ui-Node, which must be spawned as a
/// child of either [`UiLayoutRoot`] or [`UiLayout`] to work. Based on the provided layout other
/// attached components on this entity are overwritten to match the computed structure.
///
/// Direct output components:
/// - [`Transform`] - The computed position of the Ui-Node _(Read-only)_
/// - [`Dimension`] - The computed size of the Ui-Node _(Read-only)_
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_asset::prelude::*; use bevy_picking::prelude::*; use bevy_color::prelude::*; use bevy_lunex::prelude::*; use bevy_text::prelude::*; use bevy_sprite::prelude::*; use bevy_color::palettes::basic::*; use bevy_math::prelude::*;
/// # fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
/// # commands.spawn((
/// #     UiLayoutRoot::new_2d(),
/// # )).with_children(|ui| {
///       // Must be spawned as a child
///       ui.spawn((
///           // Use 1 of the 3 available layout types
///           UiLayout::solid().size((1920.0, 1080.0)).scaling(Scaling::Fill).pack(),
///           // Attach image to the node
///           Sprite::from_image(asset_server.load("images/ui/background.png")),
///       ));
/// # });
/// # }
/// ```
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
#[require(Visibility, Transform, Dimension, VisibilityClass, UiState, UiDepth)]
#[component(on_add = view::add_visibility_class::<UiLayout>)]
pub struct UiLayout {
    /// Stored layout per state
    pub layouts: HashMap<TypeId, UiLayoutType>,
}
/// Constructors
impl UiLayout {
    /// **Boundary** - Declarative layout type that is defined by its top-left corner and
    /// bottom-right corner. Nodes with this layout are not included in the ui flow.
    /// ## 🛠️ Example
    /// ```
    /// # use bevy_lunex::{UiLayout, Rl};
    /// let layout: UiLayout = UiLayout::boundary().pos1(Rl(20.0)).pos2(Rl(80.0)).pack();
    /// ```
    pub fn boundary() -> UiLayoutTypeBoundary {
        UiLayoutTypeBoundary::new()
    }
    /// **Window** - Declarative layout type that is defined by its size and position.
    /// Nodes with this layout are not included in the ui flow.
    /// ## 🛠️ Example
    /// ```
    /// # use bevy_lunex::{UiLayout, Ab, Rl};
    /// let layout: UiLayout = UiLayout::window().pos(Ab(100.0)).size(Rl(50.0)).pack();
    /// ```
    pub fn window() -> UiLayoutTypeWindow {
        UiLayoutTypeWindow::new()
    }
    /// **Solid** - Declarative layout type that is defined by its width and height ratio.
    /// Scales in a way to fit itself inside parent container. It never deforms.
    /// Nodes with this layout are not included in the ui flow.
    /// ## 🛠️ Example
    /// ```
    /// # use bevy_lunex::UiLayout;
    /// let layout: UiLayout = UiLayout::solid().size((4.0, 3.0)).align_x(-0.8).pack();
    /// ```
    pub fn solid() -> UiLayoutTypeSolid {
        UiLayoutTypeSolid::new()
    }
    /// Create multiple layouts for a different states at once.
    pub fn new(value: Vec<(TypeId, impl Into<UiLayoutType>)>) -> Self {
        let mut map = HashMap::new();
        for (state, layout) in value {
            map.insert(state, layout.into());
        }
        Self { layouts: map }
    }
    /// Try to return a reference to a stored layout
    pub fn get_boundary(&self, id: TypeId) -> Option<&UiLayoutTypeBoundary> {
        let UiLayoutType::Boundary(boundary) = self.layouts.get(&id)? else {
            return None;
        };
        Some(boundary)
    }
    /// Try to return a mut reference to a stored layout
    pub fn get_mut_boundary(&mut self, id: TypeId) -> Option<&mut UiLayoutTypeBoundary> {
        let UiLayoutType::Boundary(boundary) = self.layouts.get_mut(&id)? else {
            return None;
        };
        Some(boundary)
    }
    /// Try to return a reference to a stored layout
    pub fn get_window(&self, id: TypeId) -> Option<&UiLayoutTypeWindow> {
        let UiLayoutType::Window(window) = self.layouts.get(&id)? else {
            return None;
        };
        Some(window)
    }
    /// Try to return a mut reference to a stored layout
    pub fn get_mut_window(&mut self, id: TypeId) -> Option<&mut UiLayoutTypeWindow> {
        let UiLayoutType::Window(window) = self.layouts.get_mut(&id)? else {
            return None;
        };
        Some(window)
    }
    /// Try to return a reference to a stored layout
    pub fn get_solid(&self, id: TypeId) -> Option<&UiLayoutTypeSolid> {
        let UiLayoutType::Solid(solid) = self.layouts.get(&id)? else {
            return None;
        };
        Some(solid)
    }
    /// Try to return a mut reference to a stored layout
    pub fn get_mut_solid(&mut self, id: TypeId) -> Option<&mut UiLayoutTypeSolid> {
        let UiLayoutType::Solid(solid) = self.layouts.get_mut(&id)? else {
            return None;
        };
        Some(solid)
    }
}
/// Conversion implementations
impl From<UiLayoutType> for UiLayout {
    fn from(value: UiLayoutType) -> Self {
        let mut map = HashMap::new();
        map.insert(UiBase::id(), value);
        Self { layouts: map }
    }
}
impl From<UiLayoutTypeBoundary> for UiLayout {
    fn from(value: UiLayoutTypeBoundary) -> Self {
        let value: UiLayoutType = value.into();
        UiLayout::from(value)
    }
}
impl From<UiLayoutTypeWindow> for UiLayout {
    fn from(value: UiLayoutTypeWindow) -> Self {
        let value: UiLayoutType = value.into();
        UiLayout::from(value)
    }
}
impl From<UiLayoutTypeSolid> for UiLayout {
    fn from(value: UiLayoutTypeSolid) -> Self {
        let value: UiLayoutType = value.into();
        UiLayout::from(value)
    }
}

pub fn system_recompute_on_change<C: Component>(
    query: Query<Entity, Changed<C>>,
    mut commands: Commands,
) {
    if !query.is_empty() {
        commands.trigger(RecomputeUiLayout);
    }
}

/// This observer will mutably touch [`UiLayoutRoot`] which will trigger [`system_layout_compute`].
pub fn observer_touch_layout_root(
    _trigger: Trigger<RecomputeUiLayout>,
    mut query: Query<&mut UiLayoutRoot>,
) {
    for mut root in &mut query {
        root.as_mut();
    }
}

/// System to mark layout roots as dirty when they change
pub fn system_mark_layout_dirty(
    query: Query<
        Entity,
        (
            With<UiLayoutRoot>,
            Or<(
                Changed<UiLayoutRoot>,
                Changed<Dimension>,
                Changed<Transform>,
            )>,
        ),
    >,
    mut dirty_layout: ResMut<DirtyLayout>,
) {
    for entity in query.iter() {
        dirty_layout.dirty_roots.insert(entity);
    }
}

/// Optimized system that combines layout computation and 3D marking in a single traversal with
/// caching
pub fn system_layout_compute_and_mark_3d(
    mut commands: Commands,
    mut dirty_layout: ResMut<DirtyLayout>,
    mut layout_cache: ResMut<LayoutCache>,
    root_query: Query<
        (
            Entity,
            &UiLayoutRoot,
            &Transform,
            &Dimension,
            Has<UiRoot3d>,
            &Children,
        ),
        (
            Without<UiLayout>,
            Or<(Changed<UiLayoutRoot>, Changed<Dimension>)>,
        ),
    >,
    mut node_query: Query<
        (
            Entity,
            &UiLayout,
            &UiDepth,
            &UiState,
            &mut Transform,
            &mut Dimension,
            Has<UiRoot3d>,
            Option<&Children>,
        ),
        Without<UiLayoutRoot>,
    >,
) {
    // Process only dirty roots or mark all roots as dirty if needed
    let dirty_roots: Vec<Entity> = if dirty_layout.dirty_roots.is_empty() {
        root_query.iter().map(|(entity, ..)| entity).collect()
    } else {
        dirty_layout.dirty_roots.drain().collect()
    };

    for root_entity in dirty_roots {
        if let Ok((_, root, root_transform, root_dimension, is_root_3d, root_children)) =
            root_query.get(root_entity)
        {
            // Size of the viewport
            let root_rectangle = Rectangle2D {
                pos: root_transform.translation.xy(),
                size: **root_dimension,
            };
            layout_cache.cache.insert(root_entity, root_rectangle);

            // Stack-based traversal with 3D information
            let mut stack: Vec<(Entity, Rectangle2D, f32, bool)> = root_children
                .iter()
                .map(|child| (child, root_rectangle, 0.0, is_root_3d))
                .rev()
                .collect();

            while let Some((current_entity, parent_rectangle, depth, should_be_3d)) = stack.pop() {
                if let Ok((
                    entity,
                    node_layout,
                    node_depth,
                    node_state,
                    mut node_transform,
                    mut node_dimension,
                    is_node_3d,
                    node_children_option,
                )) = node_query.get_mut(current_entity)
                {
                    // Handle 3D marking
                    if should_be_3d != is_node_3d {
                        if should_be_3d {
                            commands.entity(entity).insert(UiRoot3d);
                        } else {
                            commands.entity(entity).remove::<UiRoot3d>();
                        }
                    }

                    // Compute all layouts for the node
                    let mut computed_rectangles = Vec::with_capacity(node_layout.layouts.len());
                    for (state, layout) in &node_layout.layouts {
                        computed_rectangles.push((
                            state,
                            layout.compute(
                                &parent_rectangle,
                                root.abs_scale,
                                root_rectangle.size,
                                16.0,
                            ),
                        ));
                    }

                    // Normalize the active state weights
                    let mut total_weight = 0.0;
                    for state in node_layout.layouts.keys() {
                        if let Some(weight) = node_state.states.get(state) {
                            total_weight += weight;
                        }
                    }

                    // Combine the state rectangles into one normalized
                    let mut node_rectangle = Rectangle2D::EMPTY;

                    // Use base if no active state
                    if total_weight == 0.0 {
                        node_rectangle.pos += computed_rectangles[0].1.pos;
                        node_rectangle.size += computed_rectangles[0].1.size;
                    // Combine the active states into one rectangle
                    } else {
                        for (state, rectangle) in computed_rectangles {
                            if let Some(weight) = node_state.states.get(state) {
                                node_rectangle.pos += rectangle.pos * (weight / total_weight);
                                node_rectangle.size += rectangle.size * (weight / total_weight);
                            }
                        }
                    }

                    // Save the computed layout
                    layout_cache.cache.insert(entity, node_rectangle);
                    node_transform.translation.x = node_rectangle.pos.x;
                    node_transform.translation.y = -node_rectangle.pos.y;

                    let new_depth = match node_depth {
                        UiDepth::Add(v) => depth + v,
                        UiDepth::Set(v) => *v,
                    };

                    node_transform.translation.z = new_depth * root.abs_scale;
                    **node_dimension = node_rectangle.size;

                    if let Some(node_children) = node_children_option {
                        // Add children to the stack with proper 3D state
                        stack.extend(
                            node_children
                                .iter()
                                .map(|child| (child, node_rectangle, new_depth, should_be_3d)),
                        );
                    }
                }
            }
        }
    }
}

//! Core UI component types for the Lunex UI system

use crate::units::UiValue;
use crate::*;

/// **Dimension** - This component holds width and height used for different Ui components
#[derive(Component, Reflect, Deref, DerefMut, Default, Clone, PartialEq, Debug)]
pub struct Dimension(pub Vec2);
/// Conversion implementations
impl<T: Into<Vec2>> From<T> for Dimension {
    fn from(value: T) -> Self {
        Dimension(value.into())
    }
}

/// **Ui Embedding** - Use this component to mark entities whose texture handles are embeddings
/// instead of regular assets. This means Lunex will resize the actual texture source when
/// [`Dimension`] has changed.
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
pub struct UiEmbedding;

/// **Ui Depth** - This component overrides the default Z axis (depth) stacking order.
/// This is useful when fixing Z order flickering. Another use can be offseting an background
/// image for example.
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
pub enum UiDepth {
    /// Add to existing depth
    Add(f32),
    /// Override existing depth
    Set(f32),
}
impl Default for UiDepth {
    fn default() -> Self {
        UiDepth::Add(1.0)
    }
}

/// **Ui Image Size** - This component makes image size the authority instead.
#[derive(Component, Reflect, Deref, DerefMut, Default, Clone, PartialEq, Debug)]
pub struct UiImageSize(pub UiValue<Vec2>);
/// Constructors
impl<T: Into<UiValue<Vec2>>> From<T> for UiImageSize {
    fn from(value: T) -> Self {
        UiImageSize(value.into())
    }
}

/// **Ui Text Size** - This component is used to control the size of a text compared
/// to other Ui-Nodes. It works by overwritting the attached [`UiLayout`] window
/// size parameter to match the text bounds. The value provided is used as a _scale_
/// to adjust this size, specificaly it's height. It is recommended to use `non-relative`
/// units such as [`Ab`], [`Rw`], [`Rh`], [`Vh`], [`Vw`] and [`Em`] for even values.
///
/// Affected components:
/// - [`UiLayout`] - **MUST BE WINDOW TYPE** for this to work
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_asset::prelude::*; use bevy_picking::prelude::*; use bevy_color::prelude::*; use bevy_lunex::prelude::*; use bevy_text::prelude::*; use bevy_sprite::prelude::*; use bevy_color::palettes::basic::*; use bevy_math::prelude::*;
/// # fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
/// # commands.spawn((
/// #     UiLayoutRoot::new_2d(),
/// # )).with_children(|ui| {
///       ui.spawn((
///           // Position the text using the window layout's position and anchor
///           UiLayout::window().pos((Rh(40.0), Rl(50.0))).anchor(Anchor::CenterLeft).pack(),
///           // This controls the height of the text, so 60% of the parent's node height
///           UiTextSize::from(Rh(60.0)),
///           // You can attach text like this
///           Text2d::new("Button"),
///           // Font size now works as "text resolution"
///           TextFont {
///               font: asset_server.load("fonts/Rajdhani.ttf"),
///               font_size: 64.0,
///               ..Default::default()
///           },
///       ));
/// # });
/// # }
/// ```
#[derive(Component, Reflect, Deref, DerefMut, Default, Clone, PartialEq, Debug)]
pub struct UiTextSize(pub UiValue<f32>);
/// Constructors
impl<T: Into<UiValue<f32>>> From<T> for UiTextSize {
    fn from(value: T) -> Self {
        UiTextSize(value.into())
    }
}

/// **Ui Mesh Plane 3d** - This component is used to mark mesh entities that can be freely replaced
/// with quad mesh on demand.
#[derive(Component, Reflect, Default, Clone, PartialEq, Debug)]
#[require(Mesh3d)]
pub struct UiMeshPlane3d;

/// **Ui Mesh Plane 2d** - This component is used to mark mesh entities that can be freely replaced
/// with quad mesh on demand.
#[derive(Component, Reflect, Default, Clone, PartialEq, Debug)]
#[require(Mesh2d)]
pub struct UiMeshPlane2d;

/// **Ui Fetch From Camera** - Attaching this component to [`UiLayoutRoot`] will make the
/// [`Dimension`] component pull data from a [`Camera`] with attached [`UiSourceCamera`] that has
/// the same index.
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
pub struct UiFetchFromCamera<const INDEX: usize>;

/// **Ui Source Camera** - Marks a [`Camera`] as a source for [`UiLayoutRoot`] with
/// [`UiFetchFromCamera`]. They must have the same index and only one [`UiSourceCamera`] can exist
/// for a single index.
#[derive(Component, Reflect, Clone, PartialEq, Debug)]
pub struct UiSourceCamera<const INDEX: usize>;

/// This system takes [`Dimension`] data and pipes them into querried [`Handle<Image>`] data to fit.
/// This will resize the original image texture.
pub fn system_embedd_resize(
    query: Query<(&Sprite, &Dimension), (With<UiEmbedding>, Changed<Dimension>)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (sprite, dimension) in &query {
        if let Some(image) = images.get_mut(&sprite.image)
            && **dimension != Vec2::ZERO {
                image.resize(Extent3d {
                    width: dimension.x as u32,
                    height: dimension.y as u32,
                    ..Default::default()
                });
            }
    }
}

/// This system takes [`Dimension`] data and pipes them into querried [`Sprite`].
pub fn system_pipe_sprite_size_from_dimension(
    mut query: Query<(&mut Sprite, &Dimension), Changed<Dimension>>,
) {
    for (mut sprite, dimension) in &mut query {
        sprite.custom_size = Some(**dimension)
    }
}

/// This system takes [`Dimension`] data and constructs a plane mesh.
pub fn system_mesh_3d_reconstruct_from_dimension(
    mut query: Query<
        (&Dimension, &mut Mesh3d, Option<&mut Aabb>),
        (With<UiMeshPlane3d>, Changed<Dimension>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (dimension, mut mesh, aabb_option) in &mut query {
        let plane_mesh = Mesh::from(Rectangle::new(dimension.x, dimension.y));
        if let Some(a) = plane_mesh.compute_aabb()
            && let Some(mut aabb) = aabb_option {
                *aabb = a;
            }
        mesh.0 = meshes.add(plane_mesh);
    }
}

/// This system takes [`Dimension`] data and constructs a plane mesh.
pub fn system_mesh_2d_reconstruct_from_dimension(
    mut query: Query<
        (&Dimension, &mut Mesh2d, Option<&mut Aabb>),
        (With<UiMeshPlane2d>, Changed<Dimension>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (dimension, mut mesh, aabb_option) in &mut query {
        let plane_mesh = Mesh::from(Rectangle::new(dimension.x, dimension.y));
        if let Some(a) = plane_mesh.compute_aabb()
            && let Some(mut aabb) = aabb_option {
                *aabb = a;
            }
        mesh.0 = meshes.add(plane_mesh);
    }
}

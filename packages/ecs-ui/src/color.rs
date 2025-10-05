//! UI color management and styling system for the Lunex UI system

use std::any::TypeId;
use std::collections::HashMap;

use crate::{UiBase, UiState, UiTheme, *};

/// **Ui Color** - This component is used to control the color of the Ui-Node.
/// It is synchronized with a state machine and allows for specifying unique
/// colors for each state.
///
/// Affected components:
/// - [`Sprite`]
/// - [`TextColor`]
/// - the [`ColorMaterial`] of [`MeshMaterial2d`]
/// - the [`StandardMaterial`] of [`MeshMaterial3d`]
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*; use bevy_asset::prelude::*; use bevy_picking::prelude::*; use bevy_color::prelude::*; use bevy_lunex::prelude::*; use bevy_text::prelude::*; use bevy_sprite::prelude::*; use bevy_color::palettes::basic::*; use bevy_math::prelude::*;
/// # fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
/// # commands.spawn((
/// #     UiLayoutRoot::new_2d(),
/// # )).with_children(|ui| {
///       // Spawn as a single color
///       ui.spawn((
///           // ... Layout, etc.
///           UiColor::from(Color::Srgba(RED).with_alpha(0.8)),
///           // ... Sprite, Text, etc.
///       ));
///
///       // Spawn as a collection for different states
///       ui.spawn((
///           // ... Layout, etc.
///           UiColor::new(vec![
///               (UiBase::id(), Color::Srgba(RED).with_alpha(0.8)),
///               (UiHover::id(), Color::Srgba(YELLOW).with_alpha(1.2))
///           ]),
///           // ... Sprite, Text, etc.
///       ));
/// # });
/// # }
/// ```
#[derive(Component, Reflect, Deref, DerefMut, Default, Clone, PartialEq, Debug)]
pub struct UiColor {
    pub colors: HashMap<TypeId, Color>,
}
/// Constructors
impl UiColor {
    /// Define multiple states at once using a vec.
    pub fn new(value: Vec<(TypeId, impl Into<Color>)>) -> Self {
        let mut map = HashMap::new();
        for (state, layout) in value {
            map.insert(state, layout.into());
        }
        Self { colors: map }
    }
}
/// Conversion implementations
impl<T: Into<Color>> From<T> for UiColor {
    fn from(value: T) -> Self {
        let mut map = HashMap::new();
        map.insert(UiBase::id(), value.into());
        Self { colors: map }
    }
}

/// This system takes care of [`UiColor`] data and updates querried [`Sprite`] and [`TextColor`]
/// components. and updates [`ColorMaterial`] and [`StandardMaterial`]. Uses UiTheme for fallback
/// colors.
pub fn system_color(
    theme: Res<UiTheme>,
    mut query: Query<
        (
            Option<&mut Sprite>,
            Option<&mut TextColor>,
            Option<&MeshMaterial2d<ColorMaterial>>,
            Option<&MeshMaterial3d<StandardMaterial>>,
            &UiColor,
            &UiState,
        ),
        Or<(Changed<UiColor>, Changed<UiState>)>,
    >,
    mut materials2d: ResMut<Assets<ColorMaterial>>,
    mut materials3d: Option<ResMut<Assets<StandardMaterial>>>,
) {
    for (node_sprite_option, node_text_option, mat2d, mat3d, node_color, node_state) in &mut query {
        // Normalize the active state weights
        let mut total_weight = 0.0;
        for state in node_color.colors.keys() {
            if let Some(weight) = node_state.states.get(state) {
                total_weight += weight;
            }
        }

        // Combine the color into one normalized
        let mut blend_color = Hsla::new(0.0, 0.0, 0.0, 0.0);

        // If no state active, use base color or theme fallback
        if total_weight == 0.0 {
            if let Some(color) = node_color.colors.get(&UiBase::id()) {
                blend_color = (*color).into();
            } else if node_color.colors.is_empty() {
                // Use theme default when no colors are specified
                if let Some(theme_color) = theme.default_color.colors.get(&UiBase::id()) {
                    blend_color = (*theme_color).into();
                } else {
                    blend_color = Color::srgb(0.8, 0.8, 0.8).into();
                }
            }

        // Blend colors from active states
        } else {
            for (state, color) in &node_color.colors {
                if let Some(weight) = node_state.states.get(state) {
                    let converted: Hsla = (*color).into();

                    if blend_color.alpha == 0.0 {
                        blend_color.hue = converted.hue;
                    } else {
                        blend_color.hue =
                            lerp_hue(blend_color.hue, converted.hue, weight / total_weight);
                    }

                    // blend_color.hue += converted.hue * (weight / total_weight);
                    blend_color.saturation += converted.saturation * (weight / total_weight);
                    blend_color.lightness += converted.lightness * (weight / total_weight);
                    blend_color.alpha += converted.alpha * (weight / total_weight);
                }
            }
        }

        // Apply the color to attached components
        if let Some(mut sprite) = node_sprite_option {
            sprite.color = blend_color.into();
        }
        if let Some(mut text) = node_text_option {
            **text = blend_color.into();
        }
        if let Some(id) = mat2d {
            if let Some(mat) = materials2d.get_mut(id) {
                mat.color = blend_color.into();
            }
        } else if let Some(id) = mat3d
            && let Some(materials3d) = &mut materials3d
                && let Some(mat) = materials3d.get_mut(id) {
                    mat.base_color = blend_color.into();
                }
    }
}

fn lerp_hue(h1: f32, h2: f32, t: f32) -> f32 {
    let diff = (h2 - h1 + 540.0) % 360.0 - 180.0; // Ensure shortest direction
    (h1 + diff * t + 360.0) % 360.0
}

//! Plugin definitions and system sets for the Lunex UI system

#[cfg(feature = "text3d")]
use bevy_rich_text3d::{Text3dPlugin, Text3dSet};

use crate::cursor::CursorPlugin;
use crate::picking::UiLunexPickingPlugin;
use crate::states::UiLunexStatePlugin;
use crate::*;
use crate::{
    // Resources
    DirtyLayout,
    LayoutCache,
    UiEvent,
    UiTheme,
    // Observers
    observer_touch_layout_root,
    system_color,
    system_embedd_resize,
    system_fetch_dimension_from_camera,
    system_image_size_to_layout,
    system_layout_compute_and_mark_3d,
    system_mark_layout_dirty,
    system_mesh_2d_reconstruct_from_dimension,
    system_mesh_3d_reconstruct_from_dimension,
    system_pipe_sprite_size_from_dimension,
    system_recompute_on_change,
    system_recompute_on_theme_change,
    // Systems
    system_state_base_balancer,
    system_text_size_from_dimension,
    system_text_size_to_layout,
    system_touch_camera_if_fetch_added,
};
#[cfg(feature = "text3d")]
use crate::{system_text_3d_size_from_dimension, system_text_3d_size_to_layout};

/// System set for [`UiLunexPlugins`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UiSystems {
    /// Systems that modify data pre-computation
    PreCompute,
    /// The computation
    Compute,
    /// Systems that modify data post-computation
    PostCompute,
}

/// Gizmo group for UI 2D node debug outlines
#[derive(GizmoConfigGroup, Default, Reflect, Clone, Debug)]
pub struct LunexGizmoGroup2d;

/// Gizmo group for UI 3D node debug outlines
#[derive(GizmoConfigGroup, Default, Reflect, Clone, Debug)]
pub struct LunexGizmoGroup3d;

/// This plugin is used for the main logic.
#[derive(Debug, Default, Clone)]
pub struct UiLunexPlugin;
impl Plugin for UiLunexPlugin {
    fn build(&self, app: &mut App) {
        // Add layout caching resources
        app.init_resource::<DirtyLayout>()
            .init_resource::<LayoutCache>();

        // Add UI theming and event resources
        app.init_resource::<UiTheme>().add_event::<UiEvent>();

        // Configure the system set
        app.configure_sets(
            PostUpdate,
            (
                UiSystems::PreCompute.before(UiSystems::Compute),
                UiSystems::PostCompute
                    .after(UiSystems::Compute)
                    .before(TransformSystem::TransformPropagate),
            ),
        );

        // Add observers
        app.add_observer(observer_touch_layout_root);

        // PRE-COMPUTE SYSTEMS
        app.add_systems(
            PostUpdate,
            (
                system_state_base_balancer,
                system_text_size_to_layout.after(update_text2d_layout),
                system_image_size_to_layout,
                system_recompute_on_change::<UiLayout>,
                system_recompute_on_theme_change,
                system_mark_layout_dirty,
            )
                .chain()
                .in_set(UiSystems::PreCompute),
        );

        #[cfg(feature = "text3d")]
        app.add_systems(
            PostUpdate,
            system_text_3d_size_to_layout
                .after(Text3dSet)
                .in_set(UiSystems::PreCompute),
        );

        // COMPUTE SYSTEMS
        app.add_systems(
            PostUpdate,
            (system_layout_compute_and_mark_3d,).in_set(UiSystems::Compute),
        );

        // POST-COMPUTE SYSTEMS
        app.add_systems(
            PostUpdate,
            (
                system_color,
                system_pipe_sprite_size_from_dimension
                    .before(bevy::sprite::SpriteSystem::ComputeSlices),
                system_text_size_from_dimension,
                system_mesh_3d_reconstruct_from_dimension,
                system_mesh_2d_reconstruct_from_dimension,
                system_embedd_resize,
            )
                .in_set(UiSystems::PostCompute),
        );

        #[cfg(feature = "text3d")]
        app.add_systems(
            PostUpdate,
            system_text_3d_size_from_dimension.in_set(UiSystems::PostCompute),
        );

        // Add index plugins
        app.add_plugins((
            CursorPlugin,
            UiLunexStatePlugin,
            UiLunexPickingPlugin,
            UiLunexIndexPlugin::<0>,
            UiLunexIndexPlugin::<1>,
            UiLunexIndexPlugin::<2>,
            UiLunexIndexPlugin::<3>,
        ));
    }
}

/// This plugin is used to enable debug functionality.
#[derive(Debug, Default, Clone)]
pub struct UiLunexDebugPlugin<const GIZMO_2D_LAYER: usize = 0, const GIZMO_3D_LAYER: usize = 0>;
impl<const GIZMO_2D_LAYER: usize, const GIZMO_3D_LAYER: usize> Plugin
    for UiLunexDebugPlugin<GIZMO_2D_LAYER, GIZMO_3D_LAYER>
{
    fn build(&self, app: &mut App) {
        // Configure the Gizmo render groups
        app.init_gizmo_group::<LunexGizmoGroup2d>()
            .init_gizmo_group::<LunexGizmoGroup3d>()
            .add_systems(Startup, |mut config_store: ResMut<GizmoConfigStore>| {
                // Configure 2D gizmo group with proper Bevy 0.16 patterns
                let (my_2d_config, _) = config_store.config_mut::<LunexGizmoGroup2d>();
                my_2d_config.depth_bias = -0.1; // Render slightly in front
                my_2d_config.enabled = true;

                // Configure 3D gizmo group with proper Bevy 0.16 patterns
                let (my_3d_config, _) = config_store.config_mut::<LunexGizmoGroup3d>();
                my_3d_config.depth_bias = -0.1; // Render slightly in front
                my_3d_config.enabled = true;
            });

        // Add the 2d and 3d gizmo outlines
        app.add_systems(
            PostUpdate,
            (system_debug_draw_gizmo_2d, system_debug_draw_gizmo_3d),
        );

        // Add the debug tree printing
        app.add_systems(
            PostUpdate,
            (system_debug_print_data,).in_set(UiSystems::PostCompute),
        );
    }
}

/// This plugin is used to register index components.
#[derive(Debug, Default, Clone)]
pub struct UiLunexIndexPlugin<const INDEX: usize>;
impl<const INDEX: usize> Plugin for UiLunexIndexPlugin<INDEX> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                system_fetch_dimension_from_camera::<INDEX>,
                system_touch_camera_if_fetch_added::<INDEX>,
            )
                .in_set(UiSystems::PreCompute),
        );
    }
}

/// Plugin group adding all necessary plugins for Lunex
pub struct UiLunexPlugins;
impl PluginGroup for UiLunexPlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut builder = PluginGroupBuilder::start::<Self>();

        // Add text 3d plugin
        #[cfg(feature = "text3d")]
        {
            builder = builder.add(Text3dPlugin {
                load_system_fonts: true,
                ..Default::default()
            });
        }

        // Add Lunex plugin
        builder = builder
            .add(UiLunexPlugin)
            .add(crate::textanim::UiLunexAnimPlugin)
            .add(crate::responsive::ResponsivePlugin);

        // Return the plugin group
        builder
    }
}

// Debug systems moved to the bottom of the module
use crate::{Dimension, NameOrEntity, UiLayout, UiLayoutRoot, UiRoot3d, UiState};

/// This system draws the outlines of [`UiLayout`] and [`UiLayoutRoot`] as gizmos.
pub fn system_debug_draw_gizmo_2d(
    query: Query<
        (&GlobalTransform, &Dimension),
        (Or<(With<UiLayout>, With<UiLayoutRoot>)>, Without<UiRoot3d>),
    >,
    mut gizmos: Gizmos<LunexGizmoGroup2d>,
) {
    for (transform, dimension) in &query {
        // Draw the gizmo outline
        gizmos.rect(
            Isometry3d::new(transform.translation(), transform.rotation()),
            **dimension,
            Color::linear_rgb(0.0, 1.0, 0.0),
        );
    }
}

/// This system draws the outlines of [`UiLayout`] and [`UiLayoutRoot`] as gizmos.
pub fn system_debug_draw_gizmo_3d(
    query: Query<
        (&GlobalTransform, &Dimension),
        (Or<(With<UiLayout>, With<UiLayoutRoot>)>, With<UiRoot3d>),
    >,
    mut gizmos: Gizmos<LunexGizmoGroup3d>,
) {
    for (transform, dimension) in &query {
        // Draw the gizmo outline
        gizmos.rect(
            Isometry3d::new(transform.translation(), transform.rotation()),
            **dimension,
            Color::linear_rgb(0.0, 1.0, 0.0),
        );
    }
}

/// This system traverses the hierarchy and prints the debug information.
pub fn system_debug_print_data(
    root_query: Query<
        (&UiLayoutRoot, NameOrEntity, &Dimension, &Children),
        (
            Without<UiLayout>,
            Or<(Changed<UiLayoutRoot>, Changed<Dimension>)>,
        ),
    >,
    node_query: Query<
        (
            &UiLayout,
            &UiState,
            NameOrEntity,
            &Dimension,
            &Transform,
            Option<&Children>,
        ),
        Without<UiLayoutRoot>,
    >,
) {
    use colored::Colorize;

    use crate::UiBase;
    use crate::layouts::UiLayoutType;

    for (_, root_name, root_dimension, root_children) in &root_query {
        // Create output string
        let mut output_string =
            format!("▶ {}", format!("{root_name}").bold().underline().magenta());

        output_string += " ⇒ ";
        output_string += &format!(
            "[w: {}, h: {}]",
            format!("{:.02}", root_dimension.x).green(),
            format!("{:.02}", root_dimension.y).green()
        );

        output_string += "\n";

        // Stack-based traversal
        let mut stack: Vec<(Entity, usize, bool)> = root_children
            .iter()
            .enumerate()
            .map(|(i, child)| (child, 1, i == root_children.len() - 1)) // Track last-child flag
            .rev()
            .collect();

        // Tracks whether previous levels had last children (for vertical bars)
        let mut last_child_levels: Vec<bool> = Vec::new();

        while let Some((current_entity, depth, is_last)) = stack.pop() {
            if let Ok((
                node_layout,
                _node_state,
                node_name,
                node_dimension,
                node_transform,
                node_children_option,
            )) = node_query.get(current_entity)
            {
                // Adjust last_child_levels size
                if last_child_levels.len() < depth {
                    last_child_levels.push(is_last);
                } else {
                    last_child_levels[depth - 1] = is_last;
                }

                // Create the tab level offset
                for &last in &last_child_levels[..depth - 1] {
                    output_string += &if last {
                        format!("{}", "  ┆".black())
                    } else {
                        "  │".to_string()
                    };
                }

                // Add the name
                output_string += if is_last { "  └" } else { "  ├" };
                if node_name.name.is_some() {
                    output_string += &format!("─ {}", format!("{node_name}").bold().yellow());
                } else {
                    output_string += &format!("─ {}", format!("{node_name}").yellow());
                }

                output_string += " ⇒ ";

                output_string += &format!(
                    "[w: {}, h: {}, d: {}]",
                    format!("{:.02}", node_dimension.x).green(),
                    format!("{:.02}", node_dimension.y).green(),
                    format!("{:.00}", node_transform.translation.z).green(),
                );

                if let Some(layout) = node_layout.layouts.get(&UiBase::id()) {
                    match layout {
                        UiLayoutType::Boundary(boundary) => {
                            output_string += &format!(
                                " ➜ {} {} p1: {}, p2: {} {}",
                                "Boundary".bold(),
                                "{",
                                boundary.pos1.to_nicestr(),
                                boundary.pos2.to_nicestr(),
                                "}",
                            );
                        },
                        UiLayoutType::Window(window) => {
                            output_string += &format!(
                                " ➜ {} {} p: {}, s: {}, a: {} {}",
                                "Window".bold(),
                                "{",
                                window.pos.to_nicestr(),
                                window.size.to_nicestr(),
                                window.anchor.to_nicestr(),
                                "}",
                            );
                        },
                        UiLayoutType::Solid(solid) => {
                            output_string += &format!(
                                " ➜ {} {} s: {}, ax: {}, ay: {}, scl: {} {}",
                                "Solid".bold(),
                                "{",
                                solid.size.to_nicestr(),
                                format!("{:.02}", solid.align_x.0).green(),
                                format!("{:.02}", solid.align_y.0).green(),
                                format!("{:?}", solid.scaling).green(),
                                "}",
                            );
                        },
                    }
                } else {
                    output_string += " ➜ [Layout data not found]";
                }

                output_string += "\n";

                if let Some(node_children) = node_children_option {
                    let child_count = node_children.len();
                    for (i, child) in node_children.iter().enumerate().rev() {
                        stack.push((child, depth + 1, i == child_count - 1));
                    }
                }
            }
        }

        // Print to console
        info!("UiLayout change detected:\n{}", output_string);
    }
}

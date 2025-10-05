//! Trait definitions and extension methods for UI system

use crate::states::{clicked_set, hover_set, selected_set};
use crate::units::Ab;
use crate::{UiClicked, UiHover, UiIntro, UiLayout, UiSelected, UiTextSize, *};

/// Extension trait for spawning UI elements with minimal boilerplate.
/// Automatically applies theming, states, and observers.
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*;
/// # use bevy_asset::prelude::*;
/// # use bevy_lunex::prelude::*;
/// fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
///     // Auto-adds hover, click states and observers
///     commands.spawn_ui_button("Click Me", UiLayout::solid().size((200.0, 50.0)).pack());
///
///     // Auto-adds hover state
///     commands.spawn_ui_panel(UiLayout::window().full().pack());
///
///     // Simple text with auto-sizing
///     commands.spawn_ui_text("Hello", UiLayout::window().pos(Rl(10.0)).pack());
/// }
/// ```
pub trait UiCommands {
    /// Spawn a button with text, hover/click states, and picking enabled
    fn spawn_ui_button(&mut self, text: &str, layout: UiLayout) -> Entity;
    /// Spawn a panel with hover state and picking enabled
    fn spawn_ui_panel(&mut self, layout: UiLayout) -> Entity;
    /// Spawn text with auto-sizing
    fn spawn_ui_text(&mut self, text: &str, layout: UiLayout) -> Entity;
    /// Spawn a centered button with default styling
    fn spawn_ui_button_centered(&mut self, text: &str) -> Entity;
    /// Spawn a full-width panel
    fn spawn_ui_panel_full(&mut self) -> Entity;
}

impl UiCommands for Commands<'_, '_> {
    fn spawn_ui_button(&mut self, text: &str, layout: UiLayout) -> Entity {
        self.spawn((
            layout,
            UiHover::new().forward_speed(10.0).backward_speed(5.0),
            UiClicked::new().forward_speed(15.0).backward_speed(8.0),
            UiSelected::new().forward_speed(8.0).backward_speed(6.0),
            Text2d::new(text),
            UiTextSize::from(Ab(24.0)),
            Pickable::default(),
        ))
        .observe(hover_set::<Pointer<Over>, true>)
        .observe(hover_set::<Pointer<Out>, false>)
        .observe(clicked_set::<Pointer<Click>, true>)
        .observe(selected_set::<Pointer<Click>, true>)
        .id()
    }

    fn spawn_ui_panel(&mut self, layout: UiLayout) -> Entity {
        self.spawn((
            layout,
            UiHover::new().forward_speed(8.0).backward_speed(4.0),
            Pickable::default(),
        ))
        .observe(hover_set::<Pointer<Over>, true>)
        .observe(hover_set::<Pointer<Out>, false>)
        .id()
    }

    fn spawn_ui_text(&mut self, text: &str, layout: UiLayout) -> Entity {
        self.spawn((
            layout,
            Text2d::new(text),
            UiTextSize::from(Ab(16.0)),
            UiIntro::new().duration(0.5),
        ))
        .id()
    }

    fn spawn_ui_button_centered(&mut self, text: &str) -> Entity {
        let layout = UiLayout::window()
            .size((Ab(200.0), Ab(50.0)))
            .centered()
            .pack();
        self.spawn_ui_button(text, layout)
    }

    fn spawn_ui_panel_full(&mut self) -> Entity {
        let layout = UiLayout::window().full().pack();
        self.spawn_ui_panel(layout)
    }
}

/// Provides utility constructor methods for [`Image`]
pub trait ImageTextureConstructor {
    /// Just a utility constructor hiding the necessary texture initialization
    fn clear_render_texture() -> Image {
        let mut image = Image::new_fill(
            Extent3d {
                width: 512,
                height: 512,
                ..Default::default()
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;
        image
    }
}
impl ImageTextureConstructor for Image {}

/// Provides utility costructor methods for [`Camera`]
pub trait CameraTextureRenderConstructor {
    /// Just a utility constructor for camera that renders to a transparent texture
    fn clear_render_to(handle: Handle<Image>) -> Camera {
        Camera {
            target: RenderTarget::Image(handle.into()),
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            ..Default::default()
        }
    }
    /// Modify the camera render order
    fn with_order(self, order: isize) -> Self;
}
impl CameraTextureRenderConstructor for Camera {
    fn with_order(mut self, order: isize) -> Self {
        self.order = order;
        self
    }
}

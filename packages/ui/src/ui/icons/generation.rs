use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;

use crate::ui::icons::types::{IconTheme, IconType};

pub fn create_generic_icon(
    _commands: &mut Commands,
    asset_server: &AssetServer,
    theme: &IconTheme,
    icon_type: IconType,
) -> Handle<Image> {
    let size = theme.icon_size.pixels();
    let mut image_data = vec![0u8; (size * size * 4) as usize];

    let (bg_color, _symbol) = match icon_type {
        IconType::Application => (Color::srgba(0.2, 0.4, 0.8, 1.0), "⚡"),
        IconType::Document => (Color::srgba(0.4, 0.6, 0.2, 1.0), "📄"),
        IconType::Folder => (Color::srgba(0.8, 0.6, 0.2, 1.0), "📁"),
        IconType::Command => (Color::srgba(0.6, 0.2, 0.6, 1.0), "⚙"),
        IconType::Unknown => (theme.theme_colors.muted, "?"),
        IconType::Terminal => (Color::srgba(0.1, 0.1, 0.1, 1.0), "▶"),
        IconType::File => (Color::srgba(0.5, 0.5, 0.5, 1.0), "📄"),
        IconType::Code => (Color::srgba(0.3, 0.5, 0.7, 1.0), "<>"),
        IconType::Config => (Color::srgba(0.7, 0.5, 0.3, 1.0), "⚙"),
        IconType::Database => (Color::srgba(0.4, 0.4, 0.6, 1.0), "🗄"),
        IconType::Text => (Color::srgba(0.6, 0.6, 0.4, 1.0), "📝"),
        IconType::Image => (Color::srgba(0.5, 0.3, 0.7, 1.0), "🖼"),
        IconType::Video => (Color::srgba(0.7, 0.3, 0.5, 1.0), "🎬"),
        IconType::Audio => (Color::srgba(0.3, 0.7, 0.5, 1.0), "🎵"),
        IconType::Archive => (Color::srgba(0.6, 0.4, 0.2, 1.0), "📦"),
        IconType::Web => (Color::srgba(0.2, 0.6, 0.8, 1.0), "🌐"),
        IconType::Spreadsheet => (Color::srgba(0.2, 0.8, 0.4, 1.0), "📊"),
        IconType::Presentation => (Color::srgba(0.8, 0.4, 0.2, 1.0), "📽"),
        IconType::Font => (Color::srgba(0.5, 0.5, 0.8, 1.0), "🔤"),
        IconType::Log => (Color::srgba(0.4, 0.4, 0.4, 1.0), "📋"),
        IconType::Lock => (Color::srgba(0.7, 0.7, 0.2, 1.0), "🔒"),
        IconType::Api => (Color::srgba(0.3, 0.8, 0.8, 1.0), "🔌"),
    };

    // Create a simple rounded rectangle with symbol
    generate_icon_pixels(&mut image_data, size, bg_color);

    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        image_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    asset_server.add(image)
}

fn generate_icon_pixels(data: &mut [u8], size: u32, bg_color: Color) {
    let [r, g, b, a] = bg_color.to_srgba().to_u8_array();

    for y in 0..size {
        for x in 0..size {
            let index = ((y * size + x) * 4) as usize;

            // Create rounded rectangle
            let center_x = size as f32 / 2.0;
            let center_y = size as f32 / 2.0;
            let radius = (size as f32 * 0.4).min(center_x).min(center_y);

            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < radius {
                data[index] = r;
                data[index + 1] = g;
                data[index + 2] = b;
                data[index + 3] = a;
            } else {
                data[index] = 0;
                data[index + 1] = 0;
                data[index + 2] = 0;
                data[index + 3] = 0;
            }
        }
    }
}

//! Text handling systems for UI elements in the Lunex UI system

use crate::layouts::UiLayoutType;
use crate::units::Ab;
use crate::{Dimension, RecomputeUiLayout, UiBase, UiLayout, UiTextSize, *};
#[cfg(feature = "text3d")]
use crate::{Text3d, Text3dDimensionOut, Text3dSegment};

/// This system takes [`TextLayoutInfo`] data and pipes them into querried [`Transform`] scale.
pub fn system_text_size_from_dimension(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &Dimension, &TextLayoutInfo), Changed<Dimension>>,
) {
    for (mut transform, dimension, text_info) in &mut query {
        // Wait for text to render
        if text_info.size.y == 0.0 {
            commands.trigger(RecomputeUiLayout);
        }

        // Scale the text
        let scale = **dimension / text_info.size;
        transform.scale.x = scale.x;
        transform.scale.y = scale.x;
    }
}

/// This system takes updated [`TextLayoutInfo`] data and overwrites coresponding [`UiLayout`] data
/// to match the text size.
pub fn system_text_size_to_layout(
    mut commands: Commands,
    mut query: Query<
        (&mut UiLayout, &Text2d, &TextLayoutInfo, &UiTextSize),
        Changed<TextLayoutInfo>,
    >,
) {
    for (mut layout, text, text_info, text_size) in &mut query {
        // Wait for text to render
        if text_info.size.y == 0.0 {
            commands.trigger(RecomputeUiLayout);
        }

        // Create the text layout
        if let Some(layout_type) = layout.layouts.get_mut(&UiBase::id()) {
            match layout_type {
                UiLayoutType::Window(window) => {
                    let lines = 1 + text.trim().matches('\n').count();
                    window.set_height(**text_size * (lines as f32));
                    window.set_width(
                        **text_size * (lines as f32) * (text_info.size.x / text_info.size.y),
                    );
                },
                UiLayoutType::Solid(solid) => {
                    solid.set_size(Ab(text_info.size));
                },
                _ => {},
            }
        } else {
            warn!("UiBase layout state not found for Text - skipping layout update");
        }
    }
}

/// This system takes updated [`Handle<Image>`] data and overwrites coresponding [`UiLayout`] data
/// to match the text size.
pub fn system_image_size_to_layout(
    images: Res<Assets<Image>>,
    mut query: Query<(&mut UiLayout, &Sprite, &crate::UiImageSize)>,
) {
    for (mut layout, sprite, image_size) in &mut query {
        if let Some(image) = images.get(&sprite.image) {
            let x = image_size.get_x() * image.width() as f32;
            let y = image_size.get_y() * image.height() as f32;

            if let Some(layout_data) = layout.layouts.get(&UiBase::id()) {
                let needs_update = match layout_data {
                    UiLayoutType::Window(window) => {
                        window.size.get_x() != x || window.size.get_y() != y
                    },
                    UiLayoutType::Solid(solid) => {
                        solid.size.get_x() != x || solid.size.get_y() != y
                    },
                    _ => false,
                };

                if needs_update {
                    if let Some(layout_mut) = layout.layouts.get_mut(&UiBase::id()) {
                        match layout_mut {
                            UiLayoutType::Window(window) => {
                                window.set_width(x);
                                window.set_height(y);
                            },
                            UiLayoutType::Solid(solid) => {
                                solid.set_width(x);
                                solid.set_height(y);
                            },
                            _ => {},
                        }
                    } else {
                        error!("Failed to get mutable layout reference for UiBase");
                    }
                }
            } else {
                error!("Layout data not found for UiBase entity");
            }
        }
    }
}

// # TEXT 3D

/// This system takes updated [`Text3dDimensionOut`] data and overwrites coresponding [`UiLayout`]
/// data to match the text size.
#[cfg(feature = "text3d")]
pub fn system_text_3d_size_to_layout(
    mut commands: Commands,
    mut query: Query<
        (&mut UiLayout, &Text3d, &Text3dDimensionOut, &UiTextSize),
        Changed<Text3dDimensionOut>,
    >,
    fetched_text_query: Query<&bevy_rich_text3d::FetchedTextSegment>,
) {
    for (mut layout, text, text_info, text_size) in &mut query {
        // Wait for text to render
        if text_info.dimension.y == 0.0 {
            commands.trigger(RecomputeUiLayout);
            continue;
        }

        // Create the text layout
        if let Some(layout_type) = layout.layouts.get_mut(&UiBase::id()) {
            match layout_type {
                UiLayoutType::Window(window) => {
                    // Extract text content and count lines (consistent with 2D text system)
                    let lines = if let Some(text_str) = text.get_single() {
                        // Single segment text - count newlines directly
                        1 + text_str.trim().matches('\n').count()
                    } else {
                        // Multi-segment rich text - count newlines across all segments
                        let total_newlines = text
                            .segments
                            .iter()
                            .map(|(segment, _)| {
                                match segment {
                                    Text3dSegment::String(text_str) => {
                                        text_str.matches('\n').count()
                                    },
                                    Text3dSegment::Extract(entity) => {
                                        // Query the FetchedTextSegment component on the referenced
                                        // entity
                                        match fetched_text_query.get(*entity) {
                                            Ok(fetched_segment) => {
                                                fetched_segment.as_str().matches('\n').count()
                                            },
                                            Err(_) => {
                                                // Entity doesn't exist or doesn't have
                                                // FetchedTextSegment
                                                // This is unusual but not necessarily an error
                                                0
                                            },
                                        }
                                    },
                                }
                            })
                            .sum::<usize>();
                        1 + total_newlines
                    };
                    window.set_height(**text_size * (lines as f32));
                    window.set_width(
                        **text_size
                            * (lines as f32)
                            * (text_info.dimension.x / text_info.dimension.y),
                    );
                },
                UiLayoutType::Solid(solid) => {
                    solid.set_size(Ab(text_info.dimension));
                },
                _ => {},
            }
        } else {
            warn!("UiBase layout state not found for Text - skipping layout update");
        }
    }
}

/// This system takes [`Text3dDimensionOut`] data and pipes them into querried [`Transform`] scale.
#[cfg(feature = "text3d")]
pub fn system_text_3d_size_from_dimension(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &Dimension, &Text3dDimensionOut), Changed<Dimension>>,
) {
    for (mut transform, dimension, text_info) in &mut query {
        // Wait for text to render
        if text_info.dimension.y == 0.0 {
            commands.trigger(RecomputeUiLayout);
            continue;
        }

        // Scale the text
        let scale = dimension.x / text_info.dimension.x;
        transform.scale.x = scale;
        transform.scale.y = scale;
    }
}

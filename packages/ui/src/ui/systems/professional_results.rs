//! Professional results display system for Action Items launcher
//!
//! Raycast-quality result rendering with smooth animations, professional typography, and
//! interaction states

use action_items_core::CurrentSearchResults;
use bevy::prelude::*;
use tracing::{info, warn};

use crate::ui::components::{
    ActionItemsSearchResultBackground, ActionItemsSearchResultData, ActionItemsSearchResultIcon,
    ActionItemsSearchResultItem, ActionItemsSearchResultShortcut, ActionItemsSearchResultSubtitle,
    ActionItemsSearchResultTitle, ResultsContainer, UiFonts,
};
use crate::ui::icons::{LauncherIconCache, get_icon_for_search_result};
use action_items_ecs_ui::theme::{ShadowElevation, Theme};
use crate::ui::typography::TypographyScale;

// Type aliases for complex query types
type HoverEffectsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        &'static ActionItemsSearchResultItem,
    ),
    (Changed<Interaction>, With<ActionItemsSearchResultBackground>),
>;

/// System to render professional search results with Raycast-quality styling
/// Updates results display when search results change
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn render_professional_results(
    mut commands: Commands,
    search_results: Res<CurrentSearchResults>,
    results_container_query: Query<Entity, With<ResultsContainer>>,
    existing_results: Query<Entity, With<ActionItemsSearchResultItem>>,
    theme: Res<Theme>,
    typography: Res<TypographyScale>,
    ui_fonts: Res<UiFonts>,
    icon_cache: Res<LauncherIconCache>,
) {
    if !search_results.is_changed() {
        return;
    }

    // Clear existing result items
    for entity in existing_results.iter() {
        commands.entity(entity).despawn();
    }

    // Find results container
    let Ok(container_entity) = results_container_query.single() else {
        warn!("Results container not found");
        return;
    };

    // Convert core ActionItems to professional display data
    let professional_results: Vec<ActionItemsSearchResultData> = search_results
        .results
        .iter()
        .enumerate()
        .map(|(index, item)| {
            ActionItemsSearchResultData::new(item.title.clone(), item.action.clone())
                .with_subtitle(item.description.clone())
                .with_category("General".to_string()) // Default category since ActionItem doesn't have category field
                .with_score(item.score)
                .with_ranking(index) // Use index for result ranking display
        })
        .collect();

    // Render professional result items
    commands.entity(container_entity).with_children(|parent| {
        for (index, result_data) in professional_results.iter().enumerate() {
            let is_selected = index == 0; // First item is selected by default
            let background_color = if is_selected {
                Color::srgba(0.3, 0.6, 1.0, 0.15) // Selected state
            } else {
                Color::NONE // Default transparent
            };

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Vh(4.5),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::VMin(1.2), Val::VMin(0.8)),
                        margin: UiRect::bottom(Val::VMin(0.5)),
                        ..default()
                    },
                    BackgroundColor(background_color),
                    BorderRadius::all(Val::VMin(0.8)),
                    theme.create_box_shadow(ShadowElevation::SM),
                    ActionItemsSearchResultItem {
                        action_id: result_data.action_id.clone(),
                        is_selected,
                        index,
                    },
                    ActionItemsSearchResultBackground,
                    Interaction::default(),
                ))
                .with_children(|result_parent| {
                    // Icon container (left side)
                    result_parent
                        .spawn((
                            Node {
                                width: Val::VMin(2.4),
                                height: Val::VMin(2.4),
                                margin: UiRect::right(Val::VMin(1.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(theme.colors.surface_default),
                            BorderRadius::all(Val::VMin(0.6)),
                        ))
                        .with_children(|icon_parent| {
                            // Get proper icon from cache using the SearchResult
                            let search_result = &search_results.results[index];
                            let icon_handle =
                                get_icon_for_search_result(search_result, &icon_cache);

                            // Create ImageNode with proper icon
                            icon_parent.spawn((
                                ImageNode::new(icon_handle.clone()),
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                ActionItemsSearchResultIcon {
                                    result_id: result_data.action_id.clone(),
                                    loading: false,
                                    image_handle: Some(icon_handle),
                                    fallback_text: None,
                                },
                            ));
                        });

                    // Content container (main area)
                    result_parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },))
                        .with_children(|content_parent| {
                            // Title
                            content_parent.spawn((
                                Text::new(result_data.title.clone()),
                                TextFont {
                                    font: ui_fonts.ubuntu_medium.clone(),
                                    font_size: typography.text_styles.body.font_size,
                                    ..default()
                                },
                                TextColor(theme.colors.text_primary),
                                ActionItemsSearchResultTitle,
                            ));

                            // Subtitle (if available)
                            if let Some(subtitle) = &result_data.subtitle
                                && !subtitle.is_empty()
                            {
                                content_parent.spawn((
                                    Text::new(subtitle.clone()),
                                    TextFont {
                                        font: ui_fonts.regular.clone(),
                                        font_size: typography.text_styles.caption.font_size,
                                        ..default()
                                    },
                                    TextColor(theme.colors.text_secondary),
                                    ActionItemsSearchResultSubtitle,
                                ));
                            }
                        });

                    // Keyboard shortcut (right side, if available)
                    if let Some(shortcut) = &result_data.shortcut {
                        result_parent
                            .spawn((
                                Node {
                                    padding: UiRect::axes(Val::VMin(0.6), Val::VMin(0.3)),
                                    margin: UiRect::left(Val::VMin(1.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(theme.colors.surface_default.with_alpha(0.6)),
                                BorderRadius::all(Val::VMin(0.4)),
                            ))
                            .with_children(|shortcut_parent| {
                                shortcut_parent.spawn((
                                    Text::new(shortcut.clone()),
                                    TextFont {
                                        font: ui_fonts.mono.clone(),
                                        font_size: typography.text_styles.caption.font_size * 0.9,
                                        ..default()
                                    },
                                    TextColor(theme.colors.text_tertiary),
                                    ActionItemsSearchResultShortcut,
                                ));
                            });
                    }
                });
        }
    });

    info!(
        "Rendered {} professional results",
        professional_results.len()
    );
}

/// System to handle result item selection and highlighting
/// Updates visual states based on keyboard navigation or mouse interaction
#[inline]
pub fn handle_result_selection(
    mut result_items: Query<(&mut BackgroundColor, &ActionItemsSearchResultItem)>,
    ui_state: Res<crate::ui::components::UiState>,
) {
    for (mut background, result_item) in result_items.iter_mut() {
        let should_be_selected = result_item.index == ui_state.selected_index;

        if result_item.is_selected != should_be_selected {
            // Update background color based on selection state
            background.0 = if should_be_selected {
                Color::srgba(0.3, 0.6, 1.0, 0.15) // Selected
            } else {
                Color::NONE // Default
            };
        }
    }
}

/// System to handle result item hover effects
/// Provides visual feedback for mouse interactions
#[inline]
pub fn handle_result_hover_effects(mut result_items: HoverEffectsQuery) {
    for (interaction, mut background, result_item) in result_items.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                if !result_item.is_selected {
                    background.0 = Color::srgba(1.0, 1.0, 1.0, 0.05); // Subtle hover
                }
            },
            Interaction::Pressed => {
                background.0 = Color::srgba(0.3, 0.6, 1.0, 0.25); // Pressed feedback
            },
            Interaction::None => {
                if !result_item.is_selected {
                    background.0 = Color::NONE; // Return to default
                }
            },
        }
    }
}

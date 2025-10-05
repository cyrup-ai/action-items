use action_items_ecs_ui::prelude::*;
use action_items_ecs_ui::theme::Theme;
use bevy::prelude::*;
use crate::ui::components::*;

/// Generate search bar screen with ecs-ui patterns
/// 
/// Creates a centered search input bar with icon and placeholder text.
/// Uses theme colors and spacing for consistent appearance.
pub fn create_search_bar_screen(
    commands: &mut Commands,
    theme: &Theme,
) -> Entity {
    let bg_color = theme.colors.background_secondary;
    let text_color = theme.colors.text_primary;
    let icon_color = theme.colors.text_secondary;
    let border_color = theme.colors.border_default;
    
    // Create search bar container
    let search_bar = commands.spawn((
        UiLayout::window()
            .size((Vw(50.0), Ab(56.0)))
            .pos((Vw(50.0), Vh(20.0)))
            .anchor(Anchor::Center)
            .pack(),
        UiColor::from(bg_color),
        BorderRadius::all(Val::Px(8.0)),
        Outline {
            width: Val::Px(1.0),
            offset: Val::Px(0.0),
            color: border_color,
        },
        SearchBarComponent::default(),
        Pickable::default(),
        Name::new("SearchBar"),
    )).id();
    
    // Add search icon child (positioned on left)
    let icon = commands.spawn((
        UiLayout::window()
            .size((Ab(20.0), Ab(20.0)))
            .pos((Ab(12.0), Ab(18.0)))
            .pack(),
        Text::new("🔍"),
        UiTextSize::from(Em(1.2)),
        TextColor(icon_color),
        SearchIcon,
        Name::new("SearchIcon"),
    )).id();
    
    commands.entity(search_bar).add_children(&[icon]);
    
    // Add input field child (positioned after icon)
    let input = commands.spawn((
        UiLayout::window()
            .size((Rl(85.0), Rl(70.0)))
            .pos((Ab(40.0), Rl(15.0)))
            .pack(),
        Text::new(""),
        UiTextSize::from(Em(1.0)),
        TextColor(text_color),
        SearchInputField,
        Pickable::default(),
        Name::new("SearchInput"),
    )).id();
    
    commands.entity(search_bar).add_children(&[input]);
    
    search_bar
}

/// Create search bar with initial visibility hidden
pub fn create_hidden_search_bar(
    commands: &mut Commands,
    theme: &Theme,
) -> Entity {
    let search_bar = create_search_bar_screen(commands, theme);
    commands.entity(search_bar).insert(Visibility::Hidden);
    search_bar
}

/// Generate search results container
/// 
/// Creates a scrollable container below the search bar for displaying results.
pub fn create_results_container(
    commands: &mut Commands,
    theme: &Theme,
) -> Entity {
    commands.spawn((
        UiLayout::window()
            .size((Vw(50.0), Vh(65.0)))
            .pos((Vw(50.0), Vh(58.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        UiColor::from(theme.colors.background_primary),
        SearchResultsContainer,
        Name::new("ResultsContainer"),
    )).id()
}

/// Generate individual result item with hover effects
/// 
/// Migrated from result_rendering.rs lines 114-180.
/// Preserves staggered animation timing and selection visual feedback.
pub fn spawn_result_item(
    commands: &mut Commands,
    parent: Entity,
    result: &crate::components::SearchResult,
    index: usize,
    is_selected: bool,
    theme: &Theme,
) {
    let bg_color = if is_selected {
        theme.colors.surface_selected
    } else {
        theme.colors.surface_default
    };
    
    // Calculate vertical position based on index (50px per item + 4px spacing)
    let y_pos = Ab((index as f32) * 54.0);
    
    let item = commands.spawn((
        UiLayout::window()
            .size((Rl(98.0), Ab(50.0)))
            .pos((Rl(1.0), y_pos))
            .pack(),
        UiColor::from(bg_color),
        UiHover::new()
            .forward_speed(10.0)
            .backward_speed(5.0),
        UiClicked::new()
            .forward_speed(15.0)
            .backward_speed(8.0),
        BorderRadius::all(Val::Px(6.0)),
        SearchResultItem {
            index,
            result_id: result.id.clone(),
            score: result.score,
        },
        Pickable::default(),
        Interaction::default(),
        Transform::from_scale(Vec3::splat(0.98))
            .with_translation(Vec3::new(0.0, -10.0, 0.0)),
        Name::new(format!("ResultItem_{}", index)),
    )).id();
    
    // Add icon (left side)
    let icon = commands.spawn((
        UiLayout::window()
            .size((Ab(24.0), Ab(24.0)))
            .pos((Ab(12.0), Ab(13.0)))
            .pack(),
        Text::new(&result.icon),
        UiTextSize::from(Em(1.4)),
        Name::new("ResultIcon"),
    )).id();
    
    // Add title (after icon)
    let title = commands.spawn((
        UiLayout::window()
            .size((Rl(70.0), Ab(20.0)))
            .pos((Ab(48.0), Ab(8.0)))
            .pack(),
        Text::new(&result.title),
        UiTextSize::from(Em(1.0)),
        TextColor(theme.colors.text_primary),
        Name::new("ResultTitle"),
    )).id();
    
    // Add subtitle (below title)
    let subtitle = commands.spawn((
        UiLayout::window()
            .size((Rl(70.0), Ab(16.0)))
            .pos((Ab(48.0), Ab(28.0)))
            .pack(),
        Text::new(&result.subtitle),
        UiTextSize::from(Em(0.85)),
        TextColor(theme.colors.text_secondary),
        Name::new("ResultSubtitle"),
    )).id();
    
    commands.entity(item).add_children(&[icon, title, subtitle]);
    commands.entity(parent).add_children(&[item]);
}

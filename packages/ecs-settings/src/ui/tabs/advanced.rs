use action_items_ecs_ui::prelude::*;
use action_items_ecs_user_settings::table_names::*;
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use crate::ui::{components::*, theme::*};

pub fn create_advanced_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        let mut y_offset = 20.0;

        commands.entity(parent).with_children(|parent| {
            // Display settings
            y_offset = create_section_header(parent, "Display", y_offset);
            y_offset = create_form_row(
                parent,
                "Animations",
                |p, y| create_checkbox(p, "animations_enabled", ADVANCED_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Transparency",
                |p, y| create_checkbox(p, "transparency_enabled", ADVANCED_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Show Icons",
                |p, y| create_checkbox(p, "show_icons", ADVANCED_SETTINGS, true, y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Keyboard behavior
            y_offset = create_section_header(parent, "Keyboard", y_offset);
            y_offset = create_form_row(
                parent,
                "Tab Key Navigation",
                |p, y| create_checkbox(p, "tab_navigation", ADVANCED_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Auto-paste on Enter",
                |p, y| create_checkbox(p, "auto_paste", ADVANCED_SETTINGS, false, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Close on Escape",
                |p, y| create_checkbox(p, "close_on_escape", ADVANCED_SETTINGS, true, y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Search sensitivity
            y_offset = create_section_header(parent, "Search", y_offset);
            y_offset = create_form_row(
                parent,
                "Sensitivity",
                |p, y| create_dropdown(p, "search_sensitivity", ADVANCED_SETTINGS, vec!["Low", "Medium", "High"], 1, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Fuzzy Match",
                |p, y| create_checkbox(p, "fuzzy_match", ADVANCED_SETTINGS, true, y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Hyper key
            y_offset = create_section_header(parent, "Hyper Key", y_offset);
            y_offset = create_form_row(
                parent,
                "Enable Hyper Key",
                |p, y| create_checkbox(p, "hyper_key_enabled", ADVANCED_SETTINGS, false, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Key Combination",
                |p, y| create_dropdown(p, "hyper_key_combo", ADVANCED_SETTINGS, vec!["⌘⌥⌃⇧", "⌥⌃", "⌘⇧"], 0, y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Additional options
            y_offset = create_section_header(parent, "Additional", y_offset);
            y_offset = create_form_row(
                parent,
                "Cache Results",
                |p, y| create_checkbox(p, "cache_results", ADVANCED_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Telemetry",
                |p, y| create_checkbox(p, "telemetry_enabled", ADVANCED_SETTINGS, false, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Auto-update",
                |p, y| create_checkbox(p, "auto_update", ADVANCED_SETTINGS, true, y),
                y_offset
            );
        });
    }
}

// Helper functions
fn create_section_header(parent: &mut ChildSpawnerCommands, title: &str, y_offset: f32) -> f32 {
    parent.spawn((
        Text::new(title),
        UiTextSize::from(Em(1.3)),
        UiColor::from(TEXT_PRIMARY),
        UiLayout::window()
            .size((Rl(90.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
    ));
    y_offset + 40.0
}

fn create_form_row<F>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    control_generator: F,
    y_offset: f32
) -> f32
where
    F: FnOnce(&mut ChildSpawnerCommands, f32)
{
    parent.spawn((
        Text::new(label),
        UiTextSize::from(Em(1.0)),
        UiColor::from(TEXT_SECONDARY),
        UiLayout::window()
            .size((Rl(LABEL_WIDTH_PCT), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
    ));

    control_generator(parent, y_offset);

    y_offset + 35.0
}

fn create_checkbox(parent: &mut ChildSpawnerCommands, field_name: &str, table: &str, checked: bool, y_offset: f32) {
    parent.spawn((
        UiLayout::window()
            .size((Ab(20.0), Ab(20.0)))
            .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset + 5.0)))
            .pack(),
        UiColor::from(if checked { CHECKBOX_CHECKED } else { CHECKBOX_BG }),
        UiHover::new().forward_speed(8.0),
        UiClicked::new().forward_speed(12.0),
        BorderRadius::all(Val::Px(4.0)),
        SettingControl {
            field_name: field_name.to_string(),
            table: table.to_string()
        },
        SettingCheckbox { checked },
        Pickable::default(),
    ));

    // Error display
    parent.spawn((
        UiLayout::window()
            .size((Rl(CONTROL_WIDTH_PCT), Ab(20.0)))
            .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset + 30.0)))
            .pack(),
        Text::new(""),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(1.0, 0.3, 0.3, 1.0)),
        Visibility::Hidden,
        SettingErrorDisplay {
            field_name: field_name.to_string(),
        },
        Name::new(format!("Error_{}", field_name)),
    ));
}

fn create_dropdown(parent: &mut ChildSpawnerCommands, field_name: &str, table: &str, options: Vec<&str>, selected: usize, y_offset: f32) {
    let options: Vec<String> = options.into_iter().map(|s| s.to_string()).collect();
    let selected_text = options.get(selected).cloned().unwrap_or_default();

    parent.spawn((
        UiLayout::window()
            .size((Rl(CONTROL_WIDTH_PCT), Ab(30.0)))
            .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset)))
            .pack(),
        UiColor::from(INPUT_BG),
        UiHover::new(),
        UiClicked::new(),
        Text::new(&selected_text),
        DropdownControl {
            field_name: field_name.to_string(),
            options,
            selected,
            is_open: false
        },
        SettingControl {
            field_name: field_name.to_string(),
            table: table.to_string()
        },
        BorderRadius::all(Val::Px(6.0)),
        Pickable::default(),
    ));

    // Error display
    parent.spawn((
        UiLayout::window()
            .size((Rl(CONTROL_WIDTH_PCT), Ab(20.0)))
            .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset + 35.0)))
            .pack(),
        Text::new(""),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(1.0, 0.3, 0.3, 1.0)),
        Visibility::Hidden,
        SettingErrorDisplay {
            field_name: field_name.to_string(),
        },
        Name::new(format!("Error_{}", field_name)),
    ));
}

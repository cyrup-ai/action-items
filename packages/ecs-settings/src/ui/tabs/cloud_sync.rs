use action_items_ecs_ui::prelude::*;
use action_items_ecs_user_settings::table_names::*;
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use crate::ui::{components::*, theme::*};

pub fn create_cloud_sync_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        let mut y_offset = 20.0;

        commands.entity(parent).with_children(|parent| {
            // Master sync toggle
            y_offset = create_section_header(parent, "Cloud Sync", y_offset);
            y_offset = create_form_row(
                parent,
                "Enable Sync",
                |p, y| create_checkbox(p, "sync_enabled", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Last Synced",
                |p, y| create_text_display(p, "Never", y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Synced items section
            y_offset = create_section_header(parent, "Synced Items", y_offset);
            y_offset = create_form_row(
                parent,
                "Snippets",
                |p, y| create_checkbox(p, "sync_snippets", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Aliases",
                |p, y| create_checkbox(p, "sync_aliases", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Hotkeys",
                |p, y| create_checkbox(p, "sync_hotkeys", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Quicklinks",
                |p, y| create_checkbox(p, "sync_quicklinks", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Extensions",
                |p, y| create_checkbox(p, "sync_extensions", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Settings",
                |p, y| create_checkbox(p, "sync_settings", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Favorites",
                |p, y| create_checkbox(p, "sync_favorites", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "History",
                |p, y| create_checkbox(p, "sync_history", CLOUD_SYNC_SETTINGS, false, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Clipboard",
                |p, y| create_checkbox(p, "sync_clipboard", CLOUD_SYNC_SETTINGS, false, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "AI Prompts",
                |p, y| create_checkbox(p, "sync_ai_prompts", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Workflows",
                |p, y| create_checkbox(p, "sync_workflows", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Scripts",
                |p, y| create_checkbox(p, "sync_scripts", CLOUD_SYNC_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Themes",
                |p, y| create_checkbox(p, "sync_themes", CLOUD_SYNC_SETTINGS, false, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Window State",
                |p, y| create_checkbox(p, "sync_window_state", CLOUD_SYNC_SETTINGS, false, y),
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

fn create_text_display(parent: &mut ChildSpawnerCommands, text: &str, y_offset: f32) {
    parent.spawn((
        UiLayout::window()
            .size((Rl(CONTROL_WIDTH_PCT), Ab(30.0)))
            .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset)))
            .pack(),
        Text::new(text),
        UiTextSize::from(Em(1.0)),
        UiColor::from(TEXT_SECONDARY),
    ));
}

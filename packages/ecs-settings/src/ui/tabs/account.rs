use action_items_ecs_ui::prelude::*;
use action_items_ecs_user_settings::table_names::*;
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use crate::ui::{components::*, theme::*};

pub fn create_account_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        let mut y_offset = 20.0;

        commands.entity(parent).with_children(|parent| {
            // User profile
            y_offset = create_section_header(parent, "Profile", y_offset);
            y_offset = create_form_row(
                parent,
                "Name",
                |p, y| create_text_display(p, "User Name", y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Email",
                |p, y| create_text_display(p, "user@example.com", y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Subscription
            y_offset = create_section_header(parent, "Subscription", y_offset);
            y_offset = create_form_row(
                parent,
                "Plan",
                |p, y| create_text_display(p, "Pro", y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Status",
                |p, y| create_text_display(p, "Active", y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Pro features
            y_offset = create_section_header(parent, "Pro Features", y_offset);
            y_offset = create_form_row(
                parent,
                "AI Assistant",
                |p, y| create_checkbox_display(p, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Cloud Sync",
                |p, y| create_checkbox_display(p, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Custom Themes",
                |p, y| create_checkbox_display(p, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "API Access",
                |p, y| create_checkbox_display(p, true, y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Organization
            y_offset = create_section_header(parent, "Organization", y_offset);
            y_offset = create_form_row(
                parent,
                "Organization",
                |p, y| create_text_display(p, "None", y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Developer
            y_offset = create_section_header(parent, "Developer", y_offset);
            y_offset = create_form_row(
                parent,
                "API Token",
                |p, y| create_text_display(p, "••••••••••••", y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Webhooks",
                |p, y| create_checkbox(p, "webhooks_enabled", ACCOUNT_SETTINGS, false, y),
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

fn create_checkbox_display(parent: &mut ChildSpawnerCommands, checked: bool, y_offset: f32) {
    parent.spawn((
        UiLayout::window()
            .size((Ab(20.0), Ab(20.0)))
            .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset + 5.0)))
            .pack(),
        UiColor::from(if checked { CHECKBOX_CHECKED } else { CHECKBOX_BG }),
        BorderRadius::all(Val::Px(4.0)),
    ));
}

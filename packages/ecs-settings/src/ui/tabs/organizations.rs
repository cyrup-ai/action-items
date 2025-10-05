use action_items_ecs_ui::prelude::*;
use action_items_ecs_user_settings::table_names::*;
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use crate::ui::{components::*, theme::*};

pub fn create_organizations_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        let mut y_offset = 20.0;

        commands.entity(parent).with_children(|parent| {
            // Organization details
            y_offset = create_section_header(parent, "Active Organization", y_offset);
            y_offset = create_form_row(
                parent,
                "Organization",
                |p, y| create_text_display(p, "My Organization", y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Your Role",
                |p, y| create_text_display(p, "Owner", y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Subscription
            y_offset = create_section_header(parent, "Subscription", y_offset);
            y_offset = create_form_row(
                parent,
                "Plan",
                |p, y| create_text_display(p, "Enterprise", y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Seats",
                |p, y| create_text_display(p, "10 / 50", y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Billing",
                |p, y| create_text_display(p, "Monthly", y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Store access
            y_offset = create_section_header(parent, "Store Access", y_offset);
            y_offset = create_form_row(
                parent,
                "Enable Store",
                |p, y| create_checkbox(p, "org_store_enabled", ORGANIZATION_SETTINGS, true, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Require Approval",
                |p, y| create_checkbox(p, "org_store_approval", ORGANIZATION_SETTINGS, true, y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Members
            y_offset = create_section_header(parent, "Members", y_offset);
            y_offset = create_form_row(
                parent,
                "Total Members",
                |p, y| create_text_display(p, "10", y),
                y_offset
            );

            y_offset += SECTION_SPACING;

            // Danger zone
            y_offset = create_section_header(parent, "Danger Zone", y_offset);
            y_offset = create_form_row(
                parent,
                "Leave Organization",
                |p, y| create_danger_button(p, "Leave", y),
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

fn create_danger_button(parent: &mut ChildSpawnerCommands, text: &str, y_offset: f32) {
    parent.spawn((
        UiLayout::window()
            .size((Ab(100.0), Ab(32.0)))
            .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset)))
            .pack(),
        UiColor::from(Color::srgba(0.8, 0.2, 0.2, 1.0)),
        UiHover::new().forward_speed(8.0),
        UiClicked::new().forward_speed(12.0),
        Text::new(text),
        UiTextSize::from(Em(1.0)),
        BorderRadius::all(Val::Px(6.0)),
        Pickable::default(),
    ));
}

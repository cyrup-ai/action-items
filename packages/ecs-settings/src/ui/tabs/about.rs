use action_items_ecs_ui::prelude::*;
use bevy::prelude::*;
use crate::ui::theme::*;

pub fn create_about_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        commands.entity(parent).with_children(|parent| {
            // Logo placeholder (left side, upper quadrant)
            parent.spawn((
                UiLayout::window()
                    .size((Ab(160.0), Ab(160.0)))
                    .pos((Ab(120.0), Ab(140.0)))
                    .pack(),
                UiColor::from(Color::srgba(0.28, 0.28, 0.31, 1.0)),  // Dark gray placeholder
                BorderRadius::all(Val::Px(20.0)),
                crate::ui::components::AboutAppLogo,
                Name::new("AboutLogo"),
            )).with_children(|logo| {
                // FontAwesome rocket icon as placeholder (until real logo added)
                logo.spawn((
                    Text::new("\u{f135}"),  // FA rocket icon
                    UiTextSize::from(Em(5.5)),
                    UiColor::from(Color::srgba(0.95, 0.45, 0.45, 1.0)),  // Coral/red tint
                    UiLayout::window()
                        .size((Rl(100.0), Rl(100.0)))
                        .pack(),
                ));
            });

            // App name (right of logo)
            parent.spawn((
                Text::new("Action Items"),
                UiTextSize::from(Em(2.2)),
                UiColor::from(TEXT_PRIMARY),
                UiLayout::window()
                    .size((Rl(50.0), Ab(50.0)))
                    .pos((Ab(310.0), Ab(145.0)))
                    .pack(),
                Name::new("AppName"),
            ));

            // Version (below app name)
            parent.spawn((
                Text::new("Version 1.0.0"),
                UiTextSize::from(Em(1.0)),
                UiColor::from(TEXT_SECONDARY),
                UiLayout::window()
                    .size((Rl(50.0), Ab(35.0)))
                    .pos((Ab(310.0), Ab(200.0)))
                    .pack(),
                Name::new("AppVersion"),
            ));

            // Copyright line 1
            parent.spawn((
                Text::new("© 2024 Action Items."),
                UiTextSize::from(Em(0.9)),
                UiColor::from(TEXT_SECONDARY),
                UiLayout::window()
                    .size((Rl(50.0), Ab(30.0)))
                    .pos((Ab(310.0), Ab(240.0)))
                    .pack(),
                Name::new("Copyright1"),
            ));

            // Copyright line 2
            parent.spawn((
                Text::new("All Rights Reserved."),
                UiTextSize::from(Em(0.9)),
                UiColor::from(TEXT_SECONDARY),
                UiLayout::window()
                    .size((Rl(50.0), Ab(30.0)))
                    .pos((Ab(310.0), Ab(265.0)))
                    .pack(),
                Name::new("Copyright2"),
            ));

            // Acknowledgements button (bottom left)
            parent.spawn((
                UiLayout::window()
                    .size((Ab(175.0), Ab(38.0)))
                    .pos((Ab(60.0), Ab(520.0)))
                    .pack(),
                UiColor::from(Color::srgba(0.23, 0.23, 0.24, 1.0)),  // Dark gray like screenshot
                UiHover::new().forward_speed(8.0),
                UiClicked::new().forward_speed(12.0),
                Text::new("Acknowledgements"),
                UiTextSize::from(Em(0.95)),
                BorderRadius::all(Val::Px(8.0)),
                Pickable::default(),
                crate::ui::components::AcknowledgementsLink,
                Name::new("AcknowledgementsLink"),
            ));

            // Bottom right button positioning
            let button_y = 520.0;
            let button_width = 145.0;
            let button_gap = 12.0;
            let right_margin = 60.0;

            // Visit Website button (first button, bottom right area)
            parent.spawn((
                UiLayout::window()
                    .size((Ab(button_width), Ab(38.0)))
                    .pos((Rl(100.0) - Ab(button_width * 2.0 + button_gap + right_margin), Ab(button_y)))
                    .pack(),
                UiColor::from(Color::srgba(0.23, 0.23, 0.24, 1.0)),
                UiHover::new().forward_speed(8.0),
                UiClicked::new().forward_speed(12.0),
                Text::new("Visit Website"),
                UiTextSize::from(Em(0.95)),
                BorderRadius::all(Val::Px(8.0)),
                Pickable::default(),
                crate::ui::components::VisitWebsiteButton,
                Name::new("VisitWebsiteButton"),
            ));

            // Send Feedback button (second button, right of Visit Website)
            parent.spawn((
                UiLayout::window()
                    .size((Ab(button_width), Ab(38.0)))
                    .pos((Rl(100.0) - Ab(button_width + right_margin), Ab(button_y)))
                    .pack(),
                UiColor::from(Color::srgba(0.23, 0.23, 0.24, 1.0)),
                UiHover::new().forward_speed(8.0),
                UiClicked::new().forward_speed(12.0),
                Text::new("Send Feedback"),
                UiTextSize::from(Em(0.95)),
                BorderRadius::all(Val::Px(8.0)),
                Pickable::default(),
                crate::ui::components::SendFeedbackButton,
                Name::new("SendFeedbackButton"),
            ));
        });
    }
}

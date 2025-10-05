use action_items_ecs_ui::prelude::*;
use bevy::prelude::*;
use crate::ui::theme::*;

pub fn create_about_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        commands.entity(parent).with_children(|parent| {
            // App logo and name (centered)
            parent.spawn((
                Text::new("Action Items"),
                UiTextSize::from(Em(2.0)),
                UiColor::from(TEXT_PRIMARY),
                UiLayout::window()
                    .size((Rl(100.0), Ab(60.0)))
                    .pos((Rl(0.0), Ab(50.0)))
                    .pack(),
            ));
            
            // Version number
            parent.spawn((
                Text::new("Version 1.0.0"),
                UiTextSize::from(Em(1.2)),
                UiColor::from(TEXT_SECONDARY),
                UiLayout::window()
                    .size((Rl(100.0), Ab(40.0)))
                    .pos((Rl(0.0), Ab(120.0)))
                    .pack(),
            ));
            
            // Copyright
            parent.spawn((
                Text::new("© 2024 Action Items. All rights reserved."),
                UiTextSize::from(Em(0.9)),
                UiColor::from(TEXT_SECONDARY),
                UiLayout::window()
                    .size((Rl(100.0), Ab(30.0)))
                    .pos((Rl(0.0), Ab(170.0)))
                    .pack(),
            ));
            
            // Buttons
            let button_y = 220.0;
            let button_spacing = 50.0;
            
            // Visit Website button
            parent.spawn((
                UiLayout::window()
                    .size((Ab(150.0), Ab(40.0)))
                    .pos((Rl(50.0), Ab(button_y)))
                    .pack(),
                UiColor::from(BUTTON_PRIMARY),
                UiHover::new().forward_speed(8.0),
                UiClicked::new().forward_speed(12.0),
                Text::new("Visit Website"),
                UiTextSize::from(Em(1.0)),
                BorderRadius::all(Val::Px(6.0)),
                Pickable::default(),
                Name::new("VisitWebsiteButton"),
            ));
            
            // Send Feedback button
            parent.spawn((
                UiLayout::window()
                    .size((Ab(150.0), Ab(40.0)))
                    .pos((Rl(50.0), Ab(button_y + button_spacing)))
                    .pack(),
                UiColor::from(BUTTON_SECONDARY),
                UiHover::new().forward_speed(8.0),
                UiClicked::new().forward_speed(12.0),
                Text::new("Send Feedback"),
                UiTextSize::from(Em(1.0)),
                BorderRadius::all(Val::Px(6.0)),
                Pickable::default(),
                Name::new("SendFeedbackButton"),
            ));
            
            // Check for Updates button
            parent.spawn((
                UiLayout::window()
                    .size((Ab(150.0), Ab(40.0)))
                    .pos((Rl(50.0), Ab(button_y + button_spacing * 2.0)))
                    .pack(),
                UiColor::from(BUTTON_SECONDARY),
                UiHover::new().forward_speed(8.0),
                UiClicked::new().forward_speed(12.0),
                Text::new("Check for Updates"),
                UiTextSize::from(Em(1.0)),
                BorderRadius::all(Val::Px(6.0)),
                Pickable::default(),
                Name::new("CheckUpdatesButton"),
            ));
            
            // Acknowledgements section
            parent.spawn((
                Text::new("Acknowledgements"),
                UiTextSize::from(Em(1.3)),
                UiColor::from(TEXT_PRIMARY),
                UiLayout::window()
                    .size((Rl(90.0), Ab(30.0)))
                    .pos((Rl(5.0), Ab(button_y + button_spacing * 3.0 + 30.0)))
                    .pack(),
            ));
            
            parent.spawn((
                Text::new("Built with Bevy Engine and open source libraries"),
                UiTextSize::from(Em(0.9)),
                UiColor::from(TEXT_SECONDARY),
                UiLayout::window()
                    .size((Rl(90.0), Ab(30.0)))
                    .pos((Rl(5.0), Ab(button_y + button_spacing * 3.0 + 70.0)))
                    .pack(),
            ));
        });
    }
}

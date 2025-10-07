use action_items_ecs_ui::prelude::*;
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use crate::ui::{components::*, theme::*};

pub fn create_account_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        commands.entity(parent).with_children(|parent| {
            // Left sidebar (25% width)
            spawn_profile_sidebar(parent);

            // Right panel (75% width)
            spawn_features_panel(parent);
        });
    }
}

fn spawn_profile_sidebar(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        UiLayout::window()
            .size((Rl(25.0), Rl(100.0)))
            .pos((Rl(0.0), Ab(0.0)))
            .pack(),
        UiColor::from(SETTINGS_SIDEBAR_BG),
        UserProfileSidebar,
        Name::new("ProfileSidebar"),
    )).with_children(|sidebar| {
        // Profile photo (circular, 120x120px)
        sidebar.spawn((
            UiLayout::window()
                .size((Ab(120.0), Ab(120.0)))
                .pos((Rl(50.0), Ab(40.0)))
                .anchor(Anchor::TopCenter)
                .pack(),
            UiColor::from(PROFILE_PHOTO_BG),
            BorderRadius::all(Val::Px(60.0)),  // Make circular
            BorderColor::from(PROFILE_BORDER),
            ProfilePhoto {
                initials: "DM".to_string(),
                avatar_url: None,  // Phase 1: static, Phase 2: load from DB
            },
            Name::new("ProfilePhoto"),
        )).with_children(|photo| {
            // Show initials (Phase 1)
            photo.spawn((
                Text::new("DM"),
                UiTextSize::from(Em(2.0)),
                UiColor::from(TEXT_PRIMARY),
                UiLayout::window()
                    .size((Ab(120.0), Ab(120.0)))
                    .pos((Rl(50.0), Rl(50.0)))
                    .anchor(Anchor::Center)
                    .pack(),
                Name::new("ProfileInitials"),
            ));
        });

        // User name
        sidebar.spawn((
            Text::new("David Maple"),
            UiTextSize::from(Em(1.5)),
            UiColor::from(TEXT_PRIMARY),
            UiLayout::window()
                .size((Rl(90.0), Ab(30.0)))
                .pos((Rl(50.0), Ab(180.0)))
                .anchor(Anchor::TopCenter)
                .pack(),
            Name::new("UserName"),
        ));

        // User email/username
        sidebar.spawn((
            Text::new("kloudsamurai · david@cloudsamur.ai"),
            UiTextSize::from(Em(0.95)),
            UiColor::from(TEXT_SECONDARY),
            UiLayout::window()
                .size((Rl(90.0), Ab(25.0)))
                .pos((Rl(50.0), Ab(215.0)))
                .anchor(Anchor::TopCenter)
                .pack(),
            Name::new("UserEmail"),
        ));

        // Subscription status box
        sidebar.spawn((
            UiLayout::window()
                .size((Rl(85.0), Ab(80.0)))
                .pos((Rl(50.0), Ab(260.0)))
                .anchor(Anchor::TopCenter)
                .pack(),
            UiColor::from(STATUS_BOX_BG),
            BorderRadius::all(Val::Px(8.0)),
            SubscriptionStatusBox,
            Name::new("SubscriptionStatus"),
        )).with_children(|status_box| {
            status_box.spawn((
                Text::new("You are subscribed to Raycast Pro via a paid Team plan."),
                UiTextSize::from(Em(0.9)),
                UiColor::from(TEXT_SECONDARY),
                UiLayout::window()
                    .size((Rl(90.0), Ab(70.0)))
                    .pos((Rl(50.0), Rl(50.0)))
                    .anchor(Anchor::Center)
                    .pack(),
                Name::new("StatusText"),
            ));
        });

        // Log Out button (bottom of sidebar)
        sidebar.spawn((
            UiLayout::window()
                .size((Rl(85.0), Ab(40.0)))
                .pos((Rl(50.0), Rl(100.0) - Ab(60.0)))
                .anchor(Anchor::TopCenter)
                .pack(),
            UiColor::from(DESTRUCTIVE_BUTTON),
            UiHover::new().forward_speed(8.0),
            UiClicked::new().forward_speed(12.0),
            BorderRadius::all(Val::Px(6.0)),
            Text::new("Log Out"),
            UiTextSize::from(Em(1.0)),
            LogOutButton,
            Pickable::default(),
            Name::new("LogOutButton"),
        ));
    });
}

fn spawn_features_panel(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        UiLayout::window()
            .size((Rl(75.0), Rl(100.0)))
            .pos((Rl(25.0), Ab(0.0)))
            .pack(),
        Name::new("FeaturesPanel"),
    )).with_children(|panel| {
        let mut y_offset = 20.0;

        // Pro section
        y_offset = spawn_section_header(panel, "Pro", y_offset);
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "pro", feature_id: "raycast_ai", icon: "✨", label: "Raycast AI", show_pro_badge: true, show_info_icon: true, y_offset });
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "pro", feature_id: "cloud_sync", icon: "☁️", label: "Cloud Sync", show_pro_badge: true, show_info_icon: true, y_offset });
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "pro", feature_id: "custom_themes", icon: "🎨", label: "Custom Themes", show_pro_badge: true, show_info_icon: true, y_offset });
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "pro", feature_id: "clipboard_history", icon: "📋", label: "Unlimited Clipboard History", show_pro_badge: true, show_info_icon: true, y_offset });
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "pro", feature_id: "scheduled_exports", icon: "📤", label: "Scheduled Exports", show_pro_badge: true, show_info_icon: true, y_offset });
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "pro", feature_id: "translator", icon: "🌐", label: "Translator", show_pro_badge: true, show_info_icon: true, y_offset });
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "pro", feature_id: "window_mgmt", icon: "🪟", label: "Custom Window Management Commands", show_pro_badge: true, show_info_icon: true, y_offset });
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "pro", feature_id: "unlimited_notes", icon: "📝", label: "Unlimited Notes", show_pro_badge: true, show_info_icon: true, y_offset });

        y_offset += 20.0;

        // Organizations section
        y_offset = spawn_section_header(panel, "Organizations", y_offset);
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "organizations", feature_id: "private_ext", icon: "⚙️", label: "Private Extensions", show_pro_badge: false, show_info_icon: true, y_offset });
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "organizations", feature_id: "shared_quicklinks", icon: "🔗", label: "Shared Quicklinks", show_pro_badge: false, show_info_icon: true, y_offset });
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "organizations", feature_id: "shared_snippets", icon: "📄", label: "Shared Snippets", show_pro_badge: false, show_info_icon: true, y_offset });
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "organizations", feature_id: "pro_for_all", icon: "🌟", label: "Pro Features for All Members", show_pro_badge: true, show_info_icon: true, y_offset });

        y_offset += 20.0;

        // Developer section
        y_offset = spawn_section_header(panel, "Developer", y_offset);
        y_offset = spawn_feature_row(panel, FeatureRowParams { section: "developer", feature_id: "dev_api", icon: "🔑", label: "Developer API", show_pro_badge: false, show_info_icon: true, y_offset });
        spawn_feature_row(panel, FeatureRowParams { section: "developer", feature_id: "custom_ext", icon: "🔧", label: "Custom Extensions", show_pro_badge: false, show_info_icon: true, y_offset });

        // Manage Subscription button (bottom right)
        panel.spawn((
            UiLayout::window()
                .size((Ab(200.0), Ab(40.0)))
                .pos((Rl(100.0) - Ab(220.0), Rl(100.0) - Ab(60.0)))
                .pack(),
            UiColor::from(BUTTON_SECONDARY),
            UiHover::new().forward_speed(8.0),
            UiClicked::new().forward_speed(12.0),
            BorderRadius::all(Val::Px(6.0)),
            Text::new("Manage Subscription"),
            UiTextSize::from(Em(1.0)),
            ManageSubscriptionButton,
            Pickable::default(),
            Name::new("ManageSubscriptionButton"),
        ));
    });
}

fn spawn_section_header(parent: &mut ChildSpawnerCommands, title: &str, y_offset: f32) -> f32 {
    parent.spawn((
        Text::new(title),
        UiTextSize::from(Em(1.3)),
        UiColor::from(TEXT_PRIMARY),
        UiLayout::window()
            .size((Rl(90.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Name::new(format!("SectionHeader_{}", title)),
    ));
    y_offset + 40.0
}

/// Parameters for spawning a feature row to reduce argument count
struct FeatureRowParams<'a> {
    section: &'a str,
    feature_id: &'a str,
    icon: &'a str,
    label: &'a str,
    show_pro_badge: bool,
    show_info_icon: bool,
    y_offset: f32,
}

fn spawn_feature_row(
    parent: &mut ChildSpawnerCommands,
    params: FeatureRowParams<'_>,
) -> f32 {
    parent.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(35.0)))
            .pos((Rl(5.0), Ab(params.y_offset)))
            .pack(),
        FeatureRow {
            feature_id: params.feature_id.to_string(),
            section: params.section.to_string(),
        },
        Name::new(format!("FeatureRow_{}", params.feature_id)),
    )).with_children(|row| {
        // Icon (left)
        row.spawn((
            Text::new(params.icon),
            UiTextSize::from(Em(1.2)),
            UiLayout::window()
                .size((Ab(30.0), Ab(30.0)))
                .pos((Ab(0.0), Ab(2.0)))
                .pack(),
            Name::new(format!("Icon_{}", params.feature_id)),
        ));

        // Label text
        row.spawn((
            Text::new(params.label),
            UiTextSize::from(Em(1.0)),
            UiColor::from(TEXT_SECONDARY),
            UiLayout::window()
                .size((Rl(60.0), Ab(30.0)))
                .pos((Ab(30.0), Ab(5.0)))
                .pack(),
            Name::new(format!("Label_{}", params.feature_id)),
        ));

        // Pro badge (if applicable)
        if params.show_pro_badge {
            row.spawn((
                UiLayout::window()
                    .size((Ab(50.0), Ab(24.0)))
                    .pos((Rl(100.0) - Ab(80.0), Ab(5.0)))
                    .pack(),
                UiColor::from(PRO_BADGE_BG),
                BorderRadius::all(Val::Px(4.0)),
                ProBadge,
                Name::new(format!("ProBadge_{}", params.feature_id)),
            )).with_children(|badge| {
                badge.spawn((
                    Text::new("Pro"),
                    UiTextSize::from(Em(0.85)),
                    UiColor::from(PRO_BADGE_TEXT),
                    UiLayout::window()
                        .size((Ab(50.0), Ab(24.0)))
                        .pos((Rl(50.0), Rl(50.0)))
                        .anchor(Anchor::Center)
                        .pack(),
                ));
            });
        }

        // Info icon (far right)
        if params.show_info_icon {
            row.spawn((
                Text::new("ⓘ"),
                UiTextSize::from(Em(1.1)),
                UiColor::from(TEXT_SECONDARY),
                UiLayout::window()
                    .size((Ab(24.0), Ab(24.0)))
                    .pos((Rl(100.0) - Ab(24.0), Ab(5.0)))
                    .pack(),
                InfoIcon,
                Name::new(format!("InfoIcon_{}", params.feature_id)),
            ));
        }
    });

    params.y_offset + 40.0  // 35px row + 5px spacing
}

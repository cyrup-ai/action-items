use action_items_ecs_ui::prelude::*;
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use crate::ui::{components::*, theme::*};

pub fn create_organizations_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        // NESTED TWO-PANEL LAYOUT inside content area

        // 1. Left sidebar (~300px)
        let _sidebar_id = spawn_organizations_sidebar(commands, parent);

        // 2. Right main panel (remaining width)
        let _main_panel_id = spawn_organization_main_panel(commands, parent);
    }
}

/// Spawn left sidebar with org list + create button
fn spawn_organizations_sidebar(commands: &mut Commands, parent: Entity) -> Entity {
    let sidebar = commands.spawn((
        UiLayout::window()
            .size((Ab(300.0), Rl(100.0)))  // Fixed 300px width, full height
            .pos((Rl(0.0), Rl(0.0)))  // Top-left of content area
            .pack(),
        UiColor::from(SETTINGS_SIDEBAR_BG),  // Dark sidebar background
        OrganizationsSidebar,
        Name::new("OrgSidebar"),
    )).id();

    commands.entity(sidebar).insert(ChildOf(parent));

    commands.entity(sidebar).with_children(|sidebar| {
        // "Organizations" header
        sidebar.spawn((
            Text::new("Organizations"),
            UiTextSize::from(Em(1.3)),
            UiColor::from(TEXT_PRIMARY),
            UiLayout::window()
                .size((Rl(90.0), Ab(40.0)))
                .pos((Rl(5.0), Ab(15.0)))
                .pack(),
        ));

        // "Cyrup.ai" org item (selected)
        spawn_org_list_item(sidebar, "cyrup", "Cyrup.ai", true, 60.0);

        // "+ Create New Organization" button (bottom)
        sidebar.spawn((
            UiLayout::window()
                .size((Rl(90.0), Ab(40.0)))
                .pos((Rl(5.0), Rl(95.0)))  // Bottom-anchored
                .anchor(Anchor::BottomLeft)
                .pack(),
            UiColor::from(BUTTON_SECONDARY),
            UiHover::new().forward_speed(8.0),
            UiClicked::new().forward_speed(12.0),
            Text::new("+ Create New Organization"),
            UiTextSize::from(Em(0.95)),
            BorderRadius::all(Val::Px(6.0)),
            Pickable::default(),
            CreateOrgButton,
            Name::new("CreateOrgButton"),
        ));
    });

    sidebar
}

/// Spawn individual org list item
fn spawn_org_list_item(
    parent: &mut ChildSpawnerCommands,
    org_id: &str,
    org_name: &str,
    selected: bool,
    y_offset: f32
) {
    let item_bg = if selected {
        Color::srgba(0.20, 0.20, 0.22, 1.0)  // Slightly lighter when selected
    } else {
        Color::srgba(0.08, 0.08, 0.08, 0.0)  // Transparent when not selected
    };

    let container = parent.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(50.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        UiColor::from(item_bg),
        UiHover::new().forward_speed(8.0),
        UiClicked::new().forward_speed(12.0),
        BorderRadius::all(Val::Px(6.0)),
        Pickable::default(),
        OrganizationListItem {
            org_id: org_id.to_string(),
            selected,
        },
        Name::new(format!("OrgItem_{}", org_id)),
    )).id();

    parent.commands().entity(container).with_children(|item| {
        // Checkmark icon (if selected)
        if selected {
            item.spawn((
                Text::new("✓"),
                UiTextSize::from(Em(1.2)),
                UiColor::from(Color::srgba(0.3, 1.0, 0.5, 1.0)),  // Green checkmark
                UiLayout::window()
                    .size((Ab(30.0), Ab(30.0)))
                    .pos((Ab(10.0), Rl(50.0)))
                    .anchor(Anchor::CenterLeft)
                    .pack(),
            ));
        }

        // Org name
        item.spawn((
            Text::new(org_name),
            UiTextSize::from(Em(1.0)),
            UiColor::from(TEXT_PRIMARY),
            UiLayout::window()
                .size((Rl(60.0), Ab(30.0)))
                .pos((Ab(40.0), Rl(50.0)))
                .anchor(Anchor::CenterLeft)
                .pack(),
        ));

        // Settings gear icon (right side)
        item.spawn((
            Text::new("⚙"),
            UiTextSize::from(Em(1.2)),
            UiColor::from(TEXT_SECONDARY),
            UiLayout::window()
                .size((Ab(30.0), Ab(30.0)))
                .pos((Rl(95.0), Rl(50.0)))
                .anchor(Anchor::Center)
                .pack(),
        ));
    });
}

/// Spawn main panel with org details
fn spawn_organization_main_panel(commands: &mut Commands, parent: Entity) -> Entity {
    let main_panel = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Rl(100.0)))  // Fill remaining width
            .pos((Ab(300.0), Rl(0.0)))  // Start after sidebar
            .pack(),
        UiColor::from(Color::srgba(0.0, 0.0, 0.0, 0.0)),  // Transparent (uses content area bg)
        Name::new("OrgMainPanel"),
    )).id();

    commands.entity(main_panel).insert(ChildOf(parent));

    commands.entity(main_panel).with_children(|panel| {
        let mut y_offset = 60.0;

        // Organization logo (120x120px circular)
        panel.spawn((
            UiLayout::window()
                .size((Ab(120.0), Ab(120.0)))
                .pos((Rl(50.0), Ab(y_offset)))
                .anchor(Anchor::TopCenter)
                .pack(),
            UiColor::from(ORG_LOGO_BG),
            BorderRadius::all(Val::Px(60.0)),  // Circular (radius = half of size)
            BorderColor::from(ORG_LOGO_BORDER),
            Outline {
                width: Val::Px(3.0),
                offset: Val::Px(0.0),
                color: ORG_LOGO_BORDER,
            },
            OrganizationLogo { org_id: "cyrup".to_string() },
            Name::new("OrgLogo"),
        )).with_children(|logo| {
            // "C" letter in center (placeholder until real logo system)
            logo.spawn((
                Text::new("C"),
                UiTextSize::from(Em(3.0)),
                UiColor::from(TEXT_PRIMARY),
                UiLayout::window()
                    .size((Rl(100.0), Rl(100.0)))
                    .pos((Rl(50.0), Rl(50.0)))
                    .anchor(Anchor::Center)
                    .pack(),
            ));
        });
        y_offset += 140.0;

        // Organization name
        panel.spawn((
            Text::new("Cyrup.ai"),
            UiTextSize::from(Em(2.0)),
            UiColor::from(TEXT_PRIMARY),
            UiLayout::window()
                .size((Rl(80.0), Ab(50.0)))
                .pos((Rl(50.0), Ab(y_offset)))
                .anchor(Anchor::TopCenter)
                .pack(),
        ));
        y_offset += 60.0;

        // "Paid Plan" badge
        panel.spawn((
            UiLayout::window()
                .size((Ab(100.0), Ab(30.0)))
                .pos((Rl(50.0), Ab(y_offset)))
                .anchor(Anchor::TopCenter)
                .pack(),
            UiColor::from(ORG_BADGE_GREEN_BG),
            BorderRadius::all(Val::Px(15.0)),  // Pill shape
            PlanBadge { plan_type: "Paid Plan".to_string() },
            Name::new("PlanBadge"),
        )).with_children(|badge| {
            badge.spawn((
                Text::new("Paid Plan"),
                UiTextSize::from(Em(0.9)),
                UiColor::from(ORG_BADGE_GREEN_TEXT),
                UiLayout::window()
                    .size((Rl(100.0), Rl(100.0)))
                    .pos((Rl(50.0), Rl(50.0)))
                    .anchor(Anchor::Center)
                    .pack(),
            ));
        });
        y_offset += 50.0;

        // "Manage Subscription" button
        panel.spawn((
            UiLayout::window()
                .size((Ab(200.0), Ab(40.0)))
                .pos((Rl(50.0), Ab(y_offset)))
                .anchor(Anchor::TopCenter)
                .pack(),
            UiColor::from(BUTTON_PRIMARY),
            UiHover::new().forward_speed(8.0),
            UiClicked::new().forward_speed(12.0),
            Text::new("Manage Subscription"),
            UiTextSize::from(Em(1.0)),
            BorderRadius::all(Val::Px(6.0)),
            Pickable::default(),
            ManageOrgSubscriptionButton { org_id: "cyrup".to_string() },
            Name::new("ManageSubscriptionButton"),
        ));
        y_offset += 70.0;

        // Info card 1: Manage Organization
        y_offset = spawn_info_card(
            panel,
            "Manage Organization",
            "You can use the Manage Organization command to see who's part of your organization, reset the invite link and edit your organization details.",
            y_offset,
            vec![
                ("Manage Organization", "ManageOrgButton", "cyrup"),
                ("Edit Organization", "EditOrgButton", "cyrup"),
            ],
            false,  // Not danger zone
        );

        // Info card 2: Store
        y_offset = spawn_info_card(
            panel,
            "Store",
            "Extend Raycast with extensions from Cyrup.ai. Open the Store to see what is available.",
            y_offset,
            vec![
                ("Open Store", "OpenStoreButton", "cyrup"),
            ],
            false,
        );

        // Info card 3: Danger Zone
        spawn_info_card(
            panel,
            "Danger Zone",
            "If you leave the organization, all the commands that are connected to the organization will be removed from your account.",
            y_offset,
            vec![
                ("Leave Organization", "LeaveOrgButton", "cyrup"),
            ],
            true,  // IS danger zone
        );
    });

    main_panel
}

/// Spawn an info card with title, description, and buttons
fn spawn_info_card(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    description: &str,
    y_offset: f32,
    buttons: Vec<(&str, &str, &str)>,  // (label, component_type, org_id)
    is_danger: bool,
) -> f32 {
    let card_height = 140.0;
    let card_bg = CARD_BG;  // Same background for all cards
    let border_color = if is_danger {
        DANGER_BORDER  // Red border for danger zone
    } else {
        Color::srgba(0.0, 0.0, 0.0, 0.0)  // No border
    };

    let card = parent.spawn((
        UiLayout::window()
            .size((Rl(85.0), Ab(card_height)))
            .pos((Rl(50.0), Ab(y_offset)))
            .anchor(Anchor::TopCenter)
            .pack(),
        UiColor::from(card_bg),
        BorderRadius::all(Val::Px(8.0)),
        InfoCard,
        Name::new(format!("InfoCard_{}", title.replace(" ", ""))),
    )).id();

    if is_danger {
        parent.commands().entity(card).insert(Outline {
            width: Val::Px(2.0),
            offset: Val::Px(0.0),
            color: border_color,
        });
    }

    parent.commands().entity(card).with_children(|card| {
        // Title
        card.spawn((
            Text::new(title),
            UiTextSize::from(Em(1.2)),
            UiColor::from(TEXT_PRIMARY),
            UiLayout::window()
                .size((Rl(90.0), Ab(30.0)))
                .pos((Ab(20.0), Ab(20.0)))
                .pack(),
        ));

        // Description
        card.spawn((
            Text::new(description),
            UiTextSize::from(Em(0.9)),
            UiColor::from(TEXT_SECONDARY),
            UiLayout::window()
                .size((Rl(90.0), Ab(40.0)))
                .pos((Ab(20.0), Ab(50.0)))
                .pack(),
        ));

        // Buttons
        let button_y = 95.0;
        for (idx, (label, component_type, org_id)) in buttons.iter().enumerate() {
            let button_x = 20.0 + (idx as f32 * 180.0);
            let button_color = if is_danger {
                Color::srgba(0.8, 0.2, 0.2, 1.0)  // Red for danger buttons
            } else {
                BUTTON_SECONDARY
            };

            let button_entity = card.spawn((
                UiLayout::window()
                    .size((Ab(170.0), Ab(32.0)))
                    .pos((Ab(button_x), Ab(button_y)))
                    .pack(),
                UiColor::from(button_color),
                UiHover::new().forward_speed(8.0),
                UiClicked::new().forward_speed(12.0),
                Text::new(*label),
                UiTextSize::from(Em(0.95)),
                BorderRadius::all(Val::Px(6.0)),
                Pickable::default(),
                Name::new(format!("{}_{}", component_type, org_id)),
            )).id();

            // Add appropriate component marker
            match *component_type {
                "ManageOrgButton" => {
                    card.commands().entity(button_entity).insert(ManageOrgButton {
                        org_id: org_id.to_string()
                    });
                }
                "EditOrgButton" => {
                    card.commands().entity(button_entity).insert(EditOrgButton {
                        org_id: org_id.to_string()
                    });
                }
                "OpenStoreButton" => {
                    card.commands().entity(button_entity).insert(OpenStoreButton {
                        org_id: org_id.to_string()
                    });
                }
                "LeaveOrgButton" => {
                    card.commands().entity(button_entity).insert(LeaveOrgButton {
                        org_id: org_id.to_string()
                    });
                }
                _ => {}
            }
        }
    });

    y_offset + card_height + 30.0  // Return next y_offset
}

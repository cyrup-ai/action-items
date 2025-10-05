use action_items_ecs_ui::prelude::*;
use bevy::prelude::*;
use std::collections::HashMap;
use crate::{
    navigation::SettingsTab, 
    resources::SettingsUIEntities,
    ui::{components::*, theme::*, tabs::*}
};

/// Pre-allocate ALL settings UI entities once on startup
pub fn setup_settings_infrastructure(mut commands: Commands) {
    // 1. BACKDROP: Full-screen semi-transparent overlay
    let backdrop = commands.spawn((
        UiLayout::window()
            .size((Vw(100.0), Vh(100.0)))
            .pos((Vw(0.0), Vh(0.0)))
            .pack(),
        UiColor::from(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        Visibility::Hidden,
        SettingsBackdrop,
        Name::new("SettingsBackdrop"),
    )).id();

    // 2. MODAL ROOT: Centered container (80% width, 85% height)
    let modal_root = commands.spawn((
        UiLayout::window()
            .size((Vw(80.0), Vh(85.0)))
            .pos((Vw(50.0), Vh(50.0)))
            .anchor(Anchor::Center)
            .pack(),
        UiColor::from(SETTINGS_WINDOW_BG),
        Visibility::Hidden,
        SettingsModalRoot,
        Name::new("SettingsModal"),
    )).id();
    
    commands.entity(modal_root).insert(ChildOf(backdrop));

    // 3. TITLE BAR with close button
    let (title_bar, close_button) = spawn_title_bar(&mut commands, modal_root);

    // 4. SIDEBAR with tab buttons
    let (sidebar, tab_buttons) = spawn_sidebar(&mut commands, modal_root);

    // 5. CONTENT AREA
    let content_area = spawn_content_area(&mut commands, modal_root);

    // 6. PRE-SPAWN ALL TAB PANELS (hidden initially)
    let tab_panels = spawn_all_tab_panels(&mut commands, content_area);

    // 7. STORE IN RESOURCE
    commands.insert_resource(SettingsUIEntities {
        backdrop,
        modal_root,
        title_bar,
        close_button,
        sidebar,
        content_area,
        tab_buttons,
        tab_panels,
    });
}

/// Spawn title bar with close button
fn spawn_title_bar(commands: &mut Commands, parent: Entity) -> (Entity, Entity) {
    let title_bar = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(50.0)))
            .pos((Rl(0.0), Rl(0.0)))
            .pack(),
        UiColor::from(SETTINGS_SIDEBAR_BG),
        SettingsTitleBar,
        Name::new("TitleBar"),
    )).id();
    
    commands.entity(title_bar).insert(ChildOf(parent));

    commands.entity(title_bar).with_children(|parent| {
        parent.spawn((
            UiLayout::window()
                .size((Ab(300.0), Ab(40.0)))
                .pos((Rl(3.0), Rl(50.0)))
                .anchor(Anchor::CenterLeft)
                .pack(),
            Text::new("Settings"),
            UiTextSize::from(Em(1.3)),
            UiColor::from(TEXT_PRIMARY),
            Name::new("TitleText"),
        ));
    });

    let close_button = commands.spawn((
        UiLayout::window()
            .size((Ab(32.0), Ab(32.0)))
            .pos((Rl(96.0), Rl(50.0)))
            .anchor(Anchor::Center)
            .pack(),
        UiColor::from(Color::srgba(0.25, 0.25, 0.28, 1.0)),
        UiHover::new().forward_speed(8.0).backward_speed(4.0),
        UiClicked::new().forward_speed(12.0).backward_speed(6.0),
        BorderRadius::all(Val::Px(6.0)),
        Text::new("×"),
        UiTextSize::from(Em(1.8)),
        CloseSettingsButton,
        Pickable::default(),
        Interaction::None,
        Name::new("CloseButton"),
    )).id();
    
    commands.entity(close_button).insert(ChildOf(title_bar));

    (title_bar, close_button)
}

/// Spawn sidebar with all tab buttons
fn spawn_sidebar(commands: &mut Commands, parent: Entity) -> (Entity, HashMap<SettingsTab, Entity>) {
    let sidebar = commands.spawn((
        UiLayout::window()
            .size((Ab(SIDEBAR_WIDTH), Rl(100.0)))
            .pos((Rl(0.0), Ab(50.0)))
            .pack(),
        UiColor::from(SETTINGS_SIDEBAR_BG),
        SettingsSidebar,
        Name::new("Sidebar"),
    )).id();
    
    commands.entity(sidebar).insert(ChildOf(parent));

    let mut tab_buttons = HashMap::new();
    
    for (idx, tab) in SettingsTab::all().iter().enumerate() {
        let button = commands.spawn((
            UiLayout::window()
                .size((Rl(100.0), Ab(TAB_HEIGHT)))
                .pos((Rl(0.0), Ab((idx as f32) * TAB_HEIGHT)))
                .pack(),
            UiColor::from(TAB_INACTIVE),
            UiHover::new().forward_speed(8.0).backward_speed(4.0),
            UiClicked::new().forward_speed(12.0).backward_speed(6.0),
            SettingsTabButton { tab: *tab },
            Pickable::default(),
            Interaction::None,
            Text::new(tab.display_name()),
            UiTextSize::from(Em(1.0)),
            UiColor::from(TEXT_PRIMARY),
            Name::new(format!("TabButton_{:?}", tab)),
        )).id();
        
        commands.entity(button).insert(ChildOf(sidebar));
        tab_buttons.insert(*tab, button);
    }

    (sidebar, tab_buttons)
}

/// Spawn content area container
fn spawn_content_area(commands: &mut Commands, parent: Entity) -> Entity {
    let content_area = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Rl(100.0)))
            .pos((Ab(SIDEBAR_WIDTH), Ab(50.0)))
            .pack(),
        UiColor::from(SETTINGS_CONTENT_BG),
        SettingsContentArea {
            active_tab: SettingsTab::default(),
        },
        Name::new("ContentArea"),
    )).id();
    
    commands.entity(content_area).insert(ChildOf(parent));
    content_area
}

/// Pre-spawn ALL tab panels with content (KEY to zero-allocation switching)
fn spawn_all_tab_panels(
    commands: &mut Commands, 
    content_area: Entity
) -> HashMap<SettingsTab, Entity> {
    let mut panels = HashMap::new();
    let default_tab = SettingsTab::default();
    
    for tab in SettingsTab::all() {
        let panel = commands.spawn((
            UiLayout::window()
                .size((Rl(100.0), Rl(100.0)))
                .pos((Rl(0.0), Rl(0.0)))
                .pack(),
            if *tab == default_tab {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Name::new(format!("Panel_{:?}", tab)),
        )).id();
        
        commands.entity(panel).insert(ChildOf(content_area));
        
        // Call tab content generators from ui/tabs/*.rs
        match *tab {
            SettingsTab::General => create_general_tab()(commands, panel),
            SettingsTab::Extensions => create_extensions_tab()(commands, panel),
            SettingsTab::AI => create_ai_tab()(commands, panel),
            SettingsTab::CloudSync => create_cloud_sync_tab()(commands, panel),
            SettingsTab::Account => create_account_tab()(commands, panel),
            SettingsTab::Organizations => create_organizations_tab()(commands, panel),
            SettingsTab::Advanced => create_advanced_tab()(commands, panel),
            SettingsTab::About => create_about_tab()(commands, panel),
        }
        
        panels.insert(*tab, panel);
    }
    
    panels
}

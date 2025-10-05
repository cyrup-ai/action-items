use action_items_ecs_ui::prelude::*;
use action_items_ecs_user_settings::table_names::*;
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use crate::ui::{components::*, theme::*};

pub fn create_general_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        let mut y_offset = 20.0;
        
        commands.entity(parent).with_children(|parent| {
            // Startup section
            y_offset = create_section_header(parent, "Startup", y_offset);
            y_offset = create_form_row(
                parent,
                "Launch at Login",
                |p, y| create_checkbox(p, "launch_at_login", STARTUP_SETTINGS, false, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Global Hotkey",
                |p, y| create_hotkey_field(p, "hotkey", HOTKEY_SETTINGS, "⌘ Space", y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Show Menu Bar Icon",
                |p, y| create_checkbox(p, "show_menu_bar_icon", STARTUP_SETTINGS, true, y),
                y_offset
            );
            
            // Separator line
            parent.spawn((
                UiLayout::window()
                    .size((Rl(90.0), Ab(1.0)))
                    .pos((Rl(5.0), Ab(y_offset)))
                    .pack(),
                UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
                Name::new("SeparatorLine1"),
            ));
            y_offset += 20.0;
            
            y_offset += SECTION_SPACING;
            
            // Appearance section
            y_offset = create_section_header(parent, "Appearance", y_offset);
            y_offset = create_form_row(
                parent,
                "Text Size",
                |p, y| {
                    // Small "Aa" button
                    p.spawn((
                        UiLayout::window()
                            .size((Ab(50.0), Ab(35.0)))
                            .pos((Rl(CONTROL_OFFSET_PCT), Ab(y)))
                            .pack(),
                        UiColor::from(BUTTON_SECONDARY),
                        UiHover::new().forward_speed(8.0),
                        UiClicked::new().forward_speed(12.0),
                        Text::new("Aa"),
                        UiTextSize::from(Em(0.9)),
                        SettingControl {
                            field_name: "text_size".to_string(),
                            table: APPEARANCE_SETTINGS.to_string()
                        },
                        BorderRadius::all(Val::Px(6.0)),
                        Pickable::default(),
                        Name::new("TextSizeSmall"),
                    ));
                    
                    // Large "Aa" button
                    p.spawn((
                        UiLayout::window()
                            .size((Ab(50.0), Ab(35.0)))
                            .pos((Rl(CONTROL_OFFSET_PCT) + Ab(60.0), Ab(y)))
                            .pack(),
                        UiColor::from(BUTTON_PRIMARY),
                        UiHover::new().forward_speed(8.0),
                        UiClicked::new().forward_speed(12.0),
                        Text::new("Aa"),
                        UiTextSize::from(Em(1.3)),
                        SettingControl {
                            field_name: "text_size".to_string(),
                            table: APPEARANCE_SETTINGS.to_string()
                        },
                        BorderRadius::all(Val::Px(6.0)),
                        Pickable::default(),
                        Name::new("TextSizeLarge"),
                    ));
                },
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Theme",
                |p, y| create_dropdown_with_icon(
                    p, 
                    "theme_dark", 
                    APPEARANCE_SETTINGS, 
                    vec!["Raycast Dark", "High Contrast", "Nord"], 
                    0, 
                    y,
                    "🌙"
                ),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "",
                |p, y| create_dropdown_with_icon(
                    p,
                    "theme_light",
                    APPEARANCE_SETTINGS,
                    vec!["Raycast Light", "High Contrast", "Solarized"],
                    0,
                    y,
                    "☀️"
                ),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Follow System",
                |p, y| create_checkbox(p, "follow_system_appearance", APPEARANCE_SETTINGS, true, y),
                y_offset
            );
            
            // Theme Studio button
            parent.spawn((
                UiLayout::window()
                    .size((Rl(CONTROL_WIDTH_PCT), Ab(35.0)))
                    .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset)))
                    .pack(),
                UiColor::from(Color::srgba(0.25, 0.25, 0.28, 1.0)),
                UiHover::new().forward_speed(8.0),
                UiClicked::new().forward_speed(12.0),
                Text::new("Open Theme Studio"),
                UiTextSize::from(Em(1.0)),
                BorderRadius::all(Val::Px(6.0)),
                Pickable::default(),
                ThemeStudioButton,
                Name::new("ThemeStudioButton"),
            ));
            y_offset += 45.0;
            
            // Separator line
            parent.spawn((
                UiLayout::window()
                    .size((Rl(90.0), Ab(1.0)))
                    .pos((Rl(5.0), Ab(y_offset)))
                    .pack(),
                UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
                Name::new("SeparatorLine2"),
            ));
            y_offset += 20.0;
            
            y_offset += SECTION_SPACING;
            
            // Window Mode section
            y_offset = create_section_header(parent, "Window Mode", y_offset);
            
            // Default mode card
            parent.spawn((
                UiLayout::window()
                    .size((Ab(140.0), Ab(110.0)))
                    .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset)))
                    .pack(),
                UiColor::from(BUTTON_PRIMARY),
                UiHover::new().forward_speed(8.0),
                UiClicked::new().forward_speed(12.0),
                BorderRadius::all(Val::Px(8.0)),
                SettingControl {
                    field_name: "window_mode".to_string(),
                    table: APPEARANCE_SETTINGS.to_string()
                },
                WindowModeCard { mode: WindowMode::Default },
                Pickable::default(),
                Name::new("WindowModeDefault"),
            )).with_children(|card| {
                // Preview image placeholder
                card.spawn((
                    UiLayout::window()
                        .size((Ab(120.0), Ab(70.0)))
                        .pos((Ab(10.0), Ab(10.0)))
                        .pack(),
                    UiColor::from(Color::srgba(0.2, 0.2, 0.25, 1.0)),
                    Name::new("DefaultPreview"),
                ));
                
                // Label
                card.spawn((
                    UiLayout::window()
                        .size((Ab(120.0), Ab(25.0)))
                        .pos((Ab(10.0), Ab(85.0)))
                        .pack(),
                    Text::new("Default"),
                    UiTextSize::from(Em(0.95)),
                    Name::new("DefaultLabel"),
                ));
            });
            
            // Compact mode card
            parent.spawn((
                UiLayout::window()
                    .size((Ab(140.0), Ab(110.0)))
                    .pos((Rl(CONTROL_OFFSET_PCT) + Ab(150.0), Ab(y_offset)))
                    .pack(),
                UiColor::from(BUTTON_SECONDARY),
                UiHover::new().forward_speed(8.0),
                UiClicked::new().forward_speed(12.0),
                BorderRadius::all(Val::Px(8.0)),
                SettingControl {
                    field_name: "window_mode".to_string(),
                    table: APPEARANCE_SETTINGS.to_string()
                },
                WindowModeCard { mode: WindowMode::Compact },
                Pickable::default(),
                Name::new("WindowModeCompact"),
            )).with_children(|card| {
                // Preview image placeholder
                card.spawn((
                    UiLayout::window()
                        .size((Ab(120.0), Ab(70.0)))
                        .pos((Ab(10.0), Ab(10.0)))
                        .pack(),
                    UiColor::from(Color::srgba(0.2, 0.2, 0.25, 1.0)),
                    Name::new("CompactPreview"),
                ));
                
                // Label
                card.spawn((
                    UiLayout::window()
                        .size((Ab(120.0), Ab(25.0)))
                        .pos((Ab(10.0), Ab(85.0)))
                        .pack(),
                    Text::new("Compact"),
                    UiTextSize::from(Em(0.95)),
                    Name::new("CompactLabel"),
                ));
            });
            
            y_offset += 120.0;
            
            // Separator line
            parent.spawn((
                UiLayout::window()
                    .size((Rl(90.0), Ab(1.0)))
                    .pos((Rl(5.0), Ab(y_offset)))
                    .pack(),
                UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
                Name::new("SeparatorLine3"),
            ));
            y_offset += 20.0;
            
            y_offset += SECTION_SPACING;
            
            // Favorites section
            y_offset = create_section_header(parent, "Favorites", y_offset);
            y_offset = create_form_row(
                parent,
                "Show favorites in compact mode",
                |p, y| create_checkbox(p, "show_favorites_in_compact", APPEARANCE_SETTINGS, true, y),
                y_offset
            );
        });
    }
}

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

fn create_hotkey_field(parent: &mut ChildSpawnerCommands, field_name: &str, table: &str, current: &str, y_offset: f32) {
    parent.spawn((
        UiLayout::window()
            .size((Rl(CONTROL_WIDTH_PCT), Ab(30.0)))
            .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset)))
            .pack(),
        UiColor::from(INPUT_BG),
        UiHover::new(),
        UiClicked::new(),
        Text::new(current),
        HotkeyRecorder {
            field_name: field_name.to_string(),
            current_combo: current.to_string(),
            is_recording: false
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

fn create_dropdown_with_icon(
    parent: &mut ChildSpawnerCommands, 
    field_name: &str, 
    table: &str, 
    options: Vec<&str>, 
    selected: usize, 
    y_offset: f32,
    icon: &str
) {
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
        Text::new(format!("{} {}", icon, selected_text)),
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

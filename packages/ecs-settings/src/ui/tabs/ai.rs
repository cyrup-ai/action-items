use action_items_ecs_ui::prelude::*;
use action_items_ecs_user_settings::table_names::*;
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use crate::ui::{components::*, theme::*};

pub fn create_ai_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        let mut y_offset = 20.0;
        
        commands.entity(parent).with_children(|parent| {
            // Master AI toggle
            y_offset = create_section_header(parent, "AI Assistant", y_offset);
            y_offset = create_form_row(
                parent,
                "Enable AI",
                |p, y| create_checkbox(p, "ai_enabled", AI_SETTINGS, false, y),
                y_offset
            );
            
            y_offset += SECTION_SPACING;
            
            // Quick AI section
            y_offset = create_section_header(parent, "Quick AI", y_offset);
            y_offset = create_form_row(
                parent,
                "Model",
                |p, y| create_dropdown(p, "quick_ai_model", AI_SETTINGS, vec!["GPT-4", "Claude", "Gemini"], 0, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Enable Quick AI",
                |p, y| create_checkbox(p, "quick_ai_enabled", AI_SETTINGS, true, y),
                y_offset
            );
            
            y_offset += SECTION_SPACING;
            
            // AI Chat section
            y_offset = create_section_header(parent, "AI Chat", y_offset);
            y_offset = create_form_row(
                parent,
                "Model",
                |p, y| create_dropdown(p, "chat_model", AI_SETTINGS, vec!["GPT-4", "Claude", "Gemini"], 1, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Temperature",
                |p, y| create_dropdown(p, "chat_temperature", AI_SETTINGS, vec!["0.3", "0.7", "1.0"], 1, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Max Tokens",
                |p, y| create_dropdown(p, "chat_max_tokens", AI_SETTINGS, vec!["1024", "2048", "4096"], 1, y),
                y_offset
            );
            
            y_offset += SECTION_SPACING;
            
            // AI Commands section
            y_offset = create_section_header(parent, "AI Commands", y_offset);
            y_offset = create_form_row(
                parent,
                "Model",
                |p, y| create_dropdown(p, "commands_model", AI_SETTINGS, vec!["GPT-4", "Claude", "Gemini"], 0, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Context Window",
                |p, y| create_dropdown(p, "commands_context", AI_SETTINGS, vec!["4K", "8K", "16K"], 1, y),
                y_offset
            );
            
            y_offset += SECTION_SPACING;
            
            // Ollama integration
            y_offset = create_section_header(parent, "Ollama Integration", y_offset);
            y_offset = create_form_row(
                parent,
                "Host URL",
                |p, y| create_text_input(p, "ollama_host", "http://localhost:11434", y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Enable Ollama",
                |p, y| create_checkbox(p, "ollama_enabled", AI_SETTINGS, false, y),
                y_offset
            );
            
            y_offset += SECTION_SPACING;
            
            // Experiments section
            y_offset = create_section_header(parent, "Experiments", y_offset);
            y_offset = create_form_row(
                parent,
                "Code Generation",
                |p, y| create_checkbox(p, "experiment_code_gen", AI_SETTINGS, false, y),
                y_offset
            );
            y_offset = create_form_row(
                parent,
                "Voice Input",
                |p, y| create_checkbox(p, "experiment_voice", AI_SETTINGS, false, y),
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

fn create_text_input(parent: &mut ChildSpawnerCommands, field_name: &str, placeholder: &str, y_offset: f32) {
    parent.spawn((
        UiLayout::window()
            .size((Rl(CONTROL_WIDTH_PCT), Ab(30.0)))
            .pos((Rl(CONTROL_OFFSET_PCT), Ab(y_offset)))
            .pack(),
        UiColor::from(INPUT_BG),
        UiHover::new(),
        Text::new(placeholder),
        TextInput {
            field_name: field_name.to_string(),
            value: placeholder.to_string()
        },
        SettingControl {
            field_name: field_name.to_string(),
            table: AI_SETTINGS.to_string()
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

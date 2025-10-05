use action_items_ecs_ui::prelude::*;
use action_items_ecs_ui::theme::Theme;
use bevy::prelude::*;
use crate::ui::components::*;

/// Generate preferences window (centered overlay)
pub fn create_preferences_window(
    commands: &mut Commands,
    theme: &Theme,
) -> Entity {
    let bg_color = theme.colors.background_primary;
    
    // Main container (centered overlay)
    let container = commands.spawn((
        UiLayout::window()
            .size((Vw(60.0), Vh(70.0)))
            .pos((Vw(50.0), Vh(50.0)))
            .anchor(Anchor::Center)
            .pack(),
        UiColor::from(bg_color),
        BorderRadius::all(Val::Px(12.0)),
        PreferencesContainer,
        Name::new("PreferencesWindow"),
    )).id();
    
    // Header section
    let header = create_header(commands, theme);
    commands.entity(container).add_children(&[header]);
    
    // Content section
    let content = create_content(commands, theme);
    commands.entity(container).add_children(&[content]);
    
    // Actions section
    let actions = create_actions(commands, theme);
    commands.entity(container).add_children(&[actions]);
    
    container
}

/// Create header with title and close button
fn create_header(
    commands: &mut Commands,
    theme: &Theme,
) -> Entity {
    let header_bg = theme.colors.surface_default;
    let text_color = theme.colors.text_primary;
    
    let header = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(60.0)))
            .pos((Ab(0.0), Ab(0.0)))
            .pack(),
        UiColor::from(header_bg),
        Name::new("Header"),
    )).id();
    
    // Title
    let title = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(30.0)))
            .pos((Ab(20.0), Ab(15.0)))
            .pack(),
        Text::new("Preferences"),
        UiTextSize::from(Em(1.5)),
        TextColor(text_color),
        Name::new("Title"),
    )).id();
    
    // Close button (top right)
    let close_btn = commands.spawn((
        UiLayout::window()
            .size((Ab(30.0), Ab(30.0)))
            .pos((Rl(95.0), Ab(15.0)))
            .anchor(Anchor::TopRight)
            .pack(),
        UiColor::from(theme.colors.surface_hover),
        UiHover::new().forward_speed(8.0).backward_speed(5.0),
        UiClicked::new().forward_speed(15.0).backward_speed(8.0),
        BorderRadius::all(Val::Px(4.0)),
        PreferencesCloseButton,
        Pickable::default(),
        Interaction::default(),
        Text::new("✕"),
        UiTextSize::from(Em(1.2)),
        TextColor(text_color),
        Name::new("CloseButton"),
    )).id();
    
    commands.entity(header).add_children(&[title, close_btn]);
    header
}

/// Create content area with hotkey display and recorder
fn create_content(
    commands: &mut Commands,
    theme: &Theme,
) -> Entity {
    let text_color = theme.colors.text_primary;
    let secondary_color = theme.colors.text_secondary;
    let button_bg = theme.colors.surface_selected;
    
    let content = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Rl(75.0)))
            .pos((Ab(0.0), Ab(60.0)))
            .pack(),
        Name::new("Content"),
    )).id();
    
    // Current hotkey display
    let hotkey_display = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(40.0)))
            .pos((Rl(5.0), Ab(20.0)))
            .pack(),
        Text::new("Current: None"),
        UiTextSize::from(Em(1.1)),
        TextColor(secondary_color),
        HotkeyDisplay { current_hotkey: None },
        Name::new("HotkeyDisplay"),
    )).id();
    
    // Recorder button
    let recorder_btn = commands.spawn((
        UiLayout::window()
            .size((Ab(200.0), Ab(40.0)))
            .pos((Rl(5.0), Ab(80.0)))
            .pack(),
        UiColor::from(button_bg),
        UiHover::new().forward_speed(10.0).backward_speed(5.0),
        UiClicked::new().forward_speed(15.0).backward_speed(8.0),
        BorderRadius::all(Val::Px(6.0)),
        HotkeyRecorderButton,
        Pickable::default(),
        Interaction::default(),
        Text::new("Record New Hotkey"),
        UiTextSize::from(Em(1.0)),
        TextColor(text_color),
        Name::new("RecorderButton"),
    )).id();
    
    // Status area
    let status_area = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(60.0)))
            .pos((Rl(5.0), Ab(140.0)))
            .pack(),
        Name::new("StatusArea"),
    )).id();
    
    commands.entity(content).add_children(&[hotkey_display, recorder_btn, status_area]);
    content
}

/// Create actions with Save/Cancel buttons
fn create_actions(
    commands: &mut Commands,
    theme: &Theme,
) -> Entity {
    let text_color = theme.colors.text_primary;
    let primary_btn = theme.colors.surface_selected;
    let secondary_btn = theme.colors.surface_hover;
    
    let actions = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(60.0)))
            .pos((Ab(0.0), Rl(92.0)))
            .anchor(Anchor::BottomLeft)
            .pack(),
        Name::new("Actions"),
    )).id();
    
    // Cancel button (left)
    let cancel_btn = commands.spawn((
        UiLayout::window()
            .size((Ab(100.0), Ab(35.0)))
            .pos((Rl(70.0), Ab(12.0)))
            .pack(),
        UiColor::from(secondary_btn),
        UiHover::new().forward_speed(10.0).backward_speed(5.0),
        UiClicked::new().forward_speed(15.0).backward_speed(8.0),
        BorderRadius::all(Val::Px(6.0)),
        PreferencesCancelButton,
        Pickable::default(),
        Interaction::default(),
        Text::new("Cancel"),
        UiTextSize::from(Em(1.0)),
        TextColor(text_color),
        Name::new("CancelButton"),
    )).id();
    
    // Save button (right)
    let save_btn = commands.spawn((
        UiLayout::window()
            .size((Ab(100.0), Ab(35.0)))
            .pos((Rl(85.0), Ab(12.0)))
            .pack(),
        UiColor::from(primary_btn),
        UiHover::new().forward_speed(10.0).backward_speed(5.0),
        UiClicked::new().forward_speed(15.0).backward_speed(8.0),
        BorderRadius::all(Val::Px(6.0)),
        PreferencesSaveButton,
        Pickable::default(),
        Interaction::default(),
        Text::new("Save"),
        UiTextSize::from(Em(1.0)),
        TextColor(text_color),
        Name::new("SaveButton"),
    )).id();
    
    commands.entity(actions).add_children(&[cancel_btn, save_btn]);
    actions
}

/// Create recording overlay (shown during hotkey capture)
pub fn create_recording_overlay(
    commands: &mut Commands,
    theme: &Theme,
) -> Entity {
    let overlay_bg = Color::srgba(0.0, 0.0, 0.0, 0.7);
    let text_color = theme.colors.text_primary;
    
    let overlay = commands.spawn((
        UiLayout::window()
            .size((Vw(100.0), Vh(100.0)))
            .pos((Vw(0.0), Vh(0.0)))
            .pack(),
        UiColor::from(overlay_bg),
        HotkeyRecordingOverlay,
        Name::new("RecordingOverlay"),
    )).id();
    
    let message = commands.spawn((
        UiLayout::window()
            .size((Ab(400.0), Ab(60.0)))
            .pos((Vw(50.0), Vh(50.0)))
            .anchor(Anchor::Center)
            .pack(),
        Text::new("Recording... Press keys or ESC to cancel"),
        UiTextSize::from(Em(1.5)),
        TextColor(text_color),
        Name::new("RecordingMessage"),
    )).id();
    
    commands.entity(overlay).add_children(&[message]);
    overlay
}

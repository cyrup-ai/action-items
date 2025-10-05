use action_items_ecs_ui::prelude::*;
use bevy::prelude::*;
use serde_json::Value;
use crate::{events::*, resources::*, navigation::SettingsTab};
use crate::ui::components::*;
use ecs_service_bridge::components::{PluginComponent, PluginStatus, PluginType};
use uuid::Uuid;
use super::theme::*;

/// Sync SettingsResource visibility state to window Visibility component
pub fn sync_window_visibility(
    settings: Res<SettingsResource>,
    mut query: Query<&mut Visibility, With<SettingsWindow>>,
) {
    if settings.is_changed() {
        if let Ok(mut visibility) = query.single_mut() {
            *visibility = if settings.is_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

/// Handle tab button clicks
pub fn handle_tab_clicks(
    query: Query<(&SettingsTabButton, &Interaction), Changed<Interaction>>,
    mut events: EventWriter<TabChangeRequested>,
    resource: Res<SettingsResource>,
) {
    for (button, interaction) in &query {
        if *interaction == Interaction::Pressed {
            events.write(TabChangeRequested::new(
                resource.current_tab,
                button.tab,
                "ui_tab_button",
            ));
        }
    }
}

/// Update tab button visual states
pub fn update_tab_states(
    resource: Res<SettingsResource>,
    mut query: Query<(&SettingsTabButton, &mut UiColor)>,
) {
    if resource.is_changed() {
        for (button, mut color) in &mut query {
            if button.tab == resource.current_tab {
                *color = UiColor::from(super::theme::TAB_ACTIVE);
            } else {
                *color = UiColor::from(super::theme::TAB_INACTIVE);
            }
        }
    }
}

/// Switch tabs by toggling visibility (ZERO entity allocation)
/// Replaces the old update_content_area which despawned/spawned entities
pub fn switch_tab_visibility(
    mut events: EventReader<TabChanged>,
    entities: Res<SettingsUIEntities>,
    mut visibility: Query<&mut Visibility>,
) {
    for event in events.read() {
        // Hide ALL tab panels
        for (_, panel_entity) in entities.tab_panels.iter() {
            if let Ok(mut vis) = visibility.get_mut(*panel_entity) {
                *vis = Visibility::Hidden;
            }
        }
        
        // Show ONLY the active tab panel
        if let Some(active_panel) = entities.tab_panels.get(&event.tab) {
            if let Ok(mut vis) = visibility.get_mut(*active_panel) {
                *vis = Visibility::Visible;
            }
        }
    }
}

/// Handle Arrow Left/Right for tab navigation
/// Only active when settings modal is visible
pub fn handle_keyboard_tab_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    settings: Res<SettingsResource>,
    entities: Res<SettingsUIEntities>,
    visibility: Query<&Visibility, With<SettingsModalRoot>>,
    mut tab_change: EventWriter<TabChangeRequested>,
) {
    // Only process keys when modal is visible
    let is_visible = visibility
        .get(entities.modal_root)
        .map(|v| *v == Visibility::Visible)
        .unwrap_or(false);
    
    if !is_visible {
        return;
    }

    let current_tab = settings.current_tab;
    let all_tabs = SettingsTab::all();
    let current_idx = all_tabs
        .iter()
        .position(|t| *t == current_tab)
        .unwrap_or(0);

    let new_tab = if keyboard.just_pressed(KeyCode::ArrowRight) {
        // Next tab (wrap around)
        let next_idx = (current_idx + 1) % all_tabs.len();
        Some(all_tabs[next_idx])
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
        // Previous tab (wrap around)
        let prev_idx = if current_idx == 0 {
            all_tabs.len() - 1
        } else {
            current_idx - 1
        };
        Some(all_tabs[prev_idx])
    } else {
        None
    };

    if let Some(tab) = new_tab {
        tab_change.write(TabChangeRequested::new(
            current_tab,
            tab,
            "keyboard_navigation",
        ));
    }
}

/// Handle Escape key to close settings
pub fn handle_escape_close(
    keyboard: Res<ButtonInput<KeyCode>>,
    entities: Res<SettingsUIEntities>,
    visibility: Query<&Visibility, With<SettingsModalRoot>>,
    mut close_events: EventWriter<SettingsCloseRequested>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        let is_visible = visibility
            .get(entities.modal_root)
            .map(|v| *v == Visibility::Visible)
            .unwrap_or(false);
        
        if is_visible {
            close_events.write(SettingsCloseRequested::new());
        }
    }
}

/// Handle checkbox toggle clicks
pub fn handle_checkbox_changes(
    mut query: Query<(&SettingControl, &mut SettingCheckbox, &Interaction), 
        (Changed<Interaction>, With<SettingCheckbox>)>,
    mut write_events: EventWriter<SettingUpdateRequested>,
    resource: Res<SettingsResource>,
) {
    for (control, mut checkbox, interaction) in &mut query {
        if *interaction == Interaction::Pressed {
            checkbox.checked = !checkbox.checked;
            write_events.write(SettingUpdateRequested::new(
                resource.current_tab,
                control.table.clone(),
                control.field_name.clone(),
                Value::Bool(checkbox.checked),
                "ui_checkbox",
            ));
        }
    }
}

/// Handle text input changes
pub fn handle_text_input_changes(
    query: Query<(&SettingControl, &TextInput), Changed<TextInput>>,
    mut write_events: EventWriter<SettingUpdateRequested>,
    resource: Res<SettingsResource>,
) {
    for (control, input) in &query {
        write_events.write(SettingUpdateRequested::new(
            resource.current_tab,
            control.table.clone(),
            control.field_name.clone(),
            Value::String(input.value.clone()),
            "ui_text_input",
        ));
    }
}

/// Handle dropdown selection changes
pub fn handle_dropdown_changes(
    mut query: Query<(&SettingControl, &mut DropdownControl, &Interaction), 
        (Changed<Interaction>, With<DropdownControl>)>,
    mut write_events: EventWriter<SettingUpdateRequested>,
    resource: Res<SettingsResource>,
) {
    for (control, mut dropdown, interaction) in &mut query {
        if *interaction == Interaction::Pressed {
            dropdown.is_open = !dropdown.is_open;
            if let Some(selected_value) = dropdown.options.get(dropdown.selected) {
                write_events.write(SettingUpdateRequested::new(
                    resource.current_tab,
                    control.table.clone(),
                    control.field_name.clone(),
                    Value::String(selected_value.clone()),
                    "ui_dropdown",
                ));
            }
        }
    }
}

/// Helper function to get plugin type icon
fn plugin_type_icon(plugin_type: &PluginType) -> &'static str {
    match plugin_type {
        PluginType::Deno => "🦕",
        PluginType::Native => "⚙️",
        PluginType::Wasm => "🔌",
        PluginType::Raycast => "🚀",
    }
}

/// Populate extension table with plugin cards
pub fn populate_extension_table(
    mut commands: Commands,
    plugins: Query<&PluginComponent, Added<PluginComponent>>,
    containers: Query<Entity, With<ExtensionsTableContainer>>,
    existing_rows: Query<&ExtensionRow>,
) {
    if plugins.is_empty() {
        return;
    }

    for container_entity in containers.iter() {
        let mut y_offset = 10.0;
        
        for plugin in plugins.iter() {
            // Skip if already exists
            if existing_rows.iter().any(|row| row.plugin_id == plugin.plugin_id) {
                continue;
            }
            
            let enabled = matches!(plugin.status, PluginStatus::Active);
            
            // Spawn extension CARD (not row)
            let card = commands.spawn((
                ExtensionRow { plugin_id: plugin.plugin_id.clone() },
                UiLayout::window()
                    .size((Rl(100.0), Ab(90.0)))  // Card height
                    .pos((Rl(0.0), Ab(y_offset)))
                    .pack(),
                UiColor::from(CARD_BG),
                BorderRadius::all(Val::Px(8.0)),
                Name::new(format!("ExtensionCard_{}", plugin.plugin_id)),
            )).id();
            
            commands.entity(container_entity).add_child(card);
            
            // Card contents
            commands.entity(card).with_children(|card| {
                // Icon + Name + Version (top row)
                card.spawn((
                    UiLayout::window()
                        .size((Rl(70.0), Ab(30.0)))
                        .pos((Ab(15.0), Ab(15.0)))
                        .pack(),
                    Text::new(format!(
                        "{} {}  v{}",
                        plugin_type_icon(&plugin.plugin_type),
                        plugin.name,
                        plugin.version
                    )),
                    UiTextSize::from(Em(1.1)),
                    UiColor::from(TEXT_PRIMARY),
                ));
                
                // Description (second row)
                card.spawn((
                    UiLayout::window()
                        .size((Rl(85.0), Ab(35.0)))
                        .pos((Ab(15.0), Ab(50.0)))
                        .pack(),
                    Text::new(&plugin.description),
                    UiTextSize::from(Em(0.9)),
                    UiColor::from(TEXT_SECONDARY),
                ));
                
                // Toggle (top right)
                card.spawn((
                    ExtensionToggle {
                        plugin_id: plugin.plugin_id.clone(),
                        enabled,
                    },
                    UiLayout::window()
                        .size((Ab(44.0), Ab(24.0)))
                        .pos((Rl(95.0), Ab(15.0)))
                        .anchor(Anchor::TopRight)
                        .pack(),
                    UiColor::from(if enabled { TOGGLE_ON } else { TOGGLE_OFF }),
                    BorderRadius::all(Val::Px(12.0)),
                    UiHover::new().forward_speed(8.0),
                    UiClicked::new().forward_speed(12.0),
                    Pickable::default(),
                    Interaction::None,
                    Name::new(format!("Toggle_{}", plugin.plugin_id)),
                ));
                
                // Settings button (bottom right) - only if plugin.has_config
                if plugin.has_config {
                    card.spawn((
                        ExtensionSettingsButton {
                            plugin_id: plugin.plugin_id.clone(),
                        },
                        UiLayout::window()
                            .size((Ab(100.0), Ab(28.0)))
                            .pos((Rl(95.0), Ab(80.0)))
                            .anchor(Anchor::BottomRight)
                            .pack(),
                        UiColor::from(BUTTON_SECONDARY),
                        UiHover::new().forward_speed(8.0).backward_speed(4.0),
                        UiClicked::new().forward_speed(15.0).backward_speed(10.0),
                        Text::new("⚙️ Settings"),
                        UiTextSize::from(Em(0.85)),
                        BorderRadius::all(Val::Px(6.0)),
                        Pickable::default(),
                        Interaction::None,
                        Name::new(format!("SettingsButton_{}", plugin.plugin_id)),
                    ));
                }
            });
            
            y_offset += 100.0;  // Card height + spacing
        }
    }
}

/// Handle extension toggle clicks
pub fn handle_extension_toggle(
    mut query: Query<(&mut ExtensionToggle, &Interaction, &mut UiColor), Changed<Interaction>>,
    mut events: EventWriter<ExtensionToggled>,
) {
    for (mut toggle, interaction, mut color) in query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let new_enabled = !toggle.enabled;
            toggle.enabled = new_enabled;
            *color = UiColor::from(if new_enabled {
                CHECKBOX_CHECKED 
            } else { 
                CHECKBOX_BG 
            });
            
            events.write(ExtensionToggled {
                operation_id: Uuid::new_v4(),
                extension_id: toggle.plugin_id.clone(),
                enabled: new_enabled,
                requester: "extensions_tab".to_string(),
            });
        }
    }
}

/// Display validation errors next to controls
pub fn display_setting_errors(
    mut commands: Commands,
    mut error_events: EventReader<SettingValidationFailed>,
    error_displays: Query<(Entity, &SettingErrorDisplay)>,
    mut visibility_query: Query<&mut Visibility>,
    mut text_query: Query<&mut Text>,
) {
    for event in error_events.read() {
        // Find error display for this field
        for (entity, display) in &error_displays {
            if display.field_name == event.field_name {
                // Show error
                if let Ok(mut visibility) = visibility_query.get_mut(entity) {
                    *visibility = Visibility::Visible;
                }
                
                // Update error text
                if let Ok(mut text) = text_query.get_mut(entity) {
                    text.0 = event.error.clone();
                }
                
                // Add timeout component
                commands.entity(entity).insert(ErrorMessage {
                    timeout: Timer::from_seconds(5.0, TimerMode::Once),
                });
            }
        }
    }
}

/// Auto-hide error messages after timeout
pub fn auto_hide_errors(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ErrorMessage, &mut Visibility)>,
) {
    for (entity, mut error_msg, mut visibility) in &mut query {
        error_msg.timeout.tick(time.delta());
        
        if error_msg.timeout.finished() {
            *visibility = Visibility::Hidden;
            commands.entity(entity).remove::<ErrorMessage>();
        }
    }
}

/// Visual feedback when settings are saved successfully
pub fn setting_save_feedback(
    mut commands: Commands,
    mut update_events: EventReader<SettingUpdated>,
    controls: Query<(Entity, &SettingControl, &UiColor)>,
) {
    use action_items_ecs_ui::state::UiBase;
    use action_items_ecs_user_settings::table_names::*;

    for event in update_events.read() {
        // Map tab → possible tables (some tabs use multiple tables)
        let target_tables: Vec<&str> = match event.tab {
            SettingsTab::General => vec![STARTUP_SETTINGS, APPEARANCE_SETTINGS, HOTKEY_SETTINGS],
            SettingsTab::AI => vec![AI_SETTINGS],
            SettingsTab::Advanced => vec![ADVANCED_SETTINGS],
            SettingsTab::CloudSync => vec![CLOUD_SYNC_SETTINGS],
            SettingsTab::Account => vec![ACCOUNT_SETTINGS],
            SettingsTab::Organizations => vec![ORGANIZATION_SETTINGS],
            _ => continue,
        };

        for (entity, control, color) in &controls {
            if control.field_name == event.field_name 
                && target_tables.contains(&control.table.as_str()) {
                // Get base color or use default
                let base_color = color.colors.get(&UiBase::id())
                    .copied()
                    .unwrap_or(Color::srgba(0.2, 0.2, 0.2, 1.0));

                // Store original color and add feedback component
                commands.entity(entity).insert(SaveSuccessFeedback {
                    timer: Timer::from_seconds(0.8, TimerMode::Once),
                    original_color: base_color,
                });
            }
        }
    }
}

/// Animate save success feedback (green flash)
pub fn animate_save_feedback(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SaveSuccessFeedback, &mut UiColor)>,
) {
    use action_items_ecs_ui::state::UiBase;
    
    for (entity, mut feedback, mut color) in &mut query {
        feedback.timer.tick(time.delta());
        
        let progress = feedback.timer.fraction();
        let success_color = Color::srgba(0.3, 0.8, 0.3, 1.0);
        
        let new_color = if progress < 0.3 {
            // Flash to green
            let t = progress / 0.3;
            feedback.original_color.mix(&success_color, t)
        } else {
            // Fade back to original
            let t = (progress - 0.3) / 0.7;
            success_color.mix(&feedback.original_color, t)
        };
        
        // Update the base state color
        color.colors.insert(UiBase::id(), new_color);
        
        if feedback.timer.finished() {
            color.colors.insert(UiBase::id(), feedback.original_color);
            commands.entity(entity).remove::<SaveSuccessFeedback>();
        }
    }
}

/// Handle window mode card selection
pub fn handle_window_mode_selection(
    query: Query<(&WindowModeCard, &Interaction, &SettingControl), Changed<Interaction>>,
    mut write_events: EventWriter<SettingUpdateRequested>,
    mut all_cards: Query<(&WindowModeCard, &mut UiColor)>,
    resource: Res<SettingsResource>,
) {
    for (card, interaction, control) in query.iter() {
        if *interaction == Interaction::Pressed {
            // Update all cards - only selected one gets primary color
            for (other_card, mut other_color) in all_cards.iter_mut() {
                if other_card.mode == card.mode {
                    *other_color = UiColor::from(BUTTON_PRIMARY);
                } else {
                    *other_color = UiColor::from(BUTTON_SECONDARY);
                }
            }
            
            // Save to database
            let value = match card.mode {
                WindowMode::Default => Value::String("default".to_string()),
                WindowMode::Compact => Value::String("compact".to_string()),
            };
            
            write_events.write(SettingUpdateRequested::new(
                resource.current_tab,
                control.table.clone(),
                control.field_name.clone(),
                value,
                "ui_window_mode_card",
            ));
        }
    }
}

/// Handle theme studio button click
pub fn handle_theme_studio_click(
    query: Query<(&Interaction, &UiClicked), (With<ThemeStudioButton>, Changed<Interaction>)>,
) {
    for (interaction, clicked) in query.iter() {
        if *interaction == Interaction::Pressed && clicked.value > 0.9 {
            info!("🎨 Opening Theme Studio (placeholder - implement theme editor)");
            // TODO: Emit event to open theme studio modal
            // ThemeStudioOpenRequested event would be handled by theme editor system
        }
    }
}

/// Handle extension search input and filter visible cards
pub fn handle_extension_search(
    search_bar: Query<&TextInput, (With<ExtensionSearchBar>, Changed<TextInput>)>,
    mut rows: Query<(&ExtensionRow, &mut Visibility)>,
    plugins: Query<&PluginComponent>,
) {
    for search_input in search_bar.iter() {
        let query = search_input.value.to_lowercase();
        
        for (row, mut visibility) in rows.iter_mut() {
            if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == row.plugin_id) {
                let matches = query.is_empty() ||
                    plugin.name.to_lowercase().contains(&query) ||
                    plugin.description.to_lowercase().contains(&query) ||
                    plugin.author.as_ref().map_or(false, |a| a.to_lowercase().contains(&query));
                
                *visibility = if matches {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

/// Handle extension filter pills
pub fn handle_extension_filters(
    mut filters: Query<(&ExtensionFilterPill, &Interaction, &mut UiColor), Changed<Interaction>>,
    mut filter_state: Local<std::collections::HashSet<PluginType>>,
    mut rows: Query<(&ExtensionRow, &mut Visibility)>,
    plugins: Query<&PluginComponent>,
) {
    for (pill, interaction, mut color) in filters.iter_mut() {
        if *interaction == Interaction::Pressed {
            // Toggle filter
            if filter_state.contains(&pill.filter_type) {
                filter_state.remove(&pill.filter_type);
                *color = UiColor::from(BUTTON_SECONDARY);
            } else {
                filter_state.insert(pill.filter_type);
                *color = UiColor::from(BUTTON_PRIMARY);
            }
        }
    }
    
    // Apply filters to all rows
    if !filter_state.is_empty() {
        for (row, mut visibility) in rows.iter_mut() {
            if let Some(plugin) = plugins.iter().find(|p| p.plugin_id == row.plugin_id) {
                let visible = filter_state.contains(&plugin.plugin_type);
                *visibility = if visible { Visibility::Visible } else { Visibility::Hidden };
            }
        }
    }
}

/// Handle extension store button clicks
pub fn handle_extension_store_button(
    query: Query<&Interaction, (With<ExtensionStoreButton>, Changed<Interaction>)>,
    mut events: EventWriter<OpenExtensionStore>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            events.write(OpenExtensionStore);
            info!("Opening extension store...");
        }
    }
}

/// Handle extension settings button clicks
pub fn handle_extension_settings_button(
    query: Query<(&ExtensionSettingsButton, &Interaction), Changed<Interaction>>,
    mut events: EventWriter<ExtensionConfigChanged>,
) {
    for (button, interaction) in query.iter() {
        if *interaction == Interaction::Pressed {
            info!("Opening settings for extension: {}", button.plugin_id);
            events.write(ExtensionConfigChanged::new(
                button.plugin_id.clone(),
                "open_settings",
                "true",
                "settings_button"
            ));
        }
    }
}

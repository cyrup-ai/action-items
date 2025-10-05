use bevy::prelude::*;
use crate::{events::*, resources::*, ui::components::CloseSettingsButton};
use action_items_ecs_user_settings::SettingsUpdateRequested;
use std::collections::HashMap;

/// Process tab change requests
pub fn process_tab_changes(
    mut events: EventReader<TabChangeRequested>,
    mut resource: ResMut<SettingsResource>,
    mut changed: EventWriter<TabChanged>,
) {
    for event in events.read() {
        resource.set_tab(event.to);
        changed.write(TabChanged {
            operation_id: event.operation_id,
            tab: event.to,
        });
    }
}

/// Process setting update requests WITH database persistence
pub fn process_setting_updates(
    mut commands: Commands,
    mut events: EventReader<SettingUpdateRequested>,
    mut updated: EventWriter<SettingUpdated>,
    mut errors: EventWriter<SettingValidationFailed>,
    mut db_update: EventWriter<SettingsUpdateRequested>,
) {
    for event in events.read() {
        // Basic validation
        if event.field_name.is_empty() {
            errors.write(SettingValidationFailed {
                operation_id: event.operation_id,
                field_name: event.field_name.clone(),
                error: "Field name cannot be empty".to_string(),
            });
            continue;
        }
        
        // Persist to database - use table from event (from UI component)
        let requester = commands.spawn_empty().id();
        let table = &event.table;
        
        // Convert field value to surrealdb::Value for database
        let db_value = match serde_json::from_value(event.new_value.clone()) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to convert setting value for database: {}", e);
                continue;
            }
        };
        
        // Use MERGE update (partial) instead of CONTENT replace (full)
        // This preserves other fields in the table record
        let mut fields = HashMap::new();
        fields.insert(event.field_name.clone(), db_value);
        
        db_update.write(SettingsUpdateRequested {
            operation_id: event.operation_id,
            table: table.to_string(),
            key: "main".to_string(),  // Single record per table
            fields,
            requester,
        });
        
        // Emit success
        updated.write(SettingUpdated {
            operation_id: event.operation_id,
            tab: event.tab,
            field_name: event.field_name.clone(),
            old_value: serde_json::Value::Null,
            new_value: event.new_value.clone(),
        });
    }
}

/// Process search query changes
pub fn process_search_changes(
    mut events: EventReader<SearchQueryChanged>,
    mut resource: ResMut<SettingsResource>,
) {
    for event in events.read() {
        resource.search_query = event.query.clone();
    }
}

/// Process filter changes
pub fn process_filter_changes(
    mut events: EventReader<FilterChanged>,
    mut resource: ResMut<SettingsResource>,
) {
    for event in events.read() {
        resource.extensions_filter = event.filter;
    }
}

/// Process extension selection
pub fn process_extension_selection(
    mut events: EventReader<ExtensionSelected>,
    mut resource: ResMut<SettingsResource>,
) {
    for event in events.read() {
        resource.selected_extension = Some(event.extension_id.clone());
    }
}

/// Process visibility events
pub fn process_visibility_events(
    mut events: EventReader<SettingsVisibilityEvent>,
    mut resource: ResMut<SettingsResource>,
) {
    for event in events.read() {
        match event {
            SettingsVisibilityEvent::Show => resource.set_visible(true),
            SettingsVisibilityEvent::Hide => resource.set_visible(false),
            SettingsVisibilityEvent::Toggle => resource.toggle_visible(),
        }
    }
}


/// Handle settings open requests
pub fn handle_settings_open(
    mut open_events: EventReader<SettingsOpenRequested>,
    entities: Res<SettingsUIEntities>,
    mut visibility: Query<&mut Visibility>,
    mut resource: ResMut<SettingsResource>,
    mut opened_events: EventWriter<SettingsWindowOpened>,
    mut tab_change: EventWriter<TabChangeRequested>,
) {
    for event in open_events.read() {
        // Show backdrop
        if let Ok(mut vis) = visibility.get_mut(entities.backdrop) {
            *vis = Visibility::Visible;
        }
        
        // Show modal
        if let Ok(mut vis) = visibility.get_mut(entities.modal_root) {
            *vis = Visibility::Visible;
        }
        
        // Set initial tab if specified
        if let Some(tab) = event.initial_tab {
            tab_change.write(TabChangeRequested::new(
                resource.current_tab,
                tab,
                "settings_open",
            ));
        }
        
        resource.set_visible(true);
        opened_events.write(SettingsWindowOpened {
            operation_id: event.operation_id,
        });
    }
}

/// Handle settings close requests
pub fn handle_settings_close(
    mut close_events: EventReader<SettingsCloseRequested>,
    entities: Res<SettingsUIEntities>,
    mut visibility: Query<&mut Visibility>,
    mut resource: ResMut<SettingsResource>,
    mut closed_events: EventWriter<SettingsWindowClosed>,
) {
    for event in close_events.read() {
        // Hide modal
        if let Ok(mut vis) = visibility.get_mut(entities.modal_root) {
            *vis = Visibility::Hidden;
        }
        
        // Hide backdrop
        if let Ok(mut vis) = visibility.get_mut(entities.backdrop) {
            *vis = Visibility::Hidden;
        }
        
        resource.set_visible(false);
        closed_events.write(SettingsWindowClosed {
            operation_id: event.operation_id,
        });
    }
}

/// Handle close button clicks
pub fn handle_close_button(
    query: Query<&Interaction, (Changed<Interaction>, With<CloseSettingsButton>)>,
    mut close_events: EventWriter<SettingsCloseRequested>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            close_events.write(SettingsCloseRequested::new());
        }
    }
}

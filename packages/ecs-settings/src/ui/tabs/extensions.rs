use action_items_ecs_ui::prelude::*;
use bevy::prelude::*;
use crate::ui::{theme::*, components::*};

pub fn create_extensions_tab() -> impl FnOnce(&mut Commands, Entity) {
    move |commands: &mut Commands, parent: Entity| {
        commands.entity(parent).with_children(|parent| {
            // Header: Search + Store button
            parent.spawn((
                UiLayout::window()
                    .size((Rl(100.0), Ab(60.0)))
                    .pos((Rl(0.0), Ab(0.0)))
                    .pack(),
                Name::new("ExtensionsHeader"),
            )).with_children(|header| {
                // Search bar (left)
                header.spawn((
                    ExtensionSearchBar,
                    TextInput {
                        field_name: "extension_search".to_string(),
                        value: String::new(),
                    },
                    UiLayout::window()
                        .size((Rl(70.0), Ab(40.0)))
                        .pos((Rl(5.0), Ab(10.0)))
                        .pack(),
                    UiColor::from(INPUT_BG),
                    Text::new("🔍 Search extensions..."),
                    UiTextSize::from(Em(1.0)),
                    BorderRadius::all(Val::Px(6.0)),
                    Pickable::default(),
                    Interaction::None,
                    Name::new("ExtensionSearchBar"),
                ));
                
                // Store button (right)
                header.spawn((
                    ExtensionStoreButton,
                    UiLayout::window()
                        .size((Ab(120.0), Ab(40.0)))
                        .pos((Rl(92.0), Ab(10.0)))
                        .anchor(Anchor::TopRight)
                        .pack(),
                    UiColor::from(BUTTON_BG),
                    UiHover::new().forward_speed(8.0).backward_speed(4.0),
                    UiClicked::new().forward_speed(15.0).backward_speed(10.0),
                    Text::new("📦 Store"),
                    UiTextSize::from(Em(1.0)),
                    BorderRadius::all(Val::Px(6.0)),
                    Pickable::default(),
                    Interaction::None,
                    Name::new("StoreButton"),
                ));
            });
            
            // Filter pills
            parent.spawn((
                UiLayout::window()
                    .size((Rl(90.0), Ab(40.0)))
                    .pos((Rl(5.0), Ab(70.0)))
                    .pack(),
                Name::new("FilterPillsContainer"),
            )).with_children(|filters| {
                use ecs_service_bridge::components::PluginType;
                
                let filter_types = [
                    (PluginType::Deno, "🦕 Deno"),
                    (PluginType::Native, "⚙️ Native"),
                    (PluginType::Wasm, "🔌 WASM"),
                ];
                
                for (i, (filter_type, label)) in filter_types.iter().enumerate() {
                    filters.spawn((
                        ExtensionFilterPill {
                            filter_type: *filter_type,
                            active: false,
                        },
                        UiLayout::window()
                            .size((Ab(110.0), Ab(32.0)))
                            .pos((Ab(10.0 + (i as f32 * 120.0)), Ab(4.0)))
                            .pack(),
                        UiColor::from(BUTTON_SECONDARY),
                        UiHover::new().forward_speed(8.0).backward_speed(4.0),
                        UiClicked::new().forward_speed(15.0).backward_speed(10.0),
                        Text::new(*label),
                        UiTextSize::from(Em(0.9)),
                        BorderRadius::all(Val::Px(16.0)),
                        Pickable::default(),
                        Interaction::None,
                        Name::new(format!("FilterPill_{:?}", filter_type)),
                    ));
                }
            });
            
            // Extensions list container (scrollable card area)
            parent.spawn((
                UiLayout::window()
                    .size((Rl(90.0), Rl(70.0)))
                    .pos((Rl(5.0), Ab(120.0)))
                    .pack(),
                ExtensionsTableContainer,
                Name::new("ExtensionsCardContainer"),
            ));
        });
    }
}

# Task 4: Pro Features List and Badge System Implementation

## Objective
Implement the right panel Pro features list with consistent icon-text-badge-info layout, dynamic Pro badges, feature-specific icons, and contextual information system.

## Implementation Details

### Target Files
- `ui/src/ui/components/account/features_list.rs:1-250` - Pro features list component
- `ui/src/ui/components/account/feature_item.rs:1-150` - Individual feature item component
- `ui/src/ui/components/common/pro_badge.rs:1-80` - Reusable Pro badge component
- `core/src/features/pro_features.rs:1-200` - Pro feature definitions and metadata

### Bevy Implementation Patterns

#### Right Panel Container Layout
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:120-150` - Multi-section vertical layout
**Reference**: `./docs/bevy/examples/ui/ui.rs:180-220` - Section headers and content organization
```rust
// Right panel container (60% width)
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(60.0),
        height: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(24.0)),
        gap: Size::all(Val::Px(20.0)),
        overflow: Overflow::clip_y(),
        ..default()
    },
    ..default()
}

// Pro features section container
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        gap: Size::all(Val::Px(12.0)),
        ..default()
    },
    ..default()
}
```

#### Feature List Item Layout
**Reference**: `./docs/bevy/examples/ui/ui.rs:260-300` - Horizontal list items with multiple elements
**Reference**: `./docs/bevy/examples/ui/button.rs:90-120` - Interactive list item styling
```rust
// Individual feature item container
ButtonBundle {
    style: Style {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        width: Val::Percent(100.0),
        height: Val::Px(44.0),
        padding: UiRect {
            left: Val::Px(12.0),
            right: Val::Px(12.0),
            top: Val::Px(8.0),
            bottom: Val::Px(8.0),
        },
        gap: Size::all(Val::Px(12.0)),
        ..default()
    },
    background_color: Color::TRANSPARENT.into(),
    ..default()
}

// Feature icon
ImageBundle {
    style: Style {
        width: Val::Px(20.0),
        height: Val::Px(20.0),
        flex_shrink: 0.0,
        ..default()
    },
    image: feature_icon_handle.clone().into(),
    ..default()
}

// Feature name text
TextBundle::from_section(
    feature.display_name.clone(),
    TextStyle {
        font: font_regular.clone(),
        font_size: 14.0,
        color: Color::WHITE,
    },
).with_style(Style {
    flex_grow: 1.0,
    ..default()
})
```

#### Pro Badge Component System
**Reference**: `./docs/bevy/examples/ui/ui.rs:340-370` - Badge styling and positioning
```rust
// Pro badge component
NodeBundle {
    style: Style {
        padding: UiRect {
            left: Val::Px(8.0),
            right: Val::Px(8.0),
            top: Val::Px(2.0),
            bottom: Val::Px(2.0),
        },
        margin: UiRect::right(Val::Px(8.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_shrink: 0.0,
        ..default()
    },
    background_color: Color::rgb(0.0, 0.48, 1.0).into(), // Blue Pro badge
    border_radius: BorderRadius::all(Val::Px(4.0)),
    ..default()
}

// Pro badge text
TextBundle::from_section(
    "Pro",
    TextStyle {
        font: font_medium.clone(),
        font_size: 11.0,
        color: Color::WHITE,
    },
)

// Conditional Pro badge based on feature requirements
#[derive(Component)]
pub struct ProBadge {
    pub feature: FeatureFlag,
}

fn pro_badge_visibility_system(
    subscription_state: Res<SubscriptionState>,
    mut query: Query<(&mut Visibility, &ProBadge)>,
) {
    for (mut visibility, pro_badge) in query.iter_mut() {
        *visibility = if pro_badge.feature.requires_subscription() && 
                         !subscription_state.features.contains(&pro_badge.feature) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
```

#### Information Icon System
**Reference**: `./docs/bevy/examples/ui/ui_texture_atlas.rs:90-120` - Icon management and hover states
**Reference**: `./docs/bevy/examples/input/mouse_input.rs:70-100` - Hover interaction for info display
```rust
// Info icon button
ButtonBundle {
    style: Style {
        width: Val::Px(20.0),
        height: Val::Px(20.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_shrink: 0.0,
        ..default()
    },
    background_color: Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(10.0)),
    ..default()
}

// Info icon text
TextBundle::from_section(
    "i",
    TextStyle {
        font: font_regular.clone(),
        font_size: 12.0,
        color: Color::rgba(0.7, 0.7, 0.7, 1.0),
    },
)

// Info tooltip system
#[derive(Component)]
pub struct InfoTooltip {
    pub content: String,
    pub feature: FeatureFlag,
}

fn info_tooltip_system(
    mut interaction_query: Query<(&Interaction, &InfoTooltip), Changed<Interaction>>,
    mut tooltip_events: EventWriter<TooltipEvent>,
) {
    for (interaction, info_tooltip) in interaction_query.iter() {
        match *interaction {
            Interaction::Hovered => {
                tooltip_events.send(TooltipEvent::Show {
                    content: info_tooltip.content.clone(),
                    position: TooltipPosition::Cursor,
                });
            }
            Interaction::None => {
                tooltip_events.send(TooltipEvent::Hide);
            }
            _ => {}
        }
    }
}
```

### Pro Features Data System

#### Feature Definition and Metadata
**Reference**: `./docs/bevy/examples/reflection/reflection.rs:90-130` - Data structure reflection and metadata
```rust
// Pro features metadata system
#[derive(Debug, Clone)]
pub struct ProFeature {
    pub id: FeatureFlag,
    pub display_name: String,
    pub description: String,
    pub icon_path: String,
    pub requires_pro: bool,
    pub usage_limit: Option<FeatureLimit>,
}

#[derive(Debug, Clone)]
pub enum FeatureLimit {
    Count(u32),
    Size(u64),
    Duration(chrono::Duration),
    Unlimited,
}

pub fn get_pro_features() -> Vec<ProFeature> {
    vec![
        ProFeature {
            id: FeatureFlag::RaycastAI,
            display_name: "Action Items AI".to_string(),
            description: "AI-powered commands and natural language processing".to_string(),
            icon_path: "icons/sparkle.png".to_string(),
            requires_pro: true,
            usage_limit: Some(FeatureLimit::Count(100)), // 100 AI queries per month
        },
        ProFeature {
            id: FeatureFlag::CloudSync,
            display_name: "Cloud Sync".to_string(),
            description: "Sync your settings, snippets, and quicklinks across devices".to_string(),
            icon_path: "icons/cloud.png".to_string(),
            requires_pro: true,
            usage_limit: None,
        },
        ProFeature {
            id: FeatureFlag::CustomThemes,
            display_name: "Custom Themes".to_string(),
            description: "Create and customize your own color themes".to_string(),
            icon_path: "icons/palette.png".to_string(),
            requires_pro: true,
            usage_limit: None,
        },
        // ... additional features
    ]
}
```

#### Dynamic Feature List Generation
**Reference**: `./docs/bevy/examples/ui/ui.rs:400-450` - Dynamic UI generation from data
```rust
// System to generate feature list UI from metadata
fn generate_features_list_system(
    mut commands: Commands,
    features_query: Query<Entity, With<FeaturesListContainer>>,
    asset_server: Res<AssetServer>,
    subscription_state: Res<SubscriptionState>,
) {
    for container_entity in features_query.iter() {
        // Clear existing children
        commands.entity(container_entity).despawn_descendants();
        
        // Generate Pro features section
        let pro_features = get_pro_features();
        
        commands.entity(container_entity).with_children(|parent| {
            // Section header
            parent.spawn(TextBundle::from_section(
                "Pro",
                TextStyle {
                    font: font_medium.clone(),
                    font_size: 16.0,
                    color: Color::rgba(0.6, 0.6, 0.6, 1.0),
                },
            )).insert(SectionHeader);
            
            // Feature items
            for feature in pro_features {
                spawn_feature_item(parent, &feature, &asset_server, &subscription_state);
            }
        });
    }
}

fn spawn_feature_item(
    parent: &mut ChildBuilder,
    feature: &ProFeature,
    asset_server: &AssetServer,
    subscription_state: &SubscriptionState,
) {
    parent.spawn(ButtonBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            padding: UiRect::all(Val::Px(12.0)),
            gap: Size::all(Val::Px(12.0)),
            ..default()
        },
        background_color: Color::TRANSPARENT.into(),
        ..default()
    })
    .insert(FeatureItem { feature: feature.id })
    .with_children(|item| {
        // Feature icon
        item.spawn(ImageBundle {
            image: asset_server.load(&feature.icon_path).into(),
            style: Style {
                width: Val::Px(20.0),
                height: Val::Px(20.0),
                ..default()
            },
            ..default()
        });
        
        // Feature name
        item.spawn(TextBundle::from_section(
            feature.display_name.clone(),
            TextStyle {
                font: font_regular.clone(),
                font_size: 14.0,
                color: Color::WHITE,
            },
        ).with_style(Style {
            flex_grow: 1.0,
            ..default()
        }));
        
        // Pro badge (if required and not owned)
        if feature.requires_pro && !subscription_state.features.contains(&feature.id) {
            spawn_pro_badge(item);
        }
        
        // Info icon
        item.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(20.0),
                height: Val::Px(20.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgba(0.3, 0.3, 0.3, 1.0).into(),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            ..default()
        })
        .insert(InfoTooltip {
            content: feature.description.clone(),
            feature: feature.id,
        });
    });
}
```

### Architecture Notes

#### Component Structure
- **FeaturesListContainer**: Container for the entire features list
- **FeatureItem**: Individual feature list item component
- **ProBadge**: Reusable Pro subscription badge
- **InfoTooltip**: Information tooltip for feature details

#### Dynamic Update Strategy
- **Subscription-Aware**: Feature display updates based on subscription changes
- **Badge Visibility**: Pro badges show/hide based on feature ownership
- **Icon Loading**: Efficient icon asset loading and caching
- **Tooltip System**: Context-sensitive information display

#### Interaction Patterns
- **Hover Effects**: Subtle visual feedback on feature item hover
- **Info Tooltips**: Detailed feature information on icon hover
- **Click Handling**: Future extensibility for feature configuration

### Quality Standards
- Consistent visual hierarchy across all feature items
- Efficient icon loading with proper caching and fallbacks
- Smooth badge transitions when subscription status changes
- Accessible tooltip system with keyboard navigation support
- Performance optimization for large feature lists

### Integration Points
- Subscription system integration for feature access control
- Asset loading system for feature icons and imagery
- Tooltip system integration for contextual help
- Theme system integration for consistent styling
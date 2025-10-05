# Task 6: Organizations Features Section Implementation

## Objective
Implement the Organizations features section with team collaboration features, shared resource management, role-based access control, and organization membership display.

## Implementation Details

### Target Files
- `ui/src/ui/components/account/organizations_section.rs:1-200` - Organizations section component
- `core/src/organizations/team_features.rs:1-180` - Team collaboration feature definitions
- `core/src/organizations/membership.rs:1-150` - Organization membership management
- `core/src/organizations/resource_sharing.rs:1-120` - Shared resource access control

### Bevy Implementation Patterns

#### Organizations Section Container
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:160-190` - Section-based vertical layout
**Reference**: `./docs/bevy/examples/ui/ui.rs:450-480` - Feature grouping and spacing
```rust
// Organizations section container
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        gap: Size::all(Val::Px(12.0)),
        margin: UiRect::top(Val::Px(24.0)),
        ..default()
    },
    ..default()
}

// Organizations section header
TextBundle::from_section(
    "Organizations",
    TextStyle {
        font: font_medium.clone(),
        font_size: 16.0,
        color: Color::rgba(0.6, 0.6, 0.6, 1.0), // Medium gray section header
    },
).with_style(Style {
    margin: UiRect::bottom(Val::Px(8.0)),
    ..default()
})
```

#### Organization Feature Items
**Reference**: `./docs/bevy/examples/ui/ui.rs:500-540` - Feature item layout without Pro badges
```rust
// Organization feature item (no Pro badge for base features)
ButtonBundle {
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
}

// Feature icon for organization features
ImageBundle {
    style: Style {
        width: Val::Px(20.0),
        height: Val::Px(20.0),
        flex_shrink: 0.0,
        ..default()
    },
    image: org_feature_icon.clone().into(),
    ..default()
}

// Organization feature name without Pro badge space
TextBundle::from_section(
    org_feature.display_name.clone(),
    TextStyle {
        font: font_regular.clone(),
        font_size: 14.0,
        color: Color::WHITE,
    },
).with_style(Style {
    flex_grow: 1.0,
    margin: UiRect::right(Val::Px(12.0)), // Space before info icon
    ..default()
})
```

#### Team-wide Pro Features Badge
**Reference**: `./docs/bevy/examples/ui/ui.rs:340-370` - Special badge for organization features
```rust
// Special Pro badge for "Pro Features for All Members"
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

// Pro badge for organization-wide features
#[derive(Component)]
pub struct OrganizationProFeature {
    pub feature: OrganizationFeatureFlag,
}

fn organization_pro_badge_system(
    organization_state: Res<OrganizationState>,
    mut query: Query<(&mut Visibility, &OrganizationProFeature)>,
) {
    for (mut visibility, org_pro_feature) in query.iter_mut() {
        *visibility = if matches!(org_pro_feature.feature, OrganizationFeatureFlag::ProFeaturesForAll) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
```

### Organization Features Data System

#### Organization Feature Definitions
**Reference**: `./docs/bevy/examples/reflection/reflection.rs:140-180` - Feature metadata and organization structure
```rust
// Organization feature definitions
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum OrganizationFeatureFlag {
    PrivateExtensions,
    SharedQuicklinks,
    SharedSnippets,
    ProFeaturesForAll,
}

#[derive(Debug, Clone)]
pub struct OrganizationFeature {
    pub id: OrganizationFeatureFlag,
    pub display_name: String,
    pub description: String,
    pub icon_path: String,
    pub requires_team_plan: bool,
    pub member_benefit: bool,
}

pub fn get_organization_features() -> Vec<OrganizationFeature> {
    vec![
        OrganizationFeature {
            id: OrganizationFeatureFlag::PrivateExtensions,
            display_name: "Private Extensions".to_string(),
            description: "Share custom extensions privately within your organization".to_string(),
            icon_path: "icons/gear_extension.png".to_string(),
            requires_team_plan: false,
            member_benefit: true,
        },
        OrganizationFeature {
            id: OrganizationFeatureFlag::SharedQuicklinks,
            display_name: "Shared Quicklinks".to_string(),
            description: "Share quicklinks and bookmarks across your team".to_string(),
            icon_path: "icons/link.png".to_string(),
            requires_team_plan: false,
            member_benefit: true,
        },
        OrganizationFeature {
            id: OrganizationFeatureFlag::SharedSnippets,
            display_name: "Shared Snippets".to_string(),
            description: "Share code snippets and text templates with your team".to_string(),
            icon_path: "icons/code_snippet.png".to_string(),
            requires_team_plan: false,
            member_benefit: true,
        },
        OrganizationFeature {
            id: OrganizationFeatureFlag::ProFeaturesForAll,
            display_name: "Pro Features for All Members".to_string(),
            description: "All team members get access to Pro features when part of the organization".to_string(),
            icon_path: "icons/team_group.png".to_string(),
            requires_team_plan: true,
            member_benefit: false,
        },
    ]
}
```

#### Organization Membership Management
**Reference**: `./docs/bevy/examples/ecs/resources.rs:80-110` - Organization state resource management
```rust
// Organization membership state
#[derive(Resource, Clone, Debug)]
pub struct OrganizationState {
    pub memberships: Vec<OrganizationMembership>,
    pub active_organization: Option<String>,
    pub available_features: HashSet<OrganizationFeatureFlag>,
    pub role_permissions: HashMap<String, Vec<Permission>>,
}

#[derive(Debug, Clone)]
pub struct OrganizationMembership {
    pub organization_id: String,
    pub organization_name: String,
    pub role: OrganizationRole,
    pub joined_at: DateTime<Utc>,
    pub status: MembershipStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrganizationRole {
    Member,
    Admin,
    Owner,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MembershipStatus {
    Active,
    Pending,
    Suspended,
}

impl OrganizationState {
    pub fn has_feature_access(&self, feature: &OrganizationFeatureFlag) -> bool {
        self.available_features.contains(feature) ||
        self.memberships.iter().any(|membership| {
            membership.status == MembershipStatus::Active &&
            self.organization_provides_feature(&membership.organization_id, feature)
        })
    }
    
    pub fn is_team_member(&self) -> bool {
        self.memberships.iter().any(|m| m.status == MembershipStatus::Active)
    }
}
```

### Shared Resource Management System

#### Resource Sharing Components
**Reference**: `./docs/bevy/examples/ecs/change_detection.rs:90-120` - Dynamic feature availability
```rust
// Shared resource access system
#[derive(Component)]
pub struct SharedResourceIndicator {
    pub resource_type: SharedResourceType,
    pub organization_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SharedResourceType {
    Extensions,
    Quicklinks,
    Snippets,
}

fn shared_resource_access_system(
    organization_state: Res<OrganizationState>,
    mut query: Query<(&mut Visibility, &SharedResourceIndicator), Changed<OrganizationState>>,
) {
    if organization_state.is_changed() {
        for (mut visibility, resource_indicator) in query.iter_mut() {
            let has_access = organization_state.memberships.iter().any(|membership| {
                membership.organization_id == resource_indicator.organization_id &&
                membership.status == MembershipStatus::Active
            });
            
            *visibility = if has_access {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
```

#### Organization Feature Generation System
**Reference**: `./docs/bevy/examples/ui/ui.rs:560-610` - Dynamic organization feature list generation
```rust
// System to generate organization features UI
fn generate_organization_features_system(
    mut commands: Commands,
    org_section_query: Query<Entity, With<OrganizationSection>>,
    organization_state: Res<OrganizationState>,
    asset_server: Res<AssetServer>,
) {
    for section_entity in org_section_query.iter() {
        // Clear existing organization features
        commands.entity(section_entity).despawn_descendants();
        
        commands.entity(section_entity).with_children(|parent| {
            // Section header
            parent.spawn(TextBundle::from_section(
                "Organizations",
                TextStyle {
                    font: font_medium.clone(),
                    font_size: 16.0,
                    color: Color::rgba(0.6, 0.6, 0.6, 1.0),
                },
            ));
            
            // Organization features
            let org_features = get_organization_features();
            for feature in org_features {
                spawn_organization_feature_item(parent, &feature, &asset_server, &organization_state);
            }
        });
    }
}

fn spawn_organization_feature_item(
    parent: &mut ChildBuilder,
    feature: &OrganizationFeature,
    asset_server: &AssetServer,
    organization_state: &OrganizationState,
) {
    let has_access = organization_state.has_feature_access(&feature.id);
    let is_team_member = organization_state.is_team_member();
    
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
    .insert(OrganizationFeatureItem { feature: feature.id })
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
                color: if has_access || is_team_member {
                    Color::WHITE
                } else {
                    Color::rgba(0.6, 0.6, 0.6, 1.0) // Grayed out if no access
                },
            },
        ).with_style(Style {
            flex_grow: 1.0,
            ..default()
        }));
        
        // Pro badge for "Pro Features for All Members"
        if feature.id == OrganizationFeatureFlag::ProFeaturesForAll {
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
            feature_type: TooltipFeatureType::Organization(feature.id),
        });
    });
}
```

### Architecture Notes

#### Component Structure
- **OrganizationSection**: Container for organization features section
- **OrganizationFeatureItem**: Individual organization feature component
- **SharedResourceIndicator**: Component for shared resource access indication
- **OrganizationState**: Global resource for membership and feature access

#### Access Control Strategy
- **Membership-Based**: Feature access based on active organization membership
- **Role-Aware**: Different access levels based on organization role
- **Dynamic Updates**: Real-time feature availability updates
- **Graceful Degradation**: Clear indication when features unavailable

#### Team Feature Integration
- **Shared Resources**: Integration with extension, quicklink, and snippet sharing
- **Pro Feature Distribution**: Organization-wide Pro feature access
- **Member Benefits**: Team collaboration features for all members
- **Administrative Controls**: Role-based access to organizational features

### Quality Standards
- Clear visual distinction between accessible and inaccessible features
- Efficient organization membership validation and caching
- Smooth feature availability updates when membership changes
- Consistent styling with Pro features section while highlighting differences
- Proper error handling for organization service integration failures

### Integration Points
- Organization membership service for real-time status
- Shared resource systems for extensions, quicklinks, and snippets
- Pro feature system integration for organization-wide benefits
- User authentication integration for membership validation
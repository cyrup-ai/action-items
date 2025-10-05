# TASK7.6: Settings Panel - Organizations

**Status**: Not Started  
**Estimated Time**: 5-6 hours  
**Priority**: Medium  
**Dependencies**: TASK7.0-INFRASTRUCTURE.md, TASK7.C-COMPONENTS.md

---

## Objective

Implement the Organizations settings panel with sidebar + detail view pattern for managing organization memberships. This panel allows users to view organizations they belong to, manage organization settings, access organization stores for extensions, and handle organization membership actions including creation and leaving organizations.

---

## Dependencies

**MUST complete first:**
1. ✅ TASK7.0-INFRASTRUCTURE.md - Settings modal, tabs, entity pre-allocation
2. ✅ TASK7.C-COMPONENTS.md - Button components

**Required systems:**
- `SettingsUIEntities` resource (from TASK7.0)
- `SettingControl` component (from TASK7.C)
- Form control spawning functions (from TASK7.C)
- Event handlers for database I/O (from TASK7.0)

**External integrations:**
- Organization management API
- Organization store/marketplace
- Subscription management for organizations

---

## Screenshot Reference

![Organizations Menu](/Volumes/samsung_t9/action-items/spec/screenshots/Organizations_Menu.png)

**Visual Structure:**

```
┌──────────────┬────────────────────────────────────────┐
│ Organizations│                                        │
│              │         ┌────┐                         │
│ ┌──────────┐ │         │ 🏢 │                         │
│ │ Y Cyrup.ai│ │         │Logo│  Cyrup.ai              │
│ │      [⚙️] │ │         └────┘                         │
│ └──────────┘ │                                        │
│              │               [Paid Plan]               │
│              │         [Manage Subscription]          │
│              │                                        │
│              │   Manage Organization                  │
│              │   You can use the Manage Organization  │
│              │   command to see who's part of your    │
│              │   organization, reset the invite link  │
│              │   and edit your organization details.  │
│              │                                        │
│              │   [Manage Organization] [Edit Org]     │
│              │                                        │
│              │   Store                                │
│              │   Extend Raycast with extensions from  │
│              │   Cyrup.ai. Open the Store to see      │
│              │   what is available.                   │
│              │                                        │
│              │   [Open Store]                         │
│              │                                        │
│              │   Danger Zone                          │
│              │   If you leave the organization, all   │
│              │   the commands that are connected to   │
│              │   the organization will be removed     │
│              │   from your account.                   │
│              │                                        │
├──────────────┤                                        │
│              │                                        │
│ [+ Create New│                                        │
│  Organization]                                        │
└──────────────┴────────────────────────────────────────┘
```

---

## Database Schema

### Table: `user_organizations`

```sql
DEFINE TABLE user_organizations SCHEMALESS;

-- Organization identification
DEFINE FIELD organization_id ON TABLE user_organizations TYPE string;
DEFINE FIELD organization_name ON TABLE user_organizations TYPE string;
DEFINE FIELD organization_slug ON TABLE user_organizations TYPE string;

-- Organization details
DEFINE FIELD logo_url ON TABLE user_organizations TYPE string;
DEFINE FIELD description ON TABLE user_organizations TYPE string;
DEFINE FIELD website_url ON TABLE user_organizations TYPE string;

-- Membership
DEFINE FIELD user_role ON TABLE user_organizations TYPE string DEFAULT "member";
  -- Values: "owner", "admin", "member"

DEFINE FIELD joined_at ON TABLE user_organizations TYPE datetime;
DEFINE FIELD invitation_code ON TABLE user_organizations TYPE string;

-- Subscription
DEFINE FIELD subscription_plan ON TABLE user_organizations TYPE string DEFAULT "free";
  -- Values: "free", "team", "enterprise"

DEFINE FIELD subscription_status ON TABLE user_organizations TYPE string DEFAULT "inactive";
  -- Values: "active", "inactive", "trial", "expired"

DEFINE FIELD has_paid_plan ON TABLE user_organizations TYPE bool DEFAULT false;

-- Features
DEFINE FIELD has_private_extensions ON TABLE user_organizations TYPE bool DEFAULT false;
DEFINE FIELD has_shared_resources ON TABLE user_organizations TYPE bool DEFAULT false;
DEFINE FIELD has_custom_store ON TABLE user_organizations TYPE bool DEFAULT false;

-- Metadata
DEFINE FIELD member_count ON TABLE user_organizations TYPE int DEFAULT 1;
DEFINE FIELD extension_count ON TABLE user_organizations TYPE int DEFAULT 0;

DEFINE INDEX idx_org_id ON TABLE user_organizations COLUMNS organization_id;
```

### Table: `organization_ui_state`

```sql
DEFINE TABLE organization_ui_state SCHEMALESS;

-- UI state
DEFINE FIELD selected_organization_id ON TABLE organization_ui_state TYPE string;
DEFINE FIELD last_viewed_at ON TABLE organization_ui_state TYPE datetime;
```

---

## Component Structure

### Components

```rust
use bevy::prelude::*;
use chrono::{DateTime, Utc};

/// Marker component for the Organizations panel root entity
#[derive(Component, Debug)]
pub struct OrganizationsPanel;

/// Component for organization sidebar item
#[derive(Component, Debug, Clone)]
pub struct OrganizationSidebarItem {
    pub organization_id: String,
    pub organization_name: String,
    pub logo_url: Option<String>,
    pub is_selected: bool,
}

/// Component for organization settings button in sidebar
#[derive(Component, Debug)]
pub struct OrganizationSettingsButton {
    pub organization_id: String,
}

/// Component for the selected organization detail view
#[derive(Component, Debug, Clone)]
pub struct OrganizationDetail {
    pub organization_id: String,
    pub organization_name: String,
    pub logo_url: Option<String>,
    pub subscription_plan: String,
    pub has_paid_plan: bool,
    pub user_role: OrganizationRole,
    pub member_count: u32,
    pub extension_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrganizationRole {
    Owner,
    Admin,
    Member,
}

impl OrganizationRole {
    pub fn can_manage(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }
    
    pub fn can_edit(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            Self::Owner => "Owner",
            Self::Admin => "Admin",
            Self::Member => "Member",
        }
    }
}

/// Component for action buttons in the detail view
#[derive(Component, Debug)]
pub struct OrganizationActionButton {
    pub organization_id: String,
    pub action: OrganizationAction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrganizationAction {
    ManageSubscription,
    ManageOrganization,
    EditOrganization,
    OpenStore,
    LeaveOrganization,
}

impl OrganizationAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ManageSubscription => "Manage Subscription",
            Self::ManageOrganization => "Manage Organization",
            Self::EditOrganization => "Edit Organization",
            Self::OpenStore => "Open Store",
            Self::LeaveOrganization => "Leave Organization",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            Self::ManageSubscription => Color::srgba(0.3, 0.5, 0.8, 1.0),
            Self::ManageOrganization => Color::srgba(0.3, 0.3, 0.35, 1.0),
            Self::EditOrganization => Color::srgba(0.3, 0.3, 0.35, 1.0),
            Self::OpenStore => Color::srgba(0.3, 0.5, 0.8, 1.0),
            Self::LeaveOrganization => Color::srgba(0.8, 0.2, 0.2, 1.0),
        }
    }
}

/// Component for the create new organization button
#[derive(Component, Debug)]
pub struct CreateOrganizationButton;

/// Component for subscription plan badge
#[derive(Component, Debug)]
pub struct SubscriptionPlanBadge {
    pub plan: String,
}
```

### Resources

```rust
/// Resource tracking all entities in the Organizations panel
#[derive(Resource)]
pub struct OrganizationsPanelEntities {
    pub panel_root: Entity,
    
    // Sidebar
    pub sidebar_container: Entity,
    pub organization_items: HashMap<String, Entity>,
    pub settings_buttons: HashMap<String, Entity>,
    pub create_button: Entity,
    
    // Detail view
    pub detail_container: Entity,
    pub detail_logo: Entity,
    pub detail_name_text: Entity,
    pub detail_plan_badge: Entity,
    
    // Action buttons
    pub manage_subscription_button: Entity,
    pub manage_org_button: Entity,
    pub edit_org_button: Entity,
    pub open_store_button: Entity,
    pub leave_org_button: Entity,
}

/// Resource containing current organization state
#[derive(Resource, Default)]
pub struct CurrentOrganizationState {
    pub organizations: Vec<OrganizationDetail>,
    pub selected_organization_id: Option<String>,
}
```

### Events

```rust
/// Event sent when user selects an organization from sidebar
#[derive(Event, Debug)]
pub struct OrganizationSelected {
    pub organization_id: String,
}

/// Event sent when user clicks organization action button
#[derive(Event, Debug)]
pub struct OrganizationActionRequested {
    pub organization_id: String,
    pub action: OrganizationAction,
}

/// Event sent when user clicks create organization button
#[derive(Event, Debug)]
pub struct CreateOrganizationRequested;

/// Event sent when user clicks leave organization (confirmation needed)
#[derive(Event, Debug)]
pub struct LeaveOrganizationRequested {
    pub organization_id: String,
    pub organization_name: String,
}

/// Event sent when organization data loads
#[derive(Event, Debug)]
pub struct OrganizationsDataLoaded {
    pub organizations: Vec<OrganizationDetail>,
}
```

---

## Implementation Details

### System 1: Setup Organizations Panel Entities

**Purpose**: Pre-allocate all Organizations panel UI entities during initialization

```rust
pub fn setup_organizations_panel(
    mut commands: Commands,
    settings_entities: Res<SettingsUIEntities>,
    asset_server: Res<AssetServer>,
) {
    let content_area = settings_entities.content_area;
    
    // Create panel root
    let panel_root = commands.spawn((
        OrganizationsPanel,
        UiLayout::window()
            .size((Rl(100.0), Rl(100.0)))
            .pos((Rl(0.0), Rl(0.0)))
            .pack(),
        Visibility::Hidden,
        Name::new("OrganizationsPanel"),
    )).id();
    
    commands.entity(content_area).add_child(panel_root);
    
    // ═══════════════════════════════════════════════════════════
    // SIDEBAR (Left - 30% width)
    // ═══════════════════════════════════════════════════════════
    
    let sidebar_container = commands.spawn((
        UiLayout::window()
            .size((Rl(28.0), Rl(100.0)))
            .pos((Rl(0.0), Rl(0.0)))
            .pack(),
        UiColor::from(Color::srgba(0.08, 0.08, 0.1, 1.0)),
        Name::new("Sidebar"),
    )).id();
    
    // Sidebar header "Organizations"
    let sidebar_header = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(40.0)))
            .pos((Ab(15.0), Ab(15.0)))
            .pack(),
        Text::new("Organizations"),
        UiTextSize::from(Em(0.9)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
        Name::new("SidebarHeader"),
    )).id();
    
    commands.entity(sidebar_container).add_child(sidebar_header);
    
    // Organization items will be dynamically spawned here
    // Placeholder for empty state or initial items
    
    // Create New Organization button at bottom
    let create_button = commands.spawn((
        CreateOrganizationButton,
        UiLayout::window()
            .size((Rl(90.0), Ab(40.0)))
            .pos((Ab(15.0), Rl(95.0)))
            .anchor(Anchor::BottomLeft)
            .pack(),
        UiColor::from(Color::srgba(0.0, 0.0, 0.0, 0.0)), // Transparent background
        UiHover::new().forward_speed(8.0).backward_speed(4.0),
        UiClicked::new().forward_speed(15.0).backward_speed(10.0),
        Text::new("+ Create New Organization"),
        UiTextSize::from(Em(0.9)),
        Pickable::default(),
        Interaction::None,
        Name::new("CreateOrganizationButton"),
    )).id();
    
    commands.entity(sidebar_container).add_child(create_button);
    commands.entity(panel_root).add_child(sidebar_container);
    
    // ═══════════════════════════════════════════════════════════
    // DETAIL VIEW (Right - 70% width)
    // ═══════════════════════════════════════════════════════════
    
    let detail_container = commands.spawn((
        UiLayout::window()
            .size((Rl(70.0), Rl(100.0)))
            .pos((Rl(30.0), Rl(0.0)))
            .pack(),
        UiColor::from(Color::srgba(0.0, 0.0, 0.0, 0.0)), // Transparent
        Visibility::Hidden, // Hidden until organization selected
        Name::new("DetailContainer"),
    )).id();
    
    // Organization logo
    let detail_logo = commands.spawn((
        UiLayout::window()
            .size((Ab(100.0), Ab(100.0)))
            .pos((Rl(50.0), Ab(40.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.5, 0.8, 1.0)), // Placeholder color
        Text::new("🏢"),
        UiTextSize::from(Em(3.0)),
        Name::new("DetailLogo"),
    )).id();
    
    // Organization name
    let detail_name_text = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(40.0)))
            .pos((Rl(50.0), Ab(155.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Text::new("Organization Name"),
        UiTextSize::from(Em(1.5)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
        Name::new("DetailNameText"),
    )).id();
    
    // Subscription plan badge
    let detail_plan_badge = commands.spawn((
        SubscriptionPlanBadge {
            plan: "free".to_string(),
        },
        UiLayout::window()
            .size((Ab(100.0), Ab(30.0)))
            .pos((Rl(50.0), Ab(205.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        UiColor::from(Color::srgba(0.0, 0.7, 0.4, 1.0)),
        Text::new("Paid Plan"),
        UiTextSize::from(Em(0.9)),
        Visibility::Hidden, // Show only if has paid plan
        Name::new("DetailPlanBadge"),
    )).id();
    
    // Manage Subscription button
    let manage_subscription_button = commands.spawn((
        OrganizationActionButton {
            organization_id: String::new(), // Will be set when org selected
            action: OrganizationAction::ManageSubscription,
        },
        UiLayout::window()
            .size((Ab(200.0), Ab(40.0)))
            .pos((Rl(50.0), Ab(250.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        UiColor::from(OrganizationAction::ManageSubscription.color()),
        UiHover::new(),
        UiClicked::new(),
        Text::new(OrganizationAction::ManageSubscription.label()),
        UiTextSize::from(Em(1.0)),
        Pickable::default(),
        Interaction::None,
        Name::new("ManageSubscriptionButton"),
    )).id();
    
    commands.entity(detail_container).push_children(&[
        detail_logo,
        detail_name_text,
        detail_plan_badge,
        manage_subscription_button,
    ]);
    
    let mut y_offset = 320.0;
    
    // ─────────────────────────────────────────────────────────
    // MANAGE ORGANIZATION SECTION
    // ─────────────────────────────────────────────────────────
    
    let manage_section_title = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(30.0)))
            .pos((Rl(10.0), Ab(y_offset)))
            .pack(),
        Text::new("Manage Organization"),
        UiTextSize::from(Em(1.1)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
        Name::new("ManageSectionTitle"),
    )).id();
    
    y_offset += 40.0;
    
    let manage_section_description = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(80.0)))
            .pos((Rl(10.0), Ab(y_offset)))
            .pack(),
        Text::new("You can use the Manage Organization command to see\nwho's part of your organization, reset the invite link\nand edit your organization details."),
        UiTextSize::from(Em(0.9)),
        UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
        Name::new("ManageSectionDescription"),
    )).id();
    
    y_offset += 90.0;
    
    // Manage and Edit buttons side by side
    let manage_org_button = commands.spawn((
        OrganizationActionButton {
            organization_id: String::new(),
            action: OrganizationAction::ManageOrganization,
        },
        UiLayout::window()
            .size((Ab(190.0), Ab(40.0)))
            .pos((Rl(10.0), Ab(y_offset)))
            .pack(),
        UiColor::from(OrganizationAction::ManageOrganization.color()),
        UiHover::new(),
        UiClicked::new(),
        Text::new(OrganizationAction::ManageOrganization.label()),
        UiTextSize::from(Em(0.95)),
        Pickable::default(),
        Interaction::None,
        Name::new("ManageOrgButton"),
    )).id();
    
    let edit_org_button = commands.spawn((
        OrganizationActionButton {
            organization_id: String::new(),
            action: OrganizationAction::EditOrganization,
        },
        UiLayout::window()
            .size((Ab(160.0), Ab(40.0)))
            .pos((Rl(10.0) + Ab(200.0), Ab(y_offset)))
            .pack(),
        UiColor::from(OrganizationAction::EditOrganization.color()),
        UiHover::new(),
        UiClicked::new(),
        Text::new(OrganizationAction::EditOrganization.label()),
        UiTextSize::from(Em(0.95)),
        Pickable::default(),
        Interaction::None,
        Name::new("EditOrgButton"),
    )).id();
    
    commands.entity(detail_container).push_children(&[
        manage_section_title,
        manage_section_description,
        manage_org_button,
        edit_org_button,
    ]);
    
    y_offset += 70.0;
    
    // ─────────────────────────────────────────────────────────
    // STORE SECTION
    // ─────────────────────────────────────────────────────────
    
    let store_section_title = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(30.0)))
            .pos((Rl(10.0), Ab(y_offset)))
            .pack(),
        Text::new("Store"),
        UiTextSize::from(Em(1.1)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
        Name::new("StoreSectionTitle"),
    )).id();
    
    y_offset += 40.0;
    
    let store_section_description = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(60.0)))
            .pos((Rl(10.0), Ab(y_offset)))
            .pack(),
        Text::new("Extend Action Items with extensions from your organization.\nOpen the Store to see what is available."),
        UiTextSize::from(Em(0.9)),
        UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
        Name::new("StoreSectionDescription"),
    )).id();
    
    y_offset += 70.0;
    
    let open_store_button = commands.spawn((
        OrganizationActionButton {
            organization_id: String::new(),
            action: OrganizationAction::OpenStore,
        },
        UiLayout::window()
            .size((Ab(140.0), Ab(40.0)))
            .pos((Rl(10.0), Ab(y_offset)))
            .pack(),
        UiColor::from(OrganizationAction::OpenStore.color()),
        UiHover::new(),
        UiClicked::new(),
        Text::new(OrganizationAction::OpenStore.label()),
        UiTextSize::from(Em(0.95)),
        Pickable::default(),
        Interaction::None,
        Name::new("OpenStoreButton"),
    )).id();
    
    commands.entity(detail_container).push_children(&[
        store_section_title,
        store_section_description,
        open_store_button,
    ]);
    
    y_offset += 70.0;
    
    // ─────────────────────────────────────────────────────────
    // DANGER ZONE SECTION
    // ─────────────────────────────────────────────────────────
    
    let danger_section_title = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(30.0)))
            .pos((Rl(10.0), Ab(y_offset)))
            .pack(),
        Text::new("Danger Zone"),
        UiTextSize::from(Em(1.1)),
        UiColor::from(Color::srgba(0.9, 0.3, 0.3, 1.0)),
        Name::new("DangerSectionTitle"),
    )).id();
    
    y_offset += 40.0;
    
    let danger_section_description = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(80.0)))
            .pos((Rl(10.0), Ab(y_offset)))
            .pack(),
        Text::new("If you leave the organization, all the commands that are\nconnected to the organization will be removed from\nyour account."),
        UiTextSize::from(Em(0.9)),
        UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
        Name::new("DangerSectionDescription"),
    )).id();
    
    y_offset += 90.0;
    
    let leave_org_button = commands.spawn((
        OrganizationActionButton {
            organization_id: String::new(),
            action: OrganizationAction::LeaveOrganization,
        },
        UiLayout::window()
            .size((Ab(180.0), Ab(40.0)))
            .pos((Rl(10.0), Ab(y_offset)))
            .pack(),
        UiColor::from(OrganizationAction::LeaveOrganization.color()),
        UiHover::new(),
        UiClicked::new(),
        Text::new("Leave Organization"),
        UiTextSize::from(Em(0.95)),
        Pickable::default(),
        Interaction::None,
        Name::new("LeaveOrgButton"),
    )).id();
    
    commands.entity(detail_container).push_children(&[
        danger_section_title,
        danger_section_description,
        leave_org_button,
    ]);
    
    commands.entity(panel_root).add_child(detail_container);
    
    // Store entities in resource
    commands.insert_resource(OrganizationsPanelEntities {
        panel_root,
        sidebar_container,
        organization_items: HashMap::new(),
        settings_buttons: HashMap::new(),
        create_button,
        detail_container,
        detail_logo,
        detail_name_text,
        detail_plan_badge,
        manage_subscription_button,
        manage_org_button,
        edit_org_button,
        open_store_button,
        leave_org_button,
    });
    
    info!("✅ Pre-allocated Organizations panel UI entities");
}
```

### System 2: Load Organizations Data

**Purpose**: Load user's organizations when panel becomes visible

```rust
pub fn load_organizations_data(
    mut panel_query: Query<&Visibility, (With<OrganizationsPanel>, Changed<Visibility>)>,
    mut read_events: EventWriter<SettingsReadRequested>,
    panel_entities: Res<OrganizationsPanelEntities>,
) {
    for visibility in panel_query.iter() {
        if *visibility == Visibility::Visible {
            // Load user organizations
            read_events.send(SettingsReadRequested {
                operation_id: Uuid::new_v4(),
                table: "user_organizations".to_string(),
                query: "SELECT * FROM user_organizations ORDER BY joined_at DESC".to_string(),
                requester: panel_entities.panel_root,
            });
            
            // Load UI state (last selected org)
            read_events.send(SettingsReadRequested {
                operation_id: Uuid::new_v4(),
                table: "organization_ui_state".to_string(),
                query: "SELECT * FROM organization_ui_state LIMIT 1".to_string(),
                requester: panel_entities.panel_root,
            });
            
            info!("📖 Loading Organizations panel data from database");
        }
    }
}
```

### System 3: Populate Sidebar with Organizations

**Purpose**: Spawn organization items in sidebar when data loads

```rust
pub fn populate_organizations_sidebar(
    mut org_load_events: EventReader<OrganizationsDataLoaded>,
    mut panel_entities: ResMut<OrganizationsPanelEntities>,
    mut commands: Commands,
    mut org_state: ResMut<CurrentOrganizationState>,
) {
    for event in org_load_events.read() {
        // Clear existing organization items
        for &item_entity in panel_entities.organization_items.values() {
            commands.entity(item_entity).despawn_recursive();
        }
        panel_entities.organization_items.clear();
        panel_entities.settings_buttons.clear();
        
        // Store organizations in state
        org_state.organizations = event.organizations.clone();
        
        // Spawn organization items
        let mut y_offset = 65.0;
        let item_height = 60.0;
        
        for org in &event.organizations {
            let is_selected = org_state.selected_organization_id
                .as_ref()
                .map(|id| id == &org.organization_id)
                .unwrap_or(false);
            
            // Organization item container
            let item_entity = commands.spawn((
                OrganizationSidebarItem {
                    organization_id: org.organization_id.clone(),
                    organization_name: org.organization_name.clone(),
                    logo_url: org.logo_url.clone(),
                    is_selected,
                },
                UiLayout::window()
                    .size((Rl(90.0), Ab(item_height)))
                    .pos((Ab(15.0), Ab(y_offset)))
                    .pack(),
                UiColor::from(if is_selected {
                    Color::srgba(0.15, 0.15, 0.18, 1.0) // Selected background
                } else {
                    Color::srgba(0.0, 0.0, 0.0, 0.0) // Transparent
                }),
                UiHover::new(),
                UiClicked::new(),
                Pickable::default(),
                Interaction::None,
                Name::new(format!("OrgItem_{}", org.organization_id)),
            )).id();
            
            // Organization logo/icon
            let org_icon = commands.spawn((
                UiLayout::window()
                    .size((Ab(40.0), Ab(40.0)))
                    .pos((Ab(5.0), Ab(10.0)))
                    .pack(),
                UiColor::from(Color::srgba(0.3, 0.5, 0.8, 1.0)),
                Text::new(org.logo_url.as_ref().map(|_| "🏢").unwrap_or("Y")),
                UiTextSize::from(Em(1.5)),
                Name::new("OrgIcon"),
            )).id();
            
            // Organization name
            let org_name = commands.spawn((
                UiLayout::window()
                    .size((Rl(60.0), Ab(30.0)))
                    .pos((Ab(52.0), Ab(15.0)))
                    .pack(),
                Text::new(&org.organization_name),
                UiTextSize::from(Em(0.95)),
                UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
                Name::new("OrgName"),
            )).id();
            
            // Settings button (gear icon)
            let settings_button = commands.spawn((
                OrganizationSettingsButton {
                    organization_id: org.organization_id.clone(),
                },
                UiLayout::window()
                    .size((Ab(30.0), Ab(30.0)))
                    .pos((Rl(95.0), Ab(15.0)))
                    .anchor(Anchor::TopRight)
                    .pack(),
                UiColor::from(Color::srgba(0.4, 0.4, 0.45, 1.0)),
                UiHover::new(),
                UiClicked::new(),
                Text::new("⚙️"),
                UiTextSize::from(Em(1.0)),
                Pickable::default(),
                Interaction::None,
                Name::new(format!("SettingsButton_{}", org.organization_id)),
            )).id();
            
            commands.entity(item_entity).push_children(&[org_icon, org_name, settings_button]);
            commands.entity(panel_entities.sidebar_container).add_child(item_entity);
            
            panel_entities.organization_items.insert(org.organization_id.clone(), item_entity);
            panel_entities.settings_buttons.insert(org.organization_id.clone(), settings_button);
            
            y_offset += item_height + 10.0;
        }
        
        info!("✅ Populated sidebar with {} organizations", event.organizations.len());
        
        // If no organization selected, select the first one
        if org_state.selected_organization_id.is_none() && !event.organizations.is_empty() {
            org_state.selected_organization_id = Some(event.organizations[0].organization_id.clone());
        }
    }
}
```

### System 4: Handle Organization Selection

**Purpose**: Update detail view when organization selected from sidebar

```rust
pub fn handle_organization_selection(
    sidebar_items: Query<
        (&OrganizationSidebarItem, &Interaction, &UiClicked),
        Changed<Interaction>
    >,
    mut selection_events: EventWriter<OrganizationSelected>,
) {
    for (item, interaction, clicked) in sidebar_items.iter() {
        if *interaction == Interaction::Pressed && clicked.progress > 0.9 {
            selection_events.send(OrganizationSelected {
                organization_id: item.organization_id.clone(),
            });
            
            info!("🏢 Organization selected: {}", item.organization_name);
        }
    }
}
```

### System 5: Update Detail View

**Purpose**: Show selected organization details in the right panel

```rust
pub fn update_detail_view(
    mut selection_events: EventReader<OrganizationSelected>,
    mut org_state: ResMut<CurrentOrganizationState>,
    panel_entities: Res<OrganizationsPanelEntities>,
    mut visibility_query: Query<&mut Visibility>,
    mut text_query: Query<&mut Text>,
    mut badge_query: Query<(&mut SubscriptionPlanBadge, &mut Visibility)>,
    mut action_buttons: Query<&mut OrganizationActionButton>,
) {
    for event in selection_events.read() {
        // Update selected organization in state
        org_state.selected_organization_id = Some(event.organization_id.clone());
        
        // Find the selected organization
        if let Some(org) = org_state.organizations.iter()
            .find(|o| o.organization_id == event.organization_id) 
        {
            // Show detail container
            if let Ok(mut vis) = visibility_query.get_mut(panel_entities.detail_container) {
                *vis = Visibility::Visible;
            }
            
            // Update organization name
            if let Ok(mut text) = text_query.get_mut(panel_entities.detail_name_text) {
                *text = Text::new(&org.organization_name);
            }
            
            // Update plan badge
            if let Ok((mut badge, mut badge_vis)) = badge_query.get_mut(panel_entities.detail_plan_badge) {
                if org.has_paid_plan {
                    badge.plan = org.subscription_plan.clone();
                    *badge_vis = Visibility::Visible;
                } else {
                    *badge_vis = Visibility::Hidden;
                }
            }
            
            // Update action buttons with current organization ID
            for mut button in action_buttons.iter_mut() {
                button.organization_id = event.organization_id.clone();
            }
            
            info!("📋 Updated detail view for organization: {}", org.organization_name);
        }
    }
}
```

### System 6: Handle Organization Action Buttons

**Purpose**: Process clicks on action buttons in detail view

```rust
pub fn handle_organization_action_buttons(
    buttons: Query<
        (&OrganizationActionButton, &Interaction, &UiClicked),
        Changed<Interaction>
    >,
    mut action_events: EventWriter<OrganizationActionRequested>,
    mut leave_events: EventWriter<LeaveOrganizationRequested>,
    org_state: Res<CurrentOrganizationState>,
) {
    for (button, interaction, clicked) in buttons.iter() {
        if *interaction == Interaction::Pressed && clicked.progress > 0.9 {
            // Special handling for leave organization (needs confirmation)
            if button.action == OrganizationAction::LeaveOrganization {
                if let Some(org) = org_state.organizations.iter()
                    .find(|o| o.organization_id == button.organization_id) 
                {
                    leave_events.send(LeaveOrganizationRequested {
                        organization_id: org.organization_id.clone(),
                        organization_name: org.organization_name.clone(),
                    });
                    
                    info!("⚠️ Leave organization requested: {}", org.organization_name);
                }
            } else {
                // All other actions
                action_events.send(OrganizationActionRequested {
                    organization_id: button.organization_id.clone(),
                    action: button.action,
                });
                
                info!("🔧 Organization action: {:?} for {}", button.action, button.organization_id);
            }
        }
    }
}
```

### System 7: Handle Create Organization Button

**Purpose**: Open create organization flow when button clicked

```rust
pub fn handle_create_organization_button(
    buttons: Query<(&Interaction, &UiClicked), (With<CreateOrganizationButton>, Changed<Interaction>)>,
    mut create_events: EventWriter<CreateOrganizationRequested>,
) {
    for (interaction, clicked) in buttons.iter() {
        if *interaction == Interaction::Pressed && clicked.progress > 0.9 {
            create_events.send(CreateOrganizationRequested);
            
            info!("➕ Create new organization requested");
            
            // TODO: Open create organization modal/flow
        }
    }
}
```

---

## Plugin Definition

```rust
pub struct OrganizationsPanelPlugin;

impl Plugin for OrganizationsPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CurrentOrganizationState>()
            .add_event::<OrganizationSelected>()
            .add_event::<OrganizationActionRequested>()
            .add_event::<CreateOrganizationRequested>()
            .add_event::<LeaveOrganizationRequested>()
            .add_event::<OrganizationsDataLoaded>()
            .add_systems(Startup, setup_organizations_panel)
            .add_systems(Update, (
                load_organizations_data,
                populate_organizations_sidebar,
                handle_organization_selection,
                update_detail_view,
                handle_organization_action_buttons,
                handle_create_organization_button,
            ).chain());
    }
}
```

---

## Acceptance Criteria

1. ✅ Panel displays with sidebar + detail view layout (30/70 split)
2. ✅ Sidebar lists all user's organizations with logos and names
3. ✅ Settings gear icon appears next to each organization
4. ✅ Create New Organization button at bottom of sidebar
5. ✅ Clicking organization shows its details in right panel
6. ✅ Detail view shows org logo, name, and paid plan badge
7. ✅ Manage Subscription button visible for paid orgs
8. ✅ Manage and Edit buttons work for admin/owner roles
9. ✅ Open Store button triggers organization store
10. ✅ Danger Zone section with Leave Organization button
11. ✅ Leave Organization requests confirmation before action
12. ✅ All database interactions via events
13. ✅ Performance targets met (load < 100ms, interactions < 16ms)
14. ✅ NO STUBS in implementation
15. ✅ Tests pass with 100% success
16. ✅ Follows architecture patterns from TASK7.0 and TASK7.C

---

## Implementation Notes

**External Integrations Required:**
- Organization management API
- Organization store/marketplace
- Invitation system
- Role-based access control

**DO NOT:**
- ❌ Allow leaving if user is sole owner
- ❌ Show management options to regular members
- ❌ Hardcode organization data

**DO:**
- ✅ Validate user permissions before actions
- ✅ Show confirmation dialogs for destructive actions
- ✅ Handle empty state (no organizations)
- ✅ Preserve selected organization across sessions

---

## Estimated Time Breakdown

- Sidebar + detail view layout: 1.5 hours
- Organization list and selection: 1.5 hours
- Detail view sections and buttons: 1.5 hours
- Database integration and events: 1 hour
- Testing and polish: 1 hour

**Total: 5-6 hours**

**Ready for code review** ✅

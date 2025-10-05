# TASK7.5: Settings Panel - Account

**Status**: Not Started  
**Estimated Time**: 4-5 hours  
**Priority**: Medium  
**Dependencies**: TASK7.0-INFRASTRUCTURE.md, TASK7.C-COMPONENTS.md

---

## Objective

Implement the Account settings panel displaying user profile information, subscription status, Pro feature access levels, organization membership benefits, and developer capabilities. This panel provides account management with visual feature lists showing which capabilities are available based on subscription tier, along with subscription management and logout functionality.

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
- Authentication system (login/logout)
- Subscription management API
- User profile service

---

## Screenshot Reference

![Account Menu](/Volumes/samsung_t9/action-items/spec/screenshots/Account_Menu.png)

**Visual Structure:**

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│       ┌─────────┐                                       │
│       │  📷     │                                        │
│       │ Avatar  │                                        │
│       │  (✓)    │  Verified badge                       │
│       └─────────┘                                       │
│                                                         │
│        David Maple                                      │
│    kloudsamurai · david@cloudsamurai.ai                │
│                                                         │
│    You are subscribed to Raycast Pro                   │
│    via a paid Team plan.                               │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│  Pro                                                    │
│                                                         │
│  ✨ Raycast AI                            [Pro]  ℹ️     │
│  ☁️  Cloud Sync                           [Pro]  ℹ️     │
│  🎨 Custom Themes                         [Pro]  ℹ️     │
│  📋 Unlimited Clipboard History           [Pro]  ℹ️     │
│  ⏰ Scheduled Exports                     [Pro]  ℹ️     │
│  🌐 Translator                            [Pro]  ℹ️     │
│  🪟 Custom Window Management Commands     [Pro]  ℹ️     │
│  T  Unlimited Notes                       [Pro]  ℹ️     │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│  Organizations                                          │
│                                                         │
│  ⚙️  Private Extensions                         ℹ️     │
│  🔗 Shared Quicklinks                            ℹ️     │
│  📋 Shared Snippets                              ℹ️     │
│  ✨ Pro Features for All Members         [Pro]  ℹ️     │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│  Developer                                              │
│                                                         │
│  🔌 Developer API                                ℹ️     │
│  ⚙️  Custom Extensions                           ℹ️     │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│  [Log Out]                  [Manage Subscription]      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## Database Schema

### Table: `user_account`

```sql
DEFINE TABLE user_account SCHEMALESS;

-- User identification
DEFINE FIELD user_id ON TABLE user_account TYPE string;
DEFINE FIELD username ON TABLE user_account TYPE string;
DEFINE FIELD display_name ON TABLE user_account TYPE string;
DEFINE FIELD email ON TABLE user_account TYPE string;

-- Profile
DEFINE FIELD avatar_url ON TABLE user_account TYPE string;
DEFINE FIELD is_verified ON TABLE user_account TYPE bool DEFAULT false;

-- Authentication
DEFINE FIELD auth_token ON TABLE user_account TYPE string;
DEFINE FIELD auth_provider ON TABLE user_account TYPE string;
  -- Values: "email", "google", "apple", "github"

DEFINE FIELD last_login_at ON TABLE user_account TYPE datetime;
DEFINE FIELD account_created_at ON TABLE user_account TYPE datetime;

DEFINE INDEX idx_user_id ON TABLE user_account COLUMNS user_id UNIQUE;
DEFINE INDEX idx_email ON TABLE user_account COLUMNS email UNIQUE;
```

### Table: `user_subscription`

```sql
DEFINE TABLE user_subscription SCHEMALESS;

-- Subscription details
DEFINE FIELD user_id ON TABLE user_subscription TYPE string;
DEFINE FIELD subscription_tier ON TABLE user_subscription TYPE string DEFAULT "free";
  -- Values: "free", "pro", "team", "enterprise"

DEFINE FIELD subscription_status ON TABLE user_subscription TYPE string DEFAULT "inactive";
  -- Values: "active", "inactive", "trial", "expired", "cancelled"

DEFINE FIELD subscription_type ON TABLE user_subscription TYPE string;
  -- Values: "individual", "team", "organization"

-- Billing
DEFINE FIELD billing_period ON TABLE user_subscription TYPE string;
  -- Values: "monthly", "annual"

DEFINE FIELD subscription_start_at ON TABLE user_subscription TYPE datetime;
DEFINE FIELD subscription_expires_at ON TABLE user_subscription TYPE datetime;
DEFINE FIELD trial_ends_at ON TABLE user_subscription TYPE datetime;

-- Feature access
DEFINE FIELD has_ai_access ON TABLE user_subscription TYPE bool DEFAULT false;
DEFINE FIELD has_cloud_sync ON TABLE user_subscription TYPE bool DEFAULT false;
DEFINE FIELD has_custom_themes ON TABLE user_subscription TYPE bool DEFAULT false;
DEFINE FIELD has_unlimited_clipboard ON TABLE user_subscription TYPE bool DEFAULT false;
DEFINE FIELD has_scheduled_exports ON TABLE user_subscription TYPE bool DEFAULT false;
DEFINE FIELD has_translator ON TABLE user_subscription TYPE bool DEFAULT false;
DEFINE FIELD has_window_management ON TABLE user_subscription TYPE bool DEFAULT false;
DEFINE FIELD has_unlimited_notes ON TABLE user_subscription TYPE bool DEFAULT false;

-- Organization membership
DEFINE FIELD organization_ids ON TABLE user_subscription TYPE array DEFAULT [];
DEFINE FIELD has_private_extensions ON TABLE user_subscription TYPE bool DEFAULT false;
DEFINE FIELD has_shared_quicklinks ON TABLE user_subscription TYPE bool DEFAULT false;
DEFINE FIELD has_shared_snippets ON TABLE user_subscription TYPE bool DEFAULT false;

-- Developer access
DEFINE FIELD has_developer_api ON TABLE user_subscription TYPE bool DEFAULT false;
DEFINE FIELD has_custom_extensions ON TABLE user_subscription TYPE bool DEFAULT true;
  -- Custom extensions available to all users

DEFINE INDEX idx_user_subscription ON TABLE user_subscription COLUMNS user_id UNIQUE;
```

### Table: `account_ui_state`

```sql
DEFINE TABLE account_ui_state SCHEMALESS;

-- UI preferences for account panel
DEFINE FIELD show_pro_features ON TABLE account_ui_state TYPE bool DEFAULT true;
DEFINE FIELD show_org_features ON TABLE account_ui_state TYPE bool DEFAULT true;
DEFINE FIELD show_developer_features ON TABLE account_ui_state TYPE bool DEFAULT true;

DEFINE FIELD last_visited_at ON TABLE account_ui_state TYPE datetime;
```

---

## Component Structure

### Components

```rust
use bevy::prelude::*;
use chrono::{DateTime, Utc};

/// Marker component for the Account panel root entity
#[derive(Component, Debug)]
pub struct AccountPanel;

/// Component for user profile display
#[derive(Component, Debug, Clone)]
pub struct UserProfile {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
}

/// Component for user avatar entity
#[derive(Component, Debug)]
pub struct UserAvatar {
    pub avatar_url: Option<String>,
}

/// Component for subscription status badge
#[derive(Component, Debug)]
pub struct SubscriptionStatusBadge {
    pub subscription_tier: SubscriptionTier,
    pub subscription_type: SubscriptionType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubscriptionTier {
    Free,
    Pro,
    Team,
    Enterprise,
}

impl SubscriptionTier {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Free => "Free",
            Self::Pro => "Pro",
            Self::Team => "Team",
            Self::Enterprise => "Enterprise",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            Self::Free => Color::srgba(0.6, 0.6, 0.65, 1.0),
            Self::Pro => Color::srgba(0.3, 0.7, 1.0, 1.0),
            Self::Team => Color::srgba(0.5, 0.3, 0.9, 1.0),
            Self::Enterprise => Color::srgba(0.9, 0.6, 0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubscriptionType {
    Individual,
    Team,
    Organization,
}

impl SubscriptionType {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Individual => "Personal subscription",
            Self::Team => "Team plan membership",
            Self::Organization => "Organization subscription",
        }
    }
}

/// Component for feature list items
#[derive(Component, Debug, Clone)]
pub struct FeatureItem {
    pub feature_id: String,
    pub display_name: String,
    pub icon: String,
    pub description: String,
    pub is_available: bool,
    pub requires_pro: bool,
    pub category: FeatureCategory,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeatureCategory {
    Pro,
    Organization,
    Developer,
}

impl FeatureCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pro => "Pro",
            Self::Organization => "Organizations",
            Self::Developer => "Developer",
        }
    }
}

impl FeatureItem {
    /// Define Pro features
    pub fn pro_features() -> Vec<Self> {
        vec![
            Self {
                feature_id: "ai".to_string(),
                display_name: "Raycast AI".to_string(),
                icon: "✨".to_string(),
                description: "AI-powered commands and chat".to_string(),
                is_available: false, // Updated from database
                requires_pro: true,
                category: FeatureCategory::Pro,
            },
            Self {
                feature_id: "cloud_sync".to_string(),
                display_name: "Cloud Sync".to_string(),
                icon: "☁️".to_string(),
                description: "Sync settings across devices".to_string(),
                is_available: false,
                requires_pro: true,
                category: FeatureCategory::Pro,
            },
            Self {
                feature_id: "custom_themes".to_string(),
                display_name: "Custom Themes".to_string(),
                icon: "🎨".to_string(),
                description: "Create and import custom themes".to_string(),
                is_available: false,
                requires_pro: true,
                category: FeatureCategory::Pro,
            },
            Self {
                feature_id: "unlimited_clipboard".to_string(),
                display_name: "Unlimited Clipboard History".to_string(),
                icon: "📋".to_string(),
                description: "Store unlimited clipboard items".to_string(),
                is_available: false,
                requires_pro: true,
                category: FeatureCategory::Pro,
            },
            Self {
                feature_id: "scheduled_exports".to_string(),
                display_name: "Scheduled Exports".to_string(),
                icon: "⏰".to_string(),
                description: "Automatically export data on schedule".to_string(),
                is_available: false,
                requires_pro: true,
                category: FeatureCategory::Pro,
            },
            Self {
                feature_id: "translator".to_string(),
                display_name: "Translator".to_string(),
                icon: "🌐".to_string(),
                description: "Translate text between languages".to_string(),
                is_available: false,
                requires_pro: true,
                category: FeatureCategory::Pro,
            },
            Self {
                feature_id: "window_management".to_string(),
                display_name: "Custom Window Management Commands".to_string(),
                icon: "🪟".to_string(),
                description: "Advanced window positioning rules".to_string(),
                is_available: false,
                requires_pro: true,
                category: FeatureCategory::Pro,
            },
            Self {
                feature_id: "unlimited_notes".to_string(),
                display_name: "Unlimited Notes".to_string(),
                icon: "T".to_string(),
                description: "Store unlimited notes and documents".to_string(),
                is_available: false,
                requires_pro: true,
                category: FeatureCategory::Pro,
            },
        ]
    }
    
    /// Define Organization features
    pub fn organization_features() -> Vec<Self> {
        vec![
            Self {
                feature_id: "private_extensions".to_string(),
                display_name: "Private Extensions".to_string(),
                icon: "⚙️".to_string(),
                description: "Share extensions within organization".to_string(),
                is_available: false,
                requires_pro: false, // Org feature, not Pro
                category: FeatureCategory::Organization,
            },
            Self {
                feature_id: "shared_quicklinks".to_string(),
                display_name: "Shared Quicklinks".to_string(),
                icon: "🔗".to_string(),
                description: "Share quicklinks with team members".to_string(),
                is_available: false,
                requires_pro: false,
                category: FeatureCategory::Organization,
            },
            Self {
                feature_id: "shared_snippets".to_string(),
                display_name: "Shared Snippets".to_string(),
                icon: "📋".to_string(),
                description: "Share text snippets with organization".to_string(),
                is_available: false,
                requires_pro: false,
                category: FeatureCategory::Organization,
            },
            Self {
                feature_id: "pro_for_all".to_string(),
                display_name: "Pro Features for All Members".to_string(),
                icon: "✨".to_string(),
                description: "All team members get Pro features".to_string(),
                is_available: false,
                requires_pro: true,
                category: FeatureCategory::Organization,
            },
        ]
    }
    
    /// Define Developer features
    pub fn developer_features() -> Vec<Self> {
        vec![
            Self {
                feature_id: "developer_api".to_string(),
                display_name: "Developer API".to_string(),
                icon: "🔌".to_string(),
                description: "Access programmatic API".to_string(),
                is_available: false,
                requires_pro: false,
                category: FeatureCategory::Developer,
            },
            Self {
                feature_id: "custom_extensions".to_string(),
                display_name: "Custom Extensions".to_string(),
                icon: "⚙️".to_string(),
                description: "Build and install custom extensions".to_string(),
                is_available: true, // Available to all users
                requires_pro: false,
                category: FeatureCategory::Developer,
            },
        ]
    }
}

/// Component for feature info buttons
#[derive(Component, Debug)]
pub struct FeatureInfoButton {
    pub feature_id: String,
}

/// Component for action buttons
#[derive(Component, Debug)]
pub struct AccountActionButton {
    pub action: AccountAction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccountAction {
    LogOut,
    ManageSubscription,
}

impl AccountAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::LogOut => "Log Out",
            Self::ManageSubscription => "Manage Subscription",
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            Self::LogOut => Color::srgba(0.8, 0.2, 0.2, 1.0),
            Self::ManageSubscription => Color::srgba(0.3, 0.5, 0.8, 1.0),
        }
    }
}
```

### Resources

```rust
/// Resource tracking all entities in the Account panel
#[derive(Resource)]
pub struct AccountPanelEntities {
    pub panel_root: Entity,
    pub user_avatar: Entity,
    pub user_name_text: Entity,
    pub user_email_text: Entity,
    pub subscription_status_text: Entity,
    
    // Feature sections
    pub pro_features_section: Entity,
    pub org_features_section: Entity,
    pub developer_features_section: Entity,
    
    // Feature items (mapped by feature_id)
    pub feature_items: HashMap<String, Entity>,
    pub info_buttons: HashMap<String, Entity>,
    
    // Action buttons
    pub logout_button: Entity,
    pub manage_subscription_button: Entity,
}

/// Resource containing current user account information
#[derive(Resource, Default)]
pub struct CurrentUserAccount {
    pub profile: Option<UserProfile>,
    pub subscription_tier: SubscriptionTier,
    pub subscription_type: SubscriptionType,
    pub subscription_status: String,
    pub feature_access: HashMap<String, bool>,
}

impl Default for SubscriptionTier {
    fn default() -> Self {
        Self::Free
    }
}

impl Default for SubscriptionType {
    fn default() -> Self {
        Self::Individual
    }
}
```

### Events

```rust
/// Event sent when user clicks Log Out button
#[derive(Event, Debug)]
pub struct LogOutRequested;

/// Event sent when user clicks Manage Subscription button
#[derive(Event, Debug)]
pub struct ManageSubscriptionRequested;

/// Event sent when user clicks feature info button
#[derive(Event, Debug)]
pub struct FeatureInfoRequested {
    pub feature_id: String,
    pub feature_description: String,
}

/// Event sent when account data loads successfully
#[derive(Event, Debug)]
pub struct AccountDataLoaded {
    pub user_profile: UserProfile,
    pub subscription_tier: SubscriptionTier,
    pub feature_access: HashMap<String, bool>,
}
```

---

## Implementation Details

### System 1: Setup Account Panel Entities

**Purpose**: Pre-allocate all Account panel UI entities during initialization

```rust
pub fn setup_account_panel(
    mut commands: Commands,
    settings_entities: Res<SettingsUIEntities>,
    asset_server: Res<AssetServer>,
) {
    let content_area = settings_entities.content_area;
    
    // Create panel root
    let panel_root = commands.spawn((
        AccountPanel,
        UiLayout::window()
            .size((Rl(100.0), Rl(100.0)))
            .pos((Rl(0.0), Rl(0.0)))
            .pack(),
        Visibility::Hidden,
        Name::new("AccountPanel"),
    )).id();
    
    commands.entity(content_area).add_child(panel_root);
    
    // ═══════════════════════════════════════════════════════════
    // USER PROFILE SECTION
    // ═══════════════════════════════════════════════════════════
    
    let profile_section = commands.spawn((
        UiLayout::window()
            .size((Rl(40.0), Ab(280.0)))
            .pos((Rl(25.0), Ab(30.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Name::new("ProfileSection"),
    )).id();
    
    // User avatar (circular)
    let user_avatar = commands.spawn((
        UserAvatar { avatar_url: None },
        UiLayout::window()
            .size((Ab(120.0), Ab(120.0)))
            .pos((Rl(50.0), Ab(0.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.5, 0.8, 1.0)),
        // TODO: Load actual avatar image from user profile
        // For now, show placeholder with initials
        Text::new("DM"),
        UiTextSize::from(Em(2.5)),
        Name::new("UserAvatar"),
    )).id();
    
    // Verified badge overlay on avatar
    let verified_badge = commands.spawn((
        UiLayout::window()
            .size((Ab(30.0), Ab(30.0)))
            .pos((Ab(95.0), Ab(95.0)))
            .pack(),
        UiColor::from(Color::srgba(0.0, 0.7, 1.0, 1.0)),
        Text::new("✓"),
        UiTextSize::from(Em(1.2)),
        Name::new("VerifiedBadge"),
    )).id();
    
    commands.entity(user_avatar).add_child(verified_badge);
    
    // Display name
    let user_name_text = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(35.0)))
            .pos((Rl(50.0), Ab(135.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Text::new("Loading..."),
        UiTextSize::from(Em(1.4)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
        Name::new("UserNameText"),
    )).id();
    
    // Username and email
    let user_email_text = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(30.0)))
            .pos((Rl(50.0), Ab(175.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Text::new("username · email@example.com"),
        UiTextSize::from(Em(0.9)),
        UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
        Name::new("UserEmailText"),
    )).id();
    
    // Subscription status text
    let subscription_status_text = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(50.0)))
            .pos((Rl(50.0), Ab(220.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Text::new("You are subscribed to Action Items Pro\nvia a paid Team plan."),
        UiTextSize::from(Em(0.9)),
        UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
        Name::new("SubscriptionStatusText"),
    )).id();
    
    commands.entity(profile_section).push_children(&[
        user_avatar,
        user_name_text,
        user_email_text,
        subscription_status_text,
    ]);
    
    commands.entity(panel_root).add_child(profile_section);
    
    let mut y_offset = 330.0;
    
    // ═══════════════════════════════════════════════════════════
    // PRO FEATURES SECTION
    // ═══════════════════════════════════════════════════════════
    
    let (pro_features_section, pro_feature_items, pro_info_buttons) = 
        create_feature_section(
            &mut commands,
            "Pro",
            FeatureItem::pro_features(),
            y_offset,
        );
    
    commands.entity(panel_root).add_child(pro_features_section);
    y_offset += 40.0 + (pro_feature_items.len() as f32 * 40.0) + 30.0;
    
    // ═══════════════════════════════════════════════════════════
    // ORGANIZATIONS FEATURES SECTION
    // ═══════════════════════════════════════════════════════════
    
    let (org_features_section, org_feature_items, org_info_buttons) = 
        create_feature_section(
            &mut commands,
            "Organizations",
            FeatureItem::organization_features(),
            y_offset,
        );
    
    commands.entity(panel_root).add_child(org_features_section);
    y_offset += 40.0 + (org_feature_items.len() as f32 * 40.0) + 30.0;
    
    // ═══════════════════════════════════════════════════════════
    // DEVELOPER FEATURES SECTION
    // ═══════════════════════════════════════════════════════════
    
    let (developer_features_section, dev_feature_items, dev_info_buttons) = 
        create_feature_section(
            &mut commands,
            "Developer",
            FeatureItem::developer_features(),
            y_offset,
        );
    
    commands.entity(panel_root).add_child(developer_features_section);
    y_offset += 40.0 + (dev_feature_items.len() as f32 * 40.0) + 50.0;
    
    // Combine all feature items and info buttons
    let mut all_feature_items = HashMap::new();
    let mut all_info_buttons = HashMap::new();
    
    all_feature_items.extend(pro_feature_items);
    all_feature_items.extend(org_feature_items);
    all_feature_items.extend(dev_feature_items);
    
    all_info_buttons.extend(pro_info_buttons);
    all_info_buttons.extend(org_info_buttons);
    all_info_buttons.extend(dev_info_buttons);
    
    // ═══════════════════════════════════════════════════════════
    // ACTION BUTTONS (bottom)
    // ═══════════════════════════════════════════════════════════
    
    let logout_button = commands.spawn((
        AccountActionButton {
            action: AccountAction::LogOut,
        },
        UiLayout::window()
            .size((Ab(120.0), Ab(40.0)))
            .pos((Rl(20.0), Ab(y_offset)))
            .pack(),
        UiColor::from(AccountAction::LogOut.color()),
        UiHover::new().forward_speed(8.0).backward_speed(4.0),
        UiClicked::new().forward_speed(15.0).backward_speed(10.0),
        Text::new(AccountAction::LogOut.label()),
        UiTextSize::from(Em(1.0)),
        Pickable::default(),
        Interaction::None,
        Name::new("LogOutButton"),
    )).id();
    
    let manage_subscription_button = commands.spawn((
        AccountActionButton {
            action: AccountAction::ManageSubscription,
        },
        UiLayout::window()
            .size((Ab(200.0), Ab(40.0)))
            .pos((Rl(60.0), Ab(y_offset)))
            .pack(),
        UiColor::from(AccountAction::ManageSubscription.color()),
        UiHover::new().forward_speed(8.0).backward_speed(4.0),
        UiClicked::new().forward_speed(15.0).backward_speed(10.0),
        Text::new(AccountAction::ManageSubscription.label()),
        UiTextSize::from(Em(1.0)),
        Pickable::default(),
        Interaction::None,
        Name::new("ManageSubscriptionButton"),
    )).id();
    
    commands.entity(panel_root).push_children(&[
        logout_button,
        manage_subscription_button,
    ]);
    
    // Store entities in resource
    commands.insert_resource(AccountPanelEntities {
        panel_root,
        user_avatar,
        user_name_text,
        user_email_text,
        subscription_status_text,
        pro_features_section,
        org_features_section,
        developer_features_section,
        feature_items: all_feature_items,
        info_buttons: all_info_buttons,
        logout_button,
        manage_subscription_button,
    });
    
    info!("✅ Pre-allocated Account panel UI entities");
}

// Helper function to create a feature section
fn create_feature_section(
    commands: &mut Commands,
    section_title: &str,
    features: Vec<FeatureItem>,
    y_offset: f32,
) -> (Entity, HashMap<String, Entity>, HashMap<String, Entity>) {
    let section = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(40.0 + features.len() as f32 * 40.0)))
            .pos((Rl(50.0), Ab(y_offset)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Name::new(format!("{}FeaturesSection", section_title)),
    )).id();
    
    // Section title
    let title = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(30.0)))
            .pos((Rl(0.0), Ab(0.0)))
            .pack(),
        Text::new(section_title),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
        Name::new("SectionTitle"),
    )).id();
    
    commands.entity(section).add_child(title);
    
    // Separator line
    let separator = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(1.0)))
            .pos((Rl(0.0), Ab(35.0)))
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
    )).id();
    
    commands.entity(section).add_child(separator);
    
    // Feature items
    let mut feature_items = HashMap::new();
    let mut info_buttons = HashMap::new();
    let mut item_y = 45.0;
    
    for feature in features {
        let item_entity = commands.spawn((
            feature.clone(),
            UiLayout::window()
                .size((Rl(100.0), Ab(35.0)))
                .pos((Rl(0.0), Ab(item_y)))
                .pack(),
            Name::new(format!("FeatureItem_{}", feature.feature_id)),
        )).id();
        
        // Icon + name
        let feature_text = commands.spawn((
            UiLayout::window()
                .size((Rl(70.0), Ab(30.0)))
                .pos((Ab(5.0), Ab(2.0)))
                .pack(),
            Text::new(format!("{}  {}", feature.icon, feature.display_name)),
            UiTextSize::from(Em(0.95)),
            UiColor::from(if feature.is_available {
                Color::srgba(0.9, 0.9, 0.95, 1.0)
            } else {
                Color::srgba(0.6, 0.6, 0.65, 1.0)
            }),
            Name::new("FeatureText"),
        )).id();
        
        commands.entity(item_entity).add_child(feature_text);
        
        // Pro badge (if requires pro)
        if feature.requires_pro {
            let pro_badge = commands.spawn((
                UiLayout::window()
                    .size((Ab(40.0), Ab(24.0)))
                    .pos((Rl(80.0), Ab(5.0)))
                    .pack(),
                UiColor::from(Color::srgba(0.3, 0.5, 0.8, 1.0)),
                Text::new("Pro"),
                UiTextSize::from(Em(0.8)),
                Name::new("ProBadge"),
            )).id();
            
            commands.entity(item_entity).add_child(pro_badge);
        }
        
        // Info button
        let info_button = commands.spawn((
            FeatureInfoButton {
                feature_id: feature.feature_id.clone(),
            },
            UiLayout::window()
                .size((Ab(24.0), Ab(24.0)))
                .pos((Rl(95.0), Ab(5.0)))
                .anchor(Anchor::TopRight)
                .pack(),
            UiColor::from(Color::srgba(0.4, 0.4, 0.45, 1.0)),
            UiHover::new(),
            UiClicked::new(),
            Text::new("ℹ️"),
            UiTextSize::from(Em(0.9)),
            Pickable::default(),
            Interaction::None,
            Name::new(format!("InfoButton_{}", feature.feature_id)),
        )).id();
        
        commands.entity(item_entity).add_child(info_button);
        
        commands.entity(section).add_child(item_entity);
        
        feature_items.insert(feature.feature_id.clone(), item_entity);
        info_buttons.insert(feature.feature_id.clone(), info_button);
        
        item_y += 40.0;
    }
    
    (section, feature_items, info_buttons)
}
```

### System 2: Load Account Data

**Purpose**: Load user account information when panel visible

```rust
pub fn load_account_data(
    mut panel_query: Query<&Visibility, (With<AccountPanel>, Changed<Visibility>)>,
    mut read_events: EventWriter<SettingsReadRequested>,
    panel_entities: Res<AccountPanelEntities>,
) {
    for visibility in panel_query.iter() {
        if *visibility == Visibility::Visible {
            // Load user account
            read_events.send(SettingsReadRequested {
                operation_id: Uuid::new_v4(),
                table: "user_account".to_string(),
                query: "SELECT * FROM user_account LIMIT 1".to_string(),
                requester: panel_entities.panel_root,
            });
            
            // Load subscription
            read_events.send(SettingsReadRequested {
                operation_id: Uuid::new_v4(),
                table: "user_subscription".to_string(),
                query: "SELECT * FROM user_subscription LIMIT 1".to_string(),
                requester: panel_entities.panel_root,
            });
            
            info!("📖 Loading Account panel data from database");
        }
    }
}
```

### System 3: Update Profile Display

**Purpose**: Update user profile UI when data loads

```rust
pub fn update_profile_display(
    mut account_load_events: EventReader<AccountDataLoaded>,
    panel_entities: Res<AccountPanelEntities>,
    mut name_text_query: Query<&mut Text>,
    mut email_text_query: Query<&mut Text>,
    mut subscription_text_query: Query<&mut Text>,
    mut avatar_query: Query<&mut UserAvatar>,
) {
    for event in account_load_events.read() {
        // Update display name
        if let Ok(mut text) = name_text_query.get_mut(panel_entities.user_name_text) {
            *text = Text::new(&event.user_profile.display_name);
        }
        
        // Update username and email
        if let Ok(mut text) = email_text_query.get_mut(panel_entities.user_email_text) {
            *text = Text::new(format!(
                "{} · {}",
                event.user_profile.username,
                event.user_profile.email
            ));
        }
        
        // Update subscription status
        if let Ok(mut text) = subscription_text_query.get_mut(panel_entities.subscription_status_text) {
            let status_text = match event.subscription_tier {
                SubscriptionTier::Free => "You are using the free version of Action Items.".to_string(),
                SubscriptionTier::Pro => "You are subscribed to Action Items Pro.".to_string(),
                SubscriptionTier::Team => "You are subscribed to Action Items Pro\nvia a paid Team plan.".to_string(),
                SubscriptionTier::Enterprise => "You are subscribed to Action Items Enterprise.".to_string(),
            };
            *text = Text::new(status_text);
        }
        
        // Update avatar
        if let Ok(mut avatar) = avatar_query.get_mut(panel_entities.user_avatar) {
            avatar.avatar_url = event.user_profile.avatar_url.clone();
            // TODO: Load actual avatar image
        }
        
        info!("✅ Updated profile display for user: {}", event.user_profile.display_name);
    }
}
```

### System 4: Update Feature Availability

**Purpose**: Update feature items based on subscription access

```rust
pub fn update_feature_availability(
    mut account_load_events: EventReader<AccountDataLoaded>,
    panel_entities: Res<AccountPanelEntities>,
    mut feature_query: Query<(&mut FeatureItem, &mut UiColor)>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut Text>,
) {
    for event in account_load_events.read() {
        // Update each feature item
        for (feature_id, &item_entity) in &panel_entities.feature_items {
            if let Ok((mut feature, mut color)) = feature_query.get_mut(item_entity) {
                // Check if user has access to this feature
                feature.is_available = event.feature_access
                    .get(feature_id)
                    .copied()
                    .unwrap_or(false);
                
                // Update text color based on availability
                if let Ok(children) = children_query.get(item_entity) {
                    for &child in children.iter() {
                        if let Ok(mut text) = text_query.get_mut(child) {
                            // Update text color
                            // (Actual implementation would set text.color)
                        }
                    }
                }
            }
        }
        
        info!("✅ Updated feature availability based on subscription");
    }
}
```

### System 5: Handle Action Buttons

**Purpose**: Handle log out and manage subscription buttons

```rust
pub fn handle_account_action_buttons(
    buttons: Query<
        (&AccountActionButton, &Interaction, &UiClicked),
        Changed<Interaction>
    >,
    mut logout_events: EventWriter<LogOutRequested>,
    mut manage_sub_events: EventWriter<ManageSubscriptionRequested>,
) {
    for (button, interaction, clicked) in buttons.iter() {
        if *interaction == Interaction::Pressed && clicked.progress > 0.9 {
            match button.action {
                AccountAction::LogOut => {
                    logout_events.send(LogOutRequested);
                    info!("🚪 Log out requested");
                }
                AccountAction::ManageSubscription => {
                    manage_sub_events.send(ManageSubscriptionRequested);
                    info!("💳 Manage subscription requested");
                }
            }
        }
    }
}
```

### System 6: Handle Feature Info Buttons

**Purpose**: Show feature details when info button clicked

```rust
pub fn handle_feature_info_buttons(
    buttons: Query<
        (&FeatureInfoButton, &Interaction, &UiClicked),
        Changed<Interaction>
    >,
    features: Query<&FeatureItem>,
    panel_entities: Res<AccountPanelEntities>,
    mut info_events: EventWriter<FeatureInfoRequested>,
) {
    for (button, interaction, clicked) in buttons.iter() {
        if *interaction == Interaction::Pressed && clicked.progress > 0.9 {
            // Find the feature to get description
            if let Some(&item_entity) = panel_entities.feature_items.get(&button.feature_id) {
                if let Ok(feature) = features.get(item_entity) {
                    info_events.send(FeatureInfoRequested {
                        feature_id: feature.feature_id.clone(),
                        feature_description: feature.description.clone(),
                    });
                    
                    info!("ℹ️ Feature info requested: {}", feature.display_name);
                    
                    // TODO: Show tooltip or modal with feature description
                }
            }
        }
    }
}
```

---

## Plugin Definition

```rust
pub struct AccountPanelPlugin;

impl Plugin for AccountPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CurrentUserAccount>()
            .add_event::<LogOutRequested>()
            .add_event::<ManageSubscriptionRequested>()
            .add_event::<FeatureInfoRequested>()
            .add_event::<AccountDataLoaded>()
            .add_systems(Startup, setup_account_panel)
            .add_systems(Update, (
                load_account_data,
                update_profile_display,
                update_feature_availability,
                handle_account_action_buttons,
                handle_feature_info_buttons,
            ).chain());
    }
}
```

---

## Acceptance Criteria

1. ✅ Panel displays user profile with avatar, name, username, email
2. ✅ Verified badge shows for verified accounts
3. ✅ Subscription status text shows current plan
4. ✅ Pro features section lists 8 Pro features with badges
5. ✅ Organization features section shows 4 org features
6. ✅ Developer features section shows 2 developer features
7. ✅ Feature availability reflects subscription tier
8. ✅ Info buttons show feature descriptions
9. ✅ Log Out button triggers logout flow
10. ✅ Manage Subscription button opens billing portal
11. ✅ All database interactions via events
12. ✅ Performance targets met (load < 100ms, interactions < 16ms)
13. ✅ NO STUBS in implementation
14. ✅ Tests pass with 100% success
15. ✅ Follows architecture patterns from TASK7.0 and TASK7.C

---

## Implementation Notes

**External Integrations Required:**
- Authentication service for login/logout
- Subscription management API (Stripe, Paddle, etc.)
- User profile service for avatar and account details
- Feature flag system for access control

**DO NOT:**
- ❌ Hardcode user data
- ❌ Store passwords or tokens in database
- ❌ Bypass authentication checks

**DO:**
- ✅ Use event-driven data loading
- ✅ Validate subscription status server-side
- ✅ Handle offline/error states gracefully
- ✅ Show loading states during data fetch

---

## Estimated Time Breakdown

- UI setup and profile display: 1.5 hours
- Feature sections and items: 1.5 hours
- Database integration and data loading: 1 hour
- Action buttons and event handlers: 0.5 hours
- Testing and polish: 1 hour

**Total: 4-5 hours**

**Ready for code review** ✅

{{ ... }}

## Overview
The Account Menu provides comprehensive user profile management, subscription status monitoring, feature access control, and organization membership administration. This interface serves as the central hub for user account operations and Pro feature management.

## Layout Architecture
- **Base Layout**: Tabbed navigation with "Account" tab active
- **Split Layout**: Left profile panel (40%) and right feature management panel (60%)
- **Vertical Sections**: Profile → Subscription Status → Features → Organizations → Developer → Actions

## Left Panel - Profile Section

### User Profile Display
{{ ... }}
- **Profile Image**: 
  - **Format**: Circular avatar with blue accent ring
  - **Size**: Large display (approximately 120x120px)
  - **Verification Badge**: Blue checkmark overlay for verified accounts
  - **Default State**: Placeholder or initials when no image provided
  - **Interactive**: Click to change/upload new profile image

### User Information
- **Display Name**: "David Maple"
  - **Font**: Large, bold display font
  - **Size**: Primary heading scale
  - **Color**: Pure white (#FFFFFF)
  - **Position**: Centered below profile image

- **Username and Email**: "kloudsamurai · david@cloudsamur.ai"
  - **Format**: Combined display with center dot separator
  - **Color**: Medium gray text
  - **Font**: Smaller than display name, regular weight
  - **Position**: Centered below display name

### Subscription Status Banner
- **Content**: "You are subscribed to Raycast Pro via a paid Team plan."
- **Styling**: Dark gray/charcoal background panel with rounded corners
- **Position**: Below user information, spanning width of left panel
- **Text Color**: Light gray/white for readability
- **Padding**: Consistent internal padding for visual balance
- **Dynamic**: Updates based on actual subscription status

## Visual Design Specifications

### Navigation Tab State
- **Account Tab**: Active state with darker background
- **Tab Icon**: User/person icon indicating account section
- **Tab Text**: "Account" in white text for active state
- **Tab Contrast**: Clear visual distinction from inactive tabs

### Split Panel Layout

## Bevy Implementation Details

### Account Menu Component Architecture

```rust
use bevy::{prelude::*, utils::HashMap};

// Account menu specific components
#[derive(Component, Reflect)]
pub struct AccountMenu;

#[derive(Component, Reflect)]
pub struct UserProfile {
    pub display_name: String,
    pub username: String,
    pub email: String,
    pub profile_image: Option<Handle<Image>>,
    pub is_verified: bool,
}

#[derive(Component, Reflect)]
pub struct ProfileAvatar {
    pub image_handle: Option<Handle<Image>>,
    pub is_loading: bool,
    pub needs_verification_badge: bool,
}

#[derive(Component, Reflect)]
pub struct SubscriptionStatus {
    pub plan_type: PlanType,
    pub status: SubscriptionState,
    pub renewal_date: Option<String>,
    pub organization_name: Option<String>,
}

#[derive(Component, Reflect)]
pub struct FeatureAccessSection {
    pub available_features: Vec<ProFeature>,
    pub usage_stats: HashMap<String, FeatureUsage>,
}

#[derive(Component, Reflect)]
pub struct OrganizationMembership {
    pub organization_id: String,
    pub organization_name: String,
    pub role: OrganizationRole,
    pub member_count: u32,
    pub status: MembershipStatus,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum PlanType {
    Free,
    Pro,
    Team,
    Enterprise,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum SubscriptionState {
    Active,
    Cancelled,
    Expired,
    PaymentPending,
    TrialActive,
}

#[derive(Clone, Reflect)]
pub struct ProFeature {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_available: bool,
    pub usage_limit: Option<u32>,
    pub current_usage: u32,
}

#[derive(Clone, Reflect)]
pub struct FeatureUsage {
    pub current_usage: u32,
    pub limit: Option<u32>,
    pub reset_date: Option<String>,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum OrganizationRole {
    Owner,
    Admin,
    Member,
    Guest,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum MembershipStatus {
    Active,
    Pending,
    Suspended,
    Invited,
}
```

### Resource Management for User Data

```rust
// Global user account state
#[derive(Resource, Reflect)]
pub struct UserAccountState {
    pub user_profile: UserProfileData,
    pub subscription: SubscriptionData,
    pub organizations: Vec<OrganizationData>,
    pub features: FeatureAccessData,
    pub developer_settings: DeveloperSettings,
}

#[derive(Clone, Reflect)]
pub struct UserProfileData {
    pub id: String,
    pub display_name: String,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub verification_status: VerificationStatus,
    pub join_date: String,
    pub last_active: Option<String>,
}

#[derive(Clone, Reflect)]
pub struct SubscriptionData {
    pub plan: PlanType,
    pub status: SubscriptionState,
    pub billing_cycle: BillingCycle,
    pub next_billing_date: Option<String>,
    pub payment_method: Option<PaymentMethod>,
    pub features_included: Vec<String>,
    pub organization_subscription: Option<OrganizationSubscription>,
}

#[derive(Clone, Reflect)]
pub struct OrganizationData {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub role: OrganizationRole,
    pub member_count: u32,
    pub subscription_tier: PlanType,
    pub features: Vec<String>,
    pub invitation_status: Option<MembershipStatus>,
}

#[derive(Clone, Reflect)]
pub struct FeatureAccessData {
    pub available_features: HashMap<String, ProFeature>,
    pub usage_tracking: HashMap<String, FeatureUsage>,
    pub feature_flags: HashMap<String, bool>,
}

#[derive(Clone, Reflect)]
pub struct DeveloperSettings {
    pub api_keys: Vec<ApiKeyData>,
    pub webhook_endpoints: Vec<WebhookData>,
    pub developer_mode_enabled: bool,
    pub debug_logging: bool,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Pending,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum BillingCycle {
    Monthly,
    Yearly,
    Lifetime,
}

#[derive(Clone, Reflect)]
pub struct PaymentMethod {
    pub type_name: String,
    pub last_four: String,
    pub expiry: Option<String>,
    pub is_default: bool,
}

#[derive(Clone, Reflect)]
pub struct OrganizationSubscription {
    pub organization_name: String,
    pub is_team_plan: bool,
    pub seats_total: u32,
    pub seats_used: u32,
}

#[derive(Clone, Reflect)]
pub struct ApiKeyData {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub created_at: String,
    pub last_used: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Clone, Reflect)]
pub struct WebhookData {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub secret: String,
}
```

### Event System for Account Management

```rust
// Account menu specific events
#[derive(Event, Reflect)]
pub enum AccountMenuEvent {
    // Profile events
    ProfileImageUploadRequested,
    ProfileImageChanged(Handle<Image>),
    DisplayNameChanged(String),
    ProfileVerificationRequested,
    
    // Subscription events
    SubscriptionDetailsRequested,
    PlanUpgradeRequested(PlanType),
    BillingHistoryRequested,
    PaymentMethodChanged,
    
    // Organization events
    OrganizationJoined(String),
    OrganizationLeft(String),
    OrganizationRoleChanged(String, OrganizationRole),
    OrganizationInviteReceived(OrganizationInvite),
    
    // Feature access events
    FeatureUsageUpdated(String, u32),
    FeatureLimitExceeded(String),
    FeatureUnlocked(String),
    
    // Account management
    AccountDeletionRequested,
    PasswordChangeRequested,
    EmailChangeRequested(String),
    TwoFactorToggled(bool),
}

#[derive(Event, Reflect)]
pub struct SubscriptionStatusChanged {
    pub old_status: SubscriptionState,
    pub new_status: SubscriptionState,
    pub effective_date: String,
}

#[derive(Event, Reflect)]
pub struct OrganizationInvite {
    pub organization_id: String,
    pub organization_name: String,
    pub inviter_name: String,
    pub role: OrganizationRole,
    pub expires_at: String,
}

#[derive(Event, Reflect)]
pub struct FeatureLimitWarning {
    pub feature_id: String,
    pub current_usage: u32,
    pub limit: u32,
    pub reset_date: Option<String>,
}
```

### System Architecture with User Management

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AccountMenuSystems {
    Input,
    ProfileManagement,
    SubscriptionSync,
    OrganizationSync,
    FeatureTracking,
    StateUpdate,
    Animation,
    Rendering,
}

impl Plugin for AccountMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<UserAccountState>()
            .init_resource::<AccountSyncManager>()
            .init_resource::<SubscriptionManager>()
            
            // Events
            .add_event::<AccountMenuEvent>()
            .add_event::<SubscriptionStatusChanged>()
            .add_event::<OrganizationInvite>()
            .add_event::<FeatureLimitWarning>()
            
            // System ordering
            .configure_sets(Update, (
                AccountMenuSystems::Input,
                AccountMenuSystems::ProfileManagement,
                AccountMenuSystems::SubscriptionSync,
                AccountMenuSystems::OrganizationSync,
                AccountMenuSystems::FeatureTracking,
                AccountMenuSystems::StateUpdate,
                AccountMenuSystems::Animation,
                AccountMenuSystems::Rendering,
            ).chain())
            
            // Systems
            .add_systems(Startup, (
                setup_account_menu,
                load_user_profile,
                initialize_subscription_status,
            ))
            
            .add_systems(Update, (
                handle_profile_interactions,
                handle_subscription_actions,
                handle_organization_actions,
                handle_feature_access_requests,
            ).in_set(AccountMenuSystems::Input))
            
            .add_systems(Update, (
                manage_profile_image_uploads,
                sync_profile_changes,
                handle_verification_requests,
            ).in_set(AccountMenuSystems::ProfileManagement))
            
            .add_systems(Update, (
                sync_subscription_status,
                process_plan_changes,
                update_billing_information,
            ).in_set(AccountMenuSystems::SubscriptionSync))
            
            .add_systems(Update, (
                sync_organization_memberships,
                process_organization_invites,
                update_organization_features,
            ).in_set(AccountMenuSystems::OrganizationSync))
            
            .add_systems(Update, (
                track_feature_usage,
                monitor_usage_limits,
                update_feature_availability,
            ).in_set(AccountMenuSystems::FeatureTracking))
            
            .add_systems(Update, (
                update_account_menu_state,
                persist_user_preferences,
                sync_remote_account_data,
            ).in_set(AccountMenuSystems::StateUpdate))
            
            .add_systems(Update, (
                animate_profile_avatar,
                animate_subscription_status_changes,
                animate_feature_unlocks,
            ).in_set(AccountMenuSystems::Animation))
            
            .add_systems(Update, (
                update_profile_display,
                update_subscription_display,
                update_organization_list,
                update_feature_usage_indicators,
            ).in_set(AccountMenuSystems::Rendering));
    }
}
```

### Layout Implementation with Profile and Subscription Management

```rust
fn setup_account_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    user_account: Res<UserAccountState>,
) {
    // Root container with split pane layout
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            max_width: Val::Px(1200.0),
            max_height: Val::Px(800.0),
            flex_direction: FlexDirection::Row,
            flex_grow: 0.0, // CRITICAL: Prevent expansion
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        AccountMenu,
    )).with_children(|parent| {
        
        // Left panel - Profile and subscription (40%)
        parent.spawn((
            Node {
                width: Val::Percent(40.0),
                max_width: Val::Px(480.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                flex_grow: 0.0,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(24.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        )).with_children(|left_parent| {
            spawn_profile_section(left_parent, &asset_server, &user_account.user_profile);
            spawn_subscription_status_section(left_parent, &asset_server, &user_account.subscription);
        });
        
        // Right panel - Features and organizations (60%)
        parent.spawn((
            Node {
                width: Val::Percent(60.0),
                max_width: Val::Px(720.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                flex_grow: 0.0,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(20.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        )).with_children(|right_parent| {
            spawn_features_section(right_parent, &asset_server, &user_account.features);
            spawn_organizations_section(right_parent, &asset_server, &user_account.organizations);
            spawn_developer_section(right_parent, &asset_server, &user_account.developer_settings);
        });
    });
}

fn spawn_profile_section(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    profile: &UserProfileData,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Auto,
            max_height: Val::Px(300.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            row_gap: Val::Px(16.0),
            ..default()
        },
        UserProfile {
            display_name: profile.display_name.clone(),
            username: profile.username.clone(),
            email: profile.email.clone(),
            profile_image: None, // Would be loaded from profile.avatar_url
            is_verified: profile.verification_status == VerificationStatus::Verified,
        },
    )).with_children(|profile_parent| {
        
        // Profile avatar with verification badge
        profile_parent.spawn((
            Node {
                width: Val::Px(120.0),
                height: Val::Px(120.0),
                max_width: Val::Px(120.0),
                max_height: Val::Px(120.0),
                position_type: PositionType::Relative,
                flex_grow: 0.0,
                ..default()
            },
        )).with_children(|avatar_parent| {
            
            // Main avatar image (circular)
            avatar_parent.spawn((
                ImageNode::new(
                    profile.avatar_url.as_ref()
                        .map(|url| asset_server.load(url))
                        .unwrap_or_else(|| asset_server.load("icons/default_avatar.png"))
                ),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BorderRadius::all(Val::Px(60.0)), // Circular
                ProfileAvatar {
                    image_handle: None,
                    is_loading: false,
                    needs_verification_badge: profile.verification_status == VerificationStatus::Verified,
                },
            ));
            
            // Verification badge overlay
            if profile.verification_status == VerificationStatus::Verified {
                avatar_parent.spawn((
                    ImageNode::new(asset_server.load("icons/verification_badge.png")),
                    Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(4.0),
                        right: Val::Px(4.0),
                        ..default()
                    },
                ));
            }
        });
        
        // User information section
        profile_parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                row_gap: Val::Px(4.0),
                ..default()
            },
        )).with_children(|info_parent| {
            
            // Display name
            info_parent.spawn((
                Text::new(&profile.display_name),
                TextFont {
                    font: asset_server.load("fonts/Inter-Bold.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));
            
            // Username and email
            info_parent.spawn((
                Text::new(&format!("{} · {}", profile.username, profile.email)),
                TextFont {
                    font: asset_server.load("fonts/Inter-Regular.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
    });
}

fn spawn_subscription_status_section(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    subscription: &SubscriptionData,
) {
    let status_text = match (&subscription.plan, &subscription.status) {
        (PlanType::Free, _) => "You are using Raycast's free tier.".to_string(),
        (PlanType::Pro, SubscriptionState::Active) => "You are subscribed to Raycast Pro.".to_string(),
        (PlanType::Team, SubscriptionState::Active) => {
            if let Some(org_sub) = &subscription.organization_subscription {
                format!("You are subscribed to Raycast Pro via {}.", org_sub.organization_name)
            } else {
                "You are subscribed to Raycast Pro via a Team plan.".to_string()
            }
        },
        (plan, status) => format!("Plan: {:?}, Status: {:?}", plan, status),
    };
    
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Auto,
            max_height: Val::Px(80.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            flex_grow: 0.0,
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        BorderColor::all(Color::srgb(0.25, 0.25, 0.25)),
        BorderRadius::all(Val::Px(8.0)),
        SubscriptionStatus {
            plan_type: subscription.plan.clone(),
            status: subscription.status.clone(),
            renewal_date: subscription.next_billing_date.clone(),
            organization_name: subscription.organization_subscription
                .as_ref()
                .map(|org| org.organization_name.clone()),
        },
    )).with_children(|status_parent| {
        status_parent.spawn((
            Text::new(&status_text),
            TextFont {
                font: asset_server.load("fonts/Inter-Regular.ttf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.85, 0.85)),
            TextLayout::new_with_justify(JustifyText::Center),
        ));
    });
}
```

### Testing Strategy for Account Management

```rust
#[cfg(test)]
mod account_menu_tests {
    use super::*;
    
    #[test]
    fn test_account_menu_initialization() {
        let mut app = setup_test_app();
        
        // Initialize with test user account data
        let test_account = UserAccountState {
            user_profile: UserProfileData {
                id: "user123".to_string(),
                display_name: "Test User".to_string(),
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
                avatar_url: None,
                verification_status: VerificationStatus::Verified,
                join_date: "2023-01-01".to_string(),
                last_active: None,
            },
            subscription: SubscriptionData {
                plan: PlanType::Pro,
                status: SubscriptionState::Active,
                billing_cycle: BillingCycle::Monthly,
                next_billing_date: Some("2024-01-01".to_string()),
                payment_method: None,
                features_included: vec!["ai".to_string(), "unlimited_history".to_string()],
                organization_subscription: None,
            },
            organizations: vec![],
            features: FeatureAccessData {
                available_features: HashMap::new(),
                usage_tracking: HashMap::new(),
                feature_flags: HashMap::new(),
            },
            developer_settings: DeveloperSettings {
                api_keys: vec![],
                webhook_endpoints: vec![],
                developer_mode_enabled: false,
                debug_logging: false,
            },
        };
        
        app.world_mut().insert_resource(test_account);
        app.update();
        
        // Verify account menu components were spawned
        let account_menu_count = app.world().query::<&AccountMenu>().iter(app.world()).count();
        assert_eq!(account_menu_count, 1);
        
        // Verify user profile was created
        let profile_count = app.world().query::<&UserProfile>().iter(app.world()).count();
        assert_eq!(profile_count, 1);
        
        // Verify subscription status was created
        let subscription_count = app.world().query::<&SubscriptionStatus>().iter(app.world()).count();
        assert_eq!(subscription_count, 1);
    }
    
    #[test]
    fn test_subscription_status_changes() {
        let mut app = setup_test_app();
        
        // Send subscription status change event
        app.world_mut().resource_mut::<Events<SubscriptionStatusChanged>>()
            .write(SubscriptionStatusChanged {
                old_status: SubscriptionState::TrialActive,
                new_status: SubscriptionState::Active,
                effective_date: "2024-01-01".to_string(),
            });
        
        app.update();
        
        // Verify the status change was processed
        // (In a real implementation, this would update the UI)
    }
    
    #[test]
    fn test_organization_invite_handling() {
        let mut app = setup_test_app();
        
        // Send organization invite event
        app.world_mut().resource_mut::<Events<OrganizationInvite>>()
            .write(OrganizationInvite {
                organization_id: "org123".to_string(),
                organization_name: "Test Organization".to_string(),
                inviter_name: "John Doe".to_string(),
                role: OrganizationRole::Member,
                expires_at: "2024-02-01".to_string(),
            });
        
        app.update();
        
        // Verify invite was processed
        let invite_events: Vec<_> = app.world()
            .resource::<Events<OrganizationInvite>>()
            .get_reader()
            .read(app.world().resource::<Events<OrganizationInvite>>())
            .collect();
        
        assert!(!invite_events.is_empty());
    }
}
- **Left Panel Width**: Approximately 40% of total interface width
- **Right Panel Width**: Approximately 60% of total interface width
- **Panel Separation**: Subtle visual separation between panels
- **Content Alignment**: Left panel center-aligned, right panel left-aligned

### Profile Section Visual Design
- **Profile Image Container**:
  - Circular frame with bright blue accent ring
  - Blue verification checkmark badge in bottom-right corner
  - Smooth circular clipping of profile photo
  - Consistent sizing and positioning
- **Text Hierarchy**:
  - Display name: Large, bold white text
  - Username/email: Medium gray text, smaller font size
  - Subscription status: Light text on dark background panel

### Feature List Visual Structure
- **Section Headers**: "Pro", "Organizations", "Developer"
  - **Font**: Medium weight, medium gray color
  - **Spacing**: Consistent vertical spacing above each section
- **Feature List Items**:
  - **Icon**: Left-aligned feature-specific icons
  - **Text**: Feature name in white text, left of center
  - **Pro Badge**: Blue "Pro" badges for premium features
  - **Info Icon**: Circular "i" icons on far right
  - **Layout**: Consistent horizontal alignment across all items

### Pro Badge Specifications
- **Color**: Bright blue (#007AFF or similar)
- **Text**: "Pro" in small white text
- **Shape**: Rounded rectangle with small border radius
- **Positioning**: Right-aligned, before info icon
- **Consistency**: Same styling across all Pro features

### Button Design System
- **Log Out Button**:
  - **Color**: Red/destructive styling (#FF4444 or similar)
  - **Position**: Bottom left of interface
  - **Text**: "Log Out" in contrast color
- **Manage Subscription Button**:
  - **Color**: Standard button styling with gray/blue background
  - **Position**: Bottom right of interface
  - **Text**: "Manage Subscription" in white text
- **Button Spacing**: Equal spacing between buttons and from panel edges

### Icon System Consistency
- **Feature Icons**: Unique icons for each feature (sparkle, cloud, palette, etc.)
- **Info Icons**: Consistent circular "i" design across all features
- **Icon Colors**: Maintain original feature branding while ensuring visibility
- **Icon Sizing**: Uniform size across all feature list items

### Color Palette Application
- **Background**: Dark theme consistent with other settings screens
- **Primary Text**: Pure white for names and primary labels
- **Secondary Text**: Medium gray for usernames, descriptions
- **Accent Blue**: Used for Pro badges, profile ring, verification badge
- **Destructive Red**: Used specifically for Log Out button
- **Panel Backgrounds**: Subtle dark gray variations for content sections

## Right Panel - Feature Management

### Pro Features Section
- **Title**: "Pro"
- **Layout**: Vertical list with consistent icon-text-badge-info structure

#### Feature List Items
1. **Raycast AI**
   - **Icon**: Sparkle/star icon
   - **Badge**: Blue "Pro" badge
   - **Info Icon**: Contextual help about AI features

2. **Cloud Sync**
   - **Icon**: Cloud icon
   - **Badge**: Blue "Pro" badge
   - **Info Icon**: Cloud sync feature details

3. **Custom Themes**
   - **Icon**: Palette/brush icon
   - **Badge**: Blue "Pro" badge
   - **Info Icon**: Theme customization capabilities

4. **Unlimited Clipboard History**
   - **Icon**: Clipboard icon
   - **Badge**: Blue "Pro" badge
   - **Info Icon**: Clipboard history feature details

5. **Scheduled Exports**
   - **Icon**: Upload/export icon
   - **Badge**: Blue "Pro" badge
   - **Info Icon**: Export scheduling functionality

6. **Translator**
   - **Icon**: Globe/translation icon
   - **Badge**: Blue "Pro" badge
   - **Info Icon**: Translation feature capabilities

7. **Custom Window Management Commands**
   - **Icon**: Window/layout icon
   - **Badge**: Blue "Pro" badge
   - **Info Icon**: Window management feature details

8. **Unlimited Notes**
   - **Icon**: Text/note icon
   - **Badge**: Blue "Pro" badge
   - **Info Icon**: Note-taking feature limitations and benefits

### Organizations Section
- **Title**: "Organizations"
- **Purpose**: Team and organization feature management

#### Organization Features
1. **Private Extensions**
   - **Icon**: Gear/extension icon
   - **Badge**: None (base feature)
   - **Info Icon**: Private extension sharing details

2. **Shared Quicklinks**
   - **Icon**: Link icon
   - **Badge**: None (base feature)
   - **Info Icon**: Team quicklink sharing functionality

3. **Shared Snippets**
   - **Icon**: Code snippet icon
   - **Badge**: None (base feature)
   - **Info Icon**: Team snippet sharing capabilities

4. **Pro Features for All Members**
   - **Icon**: Team/group icon
   - **Badge**: Blue "Pro" badge
   - **Info Icon**: Organization-wide Pro feature distribution

### Developer Section
- **Title**: "Developer"
- **Purpose**: Developer-focused features and API access

#### Developer Features
1. **Developer API**
   - **Icon**: Code/API icon
   - **Badge**: None (conditional access)
   - **Info Icon**: API access and documentation details

2. **Custom Extensions**
   - **Icon**: Plugin/extension icon
   - **Badge**: None (developer feature)
   - **Info Icon**: Custom extension development capabilities

## Bottom Action Panel

### Primary Actions
- **Log Out Button**
  - **Position**: Bottom left
  - **Style**: Red/destructive button styling
  - **Function**: Secure logout with session cleanup
  - **Confirmation**: Optional confirmation dialog for safety

- **Manage Subscription Button**
  - **Position**: Bottom right
  - **Style**: Primary button styling
  - **Function**: Navigate to subscription management interface
  - **Integration**: External billing system or in-app subscription flow

## Functional Requirements

### User Profile Management
- **Profile Image Upload**: Secure image upload with validation and resizing
- **Information Editing**: In-place editing of display name and contact information
- **Privacy Controls**: User control over information visibility and sharing
- **Verification System**: Email and identity verification workflows

### Subscription Management System
- **Real-time Status**: Dynamic subscription status monitoring
- **Feature Gating**: Automatic feature access control based on subscription
- **Usage Tracking**: Monitoring of feature usage against subscription limits
- **Billing Integration**: Seamless integration with billing and payment systems

### Feature Access Control
- **Dynamic Feature Lists**: Server-driven feature availability
- **Granular Permissions**: Individual feature enable/disable capabilities
- **Usage Limits**: Real-time monitoring of feature usage against limits
- **Upgrade Prompts**: Contextual upgrade suggestions for limited features

### Organization Management
- **Team Membership**: Display and management of organization memberships
- **Role-Based Access**: Different feature access based on organization role
- **Invitation System**: Team invitation and member management
- **Resource Sharing**: Control over shared resources and team features

## Bevy Implementation Examples

### Circular Profile Image
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Circular image clipping
- Reference: `./docs/bevy/examples/asset_loading/asset_loading.rs` - Profile image loading

### Split Panel Layout
- Reference: `./docs/bevy/examples/ui/flex_layout.rs` - Flexible panel sizing
- Reference: `./docs/bevy/examples/ui/ui.rs` - Multi-column layout structure

### Feature List with Badges
- Reference: `./docs/bevy/examples/ui/ui.rs` - List item layout with multiple elements
- Reference: `./docs/bevy/examples/ui/button.rs` - Badge styling and positioning

### Profile Information Editing
- Reference: `./docs/bevy/examples/ui/text_input.rs` - In-place text editing
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Input validation and handling

### Subscription Status Display
- Reference: `./docs/bevy/examples/ui/ui.rs` - Status panel styling
- Reference: `./docs/bevy/examples/ui/text.rs` - Dynamic text content updates

### Action Button Layout
- Reference: `./docs/bevy/examples/ui/button.rs` - Button styling variations
- Reference: `./docs/bevy/examples/ui/flex_layout.rs` - Bottom-aligned button layout

### Info Icon System
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Icon management and display
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Hover interactions for info display

## Security Requirements

### Profile Data Protection
- **Data Encryption**: Secure storage of profile information
- **Access Logging**: Comprehensive logging of profile access and modifications
- **Privacy Controls**: User control over data visibility and sharing
- **Data Retention**: Configurable data retention and deletion policies

### Authentication and Authorization
- **Session Management**: Secure session handling and timeout
- **Role-Based Access**: Granular permissions based on subscription and organization role
- **API Security**: Secure API access with proper authentication
- **Audit Trail**: Complete audit trail of account and subscription changes

### Subscription Security
- **Payment Data Protection**: Secure handling of billing and payment information
- **Feature Access Validation**: Server-side validation of feature access rights
- **Usage Monitoring**: Real-time monitoring of feature usage patterns
- **Fraud Prevention**: Automated detection of suspicious account activity

## State Management Requirements

### Profile State Synchronization
- **Real-time Updates**: Live synchronization of profile changes across devices
- **Conflict Resolution**: Handling of concurrent profile modifications
- **Offline Capability**: Local caching and offline profile access
- **Backup and Recovery**: Automated backup of profile information

### Subscription State Management
- **Live Status Updates**: Real-time subscription status monitoring
- **Feature Availability**: Dynamic feature list based on current subscription
- **Usage Tracking**: Real-time tracking of feature usage against limits
- **Billing Synchronization**: Automatic synchronization with billing systems

### Organization State Handling
- **Membership Updates**: Real-time updates of organization membership changes
- **Permission Synchronization**: Live updates of role-based permissions
- **Resource Sharing**: Dynamic updates of shared organizational resources
- **Team Notifications**: Real-time notifications of team changes and updates

## Error Handling and Recovery

### Profile Management Errors
- **Upload Failures**: Graceful handling of profile image upload failures
- **Validation Errors**: Clear feedback for profile information validation issues
- **Sync Failures**: Recovery mechanisms for profile synchronization failures
- **Network Issues**: Offline capability and network failure recovery

### Subscription Management Errors
- **Payment Failures**: Clear communication and recovery options for billing issues
- **Feature Access Errors**: Graceful degradation when feature access fails
- **Sync Issues**: Recovery mechanisms for subscription status synchronization
- **Billing System Integration**: Robust error handling for external billing system issues

### Organization Management Errors
- **Membership Errors**: Clear feedback for organization membership issues
- **Permission Failures**: Graceful handling of role-based access failures
- **Resource Sharing Issues**: Recovery mechanisms for shared resource access
- **Team Communication Failures**: Fallback options for team notification systems
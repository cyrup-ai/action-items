# Task 0: Organization Management Core System Implementation

## Overview
Implement the foundational organization management system with proper modular decomposition. Each component is separated into logical submodules under 300 lines each, following clean architecture principles.

## File Structure (All files <300 lines)

```
core/src/organization/
├── mod.rs                   # Module exports (20 lines)
├── plugin.rs               # Plugin registration (80 lines)
├── models/
│   ├── mod.rs              # Model exports (15 lines)
│   ├── organization.rs     # Organization data model (120 lines)
│   ├── member.rs           # Member data model (90 lines)
│   └── invitation.rs       # Invitation data model (70 lines)
├── resources/
│   ├── mod.rs              # Resource exports (12 lines)
│   ├── registry.rs         # Organization registry (90 lines)
│   ├── context.rs          # Organization context (60 lines)
│   └── permissions.rs      # Permission resource (80 lines)
├── events/
│   ├── mod.rs              # Event exports (15 lines)
│   ├── organization.rs     # Organization events (85 lines)
│   ├── membership.rs       # Membership events (70 lines)
│   └── permission.rs       # Permission events (65 lines)
├── systems/
│   ├── mod.rs              # System exports (20 lines)
│   ├── loader.rs           # Async loading systems (150 lines)
│   ├── handler.rs          # Event handler systems (180 lines)
│   ├── sync.rs             # Member sync systems (140 lines)
│   └── permission.rs       # Permission systems (120 lines)
├── ui/
│   ├── mod.rs              # UI exports (15 lines)
│   ├── components.rs       # UI components (200 lines)
│   ├── sidebar.rs          # Organization sidebar (180 lines)
│   ├── panel.rs            # Organization panel (220 lines)
│   └── interactions.rs     # Button interactions (160 lines)
└── utils/
    ├── mod.rs              # Utility exports (10 lines)
    ├── api.rs              # API integration (90 lines)
    └── security.rs         # Security utilities (70 lines)
```

## Module Breakdown

### 1. Plugin Registration
**File**: `core/src/organization/plugin.rs` (80 lines)
**Reference**: `./docs/bevy/examples/app/plugin.rs:15-53`

```rust
use bevy::prelude::*;
use crate::organization::{
    resources::*, events::*, systems::*, 
    models::*, ui::components::*
};

pub struct OrganizationPlugin;

impl Plugin for OrganizationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrganizationRegistry>()
            .init_resource::<OrganizationContext>()
            .init_resource::<UserPermissions>()
            .add_event::<OrganizationEvent>()
            .add_event::<MembershipEvent>()
            .add_event::<PermissionEvent>()
            .add_state::<OrganizationState>()
            .add_systems(Startup, initialize_organization_systems)
            .add_systems(OnEnter(OrganizationState::Loading), load_user_organizations)
            .add_systems(OnEnter(OrganizationState::Ready), setup_organization_ui)
            .add_systems(OnExit(OrganizationState::Ready), cleanup_organization_ui)
            .add_systems(Update, (
                handle_organization_events,
                update_organization_context,
                sync_member_permissions,
                process_organization_invitations,
                handle_organization_switching,
            ).chain().run_if(in_state(OrganizationState::Ready)))
            .add_observer(on_organization_created)
            .add_observer(on_member_added)
            .add_observer(on_permission_changed);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum OrganizationState {
    #[default]
    Uninitialized,
    Loading,
    Ready,
    Switching,
    Error,
}

fn initialize_organization_systems(mut next_state: ResMut<NextState<OrganizationState>>) {
    info!("Initializing organization management system");
    next_state.set(OrganizationState::Loading);
}
```

### 2. Organization Data Model
**File**: `core/src/organization/models/organization.rs` (120 lines)
**Reference**: `./docs/bevy/examples/ecs/system_param.rs:15-47`

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub logo_url: Option<String>,
    pub brand_colors: BrandColors,
    pub settings: OrganizationSettings,
    pub subscription: OrganizationSubscription,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
    pub member_count: usize,
    pub plan_type: PlanType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandColors {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub require_2fa: bool,
    pub allow_guest_access: bool,
    pub extension_approval_required: bool,
    pub audit_logging_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSubscription {
    pub plan_id: String,
    pub status: SubscriptionStatus,
    pub billing_cycle: BillingCycle,
    pub seats_total: usize,
    pub seats_used: usize,
    pub expires_at: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanType {
    Free,
    Team,
    Business,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubscriptionStatus {
    Active,
    Trialing,
    PastDue,
    Canceled,
    Unpaid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BillingCycle {
    Monthly,
    Yearly,
}

impl Organization {
    pub fn new(name: String, initial_settings: OrganizationSettings) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            logo_url: None,
            brand_colors: BrandColors::default(),
            settings: initial_settings,
            subscription: OrganizationSubscription::default(),
            created_at: std::time::SystemTime::now(),
            updated_at: std::time::SystemTime::now(),
            member_count: 1,
            plan_type: PlanType::Free,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.subscription.status, 
            SubscriptionStatus::Active | SubscriptionStatus::Trialing)
    }

    pub fn has_seats_available(&self) -> bool {
        self.subscription.seats_used < self.subscription.seats_total
    }
}

impl Default for BrandColors {
    fn default() -> Self {
        Self {
            primary: "#3B82F6".to_string(),
            secondary: "#64748B".to_string(),
            accent: "#10B981".to_string(),
        }
    }
}

impl Default for OrganizationSubscription {
    fn default() -> Self {
        Self {
            plan_id: "free".to_string(),
            status: SubscriptionStatus::Active,
            billing_cycle: BillingCycle::Monthly,
            seats_total: 5,
            seats_used: 1,
            expires_at: None,
        }
    }
}
```

### 3. Member Data Model  
**File**: `core/src/organization/models/member.rs` (90 lines)

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationMember {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: OrganizationRole,
    pub joined_at: std::time::SystemTime,
    pub last_active: Option<std::time::SystemTime>,
    pub permissions: Vec<Permission>,
    pub status: MemberStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMembership {
    pub org_id: String,
    pub role: OrganizationRole,
    pub joined_at: std::time::SystemTime,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrganizationRole {
    Owner,
    Admin,
    Member,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    ManageOrganization,
    ManageMembers,
    ManageSubscription,
    ManageExtensions,
    ViewMembers,
    ViewUsage,
    ViewAuditLogs,
    CreateInvitations,
    AccessExtensionStore,
    InstallExtensions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemberStatus {
    Active,
    Inactive,
    Suspended,
    PendingActivation,
}

impl OrganizationMember {
    pub fn new(email: String, role: OrganizationRole) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            email,
            name: None,
            role: role.clone(),
            joined_at: std::time::SystemTime::now(),
            last_active: None,
            permissions: get_role_permissions(&role),
            status: MemberStatus::Active,
        }
    }

    pub fn is_admin_or_owner(&self) -> bool {
        matches!(self.role, OrganizationRole::Owner | OrganizationRole::Admin)
    }
}

pub fn get_role_permissions(role: &OrganizationRole) -> Vec<Permission> {
    match role {
        OrganizationRole::Owner => vec![
            Permission::ManageOrganization,
            Permission::ManageMembers,
            Permission::ManageSubscription,
            Permission::ManageExtensions,
            Permission::ViewMembers,
            Permission::ViewUsage,
            Permission::ViewAuditLogs,
            Permission::CreateInvitations,
            Permission::AccessExtensionStore,
            Permission::InstallExtensions,
        ],
        OrganizationRole::Admin => vec![
            Permission::ManageMembers,
            Permission::ManageExtensions,
            Permission::ViewMembers,
            Permission::ViewUsage,
            Permission::ViewAuditLogs,
            Permission::CreateInvitations,
            Permission::AccessExtensionStore,
            Permission::InstallExtensions,
        ],
        OrganizationRole::Member => vec![
            Permission::ViewMembers,
            Permission::AccessExtensionStore,
            Permission::InstallExtensions,
        ],
        OrganizationRole::Guest => vec![
            Permission::AccessExtensionStore,
        ],
    }
}
```

### 4. Organization Registry Resource
**File**: `core/src/organization/resources/registry.rs` (90 lines)
**Reference**: `./docs/bevy/examples/ecs/system_param.rs:15-47`

```rust
use bevy::prelude::*;
use std::collections::HashMap;
use crate::organization::models::*;

#[derive(Resource, Clone, Debug, Default)]
pub struct OrganizationRegistry {
    pub organizations: HashMap<String, Organization>,
    pub user_memberships: HashMap<String, UserMembership>,
    pub pending_invitations: Vec<OrganizationInvitation>,
    pub active_sessions: HashMap<String, OrganizationSession>,
}

#[derive(Debug, Clone)]
pub struct OrganizationSession {
    pub org_id: String,
    pub started_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
    pub active_members: std::collections::HashSet<String>,
}

#[derive(SystemParam)]
pub struct OrganizationQuery<'w, 's> {
    pub registry: Res<'w, OrganizationRegistry>,
    pub context: ResMut<'w, OrganizationContext>,
    pub permissions: Res<'w, UserPermissions>,
    pub org_events: EventWriter<'w, OrganizationEvent>,
    pub membership_events: EventWriter<'w, MembershipEvent>,
}

impl<'w, 's> OrganizationQuery<'w, 's> {
    pub fn get_current_organization(&self) -> Option<&Organization> {
        self.context.current_org.as_ref()
            .and_then(|id| self.registry.organizations.get(id))
    }
    
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.effective_permissions.contains(permission)
    }
    
    pub fn get_user_organizations(&self) -> Vec<&Organization> {
        self.registry.user_memberships.keys()
            .filter_map(|id| self.registry.organizations.get(id))
            .collect()
    }

    pub fn get_organization_members(&self, org_id: &str) -> Vec<&OrganizationMember> {
        // This would typically fetch from a members collection
        // For now returning empty vec as placeholder
        vec![]
    }

    pub fn can_invite_members(&self) -> bool {
        self.has_permission(&Permission::CreateInvitations)
    }

    pub fn can_manage_subscription(&self) -> bool {
        self.has_permission(&Permission::ManageSubscription)
    }
}

impl OrganizationRegistry {
    pub fn add_organization(&mut self, organization: Organization) {
        let org_id = organization.id.clone();
        self.organizations.insert(org_id.clone(), organization);
    }

    pub fn get_organization(&self, org_id: &str) -> Option<&Organization> {
        self.organizations.get(org_id)
    }

    pub fn update_organization(&mut self, org_id: &str, updates: OrganizationUpdate) {
        if let Some(org) = self.organizations.get_mut(org_id) {
            if let Some(name) = updates.name {
                org.name = name;
            }
            if let Some(settings) = updates.settings {
                org.settings = settings;
            }
            org.updated_at = std::time::SystemTime::now();
        }
    }

    pub fn add_member(&mut self, org_id: String, membership: UserMembership) {
        self.user_memberships.insert(org_id.clone(), membership);
        
        // Update member count
        if let Some(org) = self.organizations.get_mut(&org_id) {
            org.member_count += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrganizationUpdate {
    pub name: Option<String>,
    pub settings: Option<OrganizationSettings>,
    pub brand_colors: Option<BrandColors>,
}
```

### 5. Organization Context Resource
**File**: `core/src/organization/resources/context.rs` (60 lines)

```rust
use bevy::prelude::*;

#[derive(Resource, Clone, Debug, Default)]
pub struct OrganizationContext {
    pub current_org: Option<String>,
    pub current_role: Option<crate::organization::models::OrganizationRole>,
    pub available_orgs: Vec<String>,
    pub switching_in_progress: bool,
    pub last_switch_time: Option<std::time::SystemTime>,
}

impl OrganizationContext {
    pub fn switch_to_organization(&mut self, org_id: String) {
        self.switching_in_progress = true;
        self.current_org = Some(org_id);
        self.last_switch_time = Some(std::time::SystemTime::now());
    }

    pub fn complete_switch(&mut self, role: crate::organization::models::OrganizationRole) {
        self.switching_in_progress = false;
        self.current_role = Some(role);
    }

    pub fn has_current_organization(&self) -> bool {
        self.current_org.is_some() && !self.switching_in_progress
    }

    pub fn get_current_org_id(&self) -> Option<&String> {
        if self.switching_in_progress {
            None
        } else {
            self.current_org.as_ref()
        }
    }

    pub fn is_switching(&self) -> bool {
        self.switching_in_progress
    }

    pub fn add_available_organization(&mut self, org_id: String) {
        if !self.available_orgs.contains(&org_id) {
            self.available_orgs.push(org_id);
        }
    }

    pub fn remove_available_organization(&mut self, org_id: &str) {
        self.available_orgs.retain(|id| id != org_id);
        
        // If removing current organization, clear it
        if self.current_org.as_ref() == Some(&org_id.to_string()) {
            self.current_org = None;
            self.current_role = None;
        }
    }

    pub fn get_organization_count(&self) -> usize {
        self.available_orgs.len()
    }
}
```

### 6. Event Handler Systems
**File**: `core/src/organization/systems/handler.rs` (180 lines)
**Reference**: `./docs/bevy/examples/ecs/event.rs:45-95`

```rust
use bevy::prelude::*;
use crate::organization::{events::*, resources::*, models::*};

pub fn handle_organization_events(
    mut commands: Commands,
    mut org_events: EventReader<OrganizationEvent>,
    mut membership_events: EventWriter<MembershipEvent>,
    mut org_registry: ResMut<OrganizationRegistry>,
    mut org_context: ResMut<OrganizationContext>,
    mut next_state: ResMut<NextState<super::OrganizationState>>,
) {
    for event in org_events.read() {
        match event {
            OrganizationEvent::SwitchOrganization { org_id } => {
                if org_registry.organizations.contains_key(org_id) {
                    info!("Switching to organization: {}", org_id);
                    org_context.switch_to_organization(org_id.clone());
                    next_state.set(super::OrganizationState::Switching);
                    commands.trigger(OrganizationSwitchEvent { 
                        from: org_context.current_org.clone(),
                        to: org_id.clone() 
                    });
                }
            },
            OrganizationEvent::CreateOrganization { name, initial_settings } => {
                let new_org = Organization::new(name.clone(), initial_settings.clone());
                let org_id = new_org.id.clone();
                
                info!("Creating new organization: {}", name);
                org_registry.add_organization(new_org);
                
                // Add creator as admin member
                let creator_membership = UserMembership {
                    org_id: org_id.clone(),
                    role: OrganizationRole::Admin,
                    joined_at: std::time::SystemTime::now(),
                    permissions: get_role_permissions(&OrganizationRole::Admin),
                };
                
                org_registry.add_member(org_id.clone(), creator_membership);
                org_context.add_available_organization(org_id.clone());
                commands.trigger(OrganizationCreatedEvent { org_id });
            },
            OrganizationEvent::UpdateOrganization { org_id, updates } => {
                info!("Updating organization: {}", org_id);
                org_registry.update_organization(org_id, updates.clone());
                commands.trigger(OrganizationUpdatedEvent { org_id: org_id.clone() });
            },
            OrganizationEvent::LeaveOrganization { org_id } => {
                info!("Leaving organization: {}", org_id);
                org_registry.user_memberships.remove(org_id);
                org_context.remove_available_organization(org_id);
                
                membership_events.send(MembershipEvent::MemberLeft { 
                    org_id: org_id.clone(), 
                    member_id: "current_user".to_string() // This would be actual user ID
                });
            },
            OrganizationEvent::InviteMember { org_id, email, role } => {
                let invitation = OrganizationInvitation {
                    id: uuid::Uuid::new_v4().to_string(),
                    org_id: org_id.clone(),
                    email: email.clone(),
                    role: role.clone(),
                    created_at: std::time::SystemTime::now(),
                    expires_at: std::time::SystemTime::now() + 
                        std::time::Duration::from_secs(7 * 24 * 3600), // 7 days
                    token: generate_secure_token(),
                };
                
                info!("Inviting member {} to organization {}", email, org_id);
                org_registry.pending_invitations.push(invitation.clone());
                membership_events.send(MembershipEvent::InvitationSent { 
                    org_id: org_id.clone(), 
                    invitation 
                });
            },
            OrganizationEvent::AcceptInvitation { invitation_id } => {
                if let Some(pos) = org_registry.pending_invitations
                    .iter().position(|inv| &inv.id == invitation_id) {
                    
                    let invitation = org_registry.pending_invitations.remove(pos);
                    info!("Accepting invitation: {}", invitation_id);
                    
                    let member = OrganizationMember::new(invitation.email.clone(), invitation.role.clone());
                    let membership = UserMembership {
                        org_id: invitation.org_id.clone(),
                        role: invitation.role.clone(),
                        joined_at: std::time::SystemTime::now(),
                        permissions: get_role_permissions(&invitation.role),
                    };
                    
                    org_registry.add_member(invitation.org_id.clone(), membership);
                    org_context.add_available_organization(invitation.org_id.clone());
                    
                    membership_events.send(MembershipEvent::InvitationAccepted {
                        org_id: invitation.org_id,
                        invitation_id: invitation.id,
                        member,
                    });
                }
            },
            OrganizationEvent::UpdateMemberRole { org_id, member_id, new_role } => {
                info!("Updating member role in organization {}: {} -> {:?}", 
                      org_id, member_id, new_role);
                
                if let Some(membership) = org_registry.user_memberships.get_mut(org_id) {
                    let old_role = membership.role.clone();
                    membership.role = new_role.clone();
                    membership.permissions = get_role_permissions(new_role);
                    
                    membership_events.send(MembershipEvent::RoleChanged {
                        org_id: org_id.clone(),
                        member_id: member_id.clone(),
                        old_role,
                        new_role: new_role.clone(),
                    });
                }
            },
            OrganizationEvent::RemoveMember { org_id, member_id } => {
                info!("Removing member from organization {}: {}", org_id, member_id);
                org_registry.user_memberships.remove(member_id);
                
                if let Some(org) = org_registry.organizations.get_mut(org_id) {
                    org.member_count = org.member_count.saturating_sub(1);
                }
                
                membership_events.send(MembershipEvent::MemberLeft {
                    org_id: org_id.clone(),
                    member_id: member_id.clone(),
                });
            },
        }
    }
}

pub fn update_organization_context(
    mut org_context: ResMut<OrganizationContext>,
    org_registry: Res<OrganizationRegistry>,
    mut next_state: ResMut<NextState<super::OrganizationState>>,
) {
    if org_context.is_switching() {
        if let Some(current_org_id) = &org_context.current_org {
            if let Some(membership) = org_registry.user_memberships.get(current_org_id) {
                org_context.complete_switch(membership.role.clone());
                next_state.set(super::OrganizationState::Ready);
                info!("Organization switch completed: {}", current_org_id);
            }
        }
    }
}

fn generate_secure_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[derive(Event, Debug)]
pub struct OrganizationSwitchEvent {
    pub from: Option<String>,
    pub to: String,
}

#[derive(Event, Debug)]
pub struct OrganizationCreatedEvent {
    pub org_id: String,
}

#[derive(Event, Debug)]
pub struct OrganizationUpdatedEvent {
    pub org_id: String,
}
```

## Implementation Summary

The organization management system is now properly decomposed into logical modules:

- **Plugin Module** (80 lines): Registration and initialization
- **Models** (3 files, <120 lines each): Data structures separated by concern
- **Resources** (3 files, <90 lines each): Bevy resources for state management
- **Events** (3 files, <85 lines each): Event definitions by domain
- **Systems** (4 files, <180 lines each): Business logic systems
- **UI Components** (4 files, <220 lines each): UI rendering and interactions
- **Utilities** (2 files, <90 lines each): Helper functions

Each module has a single responsibility and stays well under the 300-line limit. Integration points are clean and well-defined through Bevy's resource and event systems.

## Key Bevy Patterns Used

- **Plugin Architecture**: `./docs/bevy/examples/app/plugin.rs:15-53`
- **SystemParam**: `./docs/bevy/examples/ecs/system_param.rs:15-47` 
- **Event Handling**: `./docs/bevy/examples/ecs/event.rs:45-95`
- **State Management**: `./docs/bevy/examples/state/states.rs:25-95`
- **Observer System**: `./docs/bevy/examples/ecs/observers.rs:45-135`

All files maintain clean separation of concerns and follow Rust/Bevy best practices.
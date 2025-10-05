# Account_Menu Task 4: Organization Integration System

## Task Overview
Implement comprehensive team and organization membership management system, supporting role-based access, multi-organization membership, and seamless team collaboration features.

## Implementation Requirements

### Core Components
```rust
// Organization integration system
#[derive(Resource, Reflect, Debug)]
pub struct OrganizationIntegrationResource {
    pub current_organizations: Vec<OrganizationMembership>,
    pub active_organization: Option<OrganizationId>,
    pub pending_invitations: Vec<OrganizationInvitation>,
    pub organization_settings: HashMap<OrganizationId, OrganizationSettings>,
}

#[derive(Reflect, Debug, Clone)]
pub struct OrganizationMembership {
    pub organization_id: OrganizationId,
    pub organization_name: String,
    pub organization_logo: Option<String>,
    pub role: TeamRole,
    pub permissions: PermissionSet,
    pub joined_date: DateTime<Utc>,
    pub status: MembershipStatus,
    pub billing_seat: Option<BillingSeat>,
}

#[derive(Reflect, Debug, Clone)]
pub struct PermissionSet {
    pub can_manage_members: bool,
    pub can_manage_billing: bool,
    pub can_manage_extensions: bool,
    pub can_view_analytics: bool,
    pub can_modify_settings: bool,
    pub can_export_data: bool,
}

#[derive(Reflect, Debug, Clone)]
pub enum MembershipStatus {
    Active,
    Inactive,
    PendingActivation,
    Suspended,
    InvitationPending,
}

#[derive(Reflect, Debug, Clone)]
pub struct OrganizationInvitation {
    pub invitation_id: String,
    pub organization_name: String,
    pub inviter_name: String,
    pub invited_email: String,
    pub role: TeamRole,
    pub expiry_date: DateTime<Utc>,
    pub invitation_token: String,
}
```

### Organization Switching System
```rust
// Organization context switching
#[derive(Component, Reflect, Debug)]
pub struct OrganizationSwitcherComponent {
    pub dropdown_entity: Entity,
    pub current_org_display: Entity,
    pub organization_list: Vec<Entity>,
}

pub fn organization_switching_system(
    mut switcher_query: Query<&mut OrganizationSwitcherComponent>,
    mut organization_res: ResMut<OrganizationIntegrationResource>,
    mut switch_events: EventReader<OrganizationSwitchEvent>,
) {
    for switch_event in switch_events.read() {
        organization_res.active_organization = Some(switch_event.organization_id);
        // Update UI context with zero allocations
    }
}

#[derive(Event)]
pub struct OrganizationSwitchEvent {
    pub organization_id: OrganizationId,
}
```

### Team Collaboration Features  
```rust
// Team collaboration integration
#[derive(Reflect, Debug)]
pub struct TeamCollaborationFeatures {
    pub shared_shortcuts: bool,
    pub team_extensions: bool,
    pub shared_ai_models: bool,
    pub centralized_settings: bool,
    pub team_analytics: bool,
    pub audit_logging: bool,
}

impl TeamCollaborationFeatures {
    pub fn from_role_and_subscription(
        role: &TeamRole,
        subscription: &SubscriptionStatus,
    ) -> Self {
        match subscription {
            SubscriptionStatus::Team { .. } | SubscriptionStatus::Enterprise { .. } => {
                Self::enabled_features_for_role(role)
            }
            _ => Self::disabled(),
        }
    }
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `ecs/hierarchy.rs` - Organization hierarchy management  
- `ui/ui_stack.rs` - Organization switcher UI
- `input/keyboard_input.rs` - Organization switching hotkeys

### Implementation Pattern
```rust
// Based on hierarchy.rs for organization structure
fn organization_hierarchy_system(
    parent_query: Query<&Children, With<OrganizationComponent>>,
    member_query: Query<&TeamMember>,
) {
    for children in &parent_query {
        for &child in children.iter() {
            if let Ok(member) = member_query.get(child) {
                // Process team member within organization context
            }
        }
    }
}

// Based on ui_stack.rs for organization switcher
fn organization_switcher_ui_system(
    mut contexts: Local<Vec<UiContext>>,
    mut ui_stack: ResMut<UiStack>,
    organization_res: Res<OrganizationIntegrationResource>,
) {
    // Implement organization switcher UI stack
}
```

## Role-Based Access Control
- Granular permission system based on team roles
- Dynamic feature availability based on organization settings
- Secure role validation for sensitive operations
- Audit logging for organization-level actions

## Performance Constraints
- **ZERO ALLOCATIONS** during organization switching
- Efficient permission checking with cached results
- Lazy loading of organization settings
- Optimized organization list rendering

## Success Criteria
- Complete multi-organization support implementation
- Secure role-based access control system
- No unwrap()/expect() calls in production code
- Zero-allocation organization context switching
- Comprehensive team collaboration features

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for permission logic
- Integration tests for organization switching
- Security tests for role validation
- Performance tests for context switching efficiency
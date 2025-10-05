# Task 2: Member Management System

## Implementation Details

This task implements a comprehensive member management system for organizations, supporting role-based access control, invitation workflows, team hierarchies, and administrative oversight with enterprise-grade security and scalability.

### Architecture Overview

The system uses Bevy ECS with event-driven workflows, hierarchical role management, and real-time permission evaluation to provide seamless organizational member lifecycle management.

### Core Member Management Systems

#### Member Lifecycle Management
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::{HashMap, HashSet, VecDeque};

/// Member lifecycle management system
/// References: docs/bevy/examples/ecs/hierarchy.rs (hierarchical member relationships)
/// References: docs/bevy/examples/ecs/event.rs (member lifecycle events)
#[derive(Resource, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct MemberManagementSystem {
    /// Active member registry per organization
    pub member_registry: HashMap<Uuid, OrganizationMemberRegistry>,
    /// Pending invitation management
    pub invitation_manager: InvitationManager,
    /// Role hierarchy and permission cache
    pub role_hierarchy: RoleHierarchyManager,
    /// Member activity tracking
    pub activity_tracker: MemberActivityTracker,
    /// Onboarding and offboarding workflows
    pub lifecycle_workflows: MemberLifecycleWorkflows,
    /// Administrative oversight and audit
    pub admin_oversight: AdminOversightSystem,
}

/// Organization member registry with hierarchical structure
/// References: docs/bevy/examples/ecs/hierarchy.rs (parent-child member relationships)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct OrganizationMemberRegistry {
    /// Organization identifier
    pub organization_id: Uuid,
    /// All active members
    pub members: HashMap<Uuid, MemberProfile>,
    /// Team-based member organization
    pub team_structure: TeamStructure,
    /// Role assignments and hierarchy
    pub role_assignments: HashMap<Uuid, RoleAssignment>,
    /// Member permissions cache
    pub permission_cache: HashMap<Uuid, PermissionSet>,
    /// Member relationships and reporting structure
    pub reporting_structure: ReportingStructure,
}

/// Comprehensive member profile
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct MemberProfile {
    /// Member unique identifier
    pub user_id: Uuid,
    /// Organization context
    pub organization_id: Uuid,
    /// Member personal information
    pub profile_info: MemberProfileInfo,
    /// Current membership status
    pub membership_status: MembershipStatus,
    /// Role and permission information
    pub role_info: MemberRoleInfo,
    /// Team affiliations
    pub team_affiliations: Vec<TeamAffiliation>,
    /// Member preferences and settings
    pub preferences: MemberPreferences,
    /// Performance and activity metrics
    pub performance_metrics: MemberPerformanceMetrics,
}

/// Member personal and professional information
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct MemberProfileInfo {
    /// Display name
    pub display_name: String,
    /// Email address (primary identifier)
    pub email: String,
    /// Profile avatar URL
    pub avatar_url: Option<String>,
    /// Job title within organization
    pub job_title: Option<String>,
    /// Department or division
    pub department: Option<String>,
    /// Office location
    pub location: Option<String>,
    /// Contact information
    pub contact_info: ContactInformation,
    /// Member bio or description
    pub bio: Option<String>,
    /// Professional skills and expertise
    pub skills: Vec<String>,
    /// Time zone for scheduling
    pub timezone: String,
}

/// Role assignment with context and history
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RoleAssignment {
    /// Assignment unique identifier
    pub assignment_id: Uuid,
    /// Member receiving the role
    pub member_id: Uuid,
    /// Assigned role
    pub role: OrganizationRole,
    /// Assignment effective date
    pub effective_from: DateTime<Utc>,
    /// Role expiration (if temporary)
    pub expires_at: Option<DateTime<Utc>>,
    /// Who made the assignment
    pub assigned_by: Uuid,
    /// Assignment reason or context
    pub assignment_reason: String,
    /// Role scope and limitations
    pub role_scope: RoleScope,
    /// Assignment status
    pub status: RoleAssignmentStatus,
}

/// Team affiliation with role context
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TeamAffiliation {
    /// Team identifier
    pub team_id: Uuid,
    /// Team name for display
    pub team_name: String,
    /// Member's role in this team
    pub team_role: TeamRole,
    /// Join date
    pub joined_at: DateTime<Utc>,
    /// Team-specific permissions
    pub team_permissions: HashMap<String, PermissionLevel>,
    /// Active projects in this team
    pub active_projects: Vec<Uuid>,
    /// Team affiliation status
    pub status: TeamAffiliationStatus,
}
```

#### Invitation and Onboarding System
```rust
/// Comprehensive invitation management system
/// References: docs/bevy/examples/ecs/event.rs (invitation events and workflows)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct InvitationManager {
    /// Active invitations by organization
    pub active_invitations: HashMap<Uuid, Vec<MemberInvitation>>,
    /// Invitation templates and customization
    pub invitation_templates: HashMap<String, InvitationTemplate>,
    /// Bulk invitation processing
    pub bulk_invitations: VecDeque<BulkInvitationBatch>,
    /// Invitation analytics and tracking
    pub invitation_analytics: InvitationAnalytics,
    /// Security and validation rules
    pub security_rules: InvitationSecurityRules,
}

/// Detailed member invitation
/// References: docs/bevy/examples/ecs/event.rs (invitation lifecycle events)
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct MemberInvitation {
    /// Invitation unique identifier
    pub invitation_id: Uuid,
    /// Target organization
    pub organization_id: Uuid,
    /// Invitee email address
    pub invitee_email: String,
    /// Invitee name (if known)
    pub invitee_name: Option<String>,
    /// Who sent the invitation
    pub invited_by: Uuid,
    /// Proposed role for invitee
    pub proposed_role: OrganizationRole,
    /// Invitation message
    pub invitation_message: Option<String>,
    /// Security token for acceptance
    pub invitation_token: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Invitation status
    pub status: InvitationStatus,
    /// Acceptance/rejection details
    pub response_details: Option<InvitationResponse>,
    /// Security validation
    pub security_validation: SecurityValidation,
}

/// Onboarding workflow management
/// References: docs/bevy/examples/ecs/system_piping.rs (workflow state management)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct OnboardingWorkflow {
    /// Workflow unique identifier
    pub workflow_id: Uuid,
    /// New member being onboarded
    pub member_id: Uuid,
    /// Organization context
    pub organization_id: Uuid,
    /// Onboarding steps and progress
    pub steps: Vec<OnboardingStep>,
    /// Current workflow status
    pub status: WorkflowStatus,
    /// Assigned onboarding mentor
    pub mentor_id: Option<Uuid>,
    /// Workflow start time
    pub started_at: DateTime<Utc>,
    /// Expected completion time
    pub expected_completion: DateTime<Utc>,
    /// Actual completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Workflow customization
    pub customization: OnboardingCustomization,
}

/// Individual onboarding step
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct OnboardingStep {
    /// Step identifier
    pub step_id: String,
    /// Step title and description
    pub title: String,
    pub description: String,
    /// Step type and requirements
    pub step_type: OnboardingStepType,
    /// Completion requirements
    pub completion_criteria: CompletionCriteria,
    /// Step status
    pub status: StepStatus,
    /// Assigned responsible party
    pub assignee: Option<Uuid>,
    /// Step deadline
    pub deadline: Option<DateTime<Utc>>,
    /// Step completion details
    pub completion_details: Option<StepCompletionDetails>,
}
```

#### Role Hierarchy and Permission Management
```rust
/// Hierarchical role management system
/// References: docs/bevy/examples/ecs/hierarchy.rs (role inheritance patterns)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RoleHierarchyManager {
    /// Organization role hierarchies
    pub role_hierarchies: HashMap<Uuid, RoleHierarchy>,
    /// Permission inheritance rules
    pub inheritance_rules: PermissionInheritanceRules,
    /// Custom role definitions
    pub custom_roles: HashMap<Uuid, CustomRoleDefinition>,
    /// Role assignment validation
    pub assignment_validators: Vec<RoleAssignmentValidator>,
    /// Permission evaluation cache
    pub permission_cache: PermissionEvaluationCache,
}

/// Organization role hierarchy structure
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RoleHierarchy {
    /// Organization identifier
    pub organization_id: Uuid,
    /// Root roles (highest level)
    pub root_roles: Vec<OrganizationRole>,
    /// Role relationships and inheritance
    pub role_relationships: HashMap<String, RoleRelationship>,
    /// Permission inheritance matrix
    pub inheritance_matrix: PermissionInheritanceMatrix,
    /// Role constraints and limitations
    pub role_constraints: HashMap<String, RoleConstraints>,
}

/// Permission evaluation and caching system
/// References: docs/bevy/examples/ecs/change_detection.rs (permission cache invalidation)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PermissionEvaluationCache {
    /// Cached permission evaluations
    pub cached_evaluations: HashMap<PermissionCacheKey, CachedPermissionResult>,
    /// Cache invalidation tracking
    pub invalidation_tracker: CacheInvalidationTracker,
    /// Permission evaluation statistics
    pub evaluation_stats: PermissionEvaluationStats,
    /// Real-time permission updates
    pub real_time_updates: VecDeque<PermissionUpdate>,
}

/// Dynamic permission set for members
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PermissionSet {
    /// Direct permissions granted to member
    pub direct_permissions: HashMap<Permission, PermissionLevel>,
    /// Role-inherited permissions
    pub inherited_permissions: HashMap<Permission, PermissionLevel>,
    /// Team-specific permissions
    pub team_permissions: HashMap<Uuid, HashMap<Permission, PermissionLevel>>,
    /// Project-specific permissions
    pub project_permissions: HashMap<Uuid, HashMap<Permission, PermissionLevel>>,
    /// Effective permissions (computed)
    pub effective_permissions: HashMap<Permission, PermissionLevel>,
    /// Permission computation timestamp
    pub computed_at: DateTime<Utc>,
}
```

#### Team Structure and Management
```rust
/// Hierarchical team structure management
/// References: docs/bevy/examples/ecs/hierarchy.rs (nested team hierarchies)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TeamStructure {
    /// Organization identifier
    pub organization_id: Uuid,
    /// All teams in organization
    pub teams: HashMap<Uuid, OrganizationTeam>,
    /// Team hierarchy relationships
    pub team_hierarchy: TeamHierarchy,
    /// Cross-team collaboration matrix
    pub collaboration_matrix: CollaborationMatrix,
    /// Team performance metrics
    pub team_metrics: HashMap<Uuid, TeamPerformanceMetrics>,
}

/// Team hierarchy and relationships
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TeamHierarchy {
    /// Root teams (no parent)
    pub root_teams: Vec<Uuid>,
    /// Parent-child team relationships
    pub parent_child_relationships: HashMap<Uuid, Vec<Uuid>>,
    /// Team depth in hierarchy
    pub team_depth: HashMap<Uuid, u32>,
    /// Team member count hierarchy
    pub member_count_rollup: HashMap<Uuid, TeamMemberCount>,
}

/// Team role definitions specific to teams
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum TeamRole {
    /// Team leader with full team management
    TeamLeader,
    /// Senior member with mentoring responsibilities  
    SeniorMember,
    /// Regular team member
    Member,
    /// New team member in training
    Trainee,
    /// External collaborator (limited access)
    Collaborator {
        /// Specific collaboration permissions
        collaboration_scope: CollaborationScope,
        /// Collaboration expiration
        expires_at: Option<DateTime<Utc>>,
    },
    /// Custom team role
    Custom {
        /// Custom role name
        role_name: String,
        /// Team-specific permissions
        team_permissions: HashMap<String, PermissionLevel>,
    },
}

/// Team performance and activity metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct TeamPerformanceMetrics {
    /// Team member count over time
    pub member_count_history: Vec<MemberCountSnapshot>,
    /// Team productivity metrics
    pub productivity_metrics: ProductivityMetrics,
    /// Team collaboration scores
    pub collaboration_scores: CollaborationScores,
    /// Team retention rates
    pub retention_metrics: RetentionMetrics,
    /// Team goal achievement
    pub goal_achievement: GoalAchievementMetrics,
}
```

#### Member Activity and Performance Tracking
```rust
/// Comprehensive member activity tracking
/// References: docs/bevy/examples/ecs/change_detection.rs (activity change tracking)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct MemberActivityTracker {
    /// Real-time activity monitoring
    pub activity_monitors: HashMap<Uuid, ActivityMonitor>,
    /// Activity history and patterns
    pub activity_history: HashMap<Uuid, Vec<ActivityEvent>>,
    /// Performance metrics computation
    pub performance_calculator: PerformanceCalculator,
    /// Activity-based insights
    pub activity_insights: ActivityInsightsEngine,
    /// Automated activity reporting
    pub reporting_engine: ActivityReportingEngine,
}

/// Real-time member activity monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ActivityMonitor {
    /// Member being monitored
    pub member_id: Uuid,
    /// Current activity session
    pub current_session: Option<ActivitySession>,
    /// Daily activity summary
    pub daily_summary: DailyActivitySummary,
    /// Weekly activity trends
    pub weekly_trends: WeeklyActivityTrends,
    /// Activity anomaly detection
    pub anomaly_detection: ActivityAnomalyDetection,
}

/// Individual activity event
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ActivityEvent {
    /// Event unique identifier
    pub event_id: Uuid,
    /// Member who performed activity
    pub member_id: Uuid,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Activity type and details
    pub activity_type: ActivityType,
    /// Context information
    pub context: ActivityContext,
    /// Impact and metrics
    pub metrics: ActivityMetrics,
}

/// Different types of member activities
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum ActivityType {
    /// Login/logout events
    Authentication {
        action: AuthAction,
        device_info: DeviceInfo,
        location: Option<String>,
    },
    /// Permission and role changes
    PermissionChange {
        permission: Permission,
        old_level: Option<PermissionLevel>,
        new_level: PermissionLevel,
        changed_by: Uuid,
    },
    /// Team membership changes
    TeamMembership {
        team_id: Uuid,
        action: MembershipAction,
        role_change: Option<TeamRole>,
    },
    /// Project activities
    ProjectActivity {
        project_id: Uuid,
        activity_details: ProjectActivityDetails,
        impact_score: f32,
    },
    /// Extension usage
    ExtensionUsage {
        extension_id: Uuid,
        usage_type: ExtensionUsageType,
        duration_seconds: u64,
    },
    /// Administrative actions
    AdminAction {
        action_type: AdminActionType,
        target: AdminActionTarget,
        details: String,
    },
}
```

#### Administrative Oversight and Compliance
```rust
/// Administrative oversight and compliance system
/// References: docs/bevy/examples/ecs/event.rs (compliance event tracking)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct AdminOversightSystem {
    /// Administrative action tracking
    pub admin_actions: HashMap<Uuid, Vec<AdminAction>>,
    /// Compliance monitoring and reporting
    pub compliance_monitor: ComplianceMonitor,
    /// Audit trail management
    pub audit_trail: AuditTrailManager,
    /// Risk assessment and mitigation
    pub risk_assessment: RiskAssessmentEngine,
    /// Automated compliance alerts
    pub compliance_alerts: ComplianceAlertSystem,
}

/// Administrative action tracking
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct AdminAction {
    /// Action unique identifier
    pub action_id: Uuid,
    /// Administrator who performed action
    pub admin_id: Uuid,
    /// Target of administrative action
    pub target: AdminActionTarget,
    /// Action type and details
    pub action_type: AdminActionType,
    /// Action timestamp
    pub timestamp: DateTime<Utc>,
    /// Justification or reason
    pub justification: String,
    /// Action result and impact
    pub result: AdminActionResult,
    /// Approval workflow (if required)
    pub approval_workflow: Option<ApprovalWorkflow>,
}

/// Compliance monitoring system
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ComplianceMonitor {
    /// Active compliance frameworks
    pub active_frameworks: Vec<ComplianceFramework>,
    /// Compliance rule engine
    pub rule_engine: ComplianceRuleEngine,
    /// Violation tracking and resolution
    pub violation_tracker: ViolationTracker,
    /// Compliance reporting schedule
    pub reporting_schedule: ComplianceReportingSchedule,
    /// External compliance integrations
    pub external_integrations: Vec<ComplianceIntegration>,
}
```

### System Implementation

#### Member Lifecycle Systems
```rust
/// Member lifecycle management systems
/// References: docs/bevy/examples/ecs/system_piping.rs (workflow piping)

/// Member invitation processing system
pub fn process_member_invitations_system(
    mut invitation_manager: ResMut<InvitationManager>,
    mut invitation_events: EventReader<MemberInvitationEvent>,
    mut member_events: EventWriter<MemberLifecycleEvent>,
    time: Res<Time>,
) {
    let current_time = Utc::now();
    
    // Process new invitation events
    for event in invitation_events.read() {
        match event {
            MemberInvitationEvent::InvitationSent { invitation_id, .. } => {
                if let Some(invitations) = invitation_manager.active_invitations.get_mut(&event.organization_id()) {
                    if let Some(invitation) = invitations.iter_mut().find(|inv| inv.invitation_id == *invitation_id) {
                        invitation.status = InvitationStatus::Sent { sent_at: current_time };
                        
                        // Schedule expiration check
                        member_events.send(MemberLifecycleEvent::InvitationStatusChanged {
                            invitation_id: *invitation_id,
                            new_status: invitation.status.clone(),
                        });
                    }
                }
            },
            MemberInvitationEvent::InvitationAccepted { invitation_id, response } => {
                // Begin onboarding workflow
                let workflow = create_onboarding_workflow(invitation_id, response);
                member_events.send(MemberLifecycleEvent::OnboardingStarted {
                    workflow_id: workflow.workflow_id,
                    member_id: workflow.member_id,
                });
            },
            MemberInvitationEvent::InvitationDeclined { invitation_id, reason } => {
                // Update invitation status and analytics
                update_invitation_analytics(&mut invitation_manager, *invitation_id, reason);
            },
        }
    }
    
    // Check for expired invitations
    check_expired_invitations(&mut invitation_manager, &mut member_events, current_time);
}

/// Member onboarding workflow system
pub fn member_onboarding_system(
    mut lifecycle_workflows: ResMut<MemberLifecycleWorkflows>,
    mut workflow_events: EventReader<OnboardingWorkflowEvent>,
    mut member_events: EventWriter<MemberLifecycleEvent>,
    member_registry: Res<OrganizationMemberRegistry>,
) {
    for event in workflow_events.read() {
        match event {
            OnboardingWorkflowEvent::StepCompleted { workflow_id, step_id, completion_details } => {
                if let Some(workflow) = lifecycle_workflows.active_workflows.get_mut(workflow_id) {
                    if let Some(step) = workflow.steps.iter_mut().find(|s| s.step_id == *step_id) {
                        step.status = StepStatus::Completed;
                        step.completion_details = Some(completion_details.clone());
                        
                        // Check if workflow is complete
                        if workflow.steps.iter().all(|s| s.status == StepStatus::Completed) {
                            workflow.status = WorkflowStatus::Completed;
                            workflow.completed_at = Some(Utc::now());
                            
                            // Activate member account
                            member_events.send(MemberLifecycleEvent::MemberActivated {
                                member_id: workflow.member_id,
                                organization_id: workflow.organization_id,
                            });
                        }
                    }
                }
            },
            OnboardingWorkflowEvent::StepSkipped { workflow_id, step_id, reason } => {
                // Handle step skipping logic
                handle_step_skip(workflow_id, step_id, reason, &mut lifecycle_workflows);
            },
            OnboardingWorkflowEvent::WorkflowStalled { workflow_id, reason } => {
                // Handle stalled workflow recovery
                handle_workflow_stall(workflow_id, reason, &mut lifecycle_workflows, &mut member_events);
            },
        }
    }
}

/// Role assignment and permission system
/// References: docs/bevy/examples/ecs/change_detection.rs (permission change detection)
pub fn role_assignment_system(
    mut role_hierarchy: ResMut<RoleHierarchyManager>,
    mut role_events: EventReader<RoleAssignmentEvent>,
    mut permission_events: EventWriter<PermissionChangeEvent>,
    mut member_registry: ResMut<OrganizationMemberRegistry>,
) {
    for event in role_events.read() {
        match event {
            RoleAssignmentEvent::RoleAssigned { assignment } => {
                // Validate role assignment
                if validate_role_assignment(assignment, &role_hierarchy) {
                    // Apply role assignment
                    apply_role_assignment(assignment, &mut member_registry, &mut role_hierarchy);
                    
                    // Compute new permissions
                    let new_permissions = compute_member_permissions(
                        assignment.member_id,
                        &member_registry,
                        &role_hierarchy,
                    );
                    
                    // Update permission cache
                    update_permission_cache(
                        assignment.member_id,
                        new_permissions,
                        &mut role_hierarchy.permission_cache,
                    );
                    
                    // Send permission change event
                    permission_events.send(PermissionChangeEvent::PermissionsUpdated {
                        member_id: assignment.member_id,
                        organization_id: assignment.organization_id,
                        change_summary: create_permission_change_summary(assignment),
                    });
                } else {
                    // Handle invalid role assignment
                    handle_invalid_role_assignment(assignment, &mut role_events);
                }
            },
            RoleAssignmentEvent::RoleRevoked { member_id, role_id, revoked_by } => {
                // Process role revocation
                revoke_member_role(*member_id, *role_id, *revoked_by, &mut member_registry);
                
                // Recompute permissions
                recompute_member_permissions(*member_id, &member_registry, &mut role_hierarchy);
            },
        }
    }
}

/// Team management system
/// References: docs/bevy/examples/ecs/hierarchy.rs (team hierarchy management)
pub fn team_management_system(
    mut team_structure: ResMut<TeamStructure>,
    mut team_events: EventReader<TeamManagementEvent>,
    mut member_events: EventWriter<MemberLifecycleEvent>,
    member_registry: Res<OrganizationMemberRegistry>,
) {
    for event in team_events.read() {
        match event {
            TeamManagementEvent::MemberAddedToTeam { team_id, member_id, team_role } => {
                if let Some(team) = team_structure.teams.get_mut(team_id) {
                    // Add member to team
                    let affiliation = TeamAffiliation {
                        team_id: *team_id,
                        team_name: team.name.clone(),
                        team_role: team_role.clone(),
                        joined_at: Utc::now(),
                        team_permissions: compute_team_permissions(team_role, team),
                        active_projects: team.project_ids.iter().cloned().collect(),
                        status: TeamAffiliationStatus::Active,
                    };
                    
                    // Update member profile
                    update_member_team_affiliation(*member_id, affiliation, &member_registry);
                    
                    // Update team metrics
                    team.member_count += 1;
                    update_team_metrics(*team_id, &mut team_structure.team_metrics);
                    
                    member_events.send(MemberLifecycleEvent::TeamMembershipChanged {
                        member_id: *member_id,
                        team_id: *team_id,
                        change_type: TeamMembershipChangeType::Added,
                    });
                }
            },
            TeamManagementEvent::MemberRoleChangedInTeam { team_id, member_id, old_role, new_role } => {
                // Handle team role changes
                handle_team_role_change(*team_id, *member_id, old_role, new_role, &mut team_structure);
            },
            TeamManagementEvent::MemberRemovedFromTeam { team_id, member_id, removal_reason } => {
                // Handle team member removal
                handle_team_member_removal(*team_id, *member_id, removal_reason, &mut team_structure, &mut member_events);
            },
        }
    }
}
```

### Event System Integration

```rust
/// Member management events
/// References: docs/bevy/examples/ecs/event.rs (event-driven member management)

#[derive(Event, Debug, Clone)]
pub enum MemberLifecycleEvent {
    /// Member invitation sent
    InvitationSent {
        invitation_id: Uuid,
        organization_id: Uuid,
        invitee_email: String,
    },
    /// Member onboarding started
    OnboardingStarted {
        workflow_id: Uuid,
        member_id: Uuid,
    },
    /// Member account activated
    MemberActivated {
        member_id: Uuid,
        organization_id: Uuid,
    },
    /// Member role changed
    RoleChanged {
        member_id: Uuid,
        old_role: Option<OrganizationRole>,
        new_role: OrganizationRole,
        changed_by: Uuid,
    },
    /// Member team membership changed
    TeamMembershipChanged {
        member_id: Uuid,
        team_id: Uuid,
        change_type: TeamMembershipChangeType,
    },
    /// Member deactivated or removed
    MemberDeactivated {
        member_id: Uuid,
        organization_id: Uuid,
        reason: DeactivationReason,
        deactivated_by: Uuid,
    },
}

#[derive(Event, Debug, Clone)]
pub enum RoleAssignmentEvent {
    /// Role assigned to member
    RoleAssigned {
        assignment: RoleAssignment,
    },
    /// Role revoked from member
    RoleRevoked {
        member_id: Uuid,
        role_id: String,
        revoked_by: Uuid,
    },
    /// Custom role created
    CustomRoleCreated {
        role_definition: CustomRoleDefinition,
        created_by: Uuid,
    },
}

#[derive(Event, Debug, Clone)]
pub enum TeamManagementEvent {
    /// Member added to team
    MemberAddedToTeam {
        team_id: Uuid,
        member_id: Uuid,
        team_role: TeamRole,
    },
    /// Member role changed in team
    MemberRoleChangedInTeam {
        team_id: Uuid,
        member_id: Uuid,
        old_role: TeamRole,
        new_role: TeamRole,
    },
    /// Member removed from team
    MemberRemovedFromTeam {
        team_id: Uuid,
        member_id: Uuid,
        removal_reason: String,
    },
}
```

### Plugin Registration

```rust
/// Member management plugin for Bevy integration
/// References: docs/bevy/examples/app/* (plugin registration patterns)
pub struct MemberManagementPlugin;

impl Plugin for MemberManagementPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MemberManagementSystem>()
            .add_event::<MemberLifecycleEvent>()
            .add_event::<RoleAssignmentEvent>()
            .add_event::<TeamManagementEvent>()
            .add_event::<PermissionChangeEvent>()
            .add_systems(
                Update,
                (
                    process_member_invitations_system,
                    member_onboarding_system,
                    role_assignment_system,
                    team_management_system,
                    member_activity_tracking_system,
                    admin_oversight_system,
                ).chain()
            );
    }
}
```

### Implementation Requirements

1. **Hierarchical Role System**: Support complex organizational hierarchies with inherited permissions
2. **Real-time Permission Evaluation**: Dynamic permission computation with intelligent caching
3. **Comprehensive Activity Tracking**: Track all member activities for compliance and insights
4. **Scalable Team Management**: Support large teams with nested hierarchies and cross-team collaboration
5. **Enterprise-grade Security**: Secure invitation workflows with multi-factor validation
6. **Automated Workflows**: Streamlined onboarding and offboarding with customizable workflows
7. **Compliance Integration**: Built-in support for regulatory compliance and audit requirements
8. **Performance Optimization**: Efficient member lookup and permission evaluation for large organizations

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

**Implementation References:**
- `docs/bevy/examples/ecs/hierarchy.rs:1-150` - Hierarchical member and team relationships
- `docs/bevy/examples/ecs/event.rs:1-144` - Event-driven member lifecycle management
- `docs/bevy/examples/ecs/system_piping.rs:1-77` - Workflow state management and piping
- `docs/bevy/examples/ecs/change_detection.rs:1-106` - Permission and activity change detection
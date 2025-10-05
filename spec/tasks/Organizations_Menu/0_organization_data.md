# Task 0: Organization Data Models

## Implementation Details

This task implements comprehensive data models for organizational management, supporting multi-tenant architecture, role-based access control, subscription management, and enterprise-grade security features.

### Architecture Overview

The system uses Bevy ECS with Reflect-based serialization, multi-tenant data isolation, and hierarchical permission structures to support complex organizational workflows and enterprise requirements.

### Core Organization Data Structures

#### Primary Organization Entity
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

/// Primary organization entity with comprehensive metadata
/// References: docs/bevy/examples/ecs/hierarchy.rs (hierarchical data structures)
/// References: docs/bevy/examples/asset/asset_loading.rs (asset management for logos)
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Component)]
pub struct Organization {
    /// Unique organization identifier
    pub id: Uuid,
    /// Display name (e.g., "Cyrup.ai")
    pub name: String,
    /// URL-safe slug for the organization
    pub slug: String,
    /// Organization description and purpose
    pub description: Option<String>,
    /// Organization website URL
    pub website: Option<String>,
    /// Primary domain for email verification
    pub domain: Option<String>,
    /// Organization creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
    /// Organization status and lifecycle
    pub status: OrganizationStatus,
    /// Organization settings and configuration
    pub settings: OrganizationSettings,
    /// Security and compliance configuration
    pub security_config: SecurityConfiguration,
}

/// Organization lifecycle status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum OrganizationStatus {
    /// Organization is being set up
    Pending,
    /// Fully active organization
    Active,
    /// Temporarily suspended
    Suspended { reason: String, since: DateTime<Utc> },
    /// Archived organization (read-only)
    Archived { archived_at: DateTime<Utc> },
    /// Organization marked for deletion
    Deleted { deleted_at: DateTime<Utc> },
}

/// Comprehensive organization settings
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct OrganizationSettings {
    /// Organization branding and visual identity
    pub branding: BrandingConfiguration,
    /// Feature toggles and capabilities
    pub features: FeatureConfiguration,
    /// Integration settings
    pub integrations: IntegrationConfiguration,
    /// Data retention and governance policies
    pub data_policies: DataGovernancePolicies,
    /// Notification and communication settings
    pub notifications: NotificationConfiguration,
}

/// Organization branding and visual identity
/// References: docs/bevy/examples/ui/ui_texture_atlas.rs (logo management)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BrandingConfiguration {
    /// Organization logo asset handle
    pub logo_asset: Option<String>,
    /// Primary brand color (hex)
    pub primary_color: String,
    /// Secondary brand color (hex)
    pub secondary_color: String,
    /// Organization favicon
    pub favicon_asset: Option<String>,
    /// Custom CSS for organization theming
    pub custom_css: Option<String>,
    /// Brand guidelines and usage rules
    pub brand_guidelines: Option<String>,
}

/// Security and compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SecurityConfiguration {
    /// Multi-factor authentication requirements
    pub mfa_required: bool,
    /// Password policy configuration
    pub password_policy: PasswordPolicy,
    /// IP address restrictions
    pub ip_restrictions: Vec<IpRestriction>,
    /// Session management configuration
    pub session_config: SessionConfiguration,
    /// Data encryption requirements
    pub encryption_config: EncryptionConfiguration,
    /// Audit logging configuration
    pub audit_config: AuditConfiguration,
}
```

#### Organization Membership System
```rust
/// Organization membership with hierarchical roles
/// References: docs/bevy/examples/ecs/hierarchy.rs (parent-child relationships)
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct OrganizationMembership {
    /// Membership unique identifier
    pub id: Uuid,
    /// Organization being joined
    pub organization_id: Uuid,
    /// User who is a member
    pub user_id: Uuid,
    /// Member's role in the organization
    pub role: OrganizationRole,
    /// Membership status
    pub status: MembershipStatus,
    /// When the user joined
    pub joined_at: DateTime<Utc>,
    /// Who invited this member
    pub invited_by: Option<Uuid>,
    /// Member-specific permissions (overrides)
    pub permission_overrides: HashMap<String, PermissionLevel>,
    /// Teams this member belongs to
    pub team_memberships: HashSet<Uuid>,
    /// Member activity tracking
    pub activity_metrics: MemberActivityMetrics,
}

/// Organization roles with hierarchical permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum OrganizationRole {
    /// Organization owner (highest privileges)
    Owner,
    /// Administrator (full management access)
    Admin,
    /// Manager (team and project management)
    Manager {
        /// Teams this manager oversees
        managed_teams: HashSet<Uuid>,
        /// Projects this manager oversees
        managed_projects: HashSet<Uuid>,
    },
    /// Developer (development access)
    Developer {
        /// Development environment access level
        dev_access_level: DeveloperAccessLevel,
    },
    /// Member (standard user access)
    Member,
    /// Guest (limited read-only access)
    Guest {
        /// Expiration date for guest access
        expires_at: Option<DateTime<Utc>>,
    },
    /// Custom role with specific permissions
    Custom {
        /// Custom role name
        role_name: String,
        /// Specific permissions for this role
        permissions: HashSet<Permission>,
    },
}

/// Membership status tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum MembershipStatus {
    /// Invitation sent, awaiting acceptance
    Invited { invited_at: DateTime<Utc> },
    /// Membership is active
    Active,
    /// Temporarily deactivated
    Deactivated { reason: String, since: DateTime<Utc> },
    /// Member has left the organization
    Left { left_at: DateTime<Utc> },
    /// Member was removed from organization
    Removed { removed_at: DateTime<Utc>, removed_by: Uuid },
}

/// Comprehensive permission system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum Permission {
    // Organization management
    ManageOrganization,
    ViewOrganizationSettings,
    UpdateOrganizationProfile,
    ManageOrganizationMembers,
    ManageOrganizationRoles,
    DeleteOrganization,
    
    // Billing and subscriptions
    ViewBilling,
    ManageBilling,
    ViewSubscription,
    ManageSubscription,
    ViewUsage,
    
    // Extension store management
    ViewExtensionStore,
    ManageExtensionStore,
    ApproveExtensions,
    PublishExtensions,
    ManageExtensionSecurity,
    
    // Team management
    CreateTeams,
    ManageTeams,
    ViewTeamMembers,
    ManageTeamMembers,
    
    // Development and API access
    ApiAccess,
    DeveloperTools,
    ManageIntegrations,
    ViewAuditLogs,
    ManageWebhooks,
    
    // Data and security
    ManageDataPolicies,
    ViewSecuritySettings,
    ManageSecuritySettings,
    AccessComplianceReports,
    ManageAuditConfiguration,
}

/// Permission levels for fine-grained access control
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Reflect)]
pub enum PermissionLevel {
    /// No access
    None,
    /// Read-only access
    Read,
    /// Write access (create/update)
    Write,
    /// Full access (create/update/delete)
    Full,
    /// Administrative access (includes user management)
    Admin,
}
```

#### Team and Project Management
```rust
/// Team structure within organizations
/// References: docs/bevy/examples/ecs/hierarchy.rs (nested team structures)
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct OrganizationTeam {
    /// Team unique identifier
    pub id: Uuid,
    /// Parent organization
    pub organization_id: Uuid,
    /// Team name and identity
    pub name: String,
    /// Team description and purpose
    pub description: Option<String>,
    /// Parent team (for hierarchical teams)
    pub parent_team_id: Option<Uuid>,
    /// Team creation timestamp
    pub created_at: DateTime<Utc>,
    /// Team settings and configuration
    pub settings: TeamSettings,
    /// Team member count
    pub member_count: usize,
    /// Team project associations
    pub project_ids: HashSet<Uuid>,
}

/// Team-specific settings and configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TeamSettings {
    /// Team visibility (public/private)
    pub visibility: TeamVisibility,
    /// Team joining policy
    pub join_policy: TeamJoinPolicy,
    /// Team permissions and access control
    pub default_permissions: HashMap<Permission, PermissionLevel>,
    /// Team notification settings
    pub notification_settings: TeamNotificationSettings,
    /// Team resource limits
    pub resource_limits: TeamResourceLimits,
}

/// Team visibility levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum TeamVisibility {
    /// Visible to all organization members
    Public,
    /// Visible only to team members
    Private,
    /// Visible only to team and parent team members
    Restricted,
}

/// Project management within organizations
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]  
pub struct OrganizationProject {
    /// Project unique identifier
    pub id: Uuid,
    /// Parent organization
    pub organization_id: Uuid,
    /// Associated team (optional)
    pub team_id: Option<Uuid>,
    /// Project name
    pub name: String,
    /// Project description
    pub description: Option<String>,
    /// Project status
    pub status: ProjectStatus,
    /// Project creation timestamp
    pub created_at: DateTime<Utc>,
    /// Project settings
    pub settings: ProjectSettings,
    /// Project member roles
    pub member_roles: HashMap<Uuid, ProjectRole>,
}
```

#### Subscription and Billing System
```rust
/// Organization subscription management
/// References: docs/bevy/examples/time/* (subscription timing and billing cycles)
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct OrganizationSubscription {
    /// Subscription unique identifier
    pub id: Uuid,
    /// Associated organization
    pub organization_id: Uuid,
    /// Current subscription plan
    pub plan: SubscriptionPlan,
    /// Subscription status
    pub status: SubscriptionStatus,
    /// Billing cycle configuration
    pub billing_cycle: BillingCycle,
    /// Current period details
    pub current_period: BillingPeriod,
    /// Seat management
    pub seats: SeatManagement,
    /// Usage tracking
    pub usage_metrics: UsageMetrics,
    /// Payment and billing information
    pub payment_info: PaymentInformation,
}

/// Available subscription plans
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum SubscriptionPlan {
    /// Free tier with basic features
    Free {
        /// Member limit for free plan
        member_limit: usize,
    },
    /// Professional plan for small teams
    Professional {
        /// Base cost per member
        cost_per_member: f64,
        /// Minimum seats required
        minimum_seats: usize,
        /// Included features
        features: HashSet<String>,
    },
    /// Enterprise plan for large organizations
    Enterprise {
        /// Custom pricing structure
        custom_pricing: EnterprisePricing,
        /// Enterprise features and SLAs
        enterprise_features: EnterpriseFeatures,
        /// Dedicated support tier
        support_tier: SupportTier,
    },
    /// Custom negotiated plan
    Custom {
        /// Contract details
        contract_details: ContractDetails,
    },
}

/// Subscription status tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum SubscriptionStatus {
    /// Active subscription
    Active,
    /// Trial period
    Trial { ends_at: DateTime<Utc> },
    /// Payment past due
    PastDue { due_since: DateTime<Utc> },
    /// Subscription cancelled
    Cancelled { cancelled_at: DateTime<Utc> },
    /// Subscription suspended
    Suspended { reason: String, since: DateTime<Utc> },
}

/// Seat management for team billing
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SeatManagement {
    /// Total seats purchased
    pub total_seats: usize,
    /// Currently occupied seats
    pub occupied_seats: usize,
    /// Available seats
    pub available_seats: usize,
    /// Seat usage history
    pub usage_history: Vec<SeatUsageSnapshot>,
    /// Auto-scaling configuration
    pub auto_scaling: Option<SeatAutoScaling>,
}

/// Comprehensive usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct UsageMetrics {
    /// API calls made this billing period
    pub api_calls: u64,
    /// Storage used in bytes
    pub storage_bytes: u64,
    /// Bandwidth consumed in bytes
    pub bandwidth_bytes: u64,
    /// Extension installations
    pub extension_installations: u64,
    /// Feature usage statistics
    pub feature_usage: HashMap<String, u64>,
    /// Detailed usage breakdown by member
    pub member_usage: HashMap<Uuid, MemberUsageMetrics>,
}
```

#### Extension Store Management
```rust
/// Organization extension store configuration
/// References: docs/bevy/examples/asset/* (extension asset management)
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct OrganizationExtensionStore {
    /// Store unique identifier
    pub id: Uuid,
    /// Parent organization
    pub organization_id: Uuid,
    /// Store configuration
    pub config: ExtensionStoreConfig,
    /// Approved extensions for this organization
    pub approved_extensions: HashMap<Uuid, ApprovedExtension>,
    /// Private extensions developed for this organization
    pub private_extensions: HashMap<Uuid, PrivateExtension>,
    /// Extension security policies
    pub security_policies: ExtensionSecurityPolicies,
    /// Store metrics and analytics
    pub metrics: ExtensionStoreMetrics,
}

/// Extension store configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ExtensionStoreConfig {
    /// Store visibility and access
    pub visibility: StoreVisibility,
    /// Extension approval process
    pub approval_process: ApprovalProcess,
    /// Security scanning requirements
    pub security_requirements: SecurityRequirements,
    /// Installation policies
    pub installation_policies: InstallationPolicies,
    /// Update and maintenance policies
    pub update_policies: UpdatePolicies,
}

/// Approved extension with organizational metadata
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ApprovedExtension {
    /// Base extension information
    pub extension_id: Uuid,
    /// Organization-specific approval data
    pub approval_data: ApprovalData,
    /// Installation permissions and restrictions
    pub permissions: ExtensionPermissions,
    /// Usage tracking within organization
    pub usage_stats: ExtensionUsageStats,
    /// Compliance and security validation
    pub compliance_status: ComplianceStatus,
}

/// Private extension developed for organization
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PrivateExtension {
    /// Extension unique identifier
    pub id: Uuid,
    /// Extension metadata
    pub metadata: ExtensionMetadata,
    /// Development and maintenance information
    pub development_info: DevelopmentInfo,
    /// Distribution and access control
    pub distribution: DistributionConfig,
    /// Security and compliance information
    pub security_info: ExtensionSecurityInfo,
}
```

#### Resource Management
```rust
/// Organization resource management and limits
/// References: docs/bevy/examples/ecs/system_param.rs (resource injection patterns)
#[derive(Resource, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct OrganizationManager {
    /// Currently active organizations for user
    pub active_organizations: HashMap<Uuid, Organization>,
    /// Currently selected organization context
    pub current_organization_id: Option<Uuid>,
    /// Organization membership cache
    pub membership_cache: HashMap<Uuid, OrganizationMembership>,
    /// Permission evaluation cache
    pub permission_cache: HashMap<(Uuid, String), PermissionLevel>,
    /// Organization resources and limits
    pub resource_limits: HashMap<Uuid, ResourceLimits>,
    /// Multi-tenant data isolation
    pub data_isolation: DataIsolationConfig,
}

/// Resource limits and quotas per organization
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ResourceLimits {
    /// Maximum number of members
    pub max_members: usize,
    /// Maximum number of teams
    pub max_teams: usize,
    /// Maximum number of projects
    pub max_projects: usize,
    /// Storage limit in bytes
    pub storage_limit_bytes: u64,
    /// API call limit per month
    pub api_call_limit: u64,
    /// Extension installation limit
    pub extension_limit: usize,
    /// Webhook endpoint limit
    pub webhook_limit: usize,
}

/// Multi-tenant data isolation configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct DataIsolationConfig {
    /// Database isolation strategy
    pub isolation_strategy: IsolationStrategy,
    /// Encryption keys per organization
    pub encryption_keys: HashMap<Uuid, EncryptionKeyInfo>,
    /// Data residency requirements
    pub data_residency: HashMap<Uuid, DataResidencyRequirements>,
    /// Compliance framework requirements
    pub compliance_frameworks: HashMap<Uuid, Vec<ComplianceFramework>>,
}
```

### Supporting Data Types

```rust
/// Additional supporting types for organizational management

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: Option<u32>,
    pub prevent_reuse_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct IpRestriction {
    pub ip_range: String,
    pub restriction_type: IpRestrictionType,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum IpRestrictionType {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SessionConfiguration {
    pub max_duration_hours: u32,
    pub idle_timeout_minutes: u32,
    pub concurrent_session_limit: Option<usize>,
    pub secure_cookies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct AuditConfiguration {
    pub retention_days: u32,
    pub log_level: AuditLogLevel,
    pub monitored_actions: HashSet<String>,
    pub real_time_alerts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum AuditLogLevel {
    Basic,
    Detailed,
    Comprehensive,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct MemberActivityMetrics {
    pub last_active_at: Option<DateTime<Utc>>,
    pub login_count: u64,
    pub api_calls_made: u64,
    pub extensions_installed: u32,
    pub data_transferred_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum DeveloperAccessLevel {
    ReadOnly,
    Standard,
    Advanced,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum TeamJoinPolicy {
    Open,
    RequestApproval,
    InviteOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum ProjectStatus {
    Planning,
    Active,
    OnHold,
    Completed,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum ProjectRole {
    Owner,
    Maintainer,
    Developer,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BillingPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub amount_due: f64,
    pub currency: String,
    pub status: BillingStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum BillingStatus {
    Pending,
    Paid,
    Overdue,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum IsolationStrategy {
    DatabasePerTenant,
    SchemaPerTenant,
    RowLevelSecurity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum ComplianceFramework {
    GDPR,
    CCPA,
    HIPAA,
    SOX,
    PCI_DSS,
    ISO27001,
}
```

### System Integration Components

```rust
/// System components for organization management
/// References: docs/bevy/examples/ecs/system_param.rs (system parameter patterns)

/// Organization context switching component
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct OrganizationContext {
    /// Current active organization
    pub current_org_id: Uuid,
    /// Available organizations for current user
    pub available_orgs: Vec<OrganizationSummary>,
    /// User's role in current organization
    pub current_role: OrganizationRole,
    /// Cached permissions for current context
    pub cached_permissions: HashMap<String, PermissionLevel>,
    /// Context switch timestamp
    pub switched_at: DateTime<Utc>,
}

/// Lightweight organization summary for UI display
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct OrganizationSummary {
    pub id: Uuid,
    pub name: String,
    pub logo_url: Option<String>,
    pub role: OrganizationRole,
    pub member_count: usize,
    pub subscription_status: SubscriptionStatus,
}

/// Organization invitation system
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct OrganizationInvitation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inviter_id: Uuid,
    pub invitee_email: String,
    pub role: OrganizationRole,
    pub invitation_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub status: InvitationStatus,
    pub custom_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
    Cancelled,
}
```

### Implementation Requirements

1. **Multi-Tenant Architecture**: Complete data isolation between organizations with secure context switching
2. **Role-Based Access Control**: Hierarchical permission system with fine-grained access control
3. **Scalable Data Models**: Efficient data structures supporting large-scale organizational hierarchies
4. **Security-First Design**: Comprehensive security controls with audit logging and compliance support
5. **Performance Optimization**: Optimized queries and caching for large organizational datasets
6. **Reflect Integration**: Full Bevy Reflect support for serialization and component inspection
7. **Type Safety**: Strong typing throughout with comprehensive error handling
8. **Extensibility**: Modular design supporting future organizational features

### Testing Strategy

1. **Unit Tests**: Individual component testing with comprehensive data model validation
2. **Integration Tests**: Multi-organization workflows and permission system testing
3. **Performance Tests**: Large-scale organization stress testing and benchmarking
4. **Security Tests**: Penetration testing and vulnerability assessment
5. **Compliance Tests**: Regulatory compliance validation and audit trail verification

### Security Considerations

1. **Data Isolation**: Cryptographic separation of organizational data
2. **Access Control**: Multi-layered authorization with principle of least privilege
3. **Audit Logging**: Comprehensive audit trails with tamper detection
4. **Key Management**: Secure encryption key management per organization
5. **Session Security**: Secure session handling with automatic timeout and invalidation

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

**Implementation References:**
- `docs/bevy/examples/ecs/hierarchy.rs:1-150` - Hierarchical data structures for teams and roles
- `docs/bevy/examples/asset/asset_loading.rs:1-200` - Asset management for organization logos  
- `docs/bevy/examples/ecs/system_param.rs:1-180` - Resource injection and system parameter patterns
- `docs/bevy/examples/ui/ui_texture_atlas.rs:1-120` - UI texture management for branding elements
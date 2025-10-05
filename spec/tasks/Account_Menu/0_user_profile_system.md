# Account_Menu Task 0: User Profile System Data Models

## Task Overview
Implement comprehensive user profile and authentication data structures for the Account menu, supporting Pro subscriptions, organization membership, and secure profile management.

## Implementation Requirements

### Core Data Models
```rust
// User profile system
#[derive(Component, Resource, Reflect, Debug, Clone)]
pub struct UserProfileResource {
    pub user_id: UserId,
    pub email: String,
    pub display_name: String,
    pub profile_image: Option<ProfileImage>,
    pub subscription: SubscriptionStatus,
    pub organization_memberships: Vec<OrganizationMembership>,
    pub preferences: UserPreferences,
    pub security_context: SecurityContext,
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

#[derive(Reflect, Debug, Clone)]
pub struct ProfileImage {
    pub url: String,
    pub local_cache_path: Option<PathBuf>,
    pub content_hash: String,
    pub upload_timestamp: DateTime<Utc>,
}

#[derive(Reflect, Debug, Clone)]
pub enum SubscriptionStatus {
    Free {
        trial_remaining_days: Option<u32>,
    },
    Pro {
        billing_period: BillingPeriod,
        renewal_date: DateTime<Utc>,
        payment_method: PaymentMethod,
    },
    Team {
        organization_id: OrganizationId,
        role: TeamRole,
        seat_allocation: u32,
    },
    Enterprise {
        organization_id: OrganizationId,
        custom_features: Vec<EnterpriseFeature>,
        contract_expiry: DateTime<Utc>,
    },
}
```

### Authentication System
```rust
// Authentication and session management
#[derive(Resource, Reflect)]
pub struct AuthenticationResource {
    pub session_token: Option<SecureToken>,
    pub refresh_token: Option<SecureToken>,
    pub token_expiry: Option<DateTime<Utc>>,
    pub login_state: LoginState,
    pub mfa_enabled: bool,
}

#[derive(Reflect, Debug)]
pub enum LoginState {
    LoggedOut,
    LoggingIn,
    LoggedIn {
        session_start: DateTime<Utc>,
        last_activity: DateTime<Utc>,
    },
    SessionExpired,
    AuthenticationError(String),
}

#[derive(Reflect, Debug)]
pub struct SecureToken {
    pub token: String, // Store encrypted token
    pub scope: TokenScope,
    pub issued_at: DateTime<Utc>,
}
```

### Organization Integration
```rust
// Organization membership data
#[derive(Reflect, Debug, Clone)]
pub struct OrganizationMembership {
    pub organization_id: OrganizationId,
    pub organization_name: String,
    pub role: TeamRole,
    pub permissions: Vec<Permission>,
    pub joined_date: DateTime<Utc>,
    pub status: MembershipStatus,
}

#[derive(Reflect, Debug, Clone)]
pub enum TeamRole {
    Member,
    Admin,
    Owner,
    Billing,
    Developer,
}

#[derive(Reflect, Debug, Clone)]
pub struct OrganizationId(pub Uuid);
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `reflection/reflection.rs` - Reflect trait implementation
- `ecs/component_change_detection.rs` - Change detection patterns
- `async_compute/async_compute.rs` - Async authentication operations

### Implementation Pattern
```rust
// Based on reflection.rs for serializable data structures
#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct UserProfileResource {
    // All fields implement Reflect for serialization
}

// Based on component_change_detection.rs
fn user_profile_change_system(
    mut profile_query: Query<&mut UserProfileResource, Changed<UserProfileResource>>,
    mut profile_events: EventWriter<ProfileUpdateEvent>,
) {
    for profile in profile_query.iter_mut() {
        profile_events.send(ProfileUpdateEvent::ProfileChanged);
    }
}
```

## Security Requirements
- Encrypted token storage using system keychain
- Secure session management with automatic expiry
- Profile image validation and secure caching
- Audit logging for authentication events

## Performance Constraints
- **ZERO ALLOCATIONS** during profile data access
- Efficient change detection for UI updates  
- Cached profile data to minimize API calls
- Lazy loading of organization memberships

## Success Criteria
- Complete user profile data model implementation
- Secure authentication state management
- No unwrap()/expect() calls in production code
- Zero-allocation profile data access
- Comprehensive organization membership support

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for data model validation
- Integration tests for profile persistence  
- Security tests for token handling
- Performance tests for zero-allocation access
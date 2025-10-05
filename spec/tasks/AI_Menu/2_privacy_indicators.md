# AI Menu - Privacy Indicators System

## Implementation Task: Privacy Status Bar with Interactive Icons

### Architecture Overview
Create a privacy status display system showing Full Control, No Collection, and Encrypted indicators with real-time status updates.

### Core Components

#### Privacy Status Component
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PrivacyIndicators {
    pub full_control: bool,
    pub no_collection: bool, 
    pub encrypted: bool,
    pub info_expanded: bool,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PrivacyIconButton {
    pub indicator_type: IndicatorType,
    pub hover_state: HoverState,
}

#[derive(Reflect)]
pub enum IndicatorType {
    FullControl,
    NoCollection, 
    Encrypted,
    InfoDetails,
}
```

#### Privacy Bar Layout System
- **Container**: Horizontal flexbox layout at top of left pane
- **Element Spacing**: Equal spacing between 4 icons (minus, lock, shield, info)
- **Background**: Dark theme background (#2a2a2a) with subtle border
- **Icon Size**: 16x16 pixels with appropriate padding
- **Interactive States**: Hover highlighting for info button

### Bevy Implementation References

#### UI Layout System
- **Container Layout**: `docs/bevy/examples/ui/flex_layout.rs`
  - Horizontal flexbox container for indicator icons
  - Justify content spacing between elements
  - Cross-axis center alignment for visual balance

#### Icon Rendering System  
- **Vector Icons**: `docs/bevy/examples/ui/ui_texture_atlas.rs`
  - Icon sprite atlas for minus, lock, shield, info symbols
  - State-based icon rendering (active/inactive states)
  - Efficient texture atlas usage for UI icons

#### Interactive Icon Buttons
- **Button Interactions**: `docs/bevy/examples/ui/button.rs` 
  - Hover state management for info button
  - Click handling for expandable info details
  - Visual feedback on button interaction

#### Status Indicator Updates
- **Change Detection**: `docs/bevy/examples/ecs/change_detection.rs`
  - Real-time privacy status updates from data model changes
  - Efficient change detection for indicator state updates
  - Conditional rendering based on privacy configuration

### Data Integration Points

#### Privacy Configuration Resource
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct PrivacyConfiguration {
    pub data_collection_enabled: bool,
    pub full_user_control: bool,
    pub encryption_active: bool,
    pub provider_privacy_level: PrivacyLevel,
}
```

#### Status Calculation System
- Monitor AI provider configuration changes
- Calculate privacy indicators based on current settings
- Update visual indicators in real-time via change detection
- Validate encryption status with active providers

### Visual Implementation Details

#### Icon Design Requirements
- **Minus Icon**: `─` symbol for Full Control indicator
- **Lock Icon**: `🔒` symbol for No Collection indicator  
- **Shield Icon**: `🛡️` symbol for Encrypted indicator
- **Info Icon**: `ℹ️` circular info symbol for additional details
- **Color Scheme**: Light gray (#888888) for inactive, white (#ffffff) for active

#### Container Styling
- **Background Color**: Dark theme (#2a2a2a)
- **Border**: 1px solid rgba(255,255,255,0.1) 
- **Padding**: 8px horizontal, 6px vertical
- **Corner Radius**: 4px for subtle rounded corners
- **Height**: Fixed height to prevent layout shift

### Interactive Behavior

#### Info Button Expansion
- **Trigger**: Click on info icon button
- **Animation**: Smooth dropdown expansion below privacy bar
- **Content**: Detailed explanations of each privacy indicator
- **Dismissal**: Click outside or second click to collapse

#### Real-time Status Updates
- **Provider Changes**: Update indicators when AI provider changes
- **Configuration Changes**: Reflect privacy setting modifications immediately
- **Network Status**: Show encryption status based on connection security

### Testing Requirements

#### Visual State Testing
- Verify correct icon display for each privacy state combination
- Test hover states and visual feedback on info button
- Validate container styling and spacing consistency

#### Functional Testing  
- Test real-time updates when privacy configurations change
- Verify info expansion/collapse functionality
- Test accessibility features (keyboard navigation, screen reader support)

### Performance Considerations

#### Efficient Updates
- Use change detection to minimize unnecessary re-renders
- Optimize icon sprite atlas loading and texture usage
- Cache calculated privacy states to avoid repeated computation
- Minimize layout recalculations on status changes

#### Memory Management
- Avoid string allocations in update loops
- Reuse UI components across privacy state changes
- Efficient sprite atlas usage for icon rendering
- Zero-allocation status calculation where possible

### Integration Points

#### AI Configuration Dependencies
- Monitor `AIConfiguration` resource for provider changes
- Listen for privacy setting modifications via events
- Update indicators based on active AI model capabilities
- Sync with cloud synchronization privacy settings

#### Event System Integration
```rust
#[derive(Event)]
pub struct PrivacyStatusChanged {
    pub full_control: bool,
    pub no_collection: bool,
    pub encrypted: bool,
}
```

### Implementation Files
- `ai_menu/privacy_indicators.rs` - Core privacy indicator components
- `ai_menu/privacy_systems.rs` - Update systems and change detection
- `ai_menu/privacy_events.rs` - Privacy-related event definitions
- `ui/privacy_icons.rs` - Icon sprite definitions and rendering

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all update loops
- **Blazing-fast performance** - efficient change detection only
- **Production quality** - complete, robust implementation## Bevy Implementation Details

### Privacy Component Architecture

```rust
#[derive(Component, Reflect, Clone)]
pub struct PrivacyIndicator {
    pub data_category: DataCategory,
    pub collection_status: CollectionStatus,
    pub encryption_level: EncryptionLevel,
    pub retention_policy: RetentionPolicy,
    pub user_control_level: UserControlLevel,
}

#[derive(Component, Reflect)]
pub struct PrivacySettings {
    pub global_collection_enabled: bool,
    pub encryption_required: bool,
    pub audit_logging_enabled: bool,
    pub data_export_allowed: bool,
    pub third_party_sharing: ThirdPartySharing,
}

#[derive(Component, Reflect)]
pub struct DataAuditTrail {
    pub entries: VecDeque<AuditEntry>,
    pub max_entries: usize,
    pub auto_purge_days: u32,
}

#[derive(Component, Reflect)]
pub struct PrivacyUIState {
    pub indicators_visible: bool,
    pub detailed_view_open: bool,
    pub selected_category: Option<DataCategory>,
    pub user_permissions_dialog: bool,
}
```

### Privacy Resource Management

```rust
#[derive(Resource, Reflect)]
pub struct GlobalPrivacyState {
    pub privacy_level: PrivacyLevel,
    pub data_categories: HashMap<DataCategory, Entity>,
    pub consent_timestamps: HashMap<DataCategory, SystemTime>,
    pub last_audit: Option<SystemTime>,
}

#[derive(Resource)]
pub struct PrivacyValidator {
    pub encryption_service: Arc<dyn EncryptionService>,
    pub audit_service: Arc<dyn AuditService>,
    pub compliance_checker: Arc<dyn ComplianceChecker>,
}
```

### Privacy Event System

```rust
#[derive(Event, Debug)]
pub enum PrivacyEvent {
    ConsentUpdated(DataCategory, bool),
    DataCollected(DataCategory, String),
    DataDeleted(DataCategory, String),
    AuditRequested,
    ComplianceCheck(DataCategory),
    PrivacyViolation(String, Severity),
    UserPermissionChanged(DataCategory, UserControlLevel),
}

#[derive(Event, Debug)]
pub enum PrivacyValidationEvent {
    ValidateDataCollection(Entity),
    ValidateEncryption(Entity),
    ValidateRetention(Entity),
    ComplianceValidated(Entity, bool, Vec<String>),
}
```

### Security System Implementation

```rust
fn monitor_privacy_compliance(
    mut privacy_events: EventReader<PrivacyEvent>,
    mut validation_events: EventWriter<PrivacyValidationEvent>,
    privacy_indicators: Query<(Entity, &PrivacyIndicator)>,
    global_state: ResMut<GlobalPrivacyState>,
) {
    for event in privacy_events.read() {
        match event {
            PrivacyEvent::DataCollected(category, _) => {
                for (entity, indicator) in &privacy_indicators {
                    if indicator.data_category == *category {
                        validation_events.send(PrivacyValidationEvent::ValidateDataCollection(entity));
                    }
                }
            },
            PrivacyEvent::PrivacyViolation(msg, severity) => {
                error!("Privacy violation detected: {} (severity: {:?})", msg, severity);
                // Trigger immediate audit
                validation_events.send(PrivacyValidationEvent::ValidateEncryption(Entity::PLACEHOLDER));
            },
            _ => {}
        }
    }
}
```

### Flex-Based Privacy UI Layout

```rust
fn setup_privacy_indicators_ui(
    mut commands: Commands,
    privacy_settings: Res<PrivacySettings>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(400.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            flex_grow: 0.0, // Prevent expansion
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        PrivacyIndicatorUI,
    )).with_children(|parent| {
        // Privacy status indicators with proper flex constraints
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
        ));
    });
}
```
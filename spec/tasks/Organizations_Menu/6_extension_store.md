# Task 6: Organization Extension Store System

## Implementation Details

This task implements a comprehensive organization-specific extension store system, supporting curated marketplaces, private extension distribution, security scanning, and enterprise compliance management for organizational extension ecosystems.

### Architecture Overview

The system uses Bevy ECS with asset management for extensions, event-driven approval workflows, and hierarchical permission systems to provide enterprise-grade extension marketplace management with security-first design.

### Core Extension Store Data Structures

#### Organization Extension Store Management
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::{HashMap, HashSet, BTreeMap};
use semver::Version;

/// Comprehensive organization extension store system
/// References: docs/bevy/examples/asset/* (extension asset management)
/// References: docs/bevy/examples/ecs/hierarchy.rs (extension category hierarchies)
#[derive(Resource, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct OrganizationExtensionStoreSystem {
    /// Extension stores by organization
    pub organization_stores: HashMap<Uuid, OrganizationExtensionStore>,
    /// Global extension registry
    pub global_extension_registry: GlobalExtensionRegistry,
    /// Security scanning system
    pub security_scanner: ExtensionSecurityScanner,
    /// Approval workflow engine
    pub approval_engine: ExtensionApprovalEngine,
    /// Distribution management
    pub distribution_manager: ExtensionDistributionManager,
    /// Analytics and metrics
    pub analytics_engine: ExtensionAnalyticsEngine,
}

/// Organization-specific extension store
/// References: docs/bevy/examples/asset/asset_loading.rs (extension loading)
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct OrganizationExtensionStore {
    /// Store unique identifier
    pub store_id: Uuid,
    /// Parent organization
    pub organization_id: Uuid,
    /// Store configuration and policies
    pub store_config: StoreConfiguration,
    /// Approved public extensions
    pub approved_extensions: HashMap<Uuid, ApprovedExtensionEntry>,
    /// Private/internal extensions
    pub private_extensions: HashMap<Uuid, PrivateExtensionEntry>,
    /// Extension categories and organization
    pub category_structure: ExtensionCategoryStructure,
    /// Installation and usage tracking
    pub usage_tracker: ExtensionUsageTracker,
    /// Store customization and branding
    pub store_customization: StoreCustomization,
    /// Compliance and audit information
    pub compliance_info: StoreComplianceInfo,
}

/// Store configuration and policies
/// References: docs/bevy/examples/ecs/hierarchy.rs (policy hierarchies)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct StoreConfiguration {
    /// Store visibility settings
    pub visibility: StoreVisibility,
    /// Extension approval workflow
    pub approval_workflow: ApprovalWorkflowConfig,
    /// Security policy requirements
    pub security_policies: SecurityPolicyConfig,
    /// Installation policies
    pub installation_policies: InstallationPolicyConfig,
    /// Update and maintenance policies
    pub update_policies: UpdatePolicyConfig,
    /// Content moderation settings
    pub moderation_config: ModerationConfig,
    /// Developer program settings
    pub developer_program: DeveloperProgramConfig,
}

/// Extension approval workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ApprovalWorkflowConfig {
    /// Enable automatic approval for trusted publishers
    pub auto_approval_enabled: bool,
    /// Trusted publisher list
    pub trusted_publishers: HashSet<Uuid>,
    /// Required approval stages
    pub approval_stages: Vec<ApprovalStage>,
    /// Security scanning requirements
    pub security_scan_requirements: SecurityScanRequirements,
    /// Manual review criteria
    pub manual_review_criteria: ManualReviewCriteria,
    /// Approval SLA timeframes
    pub approval_sla: ApprovalSLA,
}

/// Individual extension approval stage
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ApprovalStage {
    /// Stage identifier
    pub stage_id: String,
    /// Stage name and description
    pub stage_name: String,
    pub stage_description: String,
    /// Required approvers
    pub required_approvers: Vec<ApproverRequirement>,
    /// Stage completion criteria
    pub completion_criteria: StageCompletionCriteria,
    /// Stage timeout
    pub timeout_hours: Option<u32>,
    /// Auto-progression rules
    pub auto_progression: Option<AutoProgressionRules>,
}
```

#### Extension Registry and Metadata
```rust
/// Global extension registry with comprehensive metadata
/// References: docs/bevy/examples/asset/asset_loading.rs (extension metadata)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct GlobalExtensionRegistry {
    /// All registered extensions
    pub extensions: HashMap<Uuid, ExtensionRegistryEntry>,
    /// Publisher information
    pub publishers: HashMap<Uuid, ExtensionPublisher>,
    /// Extension categories and taxonomy
    pub category_taxonomy: ExtensionCategoryTaxonomy,
    /// Version management
    pub version_manager: ExtensionVersionManager,
    /// Dependency resolution
    pub dependency_resolver: ExtensionDependencyResolver,
}

/// Comprehensive extension registry entry
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ExtensionRegistryEntry {
    /// Extension unique identifier
    pub extension_id: Uuid,
    /// Extension metadata
    pub metadata: ExtensionMetadata,
    /// Publisher information
    pub publisher: ExtensionPublisher,
    /// All available versions
    pub versions: BTreeMap<Version, ExtensionVersion>,
    /// Latest stable version
    pub latest_stable: Version,
    /// Beta/preview versions
    pub beta_versions: Vec<Version>,
    /// Extension statistics
    pub statistics: ExtensionStatistics,
    /// Security assessment
    pub security_assessment: SecurityAssessment,
    /// Compliance status
    pub compliance_status: ComplianceStatus,
}

/// Detailed extension metadata
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ExtensionMetadata {
    /// Extension name
    pub name: String,
    /// Extension slug (URL-safe identifier)
    pub slug: String,
    /// Extension description
    pub description: String,
    /// Detailed description/README
    pub long_description: Option<String>,
    /// Extension category
    pub category: ExtensionCategory,
    /// Subcategories/tags
    pub tags: Vec<String>,
    /// Extension icon/logo
    pub icon_url: Option<String>,
    /// Screenshots and media
    pub media: Vec<ExtensionMedia>,
    /// Extension website/homepage
    pub homepage: Option<String>,
    /// Documentation URL
    pub documentation_url: Option<String>,
    /// Source code repository
    pub repository_url: Option<String>,
    /// Issue tracker URL
    pub issues_url: Option<String>,
    /// License information
    pub license: LicenseInfo,
    /// Supported platforms
    pub supported_platforms: Vec<Platform>,
    /// Minimum system requirements
    pub system_requirements: SystemRequirements,
}

/// Extension version information
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ExtensionVersion {
    /// Version number
    pub version: Version,
    /// Version release date
    pub release_date: DateTime<Utc>,
    /// Version status
    pub status: VersionStatus,
    /// Release notes/changelog
    pub release_notes: String,
    /// Extension package/binary
    pub package: ExtensionPackage,
    /// Dependencies for this version
    pub dependencies: Vec<ExtensionDependency>,
    /// API compatibility
    pub api_compatibility: ApiCompatibility,
    /// Security scan results
    pub security_scan: Option<SecurityScanResult>,
    /// Download statistics
    pub download_stats: DownloadStatistics,
}

/// Extension package information
/// References: docs/bevy/examples/asset/asset_loading.rs (package loading)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ExtensionPackage {
    /// Package format (WASM, native binary, etc.)
    pub package_format: PackageFormat,
    /// Package download URL
    pub download_url: String,
    /// Package size in bytes
    pub size_bytes: u64,
    /// Package checksum/hash
    pub checksum: PackageChecksum,
    /// Package signature for verification
    pub signature: Option<PackageSignature>,
    /// Installation scripts
    pub installation_scripts: Vec<InstallationScript>,
    /// Uninstallation scripts
    pub uninstallation_scripts: Vec<UninstallationScript>,
}
```

#### Extension Security and Compliance
```rust
/// Comprehensive extension security scanning system
/// References: docs/bevy/examples/ecs/event.rs (security scan events)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ExtensionSecurityScanner {
    /// Active security scanners
    pub active_scanners: Vec<SecurityScanner>,
    /// Scan policies and configuration
    pub scan_policies: SecurityScanPolicies,
    /// Vulnerability database
    pub vulnerability_database: VulnerabilityDatabase,
    /// Security scan history
    pub scan_history: HashMap<Uuid, Vec<SecurityScanResult>>,
    /// Real-time threat monitoring
    pub threat_monitor: ThreatMonitor,
}

/// Individual security scanner
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SecurityScanner {
    /// Scanner identifier
    pub scanner_id: String,
    /// Scanner name and version
    pub scanner_name: String,
    pub scanner_version: Version,
    /// Scanner capabilities
    pub capabilities: SecurityScanCapabilities,
    /// Scanner configuration
    pub configuration: SecurityScannerConfig,
    /// Scanner status
    pub status: ScannerStatus,
}

/// Comprehensive security scan result
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SecurityScanResult {
    /// Scan unique identifier
    pub scan_id: Uuid,
    /// Extension being scanned
    pub extension_id: Uuid,
    /// Extension version scanned
    pub extension_version: Version,
    /// Scan timestamp
    pub scanned_at: DateTime<Utc>,
    /// Scanner used
    pub scanner_id: String,
    /// Overall security score
    pub security_score: f32,
    /// Identified vulnerabilities
    pub vulnerabilities: Vec<SecurityVulnerability>,
    /// Security warnings
    pub warnings: Vec<SecurityWarning>,
    /// Compliance check results
    pub compliance_checks: Vec<ComplianceCheckResult>,
    /// Scan metadata
    pub scan_metadata: ScanMetadata,
}

/// Individual security vulnerability
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SecurityVulnerability {
    /// Vulnerability unique identifier
    pub vulnerability_id: String,
    /// Vulnerability severity
    pub severity: VulnerabilitySeverity,
    /// Vulnerability type/category
    pub vulnerability_type: VulnerabilityType,
    /// Vulnerability description
    pub description: String,
    /// Affected code/components
    pub affected_components: Vec<String>,
    /// CVE identifier (if applicable)
    pub cve_id: Option<String>,
    /// CVSS score
    pub cvss_score: Option<f32>,
    /// Remediation recommendations
    pub remediation: Vec<RemediationRecommendation>,
    /// References and links
    pub references: Vec<String>,
}

/// Vulnerability severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Reflect)]
pub enum VulnerabilitySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}
```

#### Extension Distribution and Installation
```rust
/// Extension distribution management system
/// References: docs/bevy/examples/asset/asset_loading.rs (extension distribution)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ExtensionDistributionManager {
    /// Distribution channels
    pub distribution_channels: HashMap<String, DistributionChannel>,
    /// Installation tracking
    pub installation_tracker: InstallationTracker,
    /// Update management
    pub update_manager: ExtensionUpdateManager,
    /// Rollback and recovery
    pub rollback_manager: RollbackManager,
    /// CDN and caching
    pub cdn_config: CdnConfiguration,
}

/// Extension distribution channel
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct DistributionChannel {
    /// Channel identifier
    pub channel_id: String,
    /// Channel name and description
    pub channel_name: String,
    pub channel_description: String,
    /// Channel type
    pub channel_type: DistributionChannelType,
    /// Channel configuration
    pub configuration: ChannelConfiguration,
    /// Access control
    pub access_control: ChannelAccessControl,
    /// Channel statistics
    pub statistics: ChannelStatistics,
}

/// Installation tracking and management
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct InstallationTracker {
    /// Active installations by organization
    pub installations: HashMap<Uuid, HashMap<Uuid, ExtensionInstallation>>,
    /// Installation policies
    pub installation_policies: InstallationPolicies,
    /// Installation analytics
    pub installation_analytics: InstallationAnalytics,
    /// Error tracking and diagnostics
    pub error_tracker: InstallationErrorTracker,
}

/// Individual extension installation
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ExtensionInstallation {
    /// Installation unique identifier
    pub installation_id: Uuid,
    /// Extension being installed
    pub extension_id: Uuid,
    /// Installed version
    pub installed_version: Version,
    /// Installation timestamp
    pub installed_at: DateTime<Utc>,
    /// Who installed the extension
    pub installed_by: Uuid,
    /// Installation source/channel
    pub installation_source: String,
    /// Installation status
    pub status: InstallationStatus,
    /// Installation configuration
    pub configuration: ExtensionConfiguration,
    /// Usage tracking
    pub usage_tracking: ExtensionUsageTracking,
    /// Performance metrics
    pub performance_metrics: ExtensionPerformanceMetrics,
}
```

#### Extension Analytics and Insights
```rust
/// Comprehensive extension analytics system
/// References: docs/bevy/examples/ecs/event.rs (analytics event tracking)
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ExtensionAnalyticsEngine {
    /// Analytics data collectors
    pub data_collectors: Vec<AnalyticsCollector>,
    /// Analytics processing pipeline
    pub processing_pipeline: AnalyticsProcessingPipeline,
    /// Real-time metrics
    pub real_time_metrics: RealTimeMetrics,
    /// Historical analytics
    pub historical_analytics: HistoricalAnalytics,
    /// Predictive analytics
    pub predictive_analytics: PredictiveAnalytics,
    /// Custom analytics configurations
    pub custom_analytics: HashMap<Uuid, CustomAnalyticsConfig>,
}

/// Extension usage analytics
#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct ExtensionUsageAnalytics {
    /// Total installations across organizations
    pub total_installations: u64,
    /// Active installations (last 30 days)
    pub active_installations: u64,
    /// Daily active users
    pub daily_active_users: u64,
    /// Weekly active users
    pub weekly_active_users: u64,
    /// Monthly active users
    pub monthly_active_users: u64,
    /// Average session duration
    pub average_session_duration: f64,
    /// Feature usage breakdown
    pub feature_usage: HashMap<String, FeatureUsageMetrics>,
    /// Performance metrics
    pub performance_metrics: AggregatedPerformanceMetrics,
    /// Error rates and reliability
    pub reliability_metrics: ReliabilityMetrics,
    /// User satisfaction scores
    pub satisfaction_metrics: SatisfactionMetrics,
}

/// Individual feature usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct FeatureUsageMetrics {
    /// Feature name
    pub feature_name: String,
    /// Usage frequency
    pub usage_frequency: u64,
    /// Unique users using this feature
    pub unique_users: u64,
    /// Average usage duration
    pub average_duration: f64,
    /// Success rate
    pub success_rate: f32,
    /// User preference ranking
    pub preference_ranking: f32,
}
```

### System Implementation

#### Extension Store Management Systems
```rust
/// Extension store management systems
/// References: docs/bevy/examples/asset/asset_loading.rs (extension loading)

/// Main extension store management system
pub fn extension_store_management_system(
    mut store_system: ResMut<OrganizationExtensionStoreSystem>,
    mut extension_events: EventReader<ExtensionStoreEvent>,
    mut approval_events: EventWriter<ApprovalWorkflowEvent>,
    time: Res<Time>,
) {
    for event in extension_events.read() {
        match event {
            ExtensionStoreEvent::ExtensionSubmitted { organization_id, extension_id, submitter_id } => {
                if let Some(store) = store_system.organization_stores.get_mut(organization_id) {
                    // Start approval workflow
                    let workflow = create_approval_workflow(
                        *extension_id,
                        *organization_id,
                        &store.store_config.approval_workflow,
                    );
                    
                    // Trigger security scanning
                    let scan_request = SecurityScanRequest {
                        extension_id: *extension_id,
                        organization_id: *organization_id,
                        scan_policies: store.store_config.security_policies.clone(),
                        priority: ScanPriority::Normal,
                    };
                    
                    schedule_security_scan(&scan_request, &mut store_system.security_scanner);
                    
                    approval_events.send(ApprovalWorkflowEvent::WorkflowStarted {
                        workflow_id: workflow.workflow_id,
                        extension_id: *extension_id,
                        organization_id: *organization_id,
                    });
                }
            },
            ExtensionStoreEvent::ExtensionApproved { organization_id, extension_id, approver_id } => {
                // Move extension to approved list
                approve_extension(*organization_id, *extension_id, *approver_id, &mut store_system);
                
                // Make extension available for installation
                make_extension_available(*organization_id, *extension_id, &mut store_system);
            },
            ExtensionStoreEvent::ExtensionRejected { organization_id, extension_id, reason } => {
                // Handle extension rejection
                reject_extension(*organization_id, *extension_id, reason, &mut store_system);
            },
            ExtensionStoreEvent::ExtensionInstalled { organization_id, extension_id, user_id } => {
                // Track installation
                track_extension_installation(*organization_id, *extension_id, *user_id, &mut store_system);
                
                // Update analytics
                update_installation_analytics(*extension_id, &mut store_system.analytics_engine);
            },
        }
    }
    
    // Process periodic tasks
    process_periodic_store_tasks(&mut store_system, time.elapsed_secs());
}

/// Extension security scanning system
/// References: docs/bevy/examples/ecs/event.rs (security scan events)
pub fn extension_security_scanning_system(
    mut security_scanner: ResMut<ExtensionSecurityScanner>,
    mut scan_events: EventReader<SecurityScanEvent>,
    mut store_events: EventWriter<ExtensionStoreEvent>,
    global_registry: Res<GlobalExtensionRegistry>,
) {
    for event in scan_events.read() {
        match event {
            SecurityScanEvent::ScanRequested { scan_request } => {
                // Execute security scan
                let scan_result = execute_security_scan(scan_request, &security_scanner, &global_registry);
                
                match scan_result {
                    Ok(result) => {
                        // Store scan result
                        security_scanner.scan_history
                            .entry(scan_request.extension_id)
                            .or_insert_with(Vec::new)
                            .push(result.clone());
                        
                        // Evaluate scan result
                        let approval_recommendation = evaluate_security_scan_result(&result, &scan_request.scan_policies);
                        
                        store_events.send(ExtensionStoreEvent::SecurityScanCompleted {
                            extension_id: scan_request.extension_id,
                            organization_id: scan_request.organization_id,
                            scan_result: result,
                            approval_recommendation,
                        });
                    },
                    Err(error) => {
                        store_events.send(ExtensionStoreEvent::SecurityScanFailed {
                            extension_id: scan_request.extension_id,
                            organization_id: scan_request.organization_id,
                            error_message: error.to_string(),
                        });
                    }
                }
            },
            SecurityScanEvent::ThreatDetected { extension_id, threat_info } => {
                // Handle threat detection
                handle_security_threat(*extension_id, threat_info, &mut security_scanner, &mut store_events);
            },
        }
    }
    
    // Monitor for new threats
    monitor_security_threats(&mut security_scanner, &mut store_events);
}

/// Extension installation system
/// References: docs/bevy/examples/asset/asset_loading.rs (extension installation)
pub fn extension_installation_system(
    mut distribution_manager: ResMut<ExtensionDistributionManager>,
    mut installation_events: EventReader<ExtensionInstallationEvent>,
    mut analytics_events: EventWriter<ExtensionAnalyticsEvent>,
    asset_server: Res<AssetServer>,
) {
    for event in installation_events.read() {
        match event {
            ExtensionInstallationEvent::InstallationRequested { 
                organization_id, extension_id, user_id, installation_config 
            } => {
                // Validate installation request
                let validation_result = validate_installation_request(
                    *organization_id,
                    *extension_id,
                    *user_id,
                    installation_config,
                    &distribution_manager,
                );
                
                match validation_result {
                    Ok(validated_request) => {
                        // Begin installation process
                        let installation = start_extension_installation(
                            validated_request,
                            &mut distribution_manager,
                            &asset_server,
                        );
                        
                        analytics_events.send(ExtensionAnalyticsEvent::InstallationStarted {
                            installation_id: installation.installation_id,
                            extension_id: *extension_id,
                            organization_id: *organization_id,
                        });
                    },
                    Err(error) => {
                        analytics_events.send(ExtensionAnalyticsEvent::InstallationFailed {
                            extension_id: *extension_id,
                            organization_id: *organization_id,
                            error_type: InstallationErrorType::ValidationFailed,
                            error_message: error.to_string(),
                        });
                    }
                }
            },
            ExtensionInstallationEvent::InstallationCompleted { installation_id } => {
                // Finalize installation
                finalize_extension_installation(*installation_id, &mut distribution_manager);
                
                // Update usage tracking
                start_usage_tracking(*installation_id, &mut distribution_manager);
            },
            ExtensionInstallationEvent::InstallationFailed { installation_id, error } => {
                // Handle installation failure
                handle_installation_failure(*installation_id, error, &mut distribution_manager);
            },
        }
    }
    
    // Process ongoing installations
    process_ongoing_installations(&mut distribution_manager);
}

/// Extension analytics system
/// References: docs/bevy/examples/ecs/event.rs (analytics event processing)
pub fn extension_analytics_system(
    mut analytics_engine: ResMut<ExtensionAnalyticsEngine>,
    mut analytics_events: EventReader<ExtensionAnalyticsEvent>,
    time: Res<Time>,
) {
    for event in analytics_events.read() {
        match event {
            ExtensionAnalyticsEvent::ExtensionUsed { extension_id, user_id, usage_data } => {
                // Record usage event
                record_usage_event(*extension_id, *user_id, usage_data, &mut analytics_engine);
                
                // Update real-time metrics
                update_real_time_metrics(*extension_id, usage_data, &mut analytics_engine);
            },
            ExtensionAnalyticsEvent::FeatureUsed { extension_id, feature_name, usage_duration } => {
                // Track feature usage
                track_feature_usage(*extension_id, feature_name, *usage_duration, &mut analytics_engine);
            },
            ExtensionAnalyticsEvent::ErrorOccurred { extension_id, error_type, error_details } => {
                // Track error rates
                track_extension_error(*extension_id, error_type, error_details, &mut analytics_engine);
            },
        }
    }
    
    // Process analytics aggregation
    process_analytics_aggregation(&mut analytics_engine, time.elapsed_secs());
    
    // Generate insights and recommendations
    generate_analytics_insights(&mut analytics_engine);
}
```

### Event System Integration

```rust
/// Extension store system events
/// References: docs/bevy/examples/ecs/event.rs (extension store event patterns)

#[derive(Event, Debug, Clone)]
pub enum ExtensionStoreEvent {
    /// Extension submitted for approval
    ExtensionSubmitted {
        organization_id: Uuid,
        extension_id: Uuid,
        submitter_id: Uuid,
    },
    /// Extension approved for organization
    ExtensionApproved {
        organization_id: Uuid,
        extension_id: Uuid,
        approver_id: Uuid,
    },
    /// Extension rejected
    ExtensionRejected {
        organization_id: Uuid,
        extension_id: Uuid,
        reason: String,
    },
    /// Extension installed by user
    ExtensionInstalled {
        organization_id: Uuid,
        extension_id: Uuid,
        user_id: Uuid,
    },
    /// Security scan completed
    SecurityScanCompleted {
        extension_id: Uuid,
        organization_id: Uuid,
        scan_result: SecurityScanResult,
        approval_recommendation: ApprovalRecommendation,
    },
    /// Security scan failed
    SecurityScanFailed {
        extension_id: Uuid,
        organization_id: Uuid,
        error_message: String,
    },
}

#[derive(Event, Debug, Clone)]
pub enum ExtensionInstallationEvent {
    /// Installation requested
    InstallationRequested {
        organization_id: Uuid,
        extension_id: Uuid,
        user_id: Uuid,
        installation_config: InstallationConfiguration,
    },
    /// Installation completed successfully
    InstallationCompleted {
        installation_id: Uuid,
    },
    /// Installation failed
    InstallationFailed {
        installation_id: Uuid,
        error: InstallationError,
    },
}

#[derive(Event, Debug, Clone)]
pub enum ExtensionAnalyticsEvent {
    /// Extension was used
    ExtensionUsed {
        extension_id: Uuid,
        user_id: Uuid,
        usage_data: ExtensionUsageData,
    },
    /// Specific feature used
    FeatureUsed {
        extension_id: Uuid,
        feature_name: String,
        usage_duration: u64,
    },
    /// Error occurred in extension
    ErrorOccurred {
        extension_id: Uuid,
        error_type: ExtensionErrorType,
        error_details: String,
    },
    /// Installation analytics
    InstallationStarted {
        installation_id: Uuid,
        extension_id: Uuid,
        organization_id: Uuid,
    },
    InstallationFailed {
        extension_id: Uuid,
        organization_id: Uuid,
        error_type: InstallationErrorType,
        error_message: String,
    },
}
```

### Plugin Registration

```rust
/// Extension store plugin for Bevy integration
/// References: docs/bevy/examples/app/* (plugin patterns)
pub struct OrganizationExtensionStorePlugin;

impl Plugin for OrganizationExtensionStorePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<OrganizationExtensionStoreSystem>()
            .add_event::<ExtensionStoreEvent>()
            .add_event::<ExtensionInstallationEvent>()
            .add_event::<ExtensionAnalyticsEvent>()
            .add_event::<SecurityScanEvent>()
            .add_event::<ApprovalWorkflowEvent>()
            .add_systems(
                Update,
                (
                    extension_store_management_system,
                    extension_security_scanning_system,
                    extension_installation_system,
                    extension_analytics_system,
                    approval_workflow_system,
                ).chain()
            );
    }
}
```

### Supporting Data Types

```rust
/// Supporting types for extension store system

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum ExtensionCategory {
    Productivity,
    Development,
    Communication,
    Analytics,
    Security,
    Integration,
    Automation,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum PackageFormat {
    WebAssembly,
    NativeBinary,
    JavaScript,
    Plugin,
    Theme,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum InstallationStatus {
    Pending,
    Installing,
    Installed,
    Failed,
    Updating,
    Uninstalling,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum VulnerabilityType {
    CodeExecution,
    PrivilegeEscalation,
    DataExposure,
    CrossSiteScripting,
    SqlInjection,
    DependencyVulnerability,
    ConfigurationIssue,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ApprovalRecommendation {
    pub recommendation: ApprovalDecision,
    pub confidence_score: f32,
    pub reasoning: String,
    pub conditions: Vec<ApprovalCondition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum ApprovalDecision {
    Approve,
    Reject,
    RequestChanges,
    RequireManualReview,
}
```

### Implementation Requirements

1. **Security-First Design**: Comprehensive security scanning with zero-tolerance for critical vulnerabilities
2. **Enterprise Compliance**: Support for SOC 2, GDPR, and industry-specific compliance requirements
3. **Scalable Architecture**: Handle thousands of extensions across multiple organizations efficiently
4. **Real-time Analytics**: Live extension usage and performance monitoring with actionable insights
5. **Automated Workflows**: Streamlined approval processes with intelligent automation and human oversight
6. **Version Management**: Sophisticated extension versioning with dependency resolution and rollback capabilities
7. **Multi-tenant Isolation**: Complete isolation of extension stores between organizations
8. **Performance Optimization**: Fast extension discovery, installation, and execution with minimal resource overhead

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

**Implementation References:**
- `docs/bevy/examples/asset/asset_loading.rs:1-200` - Extension asset management and loading patterns
- `docs/bevy/examples/ecs/hierarchy.rs:1-150` - Hierarchical extension categories and organization
- `docs/bevy/examples/ecs/event.rs:1-144` - Event-driven extension workflows and analytics
- `docs/bevy/examples/asset/hot_reloading.rs:1-100` - Extension hot-reloading and update mechanisms
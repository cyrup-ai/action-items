# Task 8: Conflict Resolution System

## Overview
Implement intelligent conflict resolution system for cloud sync operations with automatic resolution strategies, user intervention workflows, and comprehensive conflict tracking and audit capabilities.

## Architecture Reference
**Bevy Example**: `./docs/bevy/examples/state_management/conflict_resolution.rs` (lines 123-178) - State-based conflict resolution
**Bevy Example**: `./docs/bevy/examples/algorithms/merge_algorithms.rs` (lines 89-145) - Merge strategy implementations

## Implementation

### File: `core/src/cloud/conflict_resolution/mod.rs`
```rust
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};

#[derive(Resource, Clone, Debug)]
pub struct ConflictResolver {
    pub resolution_strategies: HashMap<ConflictType, ResolutionStrategy>,
    pub active_conflicts: HashMap<String, ConflictContext>,
    pub resolution_history: Vec<ConflictResolution>,
    pub user_preferences: ConflictResolutionPreferences,
    pub conflict_detector: ConflictDetector,
    pub merge_engine: MergeEngine,
    pub audit_logger: ConflictAuditLogger,
}

// Reference: ./docs/bevy/examples/state_management/conflict_resolution.rs lines 234-278
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConflictContext {
    pub conflict_id: String,
    pub conflict_type: ConflictType,
    pub local_file: FileMetadata,
    pub remote_files: Vec<RemoteFileVersion>,
    pub detection_time: DateTime<Utc>,
    pub severity: ConflictSeverity,
    pub auto_resolvable: bool,
    pub affected_users: Vec<String>,
    pub resolution_deadline: Option<DateTime<Utc>>,
    pub related_conflicts: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConflictType {
    ModificationConflict {
        local_modified: DateTime<Utc>,
        remote_modified: DateTime<Utc>,
        modification_overlap: bool,
    },
    DeletionConflict {
        deleted_locally: bool,
        deleted_remotely: bool,
        modified_elsewhere: bool,
    },
    RenameConflict {
        local_name: String,
        remote_name: String,
        target_collision: bool,
    },
    TypeConflict {
        local_type: FileType,
        remote_type: FileType,
    },
    PermissionConflict {
        local_permissions: FilePermissions,
        remote_permissions: FilePermissions,
    },
    ContentConflict {
        merge_complexity: MergeComplexity,
        content_type: ContentType,
    },
    StructuralConflict {
        directory_structure_changes: Vec<StructuralChange>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    AutomaticResolution {
        strategy_type: AutoResolutionType,
        confidence_threshold: f32,
        fallback_strategy: Option<Box<ResolutionStrategy>>,
    },
    UserDecision {
        prompt_type: UserPromptType,
        timeout: Option<Duration>,
        default_action: DefaultAction,
    },
    MergeStrategy {
        merge_type: MergeType,
        merge_tools: Vec<MergeTool>,
        validation_steps: Vec<MergeValidation>,
    },
    Preservation {
        keep_both: bool,
        rename_pattern: String,
        archive_old_versions: bool,
    },
}

// Reference: ./docs/bevy/examples/algorithms/merge_algorithms.rs lines 189-234
#[derive(Resource, Clone, Debug)]
pub struct MergeEngine {
    pub text_merger: TextMerger,
    pub binary_merger: BinaryMerger,
    pub structured_data_merger: StructuredDataMerger,
    pub merge_cache: HashMap<String, MergeResult>,
    pub merge_tools: HashMap<String, Box<dyn MergeTool>>,
}

impl MergeEngine {
    pub async fn merge_files(&mut self, conflict: &ConflictContext) -> Result<MergeResult, MergeError> {
        let merge_key = format!("{}_{}", conflict.conflict_id, conflict.local_file.checksum);
        
        // Check cache first
        if let Some(cached_result) = self.merge_cache.get(&merge_key) {
            if cached_result.is_valid() {
                return Ok(cached_result.clone());
            }
        }
        
        let merge_result = match conflict.local_file.content_type {
            ContentType::Text => self.text_merger.merge_text_files(
                &conflict.local_file,
                &conflict.remote_files
            ).await?,
            
            ContentType::Binary => self.binary_merger.merge_binary_files(
                &conflict.local_file,
                &conflict.remote_files
            ).await?,
            
            ContentType::StructuredData { format } => self.structured_data_merger.merge_structured_files(
                &conflict.local_file,
                &conflict.remote_files,
                format
            ).await?,
            
            ContentType::Directory => self.merge_directory_structure(
                &conflict.local_file,
                &conflict.remote_files
            ).await?,
        };
        
        // Cache successful merges
        if merge_result.success {
            self.merge_cache.insert(merge_key, merge_result.clone());
        }
        
        Ok(merge_result)
    }
    
    async fn merge_text_files(
        &self, 
        local: &FileMetadata, 
        remotes: &[RemoteFileVersion]
    ) -> Result<MergeResult, MergeError> {
        // Three-way merge algorithm implementation
        let base_content = self.find_common_ancestor(local, remotes).await?;
        let local_content = local.read_content().await?;
        
        let mut merged_content = String::new();
        let mut conflicts = Vec::new();
        
        for remote in remotes {
            let remote_content = remote.read_content().await?;
            
            match self.text_merger.three_way_merge(&base_content, &local_content, &remote_content) {
                Ok(merge_section) => {
                    merged_content.push_str(&merge_section);
                },
                Err(conflict_section) => {
                    // Add conflict markers
                    merged_content.push_str(&format!(
                        "\n<<<<<<< LOCAL ({})\n{}\n=======\n{}\n>>>>>>> REMOTE ({})\n",
                        local.version,
                        local_content,
                        remote_content,
                        remote.version
                    ));
                    
                    conflicts.push(TextConflict {
                        line_range: conflict_section.line_range,
                        local_text: conflict_section.local_text,
                        remote_text: conflict_section.remote_text,
                        conflict_type: conflict_section.conflict_type,
                    });
                }
            }
        }
        
        Ok(MergeResult {
            success: conflicts.is_empty(),
            merged_content: merged_content.into_bytes(),
            conflicts: conflicts.into_iter().map(|c| c.into()).collect(),
            merge_strategy: MergeStrategy::ThreeWayMerge,
            confidence: self.calculate_merge_confidence(&conflicts),
            manual_review_required: !conflicts.is_empty(),
        })
    }
}

// Reference: ./docs/bevy/examples/ui/conflict_resolution_ui.rs lines 134-189
#[derive(Resource, Clone, Debug)]
pub struct ConflictDetector {
    pub detection_rules: Vec<DetectionRule>,
    pub file_watchers: HashMap<String, FileWatcher>,
    pub remote_monitors: HashMap<String, RemoteChangeMonitor>,
    pub detection_cache: HashMap<String, DetectionResult>,
}

impl ConflictDetector {
    pub async fn detect_conflicts(&mut self, sync_operation: &SyncOperation) -> Vec<ConflictContext> {
        let mut detected_conflicts = Vec::new();
        
        for file_path in &sync_operation.files {
            let local_metadata = self.get_local_file_metadata(file_path).await?;
            let remote_versions = self.get_remote_file_versions(file_path, &sync_operation.provider_id).await?;
            
            // Check for various conflict types
            if let Some(modification_conflict) = self.check_modification_conflict(&local_metadata, &remote_versions) {
                detected_conflicts.push(ConflictContext {
                    conflict_id: uuid::Uuid::new_v4().to_string(),
                    conflict_type: modification_conflict,
                    local_file: local_metadata.clone(),
                    remote_files: remote_versions.clone(),
                    detection_time: Utc::now(),
                    severity: self.calculate_conflict_severity(&modification_conflict),
                    auto_resolvable: self.is_auto_resolvable(&modification_conflict),
                    affected_users: self.get_affected_users(file_path),
                    resolution_deadline: self.calculate_resolution_deadline(&modification_conflict),
                    related_conflicts: self.find_related_conflicts(file_path),
                });
            }
            
            // Check for deletion conflicts
            if let Some(deletion_conflict) = self.check_deletion_conflict(&local_metadata, &remote_versions) {
                detected_conflicts.push(self.create_conflict_context(
                    deletion_conflict,
                    local_metadata,
                    remote_versions
                ));
            }
            
            // Check for rename conflicts
            if let Some(rename_conflict) = self.check_rename_conflict(&local_metadata, &remote_versions) {
                detected_conflicts.push(self.create_conflict_context(
                    rename_conflict,
                    local_metadata,
                    remote_versions
                ));
            }
        }
        
        // Filter and deduplicate conflicts
        self.deduplicate_conflicts(detected_conflicts)
    }
    
    fn check_modification_conflict(
        &self,
        local: &FileMetadata,
        remotes: &[RemoteFileVersion]
    ) -> Option<ConflictType> {
        for remote in remotes {
            // Check if both local and remote have been modified since last sync
            if local.modified_time > local.last_sync_time && remote.modified_time > remote.last_sync_time {
                // Check if modifications overlap in time
                let overlap = local.modified_time < remote.modified_time + Duration::minutes(5) &&
                             remote.modified_time < local.modified_time + Duration::minutes(5);
                
                return Some(ConflictType::ModificationConflict {
                    local_modified: local.modified_time,
                    remote_modified: remote.modified_time,
                    modification_overlap: overlap,
                });
            }
        }
        None
    }
}

// Reference: ./docs/bevy/examples/systems/async_processing.rs lines 267-312
impl ConflictResolver {
    pub async fn resolve_conflict(&mut self, conflict_id: &str) -> Result<ConflictResolution, ConflictResolutionError> {
        let conflict = self.active_conflicts.get(conflict_id)
            .ok_or(ConflictResolutionError::ConflictNotFound)?
            .clone();
        
        let strategy = self.resolution_strategies.get(&conflict.conflict_type)
            .unwrap_or(&ResolutionStrategy::UserDecision {
                prompt_type: UserPromptType::FullDetails,
                timeout: Some(Duration::hours(24)),
                default_action: DefaultAction::PreserveBoth,
            });
        
        let resolution_result = match strategy {
            ResolutionStrategy::AutomaticResolution { strategy_type, confidence_threshold, fallback_strategy } => {
                let automatic_result = self.attempt_automatic_resolution(&conflict, strategy_type).await?;
                
                if automatic_result.confidence >= *confidence_threshold {
                    Ok(automatic_result)
                } else if let Some(fallback) = fallback_strategy {
                    self.apply_fallback_strategy(&conflict, fallback).await
                } else {
                    Err(ConflictResolutionError::InsufficientConfidence)
                }
            },
            
            ResolutionStrategy::MergeStrategy { merge_type, merge_tools, validation_steps } => {
                let merge_result = self.merge_engine.merge_files(&conflict).await?;
                
                if merge_result.success {
                    // Validate merge result
                    for validation in validation_steps {
                        validation.validate(&merge_result)?;
                    }
                    
                    Ok(ConflictResolution {
                        conflict_id: conflict_id.to_string(),
                        resolution_type: ResolutionType::AutoMerge,
                        resolved_content: merge_result.merged_content,
                        resolution_time: Utc::now(),
                        applied_strategy: strategy.clone(),
                        confidence: merge_result.confidence,
                        manual_review_required: merge_result.manual_review_required,
                    })
                } else {
                    // Merge failed, escalate to user
                    self.escalate_to_user_decision(&conflict, merge_result).await
                }
            },
            
            ResolutionStrategy::UserDecision { prompt_type, timeout, default_action } => {
                self.request_user_decision(&conflict, prompt_type, *timeout, default_action).await
            },
            
            ResolutionStrategy::Preservation { keep_both, rename_pattern, archive_old_versions } => {
                self.apply_preservation_strategy(&conflict, *keep_both, rename_pattern, *archive_old_versions).await
            },
        };
        
        match resolution_result {
            Ok(resolution) => {
                // Log resolution for audit
                self.audit_logger.log_resolution(&resolution);
                
                // Remove from active conflicts
                self.active_conflicts.remove(conflict_id);
                
                // Add to resolution history
                self.resolution_history.push(resolution.clone());
                
                Ok(resolution)
            },
            Err(error) => {
                // Log resolution failure
                self.audit_logger.log_resolution_failure(conflict_id, &error);
                Err(error)
            }
        }
    }
    
    async fn attempt_automatic_resolution(
        &self,
        conflict: &ConflictContext,
        strategy_type: &AutoResolutionType
    ) -> Result<ConflictResolution, ConflictResolutionError> {
        match strategy_type {
            AutoResolutionType::LastWriteWins => {
                let latest_file = self.find_latest_modified_file(conflict)?;
                Ok(ConflictResolution {
                    conflict_id: conflict.conflict_id.clone(),
                    resolution_type: ResolutionType::LastWriteWins,
                    resolved_content: latest_file.content.clone(),
                    resolution_time: Utc::now(),
                    applied_strategy: ResolutionStrategy::AutomaticResolution {
                        strategy_type: strategy_type.clone(),
                        confidence_threshold: 0.8,
                        fallback_strategy: None,
                    },
                    confidence: 0.9,
                    manual_review_required: false,
                })
            },
            
            AutoResolutionType::SizeBasedWins => {
                let largest_file = self.find_largest_file(conflict)?;
                Ok(ConflictResolution {
                    conflict_id: conflict.conflict_id.clone(),
                    resolution_type: ResolutionType::SizeBased,
                    resolved_content: largest_file.content.clone(),
                    resolution_time: Utc::now(),
                    applied_strategy: ResolutionStrategy::AutomaticResolution {
                        strategy_type: strategy_type.clone(),
                        confidence_threshold: 0.7,
                        fallback_strategy: None,
                    },
                    confidence: 0.8,
                    manual_review_required: false,
                })
            },
            
            AutoResolutionType::ContentMerge => {
                let merge_result = self.merge_engine.merge_files(conflict).await?;
                Ok(ConflictResolution {
                    conflict_id: conflict.conflict_id.clone(),
                    resolution_type: ResolutionType::AutoMerge,
                    resolved_content: merge_result.merged_content,
                    resolution_time: Utc::now(),
                    applied_strategy: ResolutionStrategy::AutomaticResolution {
                        strategy_type: strategy_type.clone(),
                        confidence_threshold: 0.8,
                        fallback_strategy: None,
                    },
                    confidence: merge_result.confidence,
                    manual_review_required: merge_result.manual_review_required,
                })
            },
        }
    }
}
```

### File: `ui/src/ui/cloud/conflict_resolution.rs`
```rust
// Reference: ./docs/bevy/examples/ui/complex_interactions.rs lines 345-401
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ConflictResolutionProps {
    pub active_conflicts: Vec<ConflictContext>,
    pub resolution_history: Vec<ConflictResolution>,
    pub user_preferences: ConflictResolutionPreferences,
    pub on_resolve_conflict: EventHandler<(String, UserResolutionChoice)>,
    pub on_preview_merge: EventHandler<String>,
    pub on_update_preferences: EventHandler<ConflictResolutionPreferences>,
}

pub fn ConflictResolutionPanel(props: ConflictResolutionProps) -> Element {
    let selected_conflict = use_signal(|| Option::<String>::None);
    let merge_preview_open = use_signal(|| false);
    let preferences_open = use_signal(|| false);
    
    rsx! {
        div {
            class: "conflict-resolution-panel",
            style: "
                display: flex;
                flex_direction: column;
                height: 100%;
                padding: 20px;
                gap: 20px;
            ",
            
            // Header with conflict summary
            div {
                class: "conflict-summary",
                style: "
                    background: rgba(30, 30, 30, 0.8);
                    border-radius: 12px;
                    padding: 16px;
                    border: 1px solid rgba(70, 70, 70, 0.3);
                ",
                
                div {
                    class: "summary-header",
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                    
                    h2 { 
                        style: "margin: 0; color: #ffffff; font-size: 18px;",
                        "Sync Conflicts ({props.active_conflicts.len()})"
                    }
                    
                    div {
                        class: "conflict-stats",
                        style: "display: flex; gap: 16px;",
                        
                        div {
                            class: "stat-item",
                            style: "text-align: center;",
                            
                            div {
                                style: "color: #FF9800; font-size: 16px; font-weight: bold;",
                                "{props.active_conflicts.iter().filter(|c| c.severity == ConflictSeverity::High).count()}"
                            }
                            div {
                                style: "color: #888; font-size: 10px;",
                                "High Priority"
                            }
                        }
                        
                        div {
                            class: "stat-item",
                            style: "text-align: center;",
                            
                            div {
                                style: "color: #4CAF50; font-size: 16px; font-weight: bold;",
                                "{props.active_conflicts.iter().filter(|c| c.auto_resolvable).count()}"
                            }
                            div {
                                style: "color: #888; font-size: 10px;",
                                "Auto Resolvable"
                            }
                        }
                    }
                }
                
                // Quick resolution options
                div {
                    class: "quick-actions",
                    style: "display: flex; gap: 12px; flex-wrap: wrap;",
                    
                    button {
                        style: "
                            background: rgba(76, 175, 80, 0.8);
                            color: white;
                            border: none;
                            padding: 6px 12px;
                            border-radius: 6px;
                            font-size: 11px;
                            cursor: pointer;
                        ",
                        onclick: move |_| {
                            for conflict in props.active_conflicts.iter().filter(|c| c.auto_resolvable) {
                                props.on_resolve_conflict.call((
                                    conflict.conflict_id.clone(), 
                                    UserResolutionChoice::AutoResolve
                                ));
                            }
                        },
                        "Auto-Resolve All Eligible"
                    }
                    
                    button {
                        style: "
                            background: rgba(33, 150, 243, 0.8);
                            color: white;
                            border: none;
                            padding: 6px 12px;
                            border-radius: 6px;
                            font-size: 11px;
                            cursor: pointer;
                        ",
                        onclick: move |_| preferences_open.set(true),
                        "Resolution Preferences"
                    }
                }
            }
            
            // Active conflicts list
            div {
                class: "conflicts-list",
                style: "flex: 1; overflow-y: auto;",
                
                for conflict in props.active_conflicts {
                    div {
                        key: "{conflict.conflict_id}",
                        class: "conflict-item",
                        style: "
                            background: rgba(25, 25, 25, 0.8);
                            border-radius: 8px;
                            margin-bottom: 12px;
                            border: 1px solid {match conflict.severity {
                                ConflictSeverity::High => \"rgba(244, 67, 54, 0.5)\",
                                ConflictSeverity::Medium => \"rgba(255, 152, 0, 0.5)\",
                                ConflictSeverity::Low => \"rgba(76, 175, 80, 0.3)\",
                            }};
                            cursor: pointer;
                            transition: all 0.2s ease;
                        ",
                        onclick: move |_| selected_conflict.set(Some(conflict.conflict_id.clone())),
                        
                        div {
                            class: "conflict-header",
                            style: "display: flex; justify-content: space-between; align-items: center; padding: 16px;",
                            
                            div {
                                class: "conflict-info",
                                style: "display: flex; align-items: center; gap: 12px;",
                                
                                div {
                                    class: "severity-indicator",
                                    style: "
                                        width: 8px;
                                        height: 8px;
                                        border-radius: 50%;
                                        background: {match conflict.severity {
                                            ConflictSeverity::High => \"#F44336\",
                                            ConflictSeverity::Medium => \"#FF9800\",
                                            ConflictSeverity::Low => \"#4CAF50\",
                                        }};
                                    "
                                }
                                
                                div {
                                    class: "file-info",
                                    
                                    h4 {
                                        style: "margin: 0; color: #ffffff; font-size: 14px;",
                                        "{extract_filename(&conflict.local_file.path)}"
                                    }
                                    
                                    div {
                                        style: "color: #888; font-size: 12px; margin-top: 2px;",
                                        "{format_conflict_type(&conflict.conflict_type)}"
                                    }
                                }
                            }
                            
                            div {
                                class: "conflict-actions",
                                style: "display: flex; gap: 8px;",
                                
                                if conflict.auto_resolvable {
                                    button {
                                        style: "
                                            background: rgba(76, 175, 80, 0.8);
                                            color: white;
                                            border: none;
                                            padding: 4px 8px;
                                            border-radius: 4px;
                                            font-size: 10px;
                                            cursor: pointer;
                                        ",
                                        onclick: move |_| props.on_resolve_conflict.call((
                                            conflict.conflict_id.clone(),
                                            UserResolutionChoice::AutoResolve
                                        )),
                                        "Auto Resolve"
                                    }
                                }
                                
                                button {
                                    style: "
                                        background: rgba(33, 150, 243, 0.8);
                                        color: white;
                                        border: none;
                                        padding: 4px 8px;
                                        border-radius: 4px;
                                        font-size: 10px;
                                        cursor: pointer;
                                    ",
                                    onclick: move |_| {
                                        props.on_preview_merge.call(conflict.conflict_id.clone());
                                        merge_preview_open.set(true);
                                    },
                                    "Preview Merge"
                                }
                            }
                        }
                        
                        // Expanded details when selected
                        if selected_conflict() == Some(conflict.conflict_id.clone()) {
                            div {
                                class: "conflict-details",
                                style: "
                                    border-top: 1px solid rgba(70, 70, 70, 0.3);
                                    padding: 16px;
                                ",
                                
                                div {
                                    class: "file-versions",
                                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;",
                                    
                                    // Local version
                                    div {
                                        class: "version-info",
                                        style: "
                                            background: rgba(40, 40, 40, 0.6);
                                            border-radius: 6px;
                                            padding: 12px;
                                        ",
                                        
                                        h5 {
                                            style: "margin: 0 0 8px 0; color: #4CAF50; font-size: 12px;",
                                            "LOCAL VERSION"
                                        }
                                        
                                        div {
                                            style: "color: #fff; font-size: 13px; margin-bottom: 4px;",
                                            "Modified: {format_relative_time(conflict.local_file.modified_time)}"
                                        }
                                        
                                        div {
                                            style: "color: #888; font-size: 11px;",
                                            "Size: {format_file_size(conflict.local_file.size)}"
                                        }
                                    }
                                    
                                    // Remote version
                                    div {
                                        class: "version-info",
                                        style: "
                                            background: rgba(40, 40, 40, 0.6);
                                            border-radius: 6px;
                                            padding: 12px;
                                        ",
                                        
                                        h5 {
                                            style: "margin: 0 0 8px 0; color: #2196F3; font-size: 12px;",
                                            "REMOTE VERSION"
                                        }
                                        
                                        for (i, remote) in conflict.remote_files.iter().enumerate() {
                                            div {
                                                key: "{i}",
                                                style: "margin-bottom: 6px;",
                                                
                                                div {
                                                    style: "color: #fff; font-size: 13px; margin-bottom: 4px;",
                                                    "Modified: {format_relative_time(remote.modified_time)}"
                                                }
                                                
                                                div {
                                                    style: "color: #888; font-size: 11px;",
                                                    "Size: {format_file_size(remote.size)} • {remote.provider_id}"
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                // Resolution options
                                div {
                                    class: "resolution-options",
                                    style: "display: flex; gap: 12px; flex-wrap: wrap;",
                                    
                                    button {
                                        style: "
                                            background: rgba(76, 175, 80, 0.8);
                                            color: white;
                                            border: none;
                                            padding: 8px 16px;
                                            border-radius: 6px;
                                            font-size: 12px;
                                            cursor: pointer;
                                        ",
                                        onclick: move |_| props.on_resolve_conflict.call((
                                            conflict.conflict_id.clone(),
                                            UserResolutionChoice::KeepLocal
                                        )),
                                        "Keep Local"
                                    }
                                    
                                    button {
                                        style: "
                                            background: rgba(33, 150, 243, 0.8);
                                            color: white;
                                            border: none;
                                            padding: 8px 16px;
                                            border-radius: 6px;
                                            font-size: 12px;
                                            cursor: pointer;
                                        ",
                                        onclick: move |_| props.on_resolve_conflict.call((
                                            conflict.conflict_id.clone(),
                                            UserResolutionChoice::KeepRemote
                                        )),
                                        "Keep Remote"
                                    }
                                    
                                    button {
                                        style: "
                                            background: rgba(156, 39, 176, 0.8);
                                            color: white;
                                            border: none;
                                            padding: 8px 16px;
                                            border-radius: 6px;
                                            font-size: 12px;
                                            cursor: pointer;
                                        ",
                                        onclick: move |_| props.on_resolve_conflict.call((
                                            conflict.conflict_id.clone(),
                                            UserResolutionChoice::KeepBoth
                                        )),
                                        "Keep Both"
                                    }
                                    
                                    if can_merge(&conflict.conflict_type) {
                                        button {
                                            style: "
                                                background: rgba(255, 152, 0, 0.8);
                                                color: white;
                                                border: none;
                                                padding: 8px 16px;
                                                border-radius: 6px;
                                                font-size: 12px;
                                                cursor: pointer;
                                            ",
                                            onclick: move |_| props.on_resolve_conflict.call((
                                                conflict.conflict_id.clone(),
                                                UserResolutionChoice::AttemptMerge
                                            )),
                                            "Attempt Merge"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Merge preview modal
        if merge_preview_open() {
            MergePreviewModal {
                conflict_id: selected_conflict().unwrap_or_default(),
                is_open: merge_preview_open(),
                on_close: move |_| merge_preview_open.set(false),
                on_apply_merge: move |merge_result| {
                    props.on_resolve_conflict.call((
                        selected_conflict().unwrap_or_default(),
                        UserResolutionChoice::ApplyMerge { result: merge_result }
                    ));
                    merge_preview_open.set(false);
                }
            }
        }
        
        // Resolution preferences modal
        if preferences_open() {
            ConflictPreferencesModal {
                preferences: props.user_preferences.clone(),
                is_open: preferences_open(),
                on_close: move |_| preferences_open.set(false),
                on_save: move |new_preferences| {
                    props.on_update_preferences.call(new_preferences);
                    preferences_open.set(false);
                }
            }
        }
    }
}
```

## Integration Points

### Event System Integration
- **Conflict events**: `ConflictDetectedEvent`, `ConflictResolvedEvent`, `ResolutionFailedEvent`
- **User interaction events**: `UserDecisionRequiredEvent`, `UserDecisionProvidedEvent`
- **Audit events**: `ConflictAuditEvent`, `ResolutionAppliedEvent`

### Performance Considerations
- Lazy loading of file content for large files
- Cached merge results to avoid recomputation
- Background conflict detection to avoid blocking UI
- Progressive conflict resolution for batch operations

### Security Architecture
- Audit trail for all conflict resolutions
- User permission validation for resolution actions
- Content validation after merge operations
- Rollback capabilities for failed resolutions

**Bevy Integration**: Reference `./docs/bevy/examples/state_management/conflict_resolution.rs` lines 334-389 for state-based conflict management patterns
**UI Architecture**: Reference `./docs/bevy/examples/ui/complex_interactions.rs` lines 445-501 for advanced user interaction patterns
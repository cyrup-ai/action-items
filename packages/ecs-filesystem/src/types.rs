use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use bevy::prelude::{Entity, Event};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use uuid::Uuid;

/// Unique identifier for filesystem operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileOperationId(pub Uuid);

impl FileOperationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FileOperationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Filesystem operation errors with detailed context
#[derive(Debug, thiserror::Error)]
pub enum FileSystemError {
    #[error("Access denied: {path}")]
    AccessDenied { path: PathBuf },

    #[error("Path not found: {path}")]
    NotFound { path: PathBuf },

    #[error("Path traversal attempt blocked: {path}")]
    PathTraversalBlocked { path: PathBuf },

    #[error("Path not in allowed directories: {path}")]
    PathNotAllowed { path: PathBuf },

    #[error("File size exceeds limit: {size} > {limit}")]
    FileSizeExceeded { size: u64, limit: u64 },

    #[error("Unsupported file type: {extension}")]
    UnsupportedFileType { extension: String },

    #[error("Operation timeout: {operation}")]
    Timeout { operation: String },

    #[error("IO error: {source}")]
    Io { source: std::io::Error },

    #[error("Serialization error: {source}")]
    Serialization { source: serde_json::Error },

    #[error("Watch error: {source}")]
    Watch { source: notify::Error },

    #[error("System resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
}
/// File metadata with security and performance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub accessed: SystemTime,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub permissions: FilePermissions,
    pub checksum: Option<String>,
    pub mime_type: Option<String>,
}

/// Cross-platform file permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
    pub hidden: bool,
}

/// File content with metadata and security validation
#[derive(Debug, Clone)]
pub struct FileContent {
    pub metadata: FileMetadata,
    pub data: Vec<u8>,
    pub encoding: Option<String>,
    pub validated: bool,
    pub checksum: String,
}

/// Directory listing with optimized performance
#[derive(Debug, Clone)]
pub struct DirectoryListing {
    pub path: PathBuf,
    pub entries: SmallVec<[FileMetadata; 32]>, // Most directories have <32 entries
    pub total_size: u64,
    pub entry_count: usize,
    pub scan_duration: Duration,
    pub recursive: bool,
}
/// Checksum algorithm selection for file integrity verification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ChecksumAlgorithm {
    /// BLAKE3 - Fast, secure, parallelizable (recommended)
    #[default]
    Blake3,
    /// SHA-256 - Standard cryptographic hash for compatibility
    Sha256,
}

/// File watching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    pub recursive: bool,
    pub debounce_duration: Duration,
    pub ignore_patterns: SmallVec<[String; 8]>,
    pub max_events_per_second: u32,
    pub include_metadata: bool,
    /// Timeout for metadata extraction operations to prevent hanging on unresponsive filesystems
    pub metadata_timeout: Duration,
    /// Time window for tracking rename operations to correlate From/To events
    pub rename_tracking_window: Duration,
    /// Enable checksum calculation for file integrity verification (performance impact)
    pub checksum_enabled: bool,
    /// Checksum algorithm to use when checksum_enabled is true
    pub checksum_algorithm: ChecksumAlgorithm,
    /// Maximum file size (in bytes) for synchronous checksum calculation to avoid blocking
    pub sync_checksum_threshold_bytes: u64,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            recursive: true,
            debounce_duration: Duration::from_millis(500),
            ignore_patterns: SmallVec::new(),
            max_events_per_second: 100,
            include_metadata: false,
            metadata_timeout: Duration::from_millis(2000), // 2 second timeout for metadata operations
            rename_tracking_window: Duration::from_millis(1000), // 1 second window for rename correlation
            checksum_enabled: false, // Disabled by default for performance
            checksum_algorithm: ChecksumAlgorithm::default(), // Blake3 by default
            sync_checksum_threshold_bytes: 1_048_576, // 1MB threshold for sync checksum calculation
        }
    }
}

/// Filesystem change event with detailed information
#[derive(Debug, Clone)]
pub struct FileSystemChange {
    pub event_id: FileOperationId,
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub metadata: Option<FileMetadata>,
    pub timestamp: SystemTime,
}

/// Types of filesystem changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
    Moved { from: PathBuf },
    AttributesChanged,
}
/// File operation priority for batching
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
pub enum Priority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Critical = 3,
}


/// Bevy Events for filesystem operations following ARCHITECTURE.md pattern
#[derive(Event, Debug)]
pub enum FileSystemRequest {
    ReadFile {
        operation_id: FileOperationId,
        path: PathBuf,
        requester: Entity,
        priority: Priority,
    },
    WriteFile {
        operation_id: FileOperationId,
        path: PathBuf,
        content: Vec<u8>,
        atomic: bool,
        requester: Entity,
        priority: Priority,
    },
    ListDirectory {
        operation_id: FileOperationId,
        path: PathBuf,
        recursive: bool,
        requester: Entity,
        priority: Priority,
    },
    WatchDirectory {
        operation_id: FileOperationId,
        path: PathBuf,
        config: Box<WatchConfig>,
        requester: Entity,
    },
    CheckPermissions {
        operation_id: FileOperationId,
        path: PathBuf,
        requester: Entity,
    },
    CreateDirectory {
        operation_id: FileOperationId,
        path: PathBuf,
        recursive: bool,
        requester: Entity,
        priority: Priority,
    },
}

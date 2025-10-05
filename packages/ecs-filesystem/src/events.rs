use std::path::PathBuf;

use bevy::prelude::{Entity, Event};

use crate::types::{
    DirectoryListing, FileContent, FileMetadata, FileOperationId, FileSystemChange, FileSystemError,
};

/// Bevy Events for filesystem responses following ARCHITECTURE.md pattern
#[derive(Event, Debug)]
pub enum FileSystemResponse {
    ReadFileResult {
        operation_id: FileOperationId,
        requester: Entity,
        result: Box<Result<FileContent, FileSystemError>>,
    },
    WriteFileResult {
        operation_id: FileOperationId,
        requester: Entity,
        result: Result<(), FileSystemError>,
    },
    ListDirectoryResult {
        operation_id: FileOperationId,
        requester: Entity,
        result: Box<Result<DirectoryListing, FileSystemError>>,
    },
    WatchDirectoryResult {
        operation_id: FileOperationId,
        requester: Entity,
        result: Result<(), FileSystemError>,
    },
    CheckPermissionsResult {
        operation_id: FileOperationId,
        requester: Entity,
        result: Result<FileMetadata, FileSystemError>,
    },
    CreateDirectoryResult {
        operation_id: FileOperationId,
        requester: Entity,
        result: Result<(), FileSystemError>,
    },
}

/// Event fired when filesystem changes are detected
#[derive(Event, Debug)]
pub struct FileSystemChanged {
    pub changes: Vec<FileSystemChange>,
}

/// Bevy Events for filesystem requests following ARCHITECTURE.md pattern  
#[derive(Event, Debug)]
pub enum FileSystemRequest {
    ReadFile {
        operation_id: FileOperationId,
        requester: Entity,
        path: PathBuf,
    },
    WriteFile {
        operation_id: FileOperationId,
        requester: Entity,
        path: PathBuf,
        content: Vec<u8>,
    },
    ListDirectory {
        operation_id: FileOperationId,
        requester: Entity,
        path: PathBuf,
    },
    WatchDirectory {
        operation_id: FileOperationId,
        requester: Entity,
        path: PathBuf,
        config: Box<crate::types::WatchConfig>,
    },
    CheckPermissions {
        operation_id: FileOperationId,
        requester: Entity,
        path: PathBuf,
    },
    CreateDirectory {
        operation_id: FileOperationId,
        requester: Entity,
        path: PathBuf,
    },
}

/// Event fired when filesystem operation fails with context
#[derive(Event, Debug)]
pub struct FileSystemOperationFailed {
    pub operation_id: FileOperationId,
    pub requester: Entity,
    pub error: FileSystemError,
    pub retry_count: u32,
}

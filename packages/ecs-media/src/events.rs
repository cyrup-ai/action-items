use bevy::prelude::*;
use std::path::PathBuf;
use crate::types::{Media, MediaError, MediaId};

/// Media operation requests (trigger async tasks)
#[derive(Event, Debug)]
pub enum MediaRequest {
    /// Upload new media file
    UploadMedia {
        operation_id: MediaId,
        user_id: String,
        conversation_id: Option<String>,
        file_path: PathBuf,
        description: Option<String>,
        requester: Entity,
    },
    
    /// Get media by ID
    GetMedia {
        operation_id: MediaId,
        media_id: MediaId,
        requester: Entity,
    },
    
    /// Update media description
    UpdateDescription {
        operation_id: MediaId,
        media_id: MediaId,
        description: Option<String>,
        requester: Entity,
    },
    
    /// Delete media (file + metadata)
    DeleteMedia {
        operation_id: MediaId,
        media_id: MediaId,
        requester: Entity,
    },
}

/// Media operation responses
#[derive(Event, Debug)]
pub enum MediaResponse {
    UploadComplete {
        operation_id: MediaId,
        requester: Entity,
        result: Result<Media, MediaError>,
    },
    
    MediaRetrieved {
        operation_id: MediaId,
        requester: Entity,
        result: Result<Option<Media>, MediaError>,
    },
    
    DescriptionUpdated {
        operation_id: MediaId,
        requester: Entity,
        result: Result<(), MediaError>,
    },
    
    MediaDeleted {
        operation_id: MediaId,
        requester: Entity,
        result: Result<(), MediaError>,
    },
}

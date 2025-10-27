use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use uuid::Uuid;

/// Unique identifier for media records
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MediaId(pub Uuid);

impl MediaId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Convert to SurrealDB RecordId for database operations
    pub fn to_record_id(&self) -> RecordId {
        RecordId::from(("media", self.0.to_string()))
    }
}

impl Default for MediaId {
    fn default() -> Self {
        Self::new()
    }
}

/// Media metadata record (matches SurrealDB schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub id: MediaId,
    pub user_id: String,
    pub conversation_id: Option<String>,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_seconds: Option<f32>,
    pub description: Option<String>,
    pub storage_path: String,
    pub thumbnail_path: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Media operation errors
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("Media not found: {id:?}")]
    NotFound { id: MediaId },
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Invalid file type: {mime_type}")]
    InvalidFileType { mime_type: String },
    
    #[error("File too large: {size} bytes (limit: {limit})")]
    FileTooLarge { size: u64, limit: u64 },
}

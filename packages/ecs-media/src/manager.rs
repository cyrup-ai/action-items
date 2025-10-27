use crate::types::{Media, MediaError, MediaId};
use crate::schema::MEDIA_SCHEMA;
use std::path::{Path, PathBuf};
use action_items_ecs_surrealdb::DatabaseService;

pub struct MediaManager {
    storage_base_path: PathBuf,
    max_file_size: u64,
}

impl MediaManager {
    pub fn new(storage_base_path: PathBuf, max_file_size: u64) -> Self {
        Self {
            storage_base_path,
            max_file_size,
        }
    }
    
    /// Initialize database schema (called on startup)
    pub async fn initialize_schema(db: &DatabaseService) -> Result<(), MediaError> {
        db.query(MEDIA_SCHEMA)
            .await
            .map_err(|e| MediaError::DatabaseError(e.to_string()))?;
        Ok(())
    }
    
    /// Upload media file and store metadata
    pub async fn upload_media(
        &self,
        db: &DatabaseService,
        media_id: MediaId,
        user_id: String,
        conversation_id: Option<String>,
        file_path: &Path,
        description: Option<String>,
    ) -> Result<Media, MediaError> {
        // Get file metadata from filesystem
        let file_meta = std::fs::metadata(file_path)
            .map_err(|e| MediaError::StorageError(e.to_string()))?;
        
        // Validate file size
        let size_bytes = file_meta.len();
        if size_bytes > self.max_file_size {
            return Err(MediaError::FileTooLarge {
                size: size_bytes,
                limit: self.max_file_size,
            });
        }
        
        // Generate storage path
        let filename = file_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| MediaError::StorageError("Invalid filename".into()))?
            .to_string();
        
        let storage_path = self.storage_base_path
            .join(media_id.0.to_string())
            .join(&filename);
        
        // Copy file to storage using ecs-filesystem
        let parent_path = storage_path.parent()
            .ok_or_else(|| MediaError::StorageError("Invalid storage path".into()))?;
        std::fs::create_dir_all(parent_path)
            .map_err(|e| MediaError::StorageError(e.to_string()))?;
        std::fs::copy(file_path, &storage_path)
            .map_err(|e| MediaError::StorageError(e.to_string()))?;
        
        // Detect MIME type
        let mime_type = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string();
        
        // Create media record
        let media = Media {
            id: media_id,
            user_id,
            conversation_id,
            filename,
            mime_type,
            size_bytes,
            width: None,  // TODO: Extract from image in Part B
            height: None,
            duration_seconds: None,
            description,
            storage_path: storage_path.to_string_lossy().into_owned(),
            thumbnail_path: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // Clone media for return before inserting
        let media_clone = media.clone();
        
        // Insert into database (consumes media)
        let _record_id = db.create("media", media)
            .await
            .map_err(|e| MediaError::DatabaseError(e.to_string()))?;
        
        Ok(media_clone)
    }
    
    /// Get media by ID
    pub async fn get_media(
        &self,
        db: &DatabaseService,
        media_id: &MediaId,
    ) -> Result<Option<Media>, MediaError> {
        let record_id = media_id.to_record_id();
        
        // Query from database
        let result: Vec<Media> = db.select(&record_id.to_string())
            .await
            .map_err(|e| MediaError::DatabaseError(e.to_string()))?;
        
        Ok(result.into_iter().next())
    }
    
    /// Update media description
    pub async fn update_description(
        &self,
        db: &DatabaseService,
        media_id: &MediaId,
        description: Option<String>,
    ) -> Result<(), MediaError> {
        let record_id = media_id.to_record_id();
        
        // Update in database using direct query
        let query = if let Some(desc) = description {
            format!(
                "UPDATE {} SET description = '{}', updated_at = time::now()",
                record_id.to_string(),
                desc.replace('\'', "\\'")
            )
        } else {
            format!(
                "UPDATE {} SET description = NONE, updated_at = time::now()",
                record_id.to_string()
            )
        };
        
        db.query(&query)
            .await
            .map_err(|e| MediaError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Delete media (file + metadata)
    pub async fn delete_media(
        &self,
        db: &DatabaseService,
        media_id: &MediaId,
    ) -> Result<(), MediaError> {
        // Get media to find storage path
        let media = self.get_media(db, media_id).await?
            .ok_or_else(|| MediaError::NotFound { id: *media_id })?;
        
        // Store paths before consuming media
        let storage_path = media.storage_path.clone();
        let thumbnail_path = media.thumbnail_path.clone();
        
        // Delete file from storage
        std::fs::remove_file(&storage_path)
            .map_err(|e| MediaError::StorageError(e.to_string()))?;
        
        // Delete thumbnail if exists
        if let Some(thumb_path) = thumbnail_path {
            let _ = std::fs::remove_file(thumb_path); // Ignore errors
        }
        
        // Delete from database
        let record_id = media_id.to_record_id();
        db.delete(&record_id)
            .await
            .map_err(|e| MediaError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
}

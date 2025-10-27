use crate::types::{Media, MediaError, MediaId};
use crate::schema::MEDIA_SCHEMA;
use std::path::{Path, PathBuf};
use action_items_ecs_surrealdb::DatabaseService;
use image::{ImageReader, GenericImageView};
use image::imageops::FilterType;

const THUMBNAIL_MAX_DIMENSION: u32 = 256;

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
    
    /// Generate thumbnail for image file
    /// 
    /// Returns: (thumbnail_storage_path, width, height)
    pub async fn generate_thumbnail(
        &self,
        source_path: &Path,
        media_id: &MediaId,
    ) -> Result<(String, u32, u32), MediaError> {
        // Read original image
        let img = ImageReader::open(source_path)
            .map_err(|e| MediaError::StorageError(format!("Failed to open image: {}", e)))?
            .decode()
            .map_err(|e| MediaError::StorageError(format!("Failed to decode image: {}", e)))?;
        
        // Get original dimensions
        let (orig_width, orig_height) = img.dimensions();
        
        // Calculate thumbnail dimensions maintaining aspect ratio
        let (thumb_width, thumb_height) = if orig_width > orig_height {
            let ratio = THUMBNAIL_MAX_DIMENSION as f32 / orig_width as f32;
            (THUMBNAIL_MAX_DIMENSION, (orig_height as f32 * ratio) as u32)
        } else {
            let ratio = THUMBNAIL_MAX_DIMENSION as f32 / orig_height as f32;
            ((orig_width as f32 * ratio) as u32, THUMBNAIL_MAX_DIMENSION)
        };
        
        // Resize image with high-quality Lanczos3 filter
        let thumbnail = img.resize(thumb_width, thumb_height, FilterType::Lanczos3);
        
        // Generate thumbnail storage path
        let thumb_filename = format!("{}_thumb.jpg", media_id.0);
        let thumb_storage_path = self.storage_base_path
            .join(media_id.0.to_string())
            .join(&thumb_filename);
        
        // Save thumbnail as JPEG (universal compatibility)
        thumbnail.save_with_format(&thumb_storage_path, image::ImageFormat::Jpeg)
            .map_err(|e| MediaError::StorageError(format!("Failed to save thumbnail: {}", e)))?;
        
        Ok((thumb_storage_path.to_string_lossy().to_string(), orig_width, orig_height))
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
        
        // Validate MIME type against allowlist
        use crate::types::is_allowed_mime_type;
        if !is_allowed_mime_type(&mime_type) {
            return Err(MediaError::InvalidFileType {
                mime_type: mime_type.clone(),
            });
        }
        
        // Generate thumbnail and extract dimensions for images
        let (thumbnail_path, width, height) = if mime_type.starts_with("image/") {
            match self.generate_thumbnail(&storage_path, &media_id).await {
                Ok((thumb_path, w, h)) => (Some(thumb_path), Some(w), Some(h)),
                Err(e) => {
                    // Log warning but don't fail upload
                    eprintln!("Failed to generate thumbnail for {}: {}", media_id.0, e);
                    (None, None, None)
                }
            }
        } else {
            (None, None, None)
        };
        
        // Create media record with thumbnail and dimensions
        let media = Media {
            id: media_id,
            user_id,
            conversation_id,
            filename,
            mime_type,
            size_bytes,
            width,
            height,
            duration_seconds: None,  // Video duration extraction is out of scope
            description,
            storage_path: storage_path.to_string_lossy().into_owned(),
            thumbnail_path,
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
    
    /// List all media for a conversation
    pub async fn list_conversation_media(
        &self,
        db: &DatabaseService,
        conversation_id: &str,
    ) -> Result<Vec<Media>, MediaError> {
        // Query media by conversation_id
        let query = "SELECT * FROM media WHERE conversation_id = $conv_id ORDER BY created_at DESC";
        
        let mut params = std::collections::HashMap::new();
        params.insert("conv_id".to_string(), surrealdb::Value::from(conversation_id.to_string()));
        
        let mut result = db.query_with_params(query, params)
            .await
            .map_err(|e| MediaError::DatabaseError(e.to_string()))?;
        
        // Extract results from query response
        let media_list: Vec<Media> = result.take(0)
            .map_err(|e| MediaError::DatabaseError(e.to_string()))?;
        
        Ok(media_list)
    }
}

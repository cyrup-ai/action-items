use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use blake3::Hasher;
use dashmap::DashMap;
#[cfg(feature = "performance")]
use memmap2::MmapOptions;
use smallvec::SmallVec;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::security::{PathValidator, SecurityConfig};
use crate::types::{
    DirectoryListing, FileContent, FileMetadata, FileOperationId, FilePermissions, FileSystemError,
};

/// High-performance filesystem manager with caching and security
pub struct FileSystemManager {
    validator: PathValidator,
    /// Cache for file metadata (path -> (metadata, timestamp))
    metadata_cache: DashMap<PathBuf, (FileMetadata, Instant)>,
    /// Cache for small file contents (path -> (content, timestamp))  
    content_cache: DashMap<PathBuf, (Vec<u8>, Instant)>,
    /// Cache TTL for metadata and content
    cache_ttl: Duration,
    /// Maximum size for content caching (files larger than this use memory mapping)
    max_cache_size: u64,
}

impl FileSystemManager {
    pub fn new(security_config: SecurityConfig) -> Self {
        Self {
            validator: PathValidator::new(security_config),
            metadata_cache: DashMap::new(),
            content_cache: DashMap::new(),
            cache_ttl: Duration::from_secs(30),
            max_cache_size: 1024 * 1024, // 1MB cache limit for small files
        }
    }

    /// Read file with optimal performance strategy based on size
    pub async fn read_file(
        &self,
        operation_id: FileOperationId,
        path: &Path,
    ) -> Result<FileContent, FileSystemError> {
        // Validate path security
        let validated_path = self.validator.validate_path(path, "read")?;

        // Get file metadata first
        let metadata = self.get_metadata(&validated_path).await?;

        // Validate file size
        self.validator.validate_file_size(metadata.size)?;

        // Audit the operation
        self.validator
            .audit_log(operation_id, "read_file", &validated_path, false);

        // Choose read strategy based on file size
        let data = if metadata.size <= self.max_cache_size {
            self.read_small_file(&validated_path, &metadata).await?
        } else {
            self.read_large_file(&validated_path).await?
        };

        // Calculate checksum for integrity
        let checksum = self.calculate_checksum(&data);

        // Audit successful operation
        self.validator
            .audit_log(operation_id, "read_file", &validated_path, true);

        let encoding = self.detect_encoding(&data);
        Ok(FileContent {
            metadata,
            data,
            encoding,
            validated: true,
            checksum,
        })
    }
    /// Write file atomically with integrity checks
    pub async fn write_file(
        &self,
        operation_id: FileOperationId,
        path: &Path,
        content: &[u8],
        atomic: bool,
    ) -> Result<(), FileSystemError> {
        let validated_path = self.validator.validate_path(path, "write")?;

        self.validator
            .audit_log(operation_id, "write_file", &validated_path, false);

        if atomic {
            self.write_file_atomic(&validated_path, content).await?;
        } else {
            self.write_file_direct(&validated_path, content).await?;
        }

        // Invalidate caches for this path
        self.invalidate_cache(&validated_path);

        self.validator
            .audit_log(operation_id, "write_file", &validated_path, true);
        Ok(())
    }

    /// List directory with efficient caching
    pub async fn list_directory(
        &self,
        operation_id: FileOperationId,
        path: &Path,
        recursive: bool,
    ) -> Result<DirectoryListing, FileSystemError> {
        let validated_path = self.validator.validate_path(path, "list")?;

        self.validator
            .audit_log(operation_id, "list_directory", &validated_path, false);

        let start_time = Instant::now();
        let mut entries = SmallVec::new();
        let mut total_size = 0u64;

        if recursive {
            self.list_recursive(&validated_path, &mut entries, &mut total_size, 0)
                .await?;
        } else {
            self.list_single_directory(&validated_path, &mut entries, &mut total_size)
                .await?;
        }

        let scan_duration = start_time.elapsed();

        self.validator
            .audit_log(operation_id, "list_directory", &validated_path, true);

        let entry_count = entries.len();
        Ok(DirectoryListing {
            path: validated_path,
            entries,
            total_size,
            entry_count,
            scan_duration,
            recursive,
        })
    }
    /// Create directory with parent creation support
    pub async fn create_directory(
        &self,
        operation_id: FileOperationId,
        path: &Path,
        recursive: bool,
    ) -> Result<(), FileSystemError> {
        let validated_path = self.validator.validate_path(path, "create_directory")?;

        self.validator
            .audit_log(operation_id, "create_directory", &validated_path, false);

        if recursive {
            fs::create_dir_all(&validated_path).await
        } else {
            fs::create_dir(&validated_path).await
        }
        .map_err(|e| FileSystemError::Io { source: e })?;

        self.validator
            .audit_log(operation_id, "create_directory", &validated_path, true);
        Ok(())
    }

    /// Get file metadata with caching
    pub async fn get_metadata(&self, path: &Path) -> Result<FileMetadata, FileSystemError> {
        // Check cache first
        if let Some(entry) = self.metadata_cache.get(path) {
            let (metadata, timestamp) = entry.value();
            if timestamp.elapsed() < self.cache_ttl {
                return Ok(metadata.clone());
            }
        }

        let std_metadata = fs::metadata(path).await.map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => FileSystemError::NotFound {
                path: path.to_path_buf(),
            },
            std::io::ErrorKind::PermissionDenied => FileSystemError::AccessDenied {
                path: path.to_path_buf(),
            },
            _ => FileSystemError::Io { source: e },
        })?;

        let metadata = FileMetadata {
            path: path.to_path_buf(),
            size: std_metadata.len(),
            created: std_metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
            modified: std_metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            accessed: std_metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
            is_file: std_metadata.is_file(),
            is_dir: std_metadata.is_dir(),
            is_symlink: std_metadata.is_symlink(),
            permissions: self.extract_permissions(&std_metadata),
            checksum: None,
            mime_type: self.detect_mime_type(path),
        };

        // Cache the metadata
        self.metadata_cache
            .insert(path.to_path_buf(), (metadata.clone(), Instant::now()));

        Ok(metadata)
    }
    /// Read small files with caching
    async fn read_small_file(
        &self,
        path: &Path,
        metadata: &FileMetadata,
    ) -> Result<Vec<u8>, FileSystemError> {
        // Check content cache first
        if let Some(entry) = self.content_cache.get(path) {
            let (content, timestamp) = entry.value();
            if timestamp.elapsed() < self.cache_ttl {
                return Ok(content.clone());
            }
        }

        let mut file = fs::File::open(path)
            .await
            .map_err(|e| FileSystemError::Io { source: e })?;

        let mut buffer = Vec::with_capacity(metadata.size as usize);
        file.read_to_end(&mut buffer)
            .await
            .map_err(|e| FileSystemError::Io { source: e })?;

        // Cache the content
        self.content_cache
            .insert(path.to_path_buf(), (buffer.clone(), Instant::now()));

        Ok(buffer)
    }

    /// Read large files using memory mapping for performance
    async fn read_large_file(&self, path: &Path) -> Result<Vec<u8>, FileSystemError> {
        #[cfg(feature = "performance")]
        {
            // Use memory mapping for large files to avoid loading entire file into memory
            let file = std::fs::File::open(path).map_err(|e| FileSystemError::Io { source: e })?;

            let mmap = unsafe {
                MmapOptions::new()
                    .map(&file)
                    .map_err(|e| FileSystemError::Io { source: e })?
            };

            // Copy memory-mapped data to owned Vec
            Ok(mmap.to_vec())
        }

        #[cfg(not(feature = "performance"))]
        {
            // Fallback to regular file reading
            let mut file = fs::File::open(path)
                .await
                .map_err(|e| FileSystemError::Io { source: e })?;

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .await
                .map_err(|e| FileSystemError::Io { source: e })?;

            Ok(buffer)
        }
    }

    /// Write file atomically (write to temp file, then rename)
    async fn write_file_atomic(&self, path: &Path, content: &[u8]) -> Result<(), FileSystemError> {
        let temp_path = self.generate_temp_path(path);

        // Write to temporary file first
        let mut temp_file = fs::File::create(&temp_path)
            .await
            .map_err(|e| FileSystemError::Io { source: e })?;

        temp_file
            .write_all(content)
            .await
            .map_err(|e| FileSystemError::Io { source: e })?;

        temp_file
            .sync_all()
            .await
            .map_err(|e| FileSystemError::Io { source: e })?;

        // Atomically rename temp file to target path
        fs::rename(&temp_path, path)
            .await
            .map_err(|e| FileSystemError::Io { source: e })?;

        Ok(())
    }

    /// Write file directly
    async fn write_file_direct(&self, path: &Path, content: &[u8]) -> Result<(), FileSystemError> {
        let mut file = fs::File::create(path)
            .await
            .map_err(|e| FileSystemError::Io { source: e })?;

        file.write_all(content)
            .await
            .map_err(|e| FileSystemError::Io { source: e })?;

        file.sync_all()
            .await
            .map_err(|e| FileSystemError::Io { source: e })?;

        Ok(())
    }
    /// List single directory entries
    async fn list_single_directory(
        &self,
        path: &Path,
        entries: &mut SmallVec<[FileMetadata; 32]>,
        total_size: &mut u64,
    ) -> Result<(), FileSystemError> {
        let mut dir = fs::read_dir(path)
            .await
            .map_err(|e| FileSystemError::Io { source: e })?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| FileSystemError::Io { source: e })?
        {
            let metadata = self.get_metadata(&entry.path()).await?;
            *total_size += metadata.size;
            entries.push(metadata);
        }

        Ok(())
    }

    /// List directory recursively with depth control
    fn list_recursive<'a>(
        &'a self,
        path: &'a Path,
        entries: &'a mut SmallVec<[FileMetadata; 32]>,
        total_size: &'a mut u64,
        current_depth: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), FileSystemError>> + Send + 'a>>
    {
        Box::pin(async move {
            const MAX_DEPTH: usize = 10; // Prevent excessive recursion

            if current_depth >= MAX_DEPTH {
                return Ok(());
            }

            let mut dir = fs::read_dir(path)
                .await
                .map_err(|e| FileSystemError::Io { source: e })?;

            while let Some(entry) = dir
                .next_entry()
                .await
                .map_err(|e| FileSystemError::Io { source: e })?
            {
                let metadata = self.get_metadata(&entry.path()).await?;
                *total_size += metadata.size;
                entries.push(metadata.clone());

                if metadata.is_dir {
                    self.list_recursive(&entry.path(), entries, total_size, current_depth + 1)
                        .await?;
                }
            }

            Ok(())
        })
    }

    /// Generate temporary file path for atomic operations
    fn generate_temp_path(&self, path: &Path) -> PathBuf {
        let mut temp_name = path
            .file_name()
            .unwrap_or_else(|| "temp".as_ref())
            .to_os_string();
        temp_name.push(".tmp");

        if let Some(parent) = path.parent() {
            parent.join(temp_name)
        } else {
            PathBuf::from(temp_name)
        }
    }
    /// Extract cross-platform file permissions
    fn extract_permissions(&self, metadata: &std::fs::Metadata) -> FilePermissions {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();

            FilePermissions {
                readable: mode & 0o400 != 0,
                writable: mode & 0o200 != 0,
                executable: mode & 0o100 != 0,
                hidden: false, // Unix hidden files start with '.'
            }
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            let attributes = metadata.file_attributes();

            FilePermissions {
                readable: attributes & 0x1 == 0, // FILE_ATTRIBUTE_READONLY inverted
                writable: attributes & 0x1 == 0,
                executable: false, // Windows executability is by extension
                hidden: attributes & 0x2 != 0, // FILE_ATTRIBUTE_HIDDEN
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            FilePermissions {
                readable: !metadata.permissions().readonly(),
                writable: !metadata.permissions().readonly(),
                executable: false,
                hidden: false,
            }
        }
    }

    /// Detect MIME type based on file extension
    fn detect_mime_type(&self, path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "txt" => "text/plain",
                "json" => "application/json",
                "xml" => "application/xml",
                "html" | "htm" => "text/html",
                "css" => "text/css",
                "js" => "application/javascript",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "pdf" => "application/pdf",
                "zip" => "application/zip",
                _ => "application/octet-stream",
            })
            .map(String::from)
    }

    /// Calculate BLAKE3 checksum for integrity verification
    fn calculate_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize().to_hex().to_string()
    }
    /// Detect text encoding (basic detection)
    fn detect_encoding(&self, data: &[u8]) -> Option<String> {
        // Check for BOM markers
        if data.starts_with(&[0xef, 0xbb, 0xbf]) {
            return Some("UTF-8".to_string());
        }
        if data.starts_with(&[0xff, 0xfe]) {
            return Some("UTF-16LE".to_string());
        }
        if data.starts_with(&[0xfe, 0xff]) {
            return Some("UTF-16BE".to_string());
        }

        // Basic UTF-8 validation
        if std::str::from_utf8(data).is_ok() {
            Some("UTF-8".to_string())
        } else {
            Some("binary".to_string())
        }
    }

    /// Invalidate cache entries for a path
    fn invalidate_cache(&self, path: &Path) {
        self.metadata_cache.remove(path);
        self.content_cache.remove(path);

        // Also invalidate parent directory cache if it exists
        if let Some(parent) = path.parent() {
            self.metadata_cache.remove(parent);
        }
    }

    /// Clear expired cache entries
    pub fn cleanup_cache(&self) {
        let _now = Instant::now();

        self.metadata_cache
            .retain(|_, (_, timestamp)| timestamp.elapsed() < self.cache_ttl);

        self.content_cache
            .retain(|_, (_, timestamp)| timestamp.elapsed() < self.cache_ttl);
    }
}

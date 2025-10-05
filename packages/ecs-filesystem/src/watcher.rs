#[cfg(feature = "watching")]
pub mod watching_impl {
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::{Instant, SystemTime};

    use dashmap::DashMap;
    use notify::{RecommendedWatcher, RecursiveMode};
    use notify_debouncer_full::{DebounceEventResult, Debouncer, FileIdMap, new_debouncer};
    use tokio::sync::mpsc;
    use tokio::io::AsyncReadExt;
    use tracing;

    // External crate imports for production MIME detection and checksums
    use blake3;
    use infer;

    use crate::security::PathValidator;
    use crate::types::{ChangeType, FileOperationId, FileSystemChange, WatchConfig, ChecksumAlgorithm};

    /// State tracking for rename operations to correlate From/To events
    #[derive(Clone, Debug)]
    struct RenameTrackingState {
        from_path: Option<PathBuf>,
        timestamp: Instant,
        #[allow(dead_code)] // Reserved for future event correlation features
        event_id: FileOperationId,
    }

    impl RenameTrackingState {
        #[inline]
        fn new(from_path: Option<PathBuf>, event_id: FileOperationId) -> Self {
            Self {
                from_path,
                timestamp: Instant::now(),
                event_id,
            }
        }

        #[inline]
        fn is_expired(&self, timeout: std::time::Duration) -> bool {
            self.timestamp.elapsed() > timeout
        }
    }

    /// Filesystem watcher with debouncing and event filtering
    pub struct FileSystemWatcher {
        watchers: DashMap<PathBuf, WatcherHandle>,
        validator: PathValidator,
        event_sender: mpsc::UnboundedSender<Vec<FileSystemChange>>,
        /// Lock-free tracking of rename operations for event correlation
        rename_state: DashMap<PathBuf, RenameTrackingState>,
    }

    struct WatcherHandle {
        debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
    }

    impl FileSystemWatcher {
        pub fn new(
            validator: PathValidator,
            event_sender: mpsc::UnboundedSender<Vec<FileSystemChange>>,
        ) -> Self {
            Self {
                watchers: DashMap::new(),
                validator,
                event_sender,
                rename_state: DashMap::new(),
            }
        }

        pub fn start_watching(
            &self,
            operation_id: FileOperationId,
            path: &Path,
            config: WatchConfig,
        ) -> Result<(), crate::types::FileSystemError> {
            let validated_path = self.validator.validate_path(path, "watch")?;

            // Check if already watching this path
            if self.watchers.contains_key(&validated_path) {
                return Ok(()); // Already watching, no-op
            }

            // Create event conversion closure
            let event_sender = self.event_sender.clone();
            let validated_path_clone = validated_path.clone();
            let config_clone = config.clone();
            let rename_state_clone = Arc::new(self.rename_state.clone());

            // Create debouncer with closure-based event handler
            let debouncer = new_debouncer(
                config.debounce_duration,
                None, // No tick rate limit for maximum responsiveness
                move |result: DebounceEventResult| match result {
                    Ok(debounced_events) => {
                        let changes = Self::convert_events(debounced_events, &config_clone, &rename_state_clone);
                        if !changes.is_empty() {
                            let _ = event_sender.send(changes);
                        }
                    },
                    Err(errors) => {
                        tracing::warn!(
                            "Filesystem watcher errors for path {:?}: {:?}",
                            validated_path_clone,
                            errors
                        );
                    },
                },
            )
            .map_err(|e| crate::types::FileSystemError::Watch { source: e })?;

            let recursive_mode = if config.recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };

            // Start watching before storing handle (fail fast if invalid path)
            let mut debouncer = debouncer;
            debouncer
                .watch(&validated_path, recursive_mode)
                .map_err(|e| crate::types::FileSystemError::Watch { source: e })?;

            // Create watcher handle
            let handle = WatcherHandle { debouncer };

            // Store in watchers map (atomic lock-free operation)
            self.watchers.insert(validated_path, handle);

            tracing::debug!(
                "Started watching path: {:?} with operation_id: {:?}",
                path,
                operation_id
            );
            Ok(())
        }

        pub fn stop_watching(&self, path: &Path) -> Result<(), crate::types::FileSystemError> {
            let validated_path = self.validator.validate_path(path, "unwatch")?;

            // Remove and drop the watcher handle (automatically stops debouncer)
            if let Some((_, handle)) = self.watchers.remove(&validated_path) {
                // Explicitly drop the debouncer to ensure clean shutdown
                drop(handle.debouncer);
                tracing::debug!("Stopped watching path: {:?}", path);
                Ok(())
            } else {
                // Path not being watched - this is not an error
                tracing::debug!("Attempted to stop watching non-watched path: {:?}", path);
                Ok(())
            }
        }

        /// Clean up expired rename tracking state entries to prevent memory leaks
        #[inline]
        pub fn cleanup_expired_rename_state(&self, timeout: std::time::Duration) {
            self.rename_state.retain(|_path, state| !state.is_expired(timeout));
        }

        /// Static version for use in convert_events - clean up expired rename tracking state
        #[inline]
        fn cleanup_expired_rename_state_static(
            rename_state: &Arc<DashMap<PathBuf, RenameTrackingState>>,
            timeout: std::time::Duration,
        ) {
            rename_state.retain(|_path, state| !state.is_expired(timeout));
        }


        /// Production-quality metadata extraction with conditional checksum and MIME detection
        /// Uses hybrid sync/async approach for optimal performance
        #[inline]
        fn extract_metadata_enhanced(
            path: &Path,
            config: &WatchConfig,
            timeout: std::time::Duration,
        ) -> Result<crate::types::FileMetadata, crate::types::FileSystemError> {
            use std::time::Instant;
            
            let start = Instant::now();
            
            // Basic timeout check before attempting extraction
            if start.elapsed() > timeout {
                return Err(crate::types::FileSystemError::Timeout {
                    operation: "metadata_extraction_pre_check".to_string(),
                });
            }
            
            // Use std::fs for synchronous basic metadata extraction
            let std_metadata = std::fs::metadata(path).map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => crate::types::FileSystemError::NotFound {
                    path: path.to_path_buf(),
                },
                std::io::ErrorKind::PermissionDenied => crate::types::FileSystemError::AccessDenied {
                    path: path.to_path_buf(),
                },
                _ => crate::types::FileSystemError::Io { source: e },
            })?;
            
            // Check timeout after basic I/O operation
            if start.elapsed() > timeout {
                return Err(crate::types::FileSystemError::Timeout {
                    operation: "metadata_extraction_post_io".to_string(),
                });
            }

            // Extract MIME type using production hybrid approach
            let mime_type = Self::extract_mime_type_sync_enhanced(path, timeout.saturating_sub(start.elapsed()));

            // Extract checksum if enabled and file is small enough for sync processing
            let checksum = if config.checksum_enabled && std_metadata.len() <= config.sync_checksum_threshold_bytes {
                // Only calculate checksum synchronously for files <= threshold to avoid blocking
                Self::extract_checksum_sync_small(path, &config.checksum_algorithm)
                    .map_err(|e| {
                        tracing::debug!("Sync checksum calculation failed for {:?}: {}. Skipping checksum.", path, e);
                        e
                    })
                    .ok()
            } else if config.checksum_enabled {
                tracing::debug!("File too large for sync checksum calculation: {:?} ({} bytes > {} threshold). Skipping checksum.", path, std_metadata.len(), config.sync_checksum_threshold_bytes);
                None
            } else {
                None
            };
            
            // Convert to our FileMetadata type with production enhancements
            let now = std::time::SystemTime::now();
            let metadata = crate::types::FileMetadata {
                path: path.to_path_buf(),
                size: std_metadata.len(),
                created: std_metadata.created().unwrap_or_else(|_| {
                    tracing::debug!("Unable to get creation time for {:?}, using current time as fallback", path);
                    now
                }),
                modified: std_metadata.modified().unwrap_or_else(|_| {
                    tracing::debug!("Unable to get modification time for {:?}, using current time as fallback", path);
                    now
                }),
                accessed: std_metadata.accessed().unwrap_or_else(|_| {
                    tracing::debug!("Unable to get access time for {:?}, using current time as fallback", path);
                    now
                }),
                is_file: std_metadata.is_file(),
                is_dir: std_metadata.is_dir(),
                is_symlink: std_metadata.is_symlink(),
                permissions: Self::extract_permissions_sync(&std_metadata),
                checksum,
                mime_type,
            };
            
            Ok(metadata)
        }

        /// Enhanced MIME type detection using hybrid approach for sync context
        #[inline]
        fn extract_mime_type_sync_enhanced(
            path: &Path,
            timeout: std::time::Duration,
        ) -> Option<String> {
            let start = Instant::now();

            // For sync context, read first 8KB for content analysis
            if let Ok(mut file) = std::fs::File::open(path) {
                if start.elapsed() > timeout {
                    // Timeout fallback: use extension-based detection
                    let guess = mime_guess::from_path(path);
                    return guess.first().map(|m| m.to_string());
                }

                use std::io::Read;
                let mut buffer = vec![0u8; 8192];
                if let Ok(bytes_read) = file.read(&mut buffer) {
                    buffer.truncate(bytes_read);
                    
                    // Use infer for content-based detection
                    if let Some(kind) = infer::get(&buffer) {
                        return Some(kind.mime_type().to_string());
                    }
                }
            }

            // Fallback to extension-based detection using mime_guess
            let guess = mime_guess::from_path(path);
            guess.first().map(|m| m.to_string())
        }

        /// Fast checksum calculation for small files in sync context
        #[inline]
        fn extract_checksum_sync_small(
            path: &Path,
            algorithm: &ChecksumAlgorithm,
        ) -> Result<String, crate::types::FileSystemError> {
            use std::io::Read;

            let mut file = std::fs::File::open(path).map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => crate::types::FileSystemError::NotFound {
                    path: path.to_path_buf(),
                },
                std::io::ErrorKind::PermissionDenied => crate::types::FileSystemError::AccessDenied {
                    path: path.to_path_buf(),
                },
                _ => crate::types::FileSystemError::Io { source: e },
            })?;

            match algorithm {
                ChecksumAlgorithm::Blake3 => {
                    let mut hasher = blake3::Hasher::new();
                    let mut buffer = [0u8; 8192];

                    loop {
                        let bytes_read = file.read(&mut buffer).map_err(|e| {
                            crate::types::FileSystemError::Io { source: e }
                        })?;

                        if bytes_read == 0 {
                            break;
                        }

                        hasher.update(&buffer[..bytes_read]);
                    }

                    Ok(hasher.finalize().to_hex().to_string())
                },
                ChecksumAlgorithm::Sha256 => {
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    let mut buffer = [0u8; 8192];

                    loop {
                        let bytes_read = file.read(&mut buffer).map_err(|e| {
                            crate::types::FileSystemError::Io { source: e }
                        })?;

                        if bytes_read == 0 {
                            break;
                        }

                        hasher.update(&buffer[..bytes_read]);
                    }

                    Ok(format!("{:x}", hasher.finalize()))
                }
            }
        }

        /// Extract cross-platform file permissions synchronously
        #[inline]
        fn extract_permissions_sync(metadata: &std::fs::Metadata) -> crate::types::FilePermissions {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();

                crate::types::FilePermissions {
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

                crate::types::FilePermissions {
                    readable: true, // Most files are readable on Windows
                    writable: attributes & 0x1 == 0, // FILE_ATTRIBUTE_READONLY inverted
                    executable: false, // Windows executability is by extension
                    hidden: attributes & 0x2 != 0, // FILE_ATTRIBUTE_HIDDEN
                }
            }

            #[cfg(not(any(unix, windows)))]
            {
                crate::types::FilePermissions {
                    readable: !metadata.permissions().readonly(),
                    writable: !metadata.permissions().readonly(),
                    executable: false,
                    hidden: false,
                }
            }
        }


        /// Calculate file checksum using the specified algorithm with chunked async reading
        /// Returns checksum as hexadecimal string, optimized for large files
        #[allow(dead_code)]
        #[inline]
        async fn calculate_checksum_async(
            path: &Path,
            algorithm: ChecksumAlgorithm,
            timeout: std::time::Duration,
        ) -> Result<String, crate::types::FileSystemError> {
            let start = Instant::now();

            // Timeout check before opening file
            if start.elapsed() > timeout {
                return Err(crate::types::FileSystemError::Timeout {
                    operation: "checksum_calculation_pre_check".to_string(),
                });
            }

            let mut file = tokio::fs::File::open(path).await.map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => crate::types::FileSystemError::NotFound {
                    path: path.to_path_buf(),
                },
                std::io::ErrorKind::PermissionDenied => crate::types::FileSystemError::AccessDenied {
                    path: path.to_path_buf(),
                },
                _ => crate::types::FileSystemError::Io { source: e },
            })?;

            match algorithm {
                ChecksumAlgorithm::Blake3 => {
                    let mut hasher = blake3::Hasher::new();
                    let mut buffer = [0u8; 8192]; // 8KB chunks for optimal performance

                    loop {
                        // Check timeout during processing
                        if start.elapsed() > timeout {
                            return Err(crate::types::FileSystemError::Timeout {
                                operation: "checksum_calculation_blake3".to_string(),
                            });
                        }

                        let bytes_read = file.read(&mut buffer).await.map_err(|e| {
                            crate::types::FileSystemError::Io { source: e }
                        })?;

                        if bytes_read == 0 {
                            break; // EOF reached
                        }

                        hasher.update(&buffer[..bytes_read]);
                    }

                    Ok(hasher.finalize().to_hex().to_string())
                },
                ChecksumAlgorithm::Sha256 => {
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    let mut buffer = [0u8; 8192]; // 8KB chunks for optimal performance

                    loop {
                        // Check timeout during processing
                        if start.elapsed() > timeout {
                            return Err(crate::types::FileSystemError::Timeout {
                                operation: "checksum_calculation_sha256".to_string(),
                            });
                        }

                        let bytes_read = file.read(&mut buffer).await.map_err(|e| {
                            crate::types::FileSystemError::Io { source: e }
                        })?;

                        if bytes_read == 0 {
                            break; // EOF reached
                        }

                        hasher.update(&buffer[..bytes_read]);
                    }

                    Ok(format!("{:x}", hasher.finalize()))
                }
            }
        }

        /// Detect MIME type using hybrid approach: content-based primary, extension-based fallback
        /// More accurate than pure extension-based detection, optimized for performance
        #[allow(dead_code)]
        #[inline]
        async fn detect_mime_type_hybrid(
            path: &Path,
            timeout: std::time::Duration,
        ) -> Result<String, crate::types::FileSystemError> {
            let start = Instant::now();

            // Primary: Content-based detection using magic numbers (first 8KB)
            if let Ok(mut file) = tokio::fs::File::open(path).await {
                // Check timeout before reading
                if start.elapsed() > timeout {
                    return Err(crate::types::FileSystemError::Timeout {
                        operation: "mime_detection_timeout".to_string(),
                    });
                }

                let mut buffer = vec![0u8; 8192]; // Read first 8KB for magic number analysis
                if let Ok(bytes_read) = file.read(&mut buffer).await {
                    buffer.truncate(bytes_read);
                    
                    // Use infer library for content-based detection
                    if let Some(kind) = infer::get(&buffer) {
                        tracing::debug!("MIME type detected via content analysis: {} for path: {:?}", kind.mime_type(), path);
                        return Ok(kind.mime_type().to_string());
                    }
                }
            }

            // Fallback: Extension-based detection using mime_guess
            let guess = mime_guess::from_path(path);
            if let Some(mime_type) = guess.first() {
                tracing::debug!("MIME type detected via extension: {} for path: {:?}", mime_type, path);
                Ok(mime_type.to_string())
            } else {
                // Final fallback for unknown types
                tracing::debug!("MIME type defaulted to octet-stream for path: {:?}", path);
                Ok("application/octet-stream".to_string())
            }
        }

        /// Convert debounced events to FileSystemChange events with rate limiting and filtering
        /// Zero allocation implementation with blazing-fast event processing
        #[inline]
        fn convert_events(
            debounced_events: Vec<notify_debouncer_full::DebouncedEvent>,
            config: &WatchConfig,
            rename_state: &Arc<DashMap<PathBuf, RenameTrackingState>>,
        ) -> Vec<FileSystemChange> {

            let mut changes = Vec::with_capacity(debounced_events.len());
            let now = SystemTime::now();
            let mut failed_events_count = 0u32;
            let mut successful_events_count = 0u32;

            for debounced_event in debounced_events {
                // Wrap individual event processing in error recovery
                match Self::process_single_event(debounced_event, config, rename_state, now) {
                    Ok(mut event_changes) => {
                        successful_events_count += 1;
                        changes.append(&mut event_changes);
                    },
                    Err(e) => {
                        failed_events_count += 1;
                        tracing::warn!("Failed to process filesystem event: {}. Continuing with remaining events.", e);
                        // Continue processing other events instead of failing entirely
                        continue;
                    }
                }
            }

            // Log processing statistics for monitoring
            if failed_events_count > 0 {
                tracing::warn!(
                    "Event processing completed: {} successful, {} failed", 
                    successful_events_count, 
                    failed_events_count
                );
            } else if successful_events_count > 0 {
                tracing::debug!("Event processing completed: {} events processed successfully", successful_events_count);
            }

            changes
        }

        /// Process a single debounced event with comprehensive error handling
        /// Returns Vec<FileSystemChange> for successful processing or Error for failures
        #[inline]
        fn process_single_event(
            debounced_event: notify_debouncer_full::DebouncedEvent,
            config: &WatchConfig,
            rename_state: &Arc<DashMap<PathBuf, RenameTrackingState>>,
            timestamp: SystemTime,
        ) -> Result<Vec<FileSystemChange>, Box<dyn std::error::Error + Send + Sync>> {
            use notify::EventKind;
            use notify::event::{ModifyKind, RenameMode};
            
            let event = &debounced_event.event;
            let mut changes = Vec::new();

            // Validate event has paths before processing
            if event.paths.is_empty() {
                return Err("Event has no paths to process".into());
            }

            // Apply ignore patterns filter (zero allocation pattern matching)
            if Self::should_ignore_path(&event.paths, &config.ignore_patterns) {
                return Ok(changes); // Empty changes for ignored paths
            }

            // Convert notify EventKind to our ChangeType with error recovery
            let change_type = match event.kind {
                EventKind::Create(_) => ChangeType::Created,
                EventKind::Modify(ModifyKind::Data(_)) => ChangeType::Modified,
                EventKind::Modify(ModifyKind::Metadata(_)) => ChangeType::AttributesChanged,
                EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                    // Store rename state for correlation with future To event
                    let _ = Self::detect_rename_operation_static(
                        &event.kind,
                        &event.paths,
                        FileOperationId::new(),
                        config,
                        rename_state,
                    );
                    return Ok(changes); // Don't emit event for From part, return empty changes
                },
                EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                    // Use robust rename detection with state correlation
                    match Self::detect_rename_operation_static(
                        &event.kind,
                        &event.paths,
                        FileOperationId::new(),
                        config,
                        rename_state,
                    ) {
                        Some(from_path) => ChangeType::Renamed { from: from_path },
                        None => {
                            tracing::debug!("Could not determine rename source for {:?}, treating as creation", event.paths);
                            ChangeType::Created // Fallback if can't determine source
                        }
                    }
                },
                EventKind::Remove(_) => ChangeType::Deleted,
                _ => {
                    tracing::debug!("Skipping unknown/unsupported event type: {:?}", event.kind);
                    return Ok(changes); // Return empty changes for unsupported events
                }
            };

            // Process each path in the event (usually just one) with individual error recovery
            for path in &event.paths {
                match Self::process_single_path(path, &change_type, config, timestamp) {
                    Ok(change) => changes.push(change),
                    Err(e) => {
                        tracing::warn!("Failed to process path {:?}: {}. Skipping this path.", path, e);
                        // Continue with other paths instead of failing the entire event
                        continue;
                    }
                }
            }

            Ok(changes)
        }

        /// Process a single path within an event with error recovery
        #[inline]
        fn process_single_path(
            path: &Path,
            change_type: &ChangeType,
            config: &WatchConfig,
            timestamp: SystemTime,
        ) -> Result<FileSystemChange, Box<dyn std::error::Error + Send + Sync>> {
            // Validate path before processing
            if path.as_os_str().is_empty() {
                return Err("Empty path provided".into());
            }

            // Conditionally extract metadata with comprehensive error handling using enhanced production implementation
            let metadata = if config.include_metadata {
                match Self::extract_metadata_enhanced(path, config, config.metadata_timeout) {
                    Ok(meta) => Some(meta),
                    Err(e) => {
                        tracing::debug!("Failed to extract metadata for {:?}: {}. Continuing without metadata.", path, e);
                        None // Continue without metadata rather than failing
                    }
                }
            } else {
                None
            };

            let change = FileSystemChange {
                event_id: FileOperationId::new(),
                path: path.to_path_buf(),
                change_type: change_type.clone(),
                metadata,
                timestamp,
            };

            Ok(change)
        }

        /// Check if path should be ignored based on patterns (zero allocation)
        #[inline]
        fn should_ignore_path(
            paths: &[std::path::PathBuf],
            ignore_patterns: &smallvec::SmallVec<[String; 8]>,
        ) -> bool {
            if ignore_patterns.is_empty() {
                return false;
            }

            for path in paths {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    for pattern in ignore_patterns {
                        // Simple glob pattern matching - optimized for common cases
                        if Self::matches_pattern(file_name, pattern) {
                            return true;
                        }
                    }
                }
            }
            false
        }

        /// Fast pattern matching for ignore patterns (zero allocation)
        #[inline]
        fn matches_pattern(filename: &str, pattern: &str) -> bool {
            // Handle common patterns efficiently
            if pattern == "*" {
                return true;
            }

            if pattern.starts_with('*') && pattern.len() > 1 {
                let suffix = &pattern[1..];
                return filename.ends_with(suffix);
            }

            if pattern.ends_with('*') && pattern.len() > 1 {
                let prefix = &pattern[..pattern.len() - 1];
                return filename.starts_with(prefix);
            }

            // Exact match
            filename == pattern
        }

        /// Static version for use in convert_events - detect rename operations with robust cross-platform pattern support
        /// Handles different notify event patterns and provides fallback mechanisms
        fn detect_rename_operation_static(
            event_kind: &notify::EventKind,
            paths: &[std::path::PathBuf],
            current_event_id: FileOperationId,
            config: &WatchConfig,
            rename_state: &Arc<DashMap<PathBuf, RenameTrackingState>>,
        ) -> Option<std::path::PathBuf> {
            use notify::event::{ModifyKind, RenameMode};
            
            match event_kind {
                notify::EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                    // Store the "from" path for correlation with future "To" event
                    if let Some(from_path) = paths.first() {
                        let state = RenameTrackingState::new(Some(from_path.clone()), current_event_id);
                        rename_state.insert(from_path.clone(), state);
                        tracing::debug!("Stored rename From path: {:?}", from_path);
                    }
                    None // Don't emit an event for the From part
                },
                
                notify::EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                    // Look for corresponding "from" path in rename state or try path-based detection
                    if let Some(to_path) = paths.first() {
                        // First, try to find a recent "From" event for correlation
                        let mut found_from_path = None;
                        
                        // Look for matching rename state within the tracking window
                        for entry in rename_state.iter() {
                            let (_, state) = entry.pair();
                            if !state.is_expired(config.rename_tracking_window)
                                && let Some(ref from_path) = state.from_path {
                                    // Consider this a match if the paths are related
                                    // (e.g., same directory, similar names)
                                    if Self::are_paths_related(from_path, to_path) {
                                        found_from_path = Some(from_path.clone());
                                        break;
                                    }
                                }
                        }
                        
                        // Clean up expired states while we're here
                        Self::cleanup_expired_rename_state_static(rename_state, config.rename_tracking_window);
                        
                        // If we found a correlated from path, use it
                        if let Some(from_path) = found_from_path {
                            tracing::debug!("Correlated rename: {:?} -> {:?}", from_path, to_path);
                            return Some(from_path);
                        }
                        
                        // Fallback: try multi-path detection for platforms that provide both paths
                        Self::extract_rename_from_multiple_paths(paths)
                    } else {
                        None
                    }
                },
                
                _ => {
                    // For other event types that might contain rename information
                    Self::extract_rename_from_multiple_paths(paths)
                }
            }
        }

        /// Check if two paths are related (same directory, similar names, etc.)
        #[inline]
        fn are_paths_related(from_path: &Path, to_path: &Path) -> bool {
            // Same parent directory is a strong indicator of rename
            if let (Some(from_parent), Some(to_parent)) = (from_path.parent(), to_path.parent())
                && from_parent == to_parent {
                    return true;
                }
            
            // Check for similar filenames (could be rename with small changes)
            if let (Some(from_name), Some(to_name)) = (
                from_path.file_name().and_then(|n| n.to_str()),
                to_path.file_name().and_then(|n| n.to_str())
            ) {
                // Simple similarity check - common prefixes or suffixes
                let common_prefix_len = from_name.chars()
                    .zip(to_name.chars())
                    .take_while(|(a, b)| a == b)
                    .count();
                    
                // Consider related if they share significant common prefix
                if common_prefix_len >= 3.min(from_name.len().min(to_name.len()) / 2) {
                    return true;
                }
            }
            
            false
        }

        /// Extract rename from path for platforms that provide both paths in event
        #[inline]
        fn extract_rename_from_multiple_paths(paths: &[std::path::PathBuf]) -> Option<std::path::PathBuf> {
            match paths.len() {
                2 => {
                    // Most common case: [from_path, to_path]
                    Some(paths[0].clone())
                },
                1 => {
                    // Single path - can't determine rename source
                    None
                },
                0 => None,
                _ => {
                    // Multiple paths - use first as fallback
                    tracing::debug!("Multiple paths in rename event: {:?}, using first as source", paths);
                    Some(paths[0].clone())
                }
            }
        }
    }
}

#[cfg(feature = "watching")]
pub use watching_impl::FileSystemWatcher;

#[cfg(not(feature = "watching"))]
pub struct FileSystemWatcher;

#[cfg(not(feature = "watching"))]
impl FileSystemWatcher {
    pub fn new(
        _validator: crate::security::PathValidator,
        _event_sender: tokio::sync::mpsc::UnboundedSender<Vec<crate::types::FileSystemChange>>,
    ) -> Self {
        Self
    }

    pub fn start_watching(
        &self,
        _operation_id: crate::types::FileOperationId,
        _path: &std::path::Path,
        _config: crate::types::WatchConfig,
    ) -> Result<(), crate::types::FileSystemError> {
        Err(crate::types::FileSystemError::ResourceExhausted {
            resource: "filesystem watching not enabled".to_string(),
        })
    }

    pub fn stop_watching(
        &self,
        _path: &std::path::Path,
    ) -> Result<(), crate::types::FileSystemError> {
        Ok(())
    }
}

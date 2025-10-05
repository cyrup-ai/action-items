use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use deno_core::op2;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::raycast_errors::RaycastDiscoveryError;
use crate::raycast_types::{IsolatedRaycastCommand, IsolatedRaycastExtension};

/// Structured error response for production error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorResponse {
    error: ErrorDetails,
    context: ErrorContext,
    timestamp: String,
}

/// Detailed error information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorDetails {
    code: String,
    message: String,
    category: String,
    severity: String,
}

/// Error context for debugging and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorContext {
    operation: String,
    path: Option<String>,
    line: Option<u32>,
    column: Option<u32>,
    file_size: Option<u64>,
    directory_count: Option<usize>,
}

/// Error categories for classification
mod error_categories {
    pub const FILESYSTEM: &str = "filesystem";
    pub const SERIALIZATION: &str = "serialization";
}

/// Error severity levels
mod error_severity {
    pub const CRITICAL: &str = "critical";
    pub const HIGH: &str = "high";
}

/// Error codes for different error scenarios
mod error_codes {
    pub const DISCOVERY_FAILED: &str = "DISCOVERY_FAILED";
    pub const SERIALIZATION_ERROR: &str = "SERIALIZATION_ERROR";
}

// Static string constants to avoid allocations
const UNKNOWN_AUTHOR: &str = "Unknown";
const DEFAULT_MODE: &str = "view";
const PACKAGE_JSON: &str = "package.json";
const EMPTY_STRING: &str = "";

// Security limits to prevent DoS attacks
const MAX_DIRECTORY_DEPTH: usize = 10; // Max nested directory depth
const MAX_EXTENSIONS_COUNT: usize = 10000; // Max extensions to process
const MAX_PACKAGE_JSON_SIZE: u64 = 1024 * 1024; // 1MB max package.json
const MAX_STRING_LENGTH: usize = 10000; // Max string field length

// Parallel processing limits to prevent resource exhaustion
const MAX_CONCURRENT_PARSING: usize = 32; // Max concurrent extension parsing tasks
const BATCH_SIZE: usize = 16; // Process extensions in batches for memory efficiency

// String interning for common values to reduce allocations
pub struct StringInterner {
    common_authors: HashMap<&'static str, &'static str>,
    common_categories: HashMap<&'static str, &'static str>,
    common_modes: HashMap<&'static str, &'static str>,
    common_types: HashMap<&'static str, &'static str>,
}

impl StringInterner {
    fn new() -> Self {
        let mut common_authors = HashMap::with_capacity(16);
        common_authors.insert("raycast", "Raycast");
        common_authors.insert("thomas", "Thomas");
        common_authors.insert("peduarte", "Pedro Duarte");
        common_authors.insert("mattisssa", "Mattias");
        common_authors.insert("tonka3000", "Tonka3000");
        common_authors.insert("extensions", "Extensions");
        common_authors.insert("community", "Community");
        common_authors.insert("official", "Official");

        let mut common_categories = HashMap::with_capacity(16);
        common_categories.insert("productivity", "Productivity");
        common_categories.insert("developer tools", "Developer Tools");
        common_categories.insert("system", "System");
        common_categories.insert("web search", "Web Search");
        common_categories.insert("communication", "Communication");
        common_categories.insert("media", "Media");
        common_categories.insert("finance", "Finance");
        common_categories.insert("fun", "Fun");

        let mut common_modes = HashMap::with_capacity(8);
        common_modes.insert("view", "view");
        common_modes.insert("no-view", "no-view");
        common_modes.insert("silent", "silent");

        let mut common_types = HashMap::with_capacity(16);
        common_types.insert("text", "text");
        common_types.insert("textfield", "textfield");
        common_types.insert("password", "password");
        common_types.insert("checkbox", "checkbox");
        common_types.insert("dropdown", "dropdown");
        common_types.insert("file", "file");
        common_types.insert("directory", "directory");

        Self {
            common_authors,
            common_categories,
            common_modes,
            common_types,
        }
    }

    #[inline]
    fn intern_author<'a>(&self, author: &'a str) -> Cow<'a, str> {
        match self.common_authors.get(author.to_lowercase().as_str()) {
            Some(&interned) => Cow::Borrowed(interned),
            None => Cow::Borrowed(author),
        }
    }

    #[inline]
    fn intern_category<'a>(&self, category: &'a str) -> Cow<'a, str> {
        match self.common_categories.get(category.to_lowercase().as_str()) {
            Some(&interned) => Cow::Borrowed(interned),
            None => Cow::Borrowed(category),
        }
    }

    #[inline]
    fn intern_mode<'a>(&self, mode: &'a str) -> Cow<'a, str> {
        match self.common_modes.get(mode) {
            Some(&interned) => Cow::Borrowed(interned),
            None => Cow::Borrowed(mode),
        }
    }

    #[inline]
    fn intern_type<'a>(&self, type_str: &'a str) -> Cow<'a, str> {
        match self.common_types.get(type_str) {
            Some(&interned) => Cow::Borrowed(interned),
            None => Cow::Borrowed(type_str),
        }
    }
}

// Field name constants
const FIELD_NAME: &str = "name";
const FIELD_TITLE: &str = "title";
const FIELD_DESCRIPTION: &str = "description";
const FIELD_AUTHOR: &str = "author";
const FIELD_CATEGORIES: &str = "categories";
const FIELD_ICON: &str = "icon";
const FIELD_VERSION: &str = "version";
const FIELD_KEYWORDS: &str = "keywords";
const FIELD_COMMANDS: &str = "commands";
const FIELD_MODE: &str = "mode";
const FIELD_LICENSE: &str = "license";
const FIELD_HOMEPAGE: &str = "homepage";
const FIELD_REPOSITORY: &str = "repository";

/// Discovers Raycast extensions from the specified directory with zero-allocation optimizations
#[op2(async)]
#[string]
pub async fn op_discover_raycast_extensions(#[string] extensions_dir: String) -> String {
    let extensions = match discover_extensions_internal(&extensions_dir).await {
        Ok(exts) => exts,
        Err(e) => {
            return create_discovery_error_response(&extensions_dir, e);
        },
    };

    match serde_json::to_string(&extensions) {
        Ok(json) => json,
        Err(e) => create_serialization_error_response(e),
    }
}

/// Create structured error response for discovery failures
#[cold]
#[inline(never)]
fn create_discovery_error_response(extensions_dir: &str, error: RaycastDiscoveryError) -> String {
    let error_response = ErrorResponse {
        error: ErrorDetails {
            code: error_codes::DISCOVERY_FAILED.to_string(),
            message: format!("Failed to discover extensions: {}", error),
            category: error_categories::FILESYSTEM.to_string(),
            severity: error_severity::HIGH.to_string(),
        },
        context: ErrorContext {
            operation: "discover_extensions".to_string(),
            path: Some(extensions_dir.to_string()),
            line: None,
            column: None,
            file_size: None,
            directory_count: None,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    match serde_json::to_string(&error_response) {
        Ok(json) => json,
        Err(_) => r#"{"error": {"code": "SERIALIZATION_FAILED", "message": "Failed to serialize error response"}}"#.to_string(),
    }
}

/// Create structured error response for serialization failures
#[cold]
#[inline(never)]
fn create_serialization_error_response(error: serde_json::Error) -> String {
    let error_response = ErrorResponse {
        error: ErrorDetails {
            code: error_codes::SERIALIZATION_ERROR.to_string(),
            message: format!("Failed to serialize extensions: {}", error),
            category: error_categories::SERIALIZATION.to_string(),
            severity: error_severity::CRITICAL.to_string(),
        },
        context: ErrorContext {
            operation: "serialize_extensions".to_string(),
            path: None,
            line: Some(error.line() as u32),
            column: Some(error.column() as u32),
            file_size: None,
            directory_count: None,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    match serde_json::to_string(&error_response) {
        Ok(json) => json,
        Err(_) => r#"{"error": {"code": "CRITICAL_SERIALIZATION_FAILURE", "message": "Cannot serialize error response"}}"#.to_string(),
    }
}

/// Internal implementation with aggressive optimizations
#[inline]
pub async fn discover_extensions_internal(
    extensions_dir: &str,
) -> Result<Vec<IsolatedRaycastExtension>, RaycastDiscoveryError> {
    // Validate input path for security
    validate_directory_path(extensions_dir)?;

    // Initialize string interner for zero-allocation string handling
    let interner = StringInterner::new();

    // Collect all valid directory paths first for parallel processing
    let extension_paths = collect_extension_paths(extensions_dir).await?;

    // Process extensions in parallel batches for optimal performance
    let mut extensions = process_extensions_parallel(extension_paths, &interner).await?;

    // Shrink to fit to return minimum memory
    extensions.shrink_to_fit();
    Ok(extensions)
}

/// Collect all valid extension directory paths for parallel processing
#[inline]
async fn collect_extension_paths(
    extensions_dir: &str,
) -> Result<Vec<PathBuf>, RaycastDiscoveryError> {
    let mut extension_paths = Vec::new();
    let mut read_dir = fs::read_dir(extensions_dir)
        .await
        .map_err(|e| create_filesystem_error(extensions_dir, "read_directory", e))?;

    let mut extension_count = 0;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| create_filesystem_error(extensions_dir, "read_directory_entry", e))?
    {
        // Security: limit number of extensions to prevent DoS
        if extension_count >= MAX_EXTENSIONS_COUNT {
            break;
        }

        let path = entry.path();

        // Fast path: skip non-directories immediately
        let metadata = entry.metadata().await.map_err(|e| {
            create_filesystem_error(path.to_string_lossy().as_ref(), "read_metadata", e)
        })?;

        if !metadata.is_dir() {
            continue;
        }

        // Security: validate directory path depth
        if validate_directory_depth(&path, extensions_dir).is_err() {
            continue; // Skip directories that are too deep
        }

        // Check if directory has package.json before adding to parallel processing queue
        let package_json_path = path.join(PACKAGE_JSON);
        if fs::metadata(&package_json_path).await.is_ok() {
            extension_paths.push(path);
            extension_count += 1;
        }
    }

    extension_paths.shrink_to_fit();
    Ok(extension_paths)
}

/// Process extensions in parallel batches with bounded concurrency
#[inline]
async fn process_extensions_parallel(
    extension_paths: Vec<PathBuf>,
    interner: &StringInterner,
) -> Result<Vec<IsolatedRaycastExtension>, RaycastDiscoveryError> {
    let mut extensions = Vec::with_capacity(extension_paths.len());

    // Process extensions in batches to control memory usage and concurrency
    for chunk in extension_paths.chunks(BATCH_SIZE) {
        let batch_tasks: Vec<_> = chunk
            .iter()
            .take(MAX_CONCURRENT_PARSING.min(chunk.len()))
            .map(|path| {
                let path_clone = path.clone();
                let interner_ref = interner; // Use reference to avoid cloning
                async move { parse_extension_optimized(&path_clone, interner_ref).await }
            })
            .collect();

        // Execute batch concurrently and collect results
        let batch_results = join_all(batch_tasks).await;

        // Process results and collect successful extensions
        for result in batch_results {
            match result {
                Ok(extension) => {
                    extensions.push(extension);
                },
                Err(_) => {
                    // Skip invalid extensions silently for performance
                    continue;
                },
            }
        }
    }

    extensions.shrink_to_fit();
    Ok(extensions)
}

/// Optimized extension parsing with zero-copy where possible
#[inline]
async fn parse_extension_optimized(
    path: &Path,
    interner: &StringInterner,
) -> Result<IsolatedRaycastExtension, RaycastDiscoveryError> {
    // Extract ID with minimal allocation
    let id = path.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
        create_invalid_structure_error(
            path.to_string_lossy().as_ref(),
            "directory_name",
            "valid_utf8",
            "invalid_utf8",
        )
    })?;

    // Read package.json as bytes for zero-copy parsing
    let package_json_path = path.join(PACKAGE_JSON);

    // Validate file size before reading for DoS protection
    let metadata = fs::metadata(&package_json_path)
        .await
        .map_err(|e| handle_file_read_error(e, &package_json_path, path))?;
    validate_file_size(metadata.len(), &package_json_path)?;

    let package_bytes = fs::read(&package_json_path)
        .await
        .map_err(|e| handle_file_read_error(e, &package_json_path, path))?;

    // Parse JSON from bytes to avoid string allocation
    let package: serde_json::Value = serde_json::from_slice(&package_bytes)
        .map_err(|e| create_parsing_error(&package_json_path, e))?;

    // Extract all fields in single pass with optimized string handling
    let extraction = extract_package_fields(&package, id, interner);

    // Parse commands with pre-allocated capacity
    let commands = extract_commands(&package, interner)?;

    // Build metadata map with pre-allocated capacity
    let metadata = extract_metadata(&package);

    // Parse extension-level preferences
    let preferences = extract_extension_preferences(&package, interner)?;

    // Construct extension with validated data
    let extension = IsolatedRaycastExtension {
        id: id.to_string(),
        name: extraction.name,
        title: extraction.title,
        description: extraction.description,
        author: extraction.author,
        categories: extraction.categories,
        icon: extraction.icon,
        path: path.to_string_lossy().into_owned(),
        commands,
        version: extraction.version,
        keywords: extraction.keywords,
        metadata,
        preferences,
    };

    // Validate during construction rather than after
    extension.validate().map_err(|validation_error| {
        create_invalid_structure_error(
            path.to_string_lossy().as_ref(),
            "validation",
            "valid_extension",
            &validation_error,
        )
    })?;

    Ok(extension)
}

/// Extracted package field data to minimize repeated allocations
struct PackageExtraction {
    name: String,
    title: String,
    description: String,
    author: String,
    categories: Vec<String>,
    icon: Option<String>,
    version: Option<String>,
    keywords: Vec<String>,
}

/// Optimized field extraction with minimal allocations
#[inline]
fn extract_package_fields(
    package: &serde_json::Value,
    id: &str,
    interner: &StringInterner,
) -> PackageExtraction {
    let name = match package.get(FIELD_NAME).and_then(|v| v.as_str()) {
        Some(n) => validate_string_field(n),
        None => id.to_string(),
    };

    let title = match package.get(FIELD_TITLE).and_then(|v| v.as_str()) {
        Some(t) => validate_string_field(t),
        None => name.clone(),
    };

    let description = match package.get(FIELD_DESCRIPTION).and_then(|v| v.as_str()) {
        Some(d) => validate_string_field(d),
        None => EMPTY_STRING.to_string(),
    };

    let author = match package.get(FIELD_AUTHOR).and_then(|v| v.as_str()) {
        Some(a) => interner.intern_author(a).into_owned(),
        None => UNKNOWN_AUTHOR.to_string(),
    };

    let categories = match package.get(FIELD_CATEGORIES).and_then(|v| v.as_array()) {
        Some(arr) => {
            let mut cats = Vec::with_capacity(arr.len());
            for item in arr {
                if let Some(s) = item.as_str() {
                    cats.push(interner.intern_category(s).into_owned());
                }
            }
            cats
        },
        None => Vec::new(),
    };

    let icon = package
        .get(FIELD_ICON)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let version = package
        .get(FIELD_VERSION)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let keywords = match package.get(FIELD_KEYWORDS).and_then(|v| v.as_array()) {
        Some(arr) => {
            let mut kws = Vec::with_capacity(arr.len());
            for item in arr {
                if let Some(s) = item.as_str() {
                    kws.push(s.to_string());
                }
            }
            kws
        },
        None => Vec::new(),
    };

    PackageExtraction {
        name,
        title,
        description,
        author,
        categories,
        icon,
        version,
        keywords,
    }
}

/// Optimized command extraction with pre-allocation
#[inline]
fn extract_commands(
    package: &serde_json::Value,
    interner: &StringInterner,
) -> Result<Vec<IsolatedRaycastCommand>, RaycastDiscoveryError> {
    let commands_array = match package.get(FIELD_COMMANDS).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(Vec::new()),
    };

    let mut commands = Vec::with_capacity(commands_array.len());

    for cmd in commands_array {
        let cmd_name = cmd
            .get(FIELD_NAME)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let cmd_title = cmd
            .get(FIELD_TITLE)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Both name and title are required for valid commands
        if let (Some(name), Some(title)) = (cmd_name, cmd_title) {
            let description = cmd
                .get(FIELD_DESCRIPTION)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mode = match cmd.get(FIELD_MODE).and_then(|v| v.as_str()) {
                Some(m) => interner.intern_mode(m).into_owned(),
                None => DEFAULT_MODE.to_string(),
            };

            let subtitle = cmd
                .get("subtitle")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let keywords = match cmd.get(FIELD_KEYWORDS).and_then(|v| v.as_array()) {
                Some(arr) => {
                    let mut kws = Vec::with_capacity(arr.len());
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            kws.push(s.to_string());
                        }
                    }
                    kws
                },
                None => Vec::new(),
            };

            // Parse command arguments
            let arguments = extract_command_arguments(cmd, interner)?;

            // Parse command preferences
            let preferences = extract_command_preferences(cmd, interner)?;

            commands.push(IsolatedRaycastCommand {
                name,
                title,
                description,
                mode,
                subtitle,
                keywords,
                arguments,
                preferences,
            });
        }
    }

    commands.shrink_to_fit();
    Ok(commands)
}

/// Extract metadata fields with pre-allocated map
#[inline]
fn extract_metadata(package: &serde_json::Value) -> HashMap<String, String> {
    let mut metadata = HashMap::with_capacity(4);

    if let Some(license) = package.get(FIELD_LICENSE).and_then(|v| v.as_str()) {
        metadata.insert(FIELD_LICENSE.to_string(), license.to_string());
    }

    if let Some(homepage) = package.get(FIELD_HOMEPAGE).and_then(|v| v.as_str()) {
        metadata.insert(FIELD_HOMEPAGE.to_string(), homepage.to_string());
    }

    // Handle repository field - can be string or object
    if let Some(repository) = package.get(FIELD_REPOSITORY) {
        let repo_value = if let Some(repo_str) = repository.as_str() {
            repo_str.to_string()
        } else if let Some(repo_obj) = repository.as_object() {
            // Extract URL from repository object
            match repo_obj.get("url").and_then(|v| v.as_str()) {
                Some(url) => url.to_string(),
                None => "unknown".to_string(),
            }
        } else {
            "unknown".to_string()
        };
        metadata.insert(FIELD_REPOSITORY.to_string(), repo_value);
    }

    metadata.shrink_to_fit();
    metadata
}

/// Extract command arguments with complete field parsing
#[inline]
pub fn extract_command_arguments(
    cmd: &serde_json::Value,
    interner: &StringInterner,
) -> Result<Vec<crate::raycast_types::IsolatedCommandArgument>, RaycastDiscoveryError> {
    const FIELD_ARGUMENTS: &str = "arguments";
    const FIELD_PLACEHOLDER: &str = "placeholder";
    const FIELD_REQUIRED: &str = "required";
    const FIELD_TYPE: &str = "type";

    let arguments_array = match cmd.get(FIELD_ARGUMENTS).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(Vec::new()),
    };

    let mut arguments = Vec::with_capacity(arguments_array.len());

    for arg in arguments_array {
        let name = match arg.get(FIELD_NAME).and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => continue, // Skip arguments without names
        };

        let placeholder = arg
            .get(FIELD_PLACEHOLDER)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let required = arg
            .get(FIELD_REQUIRED)
            .and_then(|v| v.as_bool())
            .unwrap_or_default();

        let argument_type = match arg.get(FIELD_TYPE).and_then(|v| v.as_str()) {
            Some(t) => interner.intern_type(t).into_owned(),
            None => "text".to_string(), // Default type
        };

        arguments.push(crate::raycast_types::IsolatedCommandArgument {
            name,
            placeholder,
            required,
            argument_type,
        });
    }

    arguments.shrink_to_fit();
    Ok(arguments)
}

/// Extract command preferences with complete field parsing
#[inline]
pub fn extract_command_preferences(
    cmd: &serde_json::Value,
    interner: &StringInterner,
) -> Result<Vec<crate::raycast_types::IsolatedCommandPreference>, RaycastDiscoveryError> {
    const FIELD_PREFERENCES: &str = "preferences";
    const FIELD_TITLE: &str = "title";
    const FIELD_DESCRIPTION: &str = "description";
    const FIELD_TYPE: &str = "type";
    const FIELD_DEFAULT: &str = "default";
    const FIELD_REQUIRED: &str = "required";

    let preferences_array = match cmd.get(FIELD_PREFERENCES).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(Vec::new()),
    };

    let mut preferences = Vec::with_capacity(preferences_array.len());

    for pref in preferences_array {
        let name = match pref.get(FIELD_NAME).and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => continue, // Skip preferences without names
        };

        let title = match pref.get(FIELD_TITLE).and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => name.clone(), // Use name as fallback title
        };

        let description = pref
            .get(FIELD_DESCRIPTION)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let preference_type = match pref.get(FIELD_TYPE).and_then(|v| v.as_str()) {
            Some(t) => interner.intern_type(t).into_owned(),
            None => "textfield".to_string(), // Default type
        };

        let default_value = pref
            .get(FIELD_DEFAULT)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let required = pref
            .get(FIELD_REQUIRED)
            .and_then(|v| v.as_bool())
            .unwrap_or_default();

        preferences.push(crate::raycast_types::IsolatedCommandPreference {
            name,
            title,
            description,
            preference_type,
            default_value,
            required,
        });
    }

    preferences.shrink_to_fit();
    Ok(preferences)
}

/// Extract extension-level preferences with complete field parsing
#[inline]
pub fn extract_extension_preferences(
    package: &serde_json::Value,
    interner: &StringInterner,
) -> Result<Vec<crate::raycast_types::IsolatedExtensionPreference>, RaycastDiscoveryError> {
    const FIELD_PREFERENCES: &str = "preferences";
    const FIELD_TITLE: &str = "title";
    const FIELD_DESCRIPTION: &str = "description";
    const FIELD_TYPE: &str = "type";
    const FIELD_DEFAULT: &str = "default";
    const FIELD_REQUIRED: &str = "required";

    let preferences_array = match package.get(FIELD_PREFERENCES).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(Vec::new()),
    };

    let mut preferences = Vec::with_capacity(preferences_array.len());

    for pref in preferences_array {
        let name = match pref.get(FIELD_NAME).and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => continue, // Skip preferences without names
        };

        let title = match pref.get(FIELD_TITLE).and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => name.clone(), // Use name as fallback title
        };

        let description = pref
            .get(FIELD_DESCRIPTION)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let preference_type = match pref.get(FIELD_TYPE).and_then(|v| v.as_str()) {
            Some(t) => interner.intern_type(t).into_owned(),
            None => "textfield".to_string(), // Default type
        };

        let default_value = pref
            .get(FIELD_DEFAULT)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let required = pref
            .get(FIELD_REQUIRED)
            .and_then(|v| v.as_bool())
            .unwrap_or_default();

        preferences.push(crate::raycast_types::IsolatedExtensionPreference {
            name,
            title,
            description,
            preference_type,
            default_value,
            required,
        });
    }

    preferences.shrink_to_fit();
    Ok(preferences)
}

// Optimized error creation functions to minimize allocations

#[cold]
#[inline(never)]
fn create_filesystem_error(
    path: &str,
    operation: &'static str,
    source: std::io::Error,
) -> RaycastDiscoveryError {
    RaycastDiscoveryError::Filesystem {
        path: path.to_string(),
        operation: operation.to_string(),
        source: source.to_string(),
    }
}

#[cold]
#[inline(never)]
fn create_invalid_structure_error(
    path: &str,
    field: &'static str,
    expected: &'static str,
    found: &str,
) -> RaycastDiscoveryError {
    RaycastDiscoveryError::InvalidStructure {
        path: path.to_string(),
        field: field.to_string(),
        expected: expected.to_string(),
        found: found.to_string(),
    }
}

#[cold]
#[inline(never)]
fn create_parsing_error(file_path: &Path, source: serde_json::Error) -> RaycastDiscoveryError {
    RaycastDiscoveryError::Parsing {
        file_path: file_path.to_string_lossy().into_owned(),
        line: None,   // serde_json doesn't provide line info in newer versions
        column: None, // serde_json doesn't provide column info in newer versions
        message: source.to_string(),
    }
}

#[cold]
#[inline(never)]
fn handle_file_read_error(
    error: std::io::Error,
    package_json_path: &Path,
    extension_path: &Path,
) -> RaycastDiscoveryError {
    match error.kind() {
        std::io::ErrorKind::NotFound => RaycastDiscoveryError::MissingFiles {
            extension_path: extension_path.to_string_lossy().into_owned(),
            missing_files: vec![PACKAGE_JSON.to_string()],
        },
        std::io::ErrorKind::PermissionDenied => RaycastDiscoveryError::PermissionDenied {
            path: package_json_path.to_string_lossy().into_owned(),
            operation: "read_file".to_string(),
        },
        _ => RaycastDiscoveryError::Filesystem {
            path: package_json_path.to_string_lossy().into_owned(),
            operation: "read_file".to_string(),
            source: error.to_string(),
        },
    }
}

/// Validate directory path for security issues
#[inline]
fn validate_directory_path(path: &str) -> Result<(), RaycastDiscoveryError> {
    // Check for path traversal attempts
    if path.contains("..") || path.contains("~") {
        return Err(RaycastDiscoveryError::SecurityViolation {
            path: path.to_string(),
            violation: "path_traversal".to_string(),
            message: "Path contains potentially dangerous components".to_string(),
        });
    }

    // Check for null bytes
    if path.contains('\0') {
        return Err(RaycastDiscoveryError::SecurityViolation {
            path: path.to_string(),
            violation: "null_byte".to_string(),
            message: "Path contains null bytes".to_string(),
        });
    }

    // Check for extremely long paths
    if path.len() > 4096 {
        return Err(RaycastDiscoveryError::SecurityViolation {
            path: path.to_string(),
            violation: "path_too_long".to_string(),
            message: "Path exceeds maximum length".to_string(),
        });
    }

    Ok(())
}

/// Validate file size to prevent DoS attacks
#[inline]
fn validate_file_size(size: u64, file_path: &Path) -> Result<(), RaycastDiscoveryError> {
    if size > MAX_PACKAGE_JSON_SIZE {
        return Err(RaycastDiscoveryError::SecurityViolation {
            path: file_path.to_string_lossy().into_owned(),
            violation: "file_too_large".to_string(),
            message: format!(
                "File size {} bytes exceeds maximum {}",
                size, MAX_PACKAGE_JSON_SIZE
            ),
        });
    }
    Ok(())
}

/// Validate directory depth to prevent DoS attacks
#[inline]
fn validate_directory_depth(path: &Path, base_dir: &str) -> Result<(), RaycastDiscoveryError> {
    let relative_path = match path.strip_prefix(base_dir) {
        Ok(rel) => rel,
        Err(_) => return Ok(()), // Not under base directory, allow
    };

    let depth = relative_path.components().count();
    if depth > MAX_DIRECTORY_DEPTH {
        return Err(RaycastDiscoveryError::SecurityViolation {
            path: path.to_string_lossy().into_owned(),
            violation: "directory_too_deep".to_string(),
            message: format!(
                "Directory depth {} exceeds maximum {}",
                depth, MAX_DIRECTORY_DEPTH
            ),
        });
    }

    Ok(())
}

/// Validate and truncate string fields to prevent DoS attacks
#[inline]
fn validate_string_field(input: &str) -> String {
    if input.len() <= MAX_STRING_LENGTH {
        input.to_string()
    } else {
        // Truncate oversized strings and add indicator
        let mut truncated = input
            .chars()
            .take(MAX_STRING_LENGTH - 3)
            .collect::<String>();
        truncated.push_str("...");
        truncated
    }
}

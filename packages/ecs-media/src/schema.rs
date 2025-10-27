//! SurrealDB schema for media metadata storage
//!
//! Defines media table with SCHEMAFULL enforcement for data integrity

/// Media schema definition using SurrealQL (not SQL!)
pub const MEDIA_SCHEMA: &str = r#"
-- ============================================================================
-- MEDIA TABLE
-- ============================================================================
DEFINE TABLE media SCHEMAFULL;

-- Core identification fields
DEFINE FIELD user_id ON media TYPE string
    ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD conversation_id ON media TYPE option<string>;

-- File metadata
DEFINE FIELD filename ON media TYPE string
    ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD mime_type ON media TYPE string;
DEFINE FIELD size_bytes ON media TYPE number
    ASSERT $value > 0;

-- Media-specific dimensions (optional)
DEFINE FIELD width ON media TYPE option<number>;
DEFINE FIELD height ON media TYPE option<number>;
DEFINE FIELD duration_seconds ON media TYPE option<number>;

-- User-provided metadata
DEFINE FIELD description ON media TYPE option<string>;

-- Storage references
DEFINE FIELD storage_path ON media TYPE string
    ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD thumbnail_path ON media TYPE option<string>;

-- Automatic timestamps
DEFINE FIELD created_at ON media TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON media TYPE datetime DEFAULT time::now();

-- Performance indexes
DEFINE INDEX user_idx ON media COLUMNS user_id;
DEFINE INDEX conversation_idx ON media COLUMNS conversation_id;
DEFINE INDEX created_idx ON media COLUMNS created_at;
"#;

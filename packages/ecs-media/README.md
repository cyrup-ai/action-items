# ECS Media Package

Production-ready media upload and management system for Action Items.

## Overview

Provides foundational infrastructure for media upload support with SurrealDB schema and file storage integration. This is **Part A** of the media upload implementation.

## Features

- ✅ SurrealDB-based metadata storage with SCHEMAFULL schema
- ✅ Type-safe media operations with MediaId, Media, and MediaError types
- ✅ Event-driven architecture with MediaRequest/MediaResponse
- ✅ Async ECS-based operations using Bevy AsyncComputeTaskPool
- ✅ CRUD operations: upload, retrieve, update description, delete
- ✅ Integration with ecs-surrealdb for database operations
- ✅ Production-grade error handling (no unwrap/expect calls)

## Architecture

Follows the established ECS plugin pattern used throughout the Action Items codebase:

- **MediaConfig**: Configuration resource for storage paths and limits
- **MediaService**: Service resource wrapping MediaManager
- **MediaRequest/MediaResponse**: Event-driven request/response pattern
- **MediaManager**: Core business logic for CRUD operations
- **MediaPlugin**: Bevy plugin integration with startup and update systems

## Usage

```rust
use action_items_ecs_media::{MediaPlugin, MediaConfig};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(MediaPlugin::default())
        .run();
}
```

## Implementation Status

### ✅ Completed (Part A)

- Database schema with SurrealQL
- Media types and error handling
- Event definitions
- MediaManager CRUD operations
- MediaPlugin with ECS integration
- Workspace integration and compilation

### 🚧 Future Work

- **Part B**: Upload validation, MIME type detection, thumbnail generation
- **Part C**: UI integration and progress indicators

## Database Schema

Uses SurrealDB SCHEMAFULL enforcement with:

- Core fields: user_id, conversation_id, filename, mime_type, size_bytes
- Media dimensions: width, height, duration_seconds (optional)
- Storage references: storage_path, thumbnail_path
- Automatic timestamps: created_at, updated_at
- Performance indexes on user_id, conversation_id, created_at

## Dependencies

- `bevy`: ECS framework
- `surrealdb`: Database operations
- `tokio`: Async runtime
- `uuid`: Unique identifiers
- `chrono`: Timestamp handling
- `mime_guess`: MIME type detection
- `action_items_ecs_surrealdb`: Database service integration

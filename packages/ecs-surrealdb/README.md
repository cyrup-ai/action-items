# ECS SurrealDB Package

✅ **COMPLETED** - Extracted from core and fully implemented

## Overview
Complete, production-ready SurrealDB integration with Bevy ECS patterns.

## Features

- ✅ Complete SurrealDB v3.0 integration with LazyLock singleton pattern
- ✅ Transaction support with automatic commit/rollback
- ✅ Full CRUD operations (create, select, update, delete)  
- ✅ Health checks and graceful shutdown
- ✅ Bevy ECS integration with DatabasePlugin
- ✅ Configuration validation and security features
- ✅ Connection retry logic and timeout handling
- ✅ Table name validation and query parameter binding

## Usage

```rust
use action_items_ecs_surrealdb::{DatabasePlugin, DatabaseConfig, DatabaseService};
use bevy::prelude::*;

fn main() {
    let config = DatabaseConfig::default();
    
    App::new()
        .add_plugins(DatabasePlugin::new(config))
        .run();
}
```

## Architecture

This package provides a complete, production-ready SurrealDB integration that was extracted from the core package to maintain clean separation of concerns and follow the established ECS service pattern used throughout the Action Items codebase.

### Resources
- `DatabaseService` - Main database service with async operations
- `DatabaseConfig` - Configuration for connection, timeouts, and security
- `DatabaseServiceError` - Resource indicating service unavailability
- `DatabaseShutdown` - Resource for tracking shutdown state

### Transaction Support
- Automatic commit/rollback with panic safety
- `TransactionContext` for scoped operations
- Timeout handling and connection management

### Security
- Path traversal attack prevention
- Input validation for table names and identifiers
- Secure storage path validation
# SurrealDB Database Service Usage Guide

## Overview

The SurrealDB database service provides a production-quality, async database interface for the Action Items application. It includes connection pooling, query building, metrics tracking, and Bevy ECS integration.

## Features

- **Production-ready**: Full error handling, connection pooling, and metrics
- **SurrealDB 2.3.7**: Current stable version with upgrade path to v3.0.0 planned
- **Async/await**: Non-blocking database operations using tokio
- **Type-safe**: Generic CRUD operations with serde serialization
- **Query builder**: Fluent API for building SQL queries
- **Metrics**: Built-in performance and usage tracking
- **Bevy integration**: Plugin system compatible with ECS architecture

## Configuration

```rust
use action_items_core::plugins::services::{DatabaseConfig, DatabaseService};

let config = DatabaseConfig {
    namespace: "action_items".to_string(),
    database: "main".to_string(),
    storage_path: PathBuf::from("./data/action_items.db"),
    connection_timeout_ms: 5000,
    query_timeout_ms: 10000,
    max_connections: 10,
    enable_query_logging: false,
};

let service = DatabaseService::new(config).await?;
```

## Basic Usage

### Creating Records

```rust
#[derive(Serialize, Deserialize)]
struct Plugin {
    name: String,
    version: String,
    enabled: bool,
}

let plugin = Plugin {
    name: "example-plugin".to_string(),
    version: "1.0.0".to_string(),
    enabled: true,
};

let plugin_id = service.create("plugins", &plugin).await?;
```

### Reading Records

```rust
// Select all records
let plugins: Vec<Plugin> = service.select("plugins").await?;

// Select with query
let active_plugins = service.query("SELECT * FROM plugins WHERE enabled = true").await?;
```

### Updating Records

```rust
let updated_plugin = Plugin {
    name: "example-plugin".to_string(),
    version: "1.1.0".to_string(),
    enabled: true,
};

service.update(&plugin_id, &updated_plugin).await?;
```

### Deleting Records

```rust
service.delete(&plugin_id).await?;
```

## Query Builder

The query builder provides a fluent API for constructing SQL queries:

```rust
use action_items_core::plugins::services::QueryBuilder;

// SELECT query
let query = QueryBuilder::new("plugins")
    .select(&["name", "version", "enabled"])
    .where_clause("enabled = true")
    .order_by("name", false) // false = ASC, true = DESC
    .limit(10)
    .build_select();

// UPDATE query
let mut update_data = HashMap::new();
update_data.insert("enabled".to_string(), Value::Bool(false));

let update_query = QueryBuilder::new("plugins")
    .where_clause("version < '1.0.0'")
    .build_update(&update_data);

// DELETE query
let delete_query = QueryBuilder::new("plugins")
    .where_clause("enabled = false")
    .build_delete();
```

## Parameterized Queries

For security and performance, use parameterized queries:

```rust
let mut params = HashMap::new();
params.insert("min_version".to_string(), Value::Strand("1.0.0".into()));
params.insert("enabled".to_string(), Value::Bool(true));

let query = "SELECT * FROM plugins WHERE version >= $min_version AND enabled = $enabled";
let results = service.query_with_params(query, params).await?;
```

## Schema Management

Define schemas for data validation:

```rust
let schema = r#"
    DEFINE TABLE plugins SCHEMAFULL;
    DEFINE FIELD name ON TABLE plugins TYPE string;
    DEFINE FIELD version ON TABLE plugins TYPE string;
    DEFINE FIELD enabled ON TABLE plugins TYPE bool;
    DEFINE FIELD config ON TABLE plugins TYPE object;
    DEFINE INDEX name_idx ON TABLE plugins COLUMNS name UNIQUE;
"#;

service.query(schema).await?;
```

## Transactions

Use transactions for atomic operations:

```rust
let transaction = vec![
    "BEGIN TRANSACTION;".to_string(),
    "UPDATE plugins SET enabled = false WHERE version < '1.0.0';".to_string(),
    "DELETE FROM plugins WHERE enabled = false;".to_string(),
    "COMMIT TRANSACTION;".to_string(),
];

service.transaction(transaction).await?;
```

## Health Monitoring

Check database health and connection status:

```rust
let health = service.health_check().await?;
println!("Status: {}", health.status);
println!("Available connections: {}", health.pool_stats.available_connections);
println!("Total queries: {}", health.metrics.total_queries);
```

## Metrics

Access performance metrics:

```rust
let metrics = service.get_metrics();
println!("Total queries: {}", metrics.total_queries);
println!("Successful queries: {}", metrics.successful_queries);
println!("Failed queries: {}", metrics.failed_queries);
println!("Average query time: {:.2}ms", metrics.average_query_time_ms);
```

## Bevy Integration

The database service integrates with Bevy's ECS system:

```rust
use bevy::prelude::*;
use action_items_core::plugins::services::DatabasePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DatabasePlugin::default())
        .add_systems(Startup, setup_database)
        .run();
}

fn setup_database(mut commands: Commands, database: Res<DatabaseService>) {
    // Use database service in Bevy systems
}
```

## Error Handling

The service provides comprehensive error types:

```rust
use action_items_core::plugins::services::DatabaseError;

match service.create("plugins", &plugin).await {
    Ok(id) => println!("Created plugin with ID: {}", id),
    Err(DatabaseError::Connection(e)) => eprintln!("Connection error: {}", e),
    Err(DatabaseError::Query(e)) => eprintln!("Query error: {}", e),
    Err(DatabaseError::Serialization(e)) => eprintln!("Serialization error: {}", e),
    Err(DatabaseError::Timeout(e)) => eprintln!("Timeout error: {}", e),
}
```

## Best Practices

### 1. Use Structured Data Types

Always define proper structs with serde derives:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ActionItem {
    id: Option<String>,
    title: String,
    description: Option<String>,
    completed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

### 2. Handle Errors Gracefully

```rust
async fn create_action_item(service: &DatabaseService, item: &ActionItem) -> Result<String, Box<dyn std::error::Error>> {
    let id = service.create("action_items", item).await
        .map_err(|e| format!("Failed to create action item: {}", e))?;
    Ok(id)
}
```

### 3. Use Connection Pooling

The service automatically manages connections, but configure pool size based on your needs:

```rust
let config = DatabaseConfig {
    max_connections: 20, // Adjust based on concurrent load
    connection_timeout_ms: 5000,
    query_timeout_ms: 30000, // Longer for complex queries
    ..Default::default()
};
```

### 4. Monitor Performance

Regularly check metrics and health:

```rust
async fn monitor_database(service: &DatabaseService) {
    let health = service.health_check().await.unwrap();
    if health.pool_stats.available_connections < 2 {
        warn!("Low database connection availability");
    }
    
    let metrics = service.get_metrics();
    if metrics.failed_queries > metrics.successful_queries * 0.1 {
        warn!("High database error rate: {}%", 
               (metrics.failed_queries as f64 / metrics.total_queries as f64) * 100.0);
    }
}
```

### 5. Use Indexes for Performance

```rust
let indexes = r#"
    DEFINE INDEX title_idx ON TABLE action_items COLUMNS title;
    DEFINE INDEX completed_idx ON TABLE action_items COLUMNS completed;
    DEFINE INDEX created_at_idx ON TABLE action_items COLUMNS created_at;
"#;
service.query(indexes).await?;
```

## Migration to SurrealDB v3.0.0

The service is designed with v3.0.0 migration in mind:

1. **Current State**: Using SurrealDB 2.3.7 with `kv-surrealkv` storage
2. **Migration Path**: Update dependency, test compatibility, migrate schemas
3. **Breaking Changes**: API updates, query syntax changes, new features
4. **Timeline**: Planned for future release once v3.0.0 is stable

## Troubleshooting

### Connection Issues

```rust
// Check if database file is accessible
if !config.storage_path.exists() {
    std::fs::create_dir_all(config.storage_path.parent().unwrap())?;
}

// Verify permissions
let metadata = std::fs::metadata(&config.storage_path)?;
if metadata.permissions().readonly() {
    eprintln!("Database file is read-only");
}
```

### Query Performance

```rust
// Enable query logging for debugging
let config = DatabaseConfig {
    enable_query_logging: true,
    ..Default::default()
};

// Use EXPLAIN for query analysis
let explain_query = format!("EXPLAIN {}", your_query);
let plan = service.query(&explain_query).await?;
```

### Memory Usage

```rust
// Monitor connection pool
let health = service.health_check().await?;
println!("Active connections: {}", 
         health.pool_stats.max_connections - health.pool_stats.available_connections);

// Check metrics for resource usage
let metrics = service.get_metrics();
if metrics.average_query_time_ms > 1000.0 {
    warn!("Slow queries detected");
}
```

## Examples

See the test files for comprehensive usage examples:
- `packages/core/src/plugins/services/database_test.rs` - Unit tests
- `packages/core/tests/database_integration.rs` - Integration tests

## Support

For issues or questions:
1. Check the SurrealDB documentation: https://surrealdb.com/docs
2. Review the test files for usage patterns
3. Monitor metrics and health endpoints for performance issues
4. Use query logging for debugging complex queries
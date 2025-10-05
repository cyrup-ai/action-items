# TLS Task Management Usage

This document explains how to use the new Bevy ECS-based TLS cache management instead of `tokio::spawn`.

## Old Pattern (Deprecated)

```rust
// Don't do this - uses tokio::spawn
let tls_manager = TlsManager::new(cert_dir).await?;
tls_manager.start_ocsp_cleanup_task();
tls_manager.start_crl_cleanup_task();
```

## New Pattern (Recommended)

```rust
use bevy::prelude::*;
use action_items_core::tls::{TlsCleanupPlugin, TlsCacheHolder};

// 1. Add the plugin to your Bevy app
app.add_plugins(TlsCleanupPlugin);

// 2. Create TLS manager and spawn cache holder entity
let tls_manager = TlsManager::new(cert_dir).await?;
let cache_holder = tls_manager.create_cache_holder();

// 3. Spawn entity with cache holder component
commands.spawn(cache_holder);
```

## How It Works

1. **TlsCleanupPlugin**: Adds cleanup timer resources and systems to your Bevy app
2. **TlsCacheHolder**: Component that holds references to OCSP and CRL caches  
3. **Cleanup Systems**: Bevy systems that run periodically to clean up caches:
   - `ocsp_cache_cleanup_system`: Runs every hour
   - `crl_cache_cleanup_system`: Runs every 6 hours

## Benefits

- **Proper Bevy Integration**: Uses Bevy's ECS system instead of raw tokio tasks
- **Automatic Lifecycle**: Cache cleanup tied to entity lifecycle
- **Resource Management**: Uses Bevy's resource system for timing
- **No Task Leakage**: No orphaned tokio tasks when systems shut down

## Implementation Details

The cleanup systems use `Instant` and `Duration` to track when to run cleanup operations:

```rust
pub fn ocsp_cache_cleanup_system(
    mut ocsp_timer: ResMut<OcspCleanupTimer>,
    query: Query<&TlsCacheHolder>,
) {
    let now = Instant::now();
    
    if now.duration_since(ocsp_timer.last_cleanup) >= ocsp_timer.interval {
        for cache_holder in query.iter() {
            cache_holder.ocsp_cache.cleanup_cache();
        }
        ocsp_timer.last_cleanup = now;
    }
}
```

This approach integrates seamlessly with Bevy's system scheduling and resource management.
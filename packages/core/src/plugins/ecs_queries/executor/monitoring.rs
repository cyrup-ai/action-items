//! Execution monitoring and metrics

use crate::plugins::ecs_queries::resources::{ActionCache, ActionCacheKey, ActionMetadata};

/// Verify cached capabilities for blazing-fast lookup
#[inline(always)]
pub fn verify_cached_capabilities(
    metadata: &dashmap::mapref::one::Ref<ActionCacheKey, ActionMetadata>,
    action_id: &str,
) -> bool {
    metadata.capabilities.contains(action_id)
}

/// Get cached action metadata with zero allocation using thread-safe cache
#[inline(always)]
pub fn get_cached_metadata<'a>(
    action_cache: &'a ActionCache,
    plugin_id: &str,
    action_id: &str,
) -> Option<dashmap::mapref::one::Ref<'a, ActionCacheKey, ActionMetadata>> {
    action_cache.get_metadata(plugin_id, action_id)
}

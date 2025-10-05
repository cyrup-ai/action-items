pub mod distributed;
pub mod index;
pub mod item;
pub mod platforms;
pub mod systems;

// Re-export public types and functions
pub use distributed::{
    DistributedSearchManager, DistributedSearchQuery, broadcast_capability_updates,
    discover_plugins_via_service_bridge, distributed_search_system,
    handle_search_response_messages, monitor_search_plugin_health,
    process_distributed_search_responses,
};
pub use index::SearchIndex;
pub use item::{SearchItem, SearchItemType};
pub use systems::{
    // ECS-based search systems
    execute_action_item_ecs,
    search_system_ecs,
    setup_search_index,
};

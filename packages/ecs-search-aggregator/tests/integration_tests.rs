use action_items_ecs_search_aggregator::*;
use bevy::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_aggregator_plugin_loads() {
        let mut app = App::new();
        app.add_plugins(SearchAggregatorPlugin);

        // Verify resources are initialized
        assert!(app.world().get_resource::<SearchAggregator>().is_some());
        assert!(
            app.world()
                .get_resource::<AggregatedSearchResults>()
                .is_some()
        );
        assert!(app.world().get_resource::<SearchConfig>().is_some());
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            title: "Test Result".to_string(),
            description: "A test search result".to_string(),
            action: "test_action".to_string(),
            icon: Some("test_icon".to_string()),
            score: 0.9,
            plugin_id: "test_plugin".to_string(),
        };

        assert_eq!(result.title, "Test Result");
        assert_eq!(result.score, 0.9);
        assert_eq!(result.plugin_id, "test_plugin");
    }

    #[test]
    fn test_search_manager_validation() {
        let config = SearchConfig::default();

        // Valid query
        assert!(SearchAggregatorManager::validate_query("test query", &config).is_ok());

        // Empty query
        assert!(SearchAggregatorManager::validate_query("", &config).is_err());

        // Query too short (if min_query_length > 1)
        if config.min_query_length > 1 {
            assert!(SearchAggregatorManager::validate_query("a", &config).is_err());
        }
    }
}

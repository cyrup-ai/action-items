use serde_json::Value;

use crate::plugins::core::ActionItem;

/// Parse search response results from plugin response
pub fn parse_search_response_results(result: &Value) -> Result<Vec<ActionItem>, String> {
    let results_array = result
        .get("results")
        .and_then(|v| v.as_array())
        .ok_or("Missing or invalid 'results' field in response")?;

    let mut action_items = Vec::new();

    for result_item in results_array {
        let title = result_item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();

        let description = result_item
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let action = result_item
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let icon = result_item
            .get("icon")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let score = result_item
            .get("score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        action_items.push(ActionItem {
            title,
            description,
            action,
            icon,
            score,
        });
    }

    Ok(action_items)
}

//! Search Results and Icon Conversion
//!
//! Zero-allocation conversion utilities for transforming Raycast data formats
//! to our internal representation with blazing-fast performance.

use action_items_common::plugin_interface::ActionType;
use serde_json::Value;

use crate::plugins::interface::{ActionItem, Icon, ItemAction};

/// Convert Raycast search results to our format
pub fn convert_search_results(raycast_results: Value) -> Vec<ActionItem> {
    let items = match raycast_results.get("items").and_then(Value::as_array) {
        Some(items) => items,
        None => return Vec::new(),
    };

    items
        .iter()
        .filter_map(|item| {
            let title = item.get("title")?.as_str()?;

            let id = item
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(title)
                .to_string();

            let subtitle = item
                .get("subtitle")
                .and_then(Value::as_str)
                .map(ToString::to_string);

            let icon = convert_icon(item.get("icon"));

            let action = ItemAction {
                id: "execute".to_string(),
                title: "Execute".to_string(),
                icon: None,
                shortcut: None,
                action_type: ActionType::Custom("execute".to_string()),
            };

            let metadata = match item.clone() {
                Value::Object(map) => Some(Value::Object(map)),
                _ => None,
            };

            Some(ActionItem {
                id,
                title: title.to_string(),
                subtitle,
                description: None,
                tags: vec![],
                icon,
                actions: vec![action],
                item_badges: Vec::new(),
                metadata,
                score: 100.0,
                created_at: Some(chrono::Utc::now()),
                updated_at: Some(chrono::Utc::now()),
            })
        })
        .collect()
}

/// Convert Raycast icon to our Icon type
pub fn convert_icon(icon_value: Option<&Value>) -> Option<Icon> {
    icon_value.and_then(|icon| {
        if let Some(icon_str) = icon.as_str() {
            // Check if it's a built-in icon
            Some(Icon::BuiltIn(icon_str.to_string()))
        } else if let Some(icon_obj) = icon.as_object() {
            icon_obj
                .get("source")
                .and_then(|s| s.as_str())
                .map(|source| Icon::BuiltIn(source.to_string()))
        } else {
            None
        }
    })
}
